//! BanCompetitors Scanner
//!
//! Converted from llm_guard/input_scanners/ban_competitors.py
//!
//! ## SPARC Implementation
//!
//! This scanner detects and blocks mentions of competitor brands, companies, or products.
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

/// BanCompetitors scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanCompetitorsConfig {
    /// List of competitor names/brands to block
    pub competitors: Vec<String>,

    /// Whether matching is case-sensitive
    pub case_sensitive: bool,

    /// Whether to redact competitor names
    pub redact: bool,

    /// Whether to match whole words only
    pub whole_words_only: bool,
}

impl Default for BanCompetitorsConfig {
    fn default() -> Self {
        Self {
            competitors: Vec::new(),
            case_sensitive: false,
            redact: true,
            whole_words_only: true,
        }
    }
}

/// BanCompetitors scanner implementation
///
/// ## Enterprise Features
///
/// - Fast multi-pattern matching with Aho-Corasick
/// - Case-insensitive matching
/// - Whole word matching to avoid false positives
/// - Optional redaction
/// - Configurable competitor lists
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::input::BanCompetitors;
///
/// let config = BanCompetitorsConfig {
///     competitors: vec!["CompetitorA".to_string(), "CompetitorB".to_string()],
///     case_sensitive: false,
///     redact: true,
///     whole_words_only: true,
/// };
///
/// let scanner = BanCompetitors::new(config)?;
/// let result = scanner.scan("Check out CompetitorA's new product", &vault).await?;
/// assert!(!result.is_valid);
/// ```
pub struct BanCompetitors {
    config: BanCompetitorsConfig,
    matcher: AhoCorasick,
    patterns: Vec<String>,
}

impl BanCompetitors {
    /// Create a new BanCompetitors scanner
    pub fn new(config: BanCompetitorsConfig) -> Result<Self> {
        if config.competitors.is_empty() {
            return Err(Error::config("At least one competitor must be provided"));
        }

        // Prepare patterns for matching
        let patterns: Vec<String> = if config.case_sensitive {
            config.competitors.clone()
        } else {
            config.competitors.iter().map(|s| s.to_lowercase()).collect()
        };

        // Build Aho-Corasick automaton
        let matcher = AhoCorasick::new(&patterns)
            .map_err(|e| Error::config(format!("Failed to build pattern matcher: {}", e)))?;

        Ok(Self {
            config,
            matcher,
            patterns,
        })
    }

    /// Create with simple competitor list
    pub fn with_competitors<I, S>(competitors: I) -> Result<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let config = BanCompetitorsConfig {
            competitors: competitors.into_iter().map(|s| s.into()).collect(),
            ..Default::default()
        };
        Self::new(config)
    }

    fn find_matches(&self, text: &str) -> Vec<(usize, usize, String)> {
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

            // For whole word matching, check boundaries
            if self.config.whole_words_only {
                let before_is_boundary = start == 0
                    || !text.chars().nth(start - 1).map_or(false, |c| c.is_alphanumeric());

                let after_is_boundary = end >= text.len()
                    || !text.chars().nth(end).map_or(false, |c| c.is_alphanumeric());

                if !before_is_boundary || !after_is_boundary {
                    continue;
                }
            }

            // Get the actual text (preserving original case)
            let matched_text = &text[start..end];
            matches.push((start, end, matched_text.to_string()));
        }

        matches
    }

    fn redact_text(&self, text: &str, matches: &[(usize, usize, String)]) -> String {
        if !self.config.redact || matches.is_empty() {
            return text.to_string();
        }

        let mut result = text.to_string();
        let mut offset = 0i32;

        for (start, end, _matched_text) in matches {
            let redaction = "[COMPETITOR]";
            let actual_start = (*start as i32 + offset) as usize;
            let actual_end = (*end as i32 + offset) as usize;

            let original_len = actual_end - actual_start;
            result.replace_range(actual_start..actual_end, redaction);

            offset += redaction.len() as i32 - original_len as i32;
        }

        result
    }
}

#[async_trait]
impl Scanner for BanCompetitors {
    fn name(&self) -> &str {
        "BanCompetitors"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let matches = self.find_matches(input);

        if matches.is_empty() {
            return Ok(ScanResult::pass(input.to_string()));
        }

        // Build entities for each match
        let entities: Vec<Entity> = matches
            .iter()
            .map(|(start, end, matched_text)| {
                let mut metadata = HashMap::new();
                metadata.insert("competitor".to_string(), matched_text.clone());

                Entity {
                    entity_type: "competitor_mention".to_string(),
                    text: matched_text.clone(),
                    start: *start,
                    end: *end,
                    confidence: 1.0,
                    metadata,
                }
            })
            .collect();

        let description = format!("Found {} competitor mention(s)", matches.len());
        let risk_factor = RiskFactor::new(
            "competitor_mention",
            &description,
            Severity::High,
            1.0,
        );

        let sanitized_text = self.redact_text(input, &matches);

        let mut result = ScanResult::new(sanitized_text, false, 1.0)
            .with_risk_factor(risk_factor)
            .with_metadata("competitors_found", matches.len())
            .with_metadata("competitors", matches.iter().map(|(_, _, t)| t.as_str()).collect::<Vec<_>>().join(", "));

        for entity in entities {
            result = result.with_entity(entity);
        }

        Ok(result)
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Detects and blocks mentions of competitor brands or companies"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ban_competitors_exact_match() {
        let scanner = BanCompetitors::with_competitors(vec!["CompetitorA", "CompetitorB"]).unwrap();
        let vault = Vault::new();

        let text = "Check out CompetitorA's new product";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.risk_score, 1.0);
        assert_eq!(result.entities.len(), 1);
    }

    #[tokio::test]
    async fn test_ban_competitors_case_insensitive() {
        let scanner = BanCompetitors::with_competitors(vec!["CompetitorA"]).unwrap();
        let vault = Vault::new();

        let text = "Have you tried competitora?";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_ban_competitors_no_match() {
        let scanner = BanCompetitors::with_competitors(vec!["CompetitorA"]).unwrap();
        let vault = Vault::new();

        let text = "Our product is the best";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
    }

    #[tokio::test]
    async fn test_ban_competitors_multiple_matches() {
        let scanner = BanCompetitors::with_competitors(vec!["CompetitorA", "CompetitorB"]).unwrap();
        let vault = Vault::new();

        let text = "CompetitorA and CompetitorB are both mentioned here";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 2);
    }

    #[tokio::test]
    async fn test_ban_competitors_redaction() {
        let scanner = BanCompetitors::with_competitors(vec!["CompetitorA"]).unwrap();
        let vault = Vault::new();

        let text = "Check out CompetitorA's website";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.sanitized_text.contains("[COMPETITOR]"));
        assert!(!result.sanitized_text.contains("CompetitorA"));
    }

    #[tokio::test]
    async fn test_ban_competitors_whole_word_only() {
        let scanner = BanCompetitors::with_competitors(vec!["test"]).unwrap();
        let vault = Vault::new();

        // "test" as whole word
        let text1 = "This is a test message";
        let result1 = scanner.scan(text1, &vault).await.unwrap();
        assert!(!result1.is_valid);

        // "test" as part of word
        let text2 = "This is testing something";
        let result2 = scanner.scan(text2, &vault).await.unwrap();
        assert!(result2.is_valid);
    }

    #[tokio::test]
    async fn test_ban_competitors_case_sensitive() {
        let config = BanCompetitorsConfig {
            competitors: vec!["CompetitorA".to_string()],
            case_sensitive: true,
            ..Default::default()
        };
        let scanner = BanCompetitors::new(config).unwrap();
        let vault = Vault::new();

        // Exact case match
        let text1 = "Check out CompetitorA";
        let result1 = scanner.scan(text1, &vault).await.unwrap();
        assert!(!result1.is_valid);

        // Different case
        let text2 = "Check out competitora";
        let result2 = scanner.scan(text2, &vault).await.unwrap();
        assert!(result2.is_valid);
    }

    #[tokio::test]
    async fn test_ban_competitors_no_redaction() {
        let config = BanCompetitorsConfig {
            competitors: vec!["CompetitorA".to_string()],
            redact: false,
            ..Default::default()
        };
        let scanner = BanCompetitors::new(config).unwrap();
        let vault = Vault::new();

        let text = "Check out CompetitorA";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.sanitized_text, text);
    }

    #[tokio::test]
    async fn test_ban_competitors_partial_word() {
        let config = BanCompetitorsConfig {
            competitors: vec!["comp".to_string()],
            whole_words_only: false,  // Allow partial matches
            ..Default::default()
        };
        let scanner = BanCompetitors::new(config).unwrap();
        let vault = Vault::new();

        let text = "My computer is fast";
        let result = scanner.scan(text, &vault).await.unwrap();

        // Should match "comp" in "computer"
        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_ban_competitors_multiple_occurrences() {
        let scanner = BanCompetitors::with_competitors(vec!["CompetitorA"]).unwrap();
        let vault = Vault::new();

        let text = "CompetitorA is great. I love CompetitorA. CompetitorA rocks!";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 3);
    }

    #[tokio::test]
    async fn test_ban_competitors_mixed_case() {
        let scanner = BanCompetitors::with_competitors(vec!["competitor"]).unwrap();
        let vault = Vault::new();

        let text = "COMPETITOR, Competitor, and competitor are all here";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 3);
    }
}
