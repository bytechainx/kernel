//! 绝对时刻基础类型 [`UnixTimeNs`] 与错误 [`TimeError`]（XH-TIME-MODEL-SPEC-001 §2）。
//!
//! 设计原则（SPEC §1.1 / §2.2）：
//!
//! - 内部绝对时刻统一为 UTC POSIX epoch 纳秒（TIME-INV-005）；
//! - 不提供 `From<i64>` / `Into<i64>` / `new` / `from_timestamp`，调用点必须明确单位
//!   （TIME-INV-001 / SPEC §2.2）；
//! - seconds/millis/micros → ns 一律 checked 乘法（TIME-INV-017 / SPEC §2.3）；
//! - `duration_since` 对反向顺序返回错误，禁止饱和（SPEC §2.3）；
//! - 负 epoch 支持；负数拆分使用 Euclidean division，禁止直接 `%` 造成负余数错误
//!   （SPEC §2.3）；
//! - leap second 不单独编码，采用 POSIX time 语义（SPEC §2.3）。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 单位转换 / 时间顺序错误（SPEC §2.4）。
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TimeError {
    /// 时间单位转换溢出（超出 `i64` 纳秒可表示域）。
    #[error("时间单位转换溢出")]
    Overflow,

    /// 时间顺序非法（`later` 早于 `earlier`，`duration_since` 反向调用）。
    #[error("时间顺序非法: later={later:?}, earlier={earlier:?}")]
    InvalidOrder {
        /// 较晚时刻。
        later: UnixTimeNs,
        /// 较早时刻。
        earlier: UnixTimeNs,
    },

    /// `SystemTime` 超出 `i64` 纳秒可表示范围（含 epoch 之前）。
    #[error("系统时间超出支持范围")]
    SystemTimeOutOfRange,
}

/// 绝对时刻（UTC POSIX epoch 纳秒）。
///
/// 全系统唯一的绝对时刻基础 primitive（SPEC §3.1 / TIME-INV-005）。内部为 `i64` 纳秒，
/// 支持 epoch 前后。不实现 `Default`，禁止用零值冒充有效时间。
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnixTimeNs(i64);

impl UnixTimeNs {
    /// Unix epoch（1970-01-01T00:00:00Z）。
    pub const UNIX_EPOCH: Self = Self(0);

    /// 从纳秒值构造（唯一无检查构造入口；仅限协议转换、fixture 与 testkit）。
    ///
    /// 调用点必须明确输入单位为纳秒；禁止经泛型 `From<i64>` 静默转换
    /// （SPEC §2.2 / TIME-INV-001）。
    pub const fn from_unix_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    /// 返回内部纳秒值。
    pub const fn as_unix_nanos(self) -> i64 {
        self.0
    }

    /// 从 Unix 微秒构造（checked 乘法；溢出返回 [`TimeError::Overflow`]）。
    ///
    /// 调用点必须明确输入单位为微秒（TIME-INV-017）。
    pub fn try_from_unix_micros(micros: i64) -> Result<Self, TimeError> {
        let nanos = i128::from(micros).checked_mul(1_000).ok_or(TimeError::Overflow)?;
        let nanos_i64: i64 = nanos.try_into().map_err(|_| TimeError::Overflow)?;
        Ok(Self(nanos_i64))
    }

    /// 从 Unix 毫秒构造（checked 乘法；溢出返回 [`TimeError::Overflow`]）。
    ///
    /// 调用点必须明确输入单位为毫秒（TIME-INV-017）。
    pub fn try_from_unix_millis(millis: i64) -> Result<Self, TimeError> {
        let nanos = i128::from(millis).checked_mul(1_000_000).ok_or(TimeError::Overflow)?;
        let nanos_i64: i64 = nanos.try_into().map_err(|_| TimeError::Overflow)?;
        Ok(Self(nanos_i64))
    }

    /// 从 Unix 秒构造（checked 乘法；溢出返回 [`TimeError::Overflow`]）。
    ///
    /// 调用点必须明确输入单位为秒（TIME-INV-017）。
    pub fn try_from_unix_seconds(seconds: i64) -> Result<Self, TimeError> {
        let nanos = i128::from(seconds).checked_mul(1_000_000_000).ok_or(TimeError::Overflow)?;
        let nanos_i64: i64 = nanos.try_into().map_err(|_| TimeError::Overflow)?;
        Ok(Self(nanos_i64))
    }

    /// 安全加法（溢出返回错误；禁止 wrap）。
    pub fn checked_add(self, duration: Duration) -> Result<Self, TimeError> {
        let nanos = i128::from(self.0);
        let dur_nanos: i128 = duration.as_nanos().try_into().map_err(|_| TimeError::Overflow)?;
        let result = nanos.checked_add(dur_nanos).ok_or(TimeError::Overflow)?;
        let result_i64: i64 = result.try_into().map_err(|_| TimeError::Overflow)?;
        Ok(Self(result_i64))
    }

    /// 安全减法（溢出返回错误；禁止 wrap）。
    pub fn checked_sub(self, duration: Duration) -> Result<Self, TimeError> {
        let nanos = i128::from(self.0);
        let dur_nanos: i128 = duration.as_nanos().try_into().map_err(|_| TimeError::Overflow)?;
        let result = nanos.checked_sub(dur_nanos).ok_or(TimeError::Overflow)?;
        let result_i64: i64 = result.try_into().map_err(|_| TimeError::Overflow)?;
        Ok(Self(result_i64))
    }

    /// 返回 `self` 与 `earlier` 的持续时间差。
    ///
    /// `self < earlier` 时返回 [`TimeError::InvalidOrder`]，禁止饱和为 `Duration::ZERO`
    /// （SPEC §2.3）。
    pub fn duration_since(self, earlier: Self) -> Result<Duration, TimeError> {
        if self.0 < earlier.0 {
            return Err(TimeError::InvalidOrder { later: self, earlier });
        }
        let diff = i128::from(self.0) - i128::from(earlier.0);
        // 非负且最大为 u64::MAX（i64 全量程跨度）
        let nanos_u64: u64 = diff.try_into().map_err(|_| TimeError::Overflow)?;
        Ok(Duration::from_nanos(nanos_u64))
    }

    /// 从 `std::time::SystemTime` 构造。
    ///
    /// epoch 之前（`duration_since(UNIX_EPOCH)` 失败）或超出 `i64` 纳秒范围时返回
    /// [`TimeError::SystemTimeOutOfRange`]（SPEC §13.1：SystemTime before/after epoch）。
    pub fn try_from_system_time(value: SystemTime) -> Result<Self, TimeError> {
        match value.duration_since(UNIX_EPOCH) {
            Ok(d) => {
                let nanos: u128 = d.as_nanos();
                let nanos_i64: i64 =
                    nanos.try_into().map_err(|_| TimeError::SystemTimeOutOfRange)?;
                Ok(Self(nanos_i64))
            }
            Err(_) => Err(TimeError::SystemTimeOutOfRange),
        }
    }

    /// 转为 `std::time::SystemTime`。
    ///
    /// epoch 之前（负纳秒）无法用 `SystemTime` 表示，返回
    /// [`TimeError::SystemTimeOutOfRange`]（SPEC §13.1）。
    pub fn try_into_system_time(self) -> Result<SystemTime, TimeError> {
        if self.0 < 0 {
            return Err(TimeError::SystemTimeOutOfRange);
        }
        UNIX_EPOCH
            .checked_add(Duration::from_nanos(self.0 as u64))
            .ok_or(TimeError::SystemTimeOutOfRange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SPEC §13.1：Unix epoch 0。
    #[test]
    fn epoch_is_zero() {
        assert_eq!(UnixTimeNs::UNIX_EPOCH.as_unix_nanos(), 0);
    }

    // SPEC §13.1：正负 seconds/millis/micros 转 ns。
    #[test]
    fn positive_units_to_nanos() {
        assert_eq!(UnixTimeNs::try_from_unix_seconds(1).unwrap().as_unix_nanos(), 1_000_000_000);
        assert_eq!(UnixTimeNs::try_from_unix_millis(1).unwrap().as_unix_nanos(), 1_000_000);
        assert_eq!(UnixTimeNs::try_from_unix_micros(1).unwrap().as_unix_nanos(), 1_000);
    }

    #[test]
    fn negative_units_to_nanos() {
        assert_eq!(UnixTimeNs::try_from_unix_seconds(-1).unwrap().as_unix_nanos(), -1_000_000_000);
        assert_eq!(UnixTimeNs::try_from_unix_millis(-1).unwrap().as_unix_nanos(), -1_000_000);
        assert_eq!(UnixTimeNs::try_from_unix_micros(-1).unwrap().as_unix_nanos(), -1_000);
    }

    // SPEC §13.1：overflow 拒绝。
    #[test]
    fn overflow_rejected() {
        assert_eq!(UnixTimeNs::try_from_unix_seconds(i64::MAX), Err(TimeError::Overflow));
        assert_eq!(UnixTimeNs::try_from_unix_seconds(i64::MIN), Err(TimeError::Overflow));
        assert_eq!(UnixTimeNs::try_from_unix_millis(i64::MAX), Err(TimeError::Overflow));
        assert_eq!(UnixTimeNs::try_from_unix_micros(i64::MAX), Err(TimeError::Overflow));
    }

    // SPEC §13.1：safe range round-trip。
    #[test]
    fn safe_range_roundtrip() {
        for v in [0_i64, 1, -1, 1_700_000_000_123_456_789, -1_700_000_000_000_000_000] {
            let t = UnixTimeNs::from_unix_nanos(v);
            assert_eq!(t.as_unix_nanos(), v);
        }
    }

    // SPEC §13.1：checked_add / checked_sub。
    #[test]
    fn checked_add_sub_normal() {
        let t = UnixTimeNs::from_unix_nanos(100);
        assert_eq!(t.checked_add(Duration::from_nanos(50)).unwrap().as_unix_nanos(), 150);
        assert_eq!(t.checked_sub(Duration::from_nanos(50)).unwrap().as_unix_nanos(), 50);
    }

    #[test]
    fn checked_add_overflow() {
        let t = UnixTimeNs::from_unix_nanos(i64::MAX);
        assert_eq!(t.checked_add(Duration::from_nanos(1)), Err(TimeError::Overflow));
        let t = UnixTimeNs::from_unix_nanos(i64::MIN);
        assert_eq!(t.checked_sub(Duration::from_nanos(1)), Err(TimeError::Overflow));
    }

    // SPEC §13.1：reverse order duration_since 失败。
    #[test]
    fn duration_since_reverse_fails() {
        let earlier = UnixTimeNs::from_unix_nanos(10);
        let later = UnixTimeNs::from_unix_nanos(100);
        assert_eq!(
            earlier.duration_since(later),
            Err(TimeError::InvalidOrder { later: earlier, earlier: later })
        );
        assert_eq!(later.duration_since(earlier).unwrap(), Duration::from_nanos(90));
    }

    // SPEC §13.1：SystemTime before/after epoch。
    #[test]
    fn system_time_conversions() {
        assert_eq!(UnixTimeNs::try_from_system_time(UNIX_EPOCH).unwrap(), UnixTimeNs::UNIX_EPOCH);
        let after = UNIX_EPOCH + Duration::from_nanos(1234);
        assert_eq!(UnixTimeNs::try_from_system_time(after).unwrap().as_unix_nanos(), 1234);
        let before = UNIX_EPOCH.checked_sub(Duration::from_secs(1)).expect("epoch-1s");
        assert_eq!(UnixTimeNs::try_from_system_time(before), Err(TimeError::SystemTimeOutOfRange));
        // epoch 后超出 i64 纳秒域（约 292 年之后的墙钟）。10_000_000_000s * 1e9 ns > i64::MAX。
        let after_i64_ns = UNIX_EPOCH + Duration::from_secs(10_000_000_000);
        assert_eq!(
            UnixTimeNs::try_from_system_time(after_i64_ns),
            Err(TimeError::SystemTimeOutOfRange)
        );
        let t = UnixTimeNs::from_unix_nanos(1234);
        assert_eq!(t.try_into_system_time().unwrap(), UNIX_EPOCH + Duration::from_nanos(1234));
        assert_eq!(
            UnixTimeNs::from_unix_nanos(-1).try_into_system_time(),
            Err(TimeError::SystemTimeOutOfRange)
        );
    }

    // SPEC §13.2：单位构造器必须走具名入口；From<i64>/Into<i64> 由 tests/api_compile.rs 钉死。
    #[test]
    fn named_unit_constructors_do_not_infer_i64_unit() {
        let t = UnixTimeNs::try_from_unix_millis(42).expect("ms");
        assert_eq!(t.as_unix_nanos(), 42_000_000);
        assert_eq!(UnixTimeNs::try_from_unix_millis(i64::MIN), Err(TimeError::Overflow));
        assert_eq!(UnixTimeNs::try_from_unix_micros(i64::MIN), Err(TimeError::Overflow));
    }

    // SPEC §13.1：proptest — 安全范围内 millis round-trip 恒等。
    #[cfg(test)]
    mod proptest_roundtrip {
        use super::super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(64))]

            #[test]
            fn millis_roundtrip(millis: i64) {
                // 只对不会溢出纳秒 i64 的毫秒范围做恒等验证
                let nanos = i128::from(millis).checked_mul(1_000_000);
                if let Some(n) = nanos {
                    if let Ok(n_i64) = i64::try_from(n) {
                        let t = UnixTimeNs::try_from_unix_millis(millis).unwrap();
                        assert_eq!(t.as_unix_nanos(), n_i64);
                        let back = t.as_unix_nanos();
                        assert_eq!(UnixTimeNs::from_unix_nanos(back), t);
                    } else {
                        assert_eq!(UnixTimeNs::try_from_unix_millis(millis), Err(TimeError::Overflow));
                    }
                }
            }
        }
    }
}
