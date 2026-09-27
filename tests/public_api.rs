#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 公开 API 烟雾：ErrorKind / Shutdown 表面。
use std::time::Duration;

use kernel::{BoxError, ComponentState, ErrorKind, ShutdownSignal, XError, XResult};

#[test]
fn test_public_api_basics() {
    let err: XError = XError::invalid("test");
    let _kind: ErrorKind = err.kind();
    let _ctx: &str = err.context();
    let _retry: Option<std::time::Duration> = err.retry_after();
    let _boxed: BoxError = Box::new(std::io::Error::other("oops"));
    let _result: XResult<()> = Err(err);

    let _state: ComponentState = ComponentState::Created;
    let (_guard, _signal) = ShutdownSignal::new();
}

#[test]
fn error_kind_query_surface() {
    assert_eq!(XError::missing("x").kind(), ErrorKind::Missing);
    assert!(XError::transient("t").is_retryable());
    assert!(XError::invariant("i").is_bug());
    assert_eq!(
        XError::transient_after("a", Duration::from_millis(1)).retry_after(),
        Some(Duration::from_millis(1))
    );
}

#[test]
fn error_context_and_debug_are_observably_correct() {
    const TOKEN: &str = "unique-context-token-not-xyzzy";
    let owned = XError::invalid(TOKEN);
    assert_eq!(owned.context(), TOKEN);
    assert_ne!(owned.context(), "xyzzy");

    let e = XError::missing("needle-in-debug-output");
    let debug = format!("{e:?}");
    assert!(debug.starts_with("XError"), "debug={debug}");
    assert!(debug.contains("needle-in-debug-output"), "debug={debug}");
    assert!(debug.contains("kind"), "debug={debug}");
    assert!(debug.contains("context"), "debug={debug}");
    assert!(debug.len() > 16, "debug={debug}");
}

#[test]
fn shutdown_trigger_wakes() {
    let (g, s) = ShutdownSignal::new();
    assert!(!s.is_triggered());
    g.trigger();
    assert!(s.is_triggered());
    s.wait();
}
