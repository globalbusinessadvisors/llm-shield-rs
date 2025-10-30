//! TokenLimit Scanner
//!
//! Converted from llm_guard/input_scanners/token_limit.py
//!
//! Enforces maximum token limits on input text.

use llm_shield_core::{
    async_trait, Error, Result, RiskFactor, ScanResult, Scanner, ScannerType, Severity, Vault,
};
use serde::{Deserialize, Serialize};

/// TokenLimit scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenLimitConfig {
    /// Maximum allowed tokens
    pub limit: usize,

    /// Encoding to use (cl100k_base for GPT-4, p50k_base for GPT-3, etc.)
    pub encoding_name: String,
}

impl Default for TokenLimitConfig {
    fn default() -> Self {
        Self {
            limit: 4096,
            encoding_name: "cl100k_base".to_string(),
        }
    }
}

/// TokenLimit scanner
///
/// ## Enterprise Features
///
/// - Multiple tokenizer support
/// - Configurable limits
/// - Accurate token counting
///
/// ## Implementation Note
///
/// This is a simplified version. In production, integrate tiktoken-rs
/// for accurate OpenAI-compatible token counting.
pub struct TokenLimit {
    config: TokenLimitConfig,
}

impl TokenLimit {
    /// Create a new TokenLimit scanner
    pub fn new(config: TokenLimitConfig) -> Result<Self> {
        if config.limit == 0 {
            return Err(Error::config("Token limit must be greater than 0"));
        }

        Ok(Self { config })
    }

    /// Create with default configuration
    pub fn with_limit(limit: usize) -> Result<Self> {
        Self::new(TokenLimitConfig {
            limit,
            ..Default::default()
        })
    }

    /// Count tokens (simplified - use tiktoken-rs in production)
    fn count_tokens(&self, text: &str) -> usize {
        // Simplified token counting
        // Real implementation would use tiktoken-rs
        // Approximation: ~4 characters per token for English text
        let char_count = text.len();
        (char_count as f32 / 4.0).ceil() as usize
    }
}

#[async_trait]
impl Scanner for TokenLimit {
    fn name(&self) -> &str {
        "TokenLimit"
    }

    async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
        let token_count = self.count_tokens(input);

        if token_count <= self.config.limit {
            return Ok(ScanResult::pass(input.to_string())
                .with_metadata("token_count", token_count)
                .with_metadata("limit", self.config.limit));
        }

        let risk_score = (token_count as f32 / self.config.limit as f32).min(1.0);

        let risk_factor = RiskFactor::new(
            "token_limit_exceeded",
            format!(
                "Token count {} exceeds limit {}",
                token_count, self.config.limit
            ),
            Severity::Medium,
            risk_score,
        );

        Ok(ScanResult::fail(input.to_string(), risk_score)
            .with_risk_factor(risk_factor)
            .with_metadata("token_count", token_count)
            .with_metadata("limit", self.config.limit)
            .with_metadata("overflow", token_count - self.config.limit))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    fn description(&self) -> &str {
        "Enforces maximum token limits on input text"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_limit_within_limit() {
        let scanner = TokenLimit::with_limit(100).unwrap();
        let vault = Vault::new();

        let result = scanner.scan("Short text", &vault).await.unwrap();

        assert!(result.is_valid);
        assert_eq!(result.risk_score, 0.0);
    }

    #[tokio::test]
    async fn test_token_limit_exceeds() {
        let scanner = TokenLimit::with_limit(10).unwrap();
        let vault = Vault::new();

        let long_text = "This is a very long text that will exceed the token limit ".repeat(5);
        let result = scanner.scan(&long_text, &vault).await.unwrap();

        assert!(!result.is_valid);
        assert!(result.risk_score > 0.0);
    }
}
