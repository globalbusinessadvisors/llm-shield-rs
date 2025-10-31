//! Error types for cloud integrations.
//!
//! Provides unified error handling across all cloud providers (AWS, GCP, Azure).

use thiserror::Error;

/// Unified cloud error type for all cloud operations.
#[derive(Error, Debug)]
pub enum CloudError {
    // Initialization errors
    #[error("Failed to initialize cloud client: {0}")]
    ClientInit(String),

    #[error("Cloud provider '{0}' not enabled. Enable with feature flag (cloud-aws, cloud-gcp, cloud-azure)")]
    ProviderNotEnabled(String),

    // Secret management errors
    #[error("Failed to fetch secret '{name}': {source}")]
    SecretFetch {
        name: String,
        source: String,
    },

    #[error("Failed to create secret '{name}': {source}")]
    SecretCreate {
        name: String,
        source: String,
    },

    #[error("Failed to update secret '{name}': {source}")]
    SecretUpdate {
        name: String,
        source: String,
    },

    #[error("Failed to delete secret '{name}': {source}")]
    SecretDelete {
        name: String,
        source: String,
    },

    #[error("Invalid secret format for '{name}': {reason}")]
    SecretFormat {
        name: String,
        reason: String,
    },

    #[error("Failed to list secrets: {0}")]
    SecretList(String),

    #[error("Secret not found: '{0}'")]
    SecretNotFound(String),

    // Storage errors
    #[error("Failed to fetch object '{key}' from storage: {source}")]
    StorageFetch {
        key: String,
        source: String,
    },

    #[error("Failed to read storage object '{key}': {source}")]
    StorageRead {
        key: String,
        source: String,
    },

    #[error("Failed to put object '{key}' to storage: {source}")]
    StoragePut {
        key: String,
        source: String,
    },

    #[error("Failed to delete object '{key}' from storage: {source}")]
    StorageDelete {
        key: String,
        source: String,
    },

    #[error("Failed to list storage objects with prefix '{prefix}': {source}")]
    StorageList {
        prefix: String,
        source: String,
    },

    #[error("Storage object not found: '{0}'")]
    StorageObjectNotFound(String),

    // Observability errors
    #[error("Failed to export metrics: {0}")]
    MetricsExport(String),

    #[error("Failed to write log entry: {0}")]
    LogWrite(String),

    #[error("Failed to create trace span: {0}")]
    TraceSpanCreate(String),

    #[error("Failed to export trace: {0}")]
    TraceExport(String),

    // Authentication errors
    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    // Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Missing required configuration: {0}")]
    MissingConfig(String),

    #[error("Invalid configuration value for '{key}': {reason}")]
    InvalidConfig {
        key: String,
        reason: String,
    },

    // Network errors
    #[error("Network error: {0}")]
    Network(String),

    #[error("Connection timeout: {0}")]
    Timeout(String),

    // Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    // Generic errors
    #[error("Cloud operation failed: {0}")]
    OperationFailed(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for cloud operations.
pub type Result<T> = std::result::Result<T, CloudError>;

impl CloudError {
    /// Creates a new `SecretFetch` error.
    pub fn secret_fetch(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self::SecretFetch {
            name: name.into(),
            source: source.into(),
        }
    }

    /// Creates a new `SecretCreate` error.
    pub fn secret_create(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self::SecretCreate {
            name: name.into(),
            source: source.into(),
        }
    }

    /// Creates a new `SecretUpdate` error.
    pub fn secret_update(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self::SecretUpdate {
            name: name.into(),
            source: source.into(),
        }
    }

    /// Creates a new `StorageFetch` error.
    pub fn storage_fetch(key: impl Into<String>, source: impl Into<String>) -> Self {
        Self::StorageFetch {
            key: key.into(),
            source: source.into(),
        }
    }

    /// Creates a new `StoragePut` error.
    pub fn storage_put(key: impl Into<String>, source: impl Into<String>) -> Self {
        Self::StoragePut {
            key: key.into(),
            source: source.into(),
        }
    }
}

// Conversion from anyhow::Error
impl From<anyhow::Error> for CloudError {
    fn from(err: anyhow::Error) -> Self {
        CloudError::Internal(err.to_string())
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for CloudError {
    fn from(err: serde_json::Error) -> Self {
        CloudError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CloudError::secret_fetch("my-secret", "connection refused");
        assert_eq!(
            err.to_string(),
            "Failed to fetch secret 'my-secret': connection refused"
        );
    }

    #[test]
    fn test_secret_not_found() {
        let err = CloudError::SecretNotFound("test-secret".to_string());
        assert!(err.to_string().contains("Secret not found"));
    }

    #[test]
    fn test_provider_not_enabled() {
        let err = CloudError::ProviderNotEnabled("AWS".to_string());
        assert!(err.to_string().contains("not enabled"));
        assert!(err.to_string().contains("feature flag"));
    }

    #[test]
    fn test_from_serde_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{invalid}")
            .unwrap_err();
        let cloud_err: CloudError = json_err.into();
        assert!(matches!(cloud_err, CloudError::Serialization(_)));
    }
}
