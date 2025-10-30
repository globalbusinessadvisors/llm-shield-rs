//! ML Model Infrastructure for LLM Shield
//!
//! This crate provides infrastructure for loading and running ONNX models
//! for ML-based security scanners.

pub mod model_loader;
pub mod tokenizer;
pub mod inference;

pub use model_loader::{ModelLoader, ModelConfig, ModelType};
pub use tokenizer::{TokenizerWrapper, TokenizerConfig};
pub use inference::{InferenceEngine, InferenceResult};

use llm_shield_core::Error;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
