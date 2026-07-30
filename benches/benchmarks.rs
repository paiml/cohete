//! Criterion benchmarks for cohete E2E verification tool.
//!
//! Benchmarks JSON serialization throughput and config parsing
//! which are the hot paths in artifact emission.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde::Serialize;

/// Mirror of a tier result for benchmarking serialization throughput.
#[derive(Serialize, Clone)]
struct BenchTierResult {
    pass: bool,
    total: u32,
    passed: u32,
    failed: u32,
    checks: Vec<BenchCheck>,
}

#[derive(Serialize, Clone)]
struct BenchCheck {
    name: String,
    pass: bool,
    stdout: String,
    stderr: String,
    duration_ms: u64,
}

fn make_tier_result(n: usize) -> BenchTierResult {
    let checks: Vec<BenchCheck> = (0..n)
        .map(|i| BenchCheck {
            name: format!("check_{i}"),
            pass: i % 3 != 0,
            stdout: format!("output line {i}"),
            stderr: String::new(),
            duration_ms: 42 + i as u64,
        })
        .collect();
    // u32::try_from rather than `as`: a `usize -> u32` cast silently truncates on
    // 64-bit targets. These counts are bench fixture sizes so they never overflow
    // in practice, but the cast is what clippy denies and the gate is right to.
    let passed = u32::try_from(checks.iter().filter(|c| c.pass).count()).unwrap_or(u32::MAX);
    let total = u32::try_from(checks.len()).unwrap_or(u32::MAX);
    let failed = total - passed;
    BenchTierResult {
        pass: failed == 0,
        total,
        passed,
        failed,
        checks,
    }
}

fn bench_json_serialize_small(c: &mut Criterion) {
    let result = make_tier_result(8);
    c.bench_function("json_serialize_8_checks", |b| {
        b.iter(|| serde_json::to_string_pretty(black_box(&result)).unwrap());
    });
}

fn bench_json_serialize_large(c: &mut Criterion) {
    let result = make_tier_result(100);
    c.bench_function("json_serialize_100_checks", |b| {
        b.iter(|| serde_json::to_string_pretty(black_box(&result)).unwrap());
    });
}

fn bench_json_roundtrip(c: &mut Criterion) {
    let result = make_tier_result(20);
    let json = serde_json::to_string_pretty(&result).unwrap();
    c.bench_function("json_deserialize_20_checks", |b| {
        b.iter(|| {
            serde_json::from_str::<serde_json::Value>(black_box(&json)).unwrap();
        });
    });
}

fn bench_chrono_timestamp(c: &mut Criterion) {
    c.bench_function("chrono_utc_now_format", |b| {
        b.iter(|| {
            let now = chrono::Utc::now();
            black_box(now.to_rfc3339())
        });
    });
}

criterion_group!(
    benches,
    bench_json_serialize_small,
    bench_json_serialize_large,
    bench_json_roundtrip,
    bench_chrono_timestamp,
);
criterion_main!(benches);
