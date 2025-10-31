//! Cloud provider initialization

#[cfg(feature = "cloud")]
use crate::config::{AppConfig, CloudProvider};
#[cfg(feature = "cloud")]
use llm_shield_cloud::{CloudLogger, CloudMetrics, CloudSecretManager, CloudStorage};
#[cfg(feature = "cloud")]
use std::sync::Arc;
#[cfg(feature = "cloud")]
use thiserror::Error;

#[cfg(feature = "cloud")]
#[derive(Debug, Error)]
pub enum CloudInitError {
    #[error("Cloud provider not enabled")]
    NotEnabled,

    #[error("Cloud provider not supported: {0}")]
    UnsupportedProvider(String),

    #[error("Failed to initialize cloud provider: {0}")]
    InitializationError(String),

    #[error("Missing required configuration: {0}")]
    MissingConfiguration(String),
}

#[cfg(feature = "cloud")]
pub type Result<T> = std::result::Result<T, CloudInitError>;

/// Cloud provider instances
#[cfg(feature = "cloud")]
pub struct CloudProviders {
    pub secret_manager: Option<Arc<dyn CloudSecretManager>>,
    pub storage: Option<Arc<dyn CloudStorage>>,
    pub metrics: Option<Arc<dyn CloudMetrics>>,
    pub logger: Option<Arc<dyn CloudLogger>>,
}

/// Initialize cloud providers based on configuration
#[cfg(feature = "cloud")]
pub async fn initialize_cloud_providers(config: &AppConfig) -> Result<CloudProviders> {
    if !config.cloud.enabled {
        return Err(CloudInitError::NotEnabled);
    }

    match config.cloud.provider {
        CloudProvider::None => Err(CloudInitError::NotEnabled),
        #[cfg(feature = "cloud-aws")]
        CloudProvider::Aws => initialize_aws(config).await,
        #[cfg(feature = "cloud-gcp")]
        CloudProvider::Gcp => initialize_gcp(config).await,
        #[cfg(feature = "cloud-azure")]
        CloudProvider::Azure => initialize_azure(config).await,
        #[cfg(not(any(feature = "cloud-aws", feature = "cloud-gcp", feature = "cloud-azure")))]
        _ => Err(CloudInitError::UnsupportedProvider(format!(
            "{:?}",
            config.cloud.provider
        ))),
    }
}

/// Initialize AWS cloud providers
#[cfg(all(feature = "cloud", feature = "cloud-aws"))]
async fn initialize_aws(config: &AppConfig) -> Result<CloudProviders> {
    use llm_shield_cloud_aws::{AwsCloudWatchLogs, AwsCloudWatchMetrics, AwsS3Storage, AwsSecretsManager};

    let aws_config = aws_config::from_env();
    let aws_config = if let Some(region) = &config.cloud.aws.region {
        aws_config
            .region(aws_config::Region::new(region.clone()))
            .load()
            .await
    } else {
        aws_config.load().await
    };

    let mut secret_manager = None;
    let mut storage = None;
    let mut metrics = None;
    let mut logger = None;

    // Initialize Secrets Manager
    if config.cloud.aws.secrets.enabled {
        secret_manager = Some(
            Arc::new(
                AwsSecretsManager::new(aws_config.clone())
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudSecretManager>
        );
    }

    // Initialize S3 Storage
    if config.cloud.aws.storage.enabled {
        let bucket = config
            .cloud
            .aws
            .storage
            .bucket
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("AWS S3 bucket".to_string()))?;

        storage = Some(
            Arc::new(
                AwsS3Storage::new(aws_config.clone(), bucket)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudStorage>
        );
    }

    // Initialize CloudWatch Metrics
    if config.cloud.aws.observability.metrics_enabled {
        let namespace = config
            .cloud
            .aws
            .observability
            .namespace
            .as_ref()
            .ok_or_else(|| {
                CloudInitError::MissingConfiguration("AWS CloudWatch namespace".to_string())
            })?;

        metrics = Some(
            Arc::new(
                AwsCloudWatchMetrics::new(aws_config.clone(), namespace)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudMetrics>
        );
    }

    // Initialize CloudWatch Logs
    if config.cloud.aws.observability.logs_enabled {
        let log_group = config
            .cloud
            .aws
            .observability
            .log_group
            .as_ref()
            .ok_or_else(|| {
                CloudInitError::MissingConfiguration("AWS CloudWatch log group".to_string())
            })?;

        let log_stream = config
            .cloud
            .aws
            .observability
            .log_stream
            .as_ref()
            .ok_or_else(|| {
                CloudInitError::MissingConfiguration("AWS CloudWatch log stream".to_string())
            })?;

        logger = Some(
            Arc::new(
                AwsCloudWatchLogs::new(aws_config, log_group, log_stream)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudLogger>
        );
    }

    Ok(CloudProviders {
        secret_manager,
        storage,
        metrics,
        logger,
    })
}

/// Initialize GCP cloud providers
#[cfg(all(feature = "cloud", feature = "cloud-gcp"))]
async fn initialize_gcp(config: &AppConfig) -> Result<CloudProviders> {
    use llm_shield_cloud_gcp::{
        GcpCloudLogging, GcpCloudMonitoring, GcpCloudStorage, GcpSecretManager,
    };

    let project_id = config
        .cloud
        .gcp
        .project_id
        .as_ref()
        .ok_or_else(|| CloudInitError::MissingConfiguration("GCP project ID".to_string()))?;

    let mut secret_manager = None;
    let mut storage = None;
    let mut metrics = None;
    let mut logger = None;

    // Initialize Secret Manager
    if config.cloud.gcp.secrets.enabled {
        secret_manager = Some(
            Arc::new(
                GcpSecretManager::new(project_id)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudSecretManager>
        );
    }

    // Initialize Cloud Storage
    if config.cloud.gcp.storage.enabled {
        let bucket = config
            .cloud
            .gcp
            .storage
            .bucket
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("GCP bucket".to_string()))?;

        storage = Some(
            Arc::new(
                GcpCloudStorage::new(bucket)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudStorage>
        );
    }

    // Initialize Cloud Monitoring
    if config.cloud.gcp.observability.metrics_enabled {
        metrics = Some(
            Arc::new(
                GcpCloudMonitoring::new(project_id)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudMetrics>
        );
    }

    // Initialize Cloud Logging
    if config.cloud.gcp.observability.logs_enabled {
        let log_name = config
            .cloud
            .gcp
            .observability
            .log_name
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("GCP log name".to_string()))?;

        logger = Some(
            Arc::new(
                GcpCloudLogging::new(project_id, log_name)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudLogger>
        );
    }

    Ok(CloudProviders {
        secret_manager,
        storage,
        metrics,
        logger,
    })
}

/// Initialize Azure cloud providers
#[cfg(all(feature = "cloud", feature = "cloud-azure"))]
async fn initialize_azure(config: &AppConfig) -> Result<CloudProviders> {
    use llm_shield_cloud_azure::{
        AzureBlobStorage, AzureKeyVault, AzureMonitorLogs, AzureMonitorMetrics,
    };

    let mut secret_manager = None;
    let mut storage = None;
    let mut metrics = None;
    let mut logger = None;

    // Initialize Key Vault
    if config.cloud.azure.secrets.enabled {
        let vault_url = config
            .cloud
            .azure
            .secrets
            .vault_url
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("Azure vault URL".to_string()))?;

        secret_manager = Some(
            Arc::new(
                AzureKeyVault::new(vault_url)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudSecretManager>
        );
    }

    // Initialize Blob Storage
    if config.cloud.azure.storage.enabled {
        let account_name = config
            .cloud
            .azure
            .storage
            .account_name
            .as_ref()
            .ok_or_else(|| {
                CloudInitError::MissingConfiguration("Azure storage account".to_string())
            })?;

        let container_name = config
            .cloud
            .azure
            .storage
            .container_name
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("Azure container".to_string()))?;

        storage = Some(
            Arc::new(
                AzureBlobStorage::new(account_name, container_name)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudStorage>
        );
    }

    // Initialize Azure Monitor Metrics
    if config.cloud.azure.observability.metrics_enabled {
        let resource_id = config
            .cloud
            .azure
            .observability
            .resource_id
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("Azure resource ID".to_string()))?;

        let region = config
            .cloud
            .azure
            .observability
            .region
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("Azure region".to_string()))?;

        metrics = Some(
            Arc::new(
                AzureMonitorMetrics::new(resource_id, region)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudMetrics>
        );
    }

    // Initialize Azure Monitor Logs
    if config.cloud.azure.observability.logs_enabled {
        let workspace_id = config
            .cloud
            .azure
            .observability
            .workspace_id
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("Azure workspace ID".to_string()))?;

        let shared_key = config
            .cloud
            .azure
            .observability
            .shared_key
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("Azure shared key".to_string()))?;

        let log_type = config
            .cloud
            .azure
            .observability
            .log_type
            .as_ref()
            .ok_or_else(|| CloudInitError::MissingConfiguration("Azure log type".to_string()))?;

        logger = Some(
            Arc::new(
                AzureMonitorLogs::new(workspace_id, shared_key, log_type)
                    .await
                    .map_err(|e| CloudInitError::InitializationError(e.to_string()))?,
            ) as Arc<dyn CloudLogger>
        );
    }

    Ok(CloudProviders {
        secret_manager,
        storage,
        metrics,
        logger,
    })
}

#[cfg(test)]
#[cfg(feature = "cloud")]
mod tests {
    use super::*;
    use crate::config::CloudConfig;

    #[tokio::test]
    async fn test_cloud_not_enabled() {
        let config = AppConfig::default();
        let result = initialize_cloud_providers(&config).await;
        assert!(matches!(result, Err(CloudInitError::NotEnabled)));
    }

    #[tokio::test]
    async fn test_cloud_no_provider() {
        let mut config = AppConfig::default();
        config.cloud.enabled = true;
        config.cloud.provider = CloudProvider::None;

        let result = initialize_cloud_providers(&config).await;
        assert!(matches!(result, Err(CloudInitError::NotEnabled)));
    }
}
