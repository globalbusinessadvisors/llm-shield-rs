//! Statistical metrics calculation for benchmark results
//!
//! Provides functions to compute p50, p95, p99, mean, std dev, etc.

use crate::BenchmarkResult;
use chrono::Utc;

/// Computed statistical metrics
#[derive(Debug, Clone)]
pub struct Metrics {
    pub mean: f64,
    pub median: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub count: usize,
}

/// Compute statistical metrics from a series of measurements
///
/// # Arguments
///
/// * `measurements` - Vector of latency measurements in milliseconds
///
/// # Returns
///
/// Metrics struct with all statistical summaries
///
/// # Example
///
/// ```
/// use llm_shield_benches::metrics::compute_metrics;
///
/// let latencies = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let metrics = compute_metrics(&latencies);
///
/// assert_eq!(metrics.median, 3.0);
/// assert_eq!(metrics.min, 1.0);
/// assert_eq!(metrics.max, 5.0);
/// ```
pub fn compute_metrics(measurements: &[f64]) -> Metrics {
    if measurements.is_empty() {
        return Metrics {
            mean: 0.0,
            median: 0.0,
            p50: 0.0,
            p95: 0.0,
            p99: 0.0,
            min: 0.0,
            max: 0.0,
            std_dev: 0.0,
            count: 0,
        };
    }

    // Sort for percentile calculation
    let mut sorted = measurements.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = sorted.len();

    // Calculate mean
    let sum: f64 = sorted.iter().sum();
    let mean = sum / n as f64;

    // Calculate percentiles
    let p50_index = (n as f64 * 0.50).floor() as usize;
    let p95_index = (n as f64 * 0.95).floor() as usize;
    let p99_index = (n as f64 * 0.99).floor() as usize;

    let p50 = sorted[p50_index.min(n - 1)];
    let p95 = sorted[p95_index.min(n - 1)];
    let p99 = sorted[p99_index.min(n - 1)];

    // Calculate standard deviation
    let variance_sum: f64 = sorted.iter().map(|&x| (x - mean).powi(2)).sum();
    let variance = variance_sum / n as f64;
    let std_dev = variance.sqrt();

    // Get min/max
    let min = sorted[0];
    let max = sorted[n - 1];

    Metrics {
        mean,
        median: p50,
        p50,
        p95,
        p99,
        min,
        max,
        std_dev,
        count: n,
    }
}

/// Convert metrics to BenchmarkResult
///
/// # Arguments
///
/// * `test_name` - Name of the test
/// * `language` - "rust" or "python"
/// * `metrics` - Computed metrics
pub fn metrics_to_benchmark_result(
    test_name: &str,
    language: &str,
    metrics: &Metrics,
) -> BenchmarkResult {
    BenchmarkResult {
        test_name: test_name.to_string(),
        language: language.to_string(),
        iterations: metrics.count,
        p50_ms: metrics.p50,
        p95_ms: metrics.p95,
        p99_ms: metrics.p99,
        mean_ms: metrics.mean,
        min_ms: metrics.min,
        max_ms: metrics.max,
        std_dev: metrics.std_dev,
        timestamp: Utc::now(),
    }
}

/// Save benchmark result to CSV
///
/// # Arguments
///
/// * `result` - The benchmark result to save
/// * `output_path` - Path to the CSV file
pub fn save_benchmark_result(
    result: &BenchmarkResult,
    output_path: &std::path::Path,
) -> crate::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    // Create header if file doesn't exist
    let file_exists = output_path.exists();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(output_path)?;

    if !file_exists {
        writeln!(
            file,
            "test_name,language,iterations,p50_ms,p95_ms,p99_ms,mean_ms,min_ms,max_ms,std_dev,timestamp"
        )?;
    }

    writeln!(
        file,
        "{},{},{},{},{},{},{},{},{},{},{}",
        result.test_name,
        result.language,
        result.iterations,
        result.p50_ms,
        result.p95_ms,
        result.p99_ms,
        result.mean_ms,
        result.min_ms,
        result.max_ms,
        result.std_dev,
        result.timestamp.to_rfc3339()
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_compute_metrics_simple() {
        let measurements = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let metrics = compute_metrics(&measurements);

        assert_eq!(metrics.count, 5);
        assert_relative_eq!(metrics.mean, 3.0);
        assert_eq!(metrics.median, 3.0);
        assert_eq!(metrics.min, 1.0);
        assert_eq!(metrics.max, 5.0);
    }

    #[test]
    fn test_compute_metrics_percentiles() {
        let mut measurements = Vec::new();
        for i in 1..=100 {
            measurements.push(i as f64);
        }

        let metrics = compute_metrics(&measurements);

        assert_eq!(metrics.count, 100);
        assert_eq!(metrics.p50, 50.0);
        assert_eq!(metrics.p95, 95.0);
        assert_eq!(metrics.p99, 99.0);
    }

    #[test]
    fn test_compute_metrics_empty() {
        let measurements = Vec::new();
        let metrics = compute_metrics(&measurements);

        assert_eq!(metrics.count, 0);
        assert_eq!(metrics.mean, 0.0);
    }

    #[test]
    fn test_compute_metrics_single_value() {
        let measurements = vec![42.0];
        let metrics = compute_metrics(&measurements);

        assert_eq!(metrics.count, 1);
        assert_eq!(metrics.mean, 42.0);
        assert_eq!(metrics.median, 42.0);
        assert_eq!(metrics.min, 42.0);
        assert_eq!(metrics.max, 42.0);
        assert_eq!(metrics.std_dev, 0.0);
    }

    #[test]
    fn test_compute_metrics_unsorted_input() {
        let measurements = vec![5.0, 1.0, 3.0, 2.0, 4.0];
        let metrics = compute_metrics(&measurements);

        // Should handle unsorted input correctly
        assert_eq!(metrics.min, 1.0);
        assert_eq!(metrics.max, 5.0);
        assert_eq!(metrics.median, 3.0);
    }

    #[test]
    fn test_metrics_to_benchmark_result() {
        let measurements = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let metrics = compute_metrics(&measurements);

        let result = metrics_to_benchmark_result("test_scanner", "rust", &metrics);

        assert_eq!(result.test_name, "test_scanner");
        assert_eq!(result.language, "rust");
        assert_eq!(result.iterations, 5);
        assert_relative_eq!(result.mean_ms, 3.0);
    }
}
