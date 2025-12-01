//! Adapters module for benchmark targets
//!
//! This module defines the canonical `BenchTarget` trait and provides
//! the `all_targets()` registry function for discovering benchmark targets.
//!
//! # BenchTarget Trait
//!
//! All benchmark targets must implement the `BenchTarget` trait:
//!
//! ```rust,ignore
//! #[async_trait]
//! pub trait BenchTarget: Send + Sync {
//!     fn id(&self) -> String;
//!     async fn run(&self) -> Result<serde_json::Value, BenchTargetError>;
//! }
//! ```
//!
//! # Registering Targets
//!
//! Add new targets to the `all_targets()` function to include them
//! in the benchmark suite.

mod targets;

use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

pub use targets::*;

/// Errors that can occur during benchmark execution
#[derive(Error, Debug)]
pub enum BenchTargetError {
    #[error("Benchmark execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Scanner error: {0}")]
    ScannerError(String),

    #[error("Timeout: benchmark exceeded time limit")]
    Timeout,
}

/// Canonical BenchTarget trait for benchmark implementations
///
/// All benchmark targets must implement this trait to be included
/// in the benchmark suite. The trait provides:
///
/// - `id()` - Returns a unique identifier for the target
/// - `run()` - Executes the benchmark and returns metrics as JSON
///
/// # Example
///
/// ```rust,ignore
/// use llm_shield_benchmarks::adapters::{BenchTarget, BenchTargetError};
/// use async_trait::async_trait;
/// use serde_json::json;
///
/// struct MyBenchmark;
///
/// #[async_trait]
/// impl BenchTarget for MyBenchmark {
///     fn id(&self) -> String {
///         "my-benchmark".to_string()
///     }
///
///     async fn run(&self) -> Result<serde_json::Value, BenchTargetError> {
///         // Run benchmark logic here
///         Ok(json!({
///             "latency_ms": 0.5,
///             "throughput_rps": 10000
///         }))
///     }
/// }
/// ```
#[async_trait]
pub trait BenchTarget: Send + Sync {
    /// Returns the unique identifier for this benchmark target
    ///
    /// The ID should be descriptive and follow the naming convention:
    /// `{category}-{subcategory}` or `{package}/{benchmark-name}`
    ///
    /// Examples:
    /// - `latency-simple-prompts`
    /// - `throughput-concurrent`
    /// - `llm-shield/secrets-detection`
    fn id(&self) -> String;

    /// Execute the benchmark and return metrics
    ///
    /// # Returns
    ///
    /// A JSON Value containing benchmark metrics. Common fields include:
    /// - `mean_ms` - Mean latency in milliseconds
    /// - `p50_ms`, `p95_ms`, `p99_ms` - Percentile latencies
    /// - `throughput_rps` - Requests per second
    /// - `memory_mb` - Memory usage in megabytes
    /// - `iterations` - Number of iterations performed
    ///
    /// # Errors
    ///
    /// Returns `BenchTargetError` if the benchmark fails
    async fn run(&self) -> Result<Value, BenchTargetError>;

    /// Optional: Get a description of this benchmark
    fn description(&self) -> Option<String> {
        None
    }

    /// Optional: Get the expected runtime category
    fn runtime_category(&self) -> RuntimeCategory {
        RuntimeCategory::Medium
    }
}

/// Runtime category for benchmarks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCategory {
    /// Quick benchmarks (<1 second)
    Quick,
    /// Medium benchmarks (1-30 seconds)
    Medium,
    /// Long benchmarks (30+ seconds)
    Long,
}

/// Registry function returning all benchmark targets
///
/// This function returns a vector of all registered benchmark targets.
/// Add new targets here to include them in the benchmark suite.
///
/// # Returns
///
/// A vector of boxed BenchTarget trait objects
///
/// # Example
///
/// ```rust,ignore
/// use llm_shield_benchmarks::adapters::all_targets;
///
/// let targets = all_targets();
/// for target in &targets {
///     println!("Target: {}", target.id());
/// }
/// ```
pub fn all_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        // Latency benchmarks
        Box::new(LatencySimpleBenchmark::new()),
        Box::new(LatencyComplexBenchmark::new()),
        Box::new(LatencySecretsDetectionBenchmark::new()),

        // Throughput benchmarks
        Box::new(ThroughputBenchmark::new()),

        // Memory benchmarks
        Box::new(MemoryBenchmark::new()),

        // Scanner-specific benchmarks
        Box::new(SecretsScannerBenchmark::new()),
        Box::new(PromptInjectionBenchmark::new()),
    ]
}

/// Get a specific target by ID
pub fn get_target(id: &str) -> Option<Box<dyn BenchTarget>> {
    all_targets().into_iter().find(|t| t.id() == id)
}

/// List all available target IDs
pub fn list_target_ids() -> Vec<String> {
    all_targets().iter().map(|t| t.id()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_targets_not_empty() {
        let targets = all_targets();
        assert!(!targets.is_empty(), "Should have registered targets");
    }

    #[test]
    fn test_unique_target_ids() {
        let targets = all_targets();
        let mut ids: Vec<String> = targets.iter().map(|t| t.id()).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "All target IDs should be unique");
    }

    #[test]
    fn test_get_target() {
        let targets = all_targets();
        if let Some(first) = targets.first() {
            let id = first.id();
            let found = get_target(&id);
            assert!(found.is_some(), "Should find target by ID");
        }
    }
}
