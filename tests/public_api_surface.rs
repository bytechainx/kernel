#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! 公开消费面全量驱动：每个 crate 根 re-export 类型/构造器/方法至少调用一次并断言结果。

use std::error::Error;
use std::time::Duration;

use kernel::{
    BoxError, ComponentState, ErrorKind, LifecycleError, ShutdownSignal, WaitTimeoutError, XError,
    XResult,
};

#[test]
fn error_constructors_and_queries() {
    for (e, kind) in [
        (XError::invalid("i"), ErrorKind::Invalid),
        (XError::missing("m"), ErrorKind::Missing),
        (XError::conflict("c"), ErrorKind::Conflict),
        (XError::transient("t"), ErrorKind::Transient),
        (XError::unavailable("u"), ErrorKind::Unavailable),
        (XError::cancelled("x"), ErrorKind::Cancelled),
        (XError::deadline_exceeded("d"), ErrorKind::DeadlineExceeded),
        (XError::invariant("v"), ErrorKind::Invariant),
        (XError::internal("n"), ErrorKind::Internal),
    ] {
        assert_eq!(e.kind(), kind);
        assert_eq!(e.is_retryable(), kind == ErrorKind::Transient);
        assert_eq!(e.is_bug(), kind == ErrorKind::Invariant);
        assert!(!e.context().is_empty());
        assert!(!e.to_string().is_empty());
    }
    let e = XError::transient_after("a", Duration::from_millis(5));
    assert_eq!(e.retry_after(), Some(Duration::from_millis(5)));
    let e2 = e.with_source(std::io::Error::other("src"));
    assert_eq!(e2.kind(), ErrorKind::Transient);
    assert!(e2.source().is_some());
    let _boxed: BoxError = Box::new(std::io::Error::other("b"));
    let r: XResult<u8> = Err(XError::invalid("r"));
    assert!(r.is_err());
}

#[test]
fn lifecycle_states_and_shutdown_timeout() {
    let chain = [
        (ComponentState::Created, ComponentState::Starting),
        (ComponentState::Starting, ComponentState::Running),
        (ComponentState::Running, ComponentState::Draining),
        (ComponentState::Draining, ComponentState::Stopped),
    ];
    for (from, to) in chain {
        assert!(from.can_transition_to(to));
        assert_eq!(from.try_transition(to).unwrap(), to);
    }
    assert!(ComponentState::Starting.can_transition_to(ComponentState::Failed));
    assert!(ComponentState::Running.can_transition_to(ComponentState::Failed));
    assert!(ComponentState::Draining.can_transition_to(ComponentState::Failed));
    assert!(!ComponentState::Stopped.can_transition_to(ComponentState::Created));
    assert!(!ComponentState::Failed.can_transition_to(ComponentState::Running));

    let err = ComponentState::Created.try_transition(ComponentState::Running).unwrap_err();
    let le: LifecycleError = err;
    assert_eq!(le.from, ComponentState::Created);
    assert_eq!(le.to, ComponentState::Running);
    assert!(le.to_string().contains("非法"));

    let (g, s) = ShutdownSignal::new();
    assert!(!s.is_triggered());
    assert_eq!(s.wait_timeout(Duration::MAX), Err(WaitTimeoutError::DeadlineOverflow));
    assert!(!s.wait_timeout(Duration::from_millis(1)).unwrap());
    let observer = s.clone();
    g.trigger();
    assert!(s.is_triggered());
    assert!(observer.is_triggered());
    assert!(s.wait_timeout(Duration::from_millis(50)).unwrap());
    s.wait();
}

#[test]
fn modules_are_reachable_via_crate_paths() {
    let _ = kernel::error::XError::invalid("mod");
    let (g, s) = kernel::lifecycle::ShutdownSignal::new();
    g.trigger();
    assert!(s.is_triggered());
}
