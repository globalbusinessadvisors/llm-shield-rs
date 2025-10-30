//! Tokenizer Support
//!
//! Handles text tokenization for ML models.

use llm_shield_core::Error;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::Tokenizer;

/// Tokenizer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenizerConfig {
    /// Path to tokenizer JSON file
    pub tokenizer_path: PathBuf,

    /// Maximum sequence length
    pub max_length: usize,

    /// Padding token
    pub padding: Option<String>,

    /// Truncation enabled
    pub truncation: bool,
}

impl Default for TokenizerConfig {
    fn default() -> Self {
        Self {
            tokenizer_path: PathBuf::new(),
            max_length: 512,
            padding: Some("[PAD]".to_string()),
            truncation: true,
        }
    }
}

/// Tokenizer wrapper
pub struct TokenizerWrapper {
    tokenizer: Arc<Tokenizer>,
    config: TokenizerConfig,
}

impl TokenizerWrapper {
    /// Create a new tokenizer wrapper
    pub fn new(config: TokenizerConfig) -> crate::Result<Self> {
        let tokenizer = Tokenizer::from_file(&config.tokenizer_path)
            .map_err(|e| Error::model(format!("Failed to load tokenizer: {}", e)))?;

        Ok(Self {
            tokenizer: Arc::new(tokenizer),
            config,
        })
    }

    /// Create from a tokenizer instance
    pub fn from_tokenizer(tokenizer: Tokenizer, config: TokenizerConfig) -> Self {
        Self {
            tokenizer: Arc::new(tokenizer),
            config,
        }
    }

    /// Encode text to token IDs
    pub fn encode(&self, text: &str) -> crate::Result<Vec<u32>> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| Error::model(format!("Tokenization failed: {}", e)))?;

        let mut ids = encoding.get_ids().to_vec();

        // Apply truncation
        if self.config.truncation && ids.len() > self.config.max_length {
            ids.truncate(self.config.max_length);
        }

        // Apply padding
        while ids.len() < self.config.max_length {
            ids.push(0); // Assuming 0 is padding token ID
        }

        Ok(ids)
    }

    /// Encode text with attention mask
    pub fn encode_with_attention(&self, text: &str) -> crate::Result<(Vec<u32>, Vec<u32>)> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| Error::model(format!("Tokenization failed: {}", e)))?;

        let mut ids = encoding.get_ids().to_vec();
        let original_len = ids.len();

        // Apply truncation
        if self.config.truncation && ids.len() > self.config.max_length {
            ids.truncate(self.config.max_length);
        }

        // Create attention mask (1 for real tokens, 0 for padding)
        let mut attention_mask = vec![1u32; ids.len()];

        // Apply padding
        while ids.len() < self.config.max_length {
            ids.push(0);
            attention_mask.push(0);
        }

        Ok((ids, attention_mask))
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(true)
    }

    /// Get max length
    pub fn max_length(&self) -> usize {
        self.config.max_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_config_default() {
        let config = TokenizerConfig::default();
        assert_eq!(config.max_length, 512);
        assert_eq!(config.padding, Some("[PAD]".to_string()));
        assert!(config.truncation);
    }
}
