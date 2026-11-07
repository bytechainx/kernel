# 残余项评估与升级触发条件（KERNEL-PROD-001）

- 绑定提交：`b197591ab88237353f8b26c01aa77f6f1d5fd860`（`main`）
- 日期：2026-11-07
- 状态：裁决记录。本文不新增 Gate、不改 CI、不改元仓台账；每项给出「维持现状」的规则依据与未来升级的触发条件。
- 背景：13 项任务已全部合入 `main`，Owner 已批准（`RELEASE-DECISION.md`），5 个 Required Checks 生效，合并后 main CI 全绿；Milestone #1 与全部 Issue 已关闭。

## 依据

| 来源 | 条款 |
| --- | --- |
| 执行计划 `KERNEL-PROD-001` §8.2 | 「若后续发现这些任务对安全或正确性至关重要，应升级为 Required Gate。」升级是条件性的 |
| 执行计划 §11（KERNEL-011 验收） | 「只有稳定、可复现的基准才用于阻塞 Gate」「噪声和环境漂移不会造成大量误报」 |
| 执行计划 §12 | 「只有反复出现、影响面明确且能自动判定的风险，才升级为常驻自动化规则」 |
| `SUPPLY-CHAIN.md`「安全公告」 | CI 暂不安装 `cargo-audit`：runner 未预装，安装会增加 CI 分钟数并引入仓库未使用的工具流程 |
| 元仓 `specs/repos/type/kernel/AGENTS.md` §2 | 「新正文只进 `goal.md` · `specs/` · `adr/` · `context/`。迁入战役层按历史读。不把 COMPLETE 写成 GO。」 |
| 工作区 `AGENTS.md` | 反过度设计；按仓库现有流程走，不引入仓库没在用的工具或流程 |

## 逐项裁决

| 残余项 | 裁决 | 规则依据 | 升级触发条件 |
| --- | --- | --- | --- |
| `cargo-audit` 进 CI | 维持观察项 | `SUPPLY-CHAIN.md` 已定策略；计划 §8.2 / §12 要求「至关重要／反复出现」才升级；本机 `cargo audit` 已实测 rc=0（1422 公告 / 77 依赖） | 连续两个迭代周期出现高危公告且本机命令遗漏，或仓库统一工具链在 runner 预装 `cargo-audit` |
| 性能阈值升级为阻塞 Gate | 维持观察项 | 计划 §11：只有稳定、可复现且噪声不误报才用于阻塞；当前仅单机 5 次采样（median 34570ns、cv 0.0304），跨环境未验证 | ≥3 个不同环境／工具链各 5 次采样，变异系数 < 0.2 且阈值误报率有记录 |
| 跨环境基准复现 | 未执行 | 本机只有一台机器，无法构成跨环境样本 | 有第二台可用 runner／环境时按 `bench-baseline.mjs` 采样并比较 |
| 真实 CI 失败注入 | 未执行 | 会污染 PR / `main` 历史；计划 §12 要求不为单次故障增加长期复杂度。本地已用模拟 `0 tests` / `FAILED` 输入验证 CI 断言返回非 0 | 在专用测试分支或临时仓库做，不污染 `main` 历史 |
| 元仓 `validation/gate.md` 台账口径 | 维持现状 | 叶 `AGENTS.md` §2 规定新正文只进 `goal.md` / `specs/` / `adr/` / `context/`；`validation/` 属迁入战役层按历史读，且「不把 COMPLETE 写成 GO」。仓内 `RELEASE-DECISION.md` 已注明以 Owner 批准栏为准 | 由 Owner 发起元仓规范变更 PR，在允许路径（`context/`）新增批准记录；不得直接改写历史台账字段 |

## 结论

本轮「授权继续」不改变任何 Gate、CI 配置或元仓台账：以上五项均有现行规则依据，属于**有依据的裁决**而非遗漏。机器全绿不产生合入权；升级任一 Gate 或改元仓台账都须走对应仓库的 PR 与审批流程。

## 引用

- `docs/kernel-prod-001/RELEASE-DECISION.md`（Owner 批准栏、残余项表）
- `docs/kernel-prod-001/SUPPLY-CHAIN.md`（安全公告与阻塞策略）
- `docs/kernel-prod-001/bench-baseline.json`（采样与环境）
- 元仓 `specs/repos/type/kernel/AGENTS.md` §2
