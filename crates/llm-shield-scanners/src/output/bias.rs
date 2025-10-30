//! Bias Output Scanner
//!
//! Converted from llm_guard/output_scanners/bias.py
//!
//! ## SPARC Implementation
//!
//! Detects biased language in LLM responses across multiple dimensions.
//! Essential for fair and inclusive AI systems.
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

/// Bias scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiasConfig {
    /// Bias types to detect
    pub bias_types: Vec<BiasType>,

    /// Detection threshold (0.0 to 1.0)
    pub threshold: f32,
}

impl Default for BiasConfig {
    fn default() -> Self {
        Self {
            bias_types: vec![
                BiasType::Gender,
                BiasType::Racial,
                BiasType::Age,
                BiasType::Religious,
            ],
            threshold: 0.6,
        }
    }
}

/// Types of bias to detect
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiasType {
    Gender,
    Racial,
    Age,
    Religious,
    Political,
    Socioeconomic,
    Disability,
}

impl BiasType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BiasType::Gender => "gender",
            BiasType::Racial => "racial",
            BiasType::Age => "age",
            BiasType::Religious => "religious",
            BiasType::Political => "political",
            BiasType::Socioeconomic => "socioeconomic",
            BiasType::Disability => "disability",
        }
    }
}

/// Bias scanner implementation
///
/// ## Enterprise Features
///
/// - Detects 7 types of bias:
///   - Gender bias (stereotypes, pronouns)
///   - Racial bias (stereotypes, slurs)
///   - Age bias (ageism)
///   - Religious bias
///   - Political bias
///   - Socioeconomic bias
///   - Disability bias
/// - Pattern-based detection with confidence scoring
/// - Configurable bias types
/// - ML-ready architecture (can add transformer models)
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::output::Bias;
///
/// let scanner = Bias::default_config()?;
/// let response = "Women are not good at mathematics";
/// let result = scanner.scan_output("", response, &vault).await?;
/// assert!(!result.is_valid); // Gender bias detected
/// ```
pub struct Bias {
    config: BiasConfig,
}

impl Bias {
    /// Create a new Bias scanner
    pub fn new(config: BiasConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        if config.bias_types.is_empty() {
            return Err(Error::config("At least one bias type must be enabled"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(BiasConfig::default())
    }

    /// Detect bias in text
    fn detect_bias(&self, text: &str) -> Vec<BiasMatch> {
        let mut matches = Vec::new();
        let text_lower = text.to_lowercase();

        for bias_type in &self.config.bias_types {
            let bias_matches = match bias_type {
                BiasType::Gender => self.detect_gender_bias(&text_lower),
                BiasType::Racial => self.detect_racial_bias(&text_lower),
                BiasType::Age => self.detect_age_bias(&text_lower),
                BiasType::Religious => self.detect_religious_bias(&text_lower),
                BiasType::Political => self.detect_political_bias(&text_lower),
                BiasType::Socioeconomic => self.detect_socioeconomic_bias(&text_lower),
                BiasType::Disability => self.detect_disability_bias(&text_lower),
            };

            matches.extend(bias_matches);
        }

        matches
    }

    fn detect_gender_bias(&self, text: &str) -> Vec<BiasMatch> {
        let patterns = [
            // Stereotypes
            ("women are not good at", 0.9),
            ("men are better at", 0.9),
            ("girls can't", 0.85),
            ("boys don't", 0.80),
            ("typical woman", 0.75),
            ("like a girl", 0.75),
            ("man up", 0.80),
            ("women belong in", 0.85),
            ("women should stay", 0.85),
            ("men are naturally", 0.75),
            ("women are too emotional", 0.90),
            ("men don't cry", 0.80),
            ("acting like a woman", 0.75),
            ("feminine weakness", 0.85),
        ];

        self.find_pattern_matches(text, &patterns, BiasType::Gender)
    }

    fn detect_racial_bias(&self, text: &str) -> Vec<BiasMatch> {
        let patterns = [
            // Stereotypes (educational examples only)
            ("all [race] are", 0.90),
            ("typical [race]", 0.80),
            ("[race] people are naturally", 0.85),
            ("those [race]", 0.75),
            ("you people", 0.70),
            ("one of the good ones", 0.80),
            ("articulate for a", 0.85),
            ("credit to your race", 0.90),
            ("racial superiority", 0.95),
            ("inferior race", 0.95),
        ];

        self.find_pattern_matches(text, &patterns, BiasType::Racial)
    }

    fn detect_age_bias(&self, text: &str) -> Vec<BiasMatch> {
        let patterns = [
            ("too old to", 0.85),
            ("too young to", 0.80),
            ("past your prime", 0.85),
            ("over the hill", 0.80),
            ("youngsters don't understand", 0.75),
            ("old people can't", 0.85),
            ("millennials are lazy", 0.85),
            ("boomers are", 0.70),
            ("generation [x] is", 0.65),
            ("kids these days", 0.70),
        ];

        self.find_pattern_matches(text, &patterns, BiasType::Age)
    }

    fn detect_religious_bias(&self, text: &str) -> Vec<BiasMatch> {
        let patterns = [
            ("all [religion] are", 0.85),
            ("[religion] people are", 0.80),
            ("religious fanatics", 0.75),
            ("godless", 0.70),
            ("heathen", 0.75),
            ("infidel", 0.80),
            ("backward religion", 0.85),
            ("primitive beliefs", 0.80),
        ];

        self.find_pattern_matches(text, &patterns, BiasType::Religious)
    }

    fn detect_political_bias(&self, text: &str) -> Vec<BiasMatch> {
        let patterns = [
            ("all liberals are", 0.80),
            ("all conservatives are", 0.80),
            ("typical leftist", 0.75),
            ("typical right-winger", 0.75),
            ("libs are", 0.70),
            ("republicans are all", 0.80),
            ("democrats are all", 0.80),
            ("political extremists", 0.70),
        ];

        self.find_pattern_matches(text, &patterns, BiasType::Political)
    }

    fn detect_socioeconomic_bias(&self, text: &str) -> Vec<BiasMatch> {
        let patterns = [
            ("poor people are lazy", 0.90),
            ("welfare queens", 0.90),
            ("rich people deserve", 0.75),
            ("poverty is a choice", 0.85),
            ("poor people deserve", 0.80),
            ("trailer trash", 0.90),
            ("ghetto", 0.70),
            ("low class", 0.75),
        ];

        self.find_pattern_matches(text, &patterns, BiasType::Socioeconomic)
    }

    fn detect_disability_bias(&self, text: &str) -> Vec<BiasMatch> {
        let patterns = [
            ("disabled people can't", 0.85),
            ("handicapped", 0.70), // Often outdated term
            ("cripple", 0.90),
            ("retarded", 0.95),
            ("mentally deficient", 0.90),
            ("wheelchair bound", 0.75),
            ("suffers from", 0.65), // Context-dependent
            ("afflicted with", 0.70),
        ];

        self.find_pattern_matches(text, &patterns, BiasType::Disability)
    }

    fn find_pattern_matches(
        &self,
        text: &str,
        patterns: &[(&str, f32)],
        bias_type: BiasType,
    ) -> Vec<BiasMatch> {
        patterns
            .iter()
            .filter_map(|(pattern, confidence)| {
                if text.contains(pattern) {
                    Some(BiasMatch {
                        bias_type,
                        pattern: pattern.to_string(),
                        confidence: *confidence,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Scan output for bias
    pub async fn scan_output(
        &self,
        _prompt: &str,
        output: &str,
        _vault: &Vault,
    ) -> Result<ScanResult> {
        let matches = self.detect_bias(output);

        if matches.is_empty() {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("bias_detected", "false"));
        }

        // Filter by threshold
        let significant_matches: Vec<_> = matches
            .into_iter()
            .filter(|m| m.confidence >= self.config.threshold)
            .collect();

        if significant_matches.is_empty() {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("bias_detected", "false")
                .with_metadata("below_threshold", "true"));
        }

        // Build entities
        let entities: Vec<Entity> = significant_matches
            .iter()
            .map(|m| {
                let mut metadata = HashMap::new();
                metadata.insert("bias_type".to_string(), m.bias_type.as_str().to_string());
                metadata.insert("pattern".to_string(), m.pattern.clone());

                Entity {
                    entity_type: format!("{}_bias", m.bias_type.as_str()),
                    text: m.pattern.clone(),
                    start: 0,
                    end: output.len(),
                    confidence: m.confidence,
                    metadata,
                }
            })
            .collect();

        let max_confidence = significant_matches
            .iter()
            .map(|m| m.confidence)
            .fold(0.0f32, f32::max);

        let severity = if max_confidence >= 0.85 {
            Severity::High
        } else if max_confidence >= 0.7 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let risk_factor = RiskFactor::new(
            "biased_language",
            format!(
                "LLM response contains biased language ({} instance(s))",
                significant_matches.len()
            ),
            severity,
            max_confidence,
        );

        Ok(ScanResult::new(output.to_string(), false, max_confidence)
            .with_entities(entities)
            .with_risk_factor(risk_factor)
            .with_metadata("bias_detected", "true")
            .with_metadata("bias_count", significant_matches.len()))
    }
}

#[derive(Debug, Clone)]
struct BiasMatch {
    bias_type: BiasType,
    pattern: String,
    confidence: f32,
}

#[async_trait]
impl Scanner for Bias {
    fn name(&self) -> &str {
        "Bias"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        self.scan_output("", input, vault).await
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Output
    }

    fn description(&self) -> &str {
        "Detects biased language in LLM responses across multiple dimensions"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bias_gender() {
        let scanner = Bias::default_config().unwrap();
        let vault = Vault::new();

        let response = "Women are not good at mathematics and engineering.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.entity_type.contains("gender")));
    }

    #[tokio::test]
    async fn test_bias_age() {
        let scanner = Bias::default_config().unwrap();
        let vault = Vault::new();

        let response = "You're too old to learn new technology.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.entity_type.contains("age")));
    }

    #[tokio::test]
    async fn test_bias_religious() {
        let scanner = Bias::default_config().unwrap();
        let vault = Vault::new();

        let response = "All Muslims are religious fanatics.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.entity_type.contains("religious")));
    }

    #[tokio::test]
    async fn test_bias_clean_response() {
        let scanner = Bias::default_config().unwrap();
        let vault = Vault::new();

        let response = "People of all backgrounds contribute valuable perspectives.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.entities.len(), 0);
    }

    #[tokio::test]
    async fn test_bias_political() {
        let config = BiasConfig {
            bias_types: vec![BiasType::Political],
            threshold: 0.6,
        };
        let scanner = Bias::new(config).unwrap();
        let vault = Vault::new();

        let response = "All liberals are destroying this country.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.entity_type.contains("political")));
    }

    #[tokio::test]
    async fn test_bias_socioeconomic() {
        let config = BiasConfig {
            bias_types: vec![BiasType::Socioeconomic],
            threshold: 0.6,
        };
        let scanner = Bias::new(config).unwrap();
        let vault = Vault::new();

        let response = "Poor people are lazy and don't deserve help.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.entity_type.contains("socioeconomic")));
    }

    #[tokio::test]
    async fn test_bias_disability() {
        let config = BiasConfig {
            bias_types: vec![BiasType::Disability],
            threshold: 0.6,
        };
        let scanner = Bias::new(config).unwrap();
        let vault = Vault::new();

        let response = "Disabled people can't work effectively.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.entity_type.contains("disability")));
    }

    #[tokio::test]
    async fn test_bias_multiple_types() {
        let scanner = Bias::default_config().unwrap();
        let vault = Vault::new();

        let response = "Women are too emotional and old people can't learn.";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.len() >= 2);
    }

    #[tokio::test]
    async fn test_bias_threshold() {
        let config = BiasConfig {
            bias_types: vec![BiasType::Gender],
            threshold: 0.95, // Very high threshold
        };
        let scanner = Bias::new(config).unwrap();
        let vault = Vault::new();

        let response = "Like a girl"; // Confidence 0.75
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        // Should pass due to high threshold
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_bias_severity_high() {
        let scanner = Bias::default_config().unwrap();
        let vault = Vault::new();

        let response = "Women are not good at technical work."; // High confidence pattern
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_factors.iter().any(|r| matches!(r.severity, Severity::High | Severity::Medium)));
    }

    #[tokio::test]
    async fn test_bias_case_insensitive() {
        let scanner = Bias::default_config().unwrap();
        let vault = Vault::new();

        let response = "WOMEN ARE NOT GOOD AT mathematics";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid); // Should detect uppercase
    }

    #[tokio::test]
    async fn test_bias_selective_types() {
        let config = BiasConfig {
            bias_types: vec![BiasType::Gender],
            threshold: 0.6,
        };
        let scanner = Bias::new(config).unwrap();
        let vault = Vault::new();

        let response = "You're too old to learn"; // Age bias
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        // Should pass because age detection is disabled
        assert!(result.is_valid);
    }
}
