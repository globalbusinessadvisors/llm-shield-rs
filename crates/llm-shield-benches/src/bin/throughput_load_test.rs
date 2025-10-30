//! Rust-based load generator for throughput testing
//!
//! This tool simulates concurrent HTTP requests to benchmark throughput and latency
//! under various concurrency levels without requiring external tools like wrk.
//!
//! Features:
//! - Multiple concurrency levels (10, 50, 100, 500)
//! - Duration-based testing (configurable)
//! - Detailed latency histograms (p50, p95, p99)
//! - CSV output for analysis
//! - Both warm-up and measurement phases
//!
//! Usage:
//!   cargo build --release --bin throughput-load-test
//!   ./target/release/throughput-load-test --url http://localhost:3000/scan --duration 60 --concurrency 100

use anyhow::Result;
use chrono::Utc;
use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
struct ScanRequest {
    text: String,
}

#[derive(Debug, Deserialize)]
struct ScanResponse {
    safe: bool,
    #[allow(dead_code)]
    threats: Vec<String>,
    #[allow(dead_code)]
    latency_us: u64,
}

#[derive(Debug)]
struct LoadTestConfig {
    url: String,
    duration_secs: u64,
    concurrency: usize,
    warmup_secs: u64,
}

#[derive(Debug, Serialize)]
struct LoadTestResult {
    endpoint: String,
    duration_secs: u64,
    concurrency: usize,
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    requests_per_second: f64,
    mean_latency_ms: f64,
    p50_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    min_latency_ms: f64,
    max_latency_ms: f64,
    timestamp: String,
}

/// Generate test prompts for load testing
fn generate_test_prompts() -> Vec<String> {
    vec![
        "This is a simple test prompt for benchmarking".to_string(),
        "Check this text for any security issues".to_string(),
        "I want to buy a new laptop today".to_string(),
        "The weather is nice outside".to_string(),
        "Can you help me with my homework?".to_string(),
        "Please review this document carefully".to_string(),
        "What is the capital of France?".to_string(),
        "Tell me a joke about programming".to_string(),
        "How do I bake a chocolate cake?".to_string(),
        "Explain quantum physics simply".to_string(),
    ]
}

/// Perform a single HTTP request
async fn send_request(client: &reqwest::Client, url: &str, text: String) -> Result<Duration> {
    let start = Instant::now();

    let request = ScanRequest { text };

    let response = client
        .post(url)
        .json(&request)
        .timeout(Duration::from_secs(30))
        .send()
        .await?;

    let _body: ScanResponse = response.json().await?;

    Ok(start.elapsed())
}

/// Run load test with specified configuration
async fn run_load_test(config: LoadTestConfig) -> Result<LoadTestResult> {
    println!("\n=== Load Test Configuration ===");
    println!("Endpoint: {}", config.url);
    println!("Duration: {} seconds", config.duration_secs);
    println!("Concurrency: {}", config.concurrency);
    println!("Warmup: {} seconds", config.warmup_secs);

    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(config.concurrency)
        .pool_idle_timeout(Duration::from_secs(90))
        .build()?;

    let test_prompts = generate_test_prompts();

    // Warm-up phase
    if config.warmup_secs > 0 {
        println!("\n--- Warming up ({} seconds) ---", config.warmup_secs);
        let warmup_end = Instant::now() + Duration::from_secs(config.warmup_secs);

        while Instant::now() < warmup_end {
            for prompt in &test_prompts {
                let _ = send_request(&client, &config.url, prompt.clone()).await;
            }
        }

        println!("Warmup complete");
    }

    // Measurement phase
    println!("\n--- Running load test ({} seconds) ---", config.duration_secs);

    let mut histogram = Histogram::<u64>::new(3)?;
    let semaphore = Arc::new(Semaphore::new(config.concurrency));
    let test_end = Instant::now() + Duration::from_secs(config.duration_secs);

    let mut total_requests: u64 = 0;
    let mut successful_requests: u64 = 0;
    let mut failed_requests: u64 = 0;

    let mut tasks = Vec::new();

    let start_time = Instant::now();

    while Instant::now() < test_end {
        for prompt in &test_prompts {
            if Instant::now() >= test_end {
                break;
            }

            let permit = semaphore.clone().acquire_owned().await?;
            let client_clone = client.clone();
            let url_clone = config.url.clone();
            let prompt_clone = prompt.clone();

            let task = tokio::spawn(async move {
                let result = send_request(&client_clone, &url_clone, prompt_clone).await;
                drop(permit);
                result
            });

            tasks.push(task);
            total_requests += 1;

            // Small delay to prevent overwhelming the system
            sleep(Duration::from_micros(100)).await;
        }
    }

    println!("\n--- Waiting for in-flight requests to complete ---");

    // Wait for all tasks to complete
    for task in tasks {
        match task.await {
            Ok(Ok(duration)) => {
                successful_requests += 1;
                let _ = histogram.record(duration.as_millis() as u64);
            }
            Ok(Err(_)) => {
                failed_requests += 1;
            }
            Err(_) => {
                failed_requests += 1;
            }
        }
    }

    let actual_duration = start_time.elapsed();

    println!("\n=== Load Test Results ===");
    println!("Total requests: {}", total_requests);
    println!("Successful: {}", successful_requests);
    println!("Failed: {}", failed_requests);
    println!("Actual duration: {:.2} seconds", actual_duration.as_secs_f64());

    let rps = successful_requests as f64 / actual_duration.as_secs_f64();

    println!("\n=== Throughput ===");
    println!("Requests/second: {:.2}", rps);

    println!("\n=== Latency Distribution ===");
    println!("Mean: {:.2} ms", histogram.mean());
    println!("P50:  {:.2} ms", histogram.value_at_quantile(0.50));
    println!("P95:  {:.2} ms", histogram.value_at_quantile(0.95));
    println!("P99:  {:.2} ms", histogram.value_at_quantile(0.99));
    println!("Min:  {:.2} ms", histogram.min());
    println!("Max:  {:.2} ms", histogram.max());

    Ok(LoadTestResult {
        endpoint: config.url,
        duration_secs: config.duration_secs,
        concurrency: config.concurrency,
        total_requests,
        successful_requests,
        failed_requests,
        requests_per_second: rps,
        mean_latency_ms: histogram.mean(),
        p50_latency_ms: histogram.value_at_quantile(0.50) as f64,
        p95_latency_ms: histogram.value_at_quantile(0.95) as f64,
        p99_latency_ms: histogram.value_at_quantile(0.99) as f64,
        min_latency_ms: histogram.min() as f64,
        max_latency_ms: histogram.max() as f64,
        timestamp: Utc::now().to_rfc3339(),
    })
}

/// Run comprehensive throughput test suite
async fn run_comprehensive_test(base_url: String) -> Result<Vec<LoadTestResult>> {
    let mut results = Vec::new();

    // Test different concurrency levels on single scanner endpoint
    let concurrency_levels = vec![10, 50, 100, 500];

    println!("\n========================================");
    println!("THROUGHPUT BENCHMARK - SINGLE SCANNER");
    println!("========================================");

    for concurrency in &concurrency_levels {
        let config = LoadTestConfig {
            url: format!("{}/scan", base_url),
            duration_secs: 30, // 30 seconds per test
            concurrency: *concurrency,
            warmup_secs: 5,
        };

        let result = run_load_test(config).await?;
        results.push(result);

        // Cool-down period
        sleep(Duration::from_secs(2)).await;
    }

    // Test pipeline endpoint with moderate concurrency
    println!("\n========================================");
    println!("THROUGHPUT BENCHMARK - PIPELINE (3 SCANNERS)");
    println!("========================================");

    let config = LoadTestConfig {
        url: format!("{}/scan/pipeline", base_url),
        duration_secs: 30,
        concurrency: 100,
        warmup_secs: 5,
    };

    let result = run_load_test(config).await?;
    results.push(result);

    // Test secrets scanner
    println!("\n========================================");
    println!("THROUGHPUT BENCHMARK - SECRETS SCANNER");
    println!("========================================");

    let config = LoadTestConfig {
        url: format!("{}/scan/secrets", base_url),
        duration_secs: 30,
        concurrency: 100,
        warmup_secs: 5,
    };

    let result = run_load_test(config).await?;
    results.push(result);

    Ok(results)
}

/// Write results to CSV
async fn write_csv(results: &[LoadTestResult], output_path: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(output_path)?;

    // Write header
    writeln!(
        file,
        "endpoint,duration_secs,concurrency,total_requests,successful_requests,failed_requests,\
         requests_per_second,mean_latency_ms,p50_latency_ms,p95_latency_ms,p99_latency_ms,\
         min_latency_ms,max_latency_ms,timestamp"
    )?;

    // Write data rows
    for result in results {
        writeln!(
            file,
            "{},{},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{}",
            result.endpoint,
            result.duration_secs,
            result.concurrency,
            result.total_requests,
            result.successful_requests,
            result.failed_requests,
            result.requests_per_second,
            result.mean_latency_ms,
            result.p50_latency_ms,
            result.p95_latency_ms,
            result.p99_latency_ms,
            result.min_latency_ms,
            result.max_latency_ms,
            result.timestamp
        )?;
    }

    println!("\n✅ Results written to: {}", output_path);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let base_url = std::env::var("BENCH_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    println!("==============================================");
    println!("LLM SHIELD THROUGHPUT BENCHMARK SUITE");
    println!("==============================================");
    println!("Target: {}", base_url);

    // Check if server is up
    println!("\n--- Checking server health ---");
    let client = reqwest::Client::new();
    match client.get(format!("{}/health", base_url)).send().await {
        Ok(response) if response.status().is_success() => {
            println!("✅ Server is healthy");
        }
        Ok(response) => {
            eprintln!("❌ Server returned status: {}", response.status());
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to server: {}", e);
            eprintln!("Make sure the benchmark server is running:");
            eprintln!("  cargo run --release --bin bench-server");
            std::process::exit(1);
        }
    }

    // Run comprehensive tests
    let results = run_comprehensive_test(base_url).await?;

    // Write results
    let output_path = "benchmarks/results/throughput_results.csv";
    write_csv(&results, output_path).await?;

    // Print summary
    println!("\n==============================================");
    println!("SUMMARY");
    println!("==============================================");

    for result in &results {
        let endpoint_name = result.endpoint.split('/').last().unwrap_or("unknown");
        println!(
            "{} (c={}): {:.0} req/s, p50={:.1}ms, p99={:.1}ms",
            endpoint_name,
            result.concurrency,
            result.requests_per_second,
            result.p50_latency_ms,
            result.p99_latency_ms
        );
    }

    // Validation
    println!("\n==============================================");
    println!("CLAIM VALIDATION");
    println!("==============================================");
    println!("Target: >10,000 req/sec for simple scanners");

    let max_rps = results
        .iter()
        .filter(|r| r.endpoint.contains("/scan") && !r.endpoint.contains("pipeline"))
        .map(|r| r.requests_per_second)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    if max_rps >= 10000.0 {
        println!("✅ PASS: Achieved {:.0} req/s", max_rps);
    } else {
        println!("⚠️  WARNING: Achieved {:.0} req/s (target: 10,000)", max_rps);
    }

    Ok(())
}
