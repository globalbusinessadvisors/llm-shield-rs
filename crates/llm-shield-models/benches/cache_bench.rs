//! Performance benchmarks for ResultCache
//!
//! These benchmarks measure:
//! - Cache insert performance
//! - Cache retrieval (hit) performance
//! - Cache miss performance
//! - LRU eviction overhead
//! - Concurrent access performance
//! - Hash key generation performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llm_shield_core::ScanResult;
use llm_shield_models::cache::{CacheConfig, ResultCache};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Helper to create a test ScanResult
fn create_test_result(text: &str) -> ScanResult {
    ScanResult::pass(text.to_string())
}

/// Benchmark cache insertions
fn bench_cache_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_insert");

    for size in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let cache = ResultCache::new(CacheConfig {
                max_size: size,
                ttl: Duration::from_secs(300),
            });

            let mut i = 0;
            b.iter(|| {
                let key = format!("key{}", i);
                let result = create_test_result(&key);
                cache.insert(black_box(key), black_box(result));
                i += 1;
            });
        });
    }

    group.finish();
}

/// Benchmark cache retrieval (hits)
fn bench_cache_get_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_get_hit");

    for size in [100, 1000, 10000] {
        let cache = ResultCache::new(CacheConfig {
            max_size: size,
            ttl: Duration::from_secs(300),
        });

        // Pre-populate cache
        for i in 0..size {
            let key = format!("key{}", i);
            cache.insert(key, create_test_result(&format!("text{}", i)));
        }

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let mut i = 0;
            b.iter(|| {
                let key = format!("key{}", i % size);
                let result = cache.get(black_box(&key));
                black_box(result);
                i += 1;
            });
        });
    }

    group.finish();
}

/// Benchmark cache retrieval (misses)
fn bench_cache_get_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_get_miss");

    for size in [100, 1000, 10000] {
        let cache = ResultCache::new(CacheConfig {
            max_size: size,
            ttl: Duration::from_secs(300),
        });

        // Pre-populate cache
        for i in 0..size {
            let key = format!("key{}", i);
            cache.insert(key, create_test_result(&format!("text{}", i)));
        }

        group.throughput(Throughput::Elements(1));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            let mut i = 0;
            b.iter(|| {
                // Always miss
                let key = format!("missing_key{}", i);
                let result = cache.get(black_box(&key));
                black_box(result);
                i += 1;
            });
        });
    }

    group.finish();
}

/// Benchmark LRU eviction overhead
fn bench_cache_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_eviction");

    for capacity in [10, 100, 1000] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::from_parameter(capacity),
            &capacity,
            |b, &capacity| {
                b.iter_batched(
                    || {
                        let cache = ResultCache::new(CacheConfig {
                            max_size: capacity,
                            ttl: Duration::from_secs(300),
                        });
                        // Fill to capacity
                        for i in 0..capacity {
                            cache.insert(format!("key{}", i), create_test_result(&format!("text{}", i)));
                        }
                        cache
                    },
                    |cache| {
                        // Insert one more to trigger eviction
                        cache.insert(
                            black_box(format!("new_key")),
                            black_box(create_test_result("new_text")),
                        );
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark hash key generation
fn bench_hash_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_key_generation");

    let long_input = "This is a very long input text that simulates a large prompt. ".repeat(50);
    let inputs = vec![
        ("short", "Hello world"),
        (
            "medium",
            "This is a medium-length input text that might be typical for LLM inputs",
        ),
        (
            "long",
            &long_input,
        ),
    ];

    for (name, input) in inputs {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, &input| {
            b.iter(|| {
                let key = ResultCache::hash_key(black_box(input));
                black_box(key);
            });
        });
    }

    group.finish();
}

/// Benchmark concurrent read operations
fn bench_concurrent_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_reads");

    for thread_count in [2, 4, 8] {
        group.throughput(Throughput::Elements((thread_count * 100) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &thread_count| {
                let cache = Arc::new(ResultCache::new(CacheConfig {
                    max_size: 1000,
                    ttl: Duration::from_secs(300),
                }));

                // Pre-populate
                for i in 0..1000 {
                    cache.insert(format!("key{}", i), create_test_result(&format!("text{}", i)));
                }

                b.iter(|| {
                    let handles: Vec<_> = (0..thread_count)
                        .map(|_| {
                            let cache_clone = Arc::clone(&cache);
                            thread::spawn(move || {
                                for i in 0..100 {
                                    let key = format!("key{}", i % 1000);
                                    let _ = cache_clone.get(&key);
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent write operations
fn bench_concurrent_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_writes");

    for thread_count in [2, 4, 8] {
        group.throughput(Throughput::Elements((thread_count * 100) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            &thread_count,
            |b, &thread_count| {
                b.iter(|| {
                    let cache = Arc::new(ResultCache::new(CacheConfig {
                        max_size: 5000,
                        ttl: Duration::from_secs(300),
                    }));

                    let handles: Vec<_> = (0..thread_count)
                        .map(|thread_id| {
                            let cache_clone = Arc::clone(&cache);
                            thread::spawn(move || {
                                for i in 0..100 {
                                    let key = format!("thread{}_key{}", thread_id, i);
                                    cache_clone.insert(key, create_test_result("text"));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark mixed read/write operations
fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");

    let cache = ResultCache::new(CacheConfig {
        max_size: 10000,
        ttl: Duration::from_secs(300),
    });

    // Pre-populate
    for i in 0..5000 {
        cache.insert(format!("key{}", i), create_test_result(&format!("text{}", i)));
    }

    group.throughput(Throughput::Elements(100));
    group.bench_function("50_50_read_write", |b| {
        let mut i = 0;
        b.iter(|| {
            if i % 2 == 0 {
                // Read
                let key = format!("key{}", i % 5000);
                let _ = cache.get(black_box(&key));
            } else {
                // Write
                let key = format!("new_key{}", i);
                cache.insert(black_box(key), black_box(create_test_result("text")));
            }
            i += 1;
        });
    });

    group.finish();
}

/// Benchmark TTL expiration check overhead
fn bench_ttl_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("ttl_check");

    // Cache with short TTL
    let cache_short = ResultCache::new(CacheConfig {
        max_size: 1000,
        ttl: Duration::from_millis(10),
    });

    // Pre-populate
    for i in 0..1000 {
        cache_short.insert(format!("key{}", i), create_test_result(&format!("text{}", i)));
    }

    // Wait for expiration
    thread::sleep(Duration::from_millis(20));

    group.throughput(Throughput::Elements(1));
    group.bench_function("expired_entry", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("key{}", i % 1000);
            let result = cache_short.get(black_box(&key));
            black_box(result);
            i += 1;
        });
    });

    // Cache with long TTL (no expiration)
    let cache_long = ResultCache::new(CacheConfig {
        max_size: 1000,
        ttl: Duration::from_secs(3600),
    });

    // Pre-populate
    for i in 0..1000 {
        cache_long.insert(format!("key{}", i), create_test_result(&format!("text{}", i)));
    }

    group.bench_function("valid_entry", |b| {
        let mut i = 0;
        b.iter(|| {
            let key = format!("key{}", i % 1000);
            let result = cache_long.get(black_box(&key));
            black_box(result);
            i += 1;
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_insert,
    bench_cache_get_hit,
    bench_cache_get_miss,
    bench_cache_eviction,
    bench_hash_key_generation,
    bench_concurrent_reads,
    bench_concurrent_writes,
    bench_mixed_operations,
    bench_ttl_check,
);

criterion_main!(benches);
