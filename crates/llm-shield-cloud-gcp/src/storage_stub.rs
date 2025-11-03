//! Stub implementation for GCP Cloud Storage.
//!
//! NOTE: This is a temporary stub due to breaking changes in google-cloud SDK.
//! Full implementation requires updating to the latest SDK APIs.

use async_trait::async_trait;
use llm_shield_cloud::error::{CloudError, Result};
use llm_shield_cloud::storage::{CloudStorage, GetObjectOptions, ObjectMetadata, PutObjectOptions};

/// Stub implementation for GCP Cloud Storage.
pub struct GcpCloudStorage;

impl GcpCloudStorage {
    /// Creates a new GCP Cloud Storage client (stub).
    pub async fn new(_bucket: impl Into<String>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudStorage for GcpCloudStorage {
    async fn get_object(&self, _key: &str) -> Result<Vec<u8>> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn put_object(&self, _key: &str, _data: &[u8]) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn delete_object(&self, _key: &str) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn list_objects(&self, _prefix: &str) -> Result<Vec<String>> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn get_object_metadata(&self, _key: &str) -> Result<ObjectMetadata> {
        Err(CloudError::OperationFailed(
            "GCP Cloud Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        "gcs"
    }
}
