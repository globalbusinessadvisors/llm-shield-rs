//! Secret management abstractions.
//!
//! Provides unified trait for secret management across cloud providers:
//! - AWS Secrets Manager
//! - GCP Secret Manager
//! - Azure Key Vault

use crate::error::{CloudError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Represents a secret value that can be stored and retrieved.
#[derive(Clone, Debug)]
pub struct SecretValue {
    data: Vec<u8>,
}

impl SecretValue {
    /// Creates a secret value from a string.
    pub fn from_string(s: String) -> Self {
        Self {
            data: s.into_bytes(),
        }
    }

    /// Creates a secret value from bytes.
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns the secret value as a string.
    ///
    /// # Panics
    ///
    /// Panics if the data is not valid UTF-8.
    pub fn as_string(&self) -> &str {
        std::str::from_utf8(&self.data).expect("Secret value is not valid UTF-8")
    }

    /// Returns the secret value as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns the length of the secret in bytes.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Checks if the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Metadata about a secret.
#[derive(Debug, Clone)]
pub struct SecretMetadata {
    /// The name/ID of the secret.
    pub name: String,

    /// When the secret was created.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When the secret was last updated.
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Optional tags/labels for the secret.
    pub tags: HashMap<String, String>,

    /// Secret version (if supported by provider).
    pub version: Option<String>,
}

/// Unified trait for cloud secret management.
///
/// This trait provides a consistent interface for managing secrets across
/// different cloud providers (AWS, GCP, Azure).
#[async_trait]
pub trait CloudSecretManager: Send + Sync {
    /// Fetches a secret by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name/ID of the secret to fetch
    ///
    /// # Returns
    ///
    /// Returns the secret value if found.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::SecretNotFound` if the secret doesn't exist.
    /// Returns `CloudError::SecretFetch` if the fetch operation fails.
    async fn get_secret(&self, name: &str) -> Result<SecretValue>;

    /// Lists all secret names.
    ///
    /// # Returns
    ///
    /// Returns a vector of secret names/IDs.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::SecretList` if the list operation fails.
    async fn list_secrets(&self) -> Result<Vec<String>>;

    /// Creates a new secret.
    ///
    /// # Arguments
    ///
    /// * `name` - The name/ID for the new secret
    /// * `value` - The secret value to store
    ///
    /// # Errors
    ///
    /// Returns `CloudError::SecretCreate` if the create operation fails.
    async fn create_secret(&self, name: &str, value: &SecretValue) -> Result<()>;

    /// Updates an existing secret.
    ///
    /// # Arguments
    ///
    /// * `name` - The name/ID of the secret to update
    /// * `value` - The new secret value
    ///
    /// # Errors
    ///
    /// Returns `CloudError::SecretUpdate` if the update operation fails.
    async fn update_secret(&self, name: &str, value: &SecretValue) -> Result<()>;

    /// Deletes a secret.
    ///
    /// # Arguments
    ///
    /// * `name` - The name/ID of the secret to delete
    ///
    /// # Errors
    ///
    /// Returns `CloudError::SecretDelete` if the delete operation fails.
    async fn delete_secret(&self, name: &str) -> Result<()>;

    /// Rotates a secret (creates a new version).
    ///
    /// Default implementation calls `update_secret`.
    ///
    /// # Arguments
    ///
    /// * `name` - The name/ID of the secret to rotate
    /// * `new_value` - The new secret value
    ///
    /// # Errors
    ///
    /// Returns `CloudError::SecretUpdate` if the rotation fails.
    async fn rotate_secret(&self, name: &str, new_value: &SecretValue) -> Result<()> {
        self.update_secret(name, new_value).await
    }

    /// Gets secret metadata without fetching the value.
    ///
    /// Default implementation fetches the secret and discards the value.
    /// Providers should override this for efficiency.
    ///
    /// # Arguments
    ///
    /// * `name` - The name/ID of the secret
    ///
    /// # Errors
    ///
    /// Returns `CloudError::SecretFetch` if the operation fails.
    async fn get_secret_metadata(&self, name: &str) -> Result<SecretMetadata> {
        // Default implementation - providers should override for efficiency
        let _ = self.get_secret(name).await?;

        Ok(SecretMetadata {
            name: name.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            tags: HashMap::new(),
            version: None,
        })
    }
}

/// Cached secret with expiration.
#[derive(Clone, Debug)]
struct CachedSecret {
    value: SecretValue,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedSecret {
    /// Checks if the cached secret has expired.
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// In-memory cache for secrets with TTL.
///
/// This cache reduces the number of API calls to cloud secret managers.
pub struct SecretCache {
    cache: Arc<RwLock<HashMap<String, CachedSecret>>>,
    default_ttl: Duration,
}

impl SecretCache {
    /// Creates a new secret cache with the specified TTL.
    ///
    /// # Arguments
    ///
    /// * `ttl_seconds` - Default time-to-live for cached secrets in seconds
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Gets a secret from the cache if it exists and hasn't expired.
    ///
    /// # Arguments
    ///
    /// * `key` - The secret name/key
    ///
    /// # Returns
    ///
    /// Returns `Some(SecretValue)` if the secret is in cache and not expired,
    /// otherwise returns `None`.
    pub async fn get(&self, key: &str) -> Option<SecretValue> {
        let cache = self.cache.read().await;

        if let Some(cached) = cache.get(key) {
            if !cached.is_expired() {
                return Some(cached.value.clone());
            }
        }

        None
    }

    /// Stores a secret in the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The secret name/key
    /// * `value` - The secret value to cache
    pub async fn set(&self, key: String, value: SecretValue) {
        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CachedSecret {
                value,
                cached_at: Instant::now(),
                ttl: self.default_ttl,
            },
        );
    }

    /// Invalidates a specific secret in the cache.
    ///
    /// # Arguments
    ///
    /// * `key` - The secret name/key to invalidate
    pub async fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
    }

    /// Clears all secrets from the cache.
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Returns the number of secrets currently in the cache.
    pub async fn len(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Checks if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.read().await;
        cache.is_empty()
    }

    /// Removes expired secrets from the cache.
    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, cached| !cached.is_expired());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_value_from_string() {
        let secret = SecretValue::from_string("test-secret".to_string());
        assert_eq!(secret.as_string(), "test-secret");
        assert_eq!(secret.len(), 11);
        assert!(!secret.is_empty());
    }

    #[test]
    fn test_secret_value_from_bytes() {
        let data = vec![1, 2, 3, 4];
        let secret = SecretValue::from_bytes(data.clone());
        assert_eq!(secret.as_bytes(), &data[..]);
        assert_eq!(secret.len(), 4);
    }

    #[test]
    fn test_secret_value_empty() {
        let secret = SecretValue::from_bytes(vec![]);
        assert!(secret.is_empty());
        assert_eq!(secret.len(), 0);
    }

    #[tokio::test]
    async fn test_secret_cache_basic() {
        let cache = SecretCache::new(300);
        let secret = SecretValue::from_string("cached-value".to_string());

        // Initially empty
        assert!(cache.is_empty().await);
        assert_eq!(cache.len().await, 0);

        // Set and get
        cache.set("test-key".to_string(), secret.clone()).await;
        assert_eq!(cache.len().await, 1);

        let retrieved = cache.get("test-key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_string(), "cached-value");

        // Invalidate
        cache.invalidate("test-key").await;
        assert!(cache.get("test-key").await.is_none());
    }

    #[tokio::test]
    async fn test_secret_cache_expiration() {
        let cache = SecretCache::new(1); // 1 second TTL
        let secret = SecretValue::from_string("expires-soon".to_string());

        cache.set("expiring-key".to_string(), secret).await;

        // Should be available immediately
        assert!(cache.get("expiring-key").await.is_some());

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be expired
        assert!(cache.get("expiring-key").await.is_none());
    }

    #[tokio::test]
    async fn test_secret_cache_clear() {
        let cache = SecretCache::new(300);

        cache.set("key1".to_string(), SecretValue::from_string("val1".to_string())).await;
        cache.set("key2".to_string(), SecretValue::from_string("val2".to_string())).await;

        assert_eq!(cache.len().await, 2);

        cache.clear().await;

        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_secret_cache_cleanup_expired() {
        let cache = SecretCache::new(1); // 1 second TTL

        cache.set("short".to_string(), SecretValue::from_string("expires".to_string())).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Add a fresh entry
        cache.set("fresh".to_string(), SecretValue::from_string("current".to_string())).await;

        // Should have 2 entries (one expired)
        assert_eq!(cache.len().await, 2);

        // Cleanup expired
        cache.cleanup_expired().await;

        // Should only have the fresh entry
        assert_eq!(cache.len().await, 1);
        assert!(cache.get("fresh").await.is_some());
        assert!(cache.get("short").await.is_none());
    }
}
