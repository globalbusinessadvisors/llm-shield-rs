//! Regex Scanner
//!
//! Converted from llm_guard/input_scanners/regex.py
//!
//! Detects patterns using regular expressions.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Regex scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexConfig {
    /// Patterns to match
    pub patterns: Vec<RegexPattern>,

    /// Whether to redact matches
    pub redact: bool,
}

/// A regex pattern with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    /// Pattern name
    pub name: String,

    /// Regex pattern
    pub pattern: String,

    /// Risk score (0.0-1.0)
    pub risk_score: f32,
}

impl Default for RegexConfig {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            redact: false,
        }
    }
}

/// Regex scanner implementation
pub struct RegexScanner {
    config: RegexConfig,
    compiled_patterns: Vec<(String, Regex, f32)>,
}

impl RegexScanner {
    /// Create a new RegexScanner
    pub fn new(config: RegexConfig) -> Result<Self> {
        if config.patterns.is_empty() {
            return Err(Error::config("At least one pattern must be provided"));
        }

        let mut compiled_patterns = Vec::new();

        for pattern in &config.patterns {
            let regex = Regex::new(&pattern.pattern).map_err(|e| {
                Error::config(format!("Invalid regex '{}': {}", pattern.pattern, e))
            })?;

            compiled_patterns.push((pattern.name.clone(), regex, pattern.risk_score));
        }

        Ok(Self {
            config,
            compiled_patterns,
        })
    }

    /// Create with simple patterns (all with risk_score 1.0)
    pub fn with_patterns<I, S>(patterns: I) -> Result<Self>
    where
        I: IntoIterator<Item = (S, S)>,
        S: Into<String>,
    {
        let regex_patterns: Vec<RegexPattern> = patterns
            .into_iter()
            .map(|(name, pattern)| RegexPattern {
                name: name.into(),
                pattern: pattern.into(),
                risk_score: 1.0,
            })
            .collect();

        Self::new(RegexConfig {
            patterns: regex_patterns,
            redact: false,
        })
    }
}

#[async_trait]
impl Scanner for RegexScanner {
    fn name(&self) -> &str {
        "Regex"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let mut entities = Vec::new();
        let mut max_risk = 0.0f32;
        let mut sanitized_text = input.to_string();

        for (name, regex, risk_score) in &self.compiled_patterns {
            for mat in regex.find_iter(input) {
                let mut metadata = HashMap::new();
                metadata.insert("pattern_name".to_string(), name.clone());

                let entity = Entity {
                    entity_type: "regex_match".to_string(),
                    text: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    confidence: 1.0,
                    metadata,
                };

                entities.push(entity);
                max_risk = max_risk.max(*risk_score);

                // Redact if configured
                if self.config.redact {
                    let redaction = "*".repeat(mat.end() - mat.start());
                    sanitized_text.replace_range(mat.start()..mat.end(), &redaction);
                }
            }
        }

        if entities.is_empty() {
            return Ok(ScanResult::pass(input.to_string()));
        }

        let risk_factor = RiskFactor::new(
            "regex_match",
            format!("Found {} regex match(es)", entities.len()),
            if max_risk >= 0.7 {
                Severity::High
            } else {
                Severity::Medium
            },
            max_risk,
        );

        let mut result = ScanResult::new(sanitized_text, false, max_risk)
            .with_risk_factor(risk_factor);

        for entity in entities {
            result = result.with_entity(entity);
        }

        Ok(result)
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Detects patterns using regular expressions"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_email_detection() {
        let scanner = RegexScanner::with_patterns(vec![(
            "email",
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
        )])
        .unwrap();

        let vault = Vault::new();

        let result = scanner
            .scan("Contact me at test@example.com", &vault)
            .await
            .unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].text, "test@example.com");
    }

    #[tokio::test]
    async fn test_regex_no_match() {
        let scanner = RegexScanner::with_patterns(vec![("email", r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")]).unwrap();
        let vault = Vault::new();

        let result = scanner.scan("No emails here", &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.entities.len(), 0);
    }
}
