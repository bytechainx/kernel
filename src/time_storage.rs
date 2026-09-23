//! 持久化时间能力合同（XH-TIME-MODEL-SPEC-001 §10）。
//!
//! 本模块**只**提供纯类型与纯函数：Adapter 声明存储精度、精度损失策略，以及
//! Unix 纳秒与后端存储单位之间的有策略换算。不执行 I/O、不持有连接。
//!
//! # 不变式
//!
//! - 默认策略为 [`PrecisionLossPolicy::Reject`]，禁止静默截断 ns；
//! - `required-lossless` profile 要求 [`TimeStorageCapabilities::lossless_unix_ns`]；
//! - PostgreSQL 推荐 `event_time_ns BIGINT`（真相源）+ `event_time_utc TIMESTAMPTZ`（投影）。

use crate::error::{XError, XResult};

// ---------------------------------------------------------------------------
// TimePrecision
// ---------------------------------------------------------------------------

/// 后端可声明的时间存储精度（SPEC §10 / §4.2 同形）。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimePrecision {
    /// 秒。
    Seconds,
    /// 毫秒。
    Milliseconds,
    /// 微秒。
    Microseconds,
    /// 纳秒（完整 Unix ns）。
    Nanoseconds,
}

impl TimePrecision {
    /// 1 个本单位对应的纳秒数。
    #[must_use]
    pub const fn nanos_per_unit(self) -> i64 {
        match self {
            Self::Seconds => 1_000_000_000,
            Self::Milliseconds => 1_000_000,
            Self::Microseconds => 1_000,
            Self::Nanoseconds => 1,
        }
    }

    /// 本精度是否可无损保存完整 Unix 纳秒。
    #[must_use]
    pub const fn is_lossless_unix_ns(self) -> bool {
        matches!(self, Self::Nanoseconds)
    }
}

// ---------------------------------------------------------------------------
// Capabilities / Policy
// ---------------------------------------------------------------------------

/// Adapter 必须声明的时间存储能力（SPEC §10）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimeStorageCapabilities {
    /// 后端原生时间列可保证的精度。
    pub stored_precision: TimePrecision,
    /// 写入/读出是否归一到 UTC（无本地时区歧义）。
    pub utc_normalized: bool,
    /// 是否可无损往返完整 `i64` Unix 纳秒（原生 ns 列或 BIGINT 辅助列）。
    pub lossless_unix_ns: bool,
}

impl TimeStorageCapabilities {
    /// 构造能力声明。
    #[must_use]
    pub const fn new(
        stored_precision: TimePrecision,
        utc_normalized: bool,
        lossless_unix_ns: bool,
    ) -> Self {
        Self { stored_precision, utc_normalized, lossless_unix_ns }
    }
}

/// 精度损失策略（SPEC §10）。默认 MUST 为 [`Reject`](Self::Reject)。
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrecisionLossPolicy {
    /// 无法无损表示时返回错误（默认）。
    #[default]
    Reject,
    /// 调用方显式允许向存储精度截断（向 0 取整）。
    ExplicitTruncate,
}

// ---------------------------------------------------------------------------
// Conversion
// ---------------------------------------------------------------------------

/// 将 Unix 纳秒换算为后端存储单位数值。
///
/// - [`PrecisionLossPolicy::Reject`]：若 `unix_ns` 含低于目标精度的残余，返回
///   [`ErrorKind::Invalid`](crate::ErrorKind::Invalid)。
/// - [`PrecisionLossPolicy::ExplicitTruncate`]：向 0 截断到目标精度。
pub fn unix_ns_to_stored(
    unix_ns: i64,
    target: TimePrecision,
    policy: PrecisionLossPolicy,
) -> XResult<i64> {
    let scale = target.nanos_per_unit();
    if scale == 1 {
        return Ok(unix_ns);
    }
    let remainder = unix_ns % scale;
    if remainder != 0 {
        match policy {
            PrecisionLossPolicy::Reject => {
                return Err(XError::invalid(format!(
                    "时间精度损失被拒绝：unix_ns={unix_ns} 无法无损落入 {target:?}（scale={scale}）"
                )));
            }
            PrecisionLossPolicy::ExplicitTruncate => {}
        }
    }
    Ok(unix_ns / scale)
}

/// 将后端存储单位数值还原为 Unix 纳秒（精确乘；溢出返回 Invalid）。
pub fn stored_to_unix_ns(stored: i64, source: TimePrecision) -> XResult<i64> {
    let scale = source.nanos_per_unit();
    if scale == 1 {
        return Ok(stored);
    }
    stored
        .checked_mul(scale)
        .ok_or_else(|| XError::invalid(format!("存储时间乘 scale={scale} 溢出：stored={stored}")))
}

/// 按策略把 Unix 纳秒量化到目标精度后再还原为纳秒（round-trip 量化）。
pub fn quantize_unix_ns(
    unix_ns: i64,
    target: TimePrecision,
    policy: PrecisionLossPolicy,
) -> XResult<i64> {
    let stored = unix_ns_to_stored(unix_ns, target, policy)?;
    stored_to_unix_ns(stored, target)
}

/// `required-lossless` profile：后端必须能无损保存 Unix ns。
///
/// 不满足时返回 Invalid（调用方应拒绝 profile 或改走 BIGINT 辅助列路径）。
pub fn require_lossless_unix_ns(caps: TimeStorageCapabilities) -> XResult<()> {
    if caps.lossless_unix_ns {
        Ok(())
    } else {
        Err(XError::invalid(format!(
            "required-lossless profile 被拒绝：stored_precision={:?}, lossless_unix_ns=false",
            caps.stored_precision
        )))
    }
}

/// PostgreSQL 双列一致性：`event_time_ns`（BIGINT 真相源）与
/// `event_time_utc` 投影（按后端 TIMESTAMPTZ 精度，通常为微秒）必须一致。
///
/// `timestamptz_as_unix_ns` 为从 TIMESTAMPTZ 读回并换算成纳秒的值。
pub fn verify_pg_dual_column(event_time_ns: i64, timestamptz_as_unix_ns: i64) -> XResult<()> {
    // PostgreSQL `timestamptz` 原生精度为微秒。
    let pg_prec = TimePrecision::Microseconds;
    let expected_proj =
        quantize_unix_ns(event_time_ns, pg_prec, PrecisionLossPolicy::ExplicitTruncate)?;
    let actual_proj =
        quantize_unix_ns(timestamptz_as_unix_ns, pg_prec, PrecisionLossPolicy::ExplicitTruncate)?;
    if expected_proj != actual_proj {
        return Err(XError::invalid(format!(
            "PostgreSQL 双列不一致：event_time_ns={event_time_ns} 投影={expected_proj}，\
             event_time_utc 投影={actual_proj}"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject_default_policy() {
        assert_eq!(PrecisionLossPolicy::default(), PrecisionLossPolicy::Reject);
    }

    #[test]
    fn ns_round_trip_lossless() {
        let ns = 1_700_000_000_123_456_789_i64;
        let stored = unix_ns_to_stored(ns, TimePrecision::Nanoseconds, PrecisionLossPolicy::Reject)
            .expect("ns");
        assert_eq!(stored, ns);
        assert_eq!(stored_to_unix_ns(stored, TimePrecision::Nanoseconds).unwrap(), ns);
    }

    #[test]
    fn reject_sub_ms_bits() {
        let ns = 1_500_000_001_i64; // 1.5s + 1ns
        let err = unix_ns_to_stored(ns, TimePrecision::Milliseconds, PrecisionLossPolicy::Reject)
            .unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::Invalid);
        assert!(err.context().contains("精度损失"));
    }

    #[test]
    fn explicit_truncate_ms() {
        let ns = 1_500_000_001_i64;
        let stored = unix_ns_to_stored(
            ns,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitTruncate,
        )
        .unwrap();
        assert_eq!(stored, 1500);
        assert_eq!(stored_to_unix_ns(stored, TimePrecision::Milliseconds).unwrap(), 1_500_000_000);
    }

    #[test]
    fn aligned_ms_accepted_under_reject() {
        let ns = 1_500_000_000_i64;
        let stored =
            unix_ns_to_stored(ns, TimePrecision::Milliseconds, PrecisionLossPolicy::Reject)
                .unwrap();
        assert_eq!(stored, 1500);
    }

    #[test]
    fn required_lossless_gate() {
        let ok = TimeStorageCapabilities::new(TimePrecision::Nanoseconds, true, true);
        require_lossless_unix_ns(ok).unwrap();
        let bad = TimeStorageCapabilities::new(TimePrecision::Milliseconds, true, false);
        let err = require_lossless_unix_ns(bad).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::Invalid);
    }

    #[test]
    fn pg_dual_column_ok_when_projection_matches_us() {
        let ns = 1_700_000_000_123_456_789_i64;
        // TIMESTAMPTZ 只能保 us：123_456_789 → 123_456_000
        let tz_ns = 1_700_000_000_123_456_000_i64;
        verify_pg_dual_column(ns, tz_ns).unwrap();
    }

    #[test]
    fn pg_dual_column_rejects_mismatch() {
        let ns = 1_700_000_000_123_456_789_i64;
        let wrong = 1_700_000_000_999_999_000_i64;
        let err = verify_pg_dual_column(ns, wrong).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::Invalid);
    }

    #[test]
    fn stored_to_unix_ns_seconds_max_overflows_as_invalid() {
        let err = stored_to_unix_ns(i64::MAX, TimePrecision::Seconds).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::Invalid);
    }

    #[test]
    fn negative_pre_epoch_reject_and_truncate() {
        let ns = -1_500_000_001_i64;
        assert!(
            unix_ns_to_stored(ns, TimePrecision::Milliseconds, PrecisionLossPolicy::Reject)
                .is_err()
        );
        let stored = unix_ns_to_stored(
            ns,
            TimePrecision::Milliseconds,
            PrecisionLossPolicy::ExplicitTruncate,
        )
        .unwrap();
        // 向 0 截断：-1500000001 / 1_000_000 = -1500
        assert_eq!(stored, -1500);
    }
}
