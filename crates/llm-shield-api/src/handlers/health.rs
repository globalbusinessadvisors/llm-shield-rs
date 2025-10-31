//! Health check endpoints

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Service version
    pub version: String,

    /// Uptime in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_secs: Option<u64>,
}

/// Basic health check endpoint
///
/// Returns 200 OK if service is running
///
/// ## TDD Design
/// - Tests written first in tests module
/// - Simple, fast endpoint (< 1ms)
/// - No dependencies on external services
pub async fn health() -> impl IntoResponse {
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: None,
    };

    (StatusCode::OK, Json(response))
}

/// Readiness check endpoint
///
/// Returns 200 OK if service is ready to accept requests
/// Checks:
/// - Models loaded (if required)
/// - Cache accessible
///
/// ## Kubernetes Integration
/// Used by readiness probe to determine if pod should receive traffic
pub async fn ready() -> impl IntoResponse {
    // TODO: Check if models are loaded
    // TODO: Check if cache is accessible

    let response = HealthResponse {
        status: "ready".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: None,
    };

    (StatusCode::OK, Json(response))
}

/// Liveness check endpoint
///
/// Returns 200 OK if service is alive and responding
///
/// ## Kubernetes Integration
/// Used by liveness probe to determine if pod should be restarted
pub async fn live() -> impl IntoResponse {
    let response = HealthResponse {
        status: "live".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: None,
    };

    (StatusCode::OK, Json(response))
}

/// Version information endpoint
pub async fn version() -> impl IntoResponse {
    #[derive(Serialize)]
    struct VersionInfo {
        version: String,
        build_time: String,
        git_commit: String,
    }

    let info = VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_time: "unknown".to_string(), // TODO: Add build time
        git_commit: "unknown".to_string(), // TODO: Add git commit
    };

    (StatusCode::OK, Json(info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let response = health().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint() {
        let response = ready().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_live_endpoint() {
        let response = live().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_version_endpoint() {
        let response = version().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
