//! I/O utilities for benchmark results
//!
//! This module provides functions for reading and writing benchmark results
//! to the canonical output directories.

use super::BenchmarkResult;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Canonical output paths for benchmark results
#[derive(Debug, Clone)]
pub struct OutputPaths {
    /// Base output directory: benchmarks/output/
    pub base: PathBuf,
    /// Raw results directory: benchmarks/output/raw/
    pub raw: PathBuf,
    /// Summary file: benchmarks/output/summary.md
    pub summary: PathBuf,
}

impl Default for OutputPaths {
    fn default() -> Self {
        Self::new("benchmarks/output")
    }
}

impl OutputPaths {
    /// Create output paths from a base directory
    pub fn new(base: impl AsRef<Path>) -> Self {
        let base = base.as_ref().to_path_buf();
        Self {
            raw: base.join("raw"),
            summary: base.join("summary.md"),
            base,
        }
    }

    /// Get the path for a raw result file
    pub fn raw_result_path(&self, target_id: &str) -> PathBuf {
        self.raw.join(format!("{}.json", target_id.replace('/', "_")))
    }

    /// Get the path for combined results JSON
    pub fn combined_results_path(&self) -> PathBuf {
        self.base.join("results.json")
    }
}

/// Errors that can occur during I/O operations
#[derive(Error, Debug)]
pub enum IoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Output directory does not exist: {0}")]
    DirectoryNotFound(PathBuf),
}

/// Ensure output directories exist
///
/// Creates the following directory structure:
/// - benchmarks/output/
/// - benchmarks/output/raw/
///
/// # Arguments
///
/// * `paths` - OutputPaths configuration
///
/// # Returns
///
/// Result indicating success or failure
pub fn ensure_output_dirs(paths: &OutputPaths) -> Result<(), IoError> {
    fs::create_dir_all(&paths.base)?;
    fs::create_dir_all(&paths.raw)?;
    Ok(())
}

/// Write benchmark results to the output directory
///
/// Writes both:
/// - Combined results.json with all results
/// - Individual raw JSON files for each result
///
/// # Arguments
///
/// * `results` - Vector of BenchmarkResults to write
/// * `paths` - OutputPaths configuration
///
/// # Returns
///
/// Result indicating success or failure
pub fn write_results(results: &[BenchmarkResult], paths: &OutputPaths) -> Result<(), IoError> {
    ensure_output_dirs(paths)?;

    // Write combined results
    let combined_path = paths.combined_results_path();
    let file = File::create(&combined_path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, results)?;

    tracing::info!("Wrote combined results to {:?}", combined_path);

    // Write individual raw results
    write_results_raw(results, paths)?;

    Ok(())
}

/// Write individual raw result files
///
/// Each result is written to benchmarks/output/raw/{target_id}.json
///
/// # Arguments
///
/// * `results` - Vector of BenchmarkResults to write
/// * `paths` - OutputPaths configuration
pub fn write_results_raw(results: &[BenchmarkResult], paths: &OutputPaths) -> Result<(), IoError> {
    ensure_output_dirs(paths)?;

    for result in results {
        let path = paths.raw_result_path(&result.target_id);
        let file = File::create(&path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, result)?;

        tracing::debug!("Wrote raw result to {:?}", path);
    }

    Ok(())
}

/// Read benchmark results from the output directory
///
/// # Arguments
///
/// * `paths` - OutputPaths configuration
///
/// # Returns
///
/// Vector of BenchmarkResults read from combined results file
pub fn read_results(paths: &OutputPaths) -> Result<Vec<BenchmarkResult>, IoError> {
    let combined_path = paths.combined_results_path();

    if !combined_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&combined_path)?;
    let reader = BufReader::new(file);
    let results: Vec<BenchmarkResult> = serde_json::from_reader(reader)?;

    Ok(results)
}

/// Read a single raw result by target ID
///
/// # Arguments
///
/// * `target_id` - The target ID to read
/// * `paths` - OutputPaths configuration
///
/// # Returns
///
/// Option containing the BenchmarkResult if it exists
pub fn read_raw_result(target_id: &str, paths: &OutputPaths) -> Result<Option<BenchmarkResult>, IoError> {
    let path = paths.raw_result_path(target_id);

    if !path.exists() {
        return Ok(None);
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);
    let result: BenchmarkResult = serde_json::from_reader(reader)?;

    Ok(Some(result))
}

/// Write results to CSV format
///
/// # Arguments
///
/// * `results` - Vector of BenchmarkResults to write
/// * `path` - Path to the CSV file
pub fn write_results_csv(results: &[BenchmarkResult], path: impl AsRef<Path>) -> Result<(), IoError> {
    let mut file = File::create(path)?;

    // Write header
    writeln!(file, "target_id,timestamp,metrics_json")?;

    // Write rows
    for result in results {
        let metrics_json = serde_json::to_string(&result.metrics)?;
        // Escape quotes in JSON for CSV
        let escaped = metrics_json.replace('"', "\"\"");
        writeln!(
            file,
            "{},\"{}\",\"{}\"",
            result.target_id,
            result.timestamp.to_rfc3339(),
            escaped
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_output_paths() {
        let paths = OutputPaths::new("/tmp/benchmarks/output");

        assert_eq!(paths.base, PathBuf::from("/tmp/benchmarks/output"));
        assert_eq!(paths.raw, PathBuf::from("/tmp/benchmarks/output/raw"));
        assert_eq!(paths.summary, PathBuf::from("/tmp/benchmarks/output/summary.md"));
    }

    #[test]
    fn test_write_and_read_results() {
        let temp_dir = TempDir::new().unwrap();
        let paths = OutputPaths::new(temp_dir.path());

        let results = vec![
            BenchmarkResult::new("test-1", json!({"value": 1})),
            BenchmarkResult::new("test-2", json!({"value": 2})),
        ];

        write_results(&results, &paths).unwrap();

        let read_back = read_results(&paths).unwrap();
        assert_eq!(read_back.len(), 2);
        assert_eq!(read_back[0].target_id, "test-1");
    }

    #[test]
    fn test_raw_result_path() {
        let paths = OutputPaths::default();
        let path = paths.raw_result_path("llm-shield/latency");

        assert!(path.to_string_lossy().contains("llm-shield_latency.json"));
    }
}
