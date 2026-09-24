# 贡献指南

1. 在 feature branch 上提交变更，并通过 Pull Request 合入 `main`。
2. 修改公开 API、错误分类、时间语义或生命周期状态前，先同步更新 [`docs/标准.md`](docs/标准.md) 与 [`docs/API.md`](docs/API.md)。
3. 运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` 和 `cargo test`。
4. 不加入 `unsafe`、I/O、异步运行时或业务能力；保持 Rust 1.88 兼容。
