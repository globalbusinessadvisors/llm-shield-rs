//! AWS cloud integrations for LLM Shield.
//!
//! This crate provides AWS-specific implementations of the cloud abstraction traits
//! defined in `llm-shield-cloud`:
//!
//! - **Secrets Management**: AWS Secrets Manager via `AwsSecretsManager`
//! - **Object Storage**: AWS S3 via `AwsS3Storage`
//! - **Metrics**: CloudWatch Metrics via `CloudWatchMetrics`
//! - **Logging**: CloudWatch Logs via `CloudWatchLogger`
//!
//! # Features
//!
//! - Automatic credential discovery (environment → file → IAM role → IRSA)
//! - Built-in caching for secrets (TTL-based)
//! - Multipart uploads for large S3 objects (>5MB)
//! - Batched metrics and log export for efficiency
//! - Full support for AWS SDK retry and timeout policies
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │   LLM Shield Application            │
//! └─────────────────────────────────────┘
//!                 │
//!                 ▼
//! ┌─────────────────────────────────────┐
//! │   llm-shield-cloud (traits)         │
//! │   - CloudSecretManager              │
//! │   - CloudStorage                    │
//! │   - CloudMetrics/Logger             │
//! └─────────────────────────────────────┘
//!                 │
//!                 ▼
//! ┌─────────────────────────────────────┐
//! │   llm-shield-cloud-aws (impl)       │
//! │   - AwsSecretsManager               │
//! │   - AwsS3Storage                    │
//! │   - CloudWatchMetrics/Logger        │
//! └─────────────────────────────────────┘
//!                 │
//!                 ▼
//! ┌─────────────────────────────────────┐
//! │   AWS Services                      │
//! │   - Secrets Manager                 │
//! │   - S3                              │
//! │   - CloudWatch                      │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Usage Examples
//!
//! ## Secret Management
//!
//! ```no_run
//! use llm_shield_cloud_aws::AwsSecretsManager;
//! use llm_shield_cloud::CloudSecretManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize with default configuration (uses AWS credential chain)
//!     let secrets = AwsSecretsManager::new().await?;
//!
//!     // Fetch a secret (automatically cached for 5 minutes)
//!     let api_key = secrets.get_secret("llm-shield/openai-api-key").await?;
//!     println!("API Key: {}", api_key.as_string());
//!
//!     // List all secrets
//!     let secret_names = secrets.list_secrets().await?;
//!     println!("Found {} secrets", secret_names.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Object Storage
//!
//! ```no_run
//! use llm_shield_cloud_aws::AwsS3Storage;
//! use llm_shield_cloud::{CloudStorage, PutObjectOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let storage = AwsS3Storage::new("llm-shield-models").await?;
//!
//!     // Upload a model (automatically uses multipart for files >5MB)
//!     let model_data = tokio::fs::read("toxicity-model.onnx").await?;
//!     storage.put_object("models/toxicity.onnx", &model_data).await?;
//!
//!     // Upload with options
//!     let options = PutObjectOptions {
//!         content_type: Some("application/octet-stream".to_string()),
//!         storage_class: Some("INTELLIGENT_TIERING".to_string()),
//!         encryption: Some("AES256".to_string()),
//!         ..Default::default()
//!     };
//!     storage.put_object_with_options("models/model.onnx", &model_data, &options).await?;
//!
//!     // Download and verify
//!     let downloaded = storage.get_object("models/toxicity.onnx").await?;
//!     assert_eq!(model_data, downloaded);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Metrics
//!
//! ```no_run
//! use llm_shield_cloud_aws::CloudWatchMetrics;
//! use llm_shield_cloud::{CloudMetrics, Metric};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let metrics = CloudWatchMetrics::new("LLMShield").await?;
//!
//!     let mut dimensions = HashMap::new();
//!     dimensions.insert("Environment".to_string(), "Production".to_string());
//!     dimensions.insert("Scanner".to_string(), "Toxicity".to_string());
//!
//!     let metric = Metric {
//!         name: "ScanDuration".to_string(),
//!         value: 123.45,
//!         timestamp: std::time::SystemTime::now()
//!             .duration_since(std::time::UNIX_EPOCH)?
//!             .as_secs(),
//!         dimensions,
//!         unit: Some("Milliseconds".to_string()),
//!     };
//!
//!     metrics.export_metric(&metric).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Logging
//!
//! ```no_run
//! use llm_shield_cloud_aws::CloudWatchLogger;
//! use llm_shield_cloud::{CloudLogger, LogLevel, LogEntry};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let logger = CloudWatchLogger::new(
//!         "/llm-shield/api",
//!         "production-instance-1"
//!     ).await?;
//!
//!     // Simple logging
//!     logger.log("API server started", LogLevel::Info).await?;
//!
//!     // Structured logging
//!     let mut labels = HashMap::new();
//!     labels.insert("request_id".to_string(), "req-123".to_string());
//!     labels.insert("user_id".to_string(), "user-456".to_string());
//!
//!     let entry = LogEntry {
//!         timestamp: std::time::SystemTime::now(),
//!         level: LogLevel::Info,
//!         message: "Request processed successfully".to_string(),
//!         labels,
//!         trace_id: Some("trace-789".to_string()),
//!         span_id: Some("span-012".to_string()),
//!     };
//!
//!     logger.log_structured(&entry).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # AWS Credentials
//!
//! This crate uses the AWS SDK's default credential provider chain:
//!
//! 1. **Environment variables**: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`
//! 2. **AWS credentials file**: `~/.aws/credentials`
//! 3. **ECS container credentials**: IAM role for ECS tasks
//! 4. **EC2 instance profile**: IAM role for EC2 instances
//! 5. **EKS pod identity**: IAM Roles for Service Accounts (IRSA)
//!
//! # IAM Permissions
//!
//! Required IAM permissions are documented in `iam-policies/` directory:
//!
//! - `secrets-manager-policy.json`: Secrets Manager permissions
//! - `s3-policy.json`: S3 bucket access permissions
//! - `cloudwatch-policy.json`: CloudWatch metrics and logs permissions
//!
//! # Configuration
//!
//! Configure AWS integrations via `CloudConfig`:
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
//!       models_prefix: models/
//!       results_prefix: scan-results/
//!     cloudwatch:
//!       enabled: true
//!       namespace: LLMShield
//!       log_group: /llm-shield/api
//!       log_stream: production
//! ```
//!
//! # Performance
//!
//! - **Secret caching**: >90% cache hit rate reduces API calls
//! - **Multipart uploads**: Automatically used for objects >5MB
//! - **Batch export**: Metrics and logs are batched for efficiency
//! - **Async operations**: All I/O is fully asynchronous with tokio
//!
//! # Error Handling
//!
//! All operations return `Result<T, CloudError>` from the `llm-shield-cloud` crate:
//!
//! ```rust
//! use llm_shield_cloud::{CloudError, Result};
//! use llm_shield_cloud_aws::AwsSecretsManager;
//!
//! async fn fetch_secret(name: &str) -> Result<String> {
//!     let secrets = AwsSecretsManager::new().await?;
//!
//!     match secrets.get_secret(name).await {
//!         Ok(value) => Ok(value.as_string().to_string()),
//!         Err(CloudError::SecretNotFound(name)) => {
//!             eprintln!("Secret '{}' not found", name);
//!             Err(CloudError::SecretNotFound(name))
//!         }
//!         Err(e) => {
//!             eprintln!("Failed to fetch secret: {}", e);
//!             Err(e)
//!         }
//!     }
//! }
//! ```
//!
//! # Testing
//!
//! Run unit tests:
//!
//! ```bash
//! cargo test -p llm-shield-cloud-aws
//! ```
//!
//! Run integration tests (requires AWS credentials):
//!
//! ```bash
//! cargo test -p llm-shield-cloud-aws --test integration -- --ignored
//! ```
//!
//! # License
//!
//! MIT OR Apache-2.0

pub mod observability;
pub mod secrets;
pub mod storage;

// Re-export main types
pub use observability::{CloudWatchLogger, CloudWatchMetrics};
pub use secrets::AwsSecretsManager;
pub use storage::AwsS3Storage;

// Re-export cloud abstractions for convenience
pub use llm_shield_cloud::{
    CloudError, CloudLogger, CloudMetrics, CloudSecretManager, CloudStorage, GetObjectOptions,
    LogEntry, LogLevel, Metric, ObjectMetadata, PutObjectOptions, Result, SecretMetadata,
    SecretValue,
};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate name
pub const LIB_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(LIB_NAME, "llm-shield-cloud-aws");
    }
}
