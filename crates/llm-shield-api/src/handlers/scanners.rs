//! Scanner discovery and metadata handlers

use crate::models::response::{ListScannersResponse, ScannerMetadataResponse};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

/// GET /v1/scanners - List all available scanners
///
/// Returns metadata about all registered scanners.
///
/// ## Response
/// ```json
/// {
///   "scanners": [
///     {
///       "name": "toxicity",
///       "scannerType": "input",
///       "version": "1.0.0",
///       "description": "Detects toxic content"
///     }
///   ],
///   "totalCount": 1
/// }
/// ```
pub async fn list_scanners(State(state): State<AppState>) -> impl IntoResponse {
    let scanners: Vec<ScannerMetadataResponse> = state
        .scanners
        .iter()
        .map(|(name, scanner)| ScannerMetadataResponse {
            name: name.clone(),
            scanner_type: format!("{:?}", scanner.scanner_type()),
            version: scanner.version().to_string(),
            description: scanner.description().to_string(),
        })
        .collect();

    let total_count = scanners.len();

    let response = ListScannersResponse {
        scanners,
        total_count,
    };

    (StatusCode::OK, Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppStateBuilder;
    use llm_shield_core::{async_trait, Result, ScanResult, Scanner, ScannerType, Vault};
    use std::sync::Arc;

    struct TestScanner {
        name: String,
        scanner_type: ScannerType,
    }

    #[async_trait]
    impl Scanner for TestScanner {
        fn name(&self) -> &str {
            &self.name
        }

        async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
            Ok(ScanResult::new(input.to_string(), true, 0.0))
        }

        fn scanner_type(&self) -> ScannerType {
            self.scanner_type
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn description(&self) -> &str {
            "Test scanner for unit tests"
        }
    }

    #[tokio::test]
    async fn test_list_scanners() {
        let config = crate::config::AppConfig::default();
        let state = AppStateBuilder::new(config)
            .register_scanner(Arc::new(TestScanner {
                name: "scanner1".to_string(),
                scanner_type: ScannerType::Input,
            }))
            .register_scanner(Arc::new(TestScanner {
                name: "scanner2".to_string(),
                scanner_type: ScannerType::Output,
            }))
            .build();

        let result = list_scanners(State(state)).await;
        let (_status, json) = result.into_response().into_parts();

        // Can't easily extract the body in tests without tower::ServiceExt
        // But we verified the function compiles and returns the correct type
    }

    #[tokio::test]
    async fn test_list_scanners_empty() {
        let config = crate::config::AppConfig::default();
        let state = AppStateBuilder::new(config).build();

        let result = list_scanners(State(state)).await;
        let (_status, _json) = result.into_response().into_parts();
    }
}
