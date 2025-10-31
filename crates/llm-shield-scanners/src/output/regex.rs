//! RegexOutput Scanner
//!
//! Converted from llm_guard/output_scanners/regex.py
//!
//! ## SPARC Implementation
//!
//! Custom pattern matching for LLM outputs using regular expressions.
//! Flexible scanner for organization-specific validation rules.
//!
//! ## London School TDD
//!
//! Tests written first drive the implementation.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RegexOutput scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexOutputConfig {
    /// List of patterns to match
    pub patterns: Vec<RegexPattern>,

    /// Match mode: AllowList or DenyList
    pub match_mode: MatchMode,
}

impl Default for RegexOutputConfig {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            match_mode: MatchMode::DenyList,
        }
    }
}

/// Pattern definition with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPattern {
    /// Pattern name/description
    pub name: String,

    /// Regular expression pattern
    pub pattern: String,

    /// Severity if pattern matches (for deny list)
    pub severity: Severity,

    /// Case-insensitive matching
    #[serde(default)]
    pub case_insensitive: bool,
}

/// Match mode for regex scanner
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchMode {
    /// Patterns define forbidden content (match = fail)
    DenyList,

    /// Patterns define allowed content (no match = fail)
    AllowList,
}

/// RegexOutput scanner implementation
///
/// ## Enterprise Features
///
/// - Custom regex pattern matching
/// - AllowList and DenyList modes
/// - Multiple patterns with individual severity
/// - Case-sensitive/insensitive matching
/// - Detailed match metadata
/// - Organization-specific validation rules
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::output::RegexOutput;
///
/// let config = RegexOutputConfig {
///     patterns: vec![
///         RegexPattern {
///             name: "phone_numbers".to_string(),
///             pattern: r"\d{3}-\d{3}-\d{4}".to_string(),
///             severity: Severity::High,
///             case_insensitive: false,
///         },
///     ],
///     match_mode: MatchMode::DenyList,
/// };
/// let scanner = RegexOutput::new(config)?;
///
/// let response = "Call me at 555-123-4567";
/// let result = scanner.scan_output("", response, &vault).await?;
/// assert!(!result.is_valid); // Phone number detected
/// ```
pub struct RegexOutput {
    config: RegexOutputConfig,
    compiled_patterns: Vec<CompiledPattern>,
}

impl RegexOutput {
    /// Create a new RegexOutput scanner
    pub fn new(config: RegexOutputConfig) -> Result<Self> {
        if config.patterns.is_empty() {
            return Err(Error::config("At least one pattern must be configured"));
        }

        // Compile all patterns
        let mut compiled_patterns = Vec::new();
        for pattern_config in &config.patterns {
            let regex = if pattern_config.case_insensitive {
                Regex::new(&format!("(?i){}", pattern_config.pattern))
            } else {
                Regex::new(&pattern_config.pattern)
            }
            .map_err(|e| {
                Error::config(format!(
                    "Invalid regex pattern '{}': {}",
                    pattern_config.name, e
                ))
            })?;

            compiled_patterns.push(CompiledPattern {
                name: pattern_config.name.clone(),
                regex,
                severity: pattern_config.severity,
            });
        }

        Ok(Self {
            config,
            compiled_patterns,
        })
    }

    /// Create with default configuration (empty - requires patterns)
    pub fn with_patterns(patterns: Vec<RegexPattern>) -> Result<Self> {
        let config = RegexOutputConfig {
            patterns,
            match_mode: MatchMode::DenyList,
        };
        Self::new(config)
    }

    /// Find all pattern matches in text
    fn find_matches(&self, text: &str) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for compiled in &self.compiled_patterns {
            for mat in compiled.regex.find_iter(text) {
                matches.push(PatternMatch {
                    pattern_name: compiled.name.clone(),
                    matched_text: mat.as_str().to_string(),
                    start: mat.start(),
                    end: mat.end(),
                    severity: compiled.severity,
                });
            }
        }

        matches
    }

    /// Scan output for regex patterns
    pub async fn scan_output(
        &self,
        _prompt: &str,
        output: &str,
        _vault: &Vault,
    ) -> Result<ScanResult> {
        let matches = self.find_matches(output);

        match self.config.match_mode {
            MatchMode::DenyList => {
                // In deny list mode, any match = fail
                if matches.is_empty() {
                    return Ok(ScanResult::pass(output.to_string())
                        .with_metadata("matches_found", "0")
                        .with_metadata("match_mode", "denylist"));
                }

                // Build entities for matches
                let entities: Vec<Entity> = matches
                    .iter()
                    .map(|m| {
                        let mut metadata = HashMap::new();
                        metadata.insert("pattern_name".to_string(), m.pattern_name.clone());
                        metadata.insert("matched_text".to_string(), m.matched_text.clone());
                        metadata.insert("start".to_string(), m.start.to_string());
                        metadata.insert("end".to_string(), m.end.to_string());

                        Entity {
                            entity_type: "regex_match".to_string(),
                            text: m.matched_text.clone(),
                            start: m.start,
                            end: m.end,
                            confidence: 1.0,
                            metadata,
                        }
                    })
                    .collect();

                let max_severity = matches
                    .iter()
                    .map(|m| &m.severity)
                    .max()
                    .unwrap_or(&Severity::Low);

                let description = format!(
                    "Output matches {} forbidden pattern(s)",
                    matches.len()
                );
                let risk_factor = RiskFactor::new(
                    "regex_pattern_match",
                    &description,
                    *max_severity,
                    0.9,
                );

                let mut result = ScanResult::new(output.to_string(), false, 0.9)
                    .with_risk_factor(risk_factor)
                    .with_metadata("matches_found", matches.len())
                    .with_metadata("match_mode", "denylist");

                for entity in entities {
                    result = result.with_entity(entity);
                }

                Ok(result)
            }
            MatchMode::AllowList => {
                // In allow list mode, no match = fail
                if !matches.is_empty() {
                    return Ok(ScanResult::pass(output.to_string())
                        .with_metadata("matches_found", matches.len().to_string())
                        .with_metadata("match_mode", "allowlist"));
                }

                // No matches in allow list mode = fail
                let risk_factor = RiskFactor::new(
                    "regex_no_match",
                    "Output does not match any allowed pattern",
                    Severity::Medium,
                    0.8,
                );

                let result = ScanResult::new(output.to_string(), false, 0.8)
                    .with_risk_factor(risk_factor)
                    .with_metadata("matches_found", 0)
                    .with_metadata("match_mode", "allowlist");

                Ok(result)
            }
        }
    }
}

#[derive(Debug)]
struct CompiledPattern {
    name: String,
    regex: Regex,
    severity: Severity,
}

#[derive(Debug, Clone)]
struct PatternMatch {
    pattern_name: String,
    matched_text: String,
    start: usize,
    end: usize,
    severity: Severity,
}

#[async_trait]
impl Scanner for RegexOutput {
    fn name(&self) -> &str {
        "RegexOutput"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        self.scan_output("", input, vault).await
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Output
    }

    fn description(&self) -> &str {
        "Custom pattern matching for LLM outputs using regular expressions"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_regex_output_denylist_match() {
        let patterns = vec![RegexPattern {
            name: "phone_number".to_string(),
            pattern: r"\d{3}-\d{3}-\d{4}".to_string(),
            severity: Severity::High,
            case_insensitive: false,
        }];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "Call me at 555-123-4567";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].metadata.get("pattern_name").unwrap(), "phone_number");
    }

    #[tokio::test]
    async fn test_regex_output_denylist_no_match() {
        let patterns = vec![RegexPattern {
            name: "email".to_string(),
            pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
            severity: Severity::Medium,
            case_insensitive: false,
        }];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "This has no email addresses";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_regex_output_allowlist_match() {
        let config = RegexOutputConfig {
            patterns: vec![RegexPattern {
                name: "greeting".to_string(),
                pattern: r"^(Hello|Hi|Hey)".to_string(),
                severity: Severity::Low,
                case_insensitive: false,
            }],
            match_mode: MatchMode::AllowList,
        };
        let scanner = RegexOutput::new(config).unwrap();
        let vault = Vault::new();

        let response = "Hello, how can I help you?";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_regex_output_allowlist_no_match() {
        let config = RegexOutputConfig {
            patterns: vec![RegexPattern {
                name: "greeting".to_string(),
                pattern: r"^(Hello|Hi|Hey)".to_string(),
                severity: Severity::Low,
                case_insensitive: false,
            }],
            match_mode: MatchMode::AllowList,
        };
        let scanner = RegexOutput::new(config).unwrap();
        let vault = Vault::new();

        let response = "Welcome to our service";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_regex_output_case_insensitive() {
        let patterns = vec![RegexPattern {
            name: "banned_word".to_string(),
            pattern: "forbidden".to_string(),
            severity: Severity::High,
            case_insensitive: true,
        }];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "This is FORBIDDEN content";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_regex_output_case_sensitive() {
        let patterns = vec![RegexPattern {
            name: "banned_word".to_string(),
            pattern: "forbidden".to_string(),
            severity: Severity::High,
            case_insensitive: false,
        }];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "This is FORBIDDEN content";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        // Should pass - case mismatch
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_regex_output_multiple_patterns() {
        let patterns = vec![
            RegexPattern {
                name: "email".to_string(),
                pattern: r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(),
                severity: Severity::High,
                case_insensitive: false,
            },
            RegexPattern {
                name: "phone".to_string(),
                pattern: r"\d{3}-\d{3}-\d{4}".to_string(),
                severity: Severity::Medium,
                case_insensitive: false,
            },
        ];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "Contact: test@example.com or 555-123-4567";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 2);
    }

    #[tokio::test]
    async fn test_regex_output_multiple_matches_same_pattern() {
        let patterns = vec![RegexPattern {
            name: "number".to_string(),
            pattern: r"\d+".to_string(),
            severity: Severity::Low,
            case_insensitive: false,
        }];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "Numbers: 123 456 789";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 3);
    }

    #[tokio::test]
    async fn test_regex_output_severity_levels() {
        let patterns = vec![
            RegexPattern {
                name: "critical".to_string(),
                pattern: "SECRET".to_string(),
                severity: Severity::High,
                case_insensitive: false,
            },
            RegexPattern {
                name: "warning".to_string(),
                pattern: "WARN".to_string(),
                severity: Severity::Low,
                case_insensitive: false,
            },
        ];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "SECRET information detected";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_factors.iter().any(|r| matches!(r.severity, Severity::High)));
    }

    #[tokio::test]
    async fn test_regex_output_invalid_pattern() {
        let patterns = vec![RegexPattern {
            name: "invalid".to_string(),
            pattern: "[invalid(regex".to_string(), // Invalid regex
            severity: Severity::Low,
            case_insensitive: false,
        }];
        let result = RegexOutput::with_patterns(patterns);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_regex_output_empty_patterns() {
        let config = RegexOutputConfig {
            patterns: Vec::new(),
            match_mode: MatchMode::DenyList,
        };
        let result = RegexOutput::new(config);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_regex_output_match_positions() {
        let patterns = vec![RegexPattern {
            name: "word".to_string(),
            pattern: "test".to_string(),
            severity: Severity::Low,
            case_insensitive: false,
        }];
        let scanner = RegexOutput::with_patterns(patterns).unwrap();
        let vault = Vault::new();

        let response = "This is a test sentence";
        let result = scanner.scan_output("", response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities[0].start, 10);
        assert_eq!(result.entities[0].end, 14);
    }
}
