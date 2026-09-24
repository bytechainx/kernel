# 项目上下文

`kernel` 是 bytechainx 的 L0 值对象 crate，当前版本 `0.4.2`。

- 语义权威：[`docs/标准.md`](docs/标准.md)。
- 公开 API：[`docs/API.md`](docs/API.md)。
- 生产依赖：`thiserror`；`loom` 仅供 `cfg(loom)` 并发验证使用。
- 代码禁止 `unsafe`；公开项必须有文档。
- Rust Edition 2024，MSRV 1.88；通过 Git checkout 消费，不发布到 crates.io。
