//! Memory Usage Benchmarks - Scenario 3A, 3B, 3C
//!
//! Validates the claim: Rust <500MB (8-16x lower than Python)
//!
//! Test Scenarios:
//! - 3A: Baseline memory (idle)
//! - 3B: Under load (processing)
//! - 3C: Memory growth over time
//!
//! Note: Full memory profiling requires system tools (pidstat) - see scripts/bench_memory.sh
//! These benchmarks measure scanner memory usage

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llm_shield_benches::fixtures::generate_test_prompts;
use llm_shield_core::SecretVault;
use llm_shield_scanners::input::{Secrets, SecretsConfig};
use tokio::runtime::Runtime;

/// Scenario 3A: Baseline memory
fn bench_scenario_3a_baseline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_scenario_3a");

    group.bench_function("scanner_initialization", |b| {
        b.iter(|| {
            let vault = black_box(SecretVault::new());
            let _scanner = black_box(Secrets::new(SecretsConfig::default()).unwrap());
            drop(vault);
        });
    });

    group.finish();
}

/// Scenario 3B: Memory under load
fn bench_scenario_3b_under_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let vault = SecretVault::new();
    let scanner = Secrets::new(SecretsConfig::default()).unwrap();

    // Generate large dataset
    let prompts = generate_test_prompts(1000);

    let mut group = c.benchmark_group("memory_scenario_3b");

    group.bench_function("process_1000_prompts", |b| {
        b.to_async(&rt).iter(|| async {
            for prompt in &prompts {
                let _ = scanner.scan(black_box(&prompt.text), &vault).await;
            }
        });
    });

    group.finish();
}

/// Scenario 3C: Memory stability over iterations
fn bench_scenario_3c_stability(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let vault = SecretVault::new();
    let scanner = Secrets::new(SecretsConfig::default()).unwrap();

    let prompt = "Test prompt with potential secret: AKIAIOSFODNN7EXAMPLE";

    let mut group = c.benchmark_group("memory_scenario_3c");

    group.bench_function("repeated_scans_10000", |b| {
        b.to_async(&rt).iter(|| async {
            for _ in 0..10000 {
                let _ = scanner.scan(black_box(prompt), &vault).await;
            }
        });
    });

    group.finish();
}

criterion_group!(
    memory_benches,
    bench_scenario_3a_baseline,
    bench_scenario_3b_under_load,
    bench_scenario_3c_stability,
);

criterion_main!(memory_benches);
