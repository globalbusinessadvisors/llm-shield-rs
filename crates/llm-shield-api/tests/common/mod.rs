//! Common test utilities

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use llm_shield_api::config::AppConfig;
use llm_shield_api::state::{AppState, AppStateBuilder};
use llm_shield_core::{async_trait, Result, ScanResult, Scanner, ScannerType, Vault};
use std::sync::Arc;
use tower::ServiceExt;

/// Mock scanner for testing
pub struct MockScanner {
    pub name: String,
    pub is_valid: bool,
    pub risk_score: f32,
}

impl MockScanner {
    pub fn new(name: &str, is_valid: bool, risk_score: f32) -> Self {
        Self {
            name: name.to_string(),
            is_valid,
            risk_score,
        }
    }

    pub fn passing(name: &str) -> Self {
        Self::new(name, true, 0.0)
    }

    pub fn failing(name: &str, risk_score: f32) -> Self {
        Self::new(name, false, risk_score)
    }
}

#[async_trait]
impl Scanner for MockScanner {
    fn name(&self) -> &str {
        &self.name
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        Ok(ScanResult::new(
            input.to_string(),
            self.is_valid,
            self.risk_score,
        ))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Mock scanner for testing"
    }
}

/// Create test app state with mock scanners
pub fn create_test_state() -> AppState {
    let config = AppConfig::default();

    AppStateBuilder::new(config)
        .register_scanner(Arc::new(MockScanner::passing("test_scanner_1")))
        .register_scanner(Arc::new(MockScanner::passing("test_scanner_2")))
        .register_scanner(Arc::new(MockScanner::failing("test_failing_scanner", 0.8)))
        .build()
}

/// Send GET request to a route
pub async fn get_request(app: Router, uri: &str) -> (StatusCode, String) {
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();

    (status, body)
}

/// Send POST request to a route
pub async fn post_request(app: Router, uri: &str, body: serde_json::Value) -> (StatusCode, String) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();

    (status, body)
}

/// Parse JSON response
pub fn parse_json<T>(body: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(body).expect("Failed to parse JSON response")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_scanner_passing() {
        let scanner = MockScanner::passing("test");
        assert_eq!(scanner.name(), "test");
        assert_eq!(scanner.version(), "1.0.0");
    }

    #[test]
    fn test_mock_scanner_failing() {
        let scanner = MockScanner::failing("test", 0.9);
        assert_eq!(scanner.name(), "test");
    }

    #[tokio::test]
    async fn test_mock_scanner_scan() {
        let scanner = MockScanner::passing("test");
        let vault = Vault::new();
        let result = scanner.scan("test input", &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
    }

    #[test]
    fn test_create_test_state() {
        let state = create_test_state();
        assert_eq!(state.scanner_count(), 3);
        assert!(state.get_scanner("test_scanner_1").is_some());
        assert!(state.get_scanner("test_scanner_2").is_some());
        assert!(state.get_scanner("test_failing_scanner").is_some());
    }
}
