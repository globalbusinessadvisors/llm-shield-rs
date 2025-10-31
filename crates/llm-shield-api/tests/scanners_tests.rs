//! Integration tests for scanners endpoint

use axum::http::StatusCode;
use llm_shield_api::models::response::ListScannersResponse;
use llm_shield_api::router::create_router_with_state;

mod common;
use common::{create_test_state, get_request, parse_json};

#[tokio::test]
async fn test_list_scanners_endpoint() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let (status, body) = get_request(app, "/v1/scanners").await;

    assert_eq!(status, StatusCode::OK);

    let response: ListScannersResponse = parse_json(&body);
    assert_eq!(response.total_count, 3); // test_scanner_1, test_scanner_2, test_failing_scanner
    assert_eq!(response.scanners.len(), 3);

    // Verify scanner metadata is present
    assert!(response.scanners.iter().any(|s| s.name == "test_scanner_1"));
    assert!(response.scanners.iter().any(|s| s.name == "test_scanner_2"));
    assert!(response
        .scanners
        .iter()
        .any(|s| s.name == "test_failing_scanner"));
}

#[tokio::test]
async fn test_list_scanners_metadata() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let (status, body) = get_request(app, "/v1/scanners").await;

    assert_eq!(status, StatusCode::OK);

    let response: ListScannersResponse = parse_json(&body);

    // Verify each scanner has required metadata
    for scanner in &response.scanners {
        assert!(!scanner.name.is_empty());
        assert!(!scanner.scanner_type.is_empty());
        assert!(!scanner.version.is_empty());
        assert!(!scanner.description.is_empty());
    }
}

#[tokio::test]
async fn test_list_scanners_types() {
    let state = create_test_state();
    let app = create_router_with_state(state);

    let (status, body) = get_request(app, "/v1/scanners").await;

    assert_eq!(status, StatusCode::OK);

    let response: ListScannersResponse = parse_json(&body);

    // All test scanners are Input type
    for scanner in &response.scanners {
        assert_eq!(scanner.scanner_type, "Input");
    }
}
