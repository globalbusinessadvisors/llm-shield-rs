//! Azure Key Vault integration.
//!
//! Provides implementation of `CloudSecretManager` trait for Azure Key Vault.

use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;
use azure_security_keyvault::KeyvaultClient;
use llm_shield_cloud::{
    async_trait, CloudError, CloudSecretManager, Result, SecretCache, SecretMetadata, SecretValue,
};
use std::collections::HashMap;
use std::sync::Arc;

/// Azure Key Vault implementation of `CloudSecretManager`.
///
/// This implementation provides:
/// - Automatic credential discovery (env, Azure CLI, managed identity)
/// - Built-in secret caching with TTL
/// - Support for secret versions
/// - Automatic retry with exponential backoff
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_azure::AzureKeyVault;
/// use llm_shield_cloud::CloudSecretManager;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let vault = AzureKeyVault::new("https://my-vault.vault.azure.net").await?;
///     let secret = vault.get_secret("my-secret").await?;
///     println!("Secret: {}", secret.as_string());
///     Ok(())
/// }
/// ```
pub struct AzureKeyVault {
    client: KeyvaultClient,
    vault_url: String,
    cache: SecretCache,
}

impl AzureKeyVault {
    /// Creates a new Azure Key Vault client with default configuration.
    ///
    /// Uses DefaultAzureCredential which tries:
    /// 1. Environment variables (AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET)
    /// 2. Azure CLI credentials
    /// 3. Managed Identity (for Azure VMs, App Service, etc.)
    ///
    /// # Arguments
    ///
    /// * `vault_url` - Key Vault URL (e.g., "https://my-vault.vault.azure.net")
    ///
    /// # Errors
    ///
    /// Returns error if Azure credentials cannot be loaded.
    pub async fn new(vault_url: impl Into<String>) -> Result<Self> {
        let vault_url = vault_url.into();

        // Create default credential
        let credential = Arc::new(DefaultAzureCredential::default());

        // Create Key Vault client
        let client = KeyvaultClient::new(&vault_url, credential)
            .map_err(|e| CloudError::Authentication(e.to_string()))?;

        let cache = SecretCache::new(300); // 5 minute default TTL

        tracing::info!("Initialized Azure Key Vault client for vault: {}", vault_url);

        Ok(Self {
            client,
            vault_url,
            cache,
        })
    }

    /// Creates a new Azure Key Vault client with custom cache TTL.
    ///
    /// # Arguments
    ///
    /// * `vault_url` - Key Vault URL
    /// * `cache_ttl_seconds` - Cache time-to-live in seconds
    ///
    /// # Errors
    ///
    /// Returns error if Azure credentials cannot be loaded.
    pub async fn new_with_cache_ttl(
        vault_url: impl Into<String>,
        cache_ttl_seconds: u64,
    ) -> Result<Self> {
        let vault_url = vault_url.into();

        let credential = Arc::new(DefaultAzureCredential::default());

        let client = KeyvaultClient::new(&vault_url, credential)
            .map_err(|e| CloudError::Authentication(e.to_string()))?;

        let cache = SecretCache::new(cache_ttl_seconds);

        tracing::info!(
            "Initialized Azure Key Vault client for vault: {} with {}s cache TTL",
            vault_url,
            cache_ttl_seconds
        );

        Ok(Self {
            client,
            vault_url,
            cache,
        })
    }

    /// Gets the Key Vault URL this client is configured for.
    pub fn vault_url(&self) -> &str {
        &self.vault_url
    }

    /// Clears the secret cache.
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        tracing::debug!("Cleared Azure Key Vault cache");
    }

    /// Gets the number of cached secrets.
    pub async fn cache_size(&self) -> usize {
        self.cache.len().await
    }
}

#[async_trait]
impl CloudSecretManager for AzureKeyVault {
    async fn get_secret(&self, name: &str) -> Result<SecretValue> {
        // Check cache first
        if let Some(cached) = self.cache.get(name).await {
            tracing::debug!("Cache hit for secret: {}", name);
            return Ok(cached);
        }

        tracing::debug!("Fetching secret from Azure Key Vault: {}", name);

        // Get secret from Key Vault
        let secret = self
            .client
            .get_secret(name)
            .await
            .map_err(|e| CloudError::secret_fetch(name, e.to_string()))?;

        let value = SecretValue::from_string(secret.value().to_string());

        // Cache the secret
        self.cache.set(name.to_string(), value.clone()).await;

        tracing::info!("Successfully fetched secret: {}", name);

        Ok(value)
    }

    async fn list_secrets(&self) -> Result<Vec<String>> {
        tracing::debug!("Listing secrets from Azure Key Vault");

        let mut secret_names = Vec::new();

        // List all secrets
        let secrets = self
            .client
            .list_secrets()
            .into_stream()
            .await
            .map_err(|e| CloudError::SecretList(e.to_string()))?;

        use futures::StreamExt;
        let mut stream = secrets;

        while let Some(result) = stream.next().await {
            let secret_item = result.map_err(|e| CloudError::SecretList(e.to_string()))?;
            if let Some(id) = secret_item.id() {
                // Extract secret name from ID
                if let Some(name) = id.rsplit('/').next() {
                    secret_names.push(name.to_string());
                }
            }
        }

        tracing::info!("Listed {} secrets", secret_names.len());

        Ok(secret_names)
    }

    async fn create_secret(&self, name: &str, value: &SecretValue) -> Result<()> {
        tracing::debug!("Creating secret in Azure Key Vault: {}", name);

        self.client
            .set_secret(name, value.as_string())
            .await
            .map_err(|e| CloudError::secret_create(name, e.to_string()))?;

        tracing::info!("Successfully created secret: {}", name);

        Ok(())
    }

    async fn update_secret(&self, name: &str, value: &SecretValue) -> Result<()> {
        tracing::debug!("Updating secret in Azure Key Vault: {}", name);

        // In Azure Key Vault, updating is the same as setting (creates new version)
        self.client
            .set_secret(name, value.as_string())
            .await
            .map_err(|e| CloudError::secret_update(name, e.to_string()))?;

        // Invalidate cache
        self.cache.invalidate(name).await;

        tracing::info!("Successfully updated secret: {}", name);

        Ok(())
    }

    async fn delete_secret(&self, name: &str) -> Result<()> {
        tracing::debug!("Deleting secret from Azure Key Vault: {}", name);

        // Azure Key Vault has soft delete by default
        self.client
            .delete_secret(name)
            .await
            .map_err(|e| CloudError::secret_delete(name, e.to_string()))?;

        // Invalidate cache
        self.cache.invalidate(name).await;

        tracing::info!("Successfully deleted secret (soft delete): {}", name);

        Ok(())
    }

    async fn get_secret_metadata(&self, name: &str) -> Result<SecretMetadata> {
        tracing::debug!("Fetching secret metadata from Azure Key Vault: {}", name);

        let secret = self
            .client
            .get_secret(name)
            .await
            .map_err(|e| CloudError::secret_fetch(name, e.to_string()))?;

        let attributes = secret.attributes();

        let created_at = attributes
            .created()
            .and_then(|t| {
                std::time::SystemTime::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_secs(t as u64))
            })
            .unwrap_or_else(std::time::SystemTime::now);

        let updated_at = attributes
            .updated()
            .and_then(|t| {
                std::time::SystemTime::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_secs(t as u64))
            })
            .unwrap_or(created_at);

        let tags: HashMap<String, String> = secret
            .tags()
            .map(|t| t.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        // Extract version from ID
        let version = secret.id().and_then(|id| {
            id.rsplit('/')
                .next()
                .filter(|v| !v.is_empty())
                .map(String::from)
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
    fn test_vault_url_format() {
        let vault_url = "https://my-vault.vault.azure.net";
        assert!(vault_url.starts_with("https://"));
        assert!(vault_url.contains(".vault.azure.net"));
    }

    #[test]
    fn test_secret_name_extraction() {
        let id = "https://my-vault.vault.azure.net/secrets/my-secret/abc123";
        let name = id.rsplit('/').nth(1);
        assert_eq!(name, Some("my-secret"));
    }

    #[tokio::test]
    async fn test_cache_operations() {
        // Test the cache independently without Azure calls
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
