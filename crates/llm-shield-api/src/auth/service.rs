//! Authentication service
//!
//! ## SPARC Phase 3: Construction (TDD)
//!
//! High-level API key management service.

use super::storage::KeyStorage;
use super::types::{ApiKey, CreateKeyRequest, CreateKeyResponse, Result};
use crate::config::rate_limit::RateLimitTier;
use chrono::{Duration, Utc};
use llm_shield_core::Error;
use std::sync::Arc;

/// Authentication service
///
/// ## Features
///
/// - Create new API keys
/// - Validate keys from requests
/// - Revoke/deactivate keys
/// - List and manage keys
///
/// ## Example
///
/// ```rust,ignore
/// let storage = MemoryKeyStorage::new();
/// let service = AuthService::new(Arc::new(storage));
///
/// // Create a new key
/// let response = service.create_key(
///     "My App".to_string(),
///     RateLimitTier::Pro,
///     Some(365)
/// ).await?;
///
/// println!("Your API key: {}", response.key);
///
/// // Validate a key
/// let key = service.validate_key(&response.key).await?;
/// ```
pub struct AuthService {
    storage: Arc<dyn KeyStorage>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(storage: Arc<dyn KeyStorage>) -> Self {
        Self { storage }
    }

    /// Create a new API key
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for the key
    /// * `tier` - Rate limit tier
    /// * `expires_in_days` - Optional expiration in days from now
    ///
    /// # Returns
    ///
    /// `CreateKeyResponse` with the raw API key (only shown once!)
    pub async fn create_key(
        &self,
        name: String,
        tier: RateLimitTier,
        expires_in_days: Option<u32>,
    ) -> Result<CreateKeyResponse> {
        // Calculate expiration date
        let expires_at = expires_in_days.map(|days| Utc::now() + Duration::days(days as i64));

        // Generate new key
        let mut key = ApiKey::new(name, tier, expires_at)?;

        // Store key
        self.storage.store(&key).await?;

        // Create response (includes raw key)
        let response = CreateKeyResponse::from(key.clone());

        // Clear raw value from stored key for security
        key.clear_value();
        self.storage.update(&key).await?;

        Ok(response)
    }

    /// Create a key from request
    pub async fn create_key_from_request(
        &self,
        request: CreateKeyRequest,
    ) -> Result<CreateKeyResponse> {
        self.create_key(request.name, request.tier, request.expires_in_days)
            .await
    }

    /// Validate an API key from a request
    ///
    /// # Arguments
    ///
    /// * `raw_key` - The raw API key value from the Authorization header
    ///
    /// # Returns
    ///
    /// * `Ok(ApiKey)` - Valid, active, non-expired key
    /// * `Err(Error::Unauthorized)` - Invalid, inactive, or expired key
    ///
    /// # Security
    ///
    /// - Uses constant-time comparison via argon2
    /// - Checks expiration and active status
    /// - Returns generic error on failure (don't leak info)
    pub async fn validate_key(&self, raw_key: &str) -> Result<ApiKey> {
        // Validate format first (fast fail)
        if !ApiKey::validate_format(raw_key) {
            return Err(Error::unauthorized("Invalid API key format"));
        }

        // Get all keys and find matching hash
        // Note: In a production system with many keys, we'd optimize this
        // by indexing keys by a hash prefix or using a different lookup strategy
        let all_keys = self.storage.list().await?;

        for key in all_keys {
            // Use constant-time verification
            if key.verify(raw_key)? {
                // Found matching key - check validity
                if !key.active {
                    return Err(Error::unauthorized("API key has been revoked"));
                }

                if key.is_expired() {
                    return Err(Error::unauthorized("API key has expired"));
                }

                return Ok(key);
            }
        }

        // No matching key found
        Err(Error::unauthorized("Invalid API key"))
    }

    /// Revoke an API key by ID
    ///
    /// Sets the key's active status to false without deleting it.
    pub async fn revoke_key(&self, id: &str) -> Result<()> {
        let mut key = self
            .storage
            .get_by_id(id)
            .await?
            .ok_or_else(|| Error::not_found("API key not found"))?;

        key.active = false;
        self.storage.update(&key).await?;

        Ok(())
    }

    /// Delete an API key by ID
    ///
    /// Permanently removes the key from storage.
    pub async fn delete_key(&self, id: &str) -> Result<()> {
        self.storage.delete(id).await
    }

    /// List all API keys
    ///
    /// Returns all keys (including inactive/expired) for administrative purposes.
    pub async fn list_keys(&self) -> Result<Vec<ApiKey>> {
        self.storage.list().await
    }

    /// Get a specific key by ID
    pub async fn get_key(&self, id: &str) -> Result<Option<ApiKey>> {
        self.storage.get_by_id(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::storage::MemoryKeyStorage;

    fn create_service() -> AuthService {
        let storage = Arc::new(MemoryKeyStorage::new());
        AuthService::new(storage)
    }

    #[tokio::test]
    async fn test_create_key() {
        let service = create_service();

        let response = service
            .create_key("test-key".to_string(), RateLimitTier::Free, None)
            .await
            .unwrap();

        assert!(!response.key.is_empty());
        assert!(response.key.starts_with("llm_shield_"));
        assert_eq!(response.name, "test-key");
        assert_eq!(response.tier, RateLimitTier::Free);
    }

    #[tokio::test]
    async fn test_create_key_with_expiration() {
        let service = create_service();

        let response = service
            .create_key("test-key".to_string(), RateLimitTier::Pro, Some(30))
            .await
            .unwrap();

        assert!(response.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_validate_key_success() {
        let service = create_service();

        let response = service
            .create_key("test-key".to_string(), RateLimitTier::Free, None)
            .await
            .unwrap();

        let validated = service.validate_key(&response.key).await.unwrap();

        assert_eq!(validated.id, response.id);
        assert_eq!(validated.name, "test-key");
        assert!(validated.is_valid());
    }

    #[tokio::test]
    async fn test_validate_key_invalid_format() {
        let service = create_service();

        let result = service.validate_key("invalid_key").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_key_not_found() {
        let service = create_service();

        // Valid format but not in storage
        let result = service
            .validate_key("llm_shield_abcdefghijklmnopqrstuvwxyz01234567890123")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let service = create_service();

        // Create key
        let response = service
            .create_key("test-key".to_string(), RateLimitTier::Free, None)
            .await
            .unwrap();

        // Validate it works
        assert!(service.validate_key(&response.key).await.is_ok());

        // Revoke key
        service.revoke_key(&response.id).await.unwrap();

        // Validation should fail
        let result = service.validate_key(&response.key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_key() {
        let service = create_service();

        // Create key
        let response = service
            .create_key("test-key".to_string(), RateLimitTier::Free, None)
            .await
            .unwrap();

        // Verify it exists
        assert!(service.get_key(&response.id).await.unwrap().is_some());

        // Delete key
        service.delete_key(&response.id).await.unwrap();

        // Verify it's gone
        assert!(service.get_key(&response.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let service = create_service();

        // Create multiple keys
        service
            .create_key("key1".to_string(), RateLimitTier::Free, None)
            .await
            .unwrap();
        service
            .create_key("key2".to_string(), RateLimitTier::Pro, None)
            .await
            .unwrap();
        service
            .create_key("key3".to_string(), RateLimitTier::Enterprise, None)
            .await
            .unwrap();

        let keys = service.list_keys().await.unwrap();
        assert_eq!(keys.len(), 3);
    }

    #[tokio::test]
    async fn test_validate_expired_key() {
        let service = create_service();

        // Create key that expires immediately (0 days)
        let response = service
            .create_key("expired-key".to_string(), RateLimitTier::Free, Some(0))
            .await
            .unwrap();

        // Wait a bit to ensure it's expired
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Validation should fail
        let result = service.validate_key(&response.key).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_key_from_request() {
        let service = create_service();

        let request = CreateKeyRequest {
            name: "test-key".to_string(),
            tier: RateLimitTier::Pro,
            expires_in_days: Some(30),
        };

        let response = service.create_key_from_request(request).await.unwrap();

        assert_eq!(response.name, "test-key");
        assert_eq!(response.tier, RateLimitTier::Pro);
        assert!(response.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_raw_value_cleared_after_creation() {
        let service = create_service();

        let response = service
            .create_key("test-key".to_string(), RateLimitTier::Free, None)
            .await
            .unwrap();

        // Get the stored key
        let stored_key = service.get_key(&response.id).await.unwrap().unwrap();

        // Raw value should be cleared
        assert!(stored_key.value.is_none());

        // But we can still validate with the original key
        assert!(service.validate_key(&response.key).await.is_ok());
    }
}
