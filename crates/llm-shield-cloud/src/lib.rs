//! Cloud abstraction layer for LLM Shield.
//!
//! This crate provides unified traits and types for interacting with cloud services
//! across AWS, GCP, and Azure. It enables LLM Shield to leverage cloud-native features
//! for secrets management, object storage, metrics, logging, and distributed tracing.
//!
//! # Architecture
//!
//! The crate defines trait-based abstractions for common cloud operations:
//!
//! - **Secret Management**: [`CloudSecretManager`] for AWS Secrets Manager, GCP Secret Manager, Azure Key Vault
//! - **Object Storage**: [`CloudStorage`] for AWS S3, GCP Cloud Storage, Azure Blob Storage
//! - **Observability**: [`CloudMetrics`], [`CloudLogger`], [`CloudTracer`] for cloud-native monitoring
//!
//! # Features
//!
//! This crate provides the core abstractions. Concrete implementations are provided by:
//!
//! - `llm-shield-cloud-aws` - AWS integrations (enable with `cloud-aws` feature)
//! - `llm-shield-cloud-gcp` - GCP integrations (enable with `cloud-gcp` feature)
//! - `llm-shield-cloud-azure` - Azure integrations (enable with `cloud-azure` feature)
//!
//! # Example
//!
//! ```rust,no_run
//! use llm_shield_cloud::{CloudSecretManager, SecretValue, Result};
//!
//! async fn load_api_keys(
//!     secret_manager: &dyn CloudSecretManager
//! ) -> Result<Vec<String>> {
//!     // Fetch API keys from cloud secret manager
//!     let secret = secret_manager.get_secret("llm-shield/api-keys").await?;
//!
//!     // Parse the secret value
//!     let api_keys: Vec<String> = serde_json::from_str(secret.as_string())?;
//!
//!     Ok(api_keys)
//! }
//! ```
//!
//! # Configuration
//!
//! Cloud integrations are configured via [`CloudConfig`]:
//!
//! ```yaml
//! cloud:
//!   provider: aws
//!   aws:
//!     region: us-east-1
//!     secrets_manager:
//!       enabled: true
//!       cache_ttl_seconds: 300
//!     s3:
//!       bucket: llm-shield-models
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

// Re-export core dependencies
pub use async_trait::async_trait;

// Module declarations
pub mod config;
pub mod error;
pub mod observability;
pub mod secrets;
pub mod storage;

// Re-export commonly used types
pub use config::{
    AzureConfig, AwsConfig, CloudConfig, CloudProvider, GcpConfig,
};
pub use error::{CloudError, Result};
pub use observability::{
    CloudLogger, CloudMetrics, CloudTracer, LogEntry, LogLevel, Metric, Span,
};
pub use secrets::{CloudSecretManager, SecretCache, SecretMetadata, SecretValue};
pub use storage::{
    CloudStorage, GetObjectOptions, ObjectMetadata, PutObjectOptions,
};

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name.
pub const LIB_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(LIB_NAME, "llm-shield-cloud");
    }
}
