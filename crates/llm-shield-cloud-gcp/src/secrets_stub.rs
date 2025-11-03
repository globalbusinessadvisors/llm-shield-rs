//! Stub implementation for GCP Secret Manager.
//!
//! NOTE: This is a temporary stub due to breaking changes in google-cloud SDK.
//! Full implementation requires updating to the latest SDK APIs.

use async_trait::async_trait;
use llm_shield_cloud::error::{CloudError, Result};
use llm_shield_cloud::secrets::{CloudSecretManager, SecretMetadata, SecretValue};

/// Stub implementation for GCP Secret Manager.
pub struct GcpSecretManager;

impl GcpSecretManager {
    /// Creates a new GCP Secret Manager client (stub).
    pub async fn new(_project_id: impl Into<String>) -> Result<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl CloudSecretManager for GcpSecretManager {
    async fn get_secret(&self, _name: &str) -> Result<SecretValue> {
        Err(CloudError::OperationFailed(
            "GCP Secret Manager not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn create_secret(&self, _name: &str, _value: &SecretValue) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Secret Manager not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn update_secret(&self, _name: &str, _value: &SecretValue) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Secret Manager not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn delete_secret(&self, _name: &str) -> Result<()> {
        Err(CloudError::OperationFailed(
            "GCP Secret Manager not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn list_secrets(&self) -> Result<Vec<String>> {
        Err(CloudError::OperationFailed(
            "GCP Secret Manager not implemented - SDK API breaking changes".to_string(),
        ))
    }

    async fn get_secret_metadata(&self, _name: &str) -> Result<SecretMetadata> {
        Err(CloudError::OperationFailed(
            "GCP Secret Manager not implemented - SDK API breaking changes".to_string(),
        ))
    }
}
