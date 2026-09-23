//! 统一时间模型 Foundation 模块（XH-TIME-MODEL-SPEC-001 §1.2）。
//!
//! 本模块是时间 Foundation / 时钟 / 精度 / Wire 时间契约的**唯一权威**实现
//! （XH-TIME-MODEL-GOAL-001 / K-11）。父级 Arch SPEC-002 §4.1 已降级为摘要 +
//! Context owner 映射，不在此定义第二套时间类型系统。
//!
//! 组成（SPEC §1.2）：
//!
//! - [`UnixTimeNs`] + [`TimeError`]（§2，实现于内部 `unix_time` 子模块）；
//! - [`WallClock`] / [`MonotonicClock`] 抽象与系统实现（§3，实现于内部 `clock` 子模块）。
//!
//! 设计约束（SPEC §1.1 / §1.3）：
//!
//! - Foundation 只定义三个基础概念：绝对时刻 [`UnixTimeNs`]、持续时间
//!   `std::time::Duration`、单调时刻 `std::time::Instant`；
//! - 不长期保留 `UnixMillis` / `UnixMicros` / `MonotonicNanos` / `Timestamp` 平行类型；
//! - 本模块 std-only + `thiserror`，不引入 chrono/time/serde/tokio/uuid/reqwest
//!   （SPEC §1.3）。
//!
//! `clock` / `unix_time` 子模块为 crate 内部实现分区；跨 crate 消费者须使用
//! 本模块或 crate 根的 re-export，不得依赖子模块路径：
//!
//! ```compile_fail
//! use kernel::time::clock::WallClock;
//! ```

mod clock;
mod unix_time;

pub use clock::{MonotonicClock, RuntimeClock, SystemMonotonicClock, SystemWallClock, WallClock};
pub use unix_time::{TimeError, UnixTimeNs};
