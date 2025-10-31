//! GCP cloud integrations for LLM Shield.
//!
//! This crate provides GCP-specific implementations of the cloud abstraction traits
//! defined in `llm-shield-cloud`:
//!
//! - **Secret Management**: GCP Secret Manager via `GcpSecretManager`
//! - **Object Storage**: GCP Cloud Storage via `GcpCloudStorage`
//! - **Metrics**: Cloud Monitoring via `GcpCloudMonitoring`
//! - **Logging**: Cloud Logging via `GcpCloudLogging`
//!
//! # Features
//!
//! - Automatic credential discovery (ADC, service account, workload identity)
//! - Built-in caching for secrets (TTL-based)
//! - Resumable uploads for large Cloud Storage objects (>5MB)
//! - Batched metrics and log export for efficiency
//! - Full support for GCP retry and timeout policies
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
//! │   llm-shield-cloud-gcp (impl)       │
//! │   - GcpSecretManager                │
//! │   - GcpCloudStorage                 │
//! │   - GcpCloudMonitoring/Logging      │
//! └─────────────────────────────────────┘
//!                 │
//!                 ▼
//! ┌─────────────────────────────────────┐
//! │   GCP Services                      │
//! │   - Secret Manager                  │
//! │   - Cloud Storage                   │
//! │   - Cloud Monitoring/Logging        │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Usage Examples
//!
//! ## Secret Management
//!
//! ```no_run
//! use llm_shield_cloud_gcp::GcpSecretManager;
//! use llm_shield_cloud::CloudSecretManager;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize with project ID
//!     let secrets = GcpSecretManager::new("my-project-id").await?;
//!
//!     // Fetch a secret (automatically cached for 5 minutes)
//!     let api_key = secrets.get_secret("openai-api-key").await?;
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
//! use llm_shield_cloud_gcp::GcpCloudStorage;
//! use llm_shield_cloud::{CloudStorage, PutObjectOptions};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let storage = GcpCloudStorage::new("llm-shield-models").await?;
//!
//!     // Upload a model (automatically uses resumable for files >5MB)
//!     let model_data = tokio::fs::read("toxicity-model.onnx").await?;
//!     storage.put_object("models/toxicity.onnx", &model_data).await?;
//!
//!     // Upload with options
//!     let options = PutObjectOptions {
//!         content_type: Some("application/octet-stream".to_string()),
//!         storage_class: Some("STANDARD".to_string()),
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
//! use llm_shield_cloud_gcp::GcpCloudMonitoring;
//! use llm_shield_cloud::{CloudMetrics, Metric};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let metrics = GcpCloudMonitoring::new("my-project-id").await?;
//!
//!     let mut dimensions = HashMap::new();
//!     dimensions.insert("environment".to_string(), "production".to_string());
//!     dimensions.insert("scanner".to_string(), "toxicity".to_string());
//!
//!     let metric = Metric {
//!         name: "scan_duration".to_string(),
//!         value: 123.45,
//!         timestamp: std::time::SystemTime::now()
//!             .duration_since(std::time::UNIX_EPOCH)?
//!             .as_secs(),
//!         dimensions,
//!         unit: Some("ms".to_string()),
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
//! use llm_shield_cloud_gcp::GcpCloudLogging;
//! use llm_shield_cloud::{CloudLogger, LogLevel, LogEntry};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let logger = GcpCloudLogging::new(
//!         "my-project-id",
//!         "llm-shield-api"
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
//! # GCP Credentials
//!
//! This crate uses Application Default Credentials (ADC):
//!
//! 1. **GOOGLE_APPLICATION_CREDENTIALS** environment variable pointing to a service account key file
//! 2. **gcloud auth application-default login** credentials
//! 3. **Service account** attached to GCE instance
//! 4. **Workload Identity** for GKE pods
//!
//! # IAM Permissions
//!
//! Required IAM permissions are documented in `iam-roles/` directory:
//!
//! - `secret-manager-role.yaml`: Secret Manager permissions
//! - `storage-role.yaml`: Cloud Storage permissions
//! - `monitoring-role.yaml`: Cloud Monitoring and Logging permissions
//!
//! # Configuration
//!
//! Configure GCP integrations via `CloudConfig`:
//!
//! ```yaml
//! cloud:
//!   provider: gcp
//!   gcp:
//!     project_id: my-project-id
//!     secret_manager:
//!       enabled: true
//!       cache_ttl_seconds: 300
//!     storage:
//!       bucket: llm-shield-models
//!       models_prefix: models/
//!       results_prefix: scan-results/
//!     monitoring:
//!       enabled: true
//!     logging:
//!       enabled: true
//!       log_name: llm-shield-api
//! ```
//!
//! # Performance
//!
//! - **Secret caching**: >90% cache hit rate reduces API calls
//! - **Resumable uploads**: Automatically used for objects >5MB
//! - **Batch export**: Metrics (20/batch) and logs (100/batch)
//! - **Async operations**: All I/O is fully asynchronous with tokio
//!
//! # Testing
//!
//! Run unit tests:
//!
//! ```bash
//! cargo test -p llm-shield-cloud-gcp
//! ```
//!
//! Run integration tests (requires GCP credentials):
//!
//! ```bash
//! export TEST_GCP_PROJECT=my-project-id
//! export TEST_GCS_BUCKET=llm-shield-test-bucket
//! cargo test -p llm-shield-cloud-gcp --test integration -- --ignored
//! ```
//!
//! # License
//!
//! MIT OR Apache-2.0

pub mod observability;
pub mod secrets;
pub mod storage;

// Re-export main types
pub use observability::{GcpCloudLogging, GcpCloudMonitoring};
pub use secrets::GcpSecretManager;
pub use storage::GcpCloudStorage;

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
        assert_eq!(LIB_NAME, "llm-shield-cloud-gcp");
    }
}
