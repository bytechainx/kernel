//! # `kernel` — xhyper.rs L0 语义信任根
//!
//! `kernel` 定义全系统必须唯一且长期稳定的两类语义：
//!
//! 1. **错误分类与响应**（[`error`]）—— 按"调用方应如何反应"分类，不按模块来源分类；
//! 2. **生命周期与关停信号**（[`lifecycle`]）—— 关停一次触发、多方观察、不可逆。
//!
//! `kernel` 不提供配置、日志、网络、异步运行时、依赖注入、连接池或业务能力。
//! 任何新增公开项、依赖或 feature 都必须走 RFC。
//!
//! 0.5.0 移除时间类型、时钟与时间持久化公开面；调用方直接使用独立 `temporal` crate，
//! 并在自己的边界显式转换错误。`kernel` 与 `temporal` 互不依赖。

#![cfg_attr(
    test,
    allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]

pub mod error;
pub mod lifecycle;

pub use error::{BoxError, ErrorKind, XError, XResult};
pub use lifecycle::{
    ComponentState, LifecycleError, ShutdownGuard, ShutdownSignal, WaitTimeoutError,
};
