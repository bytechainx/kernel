#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `kernel` 的公开行为入口探针。
//!
//! // TDD-PROBE: UnixTimeNs::from_unix_nanos | 变异：把输入纳秒归零 | 红=unix_time_preserves_epoch_nanos | 绿=unix_time_preserves_epoch_nanos
//! // TDD-PROBE: UnixTimeNs::as_unix_nanos | 变异：返回值加一 | 红=unix_time_preserves_epoch_nanos | 绿=unix_time_preserves_epoch_nanos
//! // TDD-PROBE: XError::invalid | 变异：错误分类改为 internal | 红=invalid_error_is_not_retryable | 绿=invalid_error_is_not_retryable
//! // TDD-PROBE: XError::is_retryable | 变异：所有错误均返回 true | 红=invalid_error_is_not_retryable | 绿=invalid_error_is_not_retryable
//! // TDD-PROBE: ShutdownSignal::new | 变异：新信号初始为 triggered | 红=shutdown_signal_transitions_once | 绿=shutdown_signal_transitions_once
//! // TDD-PROBE: ShutdownGuard::trigger | 变异：trigger 不更新共享状态 | 红=shutdown_signal_transitions_once | 绿=shutdown_signal_transitions_once
//! // TDD-PROBE: unix_ns_to_stored | 变异：忽略 Reject 精度损失策略 | 红=time_storage_enforces_precision_policy | 绿=time_storage_enforces_precision_policy

use kernel::{
    ErrorKind, PrecisionLossPolicy, ShutdownSignal, TimePrecision, UnixTimeNs, XError,
    unix_ns_to_stored,
};

#[test]
fn unix_time_preserves_epoch_nanos() {
    let time = UnixTimeNs::from_unix_nanos(-123);
    assert_eq!(time.as_unix_nanos(), -123);
}

#[test]
fn invalid_error_is_not_retryable() {
    let error = XError::invalid("输入不合法");
    assert_eq!(error.kind(), ErrorKind::Invalid);
    assert!(!error.is_retryable());
}

#[test]
fn shutdown_signal_transitions_once() {
    let (guard, signal) = ShutdownSignal::new();
    assert!(!signal.is_triggered());
    guard.trigger();
    assert!(signal.is_triggered());
}

#[test]
fn time_storage_enforces_precision_policy() {
    assert!(
        unix_ns_to_stored(1_000_000_001, TimePrecision::Seconds, PrecisionLossPolicy::Reject)
            .is_err()
    );
    assert!(matches!(
        unix_ns_to_stored(
            1_000_000_001,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitTruncate,
        ),
        Ok(1)
    ));
}
