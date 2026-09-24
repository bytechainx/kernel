# kernel

`kernel` 提供共享错误分类、时间表示与组件生命周期信号，是 bytechainx 的 L0 kernel crate。

| 项目 | 值 |
| --- | --- |
| 版本 | `0.4.2` |
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
kernel = { version = "0.4.2", path = "../kernel" }
```

## 能力范围

- `error`：不透明错误、错误分类和结果类型。
- `time`：纳秒 Unix 时刻、墙钟与单调时钟接口。
- `lifecycle`：组件状态、关停信号、关停守卫和生命周期错误。
- `time_storage`：时间持久化精度约束，不执行 I/O。

该 crate 不提供网络、日志、异步运行时、持久化或业务规则。

## 验证

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
