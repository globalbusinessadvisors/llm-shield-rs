//! CLI binary for running LLM Shield benchmarks
//!
//! This binary provides a command-line interface for the canonical
//! benchmark system, including the `run` subcommand.

use clap::{Parser, Subcommand};
use llm_shield_benchmarks::{
    benchmarks::{
        run_all_benchmarks, run_benchmark, list_targets,
        write_results, write_summary_markdown, OutputPaths,
    },
};
use std::path::PathBuf;
use tracing_subscriber;

#[derive(Parser)]
#[command(
    name = "llm-shield-bench",
    about = "LLM Shield Canonical Benchmark Runner",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output directory for results
    #[arg(short, long, default_value = "benchmarks/output")]
    output: PathBuf,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all registered benchmarks
    Run {
        /// Specific target to run (runs all if not specified)
        #[arg(short, long)]
        target: Option<String>,

        /// Output format (json, markdown, or both)
        #[arg(short, long, default_value = "both")]
        format: String,
    },

    /// List all available benchmark targets
    List,

    /// Show benchmark results from previous run
    Show {
        /// Target ID to show (shows all if not specified)
        target: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();

    let paths = OutputPaths::new(&cli.output);

    match cli.command {
        Commands::Run { target, format } => {
            run_benchmarks(target, &paths, &format).await?;
        }
        Commands::List => {
            list_benchmark_targets();
        }
        Commands::Show { target } => {
            show_results(target, &paths)?;
        }
    }

    Ok(())
}

async fn run_benchmarks(
    target: Option<String>,
    paths: &OutputPaths,
    format: &str,
) -> anyhow::Result<()> {
    println!("🚀 LLM Shield Benchmark Runner\n");

    let results = if let Some(target_id) = target {
        println!("Running benchmark: {}", target_id);
        match run_benchmark(&target_id).await {
            Some(result) => vec![result],
            None => {
                eprintln!("❌ Target '{}' not found", target_id);
                eprintln!("\nAvailable targets:");
                for id in list_targets() {
                    eprintln!("  - {}", id);
                }
                return Ok(());
            }
        }
    } else {
        println!("Running all benchmarks...\n");
        run_all_benchmarks().await
    };

    // Display results
    println!("\n📊 Results:\n");
    for result in &results {
        let status = if result.metrics.get("error").is_some() {
            "❌"
        } else {
            "✅"
        };
        println!("{} {} - {:?}", status, result.target_id, summarize_metrics(&result.metrics));
    }

    // Write output
    if format == "json" || format == "both" {
        write_results(&results, paths)?;
        println!("\n📁 Results written to {:?}", paths.combined_results_path());
    }

    if format == "markdown" || format == "both" {
        write_summary_markdown(&results, paths)?;
        println!("📝 Summary written to {:?}", paths.summary);
    }

    // Summary
    let total = results.len();
    let passed = results.iter().filter(|r| r.metrics.get("error").is_none()).count();
    let failed = total - passed;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Total: {}  |  ✅ Passed: {}  |  ❌ Failed: {}", total, passed, failed);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    Ok(())
}

fn list_benchmark_targets() {
    println!("📋 Available Benchmark Targets:\n");

    let targets = llm_shield_benchmarks::all_targets();

    for target in targets {
        let desc = target.description().unwrap_or_else(|| "No description".to_string());
        println!("  • {} ", target.id());
        println!("    {}\n", desc);
    }

    println!("Run a specific target with: llm-shield-bench run --target <target-id>");
}

fn show_results(target: Option<String>, paths: &OutputPaths) -> anyhow::Result<()> {
    use llm_shield_benchmarks::benchmarks::read_results;

    let results = read_results(paths)?;

    if results.is_empty() {
        println!("No results found in {:?}", paths.base);
        println!("Run benchmarks first with: llm-shield-bench run");
        return Ok(());
    }

    if let Some(target_id) = target {
        if let Some(result) = results.iter().find(|r| r.target_id == target_id) {
            println!("📊 Results for: {}\n", result.target_id);
            println!("Timestamp: {}", result.timestamp);
            println!("\nMetrics:");
            println!("{}", serde_json::to_string_pretty(&result.metrics)?);
        } else {
            println!("No results found for target: {}", target_id);
        }
    } else {
        println!("📊 All Benchmark Results:\n");
        for result in &results {
            println!("─────────────────────────────────────────");
            println!("Target: {}", result.target_id);
            println!("Time: {}", result.timestamp);
            println!("Metrics: {:?}", summarize_metrics(&result.metrics));
        }
    }

    Ok(())
}

fn summarize_metrics(metrics: &serde_json::Value) -> String {
    let mut parts = Vec::new();

    if let Some(mean) = metrics.get("mean_ms").and_then(|v| v.as_f64()) {
        parts.push(format!("mean={:.2}ms", mean));
    }
    if let Some(p95) = metrics.get("p95_ms").and_then(|v| v.as_f64()) {
        parts.push(format!("p95={:.2}ms", p95));
    }
    if let Some(rps) = metrics.get("requests_per_second").and_then(|v| v.as_f64()) {
        parts.push(format!("rps={:.0}", rps));
    }
    if let Some(error) = metrics.get("error") {
        parts.push(format!("error: {}", error));
    }

    if parts.is_empty() {
        "completed".to_string()
    } else {
        parts.join(", ")
    }
}
