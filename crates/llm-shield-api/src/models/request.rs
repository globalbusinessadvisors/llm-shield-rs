//! Request DTOs

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Scan prompt request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ScanPromptRequest {
    /// Prompt text to scan
    #[validate(length(min = 1, max = 100000, message = "Prompt must be between 1 and 100000 characters"))]
    pub prompt: String,

    /// Scanners to run (empty = all input scanners)
    #[validate(length(max = 20, message = "Maximum 20 scanners allowed"))]
    #[serde(default)]
    pub scanners: Vec<String>,

    /// Enable result caching
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,
}

/// Scan output request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ScanOutputRequest {
    /// Original prompt (for context)
    #[validate(length(min = 1, max = 100000))]
    pub prompt: String,

    /// LLM output to scan
    #[validate(length(min = 1, max = 100000))]
    pub output: String,

    /// Scanners to run (empty = all output scanners)
    #[validate(length(max = 20))]
    #[serde(default)]
    pub scanners: Vec<String>,

    /// Enable result caching
    #[serde(default = "default_cache_enabled")]
    pub cache_enabled: bool,
}

/// Batch scan request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BatchScanRequest {
    /// Items to scan
    #[validate(length(min = 1, max = 100, message = "Batch size must be between 1 and 100"))]
    pub items: Vec<ScanPromptRequest>,

    /// Maximum concurrent scans
    #[serde(default = "default_max_concurrent")]
    #[validate(range(min = 1, max = 10, message = "Max concurrent must be between 1 and 10"))]
    pub max_concurrent: usize,
}

/// Anonymization request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct AnonymizeRequest {
    /// Text to anonymize
    #[validate(length(min = 1, max = 100000))]
    pub text: String,

    /// Entity types to detect (empty = all)
    #[serde(default)]
    pub entity_types: Vec<String>,
}

/// Deanonymization request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DeanonymizeRequest {
    /// Anonymized text with placeholders
    #[validate(length(min = 1, max = 100000))]
    pub text: String,

    /// Session ID from anonymization
    #[validate(length(min = 1, max = 100))]
    pub session_id: String,
}

fn default_cache_enabled() -> bool {
    true
}

fn default_max_concurrent() -> usize {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_prompt_request_valid() {
        let req = ScanPromptRequest {
            prompt: "Test prompt".to_string(),
            scanners: vec!["toxicity".to_string()],
            cache_enabled: true,
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_scan_prompt_request_empty_prompt() {
        let req = ScanPromptRequest {
            prompt: "".to_string(),
            scanners: vec![],
            cache_enabled: true,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_scan_prompt_request_too_long() {
        let req = ScanPromptRequest {
            prompt: "a".repeat(100001),
            scanners: vec![],
            cache_enabled: true,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_scan_prompt_request_too_many_scanners() {
        let req = ScanPromptRequest {
            prompt: "Test".to_string(),
            scanners: vec!["scanner".to_string(); 21],
            cache_enabled: true,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_scan_output_request_valid() {
        let req = ScanOutputRequest {
            prompt: "What is AI?".to_string(),
            output: "AI is artificial intelligence".to_string(),
            scanners: vec![],
            cache_enabled: true,
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_batch_scan_request_valid() {
        let req = BatchScanRequest {
            items: vec![
                ScanPromptRequest {
                    prompt: "Test 1".to_string(),
                    scanners: vec![],
                    cache_enabled: true,
                },
                ScanPromptRequest {
                    prompt: "Test 2".to_string(),
                    scanners: vec![],
                    cache_enabled: true,
                },
            ],
            max_concurrent: 5,
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_batch_scan_request_empty() {
        let req = BatchScanRequest {
            items: vec![],
            max_concurrent: 5,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_batch_scan_request_too_large() {
        let items = (0..101)
            .map(|i| ScanPromptRequest {
                prompt: format!("Test {}", i),
                scanners: vec![],
                cache_enabled: true,
            })
            .collect();

        let req = BatchScanRequest {
            items,
            max_concurrent: 5,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_batch_scan_request_invalid_concurrent() {
        let req = BatchScanRequest {
            items: vec![ScanPromptRequest {
                prompt: "Test".to_string(),
                scanners: vec![],
                cache_enabled: true,
            }],
            max_concurrent: 0,
        };

        assert!(req.validate().is_err());
    }

    #[test]
    fn test_anonymize_request_valid() {
        let req = AnonymizeRequest {
            text: "My email is john@example.com".to_string(),
            entity_types: vec!["EMAIL".to_string()],
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_deanonymize_request_valid() {
        let req = DeanonymizeRequest {
            text: "My email is [EMAIL_1]".to_string(),
            session_id: "session123".to_string(),
        };

        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_serialization() {
        let req = ScanPromptRequest {
            prompt: "Test".to_string(),
            scanners: vec!["toxicity".to_string()],
            cache_enabled: true,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"prompt\""));
        assert!(json.contains("\"scanners\""));
        assert!(json.contains("\"cacheEnabled\""));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{"prompt":"Test","scanners":["toxicity"],"cacheEnabled":true}"#;
        let req: ScanPromptRequest = serde_json::from_str(json).unwrap();

        assert_eq!(req.prompt, "Test");
        assert_eq!(req.scanners.len(), 1);
        assert!(req.cache_enabled);
    }

    #[test]
    fn test_default_values() {
        let json = r#"{"prompt":"Test"}"#;
        let req: ScanPromptRequest = serde_json::from_str(json).unwrap();

        assert!(req.scanners.is_empty());
        assert!(req.cache_enabled);
    }
}
