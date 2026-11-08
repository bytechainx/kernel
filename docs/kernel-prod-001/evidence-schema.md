# Evidence 统一格式（KERNEL-012）

## 路径

```text
docs/kernel-prod-001/<TASK>/<commit-sha>/            # 单任务，例如 KERNEL-006
  evidence.json
  <日志>.txt
  review.md

docs/kernel-prod-001/<GOAL>/<commit-sha>/            # 汇总，例如 KERNEL-PROD-001
  evidence.json
```

pre-commit 钩子把 `evidence` 当作独立 crate 目录、把 `.log` 当作产物路径，故首段用 `docs/kernel-prod-001`，日志后缀用 `.txt`；不得用 `--no-verify` 绕过。

## 必填字段

| 字段 | 类型 | 判定 |
| --- | --- | --- |
| `goal_id` / `task_id` / `repository` | 字符串 | 必填 |
| `commit_sha` | 40 位小写 hex | 绑定被验证提交；Gate 另核验：该提交存在于当前仓对象库、是当前 HEAD 的祖先，且父目录名与其一致 |
| `toolchain` | 对象 | 至少含 `rustc` / `cargo` 版本 |
| `checks` | 非空数组 | 每项含 `name`、`command`、`exit_code = 0`；带 `tests` 时须 `total = passed > 0` 且 `failed = 0` |
| `result` | `PASS` / `PASS_LOCAL` / 其它 | `PASS` 必须有匹配 `https://github.com/bytechainx/kernel/actions/runs/<run-id>` 的 `ci_run_url`（可带 `/job/<job-id>`）；`PASS_LOCAL` 只表示本地验证通过，`ci_run_url` 可为空 |
| `timestamp_utc` | ISO-8601 | 本地验证起始时间；须严格匹配 `YYYY-MM-DDTHH:MM:SS`（可带小数秒，须带 `Z` 或 `±HH:MM`）且可被解析 |

## Gate 判定

`scripts/check-evidence.mjs` 扫描 `docs/kernel-prod-001/**/evidence.json`：字段缺失、SHA 格式错误、检查为空、退出码非 0、测试数不守恒或为 0、`PASS` 缺 CI Run URL，任一命中即退出码 1。目录下没有任何证据文件同样判定失败。

真实性校验（2026-11-08 增补）：`commit_sha` 必须能在当前仓对象库中解析且为当前 HEAD 的祖先；`evidence.json` 所在目录名必须等于 `commit_sha`；`timestamp_utc` 必须严格可解析；`PASS` 的 `ci_run_url` 必须指向本仓 Actions Run。Git 历史核验依赖 `production-readiness` workflow 对 `actions/checkout@v4` 设置 `fetch-depth: 0`（完整历史）；git 本身不可用或非仓库判为工具错误，退出码 2。CI 的 `production-readiness` workflow 运行该检查器，缺失即 Gate 失败。

## 真实值约束

不得复制计划示例中的测试数量、退出码或耗时；数值必须来自真实日志。`PASS_LOCAL` 不得写成 `PASS`，Owner 批准不得由证据文件代签。

## 已有实例

- KERNEL-006：`docs/kernel-prod-001/KERNEL-006/c1cebb825982d57c203a37176606d457541795ac/evidence.json`（由 PR #7 引入）。
- 本轮 12 项汇总：`docs/kernel-prod-001/KERNEL-PROD-001/<commit-sha>/evidence.json`。
