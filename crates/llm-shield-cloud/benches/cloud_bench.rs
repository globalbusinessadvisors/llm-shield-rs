use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llm_shield_cloud::{
    CloudLogger, CloudMetrics, CloudSecretManager, CloudStorage, LogEntry, Metric, SecretValue,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::runtime::Runtime;

// Mock implementations for benchmarking
struct MockSecretManager {
    cache: HashMap<String, String>,
}

#[async_trait::async_trait]
impl CloudSecretManager for MockSecretManager {
    async fn get_secret(&self, name: &str) -> llm_shield_cloud::Result<SecretValue> {
        let value = self
            .cache
            .get(name)
            .cloned()
            .unwrap_or_else(|| format!("secret-{}", name));
        Ok(SecretValue::String(value))
    }

    async fn put_secret(&self, name: &str, value: &SecretValue) -> llm_shield_cloud::Result<()> {
        let _ = (name, value);
        Ok(())
    }

    async fn delete_secret(&self, name: &str) -> llm_shield_cloud::Result<()> {
        let _ = name;
        Ok(())
    }

    async fn list_secrets(&self) -> llm_shield_cloud::Result<Vec<String>> {
        Ok(self.cache.keys().cloned().collect())
    }
}

struct MockStorage {
    cache: HashMap<String, Vec<u8>>,
}

#[async_trait::async_trait]
impl CloudStorage for MockStorage {
    async fn get_object(&self, key: &str) -> llm_shield_cloud::Result<Vec<u8>> {
        Ok(self
            .cache
            .get(key)
            .cloned()
            .unwrap_or_else(|| vec![0; 1024]))
    }

    async fn put_object(&self, key: &str, data: &[u8]) -> llm_shield_cloud::Result<()> {
        let _ = (key, data);
        Ok(())
    }

    async fn delete_object(&self, key: &str) -> llm_shield_cloud::Result<()> {
        let _ = key;
        Ok(())
    }

    async fn list_objects(&self, prefix: Option<&str>) -> llm_shield_cloud::Result<Vec<String>> {
        let _ = prefix;
        Ok(self.cache.keys().cloned().collect())
    }

    async fn object_exists(&self, key: &str) -> llm_shield_cloud::Result<bool> {
        Ok(self.cache.contains_key(key))
    }

    async fn get_object_metadata(
        &self,
        key: &str,
    ) -> llm_shield_cloud::Result<HashMap<String, String>> {
        let _ = key;
        Ok(HashMap::new())
    }
}

struct MockMetrics;

#[async_trait::async_trait]
impl CloudMetrics for MockMetrics {
    async fn export_metric(&self, metric: &Metric) -> llm_shield_cloud::Result<()> {
        let _ = metric;
        Ok(())
    }

    async fn export_metrics(&self, metrics: &[Metric]) -> llm_shield_cloud::Result<()> {
        let _ = metrics;
        Ok(())
    }
}

struct MockLogger;

#[async_trait::async_trait]
impl CloudLogger for MockLogger {
    async fn log(&self, entry: &LogEntry) -> llm_shield_cloud::Result<()> {
        let _ = entry;
        Ok(())
    }

    async fn batch_log(&self, entries: &[LogEntry]) -> llm_shield_cloud::Result<()> {
        let _ = entries;
        Ok(())
    }
}

fn bench_secret_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut cache = HashMap::new();
    cache.insert("api-key".to_string(), "secret-value".to_string());
    let manager = Arc::new(MockSecretManager { cache });

    let mut group = c.benchmark_group("secret_operations");

    group.bench_function("get_secret", |b| {
        b.to_async(&rt).iter(|| async {
            let result = manager.get_secret(black_box("api-key")).await;
            black_box(result)
        })
    });

    group.bench_function("put_secret", |b| {
        b.to_async(&rt).iter(|| async {
            let value = SecretValue::String("new-secret".to_string());
            let result = manager.put_secret(black_box("new-key"), black_box(&value)).await;
            black_box(result)
        })
    });

    group.bench_function("list_secrets", |b| {
        b.to_async(&rt).iter(|| async {
            let result = manager.list_secrets().await;
            black_box(result)
        })
    });

    group.finish();
}

fn bench_storage_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let storage = Arc::new(MockStorage {
        cache: HashMap::new(),
    });

    let mut group = c.benchmark_group("storage_operations");

    // Small object (1KB)
    group.throughput(Throughput::Bytes(1024));
    group.bench_function("put_object_1kb", |b| {
        let data = vec![0u8; 1024];
        b.to_async(&rt).iter(|| async {
            let result = storage.put_object(black_box("test-key"), black_box(&data)).await;
            black_box(result)
        })
    });

    group.bench_function("get_object_1kb", |b| {
        b.to_async(&rt).iter(|| async {
            let result = storage.get_object(black_box("test-key")).await;
            black_box(result)
        })
    });

    // Medium object (1MB)
    group.throughput(Throughput::Bytes(1024 * 1024));
    group.bench_function("put_object_1mb", |b| {
        let data = vec![0u8; 1024 * 1024];
        b.to_async(&rt).iter(|| async {
            let result = storage.put_object(black_box("large-key"), black_box(&data)).await;
            black_box(result)
        })
    });

    group.bench_function("get_object_1mb", |b| {
        b.to_async(&rt).iter(|| async {
            let result = storage.get_object(black_box("large-key")).await;
            black_box(result)
        })
    });

    group.bench_function("object_exists", |b| {
        b.to_async(&rt).iter(|| async {
            let result = storage.object_exists(black_box("test-key")).await;
            black_box(result)
        })
    });

    group.bench_function("list_objects", |b| {
        b.to_async(&rt).iter(|| async {
            let result = storage.list_objects(black_box(Some("prefix/"))).await;
            black_box(result)
        })
    });

    group.finish();
}

fn bench_metrics_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let metrics = Arc::new(MockMetrics);

    let mut group = c.benchmark_group("metrics_operations");

    group.bench_function("export_single_metric", |b| {
        let metric = Metric {
            name: "scan_duration".to_string(),
            value: 123.45,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            dimensions: HashMap::new(),
            unit: Some("Milliseconds".to_string()),
        };
        b.to_async(&rt).iter(|| async {
            let result = metrics.export_metric(black_box(&metric)).await;
            black_box(result)
        })
    });

    for batch_size in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("export_batch_metrics", batch_size),
            batch_size,
            |b, &size| {
                let batch: Vec<Metric> = (0..size)
                    .map(|i| Metric {
                        name: format!("metric_{}", i),
                        value: i as f64,
                        timestamp: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        dimensions: HashMap::new(),
                        unit: Some("Count".to_string()),
                    })
                    .collect();
                b.to_async(&rt).iter(|| async {
                    let result = metrics.export_metrics(black_box(&batch)).await;
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn bench_logging_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let logger = Arc::new(MockLogger);

    let mut group = c.benchmark_group("logging_operations");

    group.bench_function("log_single_entry", |b| {
        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            level: "INFO".to_string(),
            message: "Test log message".to_string(),
            fields: HashMap::new(),
        };
        b.to_async(&rt).iter(|| async {
            let result = logger.log(black_box(&entry)).await;
            black_box(result)
        })
    });

    for batch_size in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("log_batch_entries", batch_size),
            batch_size,
            |b, &size| {
                let batch: Vec<LogEntry> = (0..size)
                    .map(|i| LogEntry {
                        timestamp: SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        level: "INFO".to_string(),
                        message: format!("Log message {}", i),
                        fields: HashMap::new(),
                    })
                    .collect();
                b.to_async(&rt).iter(|| async {
                    let result = logger.batch_log(black_box(&batch)).await;
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_secret_operations,
    bench_storage_operations,
    bench_metrics_operations,
    bench_logging_operations
);
criterion_main!(benches);
