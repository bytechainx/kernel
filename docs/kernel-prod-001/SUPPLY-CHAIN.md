# 依赖与许可证策略（KERNEL-010）

## 范围

- 生产依赖：`thiserror 2`；开发依赖：`proptest 1`、`serde 1`、`static_assertions 1`；条件依赖：`loom 0.7`（仅 `cfg(loom)`）。
- `publish = false`，本仓不执行 `cargo publish`；策略约束 Git 依赖消费者与 CI 可复现性。

## 允许与拒绝

- 允许：`MIT`、`Apache-2.0`、`BSD-2-Clause`、`BSD-3-Clause`、`ISC`、`Zlib`、`Unicode-3.0`、`Unicode-DFS-2016`、`CC0-1.0`、`0BSD`、`MPL-2.0`。
- 拒绝：GPL 系列、AGPL 系列、SSPL、许可证字段缺失或无法识别的表达式。
- 判定数据源为 `cargo metadata` 的 `license` 字段；`scripts/check-licenses.mjs` 在 CI 中校验，命中拒绝项即退出码 1。
- 按 SPDX 表达式语义判定：`OR` 任选其一，只要存在一个全部组件都在允许列表的选项即合规；`AND` / `WITH` 组合要求全部组件允许。本轮 `r-efi` 的 `LGPL-2.1-or-later` 选项未被选用，按 `MIT` 选项合规；`Unlicense`、`LLVM-exception` 只出现在含 `MIT` / `Apache-2.0` 的 `OR` 表达式中。
- 依赖清单由 `cargo tree --depth 1` 生成，归档在 `docs/kernel-prod-001/dependency-inventory.txt`。

## 安全公告

- 本机已装 `cargo-audit 0.22.2`，可重复命令：`cargo audit`（需联网取 advisory DB）。
- CI 暂不安装 `cargo-audit`：runner 未预装，安装会增加 CI 分钟数并引入仓库未使用的工具流程。安全公告检查登记为可重复的本机命令，结果写入 Evidence；待仓库统一工具链后再升级为 PR Gate。
- 高危公告：先登记例外或升级依赖；工具误报须在 PR 描述记录理由与有效期，不无条件忽略，也不因误报阻塞所有更新。

## 阻塞策略

| 情形 | 处置 |
| --- | --- |
| 许可证缺失或命中拒绝项 | CI 检查器退出码 1，阻塞 |
| 安全公告高危且非误报 | 人工确认后阻塞 |
| 工具不可用或离线 | 记为环境等待，不伪装为通过 |

## 例外台账

| 项 | Owner | 原因 | 到期时间 | 状态 |
| --- | --- | --- | --- | --- |
| 无 | — | 本轮未发现需豁免项 | — | — |

例外须同时写明 Owner、原因与到期时间；到期未复核视为失效。
