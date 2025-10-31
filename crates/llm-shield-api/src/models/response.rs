//! Response DTOs

use serde::{Deserialize, Serialize};

/// Scan result response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResponse {
    /// Whether the input is valid (passed all scanners)
    pub is_valid: bool,

    /// Highest risk score from all scanners (0.0-1.0)
    pub risk_score: f32,

    /// Sanitized text (with detected entities removed/replaced)
    pub sanitized_text: String,

    /// Scanner results
    pub scanner_results: Vec<ScannerResult>,

    /// Processing time in milliseconds
    pub scan_time_ms: u64,

    /// Whether result came from cache
    #[serde(default)]
    pub cache_hit: bool,
}

/// Individual scanner result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannerResult {
    /// Scanner name
    pub scanner: String,

    /// Whether this scanner passed
    pub is_valid: bool,

    /// Risk score from this scanner (0.0-1.0)
    pub risk_score: f32,

    /// Risk factors detected
    #[serde(default)]
    pub risk_factors: Vec<RiskFactorDto>,

    /// Entities detected
    #[serde(default)]
    pub entities: Vec<EntityDto>,

    /// Scanner execution time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_time_ms: Option<u64>,
}

/// Risk factor DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskFactorDto {
    /// Risk factor description
    pub description: String,

    /// Severity level
    pub severity: String,

    /// Risk score (0.0-1.0)
    pub score: f32,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Entity DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityDto {
    /// Entity type
    pub entity_type: String,

    /// Matched text
    pub text: String,

    /// Start position in original text
    pub start: usize,

    /// End position in original text
    pub end: usize,

    /// Confidence score (0.0-1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

/// Batch scan response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchScanResponse {
    /// Individual scan results
    pub results: Vec<ScanResponse>,

    /// Total processing time in milliseconds
    pub total_time_ms: u64,

    /// Number of items processed successfully
    pub success_count: usize,

    /// Number of items that failed
    pub failure_count: usize,
}

/// Anonymization response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnonymizeResponse {
    /// Anonymized text with placeholders
    pub anonymized_text: String,

    /// Session ID for deanonymization
    pub session_id: String,

    /// Entities detected and anonymized
    pub entities: Vec<AnonymizedEntityDto>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Anonymized entity DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnonymizedEntityDto {
    /// Entity type (e.g., EMAIL, PHONE)
    pub entity_type: String,

    /// Placeholder used (e.g., [EMAIL_1])
    pub placeholder: String,

    /// Position in original text
    pub start: usize,

    /// End position in original text
    pub end: usize,
}

/// Deanonymization response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeanonymizeResponse {
    /// Restored text with original values
    pub text: String,

    /// Number of placeholders restored
    pub restored_count: usize,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Scanner metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannerMetadataResponse {
    /// Scanner name
    pub name: String,

    /// Scanner type (input/output/bidirectional)
    pub scanner_type: String,

    /// Scanner version
    pub version: String,

    /// Scanner description
    pub description: String,
}

/// List scanners response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListScannersResponse {
    /// Available scanners
    pub scanners: Vec<ScannerMetadataResponse>,

    /// Total count
    pub total_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_response_serialization() {
        let response = ScanResponse {
            is_valid: true,
            risk_score: 0.1,
            sanitized_text: "Test text".to_string(),
            scanner_results: vec![],
            scan_time_ms: 50,
            cache_hit: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"isValid\":true"));
        assert!(json.contains("\"riskScore\":0.1"));
        assert!(json.contains("\"scanTimeMs\":50"));
    }

    #[test]
    fn test_scanner_result_serialization() {
        let result = ScannerResult {
            scanner: "toxicity".to_string(),
            is_valid: true,
            risk_score: 0.0,
            risk_factors: vec![],
            entities: vec![],
            execution_time_ms: Some(10),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"scanner\":\"toxicity\""));
        assert!(json.contains("\"executionTimeMs\":10"));
    }

    #[test]
    fn test_risk_factor_dto() {
        let risk_factor = RiskFactorDto {
            description: "Toxic language detected".to_string(),
            severity: "high".to_string(),
            score: 0.8,
            metadata: Some(serde_json::json!({"details": "test"})),
        };

        let json = serde_json::to_string(&risk_factor).unwrap();
        assert!(json.contains("\"description\""));
        assert!(json.contains("\"severity\":\"high\""));
    }

    #[test]
    fn test_entity_dto() {
        let entity = EntityDto {
            entity_type: "EMAIL".to_string(),
            text: "test@example.com".to_string(),
            start: 10,
            end: 26,
            confidence: Some(0.95),
        };

        let json = serde_json::to_string(&entity).unwrap();
        assert!(json.contains("\"entityType\":\"EMAIL\""));
        assert!(json.contains("\"confidence\":0.95"));
    }

    #[test]
    fn test_batch_scan_response() {
        let response = BatchScanResponse {
            results: vec![],
            total_time_ms: 100,
            success_count: 5,
            failure_count: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"totalTimeMs\":100"));
        assert!(json.contains("\"successCount\":5"));
    }

    #[test]
    fn test_anonymize_response() {
        let response = AnonymizeResponse {
            anonymized_text: "My email is [EMAIL_1]".to_string(),
            session_id: "session123".to_string(),
            entities: vec![],
            processing_time_ms: 25,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"sessionId\":\"session123\""));
        assert!(json.contains("\"processingTimeMs\":25"));
    }

    #[test]
    fn test_deanonymize_response() {
        let response = DeanonymizeResponse {
            text: "My email is test@example.com".to_string(),
            restored_count: 1,
            processing_time_ms: 15,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"restoredCount\":1"));
    }

    #[test]
    fn test_scanner_metadata_response() {
        let metadata = ScannerMetadataResponse {
            name: "toxicity".to_string(),
            scanner_type: "input".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects toxic language".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"scannerType\":\"input\""));
    }

    #[test]
    fn test_list_scanners_response() {
        let response = ListScannersResponse {
            scanners: vec![],
            total_count: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"totalCount\":0"));
    }

    #[test]
    fn test_camel_case_serialization() {
        let response = ScanResponse {
            is_valid: true,
            risk_score: 0.0,
            sanitized_text: "test".to_string(),
            scanner_results: vec![],
            scan_time_ms: 10,
            cache_hit: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        // Check camelCase
        assert!(json.contains("isValid"));
        assert!(json.contains("riskScore"));
        assert!(json.contains("sanitizedText"));
        assert!(json.contains("scannerResults"));
        assert!(json.contains("scanTimeMs"));
        assert!(json.contains("cacheHit"));
    }
}
