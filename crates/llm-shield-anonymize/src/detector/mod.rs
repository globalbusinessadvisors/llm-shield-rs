//! Entity Detection Module
//!
//! Detects PII entities in text using regex patterns, validation algorithms,
//! and ML-based Named Entity Recognition (NER).

pub mod ner;
pub mod patterns;
pub mod regex;
pub mod validators;

use crate::types::{EntityMatch, EntityType};
use async_trait::async_trait;
use llm_shield_core::Result;

// Re-exports
pub use ner::{BioTag, NerConfig, NerDetector};
pub use regex::RegexDetector;

/// Trait for entity detection implementations
#[async_trait]
pub trait EntityDetector: Send + Sync {
    /// Detect entities in the given text
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>>;
}
