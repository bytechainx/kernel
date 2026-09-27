#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `kernel` 的 AI 生成边界候选；逐条人工复核后保留。
//!
//! // AIDD: 零时长等待未触发信号 | 来源=AI | 复核=ZoneCNH/2026-09-24 | 依据=标准.md §范围，超时等待不应误报触发 | 结论=保留

use std::time::Duration;

use kernel::ShutdownSignal;

#[test]
fn zero_timeout_does_not_claim_an_untriggered_signal() {
    let (_, signal) = ShutdownSignal::new();
    assert_eq!(signal.wait_timeout(Duration::ZERO), Ok(false));
}
