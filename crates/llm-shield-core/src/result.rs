//! Scan result types
//!
//! ## SPARC Specification
//!
//! Standardized result format for all scanners:
//! - `ScanResult`: Main result structure
//! - `Entity`: Detected entities (PII, secrets, etc.)
//! - `RiskFactor`: Individual risk factors
//! - `Severity`: Risk severity levels

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a security scan
///
/// ## Enterprise Design
///
/// - **Immutable**: Once created, cannot be modified
/// - **Serializable**: Can be sent over network or stored
/// - **Rich Metadata**: Includes detailed information for debugging
/// - **Composable**: Multiple results can be combined
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScanResult {
    /// The sanitized/modified text (if applicable)
    pub sanitized_text: String,

    /// Whether the input passed validation
    pub is_valid: bool,

    /// Risk score from 0.0 (no risk) to 1.0 (maximum risk)
    pub risk_score: f32,

    /// Detected entities (PII, secrets, banned content, etc.)
    pub entities: Vec<Entity>,

    /// Risk factors that contributed to the score
    pub risk_factors: Vec<RiskFactor>,

    /// Additional scanner-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ScanResult {
    /// Create a new scan result
    pub fn new(sanitized_text: String, is_valid: bool, risk_score: f32) -> Self {
        Self {
            sanitized_text,
            is_valid,
            risk_score,
            entities: Vec::new(),
            risk_factors: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a passing scan result (no risks detected)
    pub fn pass(text: String) -> Self {
        Self::new(text, true, 0.0)
    }

    /// Create a failing scan result with risk score
    pub fn fail(text: String, risk_score: f32) -> Self {
        Self::new(text, false, risk_score)
    }

    /// Add an entity to the result
    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.entities.push(entity);
        self
    }

    /// Add a risk factor to the result
    pub fn with_risk_factor(mut self, factor: RiskFactor) -> Self {
        self.risk_factors.push(factor);
        self
    }

    /// Add metadata to the result
    pub fn with_metadata<K: Into<String>, V: Serialize>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), json_value);
        }
        self
    }

    /// Get the overall severity level
    pub fn severity(&self) -> Severity {
        if self.risk_score >= 0.9 {
            Severity::Critical
        } else if self.risk_score >= 0.7 {
            Severity::High
        } else if self.risk_score >= 0.4 {
            Severity::Medium
        } else if self.risk_score > 0.0 {
            Severity::Low
        } else {
            Severity::None
        }
    }

    /// Combine multiple scan results
    ///
    /// Takes the maximum risk score and merges entities
    pub fn combine(results: Vec<ScanResult>) -> Self {
        if results.is_empty() {
            return Self::pass(String::new());
        }

        let max_risk = results
            .iter()
            .map(|r| r.risk_score)
            .fold(0.0f32, f32::max);

        let is_valid = results.iter().all(|r| r.is_valid);

        let mut combined = Self::new(
            results[0].sanitized_text.clone(),
            is_valid,
            max_risk,
        );

        for result in results {
            combined.entities.extend(result.entities);
            combined.risk_factors.extend(result.risk_factors);
            for (k, v) in result.metadata {
                combined.metadata.insert(k, v);
            }
        }

        combined
    }
}

/// A detected entity in the scanned text
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    /// Type of entity (e.g., "email", "ssn", "api_key")
    pub entity_type: String,

    /// The detected text
    pub text: String,

    /// Start position in original text
    pub start: usize,

    /// End position in original text
    pub end: usize,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,

    /// Additional entity-specific data
    pub metadata: HashMap<String, String>,
}

impl Entity {
    /// Create a new entity
    pub fn new<S: Into<String>>(
        entity_type: S,
        text: S,
        start: usize,
        end: usize,
        confidence: f32,
    ) -> Self {
        Self {
            entity_type: entity_type.into(),
            text: text.into(),
            start,
            end,
            confidence,
            metadata: HashMap::new(),
        }
    }
}

/// A risk factor contributing to the overall risk score
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor type (e.g., "prompt_injection", "toxicity")
    pub factor_type: String,

    /// Human-readable description
    pub description: String,

    /// Severity level
    pub severity: Severity,

    /// Contribution to overall risk score
    pub score_contribution: f32,
}

impl RiskFactor {
    /// Create a new risk factor
    pub fn new<S: Into<String>>(
        factor_type: S,
        description: S,
        severity: Severity,
        score_contribution: f32,
    ) -> Self {
        Self {
            factor_type: factor_type.into(),
            description: description.into(),
            severity,
            score_contribution,
        }
    }
}

/// Severity levels for risks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// No risk detected
    None,
    /// Low severity (0.0 < score < 0.4)
    Low,
    /// Medium severity (0.4 <= score < 0.7)
    Medium,
    /// High severity (0.7 <= score < 0.9)
    High,
    /// Critical severity (score >= 0.9)
    Critical,
}

impl Severity {
    /// Get numeric threshold for this severity
    pub fn threshold(&self) -> f32 {
        match self {
            Severity::None => 0.0,
            Severity::Low => 0.01,
            Severity::Medium => 0.4,
            Severity::High => 0.7,
            Severity::Critical => 0.9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_result_creation() {
        let result = ScanResult::pass("test text".to_string());
        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
        assert_eq!(result.severity(), Severity::None);
    }

    #[test]
    fn test_scan_result_fail() {
        let result = ScanResult::fail("bad text".to_string(), 0.85);
        assert!(!result.is_valid);
        assert_eq!(result.risk_score, 0.85);
        assert_eq!(result.severity(), Severity::High);
    }

    #[test]
    fn test_scan_result_builder() {
        let entity = Entity::new("email", "test@example.com", 0, 16, 0.95);
        let factor = RiskFactor::new(
            "banned_content",
            "Email address detected",
            Severity::Low,
            0.2,
        );

        let result = ScanResult::pass("text".to_string())
            .with_entity(entity)
            .with_risk_factor(factor)
            .with_metadata("scanner", "test");

        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.risk_factors.len(), 1);
        assert!(result.metadata.contains_key("scanner"));
    }

    #[test]
    fn test_combine_results() {
        let r1 = ScanResult::fail("text1".to_string(), 0.3);
        let r2 = ScanResult::fail("text2".to_string(), 0.7);
        let r3 = ScanResult::pass("text3".to_string());

        let combined = ScanResult::combine(vec![r1, r2, r3]);
        assert_eq!(combined.risk_score, 0.7);
        assert!(!combined.is_valid);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::None);
    }
}
