#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 公开消费面全量驱动：每个 crate 根 re-export 类型/构造器/方法至少调用一次并断言结果。

use std::error::Error;
use std::time::Duration;

use kernel::{
    BoxError, ComponentState, ErrorKind, LifecycleError, MonotonicClock, PrecisionLossPolicy,
    ShutdownSignal, SystemMonotonicClock, SystemWallClock, TimeError, TimePrecision,
    TimeStorageCapabilities, UnixTimeNs, WaitTimeoutError, WallClock, XError, XResult,
    quantize_unix_ns, require_lossless_unix_ns, stored_to_unix_ns, unix_ns_to_stored,
    verify_pg_dual_column,
};

#[test]
fn error_constructors_and_queries() {
    for (e, kind) in [
        (XError::invalid("i"), ErrorKind::Invalid),
        (XError::missing("m"), ErrorKind::Missing),
        (XError::conflict("c"), ErrorKind::Conflict),
        (XError::transient("t"), ErrorKind::Transient),
        (XError::unavailable("u"), ErrorKind::Unavailable),
        (XError::cancelled("x"), ErrorKind::Cancelled),
        (XError::deadline_exceeded("d"), ErrorKind::DeadlineExceeded),
        (XError::invariant("v"), ErrorKind::Invariant),
        (XError::internal("n"), ErrorKind::Internal),
    ] {
        assert_eq!(e.kind(), kind);
        assert_eq!(e.is_retryable(), kind == ErrorKind::Transient);
        assert_eq!(e.is_bug(), kind == ErrorKind::Invariant);
        assert!(!e.context().is_empty());
        assert!(!e.to_string().is_empty());
    }
    let e = XError::transient_after("a", Duration::from_millis(5));
    assert_eq!(e.retry_after(), Some(Duration::from_millis(5)));
    let e2 = e.with_source(std::io::Error::other("src"));
    assert_eq!(e2.kind(), ErrorKind::Transient);
    assert!(e2.source().is_some());
    let _boxed: BoxError = Box::new(std::io::Error::other("b"));
    let r: XResult<u8> = Err(XError::invalid("r"));
    assert!(r.is_err());

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
    assert!(overflow.source().is_some());
    assert!(!overflow.context().is_empty());
}

#[test]
fn unix_time_and_clock_surface() {
    let t = UnixTimeNs::from_unix_nanos(100);
    assert_eq!(t.as_unix_nanos(), 100);
    assert_eq!(t.checked_add(Duration::from_nanos(3)).unwrap().as_unix_nanos(), 103);
    assert_eq!(t.checked_sub(Duration::from_nanos(3)).unwrap().as_unix_nanos(), 97);
    assert_eq!(
        UnixTimeNs::from_unix_nanos(110).duration_since(t).unwrap(),
        Duration::from_nanos(10)
    );
    assert_eq!(t.checked_sub(Duration::from_nanos(200)).unwrap().as_unix_nanos(), -100);
    assert!(UnixTimeNs::from_unix_nanos(i64::MIN).checked_sub(Duration::from_nanos(1)).is_err());
    assert!(t < UnixTimeNs::from_unix_nanos(101));
    assert_eq!(t.cmp(&UnixTimeNs::from_unix_nanos(100)), std::cmp::Ordering::Equal);

    let c = SystemWallClock::new();
    let now = c.now().unwrap();
    assert!(now.as_unix_nanos() > 0);
    let m = SystemMonotonicClock::new();
    let a = m.now();
    let b = m.now();
    assert!(b >= a);

    let d: SystemWallClock = Default::default();
    assert!(d.now().unwrap().as_unix_nanos() > 0);
    let cloned = c;
    assert!(cloned.now().unwrap().as_unix_nanos() > 0);

    let as_wall: &dyn WallClock = &c;
    assert!(as_wall.now().unwrap().as_unix_nanos() > 0);
    let as_mono: &dyn MonotonicClock = &m;
    let _ = as_mono.now();

    assert!(!TimeError::Overflow.to_string().is_empty());
    assert!(!TimeError::SystemTimeOutOfRange.to_string().is_empty());
    assert_eq!(TimeError::Overflow, TimeError::Overflow);
    assert_ne!(TimeError::Overflow, TimeError::SystemTimeOutOfRange);
}

#[test]
fn lifecycle_states_and_shutdown_timeout() {
    let chain = [
        (ComponentState::Created, ComponentState::Starting),
        (ComponentState::Starting, ComponentState::Running),
        (ComponentState::Running, ComponentState::Draining),
        (ComponentState::Draining, ComponentState::Stopped),
    ];
    for (from, to) in chain {
        assert!(from.can_transition_to(to));
        assert_eq!(from.try_transition(to).unwrap(), to);
    }
    assert!(ComponentState::Starting.can_transition_to(ComponentState::Failed));
    assert!(ComponentState::Running.can_transition_to(ComponentState::Failed));
    assert!(ComponentState::Draining.can_transition_to(ComponentState::Failed));
    assert!(!ComponentState::Stopped.can_transition_to(ComponentState::Created));
    assert!(!ComponentState::Failed.can_transition_to(ComponentState::Running));

    let err = ComponentState::Created.try_transition(ComponentState::Running).unwrap_err();
    let le: LifecycleError = err;
    assert_eq!(le.from, ComponentState::Created);
    assert_eq!(le.to, ComponentState::Running);
    assert!(le.to_string().contains("非法"));

    let (g, s) = ShutdownSignal::new();
    assert!(!s.is_triggered());
    assert_eq!(s.wait_timeout(Duration::MAX), Err(WaitTimeoutError::DeadlineOverflow));
    assert!(!s.wait_timeout(Duration::from_millis(1)).unwrap());
    let observer = s.clone();
    g.trigger();
    assert!(s.is_triggered());
    assert!(observer.is_triggered());
    assert!(s.wait_timeout(Duration::from_millis(50)).unwrap());
    s.wait();
}

#[test]
fn modules_are_reachable_via_crate_paths() {
    let _ = kernel::time::SystemWallClock::new();
    let _ = kernel::error::XError::invalid("mod");
    let (g, s) = kernel::lifecycle::ShutdownSignal::new();
    g.trigger();
    assert!(s.is_triggered());
    let _ = kernel::time_storage::TimePrecision::Nanoseconds;
}

#[test]
fn time_storage_surface() {
    assert_eq!(PrecisionLossPolicy::default(), PrecisionLossPolicy::Reject);
    let caps = TimeStorageCapabilities::new(TimePrecision::Nanoseconds, true, true);
    require_lossless_unix_ns(caps).unwrap();
    let ns = 1_500_000_000_i64;
    let stored =
        unix_ns_to_stored(ns, TimePrecision::Milliseconds, PrecisionLossPolicy::Reject).unwrap();
    assert_eq!(stored, 1500);
    assert_eq!(stored_to_unix_ns(stored, TimePrecision::Milliseconds).unwrap(), ns);
    assert_eq!(
        quantize_unix_ns(ns, TimePrecision::Milliseconds, PrecisionLossPolicy::Reject).unwrap(),
        ns
    );
    verify_pg_dual_column(1_000_000_123, 1_000_000_000).unwrap();
    assert!(
        unix_ns_to_stored(1, TimePrecision::Milliseconds, PrecisionLossPolicy::Reject).is_err()
    );
}
