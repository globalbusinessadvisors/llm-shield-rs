//! Stub implementation for Azure observability (monitoring and logging).
//!
//! NOTE: This is a temporary stub due to breaking changes in Azure SDK.
//! Full implementation requires updating to the latest SDK APIs.

use async_trait::async_trait;
use llm_shield_cloud::error::{CloudError, Result};
use llm_shield_cloud::observability::{CloudLogger, CloudMetrics, CloudTracer, LogEntry, LogLevel, Metric, Span};

/// Stub implementation for Azure Monitor.
pub struct AzureMonitor;

impl AzureMonitor {
    /// Creates a new Azure Monitor client (stub).
    pub async fn new() -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudMetrics for AzureMonitor {
    async fn export_metrics(&self, _metrics: &[Metric]) -> Result<()> {
        Err(CloudError::OperationFailed(
            "Azure Monitor not implemented - SDK API breaking changes".to_string(),
        ))
    }
}

/// Stub implementation for Azure Application Insights.
pub struct AzureAppInsights;

impl AzureAppInsights {
    /// Creates a new Azure Application Insights client (stub).
    pub async fn new(_instrumentation_key: impl Into<String>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudLogger for AzureAppInsights {
    async fn log(&self, _message: &str, _level: LogLevel) -> Result<()> {
        Err(CloudError::OperationFailed(
            "Azure Application Insights not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn log_structured(&self, _entry: &LogEntry) -> Result<()> {
        Err(CloudError::OperationFailed(
            "Azure Application Insights not implemented - SDK API breaking changes".to_string(),
        ))
    }
}

/// Stub implementation for Azure tracing.
pub struct AzureTracer;

#[async_trait]
impl CloudTracer for AzureTracer {
    async fn end_span(&self, _span: Span) -> Result<()> {
        Err(CloudError::OperationFailed(
            "Azure tracing not implemented - SDK API breaking changes".to_string(),
        ))
    }
}
