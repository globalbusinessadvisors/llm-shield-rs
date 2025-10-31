//! Entity Detection Module
//!
//! Detects PII entities in text using regex patterns and validation algorithms.

pub mod patterns;
pub mod regex;
pub mod validators;

use crate::types::{EntityMatch, EntityType};
use async_trait::async_trait;
use llm_shield_core::Result;

/// Trait for entity detection implementations
#[async_trait]
pub trait EntityDetector: Send + Sync {
    /// Detect entities in the given text
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>>;
}
