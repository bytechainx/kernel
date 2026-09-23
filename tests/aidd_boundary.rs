#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `kernel` 的 AI 生成边界候选；逐条人工复核后保留。
//!
//! // AIDD: epoch 前一纳秒 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §范围，时间支持 epoch 前后 | 结论=保留
//! // AIDD: Unix 毫秒转纳秒溢出 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §职责，越界必须返回错误 | 结论=保留
//! // AIDD: 秒精度遇到纳秒余数 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §兼容要求，默认拒绝精度损失 | 结论=保留
//! // AIDD: 显式截断负时间 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §兼容要求，截断朝零 | 结论=保留
//! // AIDD: 零时长等待未触发信号 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §范围，超时等待不应误报触发 | 结论=保留

use std::time::Duration;

use kernel::{PrecisionLossPolicy, ShutdownSignal, TimePrecision, UnixTimeNs, unix_ns_to_stored};

#[test]
fn pre_epoch_nanosecond_is_representable() {
    assert_eq!(UnixTimeNs::from_unix_nanos(-1).as_unix_nanos(), -1);
}

#[test]
fn millisecond_conversion_overflow_is_rejected() {
    assert!(UnixTimeNs::try_from_unix_millis(i64::MAX).is_err());
}

#[test]
fn precision_loss_is_rejected_by_default_policy() {
    assert!(
        unix_ns_to_stored(1_500_000_001, TimePrecision::Seconds, PrecisionLossPolicy::Reject)
            .is_err()
    );
}

#[test]
fn explicit_truncation_toward_zero_handles_negative_values() {
    assert!(matches!(
        unix_ns_to_stored(
            -1_500_000_000,
            TimePrecision::Seconds,
            PrecisionLossPolicy::ExplicitTruncate
        ),
        Ok(-1)
    ));
}

#[test]
fn zero_timeout_does_not_claim_an_untriggered_signal() {
    let (_, signal) = ShutdownSignal::new();
    assert_eq!(signal.wait_timeout(Duration::ZERO), Ok(false));
}
