#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `kernel` 的公开行为入口探针。
//!
//! // TDD-PROBE: XError::invalid | 变异：错误分类改为 internal | 红=invalid_error_is_not_retryable | 绿=invalid_error_is_not_retryable
//! // TDD-PROBE: XError::is_retryable | 变异：所有错误均返回 true | 红=invalid_error_is_not_retryable | 绿=invalid_error_is_not_retryable
//! // TDD-PROBE: ShutdownSignal::new | 变异：新信号初始为 triggered | 红=shutdown_signal_transitions_once | 绿=shutdown_signal_transitions_once
//! // TDD-PROBE: ShutdownGuard::trigger | 变异：trigger 不更新共享状态 | 红=shutdown_signal_transitions_once | 绿=shutdown_signal_transitions_once

use kernel::{ErrorKind, ShutdownSignal, XError};

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
