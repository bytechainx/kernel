#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 最小消费者路径：生命周期转换、关停信号与错误分类。
//!
//! ```bash
//! cargo run -p kernel --example kernel_basic
//! ```

use std::time::Duration;

use kernel::{ComponentState, ErrorKind, ShutdownSignal, XError};

fn main() {
    let err = XError::invalid("bad input");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(!err.is_retryable());

    assert!(ComponentState::Created.can_transition_to(ComponentState::Starting));
    let next = ComponentState::Created
        .try_transition(ComponentState::Starting)
        .expect("Created → Starting is legal");
    assert_eq!(next, ComponentState::Starting);

    let (guard, signal) = ShutdownSignal::new();
    assert!(!signal.is_triggered());
    guard.trigger();
    assert!(signal.is_triggered());
    assert!(signal.wait_timeout(Duration::from_millis(10)).expect("deadline 可表示"));

    let profile = std::env::var("KERNEL_LIVE_PROFILE").unwrap_or_else(|_| "development".into());
    println!("kernel-consumer: ok state={next:?} shutdown_triggered=true profile={profile}");
}
