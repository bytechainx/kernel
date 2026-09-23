//! # `kernel` — xhyper.rs L0 语义信任根
//!
//! `kernel` 定义全系统必须唯一且长期稳定的四类语义：
//!
//! 1. **错误分类与响应**（[`error`]）—— 按"调用方应如何反应"分类，不按模块来源分类；
//! 2. **时间获取与表示**（[`time`]）—— 唯一绝对时刻 [`UnixTimeNs`]，墙钟与单调钟分离，
//!    时间源必须显式注入（XH-TIME-MODEL-SPEC-001）；
//! 3. **生命周期与关停信号**（[`lifecycle`]）—— 关停一次触发、多方观察、不可逆。
//! 4. **持久化精度合同**（[`time_storage`]）—— 只提供纯函数，不执行 I/O。
//!
//! `kernel` 不提供配置、日志、网络、异步运行时、依赖注入、连接池或业务能力。
//! 任何新增公开项、依赖或 feature 都必须走 RFC。
//!
//! 旧 `clock::{Timestamp, Clock, SystemClock, MonotonicInstant, ClockDomain}` 已删除
//! （SPEC §1.2：禁止两套 clock/time 并存）。调用方改用 [`time`]。

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

pub mod error;
pub mod lifecycle;
pub mod time;
pub mod time_storage;

pub use error::{BoxError, ErrorKind, XError, XResult};
pub use lifecycle::{
    ComponentState, LifecycleError, ShutdownGuard, ShutdownSignal, WaitTimeoutError,
};
pub use time::{
    MonotonicClock, RuntimeClock, SystemMonotonicClock, SystemWallClock, TimeError, UnixTimeNs,
    WallClock,
};
pub use time_storage::{
    PrecisionLossPolicy, TimePrecision, TimeStorageCapabilities, quantize_unix_ns,
    require_lossless_unix_ns, stored_to_unix_ns, unix_ns_to_stored, verify_pg_dual_column,
};
