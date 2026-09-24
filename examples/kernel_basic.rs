#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 最小消费者路径：墙钟读取 + 关停信号 + 错误分类。
//!
//! ```bash
//! cargo run -p kernel --example kernel_basic
//! ```

use std::time::Duration;

use kernel::time_storage::{self, PrecisionLossPolicy, TimePrecision};
use kernel::{
    ComponentState, ErrorKind, MonotonicClock, ShutdownSignal, SystemMonotonicClock,
    SystemWallClock, WallClock, XError,
};

fn main() {
    let wall = SystemWallClock::new();
    let now = wall.now().expect("wall clock available");
    assert!(now.as_unix_nanos() > 0, "unix nanos must be positive");
    let _mono = SystemMonotonicClock::new().now();

    let err = XError::invalid("bad input");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(!err.is_retryable());

    assert!(ComponentState::Created.can_transition_to(ComponentState::Starting));
    let next = ComponentState::Created
        .try_transition(ComponentState::Starting)
        .expect("Created → Starting is legal");
    assert_eq!(next, ComponentState::Starting);

    // time_storage：精度合同纯函数（无 I/O）
    let stored = time_storage::unix_ns_to_stored(
        1_500,
        TimePrecision::Microseconds,
        PrecisionLossPolicy::ExplicitTruncate,
    )
    .expect("quantize to microsecond storage");
    assert_eq!(stored, 1);
    let restored = time_storage::stored_to_unix_ns(stored, TimePrecision::Microseconds)
        .expect("restore from microsecond storage");
    assert_eq!(restored, 1_000);

    let (guard, signal) = ShutdownSignal::new();
    assert!(!signal.is_triggered());
    guard.trigger();
    assert!(signal.is_triggered());
    assert!(signal.wait_timeout(Duration::from_millis(10)).expect("deadline 可表示"));

    let profile = std::env::var("KERNEL_LIVE_PROFILE").unwrap_or_else(|_| "development".into());
    println!(
        "kernel-consumer: ok now_ns={} shutdown_triggered=true profile={profile}",
        now.as_unix_nanos()
    );
}
