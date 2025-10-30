//! InvisibleText Scanner
//!
//! Converted from llm_guard/input_scanners/invisible_text.py
//!
//! ## SPARC Implementation
//!
//! This scanner detects hidden/invisible Unicode characters that could be used
//! for prompt injection, data exfiltration, or bypassing content filters.
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

/// InvisibleText scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvisibleTextConfig {
    /// Detection threshold (0.0 to 1.0)
    pub threshold: f32,

    /// Whether to remove invisible characters
    pub remove: bool,

    /// Detect zero-width characters
    pub detect_zero_width: bool,

    /// Detect control characters
    pub detect_control: bool,

    /// Detect direction marks
    pub detect_direction_marks: bool,

    /// Detect non-printable characters
    pub detect_non_printable: bool,
}

impl Default for InvisibleTextConfig {
    fn default() -> Self {
        Self {
            threshold: 0.1, // Even small amounts are suspicious
            remove: true,
            detect_zero_width: true,
            detect_control: true,
            detect_direction_marks: true,
            detect_non_printable: true,
        }
    }
}

/// Categories of invisible characters
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InvisibleCharType {
    ZeroWidth,
    Control,
    DirectionMark,
    NonPrintable,
}

impl InvisibleCharType {
    fn as_str(&self) -> &str {
        match self {
            Self::ZeroWidth => "zero_width",
            Self::Control => "control_character",
            Self::DirectionMark => "direction_mark",
            Self::NonPrintable => "non_printable",
        }
    }

    fn confidence(&self) -> f32 {
        match self {
            Self::ZeroWidth => 1.0,        // Very suspicious
            Self::Control => 0.9,           // Usually suspicious
            Self::DirectionMark => 0.7,     // Sometimes legitimate
            Self::NonPrintable => 0.85,     // Usually suspicious
        }
    }
}

// Zero-width characters (Unicode)
const ZERO_WIDTH_CHARS: &[char] = &[
    '\u{200B}', // Zero Width Space (ZWSP)
    '\u{200C}', // Zero Width Non-Joiner (ZWNJ)
    '\u{200D}', // Zero Width Joiner (ZWJ)
    '\u{FEFF}', // Zero Width No-Break Space (BOM)
    '\u{2060}', // Word Joiner
    '\u{180E}', // Mongolian Vowel Separator
];

// Direction marks
const DIRECTION_MARKS: &[char] = &[
    '\u{202A}', // Left-to-Right Embedding
    '\u{202B}', // Right-to-Left Embedding
    '\u{202C}', // Pop Directional Formatting
    '\u{202D}', // Left-to-Right Override
    '\u{202E}', // Right-to-Left Override
    '\u{2066}', // Left-to-Right Isolate
    '\u{2067}', // Right-to-Left Isolate
    '\u{2068}', // First Strong Isolate
    '\u{2069}', // Pop Directional Isolate
];

/// InvisibleText scanner implementation
///
/// ## Enterprise Features
///
/// - Detects zero-width Unicode characters (ZWSP, ZWNJ, ZWJ, etc.)
/// - Identifies Unicode control characters
/// - Detects bidirectional text marks (LTR, RTL)
/// - Finds non-printable characters
/// - Configurable detection categories
/// - Optional removal/sanitization
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::input::InvisibleText;
///
/// let config = InvisibleTextConfig::default();
/// let scanner = InvisibleText::new(config)?;
///
/// // Text with invisible zero-width space
/// let text = "Hello\u{200B}World";
/// let result = scanner.scan(text, &vault).await?;
/// assert!(!result.is_valid);
/// ```
pub struct InvisibleText {
    config: InvisibleTextConfig,
}

impl InvisibleText {
    /// Create a new InvisibleText scanner
    pub fn new(config: InvisibleTextConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(InvisibleTextConfig::default())
    }

    /// Detect invisible characters in text
    fn detect_invisible_chars(&self, text: &str) -> Vec<InvisibleMatch> {
        let mut matches = Vec::new();

        for (idx, ch) in text.char_indices() {
            // Check for zero-width characters
            if self.config.detect_zero_width && ZERO_WIDTH_CHARS.contains(&ch) {
                matches.push(InvisibleMatch {
                    position: idx,
                    character: ch,
                    char_type: InvisibleCharType::ZeroWidth,
                    name: self.get_char_name(ch),
                });
                continue;
            }

            // Check for direction marks
            if self.config.detect_direction_marks && DIRECTION_MARKS.contains(&ch) {
                matches.push(InvisibleMatch {
                    position: idx,
                    character: ch,
                    char_type: InvisibleCharType::DirectionMark,
                    name: self.get_char_name(ch),
                });
                continue;
            }

            // Check for control characters
            if self.config.detect_control && ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                matches.push(InvisibleMatch {
                    position: idx,
                    character: ch,
                    char_type: InvisibleCharType::Control,
                    name: format!("U+{:04X}", ch as u32),
                });
                continue;
            }

            // Check for non-printable characters (excluding common whitespace)
            if self.config.detect_non_printable
                && !ch.is_whitespace()
                && !ch.is_alphanumeric()
                && !ch.is_ascii_punctuation()
                && ch.is_control()
            {
                matches.push(InvisibleMatch {
                    position: idx,
                    character: ch,
                    char_type: InvisibleCharType::NonPrintable,
                    name: format!("U+{:04X}", ch as u32),
                });
            }
        }

        matches
    }

    fn get_char_name(&self, ch: char) -> String {
        match ch {
            '\u{200B}' => "Zero Width Space (ZWSP)".to_string(),
            '\u{200C}' => "Zero Width Non-Joiner (ZWNJ)".to_string(),
            '\u{200D}' => "Zero Width Joiner (ZWJ)".to_string(),
            '\u{FEFF}' => "Zero Width No-Break Space (BOM)".to_string(),
            '\u{2060}' => "Word Joiner".to_string(),
            '\u{180E}' => "Mongolian Vowel Separator".to_string(),
            '\u{202A}' => "Left-to-Right Embedding".to_string(),
            '\u{202B}' => "Right-to-Left Embedding".to_string(),
            '\u{202C}' => "Pop Directional Formatting".to_string(),
            '\u{202D}' => "Left-to-Right Override".to_string(),
            '\u{202E}' => "Right-to-Left Override".to_string(),
            '\u{2066}' => "Left-to-Right Isolate".to_string(),
            '\u{2067}' => "Right-to-Left Isolate".to_string(),
            '\u{2068}' => "First Strong Isolate".to_string(),
            '\u{2069}' => "Pop Directional Isolate".to_string(),
            _ => format!("U+{:04X}", ch as u32),
        }
    }

    fn calculate_risk_score(&self, matches: &[InvisibleMatch], text_len: usize) -> f32 {
        if matches.is_empty() {
            return 0.0;
        }

        // Calculate density of invisible characters
        let invisible_density = matches.len() as f32 / text_len.max(1) as f32;

        // Calculate weighted confidence based on character types
        let avg_confidence = matches
            .iter()
            .map(|m| m.char_type.confidence())
            .sum::<f32>() / matches.len() as f32;

        // Combine factors (density is more important for invisible chars)
        (invisible_density * 0.7 + avg_confidence * 0.3).min(1.0)
    }

    fn sanitize_text(&self, text: &str, matches: &[InvisibleMatch]) -> String {
        if !self.config.remove || matches.is_empty() {
            return text.to_string();
        }

        // Remove all invisible characters found
        let mut result = String::with_capacity(text.len());
        let invisible_positions: HashMap<usize, char> = matches
            .iter()
            .map(|m| (m.position, m.character))
            .collect();

        for (idx, ch) in text.char_indices() {
            if !invisible_positions.contains_key(&idx) {
                result.push(ch);
            }
        }

        result
    }
}

#[derive(Debug, Clone)]
struct InvisibleMatch {
    position: usize,
    character: char,
    char_type: InvisibleCharType,
    name: String,
}

#[async_trait]
impl Scanner for InvisibleText {
    fn name(&self) -> &str {
        "InvisibleText"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let matches = self.detect_invisible_chars(input);

        if matches.is_empty() {
            return Ok(ScanResult::pass(input.to_string()));
        }

        let risk_score = self.calculate_risk_score(&matches, input.len());

        // Check threshold
        if risk_score < self.config.threshold {
            return Ok(ScanResult::pass(input.to_string()));
        }

        // Build entities for each invisible character
        let entities: Vec<Entity> = matches
            .iter()
            .map(|m| {
                let mut metadata = HashMap::new();
                metadata.insert("char_type".to_string(), m.char_type.as_str().to_string());
                metadata.insert("unicode_name".to_string(), m.name.clone());
                metadata.insert("unicode_value".to_string(), format!("U+{:04X}", m.character as u32));

                Entity {
                    entity_type: "invisible_character".to_string(),
                    text: format!("[{}]", m.name),
                    start: m.position,
                    end: m.position + m.character.len_utf8(),
                    confidence: m.char_type.confidence(),
                    metadata,
                }
            })
            .collect();

        let risk_factor = RiskFactor::new(
            "invisible_characters",
            format!("Found {} invisible character(s)", matches.len()),
            if risk_score >= 0.7 {
                Severity::High
            } else if risk_score >= 0.4 {
                Severity::Medium
            } else {
                Severity::Low
            },
            risk_score,
        );

        let sanitized_text = self.sanitize_text(input, &matches);

        Ok(ScanResult::new(sanitized_text, false, risk_score)
            .with_entities(entities)
            .with_risk_factor(risk_factor)
            .with_metadata("invisible_count", matches.len())
            .with_metadata("invisible_density", (matches.len() as f32 / input.len().max(1) as f32).to_string()))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Detects hidden/invisible Unicode characters in input text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invisible_text_zero_width_space() {
        let scanner = InvisibleText::default_config().unwrap();
        let vault = Vault::new();

        // Text with zero-width space
        let text = "Hello\u{200B}World";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_score > 0.0);
        assert_eq!(result.entities.len(), 1);
        assert_eq!(result.entities[0].entity_type, "invisible_character");
    }

    #[tokio::test]
    async fn test_invisible_text_multiple_zero_width() {
        let scanner = InvisibleText::default_config().unwrap();
        let vault = Vault::new();

        // Multiple zero-width characters
        let text = "Test\u{200B}with\u{200C}multiple\u{200D}invisible";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.entities.len(), 3);
    }

    #[tokio::test]
    async fn test_invisible_text_direction_marks() {
        let scanner = InvisibleText::default_config().unwrap();
        let vault = Vault::new();

        // Right-to-left override (used in homograph attacks)
        let text = "Hello\u{202E}World";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e|
            e.metadata.get("char_type").map(|s| s.as_str()) == Some("direction_mark")
        ));
    }

    #[tokio::test]
    async fn test_invisible_text_control_characters() {
        let scanner = InvisibleText::default_config().unwrap();
        let vault = Vault::new();

        // Text with control character (bell)
        let text = "Alert\u{0007}message";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_invisible_text_clean() {
        let scanner = InvisibleText::default_config().unwrap();
        let vault = Vault::new();

        let clean_text = "This is normal text with spaces and newlines\nNo invisible characters";
        let result = scanner.scan(clean_text, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
        assert!(result.entities.is_empty());
    }

    #[tokio::test]
    async fn test_invisible_text_removal() {
        let config = InvisibleTextConfig {
            threshold: 0.1,
            remove: true,
            ..Default::default()
        };
        let scanner = InvisibleText::new(config).unwrap();
        let vault = Vault::new();

        let text = "Hello\u{200B}World\u{200C}Test";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.sanitized_text, "HelloWorldTest");
    }

    #[tokio::test]
    async fn test_invisible_text_threshold() {
        let config = InvisibleTextConfig {
            threshold: 0.5,  // High threshold
            ..Default::default()
        };
        let scanner = InvisibleText::new(config).unwrap();
        let vault = Vault::new();

        // Single invisible char in long text (low density)
        let text = format!("This is a very long text with lots of words\u{200B} and only one invisible character to make the density very low");
        let result = scanner.scan(&text, &vault).await.unwrap();

        // Should pass because density is low
        assert!(result.is_valid || result.risk_score < 0.5);
    }

    #[tokio::test]
    async fn test_invisible_text_high_density() {
        let scanner = InvisibleText::default_config().unwrap();
        let vault = Vault::new();

        // High density of invisible characters
        let text = "A\u{200B}B\u{200C}C\u{200D}D";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_score > 0.5);
    }

    #[tokio::test]
    async fn test_invisible_text_selective_detection() {
        let config = InvisibleTextConfig {
            threshold: 0.1,
            remove: false,
            detect_zero_width: true,
            detect_control: false,
            detect_direction_marks: false,
            detect_non_printable: false,
        };
        let scanner = InvisibleText::new(config).unwrap();
        let vault = Vault::new();

        // Only zero-width should be detected
        let text1 = "Test\u{200B}zero";
        let result1 = scanner.scan(text1, &vault).await.unwrap();
        assert!(!result1.is_valid);

        // Direction mark should NOT be detected
        let text2 = "Test\u{202E}direction";
        let result2 = scanner.scan(text2, &vault).await.unwrap();
        assert!(result2.is_valid);
    }

    #[tokio::test]
    async fn test_invisible_text_normal_whitespace() {
        let scanner = InvisibleText::default_config().unwrap();
        let vault = Vault::new();

        // Normal whitespace (spaces, tabs, newlines) should be allowed
        let text = "Normal text with\nlines and\ttabs  and  spaces";
        let result = scanner.scan(text, &vault).await.unwrap();

        assert!(result.is_valid);
    }
}
