//! Integration tests for AWS CloudWatch Metrics and Logs.
//!
//! These tests require:
//! - AWS credentials configured (environment, file, or IAM role)
//! - Permissions to create metrics and logs
//! - CloudWatch namespace `LLMShieldTest`
//! - Log group `/llm-shield-test/`
//!
//! Run with: cargo test --test integration_observability -- --ignored

use llm_shield_cloud::{CloudLogger, CloudMetrics, LogEntry, LogLevel, Metric};
use llm_shield_cloud_aws::{CloudWatchLogger, CloudWatchMetrics};
use std::collections::HashMap;

const TEST_NAMESPACE: &str = "LLMShieldTest";
const TEST_LOG_GROUP: &str = "/llm-shield-test/integration";

#[tokio::test]
#[ignore] // Requires AWS credentials
async fn test_export_single_metric() {
    let metrics = CloudWatchMetrics::new(TEST_NAMESPACE)
        .await
        .expect("Failed to initialize CloudWatchMetrics");

    let mut dimensions = HashMap::new();
    dimensions.insert("Environment".to_string(), "Test".to_string());
    dimensions.insert("TestType".to_string(), "Integration".to_string());

    let metric = Metric {
        name: "TestMetric".to_string(),
        value: 42.0,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        dimensions,
        unit: Some("Count".to_string()),
    };

    metrics
        .export_metric(&metric)
        .await
        .expect("Failed to export metric");

    // Flush to ensure metric is sent
    metrics.flush().await.expect("Failed to flush metrics");
}

#[tokio::test]
#[ignore]
async fn test_export_multiple_metrics() {
    let metrics = CloudWatchMetrics::new(TEST_NAMESPACE)
        .await
        .expect("Failed to initialize CloudWatchMetrics");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut dimensions = HashMap::new();
    dimensions.insert("Environment".to_string(), "Test".to_string());

    let metrics_batch = vec![
        Metric {
            name: "RequestCount".to_string(),
            value: 100.0,
            timestamp: now,
            dimensions: dimensions.clone(),
            unit: Some("Count".to_string()),
        },
        Metric {
            name: "ResponseTime".to_string(),
            value: 123.45,
            timestamp: now,
            dimensions: dimensions.clone(),
            unit: Some("Milliseconds".to_string()),
        },
        Metric {
            name: "ErrorRate".to_string(),
            value: 2.5,
            timestamp: now,
            dimensions: dimensions.clone(),
            unit: Some("Percent".to_string()),
        },
    ];

    metrics
        .export_metrics(&metrics_batch)
        .await
        .expect("Failed to export metrics batch");
}

#[tokio::test]
#[ignore]
async fn test_metric_batching() {
    let metrics = CloudWatchMetrics::new_with_config(TEST_NAMESPACE, "us-east-1", 5)
        .await
        .expect("Failed to initialize CloudWatchMetrics");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Export 10 metrics with batch size of 5
    // Should result in 2 batches
    for i in 0..10 {
        let metric = Metric {
            name: format!("BatchTest{}", i),
            value: i as f64,
            timestamp: now,
            dimensions: HashMap::new(),
            unit: Some("Count".to_string()),
        };

        metrics
            .export_metric(&metric)
            .await
            .expect("Failed to export metric");
    }

    // Flush remaining metrics
    metrics.flush().await.expect("Failed to flush metrics");
}

#[tokio::test]
#[ignore]
async fn test_metric_units() {
    let metrics = CloudWatchMetrics::new(TEST_NAMESPACE)
        .await
        .expect("Failed to initialize CloudWatchMetrics");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let test_units = vec![
        ("Seconds", 1.5),
        ("Milliseconds", 150.0),
        ("Bytes", 1024.0),
        ("Kilobytes", 1.0),
        ("Megabytes", 50.0),
        ("Percent", 99.9),
        ("Count", 42.0),
    ];

    for (unit, value) in test_units {
        let metric = Metric {
            name: format!("UnitTest_{}", unit),
            value,
            timestamp: now,
            dimensions: HashMap::new(),
            unit: Some(unit.to_string()),
        };

        metrics
            .export_metric(&metric)
            .await
            .expect("Failed to export metric with unit");
    }

    metrics.flush().await.expect("Failed to flush metrics");
}

#[tokio::test]
#[ignore]
async fn test_log_simple_message() {
    let log_stream = format!("integration-test-{}", uuid::Uuid::new_v4());

    let logger = CloudWatchLogger::new(TEST_LOG_GROUP, &log_stream)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    logger
        .log("This is a test log message", LogLevel::Info)
        .await
        .expect("Failed to log message");

    // Flush to ensure log is sent
    logger.flush().await.expect("Failed to flush logs");
}

#[tokio::test]
#[ignore]
async fn test_log_all_levels() {
    let log_stream = format!("levels-test-{}", uuid::Uuid::new_v4());

    let logger = CloudWatchLogger::new(TEST_LOG_GROUP, &log_stream)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    logger
        .log("Trace level message", LogLevel::Trace)
        .await
        .expect("Failed to log trace");
    logger
        .log("Debug level message", LogLevel::Debug)
        .await
        .expect("Failed to log debug");
    logger
        .log("Info level message", LogLevel::Info)
        .await
        .expect("Failed to log info");
    logger
        .log("Warn level message", LogLevel::Warn)
        .await
        .expect("Failed to log warn");
    logger
        .log("Error level message", LogLevel::Error)
        .await
        .expect("Failed to log error");
    logger
        .log("Fatal level message", LogLevel::Fatal)
        .await
        .expect("Failed to log fatal");

    logger.flush().await.expect("Failed to flush logs");
}

#[tokio::test]
#[ignore]
async fn test_structured_logging() {
    let log_stream = format!("structured-test-{}", uuid::Uuid::new_v4());

    let logger = CloudWatchLogger::new(TEST_LOG_GROUP, &log_stream)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    let mut labels = HashMap::new();
    labels.insert("request_id".to_string(), "req-12345".to_string());
    labels.insert("user_id".to_string(), "user-67890".to_string());
    labels.insert("endpoint".to_string(), "/api/v1/scan".to_string());

    let entry = LogEntry {
        timestamp: std::time::SystemTime::now(),
        level: LogLevel::Info,
        message: "API request processed successfully".to_string(),
        labels,
        trace_id: Some("trace-abc123".to_string()),
        span_id: Some("span-def456".to_string()),
    };

    logger
        .log_structured(&entry)
        .await
        .expect("Failed to log structured entry");

    logger.flush().await.expect("Failed to flush logs");
}

#[tokio::test]
#[ignore]
async fn test_log_batch() {
    let log_stream = format!("batch-test-{}", uuid::Uuid::new_v4());

    let logger = CloudWatchLogger::new(TEST_LOG_GROUP, &log_stream)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    let mut entries = Vec::new();

    for i in 0..50 {
        let mut labels = HashMap::new();
        labels.insert("iteration".to_string(), i.to_string());

        entries.push(LogEntry {
            timestamp: std::time::SystemTime::now(),
            level: LogLevel::Info,
            message: format!("Batch log entry {}", i),
            labels,
            trace_id: None,
            span_id: None,
        });

        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    logger
        .log_batch(&entries)
        .await
        .expect("Failed to log batch");
}

#[tokio::test]
#[ignore]
async fn test_log_buffering() {
    let log_stream = format!("buffering-test-{}", uuid::Uuid::new_v4());

    // Create logger with small batch size
    let logger = CloudWatchLogger::new_with_config(TEST_LOG_GROUP, &log_stream, "us-east-1", 10)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    // Log 25 messages (should trigger 2 automatic flushes + remainder in buffer)
    for i in 0..25 {
        logger
            .log(&format!("Buffered message {}", i), LogLevel::Info)
            .await
            .expect("Failed to log message");
    }

    // Flush remaining messages
    logger.flush().await.expect("Failed to flush logs");
}

#[tokio::test]
#[ignore]
async fn test_concurrent_logging() {
    let log_stream = format!("concurrent-test-{}", uuid::Uuid::new_v4());

    let logger = CloudWatchLogger::new(TEST_LOG_GROUP, &log_stream)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    let logger = std::sync::Arc::new(logger);

    let mut handles = vec![];

    // Spawn 10 concurrent tasks logging messages
    for task_id in 0..10 {
        let logger_clone = logger.clone();
        let handle = tokio::spawn(async move {
            for i in 0..10 {
                let mut labels = HashMap::new();
                labels.insert("task_id".to_string(), task_id.to_string());
                labels.insert("message_id".to_string(), i.to_string());

                let entry = LogEntry {
                    timestamp: std::time::SystemTime::now(),
                    level: LogLevel::Info,
                    message: format!("Concurrent log from task {} message {}", task_id, i),
                    labels,
                    trace_id: None,
                    span_id: None,
                };

                logger_clone
                    .log_structured(&entry)
                    .await
                    .expect("Failed to log from concurrent task");

                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task failed");
    }

    // Flush remaining logs
    logger.flush().await.expect("Failed to flush logs");
}

#[tokio::test]
#[ignore]
async fn test_metrics_and_logs_together() {
    let metrics = CloudWatchMetrics::new(TEST_NAMESPACE)
        .await
        .expect("Failed to initialize CloudWatchMetrics");

    let log_stream = format!("combined-test-{}", uuid::Uuid::new_v4());
    let logger = CloudWatchLogger::new(TEST_LOG_GROUP, &log_stream)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Emit metric
    let metric = Metric {
        name: "CombinedTest".to_string(),
        value: 123.0,
        timestamp: now,
        dimensions: HashMap::new(),
        unit: Some("Count".to_string()),
    };

    metrics
        .export_metric(&metric)
        .await
        .expect("Failed to export metric");

    // Log corresponding message
    logger
        .log("Emitted metric CombinedTest with value 123", LogLevel::Info)
        .await
        .expect("Failed to log message");

    // Flush both
    metrics.flush().await.expect("Failed to flush metrics");
    logger.flush().await.expect("Failed to flush logs");
}

#[tokio::test]
#[ignore]
async fn test_region_configuration() {
    // Test metrics with specific region
    let metrics = CloudWatchMetrics::new_with_config(TEST_NAMESPACE, "us-west-2", 20)
        .await
        .expect("Failed to initialize CloudWatchMetrics with region");

    assert_eq!(metrics.region(), "us-west-2");
    assert_eq!(metrics.namespace(), TEST_NAMESPACE);

    // Test logger with specific region
    let log_stream = format!("region-test-{}", uuid::Uuid::new_v4());
    let logger = CloudWatchLogger::new_with_config(TEST_LOG_GROUP, &log_stream, "us-west-2", 100)
        .await
        .expect("Failed to initialize CloudWatchLogger with region");

    assert_eq!(logger.region(), "us-west-2");
    assert_eq!(logger.log_group(), TEST_LOG_GROUP);
    assert_eq!(logger.log_stream(), log_stream);

    // Test operations work
    let metric = Metric {
        name: "RegionTest".to_string(),
        value: 1.0,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        dimensions: HashMap::new(),
        unit: Some("Count".to_string()),
    };

    metrics
        .export_metric(&metric)
        .await
        .expect("Failed to export metric in us-west-2");

    logger
        .log("Test log in us-west-2", LogLevel::Info)
        .await
        .expect("Failed to log in us-west-2");

    metrics.flush().await.expect("Failed to flush metrics");
    logger.flush().await.expect("Failed to flush logs");
}

#[tokio::test]
#[ignore]
async fn test_high_volume_metrics() {
    let metrics = CloudWatchMetrics::new(TEST_NAMESPACE)
        .await
        .expect("Failed to initialize CloudWatchMetrics");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let start = std::time::Instant::now();

    // Send 1000 metrics
    for i in 0..1000 {
        let metric = Metric {
            name: "HighVolumeTest".to_string(),
            value: i as f64,
            timestamp: now,
            dimensions: HashMap::new(),
            unit: Some("Count".to_string()),
        };

        metrics
            .export_metric(&metric)
            .await
            .expect("Failed to export metric");
    }

    metrics.flush().await.expect("Failed to flush metrics");

    let duration = start.elapsed();
    println!("Sent 1000 metrics in {:?}", duration);
    println!(
        "Average: {:?} per metric",
        duration / 1000
    );
}

#[tokio::test]
#[ignore]
async fn test_high_volume_logs() {
    let log_stream = format!("high-volume-test-{}", uuid::Uuid::new_v4());
    let logger = CloudWatchLogger::new(TEST_LOG_GROUP, &log_stream)
        .await
        .expect("Failed to initialize CloudWatchLogger");

    let start = std::time::Instant::now();

    // Send 1000 log messages
    for i in 0..1000 {
        logger
            .log(&format!("High volume log message {}", i), LogLevel::Info)
            .await
            .expect("Failed to log message");
    }

    logger.flush().await.expect("Failed to flush logs");

    let duration = start.elapsed();
    println!("Sent 1000 log messages in {:?}", duration);
    println!("Average: {:?} per message", duration / 1000);
}
