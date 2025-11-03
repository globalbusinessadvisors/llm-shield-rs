//! Stub implementation for GCP observability (monitoring and logging).
//!
//! NOTE: This is a temporary stub due to breaking changes in google-cloud SDK.
//! Full implementation requires updating to the latest SDK APIs.

use async_trait::async_trait;
use llm_shield_cloud::error::{CloudError, Result};
use llm_shield_cloud::observability::{CloudLogger, CloudMetrics, CloudTracer, LogEntry, LogLevel, Metric, Span};

/// Stub implementation for GCP Cloud Monitoring.
pub struct GcpCloudMonitoring;

impl GcpCloudMonitoring {
    /// Creates a new GCP Cloud Monitoring client (stub).
    pub async fn new(_project_id: impl Into<String>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudMetrics for GcpCloudMonitoring {
    async fn export_metrics(&self, _metrics: &[Metric]) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Monitoring not implemented - SDK API breaking changes".to_string(),
        ))
    }
}

/// Stub implementation for GCP Cloud Logging.
pub struct GcpCloudLogging;

impl GcpCloudLogging {
    /// Creates a new GCP Cloud Logging client (stub).
    pub async fn new(_project_id: impl Into<String>, _log_name: impl Into<String>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudLogger for GcpCloudLogging {
    async fn log(&self, _message: &str, _level: LogLevel) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Logging not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn log_structured(&self, _entry: &LogEntry) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Logging not implemented - SDK API breaking changes".to_string(),
        ))
    }
}

/// Stub implementation for GCP tracing.
pub struct GcpCloudTracer;

#[async_trait]
impl CloudTracer for GcpCloudTracer {
    async fn end_span(&self, _span: Span) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Tracing not implemented - SDK API breaking changes".to_string(),
        ))
    }
}
