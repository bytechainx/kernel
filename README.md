# kernel

`kernel` 提供共享错误分类与组件生命周期信号，是 bytechainx 的 L0 kernel crate。

| 项目 | 值 |
| --- | --- |
| 版本 | `0.5.0` |
| Rust | Edition 2024，MSRV 1.88 |
| 许可 | MIT |
| 发布 | 仅从 Git 源码消费，不发布到 crates.io |

## 安装

通过 Git 依赖引入：

```toml
[dependencies]
kernel = { git = "https://github.com/bytechainx/kernel" }
```

## 获取源码

```bash
git clone git@github.com:bytechainx/kernel.git
```

在 Cargo 项目中使用本地 Git checkout：

```toml
[dependencies]
kernel = { version = "0.5.0", path = "../kernel" }
```

## 能力范围

- `error`：不透明错误、错误分类和结果类型。
- `lifecycle`：组件状态、关停信号、关停守卫和生命周期错误。

该 crate 不提供网络、日志、异步运行时、持久化或业务规则。

## 从 0.4.2 迁移

0.5.0 是破坏性迁移候选：删除 `time`、`time_storage` 及其根导出和 `From<TimeError> for XError`。时间值、时钟和精度合同改由独立 [temporal](https://github.com/bytechainx/temporal) 提供；调用方直接声明该依赖并显式分类错误。`kernel` 与 `temporal` 互不依赖。

旧 `verify_pg_dual_column` 不属于通用时间能力，PostgreSQL 消费适配器应使用 `temporal::verify_projection` 实现本后端校验。新旧时间行为差异和迁移入口见 [API 文档](docs/API.md#时间公开面迁移)。

## 验证

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
