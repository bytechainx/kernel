#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! SPEC-KERNEL-002 §11.4 — 编译期面。
//!
//! `static_assertions` 证明 trait 边界；`error` 模块的 rustdoc `compile_fail` 另从真实
//! 下游编译面证明字段私有、无 `Component`、时间类型无 `Default`、`ShutdownGuard`
//! 无 `Clone`、无 serde。两者互补。

use kernel::{
    ComponentState, ErrorKind, MonotonicClock, ShutdownGuard, ShutdownSignal, SystemMonotonicClock,
    SystemWallClock, UnixTimeNs, WallClock, XError,
};
use static_assertions::{assert_impl_all, assert_not_impl_any};
use std::time::Duration;

assert_impl_all!(SystemWallClock: WallClock, Send, Sync, Clone, Copy);
assert_impl_all!(UnixTimeNs: Copy, Send, Sync);
assert_impl_all!(ErrorKind: Copy, Send, Sync);
assert_impl_all!(XError: Send, Sync);

// 业务类型不得 Default（禁止哨兵 0 时间）
assert_not_impl_any!(UnixTimeNs: Default);
// XError 不可 Clone（source chain）
assert_not_impl_any!(XError: Clone);
// ShutdownGuard 消费式 trigger，禁止 Clone
assert_not_impl_any!(ShutdownGuard: Clone);

// §5.5：禁止 From 旁路绕过显式 ErrorKind 分类
assert_not_impl_any!(XError: From<&'static str>, From<String>);

// 禁止 From<SystemTime> / From<i64> / Into<i64> 与 UnixTimeNs Display 人类时间
assert_not_impl_any!(
    UnixTimeNs: From<std::time::SystemTime>,
    From<i64>,
    Into<i64>,
    std::fmt::Display
);

// §11.4：kernel 类型无 serde derive（dev-dep 提供 trait，不进入生产图）
assert_not_impl_any!(UnixTimeNs: serde::Serialize, serde::Deserialize<'static>);
assert_not_impl_any!(ErrorKind: serde::Serialize, serde::Deserialize<'static>);
assert_not_impl_any!(ComponentState: serde::Serialize, serde::Deserialize<'static>);
assert_not_impl_any!(SystemWallClock: serde::Serialize, serde::Deserialize<'static>);
assert_not_impl_any!(SystemMonotonicClock: serde::Serialize, serde::Deserialize<'static>);
assert_not_impl_any!(XError: serde::Serialize, serde::Deserialize<'static>);
assert_not_impl_any!(ShutdownSignal: serde::Serialize, serde::Deserialize<'static>);
assert_not_impl_any!(ShutdownGuard: serde::Serialize, serde::Deserialize<'static>);

#[test]
fn public_constructors_and_queries_compile() {
    let _ = XError::invalid("x");
    let _ = XError::missing("m");
    let _ = XError::conflict("c");
    let _ = XError::transient("t");
    let _ = XError::transient_after("t", Duration::from_millis(1));
    let _ = XError::unavailable("u");
    let _ = XError::cancelled("k");
    let _ = XError::deadline_exceeded("d");
    let _ = XError::invariant("i");
    let e = XError::internal("n").with_source(std::io::Error::other("s"));
    assert_eq!(e.kind(), ErrorKind::Internal);
    assert_eq!(e.context(), "n");
    let clock = SystemWallClock::new();
    let _ = clock.now();
    let _ = SystemMonotonicClock::new().now();
    let _ = UnixTimeNs::from_unix_nanos(1).checked_add(Duration::from_nanos(1));
}

#[test]
fn test_all_public_items_accessible() {
    let _ = XError::invalid("test");
    let _ = XError::missing("test");
    let _ = XError::conflict("test");
    let _ = XError::transient("test");
    let _ = XError::transient_after("test", std::time::Duration::from_secs(1));
    let _ = XError::unavailable("test");
    let _ = XError::cancelled("test");
    let _ = XError::deadline_exceeded("test");
    let _ = XError::invariant("test");
    let _ = XError::internal("test");

    let err = XError::invalid("test");
    let _ = err.kind();
    let _ = err.context();
    let _ = err.retry_after();
    let _ = err.is_retryable();
    let _ = err.is_bug();

    let _ = err.with_source(std::io::Error::other("source"));

    let clock = SystemWallClock::new();
    let _ = clock.now();
    let _ = SystemMonotonicClock::new().now();

    let ts = UnixTimeNs::from_unix_nanos(0);
    let _ = ts.as_unix_nanos();
    let _ = ts.checked_add(std::time::Duration::from_secs(1));
    let _ = ts.checked_sub(std::time::Duration::from_secs(1));
    let _ = ts.duration_since(ts);

    let _ = ComponentState::Created.can_transition_to(ComponentState::Starting);
    let _ = ComponentState::Created.try_transition(ComponentState::Starting);

    let (_guard, _signal) = ShutdownSignal::new();
}
