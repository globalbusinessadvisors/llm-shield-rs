//! Error types for LLM Shield
//!
//! ## SPARC Specification
//!
//! Comprehensive error handling following enterprise patterns:
//! - Specific error variants for different failure modes
//! - Rich context information
//! - Integration with `anyhow` for flexibility
//! - Proper error chaining

use std::fmt;
use thiserror::Error;

/// Result type alias for LLM Shield operations
pub type Result<T> = std::result::Result<T, Error>;

/// Core error type for LLM Shield operations
///
/// ## Design Principles
///
/// 1. **Specific Variants**: Each error type has a specific variant
/// 2. **Context**: All errors include contextual information
/// 3. **Source Chaining**: Errors properly chain their sources
/// 4. **Display**: Human-readable error messages
#[derive(Debug, Error)]
pub enum Error {
    /// Scanner-specific errors
    #[error("Scanner error in {scanner}: {message}")]
    Scanner {
        scanner: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Model loading or inference errors
    #[error("Model error: {0}")]
    Model(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid input data
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Vault errors (state management)
    #[error("Vault error: {0}")]
    Vault(String),

    /// Timeout errors
    #[error("Operation timed out after {0}ms")]
    Timeout(u64),

    /// Resource exhaustion
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Internal errors (should not happen in production)
    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    /// Create a scanner error with context
    pub fn scanner<S: Into<String>, M: Into<String>>(scanner: S, message: M) -> Self {
        Self::Scanner {
            scanner: scanner.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a scanner error with source
    pub fn scanner_with_source<S: Into<String>, M: Into<String>>(
        scanner: S,
        message: M,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::Scanner {
            scanner: scanner.into(),
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a model error
    pub fn model<S: Into<String>>(message: S) -> Self {
        Self::Model(message.into())
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config(message.into())
    }

    /// Create an invalid input error
    pub fn invalid_input<S: Into<String>>(message: S) -> Self {
        Self::InvalidInput(message.into())
    }

    /// Create a vault error
    pub fn vault<S: Into<String>>(message: S) -> Self {
        Self::Vault(message.into())
    }

    /// Create a timeout error
    pub fn timeout(duration_ms: u64) -> Self {
        Self::Timeout(duration_ms)
    }

    /// Create a resource exhausted error
    pub fn resource_exhausted<S: Into<String>>(resource: S) -> Self {
        Self::ResourceExhausted(resource.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(message: S) -> Self {
        Self::Internal(message.into())
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Timeout(_) | Error::ResourceExhausted(_) | Error::Io(_)
        )
    }

    /// Get error category for metrics
    pub fn category(&self) -> &'static str {
        match self {
            Error::Scanner { .. } => "scanner",
            Error::Model(_) => "model",
            Error::Config(_) => "config",
            Error::InvalidInput(_) => "invalid_input",
            Error::Io(_) => "io",
            Error::Serialization(_) => "serialization",
            Error::Vault(_) => "vault",
            Error::Timeout(_) => "timeout",
            Error::ResourceExhausted(_) => "resource_exhausted",
            Error::Internal(_) => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::scanner("test_scanner", "test message");
        assert!(matches!(err, Error::Scanner { .. }));
        assert_eq!(err.category(), "scanner");
    }

    #[test]
    fn test_error_retryable() {
        assert!(Error::timeout(5000).is_retryable());
        assert!(!Error::config("bad config").is_retryable());
    }

    #[test]
    fn test_error_display() {
        let err = Error::scanner("ban_substrings", "pattern not found");
        let msg = format!("{}", err);
        assert!(msg.contains("ban_substrings"));
        assert!(msg.contains("pattern not found"));
    }
}
