//! BanSubstrings Scanner
//!
//! Converted from llm_guard/input_scanners/ban_substrings.py
//!
//! ## SPARC Implementation
//!
//! This scanner detects and blocks specific substrings in input text.
//!
//! ## London School TDD
//!
//! Tests are written first, driving the implementation.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BanSubstrings scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanSubstringsConfig {
    /// Substrings to ban
    pub substrings: Vec<String>,

    /// Whether matching is case-sensitive
    pub case_sensitive: bool,

    /// Match type (word boundary or contains)
    pub match_type: MatchType,

    /// Whether to redact matches
    pub redact: bool,
}

/// Match type for substring detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Match anywhere in text
    Contains,
    /// Match on word boundaries only
    Word,
}

impl Default for BanSubstringsConfig {
    fn default() -> Self {
        Self {
            substrings: Vec::new(),
            case_sensitive: false,
            match_type: MatchType::Contains,
            redact: false,
        }
    }
}

/// BanSubstrings scanner implementation
///
/// ## Enterprise Features
///
/// - Fast multi-pattern matching with Aho-Corasick algorithm
/// - Case-insensitive matching option
/// - Word boundary detection
/// - Optional redaction
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::input::BanSubstrings;
///
/// let config = BanSubstringsConfig {
///     substrings: vec!["badword".to_string(), "offensive".to_string()],
///     case_sensitive: false,
///     match_type: MatchType::Contains,
///     redact: true,
/// };
///
/// let scanner = BanSubstrings::new(config);
/// let result = scanner.scan("This has badword", &vault).await?;
/// assert!(!result.is_valid);
/// ```
pub struct BanSubstrings {
    config: BanSubstringsConfig,
    matcher: AhoCorasick,
    patterns: Vec<String>,
}

impl BanSubstrings {
    /// Create a new BanSubstrings scanner
    pub fn new(config: BanSubstringsConfig) -> Result<Self> {
        if config.substrings.is_empty() {
            return Err(Error::config("At least one substring must be provided"));
        }

        // Prepare patterns for matching
        let patterns: Vec<String> = if config.case_sensitive {
            config.substrings.clone()
        } else {
            config.substrings.iter().map(|s| s.to_lowercase()).collect()
        };

        // Build Aho-Corasick automaton for fast multi-pattern matching
        let matcher = AhoCorasick::new(&patterns)
            .map_err(|e| Error::config(format!("Failed to build pattern matcher: {}", e)))?;

        Ok(Self {
            config,
            matcher,
            patterns,
        })
    }

    /// Create with simple substring list
    pub fn with_substrings<I, S>(substrings: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let config = BanSubstringsConfig {
            substrings: substrings.into_iter().map(|s| s.into()).collect(),
            ..Default::default()
        };
        Self::new(config)
    }

    fn find_matches(&self, text: &str) -> Vec<(usize, usize, &str)> {
        let search_text = if self.config.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        let mut matches = Vec::new();

        for mat in self.matcher.find_iter(&search_text) {
            let pattern = &self.patterns[mat.pattern().as_usize()];
            let start = mat.start();
            let end = mat.end();

            // For word boundary matching, check if surrounded by word boundaries
            if self.config.match_type == MatchType::Word {
                let before_is_boundary = start == 0
                    || !text.chars().nth(start - 1).map_or(false, |c| c.is_alphanumeric());

                let after_is_boundary = end >= text.len()
                    || !text.chars().nth(end).map_or(false, |c| c.is_alphanumeric());

                if !before_is_boundary || !after_is_boundary {
                    continue;
                }
            }

            matches.push((start, end, pattern.as_str()));
        }

        matches
    }

    fn redact_text(&self, text: &str, matches: &[(usize, usize, &str)]) -> String {
        if !self.config.redact || matches.is_empty() {
            return text.to_string();
        }

        let mut result = text.to_string();
        let mut offset = 0i32;

        for (start, end, _pattern) in matches {
            let redaction = "*".repeat(end - start);
            let actual_start = (*start as i32 + offset) as usize;
            let actual_end = (*end as i32 + offset) as usize;

            result.replace_range(actual_start..actual_end, &redaction);
        }

        result
    }
}

#[async_trait]
impl Scanner for BanSubstrings {
    fn name(&self) -> &str {
        "BanSubstrings"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let matches = self.find_matches(input);

        if matches.is_empty() {
            return Ok(ScanResult::pass(input.to_string()));
        }

        // Build entities for each match
        let entities: Vec<Entity> = matches
            .iter()
            .map(|(start, end, pattern)| {
                let mut metadata = HashMap::new();
                metadata.insert("pattern".to_string(), pattern.to_string());

                Entity {
                    entity_type: "banned_substring".to_string(),
                    text: input[*start..*end].to_string(),
                    start: *start,
                    end: *end,
                    confidence: 1.0,
                    metadata,
                }
            })
            .collect();

        let risk_factor = RiskFactor::new(
            "banned_content",
            format!("Found {} banned substring(s)", matches.len()),
            Severity::High,
            1.0,
        );

        let sanitized_text = self.redact_text(input, &matches);

        Ok(ScanResult::new(sanitized_text, false, 1.0)
            .with_risk_factor(risk_factor)
            .with_metadata("matches_count", entities.len())
            .with_metadata("patterns_matched", matches.iter().map(|(_, _, p)| p).collect::<Vec<_>>()))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Detects and blocks banned substrings in input text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ban_substrings_exact_match() {
        let scanner = BanSubstrings::with_substrings(vec!["badword"]).unwrap();
        let vault = Vault::new();

        let result = scanner.scan("This contains badword", &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.risk_score, 1.0);
        assert_eq!(result.entities.len(), 1);
    }

    #[tokio::test]
    async fn test_ban_substrings_case_insensitive() {
        let scanner = BanSubstrings::with_substrings(vec!["BADWORD"]).unwrap();
        let vault = Vault::new();

        let result = scanner.scan("This contains badword", &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_ban_substrings_no_match() {
        let scanner = BanSubstrings::with_substrings(vec!["badword"]).unwrap();
        let vault = Vault::new();

        let result = scanner.scan("This is clean text", &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
    }

    #[tokio::test]
    async fn test_ban_substrings_multiple_matches() {
        let scanner = BanSubstrings::with_substrings(vec!["bad", "worse"]).unwrap();
        let vault = Vault::new();

        let result = scanner
            .scan("This is bad and even worse", &vault)
            .await
            .unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 2);
    }

    #[tokio::test]
    async fn test_ban_substrings_redaction() {
        let config = BanSubstringsConfig {
            substrings: vec!["secret".to_string()],
            case_sensitive: false,
            match_type: MatchType::Contains,
            redact: true,
        };

        let scanner = BanSubstrings::new(config).unwrap();
        let vault = Vault::new();

        let result = scanner.scan("The secret word is here", &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.sanitized_text.contains("******"));
    }

    #[tokio::test]
    async fn test_ban_substrings_word_boundary() {
        let config = BanSubstringsConfig {
            substrings: vec!["test".to_string()],
            case_sensitive: false,
            match_type: MatchType::Word,
            redact: false,
        };

        let scanner = BanSubstrings::new(config).unwrap();
        let vault = Vault::new();

        // Should match "test" as a word
        let result1 = scanner.scan("This is a test", &vault).await.unwrap();
        assert!(!result1.is_valid);

        // Should NOT match "test" within "testing"
        let result2 = scanner.scan("We are testing", &vault).await.unwrap();
        assert!(result2.is_valid);
    }
}
