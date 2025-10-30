//! LLM Shield Performance Benchmarking Suite
//!
//! This crate provides comprehensive benchmarking infrastructure to validate
//! the performance claims made in the README:
//!
//! - Latency: <20ms (10-25x faster than Python)
//! - Throughput: >10,000 req/sec (100x higher)
//! - Memory: <500MB (8-16x lower)
//! - Cold Start: <1s (10-30x faster)
//! - Binary Size: <2MB WASM gzip (60-100x smaller)
//! - CPU: 5-10x more efficient
//!
//! ## Usage
//!
//! ```bash
//! # Run all benchmarks
//! cargo bench
//!
//! # Run specific category
//! cargo bench --bench latency
//! cargo bench --bench throughput
//! ```

pub mod fixtures;
pub mod metrics;
pub mod comparison;

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Test prompt with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPrompt {
    /// Unique identifier
    pub id: String,
    /// Prompt text
    pub text: String,
    /// Category: simple, medium, long, secrets, code, injection, toxic
    pub category: String,
    /// Expected threats to be detected
    pub expected_threats: Vec<String>,
    /// Word count
    pub word_count: usize,
}

impl TestPrompt {
    /// Create a new test prompt
    pub fn new(id: String, text: String, category: String) -> Self {
        let word_count = text.split_whitespace().count();
        Self {
            id,
            text,
            category,
            expected_threats: Vec::new(),
            word_count,
        }
    }

    /// Add expected threat
    pub fn with_threat(mut self, threat: String) -> Self {
        self.expected_threats.push(threat);
        self
    }
}

/// Benchmark result for latency tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Test name
    pub test_name: String,
    /// Language: rust or python
    pub language: String,
    /// Number of iterations
    pub iterations: usize,
    /// p50 latency in milliseconds
    pub p50_ms: f64,
    /// p95 latency in milliseconds
    pub p95_ms: f64,
    /// p99 latency in milliseconds
    pub p99_ms: f64,
    /// Mean latency in milliseconds
    pub mean_ms: f64,
    /// Minimum latency in milliseconds
    pub min_ms: f64,
    /// Maximum latency in milliseconds
    pub max_ms: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Timestamp of benchmark execution
    pub timestamp: DateTime<Utc>,
}

/// Throughput benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputResult {
    /// Test name
    pub test_name: String,
    /// Language: rust or python
    pub language: String,
    /// Test duration in seconds
    pub duration_secs: u64,
    /// Concurrency level
    pub concurrency: usize,
    /// Requests per second
    pub requests_per_sec: f64,
    /// Total requests completed
    pub total_requests: u64,
    /// Latency p50 in milliseconds
    pub latency_p50_ms: f64,
    /// Latency p95 in milliseconds
    pub latency_p95_ms: f64,
    /// Latency p99 in milliseconds
    pub latency_p99_ms: f64,
    /// Number of errors
    pub errors: u64,
    /// Number of timeouts
    pub timeouts: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Memory profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    /// Process ID
    pub process_id: u32,
    /// Duration in seconds
    pub duration_secs: u64,
    /// Baseline memory in MB
    pub baseline_mb: f64,
    /// Peak memory in MB
    pub peak_mb: f64,
    /// Final memory in MB
    pub final_mb: f64,
    /// Memory growth in MB
    pub growth_mb: f64,
    /// Memory growth percentage
    pub growth_percent: f64,
    /// Mean memory in MB
    pub mean_mb: f64,
    /// Individual samples
    pub samples: Vec<MemorySample>,
}

/// Single memory measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySample {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// RSS in KB
    pub rss_kb: u64,
    /// RSS in MB
    pub rss_mb: f64,
    /// Heap in KB
    pub heap_kb: u64,
    /// Stack in KB
    pub stack_kb: u64,
}

/// Cold start measurement result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColdStartResult {
    /// Binary path
    pub binary_path: String,
    /// Number of iterations
    pub iterations: usize,
    /// Mean startup time in ms
    pub mean_ms: f64,
    /// Median startup time in ms
    pub median_ms: f64,
    /// Minimum startup time in ms
    pub min_ms: f64,
    /// Maximum startup time in ms
    pub max_ms: f64,
    /// Standard deviation in ms
    pub std_dev_ms: f64,
    /// Target claim in ms
    pub claim_target_ms: f64,
    /// Whether claim is validated
    pub claim_validated: bool,
}

/// Binary size measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinarySizeResult {
    /// Python Docker image size in MB
    pub python_docker_mb: f64,
    /// Rust Docker image size in MB
    pub rust_docker_mb: f64,
    /// Rust native binary size in MB
    pub rust_native_mb: f64,
    /// Rust stripped binary size in MB
    pub rust_stripped_mb: f64,
    /// Rust UPX compressed size in MB
    pub rust_upx_mb: f64,
    /// WASM uncompressed size in MB
    pub wasm_uncompressed_mb: f64,
    /// WASM optimized size in MB
    pub wasm_optimized_mb: f64,
    /// WASM gzipped size in MB
    pub wasm_gzip_mb: f64,
    /// Docker improvement factor
    pub docker_improvement: f64,
    /// WASM improvement factor
    pub wasm_improvement: f64,
    /// Whether claim is validated
    pub claim_validated: bool,
}

/// CPU usage profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUProfile {
    /// Process ID
    pub process_id: u32,
    /// Duration in seconds
    pub duration_secs: u64,
    /// Mean CPU percentage
    pub mean_cpu_percent: f64,
    /// Maximum CPU percentage
    pub max_cpu_percent: f64,
    /// Minimum CPU percentage
    pub min_cpu_percent: f64,
    /// Requests per second during test
    pub requests_per_sec: f64,
    /// CPU efficiency (req/sec per 1% CPU)
    pub cpu_efficiency: f64,
    /// Individual samples
    pub samples: Vec<f64>,
}

/// Comparison result between Rust and Python
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// Test name
    pub test_name: String,
    /// Rust mean value
    pub rust_mean_ms: f64,
    /// Python mean value
    pub python_mean_ms: f64,
    /// Improvement factor (Python / Rust)
    pub improvement_factor: f64,
    /// Claimed improvement from README
    pub claimed_improvement: String,
    /// Whether claim is validated
    pub claim_validated: bool,
    /// Rust p50
    pub rust_p50: f64,
    /// Rust p95
    pub rust_p95: f64,
    /// Rust p99
    pub rust_p99: f64,
    /// Python p50
    pub python_p50: f64,
    /// Python p95
    pub python_p95: f64,
    /// Python p99
    pub python_p99: f64,
}

/// Category validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryReport {
    /// Category name (latency, throughput, etc.)
    pub name: String,
    /// Individual test results
    pub tests: Vec<ComparisonResult>,
    /// Whether all tests in category passed
    pub overall_validation: bool,
}

/// Overall validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Timestamp of report generation
    pub timestamp: DateTime<Utc>,
    /// Individual category reports
    pub categories: Vec<CategoryReport>,
    /// Whether all claims validated
    pub overall_validation: bool,
    /// Summary text
    pub summary: String,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Data directory
    pub data_dir: PathBuf,
    /// Results directory
    pub results_dir: PathBuf,
    /// Number of iterations for latency tests
    pub latency_iterations: usize,
    /// Number of iterations for ML tests (lower due to cost)
    pub ml_iterations: usize,
    /// Throughput test duration in seconds
    pub throughput_duration_secs: u64,
    /// Throughput concurrency levels to test
    pub throughput_concurrency: Vec<usize>,
    /// Memory sampling interval in seconds
    pub memory_sample_interval_secs: u64,
    /// Memory test duration in seconds
    pub memory_test_duration_secs: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("benchmarks/data"),
            results_dir: PathBuf::from("benchmarks/results"),
            latency_iterations: 1000,
            ml_iterations: 100,
            throughput_duration_secs: 60,
            throughput_concurrency: vec![10, 50, 100, 500],
            memory_sample_interval_secs: 10,
            memory_test_duration_secs: 3600, // 1 hour
        }
    }
}

/// Error types for benchmarking
#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Test data not found: {0}")]
    TestDataNotFound(String),

    #[error("Benchmark execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, BenchmarkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_prompt_creation() {
        let prompt = TestPrompt::new(
            "test_1".to_string(),
            "This is a test prompt".to_string(),
            "simple".to_string(),
        );

        assert_eq!(prompt.id, "test_1");
        assert_eq!(prompt.category, "simple");
        assert_eq!(prompt.word_count, 5);
        assert!(prompt.expected_threats.is_empty());
    }

    #[test]
    fn test_test_prompt_with_threat() {
        let prompt = TestPrompt::new(
            "test_2".to_string(),
            "Text with API key".to_string(),
            "secrets".to_string(),
        )
        .with_threat("aws_key".to_string());

        assert_eq!(prompt.expected_threats.len(), 1);
        assert_eq!(prompt.expected_threats[0], "aws_key");
    }

    #[test]
    fn test_default_config() {
        let config = BenchmarkConfig::default();

        assert_eq!(config.latency_iterations, 1000);
        assert_eq!(config.ml_iterations, 100);
        assert_eq!(config.throughput_duration_secs, 60);
        assert_eq!(config.throughput_concurrency, vec![10, 50, 100, 500]);
    }
}
