//! Configuration structures for cloud integrations.
//!
//! Provides unified configuration for all cloud providers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cloud provider selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    /// Amazon Web Services
    AWS,
    /// Google Cloud Platform
    GCP,
    /// Microsoft Azure
    Azure,
    /// No cloud provider (local/development mode)
    None,
}

impl CloudProvider {
    /// Returns the provider name as a string.
    pub fn as_str(&self) -> &str {
        match self {
            CloudProvider::AWS => "aws",
            CloudProvider::GCP => "gcp",
            CloudProvider::Azure => "azure",
            CloudProvider::None => "none",
        }
    }

    /// Checks if cloud integration is enabled.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, CloudProvider::None)
    }
}

/// Root cloud configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CloudConfig {
    /// Selected cloud provider.
    #[serde(default = "default_provider")]
    pub provider: CloudProvider,

    /// AWS-specific configuration.
    #[serde(default)]
    pub aws: AwsConfig,

    /// GCP-specific configuration.
    #[serde(default)]
    pub gcp: GcpConfig,

    /// Azure-specific configuration.
    #[serde(default)]
    pub azure: AzureConfig,
}

fn default_provider() -> CloudProvider {
    CloudProvider::None
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            provider: CloudProvider::None,
            aws: AwsConfig::default(),
            gcp: GcpConfig::default(),
            azure: AzureConfig::default(),
        }
    }
}

// ============================================================================
// AWS Configuration
// ============================================================================

/// AWS-specific configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AwsConfig {
    /// AWS region (e.g., "us-east-1").
    #[serde(default = "default_aws_region")]
    pub region: String,

    /// Secrets Manager configuration.
    #[serde(default)]
    pub secrets_manager: AwsSecretsManagerConfig,

    /// S3 storage configuration.
    #[serde(default)]
    pub s3: AwsS3Config,

    /// CloudWatch configuration.
    #[serde(default)]
    pub cloudwatch: AwsCloudWatchConfig,

    /// X-Ray tracing configuration.
    #[serde(default)]
    pub xray: AwsXRayConfig,
}

fn default_aws_region() -> String {
    "us-east-1".to_string()
}

/// AWS Secrets Manager configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AwsSecretsManagerConfig {
    /// Enable Secrets Manager integration.
    #[serde(default)]
    pub enabled: bool,

    /// Cache TTL in seconds.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

impl Default for AwsSecretsManagerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_ttl_seconds: 300,
        }
    }
}

fn default_cache_ttl() -> u64 {
    300
}

/// AWS S3 configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AwsS3Config {
    /// S3 bucket name.
    #[serde(default)]
    pub bucket: String,

    /// Prefix for models.
    #[serde(default = "default_models_prefix")]
    pub models_prefix: String,

    /// Prefix for scan results.
    #[serde(default = "default_results_prefix")]
    pub results_prefix: String,
}

fn default_models_prefix() -> String {
    "models/".to_string()
}

fn default_results_prefix() -> String {
    "scan-results/".to_string()
}

/// AWS CloudWatch configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AwsCloudWatchConfig {
    /// Enable CloudWatch integration.
    #[serde(default)]
    pub enabled: bool,

    /// CloudWatch namespace for metrics.
    #[serde(default = "default_cw_namespace")]
    pub namespace: String,

    /// CloudWatch log group.
    #[serde(default = "default_cw_log_group")]
    pub log_group: String,
}

impl Default for AwsCloudWatchConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            namespace: "LLMShield".to_string(),
            log_group: "/llm-shield/api".to_string(),
        }
    }
}

fn default_cw_namespace() -> String {
    "LLMShield".to_string()
}

fn default_cw_log_group() -> String {
    "/llm-shield/api".to_string()
}

/// AWS X-Ray configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AwsXRayConfig {
    /// Enable X-Ray tracing.
    #[serde(default)]
    pub enabled: bool,
}

impl Default for AwsXRayConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

// ============================================================================
// GCP Configuration
// ============================================================================

/// GCP-specific configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GcpConfig {
    /// GCP project ID.
    #[serde(default)]
    pub project_id: String,

    /// Secret Manager configuration.
    #[serde(default)]
    pub secret_manager: GcpSecretManagerConfig,

    /// Cloud Storage configuration.
    #[serde(default)]
    pub cloud_storage: GcpCloudStorageConfig,

    /// Cloud Logging configuration.
    #[serde(default)]
    pub cloud_logging: GcpCloudLoggingConfig,

    /// Cloud Trace configuration.
    #[serde(default)]
    pub cloud_trace: GcpCloudTraceConfig,
}

/// GCP Secret Manager configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GcpSecretManagerConfig {
    /// Enable Secret Manager integration.
    #[serde(default)]
    pub enabled: bool,

    /// Cache TTL in seconds.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

impl Default for GcpSecretManagerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_ttl_seconds: 300,
        }
    }
}

/// GCP Cloud Storage configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GcpCloudStorageConfig {
    /// Cloud Storage bucket name.
    #[serde(default)]
    pub bucket: String,
}

/// GCP Cloud Logging configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GcpCloudLoggingConfig {
    /// Enable Cloud Logging integration.
    #[serde(default)]
    pub enabled: bool,

    /// Log name.
    #[serde(default = "default_gcp_log_name")]
    pub log_name: String,
}

impl Default for GcpCloudLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_name: "llm-shield-api".to_string(),
        }
    }
}

fn default_gcp_log_name() -> String {
    "llm-shield-api".to_string()
}

/// GCP Cloud Trace configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GcpCloudTraceConfig {
    /// Enable Cloud Trace integration.
    #[serde(default)]
    pub enabled: bool,
}

impl Default for GcpCloudTraceConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

// ============================================================================
// Azure Configuration
// ============================================================================

/// Azure-specific configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AzureConfig {
    /// Azure subscription ID.
    #[serde(default)]
    pub subscription_id: String,

    /// Azure resource group.
    #[serde(default)]
    pub resource_group: String,

    /// Key Vault configuration.
    #[serde(default)]
    pub key_vault: AzureKeyVaultConfig,

    /// Blob Storage configuration.
    #[serde(default)]
    pub blob_storage: AzureBlobStorageConfig,

    /// Azure Monitor configuration.
    #[serde(default)]
    pub monitor: AzureMonitorConfig,

    /// Application Insights configuration.
    #[serde(default)]
    pub application_insights: AzureApplicationInsightsConfig,
}

/// Azure Key Vault configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureKeyVaultConfig {
    /// Key Vault URL (e.g., "https://my-vault.vault.azure.net/").
    #[serde(default)]
    pub vault_url: String,

    /// Cache TTL in seconds.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

impl Default for AzureKeyVaultConfig {
    fn default() -> Self {
        Self {
            vault_url: String::new(),
            cache_ttl_seconds: 300,
        }
    }
}

/// Azure Blob Storage configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AzureBlobStorageConfig {
    /// Storage account name.
    #[serde(default)]
    pub account: String,

    /// Container name.
    #[serde(default)]
    pub container: String,
}

/// Azure Monitor configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureMonitorConfig {
    /// Enable Azure Monitor integration.
    #[serde(default)]
    pub enabled: bool,

    /// Log Analytics workspace ID.
    #[serde(default)]
    pub workspace_id: String,
}

impl Default for AzureMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            workspace_id: String::new(),
        }
    }
}

/// Azure Application Insights configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureApplicationInsightsConfig {
    /// Instrumentation key.
    #[serde(default)]
    pub instrumentation_key: String,

    /// Enable Application Insights.
    #[serde(default)]
    pub enabled: bool,
}

impl Default for AzureApplicationInsightsConfig {
    fn default() -> Self {
        Self {
            instrumentation_key: String::new(),
            enabled: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_provider_as_str() {
        assert_eq!(CloudProvider::AWS.as_str(), "aws");
        assert_eq!(CloudProvider::GCP.as_str(), "gcp");
        assert_eq!(CloudProvider::Azure.as_str(), "azure");
        assert_eq!(CloudProvider::None.as_str(), "none");
    }

    #[test]
    fn test_cloud_provider_is_enabled() {
        assert!(CloudProvider::AWS.is_enabled());
        assert!(CloudProvider::GCP.is_enabled());
        assert!(CloudProvider::Azure.is_enabled());
        assert!(!CloudProvider::None.is_enabled());
    }

    #[test]
    fn test_cloud_config_default() {
        let config = CloudConfig::default();
        assert_eq!(config.provider, CloudProvider::None);
        assert!(!config.provider.is_enabled());
    }

    #[test]
    fn test_aws_config_defaults() {
        let config = AwsConfig::default();
        assert_eq!(config.region, "us-east-1");
        assert!(!config.secrets_manager.enabled);
        assert_eq!(config.secrets_manager.cache_ttl_seconds, 300);
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
provider: aws
aws:
  region: us-west-2
  secrets_manager:
    enabled: true
    cache_ttl_seconds: 600
  s3:
    bucket: my-bucket
"#;

        let config: CloudConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.provider, CloudProvider::AWS);
        assert_eq!(config.aws.region, "us-west-2");
        assert!(config.aws.secrets_manager.enabled);
        assert_eq!(config.aws.secrets_manager.cache_ttl_seconds, 600);
        assert_eq!(config.aws.s3.bucket, "my-bucket");
    }

    #[test]
    fn test_config_serialization() {
        let config = CloudConfig {
            provider: CloudProvider::GCP,
            gcp: GcpConfig {
                project_id: "my-project".to_string(),
                secret_manager: GcpSecretManagerConfig {
                    enabled: true,
                    cache_ttl_seconds: 300,
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("provider: gcp"));
        assert!(yaml.contains("project_id: my-project"));
    }
}
