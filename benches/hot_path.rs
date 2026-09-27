#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unreachable)]
//! kernel 热路径基准：错误分类查询与关停信号观察。
use std::hint::black_box;
use std::time::Instant;

use kernel::{ShutdownSignal, XError};

fn iters() -> u32 {
    if std::env::args().any(|a| a == "--quick") { 2_000 } else { 200_000 }
}

fn main() {
    let n = iters();
    let error = XError::transient("重试提示");
    let (guard, signal) = ShutdownSignal::new();
    guard.trigger();
    for _ in 0..n.min(50) {
        black_box(black_box(&signal).is_triggered());
    }
    let start = Instant::now();
    let mut observed = 0u32;
    for _ in 0..n {
        if black_box(&error).is_retryable() && black_box(&signal).is_triggered() {
            observed += 1;
        }
    }
    let elapsed = start.elapsed();
    assert_eq!(observed, n);
    println!(
        "bench_kernel_lifecycle: iters={n} total={elapsed:?} per_iter={:?} observed={}",
        elapsed / n,
        black_box(observed)
    );
}
