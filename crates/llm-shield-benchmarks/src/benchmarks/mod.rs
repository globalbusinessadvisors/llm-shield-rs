//! Canonical benchmarks module for LLM Shield
//!
//! This module provides the standardized benchmark interface compatible with
//! all 25 benchmark-target repositories. It includes:
//!
//! - `BenchmarkResult` - Canonical result struct with target_id, metrics, timestamp
//! - `run_all_benchmarks()` - Main entrypoint returning Vec<BenchmarkResult>
//! - I/O utilities for reading/writing benchmark results
//! - Markdown report generation
//!
//! # Example
//!
//! ```no_run
//! use llm_shield_benchmarks::benchmarks::run_all_benchmarks;
//!
//! #[tokio::main]
//! async fn main() {
//!     let results = run_all_benchmarks().await;
//!     println!("Completed {} benchmarks", results.len());
//! }
//! ```

mod result;
mod io;
mod markdown;

pub use result::BenchmarkResult;
pub use io::{
    write_results,
    write_results_raw,
    read_results,
    ensure_output_dirs,
    OutputPaths,
};
pub use markdown::{
    generate_summary_markdown,
    write_summary_markdown,
};

use crate::adapters::{all_targets, BenchTarget};
use chrono::Utc;
use serde_json::json;
use std::time::Instant;

/// Run all registered benchmarks and return results
///
/// This is the canonical entrypoint for the benchmark system. It:
/// 1. Retrieves all registered benchmark targets
/// 2. Executes each target's `run()` method
/// 3. Collects results into Vec<BenchmarkResult>
///
/// # Returns
///
/// A vector of BenchmarkResult containing results from all benchmark targets
///
/// # Example
///
/// ```no_run
/// use llm_shield_benchmarks::benchmarks::run_all_benchmarks;
///
/// #[tokio::main]
/// async fn main() {
///     let results = run_all_benchmarks().await;
///     for result in &results {
///         println!("{}: {:?}", result.target_id, result.metrics);
///     }
/// }
/// ```
pub async fn run_all_benchmarks() -> Vec<BenchmarkResult> {
    let targets = all_targets();
    let mut results = Vec::with_capacity(targets.len());

    tracing::info!("Running {} benchmark targets", targets.len());

    for target in targets {
        let target_id = target.id();
        tracing::info!("Running benchmark: {}", target_id);

        let start = Instant::now();
        match target.run().await {
            Ok(metrics) => {
                let elapsed = start.elapsed();
                let mut metrics_with_meta = metrics;

                // Add execution metadata
                if let Some(obj) = metrics_with_meta.as_object_mut() {
                    obj.insert("execution_time_ms".to_string(),
                        json!(elapsed.as_secs_f64() * 1000.0));
                }

                results.push(BenchmarkResult::new(target_id.clone(), metrics_with_meta));
                tracing::info!("Completed benchmark {} in {:?}", target_id, elapsed);
            }
            Err(e) => {
                tracing::error!("Benchmark {} failed: {}", target_id, e);
                results.push(BenchmarkResult::new(
                    target_id.clone(),
                    json!({
                        "error": e.to_string(),
                        "status": "failed"
                    }),
                ));
            }
        }
    }

    tracing::info!("Completed all {} benchmarks", results.len());
    results
}

/// Run a specific benchmark target by ID
///
/// # Arguments
///
/// * `target_id` - The ID of the target to run
///
/// # Returns
///
/// Option containing the BenchmarkResult if target exists
pub async fn run_benchmark(target_id: &str) -> Option<BenchmarkResult> {
    let targets = all_targets();

    for target in targets {
        if target.id() == target_id {
            let start = Instant::now();
            match target.run().await {
                Ok(metrics) => {
                    let elapsed = start.elapsed();
                    let mut metrics_with_meta = metrics;

                    if let Some(obj) = metrics_with_meta.as_object_mut() {
                        obj.insert("execution_time_ms".to_string(),
                            json!(elapsed.as_secs_f64() * 1000.0));
                    }

                    return Some(BenchmarkResult::new(target_id, metrics_with_meta));
                }
                Err(e) => {
                    return Some(BenchmarkResult::new(
                        target_id,
                        json!({
                            "error": e.to_string(),
                            "status": "failed"
                        }),
                    ));
                }
            }
        }
    }

    None
}

/// List all available benchmark target IDs
pub fn list_targets() -> Vec<String> {
    all_targets().iter().map(|t| t.id()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_all_benchmarks() {
        let results = run_all_benchmarks().await;
        // Should return results for all registered targets
        assert!(!results.is_empty());
    }

    #[test]
    fn test_list_targets() {
        let targets = list_targets();
        // Should have at least some registered targets
        assert!(!targets.is_empty());
    }
}
