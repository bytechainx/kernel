//! 独立消费者：只经 kernel 公开 API 使用错误、生命周期与关停类型。
use std::time::Duration;

use kernel::{ComponentState, ErrorKind, ShutdownSignal, XError};

fn main() {
    let err = XError::invalid("consumer input");
    assert_eq!(err.kind(), ErrorKind::Invalid);
    assert!(!err.is_retryable());
    assert!(!err.is_bug());

    let next = ComponentState::Created
        .try_transition(ComponentState::Starting)
        .expect("Created -> Starting 是合法转换");
    assert_eq!(next, ComponentState::Starting);

    let (guard, signal) = ShutdownSignal::new();
    let observer = signal.clone();
    assert!(!observer.is_triggered());
    guard.trigger();
    assert!(observer.is_triggered());
    assert!(signal
        .wait_timeout(Duration::from_millis(10))
        .expect("deadline 可表示"));

    println!("kernel-consumer-fixture: ok");
}
