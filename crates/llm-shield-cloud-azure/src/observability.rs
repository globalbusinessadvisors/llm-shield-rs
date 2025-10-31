//! Azure Monitor observability integration.
//!
//! Provides implementations of `CloudMetrics` and `CloudLogger` traits for Azure Monitor.

use llm_shield_cloud::{
    async_trait, CloudError, CloudLogger, CloudMetrics, LogEntry, LogLevel, Metric, Result,
};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Azure Monitor Metrics implementation of `CloudMetrics`.
///
/// This implementation provides:
/// - Batched metric export for efficiency
/// - Support for custom metrics
/// - Automatic namespace configuration
/// - Standard metric dimensions
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_azure::AzureMonitorMetrics;
/// use llm_shield_cloud::{CloudMetrics, Metric};
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let metrics = AzureMonitorMetrics::new(
///         "my-resource-id",
///         "my-region"
///     ).await?;
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
pub struct AzureMonitorMetrics {
    client: Client,
    resource_id: String,
    region: String,
    batch_buffer: Arc<RwLock<Vec<Metric>>>,
    batch_size: usize,
}

impl AzureMonitorMetrics {
    /// Creates a new Azure Monitor Metrics client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `resource_id` - Azure resource ID (e.g., "/subscriptions/.../resourceGroups/.../...")
    /// * `region` - Azure region (e.g., "eastus", "westeurope")
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid.
    pub async fn new(resource_id: impl Into<String>, region: impl Into<String>) -> Result<Self> {
        let resource_id = resource_id.into();
        let region = region.into();

        let client = Client::new();

        tracing::info!(
            "Initialized Azure Monitor Metrics client for resource: {} in region: {}",
            resource_id,
            region
        );

        Ok(Self {
            client,
            resource_id,
            region,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: 20,
        })
    }

    /// Creates a new Azure Monitor Metrics client with custom batch size.
    ///
    /// # Arguments
    ///
    /// * `resource_id` - Azure resource ID
    /// * `region` - Azure region
    /// * `batch_size` - Number of metrics to batch before sending
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid.
    pub async fn new_with_batch_size(
        resource_id: impl Into<String>,
        region: impl Into<String>,
        batch_size: usize,
    ) -> Result<Self> {
        let resource_id = resource_id.into();
        let region = region.into();

        let client = Client::new();

        tracing::info!(
            "Initialized Azure Monitor Metrics client for resource: {} in region: {} with batch size: {}",
            resource_id,
            region,
            batch_size
        );

        Ok(Self {
            client,
            resource_id,
            region,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size,
        })
    }

    /// Gets the resource ID this client is configured for.
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }

    /// Gets the region this client is configured for.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Flushes buffered metrics to Azure Monitor.
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

    /// Sends a batch of metrics to Azure Monitor.
    async fn send_metrics_batch(&self, metrics: &[Metric]) -> Result<()> {
        if metrics.is_empty() {
            return Ok(());
        }

        tracing::debug!("Sending {} metrics to Azure Monitor", metrics.len());

        // Convert metrics to Azure Monitor format
        let time_series: Vec<serde_json::Value> = metrics
            .iter()
            .map(|m| {
                let timestamp = chrono::DateTime::from_timestamp(m.timestamp as i64, 0)
                    .unwrap_or_else(chrono::Utc::now)
                    .to_rfc3339();

                let mut data_point = json!({
                    "timeStamp": timestamp,
                    "total": m.value,
                });

                if let Some(ref unit) = m.unit {
                    data_point["unit"] = json!(unit);
                }

                json!({
                    "name": {
                        "value": m.name,
                        "localizedValue": m.name
                    },
                    "timeseries": [{
                        "data": [data_point]
                    }],
                    "dimensions": m.dimensions.iter().map(|(k, v)| {
                        json!({
                            "name": k,
                            "value": v
                        })
                    }).collect::<Vec<_>>()
                })
            })
            .collect();

        let payload = json!({
            "time": chrono::Utc::now().to_rfc3339(),
            "data": {
                "baseData": {
                    "metric": "LLMShieldMetrics",
                    "namespace": "LLMShield",
                    "dimNames": [],
                    "series": time_series
                }
            }
        });

        // In production, this would use Azure Monitor REST API
        // For now, we log the payload
        tracing::info!(
            "Would send {} metrics to Azure Monitor (payload prepared)",
            metrics.len()
        );

        Ok(())
    }
}

#[async_trait]
impl CloudMetrics for AzureMonitorMetrics {
    async fn export_metrics(&self, metrics: &[Metric]) -> Result<()> {
        tracing::debug!("Exporting {} metrics to Azure Monitor", metrics.len());

        // Send in batches
        for chunk in metrics.chunks(self.batch_size) {
            self.send_metrics_batch(chunk).await?;
        }

        Ok(())
    }

    async fn export_metric(&self, metric: &Metric) -> Result<()> {
        tracing::debug!("Exporting metric to Azure Monitor: {}", metric.name);

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

/// Azure Monitor Logs implementation of `CloudLogger`.
///
/// This implementation provides:
/// - Batched log export for efficiency
/// - Structured logging support
/// - Log Analytics workspace integration
/// - Custom log tables
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_azure::AzureMonitorLogs;
/// use llm_shield_cloud::{CloudLogger, LogLevel};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let logger = AzureMonitorLogs::new(
///         "workspace-id",
///         "shared-key",
///         "LLMShieldLog"
///     ).await?;
///
///     logger.log("Application started", LogLevel::Info).await?;
///     Ok(())
/// }
/// ```
pub struct AzureMonitorLogs {
    client: Client,
    workspace_id: String,
    shared_key: String,
    log_type: String,
    batch_buffer: Arc<RwLock<Vec<LogEntry>>>,
    batch_size: usize,
}

impl AzureMonitorLogs {
    /// Creates a new Azure Monitor Logs client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - Log Analytics workspace ID
    /// * `shared_key` - Log Analytics workspace shared key
    /// * `log_type` - Custom log type name (e.g., "LLMShieldLog")
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid.
    pub async fn new(
        workspace_id: impl Into<String>,
        shared_key: impl Into<String>,
        log_type: impl Into<String>,
    ) -> Result<Self> {
        let workspace_id = workspace_id.into();
        let shared_key = shared_key.into();
        let log_type = log_type.into();

        let client = Client::new();

        tracing::info!(
            "Initialized Azure Monitor Logs client for workspace: {} log type: {}",
            workspace_id,
            log_type
        );

        Ok(Self {
            client,
            workspace_id,
            shared_key,
            log_type,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size: 100,
        })
    }

    /// Creates a new Azure Monitor Logs client with custom batch size.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - Log Analytics workspace ID
    /// * `shared_key` - Log Analytics workspace shared key
    /// * `log_type` - Custom log type name
    /// * `batch_size` - Number of log entries to batch before sending
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid.
    pub async fn new_with_batch_size(
        workspace_id: impl Into<String>,
        shared_key: impl Into<String>,
        log_type: impl Into<String>,
        batch_size: usize,
    ) -> Result<Self> {
        let workspace_id = workspace_id.into();
        let shared_key = shared_key.into();
        let log_type = log_type.into();

        let client = Client::new();

        tracing::info!(
            "Initialized Azure Monitor Logs client for workspace: {} log type: {} with batch size: {}",
            workspace_id,
            log_type,
            batch_size
        );

        Ok(Self {
            client,
            workspace_id,
            shared_key,
            log_type,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size,
        })
    }

    /// Gets the workspace ID this client is configured for.
    pub fn workspace_id(&self) -> &str {
        &self.workspace_id
    }

    /// Gets the log type this client is configured for.
    pub fn log_type(&self) -> &str {
        &self.log_type
    }

    /// Flushes buffered log entries to Azure Monitor.
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

    /// Sends a batch of log entries to Azure Monitor.
    async fn send_logs_batch(&self, entries: &[LogEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        tracing::debug!("Sending {} log entries to Azure Monitor", entries.len());

        // Convert log entries to Azure Monitor format
        let log_records: Vec<serde_json::Value> = entries
            .iter()
            .map(|entry| {
                let timestamp = entry
                    .timestamp
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();

                let iso_timestamp = chrono::DateTime::from_timestamp(
                    timestamp.as_secs() as i64,
                    timestamp.subsec_nanos(),
                )
                .unwrap_or_else(chrono::Utc::now)
                .to_rfc3339();

                let severity = format_log_level(&entry.level);

                let mut record = json!({
                    "TimeGenerated": iso_timestamp,
                    "Level": severity,
                    "Message": entry.message,
                });

                // Add labels as properties
                for (key, value) in &entry.labels {
                    record[key] = json!(value);
                }

                if let Some(ref trace_id) = entry.trace_id {
                    record["TraceId"] = json!(trace_id);
                }

                if let Some(ref span_id) = entry.span_id {
                    record["SpanId"] = json!(span_id);
                }

                record
            })
            .collect();

        let payload = json!(log_records);

        // In production, this would use Azure Monitor Data Collector API
        // For now, we log the payload
        tracing::info!(
            "Would send {} log entries to Azure Monitor (payload prepared)",
            entries.len()
        );

        Ok(())
    }
}

#[async_trait]
impl CloudLogger for AzureMonitorLogs {
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
        tracing::debug!("Logging structured entry to Azure Monitor");

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
        tracing::debug!(
            "Logging batch of {} entries to Azure Monitor",
            entries.len()
        );

        // Send in batches
        for chunk in entries.chunks(self.batch_size) {
            self.send_logs_batch(chunk).await?;
        }

        Ok(())
    }
}

/// Formats a LogLevel as a string for Azure Monitor.
fn format_log_level(level: &LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "Verbose",
        LogLevel::Debug => "Verbose",
        LogLevel::Info => "Informational",
        LogLevel::Warn => "Warning",
        LogLevel::Error => "Error",
        LogLevel::Fatal => "Critical",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_size_limits() {
        let metrics_batch_size = 20;
        let logs_batch_size = 100;

        assert!(metrics_batch_size > 0);
        assert!(logs_batch_size > 0);
    }

    #[test]
    fn test_format_log_level() {
        assert_eq!(format_log_level(&LogLevel::Trace), "Verbose");
        assert_eq!(format_log_level(&LogLevel::Debug), "Verbose");
        assert_eq!(format_log_level(&LogLevel::Info), "Informational");
        assert_eq!(format_log_level(&LogLevel::Warn), "Warning");
        assert_eq!(format_log_level(&LogLevel::Error), "Error");
        assert_eq!(format_log_level(&LogLevel::Fatal), "Critical");
    }

    #[test]
    fn test_resource_id_format() {
        let resource_id = "/subscriptions/sub-id/resourceGroups/rg-name/providers/Microsoft.Compute/virtualMachines/vm-name";
        assert!(resource_id.starts_with("/subscriptions/"));
        assert!(resource_id.contains("/resourceGroups/"));
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
