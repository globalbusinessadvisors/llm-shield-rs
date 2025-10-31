//! Authentication types and models
//!
//! ## SPARC Phase 3: Construction (TDD - RED Phase)
//!
//! Core data structures for API key authentication.

use crate::config::rate_limit::RateLimitTier;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use llm_shield_core::Error;

/// Result type for auth operations
pub type Result<T> = std::result::Result<T, Error>;

/// API key with metadata
///
/// ## Security
///
/// - `value`: Raw key value (only shown once during creation)
/// - `hashed_value`: Argon2id hash (stored in database)
/// - Use constant-time comparison for validation
///
/// ## Invariants
///
/// - `value` format: `llm_shield_[a-zA-Z0-9]{40}`
/// - `hashed_value` is argon2id hash of `value`
/// - `is_valid() == active && !is_expired()`
/// - `created_at <= expires_at` (if set)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique identifier (UUID)
    pub id: String,

    /// Human-readable name/description
    pub name: String,

    /// The raw key value (only available during creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Hashed key value (argon2id)
    pub hashed_value: String,

    /// Rate limit tier
    pub tier: RateLimitTier,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Optional expiration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// Whether key is active
    pub active: bool,
}

impl ApiKey {
    /// Create a new API key (raw value will be hashed)
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name
    /// * `tier` - Rate limit tier
    /// * `expires_at` - Optional expiration date
    ///
    /// # Returns
    ///
    /// New API key with raw value and hashed value
    pub fn new(
        name: String,
        tier: RateLimitTier,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let raw_value = Self::generate_key_value()?;
        let hashed_value = Self::hash_key(&raw_value)?;

        Ok(Self {
            id,
            name,
            value: Some(raw_value),
            hashed_value,
            tier,
            created_at: Utc::now(),
            expires_at,
            active: true,
        })
    }

    /// Generate a cryptographically secure random key value
    ///
    /// Format: `llm_shield_<40 character alphanumeric string>`
    fn generate_key_value() -> Result<String> {
        use rand::Rng;

        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();

        let key_suffix: String = (0..40)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        Ok(format!("llm_shield_{}", key_suffix))
    }

    /// Hash a key value using argon2id
    fn hash_key(key: &str) -> Result<String> {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(key.as_bytes(), &salt)
            .map_err(|e| Error::auth(format!("Failed to hash key: {}", e)))?;

        Ok(hash.to_string())
    }

    /// Verify a raw key against this stored key
    ///
    /// Uses constant-time comparison to prevent timing attacks.
    pub fn verify(&self, raw_key: &str) -> Result<bool> {
        use argon2::{password_hash::PasswordVerifier, Argon2};

        let argon2 = Argon2::default();

        // Parse the stored hash
        let parsed_hash = argon2::password_hash::PasswordHash::new(&self.hashed_value)
            .map_err(|e| Error::auth(format!("Invalid stored hash: {}", e)))?;

        // Verify using constant-time comparison
        Ok(argon2
            .verify_password(raw_key.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Check if key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if key is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        self.active && !self.is_expired()
    }

    /// Clear the raw value (for security)
    ///
    /// Should be called after the key has been shown to the user once.
    pub fn clear_value(&mut self) {
        self.value = None;
    }

    /// Validate key format
    pub fn validate_format(key: &str) -> bool {
        key.starts_with("llm_shield_") && key.len() == 51 // "llm_shield_" + 40 chars
    }
}

/// Request to create a new API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    /// Human-readable name
    pub name: String,

    /// Rate limit tier
    pub tier: RateLimitTier,

    /// Optional expiration in days from now
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in_days: Option<u32>,
}

/// Response after creating an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    /// The raw API key (only shown once!)
    pub key: String,

    /// Key ID
    pub id: String,

    /// Key name
    pub name: String,

    /// Rate limit tier
    pub tier: RateLimitTier,

    /// Creation timestamp
    pub created_at: String,

    /// Optional expiration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

impl From<ApiKey> for CreateKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            key: key.value.unwrap_or_default(),
            id: key.id,
            name: key.name,
            tier: key.tier,
            created_at: key.created_at.to_rfc3339(),
            expires_at: key.expires_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_new() {
        let key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        assert!(!key.id.is_empty());
        assert_eq!(key.name, "test-key");
        assert!(key.value.is_some());
        assert!(!key.hashed_value.is_empty());
        assert_eq!(key.tier, RateLimitTier::Free);
        assert!(key.active);
        assert!(!key.is_expired());
    }

    #[test]
    fn test_api_key_format() {
        let key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        let raw_value = key.value.as_ref().unwrap();
        assert!(raw_value.starts_with("llm_shield_"));
        assert_eq!(raw_value.len(), 51); // "llm_shield_" (11) + 40 chars
        assert!(ApiKey::validate_format(raw_value));
    }

    #[test]
    fn test_api_key_verify_success() {
        let key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        let raw_value = key.value.as_ref().unwrap().clone();
        assert!(key.verify(&raw_value).unwrap());
    }

    #[test]
    fn test_api_key_verify_failure() {
        let key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        assert!(!key.verify("wrong_key").unwrap());
    }

    #[test]
    fn test_api_key_expiration() {
        use chrono::Duration;

        // Key expired 1 day ago
        let expired_key = ApiKey::new(
            "expired-key".to_string(),
            RateLimitTier::Free,
            Some(Utc::now() - Duration::days(1)),
        )
        .unwrap();

        assert!(expired_key.is_expired());
        assert!(!expired_key.is_valid());

        // Key expires in 1 day
        let valid_key = ApiKey::new(
            "valid-key".to_string(),
            RateLimitTier::Free,
            Some(Utc::now() + Duration::days(1)),
        )
        .unwrap();

        assert!(!valid_key.is_expired());
        assert!(valid_key.is_valid());
    }

    #[test]
    fn test_api_key_inactive() {
        let mut key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        assert!(key.is_valid());

        key.active = false;
        assert!(!key.is_valid());
    }

    #[test]
    fn test_api_key_clear_value() {
        let mut key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        assert!(key.value.is_some());

        key.clear_value();
        assert!(key.value.is_none());
    }

    #[test]
    fn test_validate_format() {
        assert!(ApiKey::validate_format(
            "llm_shield_abcdefghijklmnopqrstuvwxyz01234567890123"
        ));
        assert!(!ApiKey::validate_format("invalid_key"));
        assert!(!ApiKey::validate_format("llm_shield_short"));
    }

    #[test]
    fn test_create_key_response_from_api_key() {
        let key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Pro,
            None,
        )
        .unwrap();

        let response = CreateKeyResponse::from(key.clone());

        assert_eq!(response.key, key.value.unwrap());
        assert_eq!(response.id, key.id);
        assert_eq!(response.name, key.name);
        assert_eq!(response.tier, RateLimitTier::Pro);
    }

    #[test]
    fn test_hash_is_different_from_raw() {
        let key = ApiKey::new(
            "test-key".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        let raw_value = key.value.as_ref().unwrap();
        assert_ne!(raw_value, &key.hashed_value);
        assert!(key.hashed_value.starts_with("$argon2"));
    }

    #[test]
    fn test_different_keys_have_different_values() {
        let key1 = ApiKey::new(
            "key1".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        let key2 = ApiKey::new(
            "key2".to_string(),
            RateLimitTier::Free,
            None,
        )
        .unwrap();

        assert_ne!(key1.value, key2.value);
        assert_ne!(key1.hashed_value, key2.hashed_value);
    }
}
