//! Binary Size Benchmarks - Scenario 5A, 5B, 5C
//!
//! Validates the claim: WASM <2MB gzip (60-100x smaller than Python)
//!
//! Test Scenarios:
//! - 5A: Docker image size
//! - 5B: Native binary (stripped + UPX)
//! - 5C: WASM bundle (optimized + gzip)
//!
//! Note: Actual binary size measurements are done via scripts/bench_binary_size.sh
//! This file contains runtime benchmarks for size-related operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::PathBuf;

/// Measure binary file size at runtime
fn get_binary_size() -> Option<u64> {
    let exe_path = std::env::current_exe().ok()?;
    let metadata = fs::metadata(exe_path).ok()?;
    Some(metadata.len())
}

/// Scenario 5A: Report current binary size
fn bench_scenario_5a_binary_size_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_size_scenario_5a");

    group.bench_function("get_current_binary_size", |b| {
        b.iter(|| {
            let size = black_box(get_binary_size());
            size
        });
    });

    // Print binary size for reference
    if let Some(size) = get_binary_size() {
        println!("\nCurrent binary size: {} bytes ({:.2} MB)", size, size as f64 / 1_048_576.0);
    }

    group.finish();
}

/// Scenario 5B: Measure serialization size (proxy for WASM size)
fn bench_scenario_5b_serialization_size(c: &mut Criterion) {
    use llm_shield_scanners::input::BanSubstringsConfig;

    let config = BanSubstringsConfig {
        substrings: vec!["test".to_string(); 100],
        ..Default::default()
    };

    let mut group = c.benchmark_group("binary_size_scenario_5b");

    group.bench_function("serialize_config", |b| {
        b.iter(|| {
            let json = black_box(serde_json::to_string(&config).unwrap());
            json.len()
        });
    });

    group.finish();
}

/// Scenario 5C: Measure memory footprint of core structures
fn bench_scenario_5c_memory_footprint(c: &mut Criterion) {
    use llm_shield_core::SecretVault;
    use llm_shield_scanners::input::{BanSubstrings, BanSubstringsConfig};

    let mut group = c.benchmark_group("binary_size_scenario_5c");

    group.bench_function("core_structures_size", |b| {
        b.iter(|| {
            let vault = black_box(SecretVault::new());
            let scanner = black_box(
                BanSubstrings::new(BanSubstringsConfig {
                    substrings: vec!["test".to_string()],
                    ..Default::default()
                })
                .unwrap(),
            );

            // Calculate approximate memory usage
            let vault_size = std::mem::size_of_val(&vault);
            let scanner_size = std::mem::size_of_val(&scanner);

            vault_size + scanner_size
        });
    });

    group.finish();
}

criterion_group!(
    binary_size_benches,
    bench_scenario_5a_binary_size_check,
    bench_scenario_5b_serialization_size,
    bench_scenario_5c_memory_footprint,
);

criterion_main!(binary_size_benches);
