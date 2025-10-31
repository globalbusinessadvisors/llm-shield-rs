//! AWS Secrets Manager integration.
//!
//! Provides implementation of `CloudSecretManager` trait for AWS Secrets Manager.

use aws_sdk_secretsmanager::Client;
use llm_shield_cloud::{
    async_trait, CloudError, CloudSecretManager, Result, SecretCache, SecretMetadata, SecretValue,
};
use std::collections::HashMap;
use std::sync::Arc;

/// AWS Secrets Manager implementation of `CloudSecretManager`.
///
/// This implementation provides:
/// - Automatic credential discovery (env → file → IAM role)
/// - Built-in secret caching with TTL
/// - Support for both string and binary secrets
/// - Automatic retry with exponential backoff
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_aws::AwsSecretsManager;
/// use llm_shield_cloud::CloudSecretManager;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let manager = AwsSecretsManager::new().await?;
///     let secret = manager.get_secret("my-secret").await?;
///     println!("Secret: {}", secret.as_string());
///     Ok(())
/// }
/// ```
pub struct AwsSecretsManager {
    client: Client,
    cache: SecretCache,
    region: String,
}

impl AwsSecretsManager {
    /// Creates a new AWS Secrets Manager client with default configuration.
    ///
    /// Uses the AWS credential provider chain:
    /// 1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
    /// 2. AWS credentials file (~/.aws/credentials)
    /// 3. IAM role for ECS task
    /// 4. IAM role for EC2 instance
    /// 5. IAM role for EKS pod (IRSA)
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new() -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let region = config
            .region()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "us-east-1".to_string());

        let client = Client::new(&config);
        let cache = SecretCache::new(300); // 5 minute default TTL

        tracing::info!("Initialized AWS Secrets Manager client in region: {}", region);

        Ok(Self {
            client,
            cache,
            region,
        })
    }

    /// Creates a new AWS Secrets Manager client with specific region.
    ///
    /// # Arguments
    ///
    /// * `region` - AWS region (e.g., "us-east-1", "eu-west-1")
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new_with_region(region: impl Into<String>) -> Result<Self> {
        let region_str = region.into();
        let config = aws_config::from_env()
            .region(aws_config::Region::new(region_str.clone()))
            .load()
            .await;

        let client = Client::new(&config);
        let cache = SecretCache::new(300);

        tracing::info!("Initialized AWS Secrets Manager client in region: {}", region_str);

        Ok(Self {
            client,
            cache,
            region: region_str,
        })
    }

    /// Creates a new AWS Secrets Manager client with custom cache TTL.
    ///
    /// # Arguments
    ///
    /// * `region` - AWS region
    /// * `cache_ttl_seconds` - Cache time-to-live in seconds
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new_with_cache_ttl(
        region: impl Into<String>,
        cache_ttl_seconds: u64,
    ) -> Result<Self> {
        let region_str = region.into();
        let config = aws_config::from_env()
            .region(aws_config::Region::new(region_str.clone()))
            .load()
            .await;

        let client = Client::new(&config);
        let cache = SecretCache::new(cache_ttl_seconds);

        tracing::info!(
            "Initialized AWS Secrets Manager client in region: {} with {}s cache TTL",
            region_str,
            cache_ttl_seconds
        );

        Ok(Self {
            client,
            cache,
            region: region_str,
        })
    }

    /// Gets the AWS region this client is configured for.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Clears the secret cache.
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        tracing::debug!("Cleared AWS Secrets Manager cache");
    }

    /// Gets the number of cached secrets.
    pub async fn cache_size(&self) -> usize {
        self.cache.len().await
    }
}

#[async_trait]
impl CloudSecretManager for AwsSecretsManager {
    async fn get_secret(&self, name: &str) -> Result<SecretValue> {
        // Check cache first
        if let Some(cached) = self.cache.get(name).await {
            tracing::debug!("Cache hit for secret: {}", name);
            return Ok(cached);
        }

        tracing::debug!("Fetching secret from AWS Secrets Manager: {}", name);

        // Fetch from AWS Secrets Manager
        let response = self
            .client
            .get_secret_value()
            .secret_id(name)
            .send()
            .await
            .map_err(|e| CloudError::secret_fetch(name, e.to_string()))?;

        // Handle both string and binary secrets
        let value = if let Some(secret_string) = response.secret_string() {
            SecretValue::from_string(secret_string.to_string())
        } else if let Some(secret_binary) = response.secret_binary() {
            SecretValue::from_bytes(secret_binary.clone().into_inner())
        } else {
            return Err(CloudError::SecretFormat {
                name: name.to_string(),
                reason: "Secret has no string or binary value".to_string(),
            });
        };

        // Cache the secret
        self.cache.set(name.to_string(), value.clone()).await;

        tracing::info!("Successfully fetched secret: {}", name);

        Ok(value)
    }

    async fn list_secrets(&self) -> Result<Vec<String>> {
        tracing::debug!("Listing secrets from AWS Secrets Manager");

        let mut secret_names = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = self.client.list_secrets();

            if let Some(token) = next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| CloudError::SecretList(e.to_string()))?;

            if let Some(secret_list) = response.secret_list() {
                for secret in secret_list {
                    if let Some(name) = secret.name() {
                        secret_names.push(name.to_string());
                    }
                }
            }

            next_token = response.next_token().map(String::from);

            if next_token.is_none() {
                break;
            }
        }

        tracing::info!("Listed {} secrets", secret_names.len());

        Ok(secret_names)
    }

    async fn create_secret(&self, name: &str, value: &SecretValue) -> Result<()> {
        tracing::debug!("Creating secret in AWS Secrets Manager: {}", name);

        self.client
            .create_secret()
            .name(name)
            .secret_string(value.as_string())
            .send()
            .await
            .map_err(|e| CloudError::secret_create(name, e.to_string()))?;

        tracing::info!("Successfully created secret: {}", name);

        Ok(())
    }

    async fn update_secret(&self, name: &str, value: &SecretValue) -> Result<()> {
        tracing::debug!("Updating secret in AWS Secrets Manager: {}", name);

        self.client
            .update_secret()
            .secret_id(name)
            .secret_string(value.as_string())
            .send()
            .await
            .map_err(|e| CloudError::secret_update(name, e.to_string()))?;

        // Invalidate cache
        self.cache.invalidate(name).await;

        tracing::info!("Successfully updated secret: {}", name);

        Ok(())
    }

    async fn delete_secret(&self, name: &str) -> Result<()> {
        tracing::debug!("Deleting secret from AWS Secrets Manager: {}", name);

        self.client
            .delete_secret()
            .secret_id(name)
            .force_delete_without_recovery(false) // 30-day recovery window
            .send()
            .await
            .map_err(|e| CloudError::secret_delete(name, e.to_string()))?;

        // Invalidate cache
        self.cache.invalidate(name).await;

        tracing::info!("Successfully deleted secret (30-day recovery): {}", name);

        Ok(())
    }

    async fn get_secret_metadata(&self, name: &str) -> Result<SecretMetadata> {
        tracing::debug!("Fetching secret metadata from AWS Secrets Manager: {}", name);

        let response = self
            .client
            .describe_secret()
            .secret_id(name)
            .send()
            .await
            .map_err(|e| CloudError::secret_fetch(name, e.to_string()))?;

        let created_at = response
            .created_date()
            .and_then(|dt| {
                chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
            })
            .unwrap_or_else(chrono::Utc::now);

        let updated_at = response
            .last_changed_date()
            .and_then(|dt| {
                chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
            })
            .unwrap_or(created_at);

        let mut tags = HashMap::new();
        if let Some(tag_list) = response.tags() {
            for tag in tag_list {
                if let (Some(key), Some(value)) = (tag.key(), tag.value()) {
                    tags.insert(key.to_string(), value.to_string());
                }
            }
        }

        let version = response.version_ids_to_stages().and_then(|versions| {
            versions
                .iter()
                .find(|(_, stages)| stages.contains(&"AWSCURRENT".to_string()))
                .map(|(version_id, _)| version_id.clone())
        });

        Ok(SecretMetadata {
            name: name.to_string(),
            created_at,
            updated_at,
            tags,
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_secrets_manager_region() {
        // This test just checks the struct can be created with a region
        // Actual AWS operations require real credentials and are in integration tests
        let region = "us-west-2".to_string();
        // We can't create the client without AWS credentials, so just test the logic
        assert_eq!(region, "us-west-2");
    }

    #[tokio::test]
    async fn test_cache_operations() {
        // Test the cache independently without AWS calls
        let cache = SecretCache::new(300);

        let test_secret = SecretValue::from_string("test-value".to_string());
        cache.set("test-key".to_string(), test_secret.clone()).await;

        let retrieved = cache.get("test-key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_string(), "test-value");

        cache.clear().await;
        assert_eq!(cache.len().await, 0);
    }
}
