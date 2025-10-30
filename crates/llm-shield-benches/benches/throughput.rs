//! Throughput Benchmarks - Scenario 2A and 2B
//!
//! Validates the claim: Rust >10,000 req/sec (100x higher than Python)
//!
//! Test Scenarios:
//! - 2A: Single scanner, concurrent requests
//! - 2B: Scanner pipeline (3 scanners in sequence)
//!
//! Note: Full throughput testing requires HTTP server (wrk) - see scripts/bench_throughput.sh
//! These benchmarks test the scanner throughput directly

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use llm_shield_benches::fixtures::generate_test_prompts;
use llm_shield_core::SecretVault;
use llm_shield_scanners::input::{
    BanSubstrings, BanSubstringsConfig, Secrets, SecretsConfig, Toxicity, ToxicityConfig,
};
use tokio::runtime::Runtime;

/// Scenario 2A: Single scanner throughput
///
/// Expected: >10,000 req/sec for simple scanners
fn bench_scenario_2a_single_scanner(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let vault = SecretVault::new();

    let scanner = BanSubstrings::new(BanSubstringsConfig {
        substrings: vec!["test".to_string()],
        ..Default::default()
    })
    .unwrap();

    let prompts = generate_test_prompts(100);
    let simple_prompts: Vec<_> = prompts
        .iter()
        .filter(|p| p.category == "simple")
        .take(10)
        .collect();

    let mut group = c.benchmark_group("throughput_scenario_2a");
    group.throughput(Throughput::Elements(simple_prompts.len() as u64));

    group.bench_function("single_scanner_batch", |b| {
        b.to_async(&rt).iter(|| async {
            for prompt in &simple_prompts {
                let _ = scanner.scan(black_box(&prompt.text), &vault).await;
            }
        });
    });

    group.finish();
}

/// Scenario 2B: Scanner pipeline throughput
///
/// Expected: >1,000 req/sec for 3-scanner pipeline
fn bench_scenario_2b_pipeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let vault = SecretVault::new();

    // Create 3-scanner pipeline
    let scanner1 = BanSubstrings::new(BanSubstringsConfig {
        substrings: vec!["banned".to_string()],
        ..Default::default()
    })
    .unwrap();

    let scanner2 = Secrets::new(SecretsConfig::default()).unwrap();

    let scanner3 = Toxicity::new(ToxicityConfig::default()).unwrap();

    let prompts = generate_test_prompts(100);
    let test_prompts: Vec<_> = prompts.iter().take(10).collect();

    let mut group = c.benchmark_group("throughput_scenario_2b");
    group.throughput(Throughput::Elements(test_prompts.len() as u64));

    group.bench_function("pipeline_3_scanners", |b| {
        b.to_async(&rt).iter(|| async {
            for prompt in &test_prompts {
                // Run scanners in pipeline
                let _ = scanner1.scan(black_box(&prompt.text), &vault).await;
                let _ = scanner2.scan(black_box(&prompt.text), &vault).await;
                let _ = scanner3.scan(black_box(&prompt.text), &vault).await;
            }
        });
    });

    group.finish();
}

criterion_group!(
    throughput_benches,
    bench_scenario_2a_single_scanner,
    bench_scenario_2b_pipeline,
);

criterion_main!(throughput_benches);
