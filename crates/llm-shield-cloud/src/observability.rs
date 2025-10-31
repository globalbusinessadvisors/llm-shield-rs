//! Cloud observability abstractions.
//!
//! Provides unified traits for metrics, logging, and tracing across cloud providers:
//! - AWS: CloudWatch Metrics, CloudWatch Logs, X-Ray
//! - GCP: Cloud Monitoring, Cloud Logging, Cloud Trace
//! - Azure: Azure Monitor, Application Insights

use crate::error::{CloudError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::SystemTime;

/// Log severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Trace-level logging (most verbose).
    Trace,
    /// Debug-level logging.
    Debug,
    /// Info-level logging.
    Info,
    /// Warning-level logging.
    Warn,
    /// Error-level logging.
    Error,
    /// Fatal/critical-level logging.
    Fatal,
}

impl LogLevel {
    /// Converts log level to a string.
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
        }
    }

    /// Converts log level to numeric severity (higher = more severe).
    pub fn to_severity(&self) -> u8 {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
            LogLevel::Fatal => 5,
        }
    }
}

/// Structured log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp of the log entry.
    pub timestamp: SystemTime,

    /// Log severity level.
    pub level: LogLevel,

    /// Log message.
    pub message: String,

    /// Optional labels/tags for the log entry.
    pub labels: HashMap<String, String>,

    /// Optional trace context (for correlation).
    pub trace_id: Option<String>,

    /// Optional span context (for correlation).
    pub span_id: Option<String>,

    /// Source location (file, line).
    pub source: Option<String>,
}

impl LogEntry {
    /// Creates a new log entry with the given level and message.
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: SystemTime::now(),
            level,
            message: message.into(),
            labels: HashMap::new(),
            trace_id: None,
            span_id: None,
            source: None,
        }
    }

    /// Adds a label to the log entry.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Sets the trace ID for correlation.
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    /// Sets the span ID for correlation.
    pub fn with_span_id(mut self, span_id: impl Into<String>) -> Self {
        self.span_id = Some(span_id.into());
        self
    }
}

/// Metric data point.
#[derive(Debug, Clone)]
pub struct Metric {
    /// Metric name.
    pub name: String,

    /// Metric value.
    pub value: f64,

    /// Timestamp of the metric.
    pub timestamp: u64,

    /// Metric dimensions/labels.
    pub dimensions: HashMap<String, String>,

    /// Metric unit (e.g., "Count", "Seconds", "Bytes").
    pub unit: Option<String>,
}

impl Metric {
    /// Creates a new metric with the given name and value.
    pub fn new(name: impl Into<String>, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
            timestamp: SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            dimensions: HashMap::new(),
            unit: None,
        }
    }

    /// Adds a dimension to the metric.
    pub fn with_dimension(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.dimensions.insert(key.into(), value.into());
        self
    }

    /// Sets the metric unit.
    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }
}

/// Trace span representing a unit of work.
#[derive(Debug, Clone)]
pub struct Span {
    /// Span name.
    pub name: String,

    /// Span ID (unique within a trace).
    pub span_id: String,

    /// Trace ID (groups related spans).
    pub trace_id: String,

    /// Parent span ID (if this is a child span).
    pub parent_span_id: Option<String>,

    /// When the span started.
    pub start_time: SystemTime,

    /// When the span ended (None if still active).
    pub end_time: Option<SystemTime>,

    /// Span attributes/tags.
    pub attributes: HashMap<String, String>,

    /// Span status (e.g., "OK", "ERROR").
    pub status: Option<String>,
}

impl Span {
    /// Creates a new span with the given name.
    pub fn new(name: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            span_id: uuid::Uuid::new_v4().to_string(),
            trace_id: trace_id.into(),
            parent_span_id: None,
            start_time: SystemTime::now(),
            end_time: None,
            attributes: HashMap::new(),
            status: None,
        }
    }

    /// Sets the parent span ID.
    pub fn with_parent(mut self, parent_span_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_span_id.into());
        self
    }

    /// Adds an attribute to the span.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Ends the span with the given status.
    pub fn end_with_status(mut self, status: impl Into<String>) -> Self {
        self.end_time = Some(SystemTime::now());
        self.status = Some(status.into());
        self
    }

    /// Gets the span duration if ended.
    pub fn duration(&self) -> Option<std::time::Duration> {
        self.end_time
            .and_then(|end| end.duration_since(self.start_time).ok())
    }
}

/// Unified trait for cloud metrics export.
#[async_trait]
pub trait CloudMetrics: Send + Sync {
    /// Exports a batch of metrics to the cloud provider.
    ///
    /// # Arguments
    ///
    /// * `metrics` - Slice of metrics to export
    ///
    /// # Errors
    ///
    /// Returns `CloudError::MetricsExport` if the export operation fails.
    async fn export_metrics(&self, metrics: &[Metric]) -> Result<()>;

    /// Exports a single metric.
    ///
    /// # Arguments
    ///
    /// * `metric` - The metric to export
    ///
    /// # Errors
    ///
    /// Returns `CloudError::MetricsExport` if the export operation fails.
    async fn export_metric(&self, metric: &Metric) -> Result<()> {
        self.export_metrics(&[metric.clone()]).await
    }
}

/// Unified trait for cloud logging.
#[async_trait]
pub trait CloudLogger: Send + Sync {
    /// Writes a simple log message.
    ///
    /// # Arguments
    ///
    /// * `message` - The log message
    /// * `level` - The log level
    ///
    /// # Errors
    ///
    /// Returns `CloudError::LogWrite` if the write operation fails.
    async fn log(&self, message: &str, level: LogLevel) -> Result<()>;

    /// Writes a structured log entry.
    ///
    /// # Arguments
    ///
    /// * `entry` - The structured log entry
    ///
    /// # Errors
    ///
    /// Returns `CloudError::LogWrite` if the write operation fails.
    async fn log_structured(&self, entry: &LogEntry) -> Result<()>;

    /// Writes a batch of log entries.
    ///
    /// # Arguments
    ///
    /// * `entries` - Slice of log entries to write
    ///
    /// # Errors
    ///
    /// Returns `CloudError::LogWrite` if the write operation fails.
    async fn log_batch(&self, entries: &[LogEntry]) -> Result<()> {
        // Default implementation writes one by one
        for entry in entries {
            self.log_structured(entry).await?;
        }
        Ok(())
    }
}

/// Unified trait for distributed tracing.
#[async_trait]
pub trait CloudTracer: Send + Sync {
    /// Starts a new trace span.
    ///
    /// # Arguments
    ///
    /// * `name` - The span name
    ///
    /// # Returns
    ///
    /// Returns a new span with a generated trace ID.
    fn start_span(&self, name: &str) -> Span {
        Span::new(name, uuid::Uuid::new_v4().to_string())
    }

    /// Starts a child span.
    ///
    /// # Arguments
    ///
    /// * `name` - The span name
    /// * `parent` - The parent span
    ///
    /// # Returns
    ///
    /// Returns a new child span.
    fn start_child_span(&self, name: &str, parent: &Span) -> Span {
        Span::new(name, parent.trace_id.clone())
            .with_parent(parent.span_id.clone())
    }

    /// Ends a span and exports it.
    ///
    /// # Arguments
    ///
    /// * `span` - The span to end and export
    ///
    /// # Errors
    ///
    /// Returns `CloudError::TraceExport` if the export operation fails.
    async fn end_span(&self, span: Span) -> Result<()>;

    /// Exports a batch of spans.
    ///
    /// # Arguments
    ///
    /// * `spans` - Slice of spans to export
    ///
    /// # Errors
    ///
    /// Returns `CloudError::TraceExport` if the export operation fails.
    async fn export_spans(&self, spans: &[Span]) -> Result<()> {
        // Default implementation exports one by one
        for span in spans {
            self.end_span(span.clone()).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_as_str() {
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }

    #[test]
    fn test_log_level_severity() {
        assert!(LogLevel::Fatal.to_severity() > LogLevel::Error.to_severity());
        assert!(LogLevel::Error.to_severity() > LogLevel::Warn.to_severity());
        assert!(LogLevel::Warn.to_severity() > LogLevel::Info.to_severity());
    }

    #[test]
    fn test_log_entry_builder() {
        let entry = LogEntry::new(LogLevel::Info, "Test message")
            .with_label("service", "llm-shield")
            .with_trace_id("trace-123")
            .with_span_id("span-456");

        assert_eq!(entry.message, "Test message");
        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.labels.get("service"), Some(&"llm-shield".to_string()));
        assert_eq!(entry.trace_id, Some("trace-123".to_string()));
        assert_eq!(entry.span_id, Some("span-456".to_string()));
    }

    #[test]
    fn test_metric_builder() {
        let metric = Metric::new("http_requests_total", 100.0)
            .with_dimension("method", "POST")
            .with_dimension("status", "200")
            .with_unit("Count");

        assert_eq!(metric.name, "http_requests_total");
        assert_eq!(metric.value, 100.0);
        assert_eq!(metric.dimensions.get("method"), Some(&"POST".to_string()));
        assert_eq!(metric.unit, Some("Count".to_string()));
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new("test_operation", "trace-abc")
            .with_attribute("http.method", "GET")
            .with_attribute("http.status_code", "200");

        assert_eq!(span.name, "test_operation");
        assert_eq!(span.trace_id, "trace-abc");
        assert!(span.parent_span_id.is_none());
        assert!(span.end_time.is_none());
        assert_eq!(span.attributes.len(), 2);
    }

    #[test]
    fn test_span_child() {
        let parent = Span::new("parent", "trace-123");
        let child = Span::new("child", parent.trace_id.clone())
            .with_parent(parent.span_id.clone());

        assert_eq!(child.trace_id, parent.trace_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_span_duration() {
        let span = Span::new("test", "trace-1");
        assert!(span.duration().is_none());

        let ended_span = span.end_with_status("OK");
        assert!(ended_span.duration().is_some());
        assert_eq!(ended_span.status, Some("OK".to_string()));
    }
}
