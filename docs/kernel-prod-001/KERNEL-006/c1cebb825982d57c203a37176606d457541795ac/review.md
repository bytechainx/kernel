# KERNEL-006 对抗性审查与覆盖结论

- 任务：KERNEL-006 — Loom 并发模型纳入 PR 阻塞式 CI
- Issue：[#6](https://github.com/bytechainx/kernel/issues/6)
- 分支：`issue/6-kernel-006-loom-ci`
- 被验证提交：`c1cebb825982d57c203a37176606d457541795ac`（实现提交；本证据目录作为后续提交加入）
- 工具链：`rustc 1.99.0 (b940084d7 2026-09-28)` / `cargo 1.99.0 (5f94df478 2026-08-27)`
- 本地验证时间：2026-11-07T00:44:21Z 起，四条回归合计 12 秒
- CI 验证时间：2026-11-07T00:51:32Z–00:52:32Z，PR #7 Run `42320761596`

## 1. T4 模型覆盖审查

现有三个模型全部真实运行，未为凑数量新增模型：

| 模型 | 覆盖的不变量 | 结果 |
| --- | --- | --- |
| `loom_trigger_wakes_waiter` | waiter 先进入等待、trigger 后发生；线程 join；触发标志断言；无 lost wake-up | 通过 |
| `loom_two_waiters` | 两个 waiter 同时等待；`notify_all` 后均被唤醒；计数断言；多观察者共享状态 | 通过 |
| `loom_trigger_before_wait_observer_sees_flag` | trigger 先发生、waiter 后进入；trigger 与等待关系的竞争窗口；克隆观察者均见标志 | 通过 |

状态写入与通知顺序由 `src/lifecycle.rs` 的 `trigger` 持同一 mutex 写标志再 `notify_all` 保证，模型通过 `is_triggered` 断言观察。三个模型线程数最多 2，`--release` 下测试执行 1.59 秒，状态空间受控。

**已知覆盖边界：** `wait_timeout` 与 `WaitTimeoutError::DeadlineOverflow` 不在 Loom 覆盖内。原因是 loom 的 `Condvar` 没有 `wait_timeout`，实现在 `#[cfg(not(loom))]` 下才提供该方法。该语义由标准路径单元测试与 `tests/lifecycle_concurrency.rs` 覆盖。按计划 T4「新增模型必须对应明确不变量或已识别风险，并控制状态空间」，不为该边界强行新增模型，也不把它写成已由 Loom 验证。

## 2. T7 对抗性审查

| 审查问题 | 结论 | 依据 |
| --- | --- | --- |
| Loom 测试 0 tests 但 CI 绿色？ | 已防住 | CI Job 的 `run` 对 `test result: ok. [1-9][0-9]* passed; 0 failed` 做 grep 断言，`0 passed` 与 `FAILED` 均使 Job 失败；本地对真实日志、模拟 0-tests、模拟 FAILED 三种输入自查通过 |
| 只有测试文件启用 cfg、实现仍走 std::sync？ | 否 | `src/lifecycle.rs:17-20` 在 `cfg(loom)` 下导入 `loom::sync::{Arc, Condvar, Mutex}`；T3 编译日志显示 `loom v0.7.2` 与 `kernel v0.5.0` 均被编译 |
| Loom 与 std 同步类型混用？ | 否 | `std::sync` 仅出现在 `cfg(not(loom))` 导入、`cfg(all(test, not(loom)))` 单元测试的 `Barrier`，以及标准路径集成测试 `tests/lifecycle_concurrency.rs` |
| 模型未 join 线程或遗漏断言？ | 否 | 前两个模型均 `join().unwrap()` 并断言 `is_triggered` / 计数；第三个模型无线程，断言两次 `is_triggered` 并调用 `wait()` |
| 状态空间爆炸？ | 否 | 3 个模型、最多 2 个 waiter，实测 1.59 秒完成 |
| 是否在 PR 中执行？ | 是 | PR #7 的 CI Run `42320761596` 由 `pull_request` 事件触发，`loom concurrency verification` 成功（29 秒） |
| Job 失败是否真正阻塞合并？ | 是 | `main` 的 Required Status Checks 已含 `fmt / clippy / test` 与 `loom concurrency verification`（`app_id 15368`）；PR #7 两个 check 均 SUCCESS，`mergeStateStatus=BLOCKED` 只因 `REVIEW_REQUIRED`（1 人审批），不是 check 阻塞 |
| 是否意外改变公共 API？ | 否 | 实现提交仅改 `.github/workflows/ci.yml`（+17 行），`src/` 无改动 |
| 是否改变 ShutdownGuard Drop 语义？ | 否 | 未改源码 |
| 是否引入不必要的生产依赖？ | 否 | 未改 `Cargo.toml`，loom 仍是原有条件依赖 |

## 3. 文档与实现同步核对

按工作区 `docs/README.md`「文档与实现同步」第 2、4 条，核对了 `README.md`、`CONTRIBUTING.md`、`docs/标准.md`、`docs/API.md`、`CHANGELOG.md`、`checklist.md`：

- 本变更只新增 CI 检查，不改变公开接口、生产依赖、职责或对外行为；上述文档描述的事实没有失实。
- `CONTRIBUTING.md` 第 3 条列的是本地运行命令，仍准确；`checklist.md` 自带「命令差异以 `.github/workflows/ci.yml` 为准」，新增 Job 不产生矛盾。
- 本任务新增的验证证据归档在 `docs/kernel-prod-001/`，不改变上述文档描述的事实。
- 因此未改既有文档，在此记录核对结果；CI 与分支保护核验已完成，剩余未完成项只有 Owner 审查与合入。

## 4. Evidence 与未解决问题

- 归档路径说明：pre-commit 钩子的 `isCrateSourcePath` 把 `evidence` 判为独立 crate 目录，`FORBIDDEN_RE` 禁止 `.log` 入库，故证据目录改为 `docs/kernel-prod-001/`，日志后缀改为 `.txt`；内容未改，未使用 `--no-verify`。
- `test-output.txt`：在 `c1cebb8` 上四条回归命令的完整输出（fmt、clippy、test、Loom），含每条 `exit_code=0`。
- `loom-output.txt`：`test-output.txt` 中 Loom 命令段，含 `running 3 tests` 与 `test result: ok. 3 passed; 0 failed`。
- `evidence.json`：数值全部取自上述两份日志，未复制计划示例中的示意数字；`result` 为 `PASS`，`ci_run_url` 已回填 PR #7 的 CI Run。
- T3 另在基线 `2b4e8422e82914562355f1912a0c4d26089c252f` 做过一次预验证（含编译共 20 秒、3 passed / 0 failed），用于确认「先验证命令再加 CI Job」；正式证据绑定实现提交 `c1cebb8`。
- CI 结果：Run `https://github.com/bytechainx/kernel/actions/runs/42320761596` 结论 `success`；`fmt / clippy / test` 与 `loom concurrency verification` 两个 check 均成功（1m0s / 29s）。Run 的 `head_sha` 为 `880cd1f`（含本证据目录），本地测试绑定实现提交 `c1cebb8`，两者 `ci.yml` 内容相同。
- 分支保护：`main` 的 Required Status Checks 已追加 `loom concurrency verification`，保留原有 `fmt / clippy / test`；误建远端分支 `issue/undefined-kernel-006-loom-ci` 已清理。
- 未解决项：① Owner 审查与合入由人工执行，本 PR 不自动合并（`reviewDecision=REVIEW_REQUIRED`）；② `wait_timeout` 的 Loom 覆盖边界（见第 1 节）；③ `publish = false` 保持不变。
- 范围外说明：Rust 1.88 MSRV 属 KERNEL-007，本轮未验证；本地工具链为 stable 1.99.0。元仓库共享检出的 `temporal` 两处他人脏树未被触碰。
