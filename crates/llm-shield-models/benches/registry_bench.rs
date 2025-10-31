//! Performance benchmarks for ModelRegistry
//!
//! These benchmarks measure:
//! - Registry loading from JSON
//! - Model metadata retrieval
//! - Model discovery/listing operations
//! - Concurrent read performance
//! - Clone overhead

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llm_shield_models::{ModelRegistry, ModelTask, ModelVariant};
use std::sync::Arc;
use tempfile::TempDir;

/// Helper to create a test registry with N models
fn create_test_registry(num_models: usize) -> (ModelRegistry, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    // Create registry content with N models
    let mut models = Vec::new();
    let tasks = [
        ModelTask::PromptInjection,
        ModelTask::Toxicity,
        ModelTask::Sentiment,
    ];
    let variants = [ModelVariant::FP16, ModelVariant::FP32, ModelVariant::INT8];

    for i in 0..num_models {
        let task = tasks[i % tasks.len()];
        let variant = variants[i % variants.len()];
        models.push(format!(
            r#"{{
                "id": "model-{}",
                "task": "{:?}",
                "variant": "{:?}",
                "url": "https://example.com/model-{}.onnx",
                "checksum": "checksum{}",
                "size_bytes": {}
            }}"#,
            i, task, variant, i, i, 1024 * (i + 1)
        ));
    }

    let registry_content = format!(
        r#"{{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [{}]
    }}"#,
        models.join(",")
    );

    std::fs::write(&registry_path, registry_content).unwrap();

    let registry =
        ModelRegistry::from_file(registry_path.to_str().unwrap()).expect("Failed to load registry");

    (registry, temp_dir)
}

/// Benchmark registry loading from JSON
fn bench_registry_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_load");

    for num_models in [10, 50, 100, 500] {
        group.throughput(Throughput::Elements(num_models as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_models),
            &num_models,
            |b, &num_models| {
                let temp_dir = TempDir::new().unwrap();
                let registry_path = temp_dir.path().join("registry.json");

                // Pre-create registry file
                let mut models = Vec::new();
                for i in 0..num_models {
                    models.push(format!(
                        r#"{{
                            "id": "model-{}",
                            "task": "PromptInjection",
                            "variant": "FP16",
                            "url": "https://example.com/model.onnx",
                            "checksum": "abc123",
                            "size_bytes": 1024
                        }}"#,
                        i
                    ));
                }

                let registry_content = format!(
                    r#"{{
                    "cache_dir": "/tmp/llm-shield/models",
                    "models": [{}]
                }}"#,
                    models.join(",")
                );

                std::fs::write(&registry_path, registry_content).unwrap();
                let path = registry_path.to_str().unwrap().to_string();

                b.iter(|| {
                    let _registry = ModelRegistry::from_file(black_box(&path))
                        .expect("Failed to load registry");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark get_model_metadata
fn bench_get_model_metadata(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_model_metadata");

    for num_models in [10, 100, 1000] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_models),
            &num_models,
            |b, &num_models| {
                let (registry, _temp_dir) = create_test_registry(num_models);

                b.iter(|| {
                    let _metadata = registry
                        .get_model_metadata(
                            black_box(ModelTask::PromptInjection),
                            black_box(ModelVariant::FP16),
                        )
                        .expect("Model should exist");
                });
            },
        );
    }

    group.finish();
}

/// Benchmark has_model
fn bench_has_model(c: &mut Criterion) {
    let mut group = c.benchmark_group("has_model");

    for num_models in [10, 100, 1000] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_models),
            &num_models,
            |b, &num_models| {
                let (registry, _temp_dir) = create_test_registry(num_models);

                b.iter(|| {
                    let _exists = registry.has_model(
                        black_box(ModelTask::PromptInjection),
                        black_box(ModelVariant::FP16),
                    );
                });
            },
        );
    }

    group.finish();
}

/// Benchmark list_models
fn bench_list_models(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_models");

    for num_models in [10, 100, 1000] {
        group.throughput(Throughput::Elements(num_models as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_models),
            &num_models,
            |b, &num_models| {
                let (registry, _temp_dir) = create_test_registry(num_models);

                b.iter(|| {
                    let _models = registry.list_models();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark list_models_for_task
fn bench_list_models_for_task(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_models_for_task");

    for num_models in [10, 100, 1000] {
        group.throughput(Throughput::Elements(num_models as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_models),
            &num_models,
            |b, &num_models| {
                let (registry, _temp_dir) = create_test_registry(num_models);

                b.iter(|| {
                    let _models =
                        registry.list_models_for_task(black_box(ModelTask::PromptInjection));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark get_available_variants
fn bench_get_available_variants(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_available_variants");

    for num_models in [10, 100, 1000] {
        group.throughput(Throughput::Elements(num_models as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_models),
            &num_models,
            |b, &num_models| {
                let (registry, _temp_dir) = create_test_registry(num_models);

                b.iter(|| {
                    let _variants =
                        registry.get_available_variants(black_box(ModelTask::PromptInjection));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark registry clone
fn bench_registry_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_clone");

    for num_models in [10, 100, 1000] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_models),
            &num_models,
            |b, &num_models| {
                let (registry, _temp_dir) = create_test_registry(num_models);

                b.iter(|| {
                    let _cloned = black_box(registry.clone());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent reads
fn bench_concurrent_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_reads");

    for num_threads in [2, 4, 8, 16] {
        group.throughput(Throughput::Elements(num_threads as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_threads),
            &num_threads,
            |b, &num_threads| {
                let (registry, _temp_dir) = create_test_registry(100);
                let registry = Arc::new(registry);

                b.iter(|| {
                    let mut handles = vec![];

                    for i in 0..num_threads {
                        let registry_clone = Arc::clone(&registry);
                        let handle = std::thread::spawn(move || {
                            let task = if i % 3 == 0 {
                                ModelTask::PromptInjection
                            } else if i % 3 == 1 {
                                ModelTask::Toxicity
                            } else {
                                ModelTask::Sentiment
                            };

                            let _metadata = registry_clone
                                .get_model_metadata(task, ModelVariant::FP16)
                                .ok();
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_registry_load,
    bench_get_model_metadata,
    bench_has_model,
    bench_list_models,
    bench_list_models_for_task,
    bench_get_available_variants,
    bench_registry_clone,
    bench_concurrent_reads
);
criterion_main!(benches);
