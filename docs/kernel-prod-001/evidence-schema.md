# Evidence 统一格式（KERNEL-012）

## 路径

```text
docs/kernel-prod-001/<GOAL>/<TASK>/<commit-sha>/
  evidence.json
  <日志>.txt
  review.md
```

pre-commit 钩子把 `evidence` 当作独立 crate 目录、把 `.log` 当作产物路径，故首段用 `docs/kernel-prod-001`，日志后缀用 `.txt`；不得用 `--no-verify` 绕过。

## 必填字段

| 字段 | 类型 | 判定 |
| --- | --- | --- |
| `goal_id` / `task_id` / `repository` | 字符串 | 必填 |
| `commit_sha` | 40 位小写 hex | 绑定被验证提交 |
| `toolchain` | 对象 | 至少含 `rustc` / `cargo` 版本 |
| `checks` | 非空数组 | 每项含 `name`、`command`、`exit_code = 0`；带 `tests` 时须 `total = passed > 0` 且 `failed = 0` |
| `result` | `PASS` / `PASS_LOCAL` / 其它 | `PASS` 必须有可访问的 `ci_run_url`；`PASS_LOCAL` 只表示本地验证通过 |
| `timestamp_utc` | ISO-8601 | 本地验证起始时间 |

## Gate 判定

`scripts/check-evidence.mjs` 扫描 `docs/kernel-prod-001/**/evidence.json`：字段缺失、SHA 格式错误、检查为空、退出码非 0、测试数不守恒或为 0、`PASS` 缺 CI Run URL，任一命中即退出码 1。目录下没有任何证据文件同样判定失败。CI 的 `production-readiness` workflow 运行该检查器，缺失即 Gate 失败。

## 真实值约束

不得复制计划示例中的测试数量、退出码或耗时；数值必须来自真实日志。`PASS_LOCAL` 不得写成 `PASS`，Owner 批准不得由证据文件代签。

## 已有实例

- KERNEL-006：`docs/kernel-prod-001/KERNEL-006/c1cebb825982d57c203a37176606d457541795ac/evidence.json`（由 PR #7 引入）。
- 本轮 12 项汇总：`docs/kernel-prod-001/KERNEL-PROD-001/<commit-sha>/evidence.json`。
