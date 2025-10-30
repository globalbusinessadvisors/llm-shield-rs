//! Cold Start Benchmarks - Scenario 4A, 4B, 4C
//!
//! Validates the claim: Rust <1s (10-30x faster than Python)
//!
//! Test Scenarios:
//! - 4A: Application startup time
//! - 4B: First request latency
//! - 4C: Serverless cold start
//!
//! Note: Full cold start testing requires hyperfine - see scripts/bench_cold_start.sh
//! These benchmarks measure scanner initialization time

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llm_shield_core::SecretVault;
use llm_shield_scanners::input::{
    BanSubstrings, BanSubstringsConfig, Secrets, SecretsConfig, Toxicity, ToxicityConfig,
};
use tokio::runtime::Runtime;

/// Scenario 4A: Scanner initialization time
fn bench_scenario_4a_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_scenario_4a");

    group.bench_function("ban_substrings_init", |b| {
        b.iter(|| {
            let _scanner = black_box(
                BanSubstrings::new(BanSubstringsConfig {
                    substrings: vec!["test".to_string()],
                    ..Default::default()
                })
                .unwrap(),
            );
        });
    });

    group.bench_function("secrets_init", |b| {
        b.iter(|| {
            let _scanner = black_box(Secrets::new(SecretsConfig::default()).unwrap());
        });
    });

    group.bench_function("toxicity_init", |b| {
        b.iter(|| {
            let _scanner = black_box(Toxicity::new(ToxicityConfig::default()).unwrap());
        });
    });

    group.finish();
}

/// Scenario 4B: First request latency (init + first scan)
fn bench_scenario_4b_first_request(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("cold_start_scenario_4b");

    group.bench_function("first_scan_ban_substrings", |b| {
        b.to_async(&rt).iter(|| async {
            let vault = SecretVault::new();
            let scanner = BanSubstrings::new(BanSubstringsConfig {
                substrings: vec!["test".to_string()],
                ..Default::default()
            })
            .unwrap();

            let _ = scanner.scan(black_box("test prompt"), &vault).await;
        });
    });

    group.bench_function("first_scan_secrets", |b| {
        b.to_async(&rt).iter(|| async {
            let vault = SecretVault::new();
            let scanner = Secrets::new(SecretsConfig::default()).unwrap();

            let _ = scanner
                .scan(black_box("test with key: AKIAIOSFODNN7EXAMPLE"), &vault)
                .await;
        });
    });

    group.finish();
}

/// Scenario 4C: Rapid initialization cycles (simulates serverless)
fn bench_scenario_4c_serverless_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_scenario_4c");

    group.bench_function("rapid_init_destroy_100x", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _scanner = black_box(
                    BanSubstrings::new(BanSubstringsConfig {
                        substrings: vec!["test".to_string()],
                        ..Default::default()
                    })
                    .unwrap(),
                );
            }
        });
    });

    group.finish();
}

criterion_group!(
    cold_start_benches,
    bench_scenario_4a_initialization,
    bench_scenario_4b_first_request,
    bench_scenario_4c_serverless_simulation,
);

criterion_main!(cold_start_benches);
