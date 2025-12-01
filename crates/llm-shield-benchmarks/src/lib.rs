//! LLM Shield Canonical Benchmark Interface
//!
//! This crate provides the standardized benchmark interface compatible with
//! all 25 benchmark-target repositories. It implements the canonical structure:
//!
//! - `BenchmarkResult` struct with `target_id`, `metrics`, `timestamp` fields
//! - `run_all_benchmarks()` entrypoint returning `Vec<BenchmarkResult>`
//! - `BenchTarget` trait with `id()` and `run()` methods
//! - `all_targets()` registry function
//! - I/O utilities for benchmark output directories
//! - Markdown summary generation
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use llm_shield_benchmarks::benchmarks::{run_all_benchmarks, write_results, OutputPaths};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Run all benchmarks
//!     let results = run_all_benchmarks().await;
//!
//!     // Write results to canonical output directories
//!     let paths = OutputPaths::default();
//!     write_results(&results, &paths).unwrap();
//! }
//! ```
//!
//! # Module Structure
//!
//! - `benchmarks` - Core benchmark infrastructure
//!   - `BenchmarkResult` - Canonical result struct
//!   - `run_all_benchmarks()` - Main entrypoint
//!   - `io` - File I/O utilities
//!   - `markdown` - Report generation
//!
//! - `adapters` - Benchmark target definitions
//!   - `BenchTarget` trait - Interface for benchmark implementations
//!   - `all_targets()` - Registry of all benchmark targets
//!   - Concrete target implementations
//!
//! # Canonical Output Structure
//!
//! ```text
//! benchmarks/output/
//! ├── results.json      # Combined results file
//! ├── summary.md        # Markdown summary report
//! └── raw/              # Individual result files
//!     ├── target-1.json
//!     ├── target-2.json
//!     └── ...
//! ```
//!
//! # Compatibility
//!
//! This crate is designed to be compatible with the benchmark interface
//! used across all 25 benchmark-target repositories, ensuring consistent
//! benchmark data interchange and reporting.

pub mod adapters;
pub mod benchmarks;

// Re-export commonly used items at crate root
pub use adapters::{all_targets, BenchTarget, BenchTargetError};
pub use benchmarks::{
    run_all_benchmarks,
    BenchmarkResult,
    OutputPaths,
    write_results,
    write_summary_markdown,
    generate_summary_markdown,
};
