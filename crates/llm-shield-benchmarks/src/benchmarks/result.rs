//! Canonical BenchmarkResult struct for cross-repository compatibility
//!
//! This module defines the standardized BenchmarkResult used across all 25
//! benchmark-target repositories for consistent benchmark data interchange.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Canonical BenchmarkResult struct with standardized fields.
///
/// This struct is designed to be compatible across all benchmark-target
/// repositories, providing a consistent interface for benchmark results.
///
/// # Fields
///
/// * `target_id` - Unique identifier for the benchmark target
/// * `metrics` - Flexible JSON value containing benchmark metrics
/// * `timestamp` - UTC timestamp when the benchmark was executed
///
/// # Example
///
/// ```
/// use llm_shield_benchmarks::benchmarks::BenchmarkResult;
/// use chrono::Utc;
/// use serde_json::json;
///
/// let result = BenchmarkResult {
///     target_id: "llm-shield-latency-simple".to_string(),
///     metrics: json!({
///         "mean_ms": 0.03,
///         "p50_ms": 0.02,
///         "p95_ms": 0.05,
///         "p99_ms": 0.08,
///         "iterations": 1000
///     }),
///     timestamp: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkResult {
    /// Unique identifier for the benchmark target
    pub target_id: String,

    /// Flexible metrics container as JSON value
    /// Can contain any benchmark-specific measurements
    pub metrics: serde_json::Value,

    /// UTC timestamp of benchmark execution
    pub timestamp: DateTime<Utc>,
}

impl BenchmarkResult {
    /// Create a new BenchmarkResult with current timestamp
    ///
    /// # Arguments
    ///
    /// * `target_id` - Unique identifier for the benchmark target
    /// * `metrics` - JSON value containing benchmark metrics
    ///
    /// # Returns
    ///
    /// A new BenchmarkResult with the current UTC timestamp
    pub fn new(target_id: impl Into<String>, metrics: serde_json::Value) -> Self {
        Self {
            target_id: target_id.into(),
            metrics,
            timestamp: Utc::now(),
        }
    }

    /// Create a new BenchmarkResult with a specific timestamp
    ///
    /// # Arguments
    ///
    /// * `target_id` - Unique identifier for the benchmark target
    /// * `metrics` - JSON value containing benchmark metrics
    /// * `timestamp` - Specific UTC timestamp for the result
    pub fn with_timestamp(
        target_id: impl Into<String>,
        metrics: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            target_id: target_id.into(),
            metrics,
            timestamp,
        }
    }

    /// Get a metric value by key
    ///
    /// # Arguments
    ///
    /// * `key` - The metric key to retrieve
    ///
    /// # Returns
    ///
    /// Option containing the metric value if it exists
    pub fn get_metric(&self, key: &str) -> Option<&serde_json::Value> {
        self.metrics.get(key)
    }

    /// Get a metric value as f64
    ///
    /// # Arguments
    ///
    /// * `key` - The metric key to retrieve
    ///
    /// # Returns
    ///
    /// Option containing the metric as f64 if it exists and is numeric
    pub fn get_metric_f64(&self, key: &str) -> Option<f64> {
        self.metrics.get(key).and_then(|v| v.as_f64())
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for BenchmarkResult {
    fn default() -> Self {
        Self {
            target_id: String::new(),
            metrics: serde_json::Value::Object(serde_json::Map::new()),
            timestamp: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_benchmark_result_new() {
        let metrics = json!({
            "latency_ms": 0.5,
            "throughput_rps": 10000
        });
        let result = BenchmarkResult::new("test-target", metrics);

        assert_eq!(result.target_id, "test-target");
        assert_eq!(result.get_metric_f64("latency_ms"), Some(0.5));
        assert_eq!(result.get_metric_f64("throughput_rps"), Some(10000.0));
    }

    #[test]
    fn test_benchmark_result_serialization() {
        let metrics = json!({"value": 42});
        let result = BenchmarkResult::new("serialize-test", metrics);

        let json = result.to_json().unwrap();
        let deserialized = BenchmarkResult::from_json(&json).unwrap();

        assert_eq!(result.target_id, deserialized.target_id);
        assert_eq!(result.metrics, deserialized.metrics);
    }

    #[test]
    fn test_benchmark_result_with_timestamp() {
        let timestamp = Utc::now();
        let result = BenchmarkResult::with_timestamp(
            "timestamp-test",
            json!({}),
            timestamp,
        );

        assert_eq!(result.timestamp, timestamp);
    }
}
