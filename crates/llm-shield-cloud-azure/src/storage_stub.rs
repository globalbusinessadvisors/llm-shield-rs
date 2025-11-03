//! Stub implementation for Azure Blob Storage.
//!
//! NOTE: This is a temporary stub due to breaking changes in Azure SDK.
//! Full implementation requires updating to the latest SDK APIs.

use async_trait::async_trait;
use llm_shield_cloud::error::{CloudError, Result};
use llm_shield_cloud::storage::{CloudStorage, GetObjectOptions, ObjectMetadata, PutObjectOptions};

/// Stub implementation for Azure Blob Storage.
pub struct AzureBlobStorage;

impl AzureBlobStorage {
    /// Creates a new Azure Blob Storage client (stub).
    pub async fn new(_container: impl Into<String>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudStorage for AzureBlobStorage {
    async fn get_object(&self, _key: &str) -> Result<Vec<u8>> {
        Err(CloudError::OperationFailed(
            "Azure Blob Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn put_object(&self, _key: &str, _data: &[u8]) -> Result<()> {
        Err(CloudError::OperationFailed(
            "Azure Blob Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn delete_object(&self, _key: &str) -> Result<()> {
        Err(CloudError::OperationFailed(
            "Azure Blob Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn list_objects(&self, _prefix: &str) -> Result<Vec<String>> {
        Err(CloudError::OperationFailed(
            "Azure Blob Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn get_object_metadata(&self, _key: &str) -> Result<ObjectMetadata> {
        Err(CloudError::OperationFailed(
            "Azure Blob Storage not implemented - SDK API breaking changes".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        "azure"
    }
}
