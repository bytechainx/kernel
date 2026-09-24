# 变更记录

本仓库从独立 Git 源码版本 `0.4.2` 开始维护，不发布到 crates.io。

## [Unreleased]

- 将本体角色登记为 `core`，与基础值仓的 008 名册一致；公开 API 不变。

## [0.4.2] - 2026-09-23 — bytechainx 初始迁移

### 说明

- 从 `xhyper.rs` 迁移 kernel 实现、测试、示例与基准代码，保留 crate 版本和 Rust API。
- 为独立仓库补充本地 Cargo manifest、文档、许可证与 CI。
