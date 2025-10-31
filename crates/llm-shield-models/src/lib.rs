//! ML Model Infrastructure for LLM Shield
//!
//! This crate provides infrastructure for loading and running ONNX models
//! for ML-based security scanners.

pub mod model_loader;
pub mod tokenizer;
pub mod inference;
pub mod registry;
pub mod cache;
pub mod types;

pub use model_loader::{ModelLoader, ModelConfig, ModelType};
pub use tokenizer::{TokenizerWrapper, TokenizerConfig, Encoding};
pub use inference::{InferenceEngine, InferenceResult, PostProcessing};
pub use registry::{ModelRegistry, ModelTask, ModelVariant, ModelMetadata};
pub use cache::{ResultCache, CacheConfig, CacheStats};
pub use types::{
    MLConfig, CacheSettings, HybridMode, DetectionMethod, InferenceMetrics,
};

use llm_shield_core::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
