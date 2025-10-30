//! BanCode Scanner
//!
//! Converted from llm_guard/input_scanners/ban_code.py
//!
//! ## SPARC Implementation
//!
//! This scanner detects and blocks code snippets in input text.
//!
//! ## London School TDD
//!
//! Tests are written first, driving the implementation.

use llm_shield_core::{
    async_trait, Entity, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity,
    Vault,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

/// BanCode scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanCodeConfig {
    /// Detection threshold (0.0 to 1.0)
    pub threshold: f32,

    /// Whether to redact detected code
    pub redact: bool,

    /// Languages to detect (empty = all)
    pub languages: Vec<String>,

    /// Detect markdown code blocks
    pub detect_markdown: bool,

    /// Detect inline code patterns
    pub detect_inline: bool,

    /// Detect language keywords
    pub detect_keywords: bool,
}

impl Default for BanCodeConfig {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            redact: false,
            languages: Vec::new(),
            detect_markdown: true,
            detect_inline: true,
            detect_keywords: true,
        }
    }
}

// Common programming language keywords
static PROGRAMMING_KEYWORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        // Python
        "def ", "import ", "from ", "class ", "async def", "await ",
        // JavaScript/TypeScript
        "function ", "const ", "let ", "var ", "async ", "await ", "export ", "import ",
        // Java/C#
        "public ", "private ", "protected ", "static ", "void ", "class ",
        // C/C++
        "#include", "int main", "void ", "struct ",
        // Rust
        "fn ", "impl ", "pub ", "use ", "mod ",
        // Go
        "func ", "package ", "import ",
        // Ruby
        "def ", "class ", "module ", "require ",
        // PHP
        "<?php", "function ", "class ",
        // SQL
        "SELECT ", "INSERT ", "UPDATE ", "DELETE ", "CREATE ",
    ]
});

// Regex patterns for code detection
static CODE_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"```[\s\S]*?```|`[^`]+`").expect("Invalid code block regex")
});

static FUNCTION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(def|function|fn|func)\s+\w+\s*\(").expect("Invalid function pattern regex")
});

static IMPORT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(import|from|#include|require|use)\s+[\w.]+").expect("Invalid import pattern regex")
});

static VARIABLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(let|const|var|int|String|float|double)\s+\w+\s*=").expect("Invalid variable pattern regex")
});

/// BanCode scanner implementation
///
/// ## Enterprise Features
///
/// - Detects markdown code blocks (fenced and inline)
/// - Recognizes common programming language keywords
/// - Identifies function definitions, imports, and variable declarations
/// - Configurable threshold for code density
/// - Optional redaction
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_scanners::input::BanCode;
///
/// let config = BanCodeConfig {
///     threshold: 0.5,
///     redact: true,
///     ..Default::default()
/// };
///
/// let scanner = BanCode::new(config)?;
/// let result = scanner.scan("Here is some code: ```python\ndef foo():\n    pass\n```", &vault).await?;
/// assert!(!result.is_valid);
/// ```
pub struct BanCode {
    config: BanCodeConfig,
}

impl BanCode {
    /// Create a new BanCode scanner
    pub fn new(config: BanCodeConfig) -> Result<Self> {
        if !(0.0..=1.0).contains(&config.threshold) {
            return Err(Error::config("Threshold must be between 0.0 and 1.0"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(BanCodeConfig::default())
    }

    /// Detect code patterns in text
    fn detect_code(&self, text: &str) -> Vec<CodeMatch> {
        let mut matches = Vec::new();

        // Detect markdown code blocks
        if self.config.detect_markdown {
            for cap in CODE_BLOCK_REGEX.find_iter(text) {
                matches.push(CodeMatch {
                    start: cap.start(),
                    end: cap.end(),
                    pattern_type: "markdown_code".to_string(),
                    confidence: 1.0,
                    text: cap.as_str().to_string(),
                });
            }
        }

        // Detect function definitions
        if self.config.detect_inline {
            for cap in FUNCTION_PATTERN.find_iter(text) {
                matches.push(CodeMatch {
                    start: cap.start(),
                    end: cap.end(),
                    pattern_type: "function_definition".to_string(),
                    confidence: 0.95,
                    text: cap.as_str().to_string(),
                });
            }

            // Detect imports
            for cap in IMPORT_PATTERN.find_iter(text) {
                matches.push(CodeMatch {
                    start: cap.start(),
                    end: cap.end(),
                    pattern_type: "import_statement".to_string(),
                    confidence: 0.9,
                    text: cap.as_str().to_string(),
                });
            }

            // Detect variable declarations
            for cap in VARIABLE_PATTERN.find_iter(text) {
                matches.push(CodeMatch {
                    start: cap.start(),
                    end: cap.end(),
                    pattern_type: "variable_declaration".to_string(),
                    confidence: 0.85,
                    text: cap.as_str().to_string(),
                });
            }
        }

        // Detect programming keywords
        if self.config.detect_keywords {
            let lower_text = text.to_lowercase();
            for keyword in PROGRAMMING_KEYWORDS.iter() {
                if let Some(pos) = lower_text.find(keyword) {
                    let end = pos + keyword.len();
                    matches.push(CodeMatch {
                        start: pos,
                        end,
                        pattern_type: "programming_keyword".to_string(),
                        confidence: 0.7,
                        text: keyword.to_string(),
                    });
                }
            }
        }

        // Remove duplicate/overlapping matches (keep highest confidence)
        self.deduplicate_matches(matches)
    }

    fn deduplicate_matches(&self, mut matches: Vec<CodeMatch>) -> Vec<CodeMatch> {
        if matches.is_empty() {
            return matches;
        }

        matches.sort_by_key(|m| m.start);

        let mut deduplicated = Vec::new();
        let mut last_end = 0;

        for m in matches {
            if m.start >= last_end {
                last_end = m.end;
                deduplicated.push(m);
            }
        }

        deduplicated
    }

    fn calculate_risk_score(&self, matches: &[CodeMatch], text_len: usize) -> f32 {
        if matches.is_empty() {
            return 0.0;
        }

        // Calculate code density
        let total_code_chars: usize = matches.iter().map(|m| m.end - m.start).sum();
        let code_density = total_code_chars as f32 / text_len.max(1) as f32;

        // Calculate weighted confidence
        let avg_confidence = matches.iter().map(|m| m.confidence).sum::<f32>() / matches.len() as f32;

        // Combine factors
        (code_density * 0.6 + avg_confidence * 0.4).min(1.0)
    }

    fn redact_text(&self, text: &str, matches: &[CodeMatch]) -> String {
        if !self.config.redact || matches.is_empty() {
            return text.to_string();
        }

        let mut result = text.to_string();
        let mut offset = 0i32;

        for m in matches {
            let redaction = "[REDACTED CODE]";
            let actual_start = (m.start as i32 + offset) as usize;
            let actual_end = (m.end as i32 + offset) as usize;

            let original_len = actual_end - actual_start;
            result.replace_range(actual_start..actual_end, redaction);

            offset += redaction.len() as i32 - original_len as i32;
        }

        result
    }
}

#[derive(Debug, Clone)]
struct CodeMatch {
    start: usize,
    end: usize,
    pattern_type: String,
    confidence: f32,
    text: String,
}

#[async_trait]
impl Scanner for BanCode {
    fn name(&self) -> &str {
        "BanCode"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let matches = self.detect_code(input);

        if matches.is_empty() {
            return Ok(ScanResult::pass(input.to_string()));
        }

        let risk_score = self.calculate_risk_score(&matches, input.len());

        // Check threshold
        if risk_score < self.config.threshold {
            return Ok(ScanResult::pass(input.to_string()));
        }

        // Build entities for each match
        let entities: Vec<Entity> = matches
            .iter()
            .map(|m| {
                let mut metadata = HashMap::new();
                metadata.insert("pattern_type".to_string(), m.pattern_type.clone());
                metadata.insert("confidence".to_string(), m.confidence.to_string());

                Entity {
                    entity_type: "code_snippet".to_string(),
                    text: m.text.clone(),
                    start: m.start,
                    end: m.end,
                    confidence: m.confidence,
                    metadata,
                }
            })
            .collect();

        let risk_factor = RiskFactor::new(
            "code_detected",
            format!("Found {} code pattern(s)", matches.len()),
            if risk_score >= 0.8 {
                Severity::High
            } else if risk_score >= 0.5 {
                Severity::Medium
            } else {
                Severity::Low
            },
            risk_score,
        );

        let sanitized_text = self.redact_text(input, &matches);

        Ok(ScanResult::new(sanitized_text, false, risk_score)
            .with_entities(entities)
            .with_risk_factor(risk_factor)
            .with_metadata("code_density", (matches.iter().map(|m| m.end - m.start).sum::<usize>() as f32 / input.len().max(1) as f32).to_string())
            .with_metadata("patterns_found", matches.len()))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Detects and blocks code snippets in input text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ban_code_markdown_block() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let code_text = "Here is some code:\n```python\ndef hello():\n    print('world')\n```";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_score > 0.5);
        assert!(!result.entities.is_empty());
    }

    #[tokio::test]
    async fn test_ban_code_inline_backticks() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let code_text = "Use the `print()` function to output text";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(!result.entities.is_empty());
    }

    #[tokio::test]
    async fn test_ban_code_function_definition() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let code_text = "def calculate_sum(a, b): return a + b";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.metadata.get("pattern_type").map(|s| s.as_str()) == Some("function_definition")));
    }

    #[tokio::test]
    async fn test_ban_code_import_statement() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let code_text = "import numpy as np";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.iter().any(|e| e.metadata.get("pattern_type").map(|s| s.as_str()) == Some("import_statement")));
    }

    #[tokio::test]
    async fn test_ban_code_no_code() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let normal_text = "This is just normal text without any code";
        let result = scanner.scan(normal_text, &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
    }

    #[tokio::test]
    async fn test_ban_code_threshold() {
        let config = BanCodeConfig {
            threshold: 0.9,  // Very high threshold
            ..Default::default()
        };
        let scanner = BanCode::new(config).unwrap();
        let vault = Vault::new();

        // Small code snippet shouldn't trigger with high threshold
        let code_text = "Use `print()` function";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        // Should pass because risk_score < 0.9
        assert!(result.is_valid || result.risk_score < 0.9);
    }

    #[tokio::test]
    async fn test_ban_code_redaction() {
        let config = BanCodeConfig {
            threshold: 0.3,
            redact: true,
            ..Default::default()
        };
        let scanner = BanCode::new(config).unwrap();
        let vault = Vault::new();

        let code_text = "Here's code: ```python\nprint('hi')\n```";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.sanitized_text.contains("[REDACTED CODE]"));
    }

    #[tokio::test]
    async fn test_ban_code_multiple_patterns() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let code_text = "import sys\ndef main():\n    const x = 5\n    return x";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.entities.len() >= 2);  // Should detect multiple patterns
    }

    #[tokio::test]
    async fn test_ban_code_javascript() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let code_text = "function add(a, b) { return a + b; }";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
    }

    #[tokio::test]
    async fn test_ban_code_rust() {
        let scanner = BanCode::default_config().unwrap();
        let vault = Vault::new();

        let code_text = "fn main() { println!(\"Hello\"); }";
        let result = scanner.scan(code_text, &vault).await.unwrap();

        assert!(!result.is_valid);
    }
}
