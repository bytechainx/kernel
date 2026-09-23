#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! `kernel` 标准章节对应的可执行规格检查。
//!
//! // SPEC-MAP: K-1 | 职责 | assert_duties
//! // SPEC-MAP: K-2 | 范围 | assert_scope
//! // SPEC-MAP: K-3 | 兼容要求 | assert_compatibility

const STANDARD: &str = include_str!("../docs/标准.md");

#[test]
fn assert_duties() {
    assert!(STANDARD.contains("## 职责"));
    assert!(STANDARD.contains("L0"));
}

#[test]
fn assert_scope() {
    assert!(STANDARD.contains("## 范围"));
    assert!(STANDARD.contains("不承担配置"));
}

#[test]
fn assert_compatibility() {
    assert!(STANDARD.contains("## 兼容要求"));
    assert!(STANDARD.contains("UnixTimeNs"));
}
