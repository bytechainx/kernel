#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 错误分类与生命周期合同：ErrorKind / ComponentState。

use kernel::{ComponentState, ErrorKind, XError};
use proptest::prelude::*;

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

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

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
