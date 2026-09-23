//! 时钟抽象与系统实现（XH-TIME-MODEL-SPEC-001 §3）。
//!
//! 设计原则（SPEC §3.1 / §3.3）：
//!
//! - [`WallClock`] 返回绝对时刻 [`UnixTimeNs`]；[`MonotonicClock`] 返回
//!   `std::time::Instant`（仅进程内 elapsed/deadline/timeout）；
//! - Domain 不直接持有 Clock；Application/Runtime 负责 Clock 注入
//!   （TIME-INV-003 / SPEC §3.3）；
//! - `SystemTime::now()` / `Utc::now()` 不进入 Domain/Application Core；
//! - `std::time::Instant` 不得序列化、持久化或跨进程传输（TIME-INV-007 / SPEC §3.4）；
//! - 组合 trait [`RuntimeClock`] MAY 提供，但 Domain 不得依赖它（SPEC §3.1）。

use std::time::Instant;

use super::unix_time::{TimeError, UnixTimeNs};

/// 墙钟抽象：返回 UTC POSIX epoch 纳秒绝对时刻。
///
/// 允许因 NTP 或人工校时回退，调用方不得假设非递减（SPEC §3.1）。
pub trait WallClock: Send + Sync {
    /// 获取当前墙钟时间。
    ///
    /// # Errors
    ///
    /// 时间源不可用或超出可表示范围时返回 [`TimeError`]。
    fn now(&self) -> Result<UnixTimeNs, TimeError>;
}

/// 单调时钟抽象：返回进程内单调时刻。
///
/// 仅用于测量间隔、deadline 与 timeout；绝对值无业务意义，不可持久化
/// （SPEC §3.1 / TIME-INV-007）。
pub trait MonotonicClock: Send + Sync {
    /// 获取当前单调采样点。
    fn now(&self) -> Instant;
}

/// 组合时钟（墙钟 + 单调钟）。
///
/// MAY 提供，但 Domain 不得依赖（SPEC §3.1）。
pub trait RuntimeClock: WallClock + MonotonicClock {}

impl<T: WallClock + MonotonicClock> RuntimeClock for T {}

/// 基于 `std::time` 的系统墙钟实现（生产唯一墙钟实现）。
///
/// 测试应使用 `testkit::ManualWallClock`。
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemWallClock;

impl SystemWallClock {
    /// 创建系统墙钟。
    pub const fn new() -> Self {
        Self
    }
}

impl WallClock for SystemWallClock {
    fn now(&self) -> Result<UnixTimeNs, TimeError> {
        UnixTimeNs::try_from_system_time(std::time::SystemTime::now())
    }
}

/// 基于 `std::time::Instant` 的系统单调时钟实现。
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemMonotonicClock;

impl SystemMonotonicClock {
    /// 创建系统单调时钟。
    pub const fn new() -> Self {
        Self
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_wall_clock_returns_epoch_nanos() {
        let clock = SystemWallClock::new();
        let ts = clock.now().expect("墙钟应可用");
        // 2001-09-09T01:46:40Z = 1_000_000_000_000_000_000 ns；当前时间应在其后
        assert!(ts.as_unix_nanos() > 1_000_000_000_000_000_000);
    }

    #[test]
    fn system_monotonic_clock_advances() {
        let clock = SystemMonotonicClock::new();
        let a = clock.now();
        let b = clock.now();
        assert!(b >= a);
    }

    #[test]
    fn runtime_clock_is_both() {
        fn assert_wall<C: WallClock>(_: &C) {}
        fn assert_mono<C: MonotonicClock>(_: &C) {}
        let c = SystemWallClock::new();
        assert_wall(&c);
        let m = SystemMonotonicClock::new();
        assert_mono(&m);
    }

    #[test]
    fn system_types_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SystemWallClock>();
        assert_send_sync::<SystemMonotonicClock>();
    }
}
