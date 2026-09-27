# 变更记录

本仓库从独立 Git 源码版本 `0.4.2` 开始维护，不发布到 crates.io。

## [Unreleased]

## [0.5.0] - 2026-09-27

### 变更

- **破坏性变更**：移除 `time`、`time_storage`、对应根导出和 `From<TimeError> for XError`。时间消费者直接使用独立 `temporal 0.1.0`，在自身边界显式转换错误；迁移入口见 [`docs/API.md`](docs/API.md#时间公开面迁移)。
- **破坏性变更**：移除 `verify_pg_dual_column`；PostgreSQL 适配器须基于 `temporal::verify_projection` 实现后端校验。
- 按错误与生命周期语义保留混合测试、示例和基准，时间专属测试从本仓移出；生产依赖没有新增。

### 说明

- 本候选的影响等级为 MAJOR；数值采用 temporal 迁移规格约定的 0.x 次版本 `0.5.0`，不代表已发布或完成最终发布审批。
- temporal 支持平台可表示的负 epoch、严格检查 `SystemTime` 精度、使用本地 `PrecisionError`，并提供显式向下取整与严格投影合同。调用方须验证这些行为变化，不能仅替换 import。
- `kernel` 保留错误分类和生命周期，`retry_after` 与关停等待仍使用标准库 `Duration`；与 `temporal` 互不依赖。

## [0.4.3] - 2026-09-27

- 将本体角色登记为 `kernel`，与 Feature 008 冻结名册一致；公开 API 不变。

## [0.4.2] - 2026-09-23 — bytechainx 初始迁移

### 说明

- 从 `xhyper.rs` 迁移 kernel 实现、测试、示例与基准代码，保留 crate 版本和 Rust API。
- 为独立仓库补充本地 Cargo manifest、文档、许可证与 CI。
