#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 统一时间模型合同：`UnixTimeNs` / Clock / ErrorKind / ComponentState。

use kernel::{
    ComponentState, ErrorKind, MonotonicClock, SystemMonotonicClock, SystemWallClock, TimeError,
    UnixTimeNs, WallClock, XError,
};
use proptest::prelude::*;
use std::time::Duration;

#[test]
fn system_clocks_now_and_mono_contract() {
    let wall = SystemWallClock::new();
    let mono = SystemMonotonicClock::new();
    let _timestamp = wall.now().expect("SystemWallClock::now 必须可表示");
    let a = mono.now();
    let b = mono.now();
    assert!(b >= a);
}

#[test]
fn system_monotonic_series_non_decreasing() {
    let wall = SystemWallClock::new();
    let mono = SystemMonotonicClock::new();
    let mut prev = mono.now();
    for _ in 0..32 {
        let _wall = wall.now().expect("墙钟必须可表示");
        let m = mono.now();
        assert!(m >= prev);
        prev = m;
    }
}

#[test]
fn test_system_wall_clock_now_returns_valid_timestamp() {
    let clock = SystemWallClock::new();
    let ts = clock.now().expect("墙钟应可用");
    let y2k_nanos: i64 = 946_684_800_000_000_000;
    assert!(ts.as_unix_nanos() > y2k_nanos);
}

#[test]
fn time_error_maps_to_xerror() {
    let overflow: XError = TimeError::Overflow.into();
    let range: XError = TimeError::SystemTimeOutOfRange.into();
    let order: XError = TimeError::InvalidOrder {
        later: UnixTimeNs::UNIX_EPOCH,
        earlier: UnixTimeNs::from_unix_nanos(1),
    }
    .into();
    assert_eq!(overflow.kind(), ErrorKind::Unavailable);
    assert_eq!(range.kind(), ErrorKind::Unavailable);
    assert_eq!(order.kind(), ErrorKind::Invalid);
}

/// UnixTimeNs 边界：MIN/MAX 与 u64 Duration 溢出语义。
#[test]
fn unix_time_min_max_and_u64_edges() {
    let min = UnixTimeNs::from_unix_nanos(i64::MIN);
    let max = UnixTimeNs::from_unix_nanos(i64::MAX);
    assert_eq!(min.as_unix_nanos(), i64::MIN);
    assert_eq!(max.as_unix_nanos(), i64::MAX);
    let full_i64_span = Duration::from_nanos(u64::MAX);
    assert_eq!(min.checked_add(full_i64_span), Ok(max));
    assert_eq!(max.checked_sub(full_i64_span), Ok(min));
    assert_eq!(max.duration_since(min), Ok(full_i64_span));
    assert!(max.duration_since(min).is_ok());
    assert!(min.duration_since(max).is_err());
    assert!(max.checked_add(Duration::from_nanos(1)).is_err());
    assert!(min.checked_sub(Duration::from_nanos(1)).is_err());
    assert_eq!(max.duration_since(max), Ok(Duration::ZERO));
    assert_eq!(UnixTimeNs::UNIX_EPOCH.checked_add(Duration::MAX), Err(TimeError::Overflow));
    let huge = Duration::from_secs(u64::from(u32::MAX));
    assert!(UnixTimeNs::from_unix_nanos(0).checked_add(huge).is_ok());
    let near_max = UnixTimeNs::from_unix_nanos(i64::MAX - 10);
    assert!(near_max.checked_add(Duration::from_nanos(20)).is_err());
    assert_eq!(near_max.checked_add(Duration::from_nanos(10)).unwrap().as_unix_nanos(), i64::MAX);
}

/// ComponentState 合法边全枚举 + 非法边全矩阵。
#[test]
fn component_state_transition_matrix() {
    use ComponentState::*;
    let legal = [
        (Created, Starting),
        (Starting, Running),
        (Starting, Failed),
        (Running, Draining),
        (Running, Failed),
        (Draining, Stopped),
        (Draining, Failed),
    ];
    let all = [Created, Starting, Running, Draining, Stopped, Failed];
    for (from, to) in legal {
        assert!(from.can_transition_to(to), "{from:?}->{to:?}");
        assert_eq!(from.try_transition(to).unwrap(), to);
    }
    for from in all {
        for to in all {
            let allowed = legal.contains(&(from, to));
            assert_eq!(from.can_transition_to(to), allowed, "矩阵不一致 {from:?}->{to:?}");
            assert_eq!(from.try_transition(to).is_ok(), allowed);
        }
    }
}

/// 任意 `Duration`：由秒 + 亚秒纳秒合成，覆盖大秒数与亚秒分量。
fn arb_duration() -> impl Strategy<Value = Duration> {
    (
        prop_oneof![
            Just(0u64),
            0u64..10_000,
            10_000u64..1_000_000,
            any::<u32>().prop_map(u64::from),
            Just(u64::MAX),
        ],
        0u32..1_000_000_000,
    )
        .prop_map(|(secs, nanos)| Duration::new(secs, nanos))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn unix_time_checked_add_sub_agree(
        nanos in any::<i64>(),
        d in arb_duration(),
    ) {
        let t = UnixTimeNs::from_unix_nanos(nanos);
        if let Ok(t2) = t.checked_add(d) {
            if let (Ok(delta_i64), Ok(back)) = (
                i64::try_from(d.as_nanos()),
                t2.duration_since(t),
            )
                && t2.as_unix_nanos().checked_sub(t.as_unix_nanos()) == Some(delta_i64) {
                    prop_assert_eq!(back, d);
                }
            if let Ok(t0) = t2.checked_sub(d) {
                prop_assert_eq!(t0.as_unix_nanos(), t.as_unix_nanos());
            }
        } else if nanos == i64::MAX && d > Duration::ZERO {
            prop_assert!(t.checked_add(d).is_err());
        }
        let _ = t.checked_sub(d);
    }

    #[test]
    fn unix_time_reverse_since_is_err(
        a in any::<i64>(),
        b in any::<i64>(),
    ) {
        let ta = UnixTimeNs::from_unix_nanos(a);
        let tb = UnixTimeNs::from_unix_nanos(b);
        if a < b {
            prop_assert!(ta.duration_since(tb).is_err());
        }
        if a > b {
            prop_assert!(tb.duration_since(ta).is_err());
        }
        if a == b {
            prop_assert_eq!(ta.duration_since(tb), Ok(Duration::ZERO));
        }
    }

    #[test]
    fn unix_time_near_i64_bounds(
        base in prop_oneof![Just(i64::MIN), Just(i64::MAX), Just(0i64), any::<i64>()],
        d in arb_duration(),
    ) {
        let t = UnixTimeNs::from_unix_nanos(base);
        let _ = t.checked_add(d);
        let _ = t.checked_sub(d);
        if base == i64::MAX && d > Duration::ZERO {
            prop_assert!(t.checked_add(d).is_err());
        }
        if base == i64::MIN && d > Duration::ZERO {
            prop_assert!(t.checked_sub(d).is_err());
        }
        prop_assert_eq!(t.as_unix_nanos(), base);
    }

    #[test]
    fn error_kind_constructor_matrix(kind_idx in 0usize..9) {
        let (err, expect) = match kind_idx {
            0 => (XError::invalid("x"), ErrorKind::Invalid),
            1 => (XError::missing("x"), ErrorKind::Missing),
            2 => (XError::conflict("x"), ErrorKind::Conflict),
            3 => (XError::transient("x"), ErrorKind::Transient),
            4 => (XError::unavailable("x"), ErrorKind::Unavailable),
            5 => (XError::cancelled("x"), ErrorKind::Cancelled),
            6 => (XError::deadline_exceeded("x"), ErrorKind::DeadlineExceeded),
            7 => (XError::invariant("x"), ErrorKind::Invariant),
            _ => (XError::internal("x"), ErrorKind::Internal),
        };
        prop_assert_eq!(err.kind(), expect);
        prop_assert_eq!(err.is_retryable(), matches!(expect, ErrorKind::Transient));
        prop_assert_eq!(err.is_bug(), matches!(expect, ErrorKind::Invariant));
        prop_assert_eq!(err.context(), "x");
    }

    #[test]
    fn component_state_pair_matrix(from_idx in 0usize..6, to_idx in 0usize..6) {
        use ComponentState::*;
        let all = [Created, Starting, Running, Draining, Stopped, Failed];
        let legal = [
            (Created, Starting),
            (Starting, Running),
            (Starting, Failed),
            (Running, Draining),
            (Running, Failed),
            (Draining, Stopped),
            (Draining, Failed),
        ];
        let from = all[from_idx];
        let to = all[to_idx];
        let allowed = legal.contains(&(from, to));
        prop_assert_eq!(from.can_transition_to(to), allowed);
        prop_assert_eq!(from.try_transition(to).is_ok(), allowed);
        if allowed {
            prop_assert_eq!(from.try_transition(to).unwrap(), to);
        } else {
            let err = from.try_transition(to).unwrap_err();
            prop_assert_eq!(err.from, from);
            prop_assert_eq!(err.to, to);
        }
    }
}
