# kernel 公开 API

**Package**：`kernel` · **角色**：L0 语义信任根  
**生产层级**：L1 Internal Ready + L4 Platform Ready

## 公开消费面（crate 根 re-export）

### error

| 符号 | 说明 |
|------|------|
| `XError` | 不透明错误；构造器：`invalid` / `missing` / `conflict` / `transient` / `transient_after` / `unavailable` / `cancelled` / `deadline_exceeded` / `invariant` / `internal` |
| `XError::kind` / `context` / `retry_after` / `is_retryable` / `is_bug` / `with_source` | 查询与 source 链 |
| `ErrorKind` | 9 种分类：`Invalid` · `Missing` · `Conflict` · `Transient` · `Unavailable` · `Cancelled` · `DeadlineExceeded` · `Invariant` · `Internal` |
| `XResult<T>` | `Result<T, XError>` |
| `BoxError` | `Box<dyn Error + Send + Sync + 'static>` |

### lifecycle

| 符号 | 说明 |
|------|------|
| `ComponentState` | `Created` · `Starting` · `Running` · `Draining` · `Stopped` · `Failed`；`can_transition_to` / `try_transition` |
| `LifecycleError` | 非法转换：`from` / `to` 字段 |
| `ShutdownSignal` | `new` → `(ShutdownGuard, ShutdownSignal)`；`is_triggered` / `wait` / `wait_timeout -> Result<bool, WaitTimeoutError>` / `Clone` |
| `WaitTimeoutError` | `DeadlineOverflow`：deadline 无法表示，不得伪装为普通超时 |
| `ShutdownGuard` | 唯一触发入口：`trigger(self)` |

### 模块路径

`kernel::error` · `kernel::lifecycle` 亦可直接使用（与根 re-export 同义）。

## 最小用法

```rust
use kernel::{XError, ShutdownSignal};

let (guard, signal) = ShutdownSignal::new();
guard.trigger();
assert!(signal.is_triggered());
let _ = XError::invalid("bad input");
```

```bash
cargo run -p kernel --example kernel_basic
```

## 覆盖

集成测试 `tests/public_api_surface.rs` 驱动上述全部根 re-export 构造器/方法并断言返回值。  
公开 API 的行为与源码保持一致；新增或变更公开项时更新本 crate 的文档与测试。

## 时间公开面迁移

0.5.0 删除 `kernel::time`、`kernel::time_storage`、全部对应根导出，以及 `From<TimeError> for XError`。直接使用独立 [temporal 0.1.0](https://github.com/bytechainx/temporal) 的消费者应固定经审查的完整 Git commit 并提交自己的锁文件；本 crate 不添加对 `temporal` 的依赖。

| 旧公开面 | 迁移入口与行为差异 |
| --- | --- |
| `UnixTimeNs`、`TimeError`、时钟 trait 与系统实现 | 使用 `temporal` 对应类型；负 epoch 在平台可表示时支持，`SystemTime` 转换严格拒绝平台精度损失，不再一律拒绝负值。 |
| 通用 `time_storage` 换算、精度策略与能力 | 使用 `temporal` 的精度 API；返回本地 `PrecisionError`，增加显式向下取整策略。 |
| 旧错误变体与能力辅助方法 | `TimeError::InvalidOrder { later, earlier }` 改为单元变体 `TimeError::InvalidOrder`，须修改构造和模式匹配；`temporal` 不提供旧 `TimeStorageCapabilities::new` 与 `TimePrecision::is_lossless_unix_ns`，保留这些调用将无法编译，须按新精度 API 改写。 |
| `From<TimeError> for XError` | 消费方用 `map_err` 明确分类，可用 `XError::with_source` 保留来源；不得期待隐式 `?` 转换。 |
| `verify_pg_dual_column` | PostgreSQL 适配器负责调用通用 `temporal::verify_projection`，严格核对声明的投影策略；本函数不迁入 `temporal`。 |

`XError::retry_after`、`ShutdownSignal::wait_timeout` 与 `WaitTimeoutError` 的标准库时长语义保持不变。迁移时应检查整个应用依赖闭包，避免同时使用旧 `kernel` 与 `temporal` 的两套绝对时间类型。
