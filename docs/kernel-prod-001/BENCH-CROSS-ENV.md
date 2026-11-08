# 跨环境基准复现（KERNEL-PROD-001 残余项）

- 绑定提交：`a48e4e1a8b73dd55ae5b3e249a10e1ee9e8bfb07`（`main`）
- 日期：2026-11-08
- 状态：残余项「跨环境基准复现」已执行。本文只记录读数与判定，不新增 Gate、不改 CI 配置、不改元仓台账。
- 被测对象：`benches/hot_path.rs`。采样工具 `scripts/bench-baseline.mjs`，内部执行 `cargo bench --bench hot_path -- --quick`。

## 1. 读数

### 1.1 本机环境

环境：`linux 7.0.0-38-generic` / `x64` / `Intel(R) Xeon(R) W-10885M CPU @ 2.40GHz` / 16 核。工具链 `rustc 1.99.0 (b940084d7 2026-09-28)`、`cargo 1.99.0 (5f94df478 2026-08-27)`。

| 采样批次 | 命令 | 样本（ns） | min | median | max | 变异系数 |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-11-07（`bench-baseline.json` 现值） | `node scripts/bench-baseline.mjs 5` | 35704、34630、33903、32537、34570 | 32537 | 34570 | 35704 | 0.0304 |
| 2026-11-08（本轮复验） | `node scripts/bench-baseline.mjs 5` | 33514、33033、32886、30748、31611 | 30748 | 32886 | 33514 | 0.0316 |

同一环境两次采样的中位数相差 4.9%。本轮复验读数不写回 `bench-baseline.json`：该文件是受版本管理的机器产物，本 PR 不改动它。

### 1.2 CI 环境（GitHub Actions hosted runner）

作业：`Production Readiness` → `supply chain, evidence and bench`，命令 `node scripts/bench-baseline.mjs 3`。runner 镜像均为 `ubuntu-26.04`，region 每次不同。

| CI Run | 触发 | 提交 | runner region | 样本数 | median（ns） | 变异系数 |
| --- | --- | --- | --- | --- | --- | --- |
| `42323367443` | pull_request | `ad37239` | centralus | 3 | 21021 | 0.0013 |
| `42351667447` | push | `892c812` | northcentralus | 3 | 34157 | 0.0164 |
| `42353102648` | pull_request | `b9b303e` | centralus | 3 | 19960 | 0.0101 |
| `42353673812` | push | `b197591` | southcentralus | 3 | 9073 | 0.0142 |
| `42354823359` | pull_request | `2f137d1` | westus3 | 3 | 8933 | 0.0598 |
| `42361168910` | push | `a48e4e1` | eastus | 3 | 10139 | 0.8413 |

6 次 Run 的 median 落在 8933–34157 ns，跨度 3.82 倍。

## 2. 判定

1. 跨环境中位数不可直接比较。同一份 `hot_path` 基准在 hosted runner 上的中位数最低 8933 ns、最高 34157 ns；本机两次中位数 34570 ns 与 32886 ns 落在该区间高端，并与 `northcentralus` 那次（34157 ns）接近。「CI 比本机快固定倍数」这类单次结论不成立。
2. 环境之间的差异大于环境内部的采样噪声。除 `42361168910` 外，单次 Run 内的变异系数为 0.0013–0.0598；环境之间的中位数跨度为 3.82 倍。
3. 单次 Run 内的变异系数可以失稳。`42361168910`（`eastus`，3 样本）变异系数 0.8413，已超过 `noise_tolerance` 的 0.2 门槛。按「变异系数 < 0.2」自动升级阻塞 Gate，会在 hosted runner 上随机误报。
4. 结论：性能回归阈值必须按环境分别绑定基线（同一 runner 类型的连续读数序列），不得使用跨环境统一阈值。`RESIDUAL-ITEMS.md` 中「性能阈值升级为阻塞 Gate」的触发条件（≥3 个环境／工具链各 5 次采样且变异系数 < 0.2）仍未满足：本组 CI 每个环境只有 3 次采样，且有一次变异系数 0.8413。该残余项维持观察项。

## 3. 已知限制

- CI 每次 Run 把 `bench-baseline.json` 写进 checkout 后丢弃，不归档为 artifact。跨环境读数只能从 Job 日志取。按环境归档基线需要先增加 artifact 上传，属于新增 CI 复杂度，本文不实施。
- CI 每环境 3 次采样，本机 5 次，采样次数口径不一致。提高 CI 采样次数直接增加 CI 分钟数，本轮不改。
- `--quick` 的 criterion 内部样本数由 criterion 决定，本文不改基准配置。
- 本文 CI 读数来自 Job 日志文本匹配（`bench baseline: N 次，median=<值>ns，cv=<值>`），不是归档文件；日志保留期外需重跑取得。

## 引用

- `scripts/bench-baseline.mjs`（采样与统计、`threshold_policy`、`noise_tolerance`）
- `docs/kernel-prod-001/bench-baseline.json`（本机 2026-11-07 读数）
- `docs/kernel-prod-001/RESIDUAL-ITEMS.md`（残余项裁决与升级触发条件）
- CI：`Production Readiness` workflow 的 Run `42323367443`、`42351667447`、`42353102648`、`42353673812`、`42354823359`、`42361168910`
