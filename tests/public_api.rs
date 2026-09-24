#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 公开 API 烟雾：ErrorKind / time / Shutdown 表面。
use std::time::Duration;

use kernel::{
    BoxError, ComponentState, ErrorKind, MonotonicClock, ShutdownSignal, SystemMonotonicClock,
    SystemWallClock, TimeError, UnixTimeNs, WallClock, XError, XResult,
};

#[test]
fn test_public_api_basics() {
    let err: XError = XError::invalid("test");
    let _kind: ErrorKind = err.kind();
    let _ctx: &str = err.context();
    let _retry: Option<std::time::Duration> = err.retry_after();
    let _boxed: BoxError = Box::new(std::io::Error::other("oops"));
    let _result: XResult<()> = Err(err);

    let clock = SystemWallClock::new();
    let _ts: Result<UnixTimeNs, TimeError> = clock.now();
    let _mono = SystemMonotonicClock::new().now();

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
fn clock_contract_system() {
    let wall = SystemWallClock::new();
    let mono = SystemMonotonicClock::new();
    assert!(wall.now().unwrap().as_unix_nanos() > 0);
    let a = mono.now();
    let _sum: u64 = (0..1_000_000).sum();
    let b = mono.now();
    assert!(b >= a);
    let t = UnixTimeNs::from_unix_nanos(10);
    assert_eq!(t.checked_sub(Duration::from_nanos(3)).unwrap().as_unix_nanos(), 7);
}

#[test]
fn shutdown_trigger_wakes() {
    let (g, s) = ShutdownSignal::new();
    assert!(!s.is_triggered());
    g.trigger();
    assert!(s.is_triggered());
    s.wait();
}
