//! Model Loading and Caching
//!
//! Handles ONNX model loading, caching, and management.

use llm_shield_core::Error;
use ort::{Session, SessionBuilder};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model type
    pub model_type: ModelType,

    /// Path to ONNX model file
    pub model_path: PathBuf,

    /// Number of threads for inference
    pub num_threads: Option<usize>,

    /// Use GPU if available
    pub use_gpu: bool,
}

/// Supported model types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// Prompt injection detection (DeBERTa)
    PromptInjection,

    /// Toxicity classification (RoBERTa)
    Toxicity,

    /// Sentiment analysis
    Sentiment,
}

impl ModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::PromptInjection => "prompt_injection",
            ModelType::Toxicity => "toxicity",
            ModelType::Sentiment => "sentiment",
        }
    }
}

/// Model loader with caching
pub struct ModelLoader {
    cache_dir: PathBuf,
}

impl ModelLoader {
    /// Create a new model loader
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Load a model from configuration
    pub fn load_model(&self, config: &ModelConfig) -> crate::Result<Arc<Session>> {
        let model_path = &config.model_path;

        if !model_path.exists() {
            return Err(Error::model(format!(
                "Model file not found: {}",
                model_path.display()
            )));
        }

        // Build session with configuration
        let mut builder = SessionBuilder::new()
            .map_err(|e| Error::model(format!("Failed to create session builder: {}", e)))?;

        if let Some(threads) = config.num_threads {
            builder = builder
                .with_intra_threads(threads)
                .map_err(|e| Error::model(format!("Failed to set thread count: {}", e)))?;
        }

        let session = builder
            .commit_from_file(&model_path)
            .map_err(|e| Error::model(format!("Failed to load model: {}", e)))?;

        Ok(Arc::new(session))
    }

    /// Get default cache directory
    pub fn default_cache_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(".cache/llm-shield/models")
    }
}

impl Default for ModelLoader {
    fn default() -> Self {
        Self::new(Self::default_cache_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_type_as_str() {
        assert_eq!(ModelType::PromptInjection.as_str(), "prompt_injection");
        assert_eq!(ModelType::Toxicity.as_str(), "toxicity");
        assert_eq!(ModelType::Sentiment.as_str(), "sentiment");
    }

    #[test]
    fn test_model_loader_creation() {
        let loader = ModelLoader::default();
        assert!(loader.cache_dir.to_string_lossy().contains(".cache/llm-shield/models"));
    }
}
