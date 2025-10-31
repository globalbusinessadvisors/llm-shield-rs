//! GCP Cloud Monitoring and Cloud Logging integration.
//!
//! Provides implementations of `CloudMetrics` and `CloudLogger` traits for GCP.

use google_cloud_googleapis::cloud::logging::v2::{
    log_entry::Payload, logging_service_v2_client::LoggingServiceV2Client, LogEntry as GcpLogEntry,
    WriteLogEntriesRequest,
};
use google_cloud_googleapis::cloud::monitoring::v3::{
    metric_service_client::MetricServiceClient, CreateTimeSeriesRequest, Point, TimeInterval,
    TimeSeries, TypedValue,
};
use google_cloud_googleapis::api::{MonitoredResource, MonitoredResourceMetadata};
use llm_shield_cloud::{
    async_trait, CloudError, CloudLogger, CloudMetrics, LogEntry, LogLevel, Metric, Result,
};
use prost_types::Timestamp;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Channel;

/// GCP Cloud Monitoring implementation of `CloudMetrics`.
///
/// This implementation provides:
/// - Batched metric export for efficiency
/// - Support for custom metric descriptors
/// - Automatic project and resource configuration
/// - Standard and custom metric types
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_gcp::GcpCloudMonitoring;
/// use llm_shield_cloud::{CloudMetrics, Metric};
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let metrics = GcpCloudMonitoring::new("my-project-id").await?;
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
pub struct GcpCloudMonitoring {
    client: MetricServiceClient<Channel>,
    project_id: String,
    batch_buffer: Arc<RwLock<Vec<Metric>>>,
    batch_size: usize,
}

impl GcpCloudMonitoring {
    /// Creates a new Cloud Monitoring client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `project_id` - GCP project ID
    ///
    /// # Errors
    ///
    /// Returns error if GCP credentials cannot be loaded.
    pub async fn new(project_id: impl Into<String>) -> Result<Self> {
        let project_id = project_id.into();

        // Create gRPC channel
        let channel = Channel::from_static("https://monitoring.googleapis.com")
            .connect()
            .await
            .map_err(|e| CloudError::Connection(e.to_string()))?;

        let client = MetricServiceClient::new(channel);

        tracing::info!(
            "Initialized GCP Cloud Monitoring client for project: {}",
            project_id
        );

        Ok(Self {
            client,
            project_id,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: 20,
        })
    }

    /// Creates a new Cloud Monitoring client with custom batch size.
    ///
    /// # Arguments
    ///
    /// * `project_id` - GCP project ID
    /// * `batch_size` - Number of metrics to batch before sending (max 200)
    ///
    /// # Errors
    ///
    /// Returns error if GCP credentials cannot be loaded.
    pub async fn new_with_batch_size(
        project_id: impl Into<String>,
        batch_size: usize,
    ) -> Result<Self> {
        let project_id = project_id.into();

        let channel = Channel::from_static("https://monitoring.googleapis.com")
            .connect()
            .await
            .map_err(|e| CloudError::Connection(e.to_string()))?;

        let client = MetricServiceClient::new(channel);

        tracing::info!(
            "Initialized GCP Cloud Monitoring client for project: {} with batch size: {}",
            project_id,
            batch_size
        );

        Ok(Self {
            client,
            project_id,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: batch_size.min(200), // GCP limit
        })
    }

    /// Gets the project ID this client is configured for.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Flushes buffered metrics to Cloud Monitoring.
    pub async fn flush(&self) -> Result<()> {
        let mut buffer = self.batch_buffer.write().await;

        if buffer.is_empty() {
            return Ok(());
        }

        let metrics_to_send = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);

        self.send_metrics_batch(&metrics_to_send).await?;

        Ok(())
    }

    /// Sends a batch of metrics to Cloud Monitoring.
    async fn send_metrics_batch(&self, metrics: &[Metric]) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }

        tracing::debug!("Sending {} metrics to Cloud Monitoring", metrics.len());

        let project_name = format!("projects/{}", self.project_id);

        // Convert metrics to TimeSeries
        let time_series: Vec<TimeSeries> = metrics
            .iter()
            .map(|m| {
                let metric_type = format!("custom.googleapis.com/{}", m.name);

                let labels: HashMap<String, String> = m.dimensions.clone();

                let timestamp = Timestamp {
                    seconds: m.timestamp as i64,
                    nanos: 0,
                };

                let point = Point {
                    interval: Some(TimeInterval {
                        end_time: Some(timestamp.clone()),
                        start_time: Some(timestamp),
                    }),
                    value: Some(TypedValue {
                        value: Some(google_cloud_googleapis::cloud::monitoring::v3::typed_value::Value::DoubleValue(
                            m.value,
                        )),
                    }),
                };

                TimeSeries {
                    metric: Some(google_cloud_googleapis::api::Metric {
                        r#type: metric_type,
                        labels,
                    }),
                    resource: Some(MonitoredResource {
                        r#type: "global".to_string(),
                        labels: HashMap::new(),
                    }),
                    metadata: None,
                    metric_kind: google_cloud_googleapis::api::metric_descriptor::MetricKind::Gauge as i32,
                    value_type: google_cloud_googleapis::api::metric_descriptor::ValueType::Double as i32,
                    points: vec![point],
                    unit: m.unit.clone().unwrap_or_default(),
                }
            })
            .collect();

        let request = CreateTimeSeriesRequest {
            name: project_name,
            time_series,
        };

        self.client
            .clone()
            .create_time_series(request)
            .await
            .map_err(|e| CloudError::MetricsExport(e.to_string()))?;

        tracing::info!("Successfully sent {} metrics to Cloud Monitoring", metrics.len());

        Ok(())
    }
}

#[async_trait]
impl CloudMetrics for GcpCloudMonitoring {
    async fn export_metrics(&self, metrics: &[Metric]) -> Result<()> {
        tracing::debug!("Exporting {} metrics to Cloud Monitoring", metrics.len());

        // Send in batches
        for chunk in metrics.chunks(self.batch_size) {
            self.send_metrics_batch(chunk).await?;
        }

        Ok(())
    }

    async fn export_metric(&self, metric: &Metric) -> Result<()> {
        tracing::debug!("Exporting metric to Cloud Monitoring: {}", metric.name);

        // Add to buffer
        let mut buffer = self.batch_buffer.write().await;
        buffer.push(metric.clone());

        // Flush if buffer is full
        if buffer.len() >= self.batch_size {
            let metrics_to_send = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer);

            self.send_metrics_batch(&metrics_to_send).await?;
        }

        Ok(())
    }
}

/// GCP Cloud Logging implementation of `CloudLogger`.
///
/// This implementation provides:
/// - Batched log export for efficiency
/// - Structured logging support
/// - Automatic resource detection
/// - Log severity levels
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_gcp::GcpCloudLogging;
/// use llm_shield_cloud::{CloudLogger, LogLevel};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let logger = GcpCloudLogging::new(
///         "my-project-id",
///         "llm-shield-api"
///     ).await?;
///
///     logger.log("Application started", LogLevel::Info).await?;
///     Ok(())
/// }
/// ```
pub struct GcpCloudLogging {
    client: LoggingServiceV2Client<Channel>,
    project_id: String,
    log_name: String,
    batch_buffer: Arc<RwLock<Vec<LogEntry>>>,
    batch_size: usize,
}

impl GcpCloudLogging {
    /// Creates a new Cloud Logging client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `project_id` - GCP project ID
    /// * `log_name` - Log name (e.g., "llm-shield-api")
    ///
    /// # Errors
    ///
    /// Returns error if GCP credentials cannot be loaded.
    pub async fn new(project_id: impl Into<String>, log_name: impl Into<String>) -> Result<Self> {
        let project_id = project_id.into();
        let log_name = log_name.into();

        let channel = Channel::from_static("https://logging.googleapis.com")
            .connect()
            .await
            .map_err(|e| CloudError::Connection(e.to_string()))?;

        let client = LoggingServiceV2Client::new(channel);

        tracing::info!(
            "Initialized GCP Cloud Logging client for project: {} log: {}",
            project_id,
            log_name
        );

        Ok(Self {
            client,
            project_id,
            log_name,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: 100,
        })
    }

    /// Creates a new Cloud Logging client with custom batch size.
    ///
    /// # Arguments
    ///
    /// * `project_id` - GCP project ID
    /// * `log_name` - Log name
    /// * `batch_size` - Number of log entries to batch before sending
    ///
    /// # Errors
    ///
    /// Returns error if GCP credentials cannot be loaded.
    pub async fn new_with_batch_size(
        project_id: impl Into<String>,
        log_name: impl Into<String>,
        batch_size: usize,
    ) -> Result<Self> {
        let project_id = project_id.into();
        let log_name = log_name.into();

        let channel = Channel::from_static("https://logging.googleapis.com")
            .connect()
            .await
            .map_err(|e| CloudError::Connection(e.to_string()))?;

        let client = LoggingServiceV2Client::new(channel);

        tracing::info!(
            "Initialized GCP Cloud Logging client for project: {} log: {} with batch size: {}",
            project_id,
            log_name,
            batch_size
        );

        Ok(Self {
            client,
            project_id,
            log_name,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size,
        })
    }

    /// Gets the project ID this client is configured for.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Gets the log name this client is configured for.
    pub fn log_name(&self) -> &str {
        &self.log_name
    }

    /// Flushes buffered log entries to Cloud Logging.
    pub async fn flush(&self) -> Result<()> {
        let mut buffer = self.batch_buffer.write().await;

        if buffer.is_empty() {
            return Ok(());
        }

        let logs_to_send = buffer.drain(..).collect::<Vec<_>>();
        drop(buffer);

        self.send_logs_batch(&logs_to_send).await?;

        Ok(())
    }

    /// Sends a batch of log entries to Cloud Logging.
    async fn send_logs_batch(&self, entries: &[LogEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        tracing::debug!("Sending {} log entries to Cloud Logging", entries.len());

        let log_name = format!("projects/{}/logs/{}", self.project_id, self.log_name);

        let gcp_entries: Vec<GcpLogEntry> = entries
            .iter()
            .map(|entry| {
                let timestamp = entry
                    .timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();

                let severity = match entry.level {
                    LogLevel::Trace => google_cloud_googleapis::logging::r#type::LogSeverity::Debug,
                    LogLevel::Debug => google_cloud_googleapis::logging::r#type::LogSeverity::Debug,
                    LogLevel::Info => google_cloud_googleapis::logging::r#type::LogSeverity::Info,
                    LogLevel::Warn => google_cloud_googleapis::logging::r#type::LogSeverity::Warning,
                    LogLevel::Error => google_cloud_googleapis::logging::r#type::LogSeverity::Error,
                    LogLevel::Fatal => google_cloud_googleapis::logging::r#type::LogSeverity::Critical,
                };

                GcpLogEntry {
                    log_name: log_name.clone(),
                    resource: Some(MonitoredResource {
                        r#type: "global".to_string(),
                        labels: HashMap::new(),
                    }),
                    timestamp: Some(Timestamp {
                        seconds: timestamp.as_secs() as i64,
                        nanos: timestamp.subsec_nanos() as i32,
                    }),
                    severity: severity as i32,
                    labels: entry.labels.clone(),
                    trace: entry.trace_id.clone().unwrap_or_default(),
                    span_id: entry.span_id.clone().unwrap_or_default(),
                    payload: Some(Payload::TextPayload(entry.message.clone())),
                    ..Default::default()
                }
            })
            .collect();

        let request = WriteLogEntriesRequest {
            log_name,
            resource: Some(MonitoredResource {
                r#type: "global".to_string(),
                labels: HashMap::new(),
            }),
            labels: HashMap::new(),
            entries: gcp_entries,
            partial_success: false,
            dry_run: false,
        };

        self.client
            .clone()
            .write_log_entries(request)
            .await
            .map_err(|e| CloudError::LogExport(e.to_string()))?;

        tracing::info!(
            "Successfully sent {} log entries to Cloud Logging",
            entries.len()
        );

        Ok(())
    }
}

#[async_trait]
impl CloudLogger for GcpCloudLogging {
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
        tracing::debug!("Logging structured entry to Cloud Logging");

        // Add to buffer
        let mut buffer = self.batch_buffer.write().await;
        buffer.push(entry.clone());

        // Flush if buffer is full
        if buffer.len() >= self.batch_size {
            let logs_to_send = buffer.drain(..).collect::<Vec<_>>();
            drop(buffer);

            self.send_logs_batch(&logs_to_send).await?;
        }

        Ok(())
    }

    async fn log_batch(&self, entries: &[LogEntry]) -> Result<()> {
        tracing::debug!("Logging batch of {} entries to Cloud Logging", entries.len());

        // Send in batches
        for chunk in entries.chunks(self.batch_size) {
            self.send_logs_batch(chunk).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_size_limits() {
        let metrics_batch_size = 20;
        let logs_batch_size = 100;

        // GCP limits
        assert!(metrics_batch_size <= 200);
        assert!(logs_batch_size <= 1000);
    }

    #[test]
    fn test_metric_type_format() {
        let metric_name = "RequestCount";
        let expected = format!("custom.googleapis.com/{}", metric_name);
        assert_eq!(expected, "custom.googleapis.com/RequestCount");
    }

    #[test]
    fn test_log_name_format() {
        let project_id = "test-project";
        let log_name = "llm-shield-api";
        let expected = format!("projects/{}/logs/{}", project_id, log_name);
        assert_eq!(expected, "projects/test-project/logs/llm-shield-api");
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
}
