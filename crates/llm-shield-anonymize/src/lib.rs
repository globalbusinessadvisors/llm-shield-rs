//! LLM Shield Anonymization - PII Detection and Anonymization

pub mod anonymizer;
pub mod config;
// pub mod deanonymizer; // TODO: Phase 9B
pub mod detector;
pub mod placeholder;
pub mod replacer;
pub mod types;
pub mod vault;

// Re-exports
pub use anonymizer::{Anonymizer, AnonymizeResult};
pub use config::{AnonymizerConfig, PlaceholderFormat};
pub use detector::EntityDetector;
pub use placeholder::PlaceholderGenerator;
pub use replacer::replace_entities;
pub use types::{EntityMatch, EntityMapping, EntityType};

/// Result type for anonymization operations
pub type Result<T> = std::result::Result<T, AnonymizationError>;

/// Errors that can occur during anonymization
#[derive(Debug, thiserror::Error)]
pub enum AnonymizationError {
    #[error("Empty input text")]
    EmptyInput,

    #[error("Invalid entity range: {0}")]
    InvalidRange(String),

    #[error("Detector error: {0}")]
    DetectorError(String),

    #[error("Vault error: {0}")]
    VaultError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Placeholder generation failed: {0}")]
    PlaceholderError(String),
}
