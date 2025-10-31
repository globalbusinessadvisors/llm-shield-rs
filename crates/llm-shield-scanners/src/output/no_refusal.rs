//! NoRefusal Output Scanner
//!
//! Converted from llm_guard/output_scanners/no_refusal.py
//!
//! ## SPARC Implementation
//!
//! Detects when an LLM refuses to answer legitimate user requests.
//! This is critical for detecting over-cautious models that refuse benign queries.
//!
//! ## London School TDD
//!
//! Tests written first drive the implementation.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// NoRefusal scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoRefusalConfig {
    /// Detection threshold (0.0 to 1.0)
    pub threshold: f32,

    /// Match sensitivity (strict = more patterns, loose = fewer patterns)
    pub sensitivity: RefusalSensitivity,
}

impl Default for NoRefusalConfig {
    fn default() -> Self {
        Self {
            threshold: 0.7,
            sensitivity: RefusalSensitivity::Medium,
        }
    }
}

/// Refusal detection sensitivity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefusalSensitivity {
    Strict,  // Detects more refusals (may have false positives)
    Medium,  // Balanced detection
    Loose,   // Only obvious refusals
}

/// NoRefusal scanner implementation
///
/// ## Enterprise Features
///
/// - Detects various refusal patterns:
///   - Direct refusals ("I cannot help with that")
///   - Safety-based refusals ("This violates my guidelines")
///   - Capability-based refusals ("I'm not able to")
///   - Ethical refusals ("I shouldn't assist with")
/// - Configurable sensitivity
/// - Confidence scoring
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::output::NoRefusal;
///
/// let scanner = NoRefusal::default_config()?;
/// let response = "I'm sorry, I cannot assist with that request.";
/// let result = scanner.scan_output("original query", response, &vault).await?;
/// assert!(!result.is_valid); // Refusal detected
/// ```
pub struct NoRefusal {
    config: NoRefusalConfig,
}

impl NoRefusal {
    /// Create a new NoRefusal scanner
    pub fn new(config: NoRefusalConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(NoRefusalConfig::default())
    }

    /// Detect refusal patterns in response
    fn detect_refusal(&self, response: &str) -> (f32, Vec<RefusalMatch>) {
        let response_lower = response.to_lowercase();
        let mut matches = Vec::new();

        // Pattern set 1: Direct refusals (high confidence)
        let direct_refusals = [
            ("i cannot", 0.95),
            ("i can't", 0.95),
            ("i'm unable to", 0.95),
            ("i am unable to", 0.95),
            ("i won't", 0.90),
            ("i will not", 0.90),
            ("i cannot help", 0.95),
            ("i can't help", 0.95),
            ("i cannot assist", 0.95),
            ("i can't assist", 0.95),
            ("i'm not able to", 0.90),
            ("i am not able to", 0.90),
        ];

        for (pattern, confidence) in &direct_refusals {
            if response_lower.contains(pattern) {
                matches.push(RefusalMatch {
                    pattern: pattern.to_string(),
                    confidence: *confidence,
                    category: "direct_refusal".to_string(),
                });
            }
        }

        // Pattern set 2: Safety/policy refusals (high confidence)
        let safety_refusals = [
            ("violates", 0.85),
            ("against my guidelines", 0.90),
            ("against my programming", 0.90),
            ("against my policies", 0.90),
            ("not appropriate", 0.80),
            ("inappropriate", 0.75),
            ("not ethical", 0.85),
            ("unethical", 0.80),
            ("harmful", 0.75),
            ("dangerous", 0.75),
        ];

        for (pattern, confidence) in &safety_refusals {
            if response_lower.contains(pattern) {
                // Only flag if combined with refusal language
                if response_lower.contains("cannot") || response_lower.contains("can't")
                    || response_lower.contains("won't") || response_lower.contains("shouldn't")
                {
                    matches.push(RefusalMatch {
                        pattern: pattern.to_string(),
                        confidence: *confidence,
                        category: "safety_refusal".to_string(),
                    });
                }
            }
        }

        // Pattern set 3: Capability refusals (medium confidence)
        if self.config.sensitivity != RefusalSensitivity::Loose {
            let capability_refusals = [
                ("not capable of", 0.80),
                ("don't have the ability", 0.80),
                ("beyond my capabilities", 0.85),
                ("i'm just an ai", 0.70),
                ("i'm only an ai", 0.70),
                ("as an ai", 0.65), // Lower confidence - common phrasing
            ];

            for (pattern, confidence) in &capability_refusals {
                if response_lower.contains(pattern) {
                    matches.push(RefusalMatch {
                        pattern: pattern.to_string(),
                        confidence: *confidence,
                        category: "capability_refusal".to_string(),
                    });
                }
            }
        }

        // Pattern set 4: Apology-based refusals (lower confidence - needs context)
        if self.config.sensitivity == RefusalSensitivity::Strict {
            if (response_lower.contains("sorry") || response_lower.contains("apologize"))
                && (response_lower.contains("cannot") || response_lower.contains("can't")
                    || response_lower.contains("unable"))
            {
                matches.push(RefusalMatch {
                    pattern: "apology_refusal".to_string(),
                    confidence: 0.70,
                    category: "apology_refusal".to_string(),
                });
            }
        }

        // Calculate overall score
        let max_score = matches.iter().map(|m| m.confidence).fold(0.0f32, f32::max);

        (max_score, matches)
    }

    /// Scan output for refusals
    pub async fn scan_output(
        &self,
        _prompt: &str,
        output: &str,
        _vault: &Vault,
    ) -> Result<ScanResult> {
        let (score, matches) = self.detect_refusal(output);

        if score < self.config.threshold {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("refusal_score", score.to_string()));
        }

        // Build entities
        let entities: Vec<Entity> = matches
            .iter()
            .map(|m| {
                let mut metadata = HashMap::new();
                metadata.insert("pattern".to_string(), m.pattern.clone());
                metadata.insert("category".to_string(), m.category.clone());

                Entity {
                    entity_type: "refusal_detected".to_string(),
                    text: format!("[{}]", m.category),
                    start: 0,
                    end: output.len(),
                    confidence: m.confidence,
                    metadata,
                }
            })
            .collect();

        let severity = if score >= 0.9 {
            Severity::High
        } else if score >= 0.7 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let description = format!("LLM refused to answer (detected {} pattern(s))", matches.len());
        let risk_factor = RiskFactor::new(
            "llm_refusal",
            &description,
            severity,
            score,
        );

        let mut result = ScanResult::new(output.to_string(), false, score)
            .with_risk_factor(risk_factor)
            .with_metadata("refusal_score", score.to_string())
            .with_metadata("refusal_patterns", matches.len());

        for entity in entities {
            result = result.with_entity(entity);
        }

        Ok(result)
    }
}

#[derive(Debug, Clone)]
struct RefusalMatch {
    pattern: String,
    confidence: f32,
    category: String,
}

#[async_trait]
impl Scanner for NoRefusal {
    fn name(&self) -> &str {
        "NoRefusal"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        // For output scanners, we expect input to be the output to validate
        // In production, you'd pass both prompt and output
        self.scan_output("", input, vault).await
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Output
    }

    fn description(&self) -> &str {
        "Detects when an LLM refuses to answer legitimate requests"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_refusal_direct() {
        let scanner = NoRefusal::default_config().unwrap();
        let vault = Vault::new();

        let response = "I cannot help you with that request.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_score > 0.7);
    }

    #[tokio::test]
    async fn test_no_refusal_safety() {
        let scanner = NoRefusal::default_config().unwrap();
        let vault = Vault::new();

        let response = "I cannot assist as this violates my guidelines.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_no_refusal_capability() {
        let scanner = NoRefusal::default_config().unwrap();
        let vault = Vault::new();

        let response = "I'm just an AI and not capable of performing that action.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_no_refusal_normal_response() {
        let scanner = NoRefusal::default_config().unwrap();
        let vault = Vault::new();

        let response = "Here is the answer to your question: The capital of France is Paris.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        assert!(result.is_valid);
        assert!(result.risk_score < 0.7);
    }

    #[tokio::test]
    async fn test_no_refusal_apology_without_refusal() {
        let scanner = NoRefusal::default_config().unwrap();
        let vault = Vault::new();

        let response = "I'm sorry for the confusion. Let me clarify: the answer is 42.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        // Should pass - apology without refusal
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_no_refusal_sensitivity_strict() {
        let config = NoRefusalConfig {
            threshold: 0.6,
            sensitivity: RefusalSensitivity::Strict,
        };
        let scanner = NoRefusal::new(config).unwrap();
        let vault = Vault::new();

        let response = "I'm sorry, I cannot do that.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_no_refusal_sensitivity_loose() {
        let config = NoRefusalConfig {
            threshold: 0.7,
            sensitivity: RefusalSensitivity::Loose,
        };
        let scanner = NoRefusal::new(config).unwrap();
        let vault = Vault::new();

        let response = "As an AI, I can help you with that!";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        // Should pass with loose sensitivity
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_no_refusal_multiple_patterns() {
        let scanner = NoRefusal::default_config().unwrap();
        let vault = Vault::new();

        let response = "I cannot and will not help with that as it violates my guidelines.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.len() >= 2);
    }

    #[tokio::test]
    async fn test_no_refusal_threshold() {
        let config = NoRefusalConfig {
            threshold: 0.95,
            ..Default::default()
        };
        let scanner = NoRefusal::new(config).unwrap();
        let vault = Vault::new();

        // Weak refusal indicator
        let response = "This might not be appropriate.";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        // Should pass with high threshold
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_no_refusal_helpful_response() {
        let scanner = NoRefusal::default_config().unwrap();
        let vault = Vault::new();

        let response = "I'd be happy to help! Here's what you need to know...";
        let result = scanner.scan_output("test", response, &vault).await.unwrap();

        assert!(result.is_valid);
    }
}
