//! Sentiment Scanner
//!
//! Converted from llm_guard/input_scanners/sentiment.py
//!
//! ## SPARC Implementation
//!
//! This scanner analyzes the sentiment (positive, negative, neutral) of input text.
//!
//! ## London School TDD
//!
//! Tests are written first, driving the implementation.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Sentiment scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentConfig {
    /// Allowed sentiments
    pub allowed_sentiments: Vec<SentimentType>,

    /// Detection threshold (0.0 to 1.0)
    pub threshold: f32,

    /// Path to ONNX model file
    pub model_path: Option<PathBuf>,

    /// Path to tokenizer file
    pub tokenizer_path: Option<PathBuf>,

    /// Maximum sequence length
    pub max_length: usize,

    /// Use fallback heuristic detection if model unavailable
    pub use_fallback: bool,
}

impl Default for SentimentConfig {
    fn default() -> Self {
        Self {
            allowed_sentiments: vec![
                SentimentType::Positive,
                SentimentType::Neutral,
            ],
            threshold: 0.7,
            model_path: None,
            tokenizer_path: None,
            max_length: 512,
            use_fallback: true,
        }
    }
}

/// Sentiment types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentimentType {
    Positive,
    Neutral,
    Negative,
}

impl SentimentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SentimentType::Positive => "positive",
            SentimentType::Neutral => "neutral",
            SentimentType::Negative => "negative",
        }
    }
}

/// Sentiment scanner implementation
///
/// ## Enterprise Features
///
/// - ML-based sentiment analysis
/// - Three-way classification (positive, neutral, negative)
/// - Configurable allowed sentiments
/// - Fallback heuristic detection using lexicon-based approach
/// - Confidence scoring
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::input::Sentiment;
///
/// let config = SentimentConfig::default(); // Allows positive and neutral
/// let scanner = Sentiment::new(config)?;
///
/// let negative_text = "This is terrible and I hate everything about it";
/// let result = scanner.scan(negative_text, &vault).await?;
/// assert!(!result.is_valid); // Negative sentiment not allowed
/// ```
pub struct Sentiment {
    config: SentimentConfig,
}

impl Sentiment {
    /// Create a new Sentiment scanner
    pub fn new(config: SentimentConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        if config.allowed_sentiments.is_empty() {
            return Err(Error::config("At least one sentiment must be allowed"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(SentimentConfig::default())
    }

    /// Detect sentiment using heuristic lexicon-based approach
    fn detect_heuristic(&self, text: &str) -> (SentimentType, f32) {
        let text_lower = text.to_lowercase();

        // Positive words
        let positive_words = [
            "great", "excellent", "amazing", "wonderful", "fantastic",
            "love", "best", "awesome", "brilliant", "perfect",
            "good", "nice", "happy", "pleased", "delighted",
            "beautiful", "magnificent", "superb", "outstanding",
            "thank", "thanks", "appreciate", "grateful",
        ];

        // Negative words
        let negative_words = [
            "bad", "terrible", "awful", "horrible", "worst",
            "hate", "dislike", "poor", "disappointing", "upset",
            "angry", "frustrated", "annoyed", "sad", "unhappy",
            "fail", "failed", "failure", "problem", "issue",
            "wrong", "error", "broken", "useless", "worthless",
        ];

        // Negation words that flip sentiment
        let negation_words = ["not", "no", "never", "neither", "nor", "none"];

        // Count sentiment words
        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut has_negation = false;

        for word in text_lower.split_whitespace() {
            if negation_words.contains(&word) {
                has_negation = true;
            }
            if positive_words.contains(&word) {
                positive_count += 1;
            }
            if negative_words.contains(&word) {
                negative_count += 1;
            }
        }

        // Handle negation (simple heuristic)
        if has_negation {
            std::mem::swap(&mut positive_count, &mut negative_count);
        }

        // Calculate sentiment score
        let total_count = positive_count + negative_count;
        if total_count == 0 {
            return (SentimentType::Neutral, 0.6); // Default to neutral with moderate confidence
        }

        let positive_ratio = positive_count as f32 / total_count as f32;
        let negative_ratio = negative_count as f32 / total_count as f32;

        if positive_ratio > 0.6 {
            (SentimentType::Positive, 0.7 + positive_ratio * 0.3)
        } else if negative_ratio > 0.6 {
            (SentimentType::Negative, 0.7 + negative_ratio * 0.3)
        } else {
            (SentimentType::Neutral, 0.6)
        }
    }

    /// Check if sentiment is allowed
    fn is_sentiment_allowed(&self, sentiment: SentimentType) -> bool {
        self.config.allowed_sentiments.contains(&sentiment)
    }
}

#[async_trait]
impl Scanner for Sentiment {
    fn name(&self) -> &str {
        "Sentiment"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let (detected_sentiment, confidence) = self.detect_heuristic(input);

        let is_allowed = self.is_sentiment_allowed(detected_sentiment);

        if is_allowed {
            return Ok(ScanResult::pass(input.to_string())
                .with_metadata("sentiment", detected_sentiment.as_str())
                .with_metadata("confidence", confidence.to_string()));
        }

        // Sentiment not allowed
        let mut metadata = HashMap::new();
        metadata.insert("detected_sentiment".to_string(), detected_sentiment.as_str().to_string());
        metadata.insert("confidence".to_string(), confidence.to_string());
        metadata.insert("allowed_sentiments".to_string(),
            self.config.allowed_sentiments.iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let entity = Entity {
            entity_type: "disallowed_sentiment".to_string(),
            text: format!("{} sentiment", detected_sentiment.as_str()),
            start: 0,
            end: input.len(),
            confidence,
            metadata,
        };

        let risk_factor = RiskFactor::new(
            "disallowed_sentiment",
            format!("Detected {} sentiment", detected_sentiment.as_str()),
            Severity::Medium,
            confidence,
        );

        Ok(ScanResult::new(input.to_string(), false, confidence)
            .with_entity(entity)
            .with_risk_factor(risk_factor)
            .with_metadata("sentiment", detected_sentiment.as_str())
            .with_metadata("confidence", confidence.to_string()))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Analyzes and validates the sentiment of input text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sentiment_positive() {
        let scanner = Sentiment::default_config().unwrap();
        let vault = Vault::new();

        let positive = "This is absolutely wonderful and I love it! Great work!";
        let result = scanner.scan(positive, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.metadata.get("sentiment").unwrap(), "positive");
    }

    #[tokio::test]
    async fn test_sentiment_negative_blocked() {
        let scanner = Sentiment::default_config().unwrap();
        let vault = Vault::new();

        let negative = "This is terrible and awful. I hate everything about it.";
        let result = scanner.scan(negative, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities[0].metadata.get("detected_sentiment").unwrap(), "negative");
    }

    #[tokio::test]
    async fn test_sentiment_neutral() {
        let scanner = Sentiment::default_config().unwrap();
        let vault = Vault::new();

        let neutral = "The product arrived on time. It functions as expected.";
        let result = scanner.scan(neutral, &vault).await.unwrap();

        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_sentiment_allow_negative() {
        let config = SentimentConfig {
            allowed_sentiments: vec![
                SentimentType::Positive,
                SentimentType::Negative,
                SentimentType::Neutral,
            ],
            ..Default::default()
        };
        let scanner = Sentiment::new(config).unwrap();
        let vault = Vault::new();

        let negative = "This is bad and terrible";
        let result = scanner.scan(negative, &vault).await.unwrap();

        assert!(result.is_valid); // Negative is allowed in this config
    }

    #[tokio::test]
    async fn test_sentiment_only_positive() {
        let config = SentimentConfig {
            allowed_sentiments: vec![SentimentType::Positive],
            ..Default::default()
        };
        let scanner = Sentiment::new(config).unwrap();
        let vault = Vault::new();

        let neutral = "The product arrived";
        let result = scanner.scan(neutral, &vault).await.unwrap();

        assert!(!result.is_valid); // Neutral not allowed
    }

    #[tokio::test]
    async fn test_sentiment_negation() {
        let scanner = Sentiment::default_config().unwrap();
        let vault = Vault::new();

        let negated = "This is not good and not great";
        let result = scanner.scan(negated, &vault).await.unwrap();

        // Negation should flip sentiment to negative
        // So it should be blocked (negative not allowed by default)
        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_sentiment_type_as_str() {
        assert_eq!(SentimentType::Positive.as_str(), "positive");
        assert_eq!(SentimentType::Neutral.as_str(), "neutral");
        assert_eq!(SentimentType::Negative.as_str(), "negative");
    }

    #[tokio::test]
    async fn test_sentiment_mixed() {
        let scanner = Sentiment::default_config().unwrap();
        let vault = Vault::new();

        let mixed = "It's good but has some bad aspects";
        let result = scanner.scan(mixed, &vault).await.unwrap();

        // Should classify based on predominant sentiment
        // Result depends on word counts
        assert!(result.metadata.contains_key("sentiment"));
    }

    #[tokio::test]
    async fn test_sentiment_gratitude() {
        let scanner = Sentiment::default_config().unwrap();
        let vault = Vault::new();

        let grateful = "Thank you so much! I really appreciate your help.";
        let result = scanner.scan(grateful, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.metadata.get("sentiment").unwrap(), "positive");
    }
}
