# KERNEL-PROD-001 基线清单（KERNEL-001）

- 绑定提交：`2b4e8422e82914562355f1912a0c4d26089c252f`（`origin/main`）
- 采集时间：2026-11-07
- 采集方式：只读核对工作树、`Cargo.toml`、`cargo metadata`、测试清单与 CI；未修改源码
- 状态：本文是基线记录，不表示其余任务已通过验收

## 1. 工具链与版本

| 项 | 值 | 来源 |
| --- | --- | --- |
| crate / version | `kernel 0.5.0` | `Cargo.toml` |
| edition / MSRV | `2024` / `1.88` | `Cargo.toml` |
| license / 发布 | `MIT` / `publish = false` | `Cargo.toml`、`LICENSE` |
| 本机工具链 | `rustc 1.99.0`、`cargo 1.99.0`；另装 `1.88.0` | `rustup toolchain list` |
| 生产依赖 | `thiserror 2` | `cargo tree --depth 1` |
| 开发依赖 | `proptest 1`、`serde 1`、`static_assertions 1` | 同上 |
| 条件依赖 | `loom 0.7`（`cfg(loom)`） | `Cargo.toml` |

## 2. 契约与公开面正本

- 语义正本：`docs/标准.md`；公开面清单：`docs/API.md`；变更记录：`CHANGELOG.md`。
- `src/lib.rs` 导出 `error` / `lifecycle` 两个模块与 9 个根 re-export 符号；`#[non_exhaustive]` 用于 `ErrorKind`、`LifecycleError`、`WaitTimeoutError`。
- 编译期 API 驱动：`tests/public_api_surface.rs`（3 个测试）。

## 3. 现有自动化与测试载体

- 按 `origin/main` 盘点：`.github/workflows/ci.yml` 仅 `gate` Job（`fmt / clippy / test`）。远端 `main` 的 Required Checks 已由 KERNEL-006 的 PR #7 扩展为 `fmt / clippy / test` 与 `loom concurrency verification`，但 PR #7 尚未合入 `main`，故基线仍记为单 Job。
- 同源代码基线的 KERNEL-006 回归实测 `cargo test` 为 74 passed / 0 failed；`tests/lifecycle_concurrency_loom.rs` 在默认路径为 `running 0 tests`，须由 `RUSTFLAGS="--cfg loom"` 单独运行。
- 契约载体：`contracts.rs`（状态转换矩阵）、`tdd_contracts.rs`、`aidd_boundary.rs`、`sdd_spec.rs`、`public_api.rs`、`lifecycle_concurrency.rs`（并发、超时、多观察者）、`src/lifecycle.rs` 单元测试（poison 恢复、deadline overflow）。
- 基准：`benches/hot_path.rs`（`--quick` 2,000 次 / 默认 200,000 次，自定义 harness）。
- 打包：`cargo package --no-verify` 实测退出码 0（29 文件）。

## 4. 12 项任务差距（按等价载体盘点）

| 任务 | 已有等价载体 | 真实缺口 | 本轮处置 |
| --- | --- | --- | --- |
| 001 基线清单 | 上述文档与元数据 | 无绑定 commit 的汇总 | 本文 |
| 002 API 契约 | API.md、public_api_surface.rs、non_exhaustive | 缺 0.x 兼容政策与机器可读基线 | 补 API.md 章节、api-baseline.json、检查器 |
| 003 Error 契约 | tdd_contracts、aidd_boundary、public_api | 无 | 盘点核对，不重造 |
| 004 Lifecycle 契约 | contracts.rs 状态矩阵、aidd_boundary | 无 | 盘点核对，不重造 |
| 005 Shutdown 契约 | lifecycle_concurrency、poison/timeout 单测、KERNEL-006 Loom 模型 | 无 | 盘点核对，不重造 |
| 007 MSRV | 声明 1.88、本机 1.88 工具链 | 未实测、CI 无 Job | 实测 + 独立 workflow Job |
| 008 API 兼容 | public_api_surface.rs | 无机器可读基线／差异检测 | 基线 + 检查器 + CI |
| 009 Consumer | examples/kernel_basic.rs（仓内示例） | 无独立 crate | 新建 fixtures/consumer |
| 010 供应链 | MIT LICENSE、本机 cargo-deny/cargo-audit | 无策略、例外台账、可重复检查 | 策略文档 + 依赖清单 + 许可证检查器 |
| 011 基准 | hot_path.rs | 无多次采样、统计口径、阈值 | bench-baseline 脚本与基线文档 |
| 012 Evidence Gate | KERNEL-006 的 evidence.json（在 PR #7 分支） | main 上无格式规范／检查器 | 格式规范 + 检查器 + 汇总证据 |
| 013 发布裁决 | 元仓 validation/gate.md（INTERNAL_RELEASE_CANDIDATE） | Owner 批准不可代行 | 只产出待批报告 |

## 5. 受影响正本与同步结果

按工作区 `docs/README.md`「文档与实现同步」第 2、4 条核对：`README.md`、`CONTRIBUTING.md`、`docs/标准.md`、`docs/API.md`、`CHANGELOG.md`、`checklist.md`。本轮改 `docs/API.md`（补 0.x 兼容政策），新增 `docs/kernel-prod-001/`、`fixtures/consumer/`、`scripts/*.mjs`、`.github/workflows/production-readiness.yml`。未改 `CHANGELOG.md`（未改版本、公开 API 或对外行为，变更记入 PR 描述）；未改 `README.md` / `CONTRIBUTING.md` / `checklist.md`（其描述的事实仍准确，`checklist.md` 自带「以 ci.yml 为准」声明，新增独立 workflow 不使其失实）。
