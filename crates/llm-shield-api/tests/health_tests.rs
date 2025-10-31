//! Integration tests for health endpoints

mod common;

use axum::http::StatusCode;
use common::get_request;
use llm_shield_api::router::create_router;

#[tokio::test]
async fn test_health_endpoint_integration() {
    let app = create_router();
    let (status, body) = get_request(app, "/health").await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn test_ready_endpoint_integration() {
    let app = create_router();
    let (status, body) = get_request(app, "/health/ready").await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "ready");
}

#[tokio::test]
async fn test_live_endpoint_integration() {
    let app = create_router();
    let (status, body) = get_request(app, "/health/live").await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "live");
}

#[tokio::test]
async fn test_version_endpoint_integration() {
    let app = create_router();
    let (status, body) = get_request(app, "/version").await;

    assert_eq!(status, StatusCode::OK);

    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["version"].is_string());
    assert!(json["build_time"].is_string());
    assert!(json["git_commit"].is_string());
}

#[tokio::test]
async fn test_404_not_found() {
    let app = create_router();
    let (status, _body) = get_request(app, "/nonexistent").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_health_endpoints_response_time() {
    use std::time::Instant;

    let app = create_router();

    let start = Instant::now();
    let (status, _) = get_request(app, "/health").await;
    let duration = start.elapsed();

    assert_eq!(status, StatusCode::OK);
    // Health endpoint should respond in < 10ms
    assert!(
        duration.as_millis() < 10,
        "Health endpoint took {}ms (should be < 10ms)",
        duration.as_millis()
    );
}

#[tokio::test]
async fn test_concurrent_health_requests() {
    use futures::future::join_all;

    let app = create_router();

    // Send 10 concurrent requests
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let app = app.clone();
            tokio::spawn(async move { get_request(app, "/health").await })
        })
        .collect();

    let results = join_all(handles).await;

    // All requests should succeed
    for result in results {
        let (status, _) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
    }
}
