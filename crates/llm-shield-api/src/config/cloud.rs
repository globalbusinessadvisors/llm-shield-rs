//! Cloud integration configuration

use super::{ConfigError, Result};
use serde::{Deserialize, Serialize};

/// Cloud provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// Enable cloud integrations
    #[serde(default)]
    pub enabled: bool,

    /// Cloud provider (aws, gcp, azure)
    #[serde(default)]
    pub provider: CloudProvider,

    /// AWS configuration
    #[serde(default)]
    pub aws: AwsConfig,

    /// GCP configuration
    #[serde(default)]
    pub gcp: GcpConfig,

    /// Azure configuration
    #[serde(default)]
    pub azure: AzureConfig,
}

impl CloudConfig {
    /// Validate cloud configuration
    pub fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        match self.provider {
            CloudProvider::Aws => self.aws.validate()?,
            CloudProvider::Gcp => self.gcp.validate()?,
            CloudProvider::Azure => self.azure.validate()?,
            CloudProvider::None => {
                return Err(ConfigError::ValidationError(
                    "Cloud enabled but no provider specified".to_string(),
                ))
            }
        }

        Ok(())
    }
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: CloudProvider::None,
            aws: AwsConfig::default(),
            gcp: GcpConfig::default(),
            azure: AzureConfig::default(),
        }
    }
}

/// Cloud provider selection
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    #[default]
    None,
    Aws,
    Gcp,
    Azure,
}

/// AWS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    /// AWS region
    pub region: Option<String>,

    /// Secrets Manager configuration
    #[serde(default)]
    pub secrets: AwsSecretsConfig,

    /// S3 configuration
    #[serde(default)]
    pub storage: AwsStorageConfig,

    /// CloudWatch configuration
    #[serde(default)]
    pub observability: AwsObservabilityConfig,
}

impl AwsConfig {
    fn validate(&self) -> Result<()> {
        self.secrets.validate()?;
        self.storage.validate()?;
        self.observability.validate()?;
        Ok(())
    }
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            region: None,
            secrets: AwsSecretsConfig::default(),
            storage: AwsStorageConfig::default(),
            observability: AwsObservabilityConfig::default(),
        }
    }
}

/// AWS Secrets Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwsSecretsConfig {
    /// Enable Secrets Manager
    #[serde(default)]
    pub enabled: bool,

    /// Secret cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

impl AwsSecretsConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.cache_ttl_seconds == 0 {
            return Err(ConfigError::ValidationError(
                "AWS Secrets cache TTL must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// AWS S3 configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwsStorageConfig {
    /// Enable S3 storage
    #[serde(default)]
    pub enabled: bool,

    /// S3 bucket name
    pub bucket: Option<String>,

    /// Optional prefix for all objects
    pub prefix: Option<String>,
}

impl AwsStorageConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.bucket.is_none() {
            return Err(ConfigError::ValidationError(
                "AWS S3 bucket must be specified when enabled".to_string(),
            ));
        }
        Ok(())
    }
}

/// AWS CloudWatch configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwsObservabilityConfig {
    /// Enable CloudWatch metrics
    #[serde(default)]
    pub metrics_enabled: bool,

    /// Enable CloudWatch logs
    #[serde(default)]
    pub logs_enabled: bool,

    /// CloudWatch namespace
    pub namespace: Option<String>,

    /// CloudWatch log group
    pub log_group: Option<String>,

    /// CloudWatch log stream
    pub log_stream: Option<String>,
}

impl AwsObservabilityConfig {
    fn validate(&self) -> Result<()> {
        if self.metrics_enabled && self.namespace.is_none() {
            return Err(ConfigError::ValidationError(
                "AWS CloudWatch namespace must be specified when metrics enabled".to_string(),
            ));
        }

        if self.logs_enabled {
            if self.log_group.is_none() {
                return Err(ConfigError::ValidationError(
                    "AWS CloudWatch log group must be specified when logs enabled".to_string(),
                ));
            }
            if self.log_stream.is_none() {
                return Err(ConfigError::ValidationError(
                    "AWS CloudWatch log stream must be specified when logs enabled".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// GCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpConfig {
    /// GCP project ID
    pub project_id: Option<String>,

    /// Secret Manager configuration
    #[serde(default)]
    pub secrets: GcpSecretsConfig,

    /// Cloud Storage configuration
    #[serde(default)]
    pub storage: GcpStorageConfig,

    /// Cloud Monitoring/Logging configuration
    #[serde(default)]
    pub observability: GcpObservabilityConfig,
}

impl GcpConfig {
    fn validate(&self) -> Result<()> {
        self.secrets.validate()?;
        self.storage.validate()?;
        self.observability.validate()?;
        Ok(())
    }
}

impl Default for GcpConfig {
    fn default() -> Self {
        Self {
            project_id: None,
            secrets: GcpSecretsConfig::default(),
            storage: GcpStorageConfig::default(),
            observability: GcpObservabilityConfig::default(),
        }
    }
}

/// GCP Secret Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GcpSecretsConfig {
    /// Enable Secret Manager
    #[serde(default)]
    pub enabled: bool,

    /// Secret cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

impl GcpSecretsConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.cache_ttl_seconds == 0 {
            return Err(ConfigError::ValidationError(
                "GCP Secrets cache TTL must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// GCP Cloud Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GcpStorageConfig {
    /// Enable Cloud Storage
    #[serde(default)]
    pub enabled: bool,

    /// Cloud Storage bucket name
    pub bucket: Option<String>,

    /// Optional prefix for all objects
    pub prefix: Option<String>,
}

impl GcpStorageConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled && self.bucket.is_none() {
            return Err(ConfigError::ValidationError(
                "GCP Cloud Storage bucket must be specified when enabled".to_string(),
            ));
        }
        Ok(())
    }
}

/// GCP Cloud Monitoring/Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GcpObservabilityConfig {
    /// Enable Cloud Monitoring metrics
    #[serde(default)]
    pub metrics_enabled: bool,

    /// Enable Cloud Logging
    #[serde(default)]
    pub logs_enabled: bool,

    /// Log name for Cloud Logging
    pub log_name: Option<String>,
}

impl GcpObservabilityConfig {
    fn validate(&self) -> Result<()> {
        if self.logs_enabled && self.log_name.is_none() {
            return Err(ConfigError::ValidationError(
                "GCP Cloud Logging log name must be specified when logs enabled".to_string(),
            ));
        }
        Ok(())
    }
}

/// Azure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    /// Key Vault configuration
    #[serde(default)]
    pub secrets: AzureSecretsConfig,

    /// Blob Storage configuration
    #[serde(default)]
    pub storage: AzureStorageConfig,

    /// Azure Monitor configuration
    #[serde(default)]
    pub observability: AzureObservabilityConfig,
}

impl AzureConfig {
    fn validate(&self) -> Result<()> {
        self.secrets.validate()?;
        self.storage.validate()?;
        self.observability.validate()?;
        Ok(())
    }
}

impl Default for AzureConfig {
    fn default() -> Self {
        Self {
            secrets: AzureSecretsConfig::default(),
            storage: AzureStorageConfig::default(),
            observability: AzureObservabilityConfig::default(),
        }
    }
}

/// Azure Key Vault configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzureSecretsConfig {
    /// Enable Key Vault
    #[serde(default)]
    pub enabled: bool,

    /// Key Vault URL
    pub vault_url: Option<String>,

    /// Secret cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

impl AzureSecretsConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            if self.vault_url.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure Key Vault URL must be specified when enabled".to_string(),
                ));
            }
            if self.cache_ttl_seconds == 0 {
                return Err(ConfigError::ValidationError(
                    "Azure Key Vault cache TTL must be greater than 0".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Azure Blob Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzureStorageConfig {
    /// Enable Blob Storage
    #[serde(default)]
    pub enabled: bool,

    /// Storage account name
    pub account_name: Option<String>,

    /// Container name
    pub container_name: Option<String>,
}

impl AzureStorageConfig {
    fn validate(&self) -> Result<()> {
        if self.enabled {
            if self.account_name.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure storage account name must be specified when enabled".to_string(),
                ));
            }
            if self.container_name.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure container name must be specified when enabled".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Azure Monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AzureObservabilityConfig {
    /// Enable Azure Monitor metrics
    #[serde(default)]
    pub metrics_enabled: bool,

    /// Enable Azure Monitor logs
    #[serde(default)]
    pub logs_enabled: bool,

    /// Azure Monitor resource ID
    pub resource_id: Option<String>,

    /// Azure region
    pub region: Option<String>,

    /// Log Analytics workspace ID
    pub workspace_id: Option<String>,

    /// Log Analytics shared key
    pub shared_key: Option<String>,

    /// Log type for Log Analytics
    pub log_type: Option<String>,
}

impl AzureObservabilityConfig {
    fn validate(&self) -> Result<()> {
        if self.metrics_enabled {
            if self.resource_id.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure Monitor resource ID must be specified when metrics enabled"
                        .to_string(),
                ));
            }
            if self.region.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure region must be specified when metrics enabled".to_string(),
                ));
            }
        }

        if self.logs_enabled {
            if self.workspace_id.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure Log Analytics workspace ID must be specified when logs enabled"
                        .to_string(),
                ));
            }
            if self.shared_key.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure Log Analytics shared key must be specified when logs enabled"
                        .to_string(),
                ));
            }
            if self.log_type.is_none() {
                return Err(ConfigError::ValidationError(
                    "Azure log type must be specified when logs enabled".to_string(),
                ));
            }
        }

        Ok(())
    }
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_config_default() {
        let config = CloudConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.provider, CloudProvider::None);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cloud_config_validation_no_provider() {
        let mut config = CloudConfig::default();
        config.enabled = true;
        // Provider is None
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_aws_config_validation() {
        let mut config = AwsConfig::default();

        // Storage enabled but no bucket
        config.storage.enabled = true;
        assert!(config.validate().is_err());

        // Add bucket
        config.storage.bucket = Some("my-bucket".to_string());
        assert!(config.validate().is_ok());

        // Metrics enabled but no namespace
        config.observability.metrics_enabled = true;
        assert!(config.validate().is_err());

        // Add namespace
        config.observability.namespace = Some("LLMShield".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_gcp_config_validation() {
        let mut config = GcpConfig::default();

        // Storage enabled but no bucket
        config.storage.enabled = true;
        assert!(config.validate().is_err());

        // Add bucket
        config.storage.bucket = Some("my-bucket".to_string());
        assert!(config.validate().is_ok());

        // Logs enabled but no log name
        config.observability.logs_enabled = true;
        assert!(config.validate().is_err());

        // Add log name
        config.observability.log_name = Some("llm-shield-logs".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_azure_config_validation() {
        let mut config = AzureConfig::default();

        // Secrets enabled but no vault URL
        config.secrets.enabled = true;
        assert!(config.validate().is_err());

        // Add vault URL
        config.secrets.vault_url = Some("https://my-vault.vault.azure.net".to_string());
        assert!(config.validate().is_ok());

        // Storage enabled but no account name
        config.storage.enabled = true;
        assert!(config.validate().is_err());

        // Add account name but no container
        config.storage.account_name = Some("myaccount".to_string());
        assert!(config.validate().is_err());

        // Add container
        config.storage.container_name = Some("models".to_string());
        assert!(config.validate().is_ok());
    }
}
