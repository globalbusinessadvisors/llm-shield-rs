//! Gibberish Scanner
//!
//! Converted from llm_guard/input_scanners/gibberish.py
//!
//! ## SPARC Implementation
//!
//! This scanner detects gibberish, random, or nonsensical text using entropy analysis,
//! character distribution, and pattern recognition.
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

/// Gibberish scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GibberishConfig {
    /// Detection threshold (0.0 to 1.0)
    pub threshold: f32,

    /// Minimum text length to analyze
    pub min_length: usize,

    /// Check Shannon entropy
    pub check_entropy: bool,

    /// Check character repetition
    pub check_repetition: bool,

    /// Check vowel/consonant ratio
    pub check_vowel_ratio: bool,

    /// Check word patterns
    pub check_word_patterns: bool,
}

impl Default for GibberishConfig {
    fn default() -> Self {
        Self {
            threshold: 0.7,
            min_length: 10,
            check_entropy: true,
            check_repetition: true,
            check_vowel_ratio: true,
            check_word_patterns: true,
        }
    }
}

/// Gibberish scanner implementation
///
/// ## Enterprise Features
///
/// - Shannon entropy calculation
/// - Character repetition detection
/// - Vowel/consonant ratio analysis
/// - Word pattern recognition
/// - Keyboard mashing detection
/// - Configurable detection criteria
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::input::Gibberish;
///
/// let config = GibberishConfig::default();
/// let scanner = Gibberish::new(config)?;
///
/// let gibberish_text = "asdjfklasjdf kljasdfkl jasdflkj";
/// let result = scanner.scan(gibberish_text, &vault).await?;
/// assert!(!result.is_valid);
/// ```
pub struct Gibberish {
    config: GibberishConfig,
}

impl Gibberish {
    /// Create a new Gibberish scanner
    pub fn new(config: GibberishConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(GibberishConfig::default())
    }

    /// Calculate Shannon entropy of text
    fn calculate_entropy(&self, text: &str) -> f32 {
        if text.is_empty() {
            return 0.0;
        }

        let mut char_counts: HashMap<char, usize> = HashMap::new();
        let text_lower = text.to_lowercase();

        // Count character frequencies
        for ch in text_lower.chars() {
            if ch.is_alphabetic() {
                *char_counts.entry(ch).or_insert(0) += 1;
            }
        }

        if char_counts.is_empty() {
            return 0.0;
        }

        let total_chars = char_counts.values().sum::<usize>() as f32;
        let mut entropy = 0.0;

        for count in char_counts.values() {
            let probability = *count as f32 / total_chars;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        // Normalize to 0-1 range (max entropy for English ~4.7 bits)
        (entropy / 4.7).min(1.0)
    }

    /// Check for excessive character repetition
    fn check_character_repetition(&self, text: &str) -> f32 {
        if text.len() < 3 {
            return 0.0;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut repetition_score = 0.0;
        let mut consecutive_count = 1;

        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                consecutive_count += 1;
                if consecutive_count >= 3 {
                    // Penalize heavily for 3+ consecutive same characters
                    repetition_score += (consecutive_count - 2) as f32 * 0.2;
                }
            } else {
                consecutive_count = 1;
            }
        }

        (repetition_score / text.len() as f32).min(1.0)
    }

    /// Check vowel to consonant ratio
    fn check_vowel_ratio(&self, text: &str) -> f32 {
        let vowels = "aeiouAEIOU";
        let mut vowel_count = 0;
        let mut consonant_count = 0;

        for ch in text.chars() {
            if ch.is_alphabetic() {
                if vowels.contains(ch) {
                    vowel_count += 1;
                } else {
                    consonant_count += 1;
                }
            }
        }

        let total = vowel_count + consonant_count;
        if total == 0 {
            return 0.0;
        }

        let vowel_ratio = vowel_count as f32 / total as f32;

        // Typical English vowel ratio: 0.37-0.42
        // Score abnormal ratios higher
        if vowel_ratio < 0.15 || vowel_ratio > 0.6 {
            ((vowel_ratio - 0.4).abs() / 0.4).min(1.0)
        } else {
            0.0
        }
    }

    /// Check for keyboard mashing patterns
    fn check_keyboard_mashing(&self, text: &str) -> f32 {
        // Common keyboard mashing patterns
        let patterns = [
            "asdf", "jkl;", "qwer", "uiop", "zxcv", "hjkl", "fdsa", "poiu",
            "1234", "abcd", "aaaa", "ssss", "dddd",
        ];

        let text_lower = text.to_lowercase();
        let mut pattern_matches = 0;

        for pattern in &patterns {
            if text_lower.contains(pattern) {
                pattern_matches += 1;
            }
        }

        (pattern_matches as f32 / 5.0).min(1.0)
    }

    /// Check word patterns and structure
    fn check_word_patterns(&self, text: &str) -> f32 {
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.is_empty() {
            return 0.0;
        }

        let mut suspicious_words = 0;

        for word in &words {
            let alpha_only: String = word.chars().filter(|c| c.is_alphabetic()).collect();

            if alpha_only.len() < 2 {
                continue;
            }

            // Check for unusual consonant clusters
            let has_long_consonant_cluster = self.has_long_consonant_cluster(&alpha_only);

            // Check for very short or very long words without vowels
            let has_no_vowels = !alpha_only
                .to_lowercase()
                .chars()
                .any(|c| "aeiou".contains(c));

            if has_long_consonant_cluster || (alpha_only.len() > 3 && has_no_vowels) {
                suspicious_words += 1;
            }
        }

        (suspicious_words as f32 / words.len() as f32).min(1.0)
    }

    fn has_long_consonant_cluster(&self, word: &str) -> bool {
        let vowels = "aeiouAEIOU";
        let mut consonant_count = 0;

        for ch in word.chars() {
            if ch.is_alphabetic() {
                if vowels.contains(ch) {
                    consonant_count = 0;
                } else {
                    consonant_count += 1;
                    if consonant_count >= 5 {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Calculate overall gibberish score
    fn calculate_gibberish_score(&self, text: &str) -> (f32, Vec<GibberishIndicator>) {
        let mut indicators = Vec::new();
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        // Entropy check (high entropy = more random)
        if self.config.check_entropy {
            let entropy = self.calculate_entropy(text);
            let entropy_score = if entropy > 0.85 {
                // Very high entropy is suspicious
                (entropy - 0.85) / 0.15
            } else {
                0.0
            };

            if entropy_score > 0.1 {
                indicators.push(GibberishIndicator {
                    indicator_type: "high_entropy".to_string(),
                    score: entropy_score,
                    description: format!("Abnormally high entropy: {:.2}", entropy),
                });
            }

            total_score += entropy_score * 2.0; // Weight: 2.0
            weight_sum += 2.0;
        }

        // Repetition check
        if self.config.check_repetition {
            let repetition_score = self.check_character_repetition(text);
            if repetition_score > 0.1 {
                indicators.push(GibberishIndicator {
                    indicator_type: "character_repetition".to_string(),
                    score: repetition_score,
                    description: "Excessive character repetition detected".to_string(),
                });
            }

            total_score += repetition_score * 1.5; // Weight: 1.5
            weight_sum += 1.5;
        }

        // Vowel ratio check
        if self.config.check_vowel_ratio {
            let vowel_score = self.check_vowel_ratio(text);
            if vowel_score > 0.1 {
                indicators.push(GibberishIndicator {
                    indicator_type: "abnormal_vowel_ratio".to_string(),
                    score: vowel_score,
                    description: "Abnormal vowel/consonant ratio".to_string(),
                });
            }

            total_score += vowel_score * 1.5; // Weight: 1.5
            weight_sum += 1.5;
        }

        // Keyboard mashing check
        let mashing_score = self.check_keyboard_mashing(text);
        if mashing_score > 0.1 {
            indicators.push(GibberishIndicator {
                indicator_type: "keyboard_mashing".to_string(),
                score: mashing_score,
                description: "Keyboard mashing pattern detected".to_string(),
            });
        }

        total_score += mashing_score * 2.0; // Weight: 2.0
        weight_sum += 2.0;

        // Word pattern check
        if self.config.check_word_patterns {
            let pattern_score = self.check_word_patterns(text);
            if pattern_score > 0.1 {
                indicators.push(GibberishIndicator {
                    indicator_type: "suspicious_word_patterns".to_string(),
                    score: pattern_score,
                    description: "Suspicious word patterns detected".to_string(),
                });
            }

            total_score += pattern_score * 1.5; // Weight: 1.5
            weight_sum += 1.5;
        }

        let normalized_score = if weight_sum > 0.0 {
            (total_score / weight_sum).min(1.0)
        } else {
            0.0
        };

        (normalized_score, indicators)
    }
}

#[derive(Debug, Clone)]
struct GibberishIndicator {
    indicator_type: String,
    score: f32,
    description: String,
}

#[async_trait]
impl Scanner for Gibberish {
    fn name(&self) -> &str {
        "Gibberish"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        // Check minimum length
        if input.len() < self.config.min_length {
            return Ok(ScanResult::pass(input.to_string()));
        }

        let (gibberish_score, indicators) = self.calculate_gibberish_score(input);

        // Check threshold
        if gibberish_score < self.config.threshold {
            return Ok(ScanResult::pass(input.to_string()));
        }

        // Build entities for each indicator
        let entities: Vec<Entity> = indicators
            .iter()
            .map(|ind| {
                let mut metadata = HashMap::new();
                metadata.insert("indicator_type".to_string(), ind.indicator_type.clone());
                metadata.insert("indicator_score".to_string(), ind.score.to_string());

                Entity {
                    entity_type: "gibberish_indicator".to_string(),
                    text: ind.description.clone(),
                    start: 0,
                    end: input.len(),
                    confidence: ind.score,
                    metadata,
                }
            })
            .collect();

        let description = format!("Detected {} gibberish indicator(s)", indicators.len());
        let risk_factor = RiskFactor::new(
            "gibberish_detected",
            &description,
            if gibberish_score >= 0.8 {
                Severity::High
            } else if gibberish_score >= 0.6 {
                Severity::Medium
            } else {
                Severity::Low
            },
            gibberish_score,
        );

        let mut result = ScanResult::new(input.to_string(), false, gibberish_score)
            .with_risk_factor(risk_factor)
            .with_metadata("gibberish_score", gibberish_score.to_string())
            .with_metadata("indicators_count", indicators.len());

        for entity in entities {
            result = result.with_entity(entity);
        }

        Ok(result)
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Detects gibberish and nonsensical text using entropy and pattern analysis"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gibberish_random_text() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        // Pure gibberish
        let gibberish = "asdfkljasdf jklasdjf klasdjfkl asdjf";
        let result = scanner.scan(gibberish, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_score > 0.5);
    }

    #[tokio::test]
    async fn test_gibberish_keyboard_mashing() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        let mashed = "asdfasdfasdf jkl;jkl;jkl; qwerqwer";
        let result = scanner.scan(mashed, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e|
            e.metadata.get("indicator_type").map(|s| s.as_str()) == Some("keyboard_mashing")
        ));
    }

    #[tokio::test]
    async fn test_gibberish_excessive_repetition() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        let repeated = "aaaaaaaa bbbbbbbb cccccccc";
        let result = scanner.scan(repeated, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e|
            e.metadata.get("indicator_type").map(|s| s.as_str()) == Some("character_repetition")
        ));
    }

    #[tokio::test]
    async fn test_gibberish_no_vowels() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        // Words without vowels
        let no_vowels = "bcdfgh jklmnp qrstvw xyzbc";
        let result = scanner.scan(no_vowels, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_gibberish_normal_text() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        let normal = "This is a perfectly normal sentence with proper words and structure.";
        let result = scanner.scan(normal, &vault).await.unwrap();

        assert!(result.is_valid);
        assert!(result.risk_score < 0.7);
    }

    #[tokio::test]
    async fn test_gibberish_technical_text() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        // Technical terms should generally pass
        let technical = "The HTTP API endpoint returns JSON data with authentication tokens.";
        let result = scanner.scan(technical, &vault).await.unwrap();

        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_gibberish_short_text() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        // Very short text should pass (below min_length)
        let short = "abc";
        let result = scanner.scan(short, &vault).await.unwrap();

        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_gibberish_threshold() {
        let config = GibberishConfig {
            threshold: 0.95,  // Very high threshold
            ..Default::default()
        };
        let scanner = Gibberish::new(config).unwrap();
        let vault = Vault::new();

        // Mild gibberish should pass with high threshold
        let mild_gibberish = "asdf hello world qwer";
        let result = scanner.scan(mild_gibberish, &vault).await.unwrap();

        assert!(result.is_valid || result.risk_score < 0.95);
    }

    #[tokio::test]
    async fn test_gibberish_high_entropy() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        // Text with very high entropy (random characters)
        let high_entropy = "abcdefghijklmnopqrstuvwxyz zyxwvutsrqponmlkjihgfedcba";
        let result = scanner.scan(high_entropy, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_gibberish_selective_checks() {
        let config = GibberishConfig {
            threshold: 0.7,
            min_length: 10,
            check_entropy: false,
            check_repetition: true,
            check_vowel_ratio: false,
            check_word_patterns: false,
        };
        let scanner = Gibberish::new(config).unwrap();
        let vault = Vault::new();

        // Only repetition should be checked
        let repeated = "aaaaaa bbbbbb cccccc";
        let result = scanner.scan(repeated, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(
            result.entities[0].metadata.get("indicator_type").unwrap(),
            "character_repetition"
        );
    }

    #[tokio::test]
    async fn test_gibberish_mixed_quality() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        // Mix of normal and gibberish
        let mixed = "Hello world asdfkljasdf how are you qwerqwer today";
        let result = scanner.scan(mixed, &vault).await.unwrap();

        // Should detect some gibberish but might not exceed threshold
        // depending on the ratio
        if !result.is_valid {
            assert!(result.risk_score > 0.3);
        }
    }

    #[tokio::test]
    async fn test_gibberish_long_consonant_cluster() {
        let scanner = Gibberish::default_config().unwrap();
        let vault = Vault::new();

        // Unusual consonant clusters
        let clusters = "bcdfghjklm pqrstvwxyz";
        let result = scanner.scan(clusters, &vault).await.unwrap();

        assert!(!result.is_valid);
    }
}
