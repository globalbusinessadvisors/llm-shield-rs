//! Scanner service for executing scans

use crate::models::{EntityDto, RiskFactorDto, ScanResponse, ScannerResult};
use llm_shield_core::{Scanner, ScanResult, Vault};
use std::sync::Arc;
use std::time::Instant;

/// Scanner execution service
pub struct ScannerService {
    vault: Arc<Vault>,
}

impl ScannerService {
    /// Create new scanner service
    pub fn new() -> Self {
        Self {
            vault: Arc::new(Vault::new()),
        }
    }

    /// Execute a single scanner
    pub async fn execute_scanner(
        &self,
        scanner: Arc<dyn Scanner>,
        input: &str,
    ) -> Result<ScannerResult, String> {
        let start = Instant::now();

        let scan_result = scanner
            .scan(input, &self.vault)
            .await
            .map_err(|e| format!("Scanner execution failed: {}", e))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(self.convert_scan_result(scanner.name(), scan_result, Some(execution_time_ms)))
    }

    /// Execute multiple scanners in sequence
    pub async fn execute_scanners(
        &self,
        scanners: Vec<Arc<dyn Scanner>>,
        input: &str,
    ) -> Result<Vec<ScannerResult>, String> {
        let mut results = Vec::new();

        for scanner in scanners {
            let result = self.execute_scanner(scanner, input).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Create scan response from scanner results
    pub fn create_scan_response(
        &self,
        scanner_results: Vec<ScannerResult>,
        scan_time_ms: u64,
        cache_hit: bool,
    ) -> ScanResponse {
        // Calculate overall validity (all scanners must pass)
        let is_valid = scanner_results.iter().all(|r| r.is_valid);

        // Get highest risk score
        let risk_score = scanner_results
            .iter()
            .map(|r| r.risk_score)
            .fold(0.0f32, f32::max);

        // Get sanitized text from first result (or use original if no results)
        let sanitized_text = scanner_results
            .first()
            .map(|r| r.scanner.clone()) // Placeholder: should be actual sanitized text
            .unwrap_or_else(|| "".to_string());

        ScanResponse {
            is_valid,
            risk_score,
            sanitized_text,
            scanner_results,
            scan_time_ms,
            cache_hit,
        }
    }

    /// Convert ScanResult to ScannerResult DTO
    fn convert_scan_result(
        &self,
        scanner_name: &str,
        scan_result: ScanResult,
        execution_time_ms: Option<u64>,
    ) -> ScannerResult {
        // Convert risk factors
        let risk_factors = scan_result
            .risk_factors
            .iter()
            .map(|rf| RiskFactorDto {
                description: rf.description.clone(),
                severity: format!("{:?}", rf.severity),
                score: rf.score_contribution,
                metadata: None, // RiskFactor doesn't have metadata field
            })
            .collect();

        // Convert entities
        let entities = scan_result
            .entities
            .iter()
            .map(|e| EntityDto {
                entity_type: e.entity_type.clone(),
                text: e.text.clone(),
                start: e.start,
                end: e.end,
                confidence: Some(e.confidence),
            })
            .collect();

        ScannerResult {
            scanner: scanner_name.to_string(),
            is_valid: scan_result.is_valid,
            risk_score: scan_result.risk_score,
            risk_factors,
            entities,
            execution_time_ms,
        }
    }
}

impl Default for ScannerService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_shield_core::{async_trait, Entity, Result, RiskFactor, ScannerType, Severity};

    struct TestScanner {
        name: String,
        is_valid: bool,
        risk_score: f32,
    }

    #[async_trait]
    impl Scanner for TestScanner {
        fn name(&self) -> &str {
            &self.name
        }

        async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
            let mut result = ScanResult::new(input.to_string(), self.is_valid, self.risk_score);

            if !self.is_valid {
                result = result.with_risk_factor(RiskFactor::new(
                    "test_risk",
                    "Test risk factor",
                    Severity::High,
                    self.risk_score,
                ));
            }

            Ok(result)
        }

        fn scanner_type(&self) -> ScannerType {
            ScannerType::Input
        }
    }

    #[tokio::test]
    async fn test_execute_single_scanner_success() {
        let service = ScannerService::new();
        let scanner = Arc::new(TestScanner {
            name: "test_scanner".to_string(),
            is_valid: true,
            risk_score: 0.0,
        });

        let result = service.execute_scanner(scanner, "test input").await;

        assert!(result.is_ok());
        let scanner_result = result.unwrap();
        assert_eq!(scanner_result.scanner, "test_scanner");
        assert!(scanner_result.is_valid);
        assert_eq!(scanner_result.risk_score, 0.0);
        assert!(scanner_result.execution_time_ms.is_some());
    }

    #[tokio::test]
    async fn test_execute_single_scanner_failing() {
        let service = ScannerService::new();
        let scanner = Arc::new(TestScanner {
            name: "test_scanner".to_string(),
            is_valid: false,
            risk_score: 0.8,
        });

        let result = service.execute_scanner(scanner, "test input").await;

        assert!(result.is_ok());
        let scanner_result = result.unwrap();
        assert!(!scanner_result.is_valid);
        assert_eq!(scanner_result.risk_score, 0.8);
        assert!(!scanner_result.risk_factors.is_empty());
    }

    #[tokio::test]
    async fn test_execute_multiple_scanners() {
        let service = ScannerService::new();
        let scanners = vec![
            Arc::new(TestScanner {
                name: "scanner1".to_string(),
                is_valid: true,
                risk_score: 0.0,
            }) as Arc<dyn Scanner>,
            Arc::new(TestScanner {
                name: "scanner2".to_string(),
                is_valid: true,
                risk_score: 0.1,
            }),
        ];

        let result = service.execute_scanners(scanners, "test input").await;

        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].scanner, "scanner1");
        assert_eq!(results[1].scanner, "scanner2");
    }

    #[tokio::test]
    async fn test_create_scan_response_all_valid() {
        let service = ScannerService::new();
        let scanner_results = vec![
            ScannerResult {
                scanner: "scanner1".to_string(),
                is_valid: true,
                risk_score: 0.1,
                risk_factors: vec![],
                entities: vec![],
                execution_time_ms: Some(10),
            },
            ScannerResult {
                scanner: "scanner2".to_string(),
                is_valid: true,
                risk_score: 0.2,
                risk_factors: vec![],
                entities: vec![],
                execution_time_ms: Some(15),
            },
        ];

        let response = service.create_scan_response(scanner_results, 100, false);

        assert!(response.is_valid);
        assert_eq!(response.risk_score, 0.2); // Max of 0.1 and 0.2
        assert_eq!(response.scan_time_ms, 100);
        assert!(!response.cache_hit);
        assert_eq!(response.scanner_results.len(), 2);
    }

    #[tokio::test]
    async fn test_create_scan_response_one_failing() {
        let service = ScannerService::new();
        let scanner_results = vec![
            ScannerResult {
                scanner: "scanner1".to_string(),
                is_valid: true,
                risk_score: 0.1,
                risk_factors: vec![],
                entities: vec![],
                execution_time_ms: Some(10),
            },
            ScannerResult {
                scanner: "scanner2".to_string(),
                is_valid: false,
                risk_score: 0.9,
                risk_factors: vec![],
                entities: vec![],
                execution_time_ms: Some(15),
            },
        ];

        let response = service.create_scan_response(scanner_results, 100, false);

        assert!(!response.is_valid); // Should be false if any scanner fails
        assert_eq!(response.risk_score, 0.9); // Max risk score
    }

    #[tokio::test]
    async fn test_convert_scan_result_with_entities() {
        let service = ScannerService::new();
        let mut scan_result = ScanResult::new("test".to_string(), true, 0.0);
        scan_result = scan_result.with_entity(Entity::new(
            "EMAIL",
            "test@example.com",
            0,
            16,
            0.95,
        ));

        let scanner_result = service.convert_scan_result("test_scanner", scan_result, Some(10));

        assert_eq!(scanner_result.entities.len(), 1);
        assert_eq!(scanner_result.entities[0].entity_type, "EMAIL");
        assert_eq!(scanner_result.entities[0].confidence, Some(0.95));
    }
}
