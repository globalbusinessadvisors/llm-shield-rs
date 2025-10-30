//! CPU Usage Benchmarks - Scenario 6A, 6B, 6C
//!
//! Validates the claim: 5-10x more efficient CPU usage
//!
//! Test Scenarios:
//! - 6A: Single request CPU time
//! - 6B: CPU % under sustained load
//! - 6C: CPU efficiency (req/sec per core)
//!
//! Note: Full CPU profiling requires pidstat/perf - see scripts/bench_cpu.sh
//! These benchmarks measure computational efficiency

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use llm_shield_benches::fixtures::generate_test_prompts;
use llm_shield_core::SecretVault;
use llm_shield_scanners::input::{Secrets, SecretsConfig};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Scenario 6A: CPU time per request
fn bench_scenario_6a_cpu_per_request(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let vault = SecretVault::new();
    let scanner = Secrets::new(SecretsConfig::default()).unwrap();

    let prompt = "Test prompt with AWS key: AKIAIOSFODNN7EXAMPLE";

    let mut group = c.benchmark_group("cpu_scenario_6a");

    group.bench_function("cpu_time_single_request", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = scanner.scan(black_box(prompt), &vault).await;
        });
    });

    group.finish();
}

/// Scenario 6B: CPU efficiency under load
fn bench_scenario_6b_sustained_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let vault = SecretVault::new();
    let scanner = Secrets::new(SecretsConfig::default()).unwrap();

    let prompts = generate_test_prompts(100);

    let mut group = c.benchmark_group("cpu_scenario_6b");
    group.throughput(Throughput::Elements(prompts.len() as u64));

    group.bench_function("sustained_load_100_requests", |b| {
        b.to_async(&rt).iter(|| async {
            for prompt in &prompts {
                let _ = scanner.scan(black_box(&prompt.text), &vault).await;
            }
        });
    });

    group.finish();
}

/// Scenario 6C: Calculate requests per second
fn bench_scenario_6c_req_per_sec(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let vault = SecretVault::new();
    let scanner = Secrets::new(SecretsConfig::default()).unwrap();

    let prompt = "Test prompt";
    let duration = Duration::from_secs(1);

    let mut group = c.benchmark_group("cpu_scenario_6c");

    // Measure how many requests we can process in 1 second
    let start = Instant::now();
    let mut count = 0;

    rt.block_on(async {
        while start.elapsed() < duration {
            let _ = scanner.scan(prompt, &vault).await;
            count += 1;
        }
    });

    println!("\nRequests per second: {}", count);
    println!("Average latency: {:.4}ms", 1000.0 / count as f64);

    group.bench_function("throughput_measurement", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = scanner.scan(black_box(prompt), &vault).await;
        });
    });

    group.finish();
}

criterion_group!(
    cpu_benches,
    bench_scenario_6a_cpu_per_request,
    bench_scenario_6b_sustained_load,
    bench_scenario_6c_req_per_sec,
);

criterion_main!(cpu_benches);
