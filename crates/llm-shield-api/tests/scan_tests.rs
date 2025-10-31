//! Integration tests for scan endpoints

use axum::http::StatusCode;
use llm_shield_api::models::response::{BatchScanResponse, ScanResponse};
use llm_shield_api::router::create_router_with_state;
use serde_json::json;

mod common;
use common::{create_test_state, parse_json, post_request};

#[tokio::test]
async fn test_scan_prompt_endpoint_success() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Hello world, this is a test prompt",
        "scanners": ["test_scanner_1"],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    assert!(response.is_valid);
    assert_eq!(response.risk_score, 0.0);
    assert_eq!(response.scanner_results.len(), 1);
    assert_eq!(response.scanner_results[0].scanner, "test_scanner_1");
    assert!(!response.cache_hit);
}

#[tokio::test]
async fn test_scan_prompt_endpoint_failing_scanner() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test prompt",
        "scanners": ["test_failing_scanner"],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    assert!(!response.is_valid); // Should fail
    assert_eq!(response.risk_score, 0.8); // From failing scanner
    assert_eq!(response.scanner_results.len(), 1);
    assert_eq!(response.scanner_results[0].scanner, "test_failing_scanner");
}

#[tokio::test]
async fn test_scan_prompt_endpoint_multiple_scanners() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test with multiple scanners",
        "scanners": ["test_scanner_1", "test_scanner_2"],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    assert!(response.is_valid);
    assert_eq!(response.scanner_results.len(), 2);
    assert_eq!(response.scanner_results[0].scanner, "test_scanner_1");
    assert_eq!(response.scanner_results[1].scanner, "test_scanner_2");
}

#[tokio::test]
async fn test_scan_prompt_endpoint_all_scanners() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test with all scanners",
        "scanners": [],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    // Should run all 3 scanners (test_scanner_1, test_scanner_2, test_failing_scanner)
    assert!(!response.is_valid); // One scanner fails
    assert_eq!(response.risk_score, 0.8); // Max risk from failing scanner
    assert_eq!(response.scanner_results.len(), 3);
}

#[tokio::test]
async fn test_scan_prompt_endpoint_empty_prompt_validation() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "",
        "scanners": [],
        "cacheEnabled": false
    });

    let (status, _body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Validation error returns 422
}

#[tokio::test]
async fn test_scan_prompt_endpoint_nonexistent_scanner() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test prompt",
        "scanners": ["nonexistent_scanner"],
        "cacheEnabled": false
    });

    let (status, _body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::NOT_FOUND); // Scanner not found
}

#[tokio::test]
async fn test_scan_prompt_endpoint_timing() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test timing",
        "scanners": ["test_scanner_1"],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    // Timing should be measured (very fast for mock scanners, but should be reasonable)
    assert!(response.scan_time_ms < 1000); // Should complete in under 1 second
    assert!(response.scanner_results[0].execution_time_ms.is_some());
    if let Some(exec_time) = response.scanner_results[0].execution_time_ms {
        assert!(exec_time < 1000); // Individual scanner should also be fast
    }
}

#[tokio::test]
async fn test_scan_prompt_endpoint_default_cache_enabled() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    // Without cacheEnabled field, should default to true
    let req_body = json!({
        "prompt": "Test default cache",
        "scanners": ["test_scanner_1"]
    });

    let (status, body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    assert!(response.is_valid);
}

#[tokio::test]
async fn test_scan_prompt_endpoint_invalid_json() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    // Missing required field
    let req_body = json!({
        "scanners": ["test_scanner_1"],
        "cacheEnabled": false
    });

    let (status, _body) = post_request(app, "/v1/scan/prompt", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Deserialization error
}

// Tests for POST /v1/scan/output

#[tokio::test]
async fn test_scan_output_endpoint_success() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "What is the weather today?",
        "output": "The weather is sunny with a high of 75°F.",
        "scanners": ["test_scanner_1"],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/output", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    assert!(response.is_valid);
    assert_eq!(response.risk_score, 0.0);
    assert_eq!(response.scanner_results.len(), 1);
    assert_eq!(response.scanner_results[0].scanner, "test_scanner_1");
    assert!(!response.cache_hit);
}

#[tokio::test]
async fn test_scan_output_endpoint_failing_scanner() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test prompt",
        "output": "Test output with risk",
        "scanners": ["test_failing_scanner"],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/output", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    assert!(!response.is_valid); // Should fail
    assert_eq!(response.risk_score, 0.8); // From failing scanner
    assert_eq!(response.scanner_results.len(), 1);
}

#[tokio::test]
async fn test_scan_output_endpoint_empty_prompt_validation() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "",
        "output": "Some output",
        "scanners": [],
        "cacheEnabled": false
    });

    let (status, _body) = post_request(app, "/v1/scan/output", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Validation error
}

#[tokio::test]
async fn test_scan_output_endpoint_empty_output_validation() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test prompt",
        "output": "",
        "scanners": [],
        "cacheEnabled": false
    });

    let (status, _body) = post_request(app, "/v1/scan/output", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Validation error
}

#[tokio::test]
async fn test_scan_output_endpoint_multiple_scanners() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test prompt",
        "output": "Test output with multiple scanners",
        "scanners": ["test_scanner_1", "test_scanner_2"],
        "cacheEnabled": false
    });

    let (status, body) = post_request(app, "/v1/scan/output", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: ScanResponse = parse_json(&body);
    assert!(response.is_valid);
    assert_eq!(response.scanner_results.len(), 2);
}

#[tokio::test]
async fn test_scan_output_endpoint_nonexistent_scanner() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "prompt": "Test prompt",
        "output": "Test output",
        "scanners": ["nonexistent_scanner"],
        "cacheEnabled": false
    });

    let (status, _body) = post_request(app, "/v1/scan/output", req_body).await;

    assert_eq!(status, StatusCode::NOT_FOUND); // Scanner not found
}

#[tokio::test]
async fn test_scan_output_endpoint_invalid_json() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    // Missing output field
    let req_body = json!({
        "prompt": "Test prompt",
        "scanners": [],
        "cacheEnabled": false
    });

    let (status, _body) = post_request(app, "/v1/scan/output", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Deserialization error
}

// Tests for POST /v1/scan/batch

#[tokio::test]
async fn test_scan_batch_endpoint_success() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "items": [
            {
                "prompt": "First prompt to scan",
                "scanners": ["test_scanner_1"],
                "cacheEnabled": false
            },
            {
                "prompt": "Second prompt to scan",
                "scanners": ["test_scanner_2"],
                "cacheEnabled": false
            }
        ],
        "maxConcurrent": 2
    });

    let (status, body) = post_request(app, "/v1/scan/batch", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: BatchScanResponse = parse_json(&body);
    assert_eq!(response.results.len(), 2);
    assert_eq!(response.success_count, 2);
    assert_eq!(response.failure_count, 0);
    assert!(response.total_time_ms < 1000); // Should complete quickly
}

#[tokio::test]
async fn test_scan_batch_endpoint_multiple_items() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let items: Vec<_> = (0..5)
        .map(|i| json!({
            "prompt": format!("Test prompt {}", i),
            "scanners": [],
            "cacheEnabled": false
        }))
        .collect();

    let req_body = json!({
        "items": items,
        "maxConcurrent": 3
    });

    let (status, body) = post_request(app, "/v1/scan/batch", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: BatchScanResponse = parse_json(&body);
    // Note: may have failures if "all scanners" returns empty for bidirectional-only state
    // But with our test state which has Input scanners, should work
    assert_eq!(response.results.len() + response.failure_count, 5);
}

#[tokio::test]
async fn test_scan_batch_endpoint_empty_items_validation() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "items": [],
        "maxConcurrent": 2
    });

    let (status, _body) = post_request(app, "/v1/scan/batch", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Validation error
}

#[tokio::test]
async fn test_scan_batch_endpoint_invalid_concurrency() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "items": [
            {
                "prompt": "Test prompt",
                "scanners": [],
                "cacheEnabled": false
            }
        ],
        "maxConcurrent": 0 // Invalid: must be >= 1
    });

    let (status, _body) = post_request(app, "/v1/scan/batch", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Validation error
}

#[tokio::test]
async fn test_scan_batch_endpoint_high_concurrency() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "items": [
            {
                "prompt": "Test prompt",
                "scanners": [],
                "cacheEnabled": false
            }
        ],
        "maxConcurrent": 11 // Invalid: max is 10
    });

    let (status, _body) = post_request(app, "/v1/scan/batch", req_body).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY); // Validation error
}

#[tokio::test]
async fn test_scan_batch_endpoint_default_concurrency() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    // Without maxConcurrent, should default to 5
    let req_body = json!({
        "items": [
            {
                "prompt": "Test prompt 1",
                "scanners": ["test_scanner_1"],
                "cacheEnabled": false
            },
            {
                "prompt": "Test prompt 2",
                "scanners": ["test_scanner_1"],
                "cacheEnabled": false
            }
        ]
    });

    let (status, body) = post_request(app, "/v1/scan/batch", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: BatchScanResponse = parse_json(&body);
    assert_eq!(response.success_count, 2);
}

#[tokio::test]
async fn test_scan_batch_endpoint_with_failures() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let req_body = json!({
        "items": [
            {
                "prompt": "Valid prompt",
                "scanners": ["test_scanner_1"],
                "cacheEnabled": false
            },
            {
                "prompt": "Invalid with bad scanner",
                "scanners": ["nonexistent_scanner"],
                "cacheEnabled": false
            }
        ],
        "maxConcurrent": 2
    });

    let (status, body) = post_request(app, "/v1/scan/batch", req_body).await;

    assert_eq!(status, StatusCode::OK);

    let response: BatchScanResponse = parse_json(&body);
    // Should have 1 success and 1 failure
    assert_eq!(response.success_count, 1);
    assert_eq!(response.failure_count, 1);
}
