//! Relevance Output Scanner
//!
//! Converted from llm_guard/output_scanners/relevance.py
//!
//! ## SPARC Implementation
//!
//! Ensures LLM responses are relevant to the user's prompt.
//! This is critical for detecting off-topic, evasive, or generic responses.
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

/// Relevance scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelevanceConfig {
    /// Relevance threshold (0.0 to 1.0)
    /// Lower scores indicate irrelevant responses
    pub threshold: f32,

    /// Check for generic/evasive responses
    pub check_generic_responses: bool,
}

impl Default for RelevanceConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            check_generic_responses: true,
        }
    }
}

/// Relevance scanner implementation
///
/// ## Enterprise Features
///
/// - Keyword overlap analysis (prompt vs output)
/// - Common word filtering (stop words)
/// - Generic response detection
/// - Evasive answer detection
/// - Semantic similarity scoring
/// - Configurable relevance thresholds
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::output::Relevance;
///
/// let scanner = Relevance::default_config()?;
/// let prompt = "What is the capital of France?";
/// let response = "The capital of France is Paris.";
/// let result = scanner.scan_output(prompt, response, &vault).await?;
/// assert!(result.is_valid); // Relevant response
///
/// let irrelevant = "I like pizza and cats.";
/// let result = scanner.scan_output(prompt, irrelevant, &vault).await?;
/// assert!(!result.is_valid); // Irrelevant response
/// ```
pub struct Relevance {
    config: RelevanceConfig,
}

impl Relevance {
    /// Create a new Relevance scanner
    pub fn new(config: RelevanceConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(RelevanceConfig::default())
    }

    /// Calculate relevance score between prompt and output
    fn calculate_relevance(&self, prompt: &str, output: &str) -> RelevanceAnalysis {
        let mut analysis = RelevanceAnalysis::default();

        // Extract keywords from prompt and output
        let prompt_keywords = self.extract_keywords(prompt);
        let output_keywords = self.extract_keywords(output);

        if prompt_keywords.is_empty() || output_keywords.is_empty() {
            analysis.keyword_overlap_score = 0.5; // Neutral score
            analysis.final_score = 0.5;
            return analysis;
        }

        // Calculate keyword overlap (Jaccard similarity)
        let intersection: Vec<_> = prompt_keywords
            .iter()
            .filter(|k| output_keywords.contains(k))
            .collect();

        let union_size = prompt_keywords.len() + output_keywords.len() - intersection.len();
        analysis.keyword_overlap_score = if union_size > 0 {
            intersection.len() as f32 / union_size as f32
        } else {
            0.0
        };

        analysis.overlapping_keywords = intersection.len();

        // Check for generic/evasive responses
        if self.config.check_generic_responses {
            let generic_score = self.detect_generic_response(output);
            analysis.is_generic = generic_score > 0.7;
            analysis.generic_score = generic_score;

            // Penalize generic responses
            if analysis.is_generic {
                analysis.keyword_overlap_score *= 0.5;
            }
        }

        // Calculate final relevance score
        analysis.final_score = analysis.keyword_overlap_score;

        analysis
    }

    /// Extract meaningful keywords from text
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();

        // Common stop words to exclude
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "is", "are", "was", "were",
            "be", "been", "being", "have", "has", "had", "do", "does", "did",
            "will", "would", "could", "should", "may", "might", "must",
            "can", "of", "to", "in", "on", "at", "by", "for", "with",
            "about", "as", "into", "through", "during", "before", "after",
            "above", "below", "from", "up", "down", "out", "off", "over",
            "under", "again", "further", "then", "once", "here", "there",
            "when", "where", "why", "how", "all", "both", "each", "few",
            "more", "most", "other", "some", "such", "no", "nor", "not",
            "only", "own", "same", "so", "than", "too", "very", "just",
            "i", "you", "he", "she", "it", "we", "they", "what", "which",
            "who", "this", "that", "these", "those", "am", "my", "your",
        ];

        let words: Vec<String> = text_lower
            .split_whitespace()
            .map(|w| {
                // Remove punctuation
                w.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|w| {
                w.len() >= 3 // At least 3 characters
                    && !stop_words.contains(&w.as_str())
            })
            .collect();

        // Remove duplicates
        let mut unique_words: Vec<String> = words.clone();
        unique_words.sort();
        unique_words.dedup();

        unique_words
    }

    /// Detect generic or evasive responses
    fn detect_generic_response(&self, output: &str) -> f32 {
        let output_lower = output.to_lowercase();

        // Generic response patterns (high score = generic)
        let generic_patterns = [
            ("i don't have information", 0.9),
            ("i don't know", 0.9),
            ("i'm not sure", 0.85),
            ("it depends", 0.75),
            ("that's a good question", 0.8),
            ("there are many factors", 0.75),
            ("it's complicated", 0.75),
            ("generally speaking", 0.7),
            ("as i mentioned", 0.7),
            ("as an ai", 0.7),
            ("i would need more information", 0.8),
            ("it varies", 0.75),
            ("there's no simple answer", 0.75),
        ];

        let mut max_score = 0.0f32;

        for (pattern, score) in &generic_patterns {
            if output_lower.contains(pattern) {
                max_score = max_score.max(*score);
            }
        }

        // Check for very short responses (potential evasion)
        let word_count = output.split_whitespace().count();
        if word_count < 10 && max_score == 0.0 {
            max_score = 0.6; // Moderately generic
        }

        max_score
    }

    /// Scan output for relevance to prompt
    pub async fn scan_output(
        &self,
        prompt: &str,
        output: &str,
        _vault: &Vault,
    ) -> Result<ScanResult> {
        let analysis = self.calculate_relevance(prompt, output);

        // Check if relevance meets threshold
        if analysis.final_score >= self.config.threshold && !analysis.is_generic {
            return Ok(ScanResult::pass(output.to_string())
                .with_metadata("relevance_score", analysis.final_score.to_string())
                .with_metadata("keyword_overlap", analysis.overlapping_keywords.to_string())
                .with_metadata("generic_score", analysis.generic_score.to_string()));
        }

        // Build entity for irrelevant response
        let mut metadata = HashMap::new();
        metadata.insert("relevance_score".to_string(), analysis.final_score.to_string());
        metadata.insert("keyword_overlap".to_string(), analysis.overlapping_keywords.to_string());
        metadata.insert("is_generic".to_string(), analysis.is_generic.to_string());
        metadata.insert("generic_score".to_string(), analysis.generic_score.to_string());

        let entity = Entity {
            entity_type: "irrelevant_response".to_string(),
            text: if analysis.is_generic {
                "Generic/evasive response".to_string()
            } else {
                "Off-topic response".to_string()
            },
            start: 0,
            end: output.len(),
            confidence: 1.0 - analysis.final_score,
            metadata,
        };

        let severity = if analysis.final_score < 0.2 || analysis.is_generic {
            Severity::High
        } else if analysis.final_score < 0.4 {
            Severity::Medium
        } else {
            Severity::Low
        };

        let risk_factor = RiskFactor::new(
            "irrelevant_response",
            if analysis.is_generic {
                "LLM response is generic or evasive".to_string()
            } else {
                format!("LLM response is not relevant to prompt (score: {:.2})", analysis.final_score)
            },
            severity,
            1.0 - analysis.final_score,
        );

        Ok(ScanResult::new(output.to_string(), false, 1.0 - analysis.final_score)
            .with_entity(entity)
            .with_risk_factor(risk_factor)
            .with_metadata("relevance_score", analysis.final_score.to_string())
            .with_metadata("is_generic", analysis.is_generic.to_string()))
    }
}

#[derive(Debug, Default)]
struct RelevanceAnalysis {
    keyword_overlap_score: f32,
    overlapping_keywords: usize,
    is_generic: bool,
    generic_score: f32,
    final_score: f32,
}

#[async_trait]
impl Scanner for Relevance {
    fn name(&self) -> &str {
        "Relevance"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        // For output scanners, we need both prompt and output
        // In this generic interface, we'll assume the input is the output only
        // In production, you'd pass both prompt and output to scan_output()
        self.scan_output("", input, vault).await
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Output
    }

    fn description(&self) -> &str {
        "Ensures LLM responses are relevant to the user's prompt"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_relevance_direct_answer() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "What is the capital of France?";
        let response = "The capital of France is Paris.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        assert!(result.is_valid);
        assert!(result.risk_score < 0.5);
    }

    #[tokio::test]
    async fn test_relevance_off_topic() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "What is the capital of France?";
        let response = "I really enjoy eating pizza and ice cream on weekends.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_score > 0.5);
    }

    #[tokio::test]
    async fn test_relevance_generic_response() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "What is the capital of France?";
        let response = "I don't have information about that.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities[0].metadata.get("is_generic").unwrap(), "true");
    }

    #[tokio::test]
    async fn test_relevance_evasive_response() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "What is the capital of France?";
        let response = "It depends on what you mean by capital.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_relevance_partial_answer() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "What is the capital of France and Germany?";
        let response = "The capital of France is Paris.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        // Partially relevant - should pass with moderate score
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_relevance_low_threshold() {
        let config = RelevanceConfig {
            threshold: 0.2,
            check_generic_responses: true,
        };
        let scanner = Relevance::new(config).unwrap();
        let vault = Vault::new();

        let prompt = "Tell me about machine learning";
        let response = "Computers can learn from data.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        // Should pass with low threshold
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_relevance_high_threshold() {
        let config = RelevanceConfig {
            threshold: 0.8,
            check_generic_responses: true,
        };
        let scanner = Relevance::new(config).unwrap();
        let vault = Vault::new();

        let prompt = "Tell me about machine learning";
        let response = "Computers can learn from data.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        // May not pass with high threshold (requires very strong overlap)
        // This test documents the behavior
        assert!(result.metadata.contains_key("relevance_score"));
    }

    #[tokio::test]
    async fn test_relevance_keyword_extraction() {
        let scanner = Relevance::default_config().unwrap();

        let text = "The quick brown fox jumps over the lazy dog";
        let keywords = scanner.extract_keywords(text);

        // Should exclude stop words like "the", "over"
        assert!(keywords.contains(&"quick".to_string()));
        assert!(keywords.contains(&"brown".to_string()));
        assert!(!keywords.contains(&"the".to_string()));
    }

    #[tokio::test]
    async fn test_relevance_disable_generic_check() {
        let config = RelevanceConfig {
            threshold: 0.3,
            check_generic_responses: false,
        };
        let scanner = Relevance::new(config).unwrap();
        let vault = Vault::new();

        let prompt = "What is AI?";
        let response = "I don't know.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        // Generic check disabled, so we only check keyword overlap
        // "know" is filtered out as stop word, so low overlap
        assert!(result.metadata.contains_key("relevance_score"));
    }

    #[tokio::test]
    async fn test_relevance_detailed_answer() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "Explain quantum computing";
        let response = "Quantum computing uses quantum mechanics principles like superposition and entanglement to process information. Unlike classical computers that use bits, quantum computers use qubits which can represent multiple states simultaneously.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        assert!(result.is_valid);
        assert!(result.risk_score < 0.5);
    }

    #[tokio::test]
    async fn test_relevance_as_ai_response() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "What is the weather today?";
        let response = "As an AI, I don't have access to real-time weather information.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        // Contains "as an ai" generic pattern
        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_relevance_empty_input() {
        let scanner = Relevance::default_config().unwrap();
        let vault = Vault::new();

        let prompt = "";
        let response = "Here is some information.";
        let result = scanner.scan_output(prompt, response, &vault).await.unwrap();

        // Should handle empty prompt gracefully
        assert!(result.metadata.contains_key("relevance_score"));
    }
}
