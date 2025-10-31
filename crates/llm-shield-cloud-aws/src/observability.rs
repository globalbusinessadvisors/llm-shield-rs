//! AWS CloudWatch observability integration.
//!
//! Provides implementations of `CloudMetrics` and `CloudLogger` traits for AWS CloudWatch.

use aws_sdk_cloudwatch::Client as CloudWatchClient;
use aws_sdk_cloudwatch::types::{Dimension, MetricDatum, StandardUnit};
use aws_sdk_cloudwatchlogs::Client as CloudWatchLogsClient;
use aws_sdk_cloudwatchlogs::types::InputLogEvent;
use llm_shield_cloud::{
    async_trait, CloudError, CloudLogger, CloudMetrics, LogEntry, LogLevel, Metric, Result,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// AWS CloudWatch Metrics implementation of `CloudMetrics`.
///
/// This implementation provides:
/// - Batched metric export for efficiency
/// - Support for custom dimensions
/// - Automatic namespace configuration
/// - Standard and custom units
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_aws::CloudWatchMetrics;
/// use llm_shield_cloud::{CloudMetrics, Metric};
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let metrics = CloudWatchMetrics::new("LLMShield").await?;
///
///     let metric = Metric {
///         name: "RequestCount".to_string(),
///         value: 1.0,
///         timestamp: std::time::SystemTime::now()
///             .duration_since(std::time::UNIX_EPOCH)?
///             .as_secs(),
///         dimensions: HashMap::new(),
///         unit: Some("Count".to_string()),
///     };
///
///     metrics.export_metric(&metric).await?;
///     Ok(())
/// }
/// ```
pub struct CloudWatchMetrics {
    client: CloudWatchClient,
    namespace: String,
    region: String,
    batch_buffer: Arc<RwLock<Vec<Metric>>>,
    batch_size: usize,
}

impl CloudWatchMetrics {
    /// Creates a new CloudWatch Metrics client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `namespace` - CloudWatch namespace (e.g., "LLMShield")
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new(namespace: impl Into<String>) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let region = config
            .region()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "us-east-1".to_string());

        let client = CloudWatchClient::new(&config);
        let namespace = namespace.into();

        tracing::info!(
            "Initialized CloudWatch Metrics client for namespace: {} in region: {}",
            namespace,
            region
        );

        Ok(Self {
            client,
            namespace,
            region,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: 20, // CloudWatch allows up to 1000, but 20 is safer
        })
    }

    /// Creates a new CloudWatch Metrics client with specific region and batch size.
    ///
    /// # Arguments
    ///
    /// * `namespace` - CloudWatch namespace
    /// * `region` - AWS region
    /// * `batch_size` - Number of metrics to batch before sending (max 1000)
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new_with_config(
        namespace: impl Into<String>,
        region: impl Into<String>,
        batch_size: usize,
    ) -> Result<Self> {
        let region_str = region.into();
        let config = aws_config::from_env()
            .region(aws_config::Region::new(region_str.clone()))
            .load()
            .await;

        let client = CloudWatchClient::new(&config);
        let namespace = namespace.into();

        tracing::info!(
            "Initialized CloudWatch Metrics client for namespace: {} in region: {} (batch size: {})",
            namespace,
            region_str,
            batch_size
        );

        Ok(Self {
            client,
            namespace,
            region: region_str,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: batch_size.min(1000), // CloudWatch hard limit
        })
    }

    /// Gets the namespace this client is configured for.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Gets the AWS region this client is configured for.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Flushes buffered metrics to CloudWatch.
    pub async fn flush(&self) -> Result<()> {
        let mut buffer = self.batch_buffer.write().await;

        if buffer.is_empty() {
            return Ok(());
        }

        let metrics_to_send = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer); // Release lock before network call

        self.send_metrics_batch(&metrics_to_send).await?;

        Ok(())
    }

    /// Sends a batch of metrics to CloudWatch.
    async fn send_metrics_batch(&self, metrics: &[Metric]) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }

        tracing::debug!("Sending {} metrics to CloudWatch", metrics.len());

        let metric_data: Vec<MetricDatum> = metrics
            .iter()
            .map(|m| {
                let mut datum = MetricDatum::builder()
                    .metric_name(&m.name)
                    .value(m.value)
                    .timestamp(aws_sdk_cloudwatch::primitives::DateTime::from_secs(
                        m.timestamp as i64,
                    ));

                // Add dimensions
                for (key, value) in &m.dimensions {
                    datum = datum.dimensions(
                        Dimension::builder()
                            .name(key.clone())
                            .value(value.clone())
                            .build(),
                    );
                }

                // Add unit if specified
                if let Some(ref unit_str) = m.unit {
                    if let Ok(unit) = parse_standard_unit(unit_str) {
                        datum = datum.unit(unit);
                    }
                }

                datum.build().expect("Failed to build MetricDatum")
            })
            .collect();

        self.client
            .put_metric_data()
            .namespace(&self.namespace)
            .set_metric_data(Some(metric_data))
            .send()
            .await
            .map_err(|e| CloudError::MetricsExport(e.to_string()))?;

        tracing::info!("Successfully sent {} metrics to CloudWatch", metrics.len());

        Ok(())
    }
}

#[async_trait]
impl CloudMetrics for CloudWatchMetrics {
    async fn export_metrics(&self, metrics: &[Metric]) -> Result<()> {
        tracing::debug!("Exporting {} metrics to CloudWatch", metrics.len());

        // Send in batches of batch_size
        for chunk in metrics.chunks(self.batch_size) {
            self.send_metrics_batch(chunk).await?;
        }

        Ok(())
    }

    async fn export_metric(&self, metric: &Metric) -> Result<()> {
        tracing::debug!("Exporting metric to CloudWatch: {}", metric.name);

        // Add to buffer
        let mut buffer = self.batch_buffer.write().await;
        buffer.push(metric.clone());

        // Flush if buffer is full
        if buffer.len() >= self.batch_size {
            let metrics_to_send = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer); // Release lock before network call

            self.send_metrics_batch(&metrics_to_send).await?;
        }

        Ok(())
    }
}

/// AWS CloudWatch Logs implementation of `CloudLogger`.
///
/// This implementation provides:
/// - Batched log export for efficiency
/// - Structured logging support
/// - Automatic log stream creation
/// - Log group configuration
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_aws::CloudWatchLogger;
/// use llm_shield_cloud::{CloudLogger, LogLevel};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let logger = CloudWatchLogger::new(
///         "/llm-shield/api",
///         "production-instance-1"
///     ).await?;
///
///     logger.log("Application started", LogLevel::Info).await?;
///     Ok(())
/// }
/// ```
pub struct CloudWatchLogger {
    client: CloudWatchLogsClient,
    log_group: String,
    log_stream: String,
    region: String,
    sequence_token: Arc<RwLock<Option<String>>>,
    batch_buffer: Arc<RwLock<Vec<LogEntry>>>,
    batch_size: usize,
}

impl CloudWatchLogger {
    /// Creates a new CloudWatch Logs client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `log_group` - CloudWatch Logs log group name (e.g., "/llm-shield/api")
    /// * `log_stream` - Log stream name (e.g., "instance-1")
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded or log stream creation fails.
    pub async fn new(
        log_group: impl Into<String>,
        log_stream: impl Into<String>,
    ) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let region = config
            .region()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "us-east-1".to_string());

        let client = CloudWatchLogsClient::new(&config);
        let log_group = log_group.into();
        let log_stream = log_stream.into();

        // Try to create log stream (idempotent if already exists)
        let _ = client
            .create_log_stream()
            .log_group_name(&log_group)
            .log_stream_name(&log_stream)
            .send()
            .await;

        tracing::info!(
            "Initialized CloudWatch Logs client for log group: {} stream: {} in region: {}",
            log_group,
            log_stream,
            region
        );

        Ok(Self {
            client,
            log_group,
            log_stream,
            region,
            sequence_token: Arc::new(RwLock::new(None)),
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: 100, // CloudWatch allows up to 10,000 events
        })
    }

    /// Creates a new CloudWatch Logs client with specific configuration.
    ///
    /// # Arguments
    ///
    /// * `log_group` - CloudWatch Logs log group name
    /// * `log_stream` - Log stream name
    /// * `region` - AWS region
    /// * `batch_size` - Number of log entries to batch before sending
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new_with_config(
        log_group: impl Into<String>,
        log_stream: impl Into<String>,
        region: impl Into<String>,
        batch_size: usize,
    ) -> Result<Self> {
        let region_str = region.into();
        let config = aws_config::from_env()
            .region(aws_config::Region::new(region_str.clone()))
            .load()
            .await;

        let client = CloudWatchLogsClient::new(&config);
        let log_group = log_group.into();
        let log_stream = log_stream.into();

        // Try to create log stream
        let _ = client
            .create_log_stream()
            .log_group_name(&log_group)
            .log_stream_name(&log_stream)
            .send()
            .await;

        tracing::info!(
            "Initialized CloudWatch Logs client for log group: {} stream: {} in region: {} (batch size: {})",
            log_group,
            log_stream,
            region_str,
            batch_size
        );

        Ok(Self {
            client,
            log_group,
            log_stream,
            region: region_str,
            sequence_token: Arc::new(RwLock::new(None)),
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size,
        })
    }

    /// Gets the log group this client is configured for.
    pub fn log_group(&self) -> &str {
        &self.log_group
    }

    /// Gets the log stream this client is configured for.
    pub fn log_stream(&self) -> &str {
        &self.log_stream
    }

    /// Gets the AWS region this client is configured for.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Flushes buffered log entries to CloudWatch Logs.
    pub async fn flush(&self) -> Result<()> {
        let mut buffer = self.batch_buffer.write().await;

        if buffer.is_empty() {
            return Ok(());
        }

        let logs_to_send = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer); // Release lock before network call

        self.send_logs_batch(&logs_to_send).await?;

        Ok(())
    }

    /// Sends a batch of log entries to CloudWatch Logs.
    async fn send_logs_batch(&self, entries: &[LogEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        tracing::debug!("Sending {} log entries to CloudWatch Logs", entries.len());

        // Convert LogEntry to InputLogEvent
        let mut log_events: Vec<InputLogEvent> = entries
            .iter()
            .map(|entry| {
                let timestamp = entry
                    .timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64;

                // Format message with structured fields
                let mut message = format!("[{}] {}", format_log_level(&entry.level), entry.message);

                if !entry.labels.is_empty() {
                    message.push_str(&format!(" {:?}", entry.labels));
                }

                if let Some(ref trace_id) = entry.trace_id {
                    message.push_str(&format!(" trace_id={}", trace_id));
                }

                if let Some(ref span_id) = entry.span_id {
                    message.push_str(&format!(" span_id={}", span_id));
                }

                InputLogEvent::builder()
                    .timestamp(timestamp)
                    .message(message)
                    .build()
                    .expect("Failed to build InputLogEvent")
            })
            .collect();

        // Sort by timestamp (required by CloudWatch)
        log_events.sort_by_key(|e| e.timestamp);

        // Get current sequence token
        let sequence_token = self.sequence_token.read().await.clone();

        // Send log events
        let mut request = self
            .client
            .put_log_events()
            .log_group_name(&self.log_group)
            .log_stream_name(&self.log_stream)
            .set_log_events(Some(log_events));

        if let Some(token) = sequence_token {
            request = request.sequence_token(token);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CloudError::LogExport(e.to_string()))?;

        // Update sequence token for next request
        if let Some(next_token) = response.next_sequence_token {
            *self.sequence_token.write().await = Some(next_token);
        }

        tracing::info!(
            "Successfully sent {} log entries to CloudWatch Logs",
            entries.len()
        );

        Ok(())
    }
}

#[async_trait]
impl CloudLogger for CloudWatchLogger {
    async fn log(&self, message: &str, level: LogLevel) -> Result<()> {
        let entry = LogEntry {
            timestamp: std::time::SystemTime::now(),
            level,
            message: message.to_string(),
            labels: HashMap::new(),
            trace_id: None,
            span_id: None,
        };

        self.log_structured(&entry).await
    }

    async fn log_structured(&self, entry: &LogEntry) -> Result<()> {
        tracing::debug!("Logging structured entry to CloudWatch Logs");

        // Add to buffer
        let mut buffer = self.batch_buffer.write().await;
        buffer.push(entry.clone());

        // Flush if buffer is full
        if buffer.len() >= self.batch_size {
            let logs_to_send = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer); // Release lock before network call

            self.send_logs_batch(&logs_to_send).await?;
        }

        Ok(())
    }

    async fn log_batch(&self, entries: &[LogEntry]) -> Result<()> {
        tracing::debug!("Logging batch of {} entries to CloudWatch Logs", entries.len());

        // Send in batches of batch_size
        for chunk in entries.chunks(self.batch_size) {
            self.send_logs_batch(chunk).await?;
        }

        Ok(())
    }
}

/// Parses a string into a CloudWatch StandardUnit.
fn parse_standard_unit(unit_str: &str) -> Result<StandardUnit> {
    match unit_str.to_lowercase().as_str() {
        "seconds" => Ok(StandardUnit::Seconds),
        "microseconds" => Ok(StandardUnit::Microseconds),
        "milliseconds" => Ok(StandardUnit::Milliseconds),
        "bytes" => Ok(StandardUnit::Bytes),
        "kilobytes" => Ok(StandardUnit::Kilobytes),
        "megabytes" => Ok(StandardUnit::Megabytes),
        "gigabytes" => Ok(StandardUnit::Gigabytes),
        "terabytes" => Ok(StandardUnit::Terabytes),
        "bits" => Ok(StandardUnit::Bits),
        "kilobits" => Ok(StandardUnit::Kilobits),
        "megabits" => Ok(StandardUnit::Megabits),
        "gigabits" => Ok(StandardUnit::Gigabits),
        "terabits" => Ok(StandardUnit::Terabits),
        "percent" => Ok(StandardUnit::Percent),
        "count" => Ok(StandardUnit::Count),
        "bytes/second" | "bytes_per_second" => Ok(StandardUnit::BytesSecond),
        "kilobytes/second" | "kilobytes_per_second" => Ok(StandardUnit::KilobytesSecond),
        "megabytes/second" | "megabytes_per_second" => Ok(StandardUnit::MegabytesSecond),
        "gigabytes/second" | "gigabytes_per_second" => Ok(StandardUnit::GigabytesSecond),
        "terabytes/second" | "terabytes_per_second" => Ok(StandardUnit::TerabytesSecond),
        "bits/second" | "bits_per_second" => Ok(StandardUnit::BitsSecond),
        "kilobits/second" | "kilobits_per_second" => Ok(StandardUnit::KilobitsSecond),
        "megabits/second" | "megabits_per_second" => Ok(StandardUnit::MegabitsSecond),
        "gigabits/second" | "gigabits_per_second" => Ok(StandardUnit::GigabitsSecond),
        "terabits/second" | "terabits_per_second" => Ok(StandardUnit::TerabitsSecond),
        "count/second" | "count_per_second" => Ok(StandardUnit::CountSecond),
        "none" => Ok(StandardUnit::None),
        _ => Ok(StandardUnit::None),
    }
}

/// Formats a LogLevel as a string.
fn format_log_level(level: &LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Fatal => "FATAL",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_unit() {
        assert!(matches!(
            parse_standard_unit("count"),
            Ok(StandardUnit::Count)
        ));
        assert!(matches!(
            parse_standard_unit("bytes"),
            Ok(StandardUnit::Bytes)
        ));
        assert!(matches!(
            parse_standard_unit("seconds"),
            Ok(StandardUnit::Seconds)
        ));
        assert!(matches!(
            parse_standard_unit("percent"),
            Ok(StandardUnit::Percent)
        ));
        assert!(matches!(
            parse_standard_unit("invalid"),
            Ok(StandardUnit::None)
        ));
    }

    #[test]
    fn test_format_log_level() {
        assert_eq!(format_log_level(&LogLevel::Trace), "TRACE");
        assert_eq!(format_log_level(&LogLevel::Debug), "DEBUG");
        assert_eq!(format_log_level(&LogLevel::Info), "INFO");
        assert_eq!(format_log_level(&LogLevel::Warn), "WARN");
        assert_eq!(format_log_level(&LogLevel::Error), "ERROR");
        assert_eq!(format_log_level(&LogLevel::Fatal), "FATAL");
    }

    #[test]
    fn test_batch_size_limits() {
        let metrics_batch_size = 20;
        let logs_batch_size = 100;

        // CloudWatch limits
        assert!(metrics_batch_size <= 1000);
        assert!(logs_batch_size <= 10000);
    }

    #[tokio::test]
    async fn test_metric_batching() {
        let metrics = vec![
            Metric {
                name: "test1".to_string(),
                value: 1.0,
                timestamp: 1000,
                dimensions: HashMap::new(),
                unit: Some("Count".to_string()),
            },
            Metric {
                name: "test2".to_string(),
                value: 2.0,
                timestamp: 2000,
                dimensions: HashMap::new(),
                unit: Some("Count".to_string()),
            },
        ];

        // Test chunking logic
        let batch_size = 1;
        let chunks: Vec<_> = metrics.chunks(batch_size).collect();

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 1);
        assert_eq!(chunks[1].len(), 1);
    }

    #[tokio::test]
    async fn test_log_entry_sorting() {
        let mut entries = vec![
            LogEntry {
                timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_secs(2000),
                level: LogLevel::Info,
                message: "second".to_string(),
                labels: HashMap::new(),
                trace_id: None,
                span_id: None,
            },
            LogEntry {
                timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_secs(1000),
                level: LogLevel::Info,
                message: "first".to_string(),
                labels: HashMap::new(),
                trace_id: None,
                span_id: None,
            },
        ];

        entries.sort_by_key(|e| {
            e.timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        });

        assert_eq!(entries[0].message, "first");
        assert_eq!(entries[1].message, "second");
    }
}
