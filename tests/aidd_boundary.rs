#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `kernel` 的 AI 生成边界候选；逐条人工复核后保留。
//!
//! // AIDD: 零时长等待未触发信号 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §范围，超时等待不应误报触发 | 结论=保留
//! // AIDD: 零退避的瞬时错误仍可重试 | 来源=AI | 复核= | 依据=标准.md §兼容要求，重试分类与退避提示独立 | 结论=待复核
//! // AIDD: 内部错误不可自动重试 | 来源=AI | 复核= | 依据=标准.md §范围，错误分类由调用方反应决定 | 结论=待复核
//! // AIDD: 已停止状态不能再次转换 | 来源=AI | 复核= | 依据=标准.md §范围，生命周期状态转换显式拒绝非法边 | 结论=待复核
//! // AIDD: 已触发信号优先于不可表示的等待时长 | 来源=AI | 复核= | 依据=标准.md §兼容要求，关停触发不可逆 | 结论=待复核

use std::time::Duration;

use kernel::{ComponentState, ErrorKind, ShutdownSignal, XError};

#[test]
fn zero_timeout_does_not_claim_an_untriggered_signal() {
    let (_, signal) = ShutdownSignal::new();
    assert_eq!(signal.wait_timeout(Duration::ZERO), Ok(false));
}

#[test]
fn zero_backoff_transient_error_remains_retryable() {
    let error = XError::transient_after("稍后重试", Duration::ZERO);
    assert_eq!(error.kind(), ErrorKind::Transient);
    assert!(error.is_retryable());
    assert_eq!(error.retry_after(), Some(Duration::ZERO));
}

#[test]
fn internal_error_is_not_automatically_retryable() {
    let error = XError::internal("内部故障");
    assert_eq!(error.kind(), ErrorKind::Internal);
    assert!(!error.is_retryable());
}

#[test]
fn stopped_state_rejects_reentry() {
    let error = ComponentState::Stopped.try_transition(ComponentState::Stopped).unwrap_err();
    assert_eq!(error.from, ComponentState::Stopped);
    assert_eq!(error.to, ComponentState::Stopped);
}

#[test]
fn triggered_signal_precedes_unrepresentable_timeout() {
    let (guard, signal) = ShutdownSignal::new();
    guard.trigger();
    assert_eq!(signal.wait_timeout(Duration::MAX), Ok(true));
}
