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
| `From<TimeError> for XError` | `Overflow` / `SystemTimeOutOfRange` → `Unavailable`；`InvalidOrder` → `Invalid` |

### time

| 符号 | 说明 |
|------|------|
| `UnixTimeNs` | UTC POSIX epoch 纳秒：`from_unix_nanos` / `try_from_unix_{seconds,millis,micros}` / `checked_add` / `checked_sub` / `duration_since` |
| `TimeError` | `Overflow` · `InvalidOrder` · `SystemTimeOutOfRange` |
| `WallClock` | trait：`now() -> Result<UnixTimeNs, TimeError>` |
| `MonotonicClock` | trait：`now() -> Instant` |
| `SystemWallClock` / `SystemMonotonicClock` | 系统实现 |
| `RuntimeClock` | `WallClock + MonotonicClock` 组合；Domain 不得依赖 |

### lifecycle

| 符号 | 说明 |
|------|------|
| `ComponentState` | `Created` · `Starting` · `Running` · `Draining` · `Stopped` · `Failed`；`can_transition_to` / `try_transition` |
| `LifecycleError` | 非法转换：`from` / `to` 字段 |
| `ShutdownSignal` | `new` → `(ShutdownGuard, ShutdownSignal)`；`is_triggered` / `wait` / `wait_timeout -> Result<bool, WaitTimeoutError>` / `Clone` |
| `WaitTimeoutError` | `DeadlineOverflow`：deadline 无法表示，不得伪装为普通超时 |
| `ShutdownGuard` | 唯一触发入口：`trigger(self)` |

### 模块路径

`kernel::time` · `kernel::error` · `kernel::lifecycle` 亦可直接使用（与根 re-export 同义）。

## 最小用法

```rust
use kernel::{SystemWallClock, WallClock, XError, ShutdownSignal};

let clock = SystemWallClock::new();
let _ts = clock.now()?;
let (guard, signal) = ShutdownSignal::new();
guard.trigger();
assert!(signal.is_triggered());
let _ = XError::invalid("bad input");
# Ok::<(), kernel::XError>(())
```

```bash
cargo run -p kernel --example kernel_basic
```

## 覆盖

集成测试 `tests/public_api_surface.rs` 驱动上述全部根 re-export 构造器/方法并断言返回值。  
公开 API 的行为与源码保持一致；新增或变更公开项时更新本 crate 的文档与测试。
