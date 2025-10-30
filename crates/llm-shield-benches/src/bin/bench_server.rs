//! High-performance HTTP benchmark server for throughput testing
//!
//! This server provides realistic endpoints for load testing with wrk or similar tools.
//! It simulates production-grade concurrent request handling using tokio and axum.
//!
//! Endpoints:
//! - POST /scan - Single scanner endpoint (BanSubstrings)
//! - POST /scan/pipeline - 3-scanner pipeline endpoint
//! - POST /scan/secrets - Secrets scanner endpoint
//! - GET /health - Health check
//! - GET /metrics - Performance metrics
//!
//! Usage:
//!   cargo build --release --bin bench-server
//!   ./target/release/bench-server
//!
//! Load test with wrk:
//!   wrk -t4 -c100 -d60s --latency -s wrk_script.lua http://localhost:3000/scan

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use hdrhistogram::Histogram;
use llm_shield_core::SecretVault;
use llm_shield_scanners::input::{
    BanSubstrings, BanSubstringsConfig, Secrets, SecretsConfig, Toxicity, ToxicityConfig,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Scan request payload
#[derive(Debug, Deserialize)]
struct ScanRequest {
    text: String,
}

/// Scan response
#[derive(Debug, Serialize)]
struct ScanResponse {
    safe: bool,
    threats: Vec<String>,
    latency_us: u64,
}

/// Metrics response
#[derive(Debug, Serialize)]
struct MetricsResponse {
    total_requests: u64,
    total_errors: u64,
    mean_latency_us: f64,
    p50_latency_us: u64,
    p95_latency_us: u64,
    p99_latency_us: u64,
    requests_per_second: f64,
}

/// Shared application state
struct AppState {
    // Scanners (pre-initialized for fast access)
    ban_scanner: BanSubstrings,
    secrets_scanner: Secrets,
    toxicity_scanner: Toxicity,

    // Secret vault
    vault: SecretVault,

    // Metrics
    metrics: RwLock<ServerMetrics>,
}

/// Server-side metrics tracking
struct ServerMetrics {
    total_requests: u64,
    total_errors: u64,
    latency_histogram: Histogram<u64>,
    start_time: Instant,
}

impl ServerMetrics {
    fn new() -> Self {
        Self {
            total_requests: 0,
            total_errors: 0,
            latency_histogram: Histogram::new(3).unwrap(), // 3 significant figures
            start_time: Instant::now(),
        }
    }

    fn record_request(&mut self, latency_us: u64) {
        self.total_requests += 1;
        let _ = self.latency_histogram.record(latency_us);
    }

    fn record_error(&mut self) {
        self.total_errors += 1;
    }

    fn get_summary(&self) -> MetricsResponse {
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        let rps = if elapsed_secs > 0.0 {
            self.total_requests as f64 / elapsed_secs
        } else {
            0.0
        };

        MetricsResponse {
            total_requests: self.total_requests,
            total_errors: self.total_errors,
            mean_latency_us: self.latency_histogram.mean(),
            p50_latency_us: self.latency_histogram.value_at_quantile(0.50),
            p95_latency_us: self.latency_histogram.value_at_quantile(0.95),
            p99_latency_us: self.latency_histogram.value_at_quantile(0.99),
            requests_per_second: rps,
        }
    }
}

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

/// Single scanner endpoint (BanSubstrings - fastest)
async fn scan_single(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ScanRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    match state.ban_scanner.scan(&payload.text, &state.vault).await {
        Ok(result) => {
            let latency_us = start.elapsed().as_micros() as u64;

            // Record metrics
            {
                let mut metrics = state.metrics.write().await;
                metrics.record_request(latency_us);
            }

            let response = ScanResponse {
                safe: !result.detected,
                threats: result.detected_scanners,
                latency_us,
            };

            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            {
                let mut metrics = state.metrics.write().await;
                metrics.record_error();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ScanResponse {
                    safe: false,
                    threats: vec![format!("Error: {}", e)],
                    latency_us: start.elapsed().as_micros() as u64,
                }),
            )
        }
    }
}

/// Pipeline endpoint (3 scanners in sequence)
async fn scan_pipeline(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ScanRequest>,
) -> impl IntoResponse {
    let start = Instant::now();
    let mut all_threats = Vec::new();
    let mut detected = false;

    // Scanner 1: BanSubstrings
    match state.ban_scanner.scan(&payload.text, &state.vault).await {
        Ok(result) => {
            if result.detected {
                detected = true;
                all_threats.extend(result.detected_scanners);
            }
        }
        Err(_) => {
            let mut metrics = state.metrics.write().await;
            metrics.record_error();
        }
    }

    // Scanner 2: Secrets
    match state.secrets_scanner.scan(&payload.text, &state.vault).await {
        Ok(result) => {
            if result.detected {
                detected = true;
                all_threats.extend(result.detected_scanners);
            }
        }
        Err(_) => {
            let mut metrics = state.metrics.write().await;
            metrics.record_error();
        }
    }

    // Scanner 3: Toxicity
    match state.toxicity_scanner.scan(&payload.text, &state.vault).await {
        Ok(result) => {
            if result.detected {
                detected = true;
                all_threats.extend(result.detected_scanners);
            }
        }
        Err(_) => {
            let mut metrics = state.metrics.write().await;
            metrics.record_error();
        }
    }

    let latency_us = start.elapsed().as_micros() as u64;

    // Record metrics
    {
        let mut metrics = state.metrics.write().await;
        metrics.record_request(latency_us);
    }

    let response = ScanResponse {
        safe: !detected,
        threats: all_threats,
        latency_us,
    };

    (StatusCode::OK, Json(response))
}

/// Secrets scanner endpoint
async fn scan_secrets(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ScanRequest>,
) -> impl IntoResponse {
    let start = Instant::now();

    match state.secrets_scanner.scan(&payload.text, &state.vault).await {
        Ok(result) => {
            let latency_us = start.elapsed().as_micros() as u64;

            // Record metrics
            {
                let mut metrics = state.metrics.write().await;
                metrics.record_request(latency_us);
            }

            let response = ScanResponse {
                safe: !result.detected,
                threats: result.detected_scanners,
                latency_us,
            };

            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            {
                let mut metrics = state.metrics.write().await;
                metrics.record_error();
            }

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ScanResponse {
                    safe: false,
                    threats: vec![format!("Error: {}", e)],
                    latency_us: start.elapsed().as_micros() as u64,
                }),
            )
        }
    }
}

/// Metrics endpoint
async fn metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let metrics = state.metrics.read().await;
    let summary = metrics.get_summary();

    (StatusCode::OK, Json(summary))
}

/// Reset metrics (useful for multiple test runs)
async fn reset_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut metrics = state.metrics.write().await;
    *metrics = ServerMetrics::new();

    (StatusCode::OK, "Metrics reset")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Initializing LLM Shield benchmark server...");

    // Initialize scanners
    let ban_scanner = BanSubstrings::new(BanSubstringsConfig {
        substrings: vec![
            "banned".to_string(),
            "forbidden".to_string(),
            "prohibited".to_string(),
        ],
        ..Default::default()
    })?;

    let secrets_scanner = Secrets::new(SecretsConfig::default())?;
    let toxicity_scanner = Toxicity::new(ToxicityConfig::default())?;
    let vault = SecretVault::new();

    info!("Scanners initialized successfully");

    // Create shared state
    let state = Arc::new(AppState {
        ban_scanner,
        secrets_scanner,
        toxicity_scanner,
        vault,
        metrics: RwLock::new(ServerMetrics::new()),
    });

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/scan", post(scan_single))
        .route("/scan/pipeline", post(scan_pipeline))
        .route("/scan/secrets", post(scan_secrets))
        .route("/metrics", get(metrics))
        .route("/metrics/reset", post(reset_metrics))
        .with_state(state);

    // Bind and serve
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on http://{}", addr);
    info!("Endpoints:");
    info!("  POST /scan - Single scanner (BanSubstrings)");
    info!("  POST /scan/pipeline - 3-scanner pipeline");
    info!("  POST /scan/secrets - Secrets scanner");
    info!("  GET  /metrics - Performance metrics");
    info!("  POST /metrics/reset - Reset metrics");
    info!("  GET  /health - Health check");
    info!("");
    info!("Ready for load testing!");

    axum::serve(listener, app).await?;

    Ok(())
}
