//! Authentication and authorization

use crate::error::{DashboardError, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,        // User ID
    pub tenant_id: Uuid,  // Tenant ID
    pub role: String,     // User role
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
}

/// Generate JWT token
pub fn generate_token(
    user_id: Uuid,
    tenant_id: Uuid,
    role: &str,
    secret: &str,
    expiration_secs: u64,
) -> Result<String> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + expiration_secs as usize;

    let claims = Claims {
        sub: user_id,
        tenant_id,
        role: role.to_string(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| DashboardError::Jwt(e))
}

/// Verify JWT token
pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| DashboardError::Jwt(e))
}

/// Hash password
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| DashboardError::Authentication(e.to_string()))
}

/// Verify password
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(password_hash).map_err(|e| DashboardError::Authentication(e.to_string()))?;

    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

/// Generate API key
pub fn generate_api_key() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const KEY_LENGTH: usize = 32;

    let mut rng = rand::thread_rng();
    let key: String = (0..KEY_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("llms_{}", key)
}

/// Hash API key
pub fn hash_api_key(api_key: &str) -> Result<String> {
    hash_password(api_key)
}

/// Verify API key against database
pub async fn verify_api_key(
    pool: &sqlx::PgPool,
    api_key: &str,
) -> Result<Claims> {
    use crate::models::ApiKey;

    // Query all active API keys for comparison
    // Note: In production, you might want to add an index on key_hash for performance
    let api_keys = sqlx::query_as!(
        ApiKey,
        r#"SELECT id, tenant_id, user_id, key_hash, name, permissions,
           last_used_at, expires_at, created_at
           FROM api_keys
           WHERE expires_at IS NULL OR expires_at > NOW()"#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DashboardError::Database(e))?;

    // Find matching API key by verifying hash
    for stored_key in api_keys {
        if verify_password(api_key, &stored_key.key_hash)? {
            // Check if key is expired
            if let Some(expires_at) = stored_key.expires_at {
                let now = chrono::Utc::now();
                if expires_at < now {
                    return Err(DashboardError::Authentication(
                        "API key has expired".to_string(),
                    ));
                }
            }

            // Update last_used_at timestamp
            let _ = sqlx::query!(
                "UPDATE api_keys SET last_used_at = NOW() WHERE id = $1",
                stored_key.id
            )
            .execute(pool)
            .await; // Ignore errors for last_used_at update

            // Create claims from API key
            // For API keys without a user_id, we use the tenant's default user
            let user_id = stored_key.user_id.unwrap_or(stored_key.tenant_id);

            // API keys get "developer" role by default
            // In production, you might want to store role in permissions
            let role = if stored_key.permissions.contains(&"admin".to_string()) {
                "tenant_admin"
            } else {
                "developer"
            };

            let now = chrono::Utc::now().timestamp() as usize;
            return Ok(Claims {
                sub: user_id,
                tenant_id: stored_key.tenant_id,
                role: role.to_string(),
                exp: now + 3600, // 1 hour validity for API key session
                iat: now,
            });
        }
    }

    Err(DashboardError::Authentication(
        "Invalid API key".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_token() {
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let secret = "test_secret";

        let token = generate_token(user_id, tenant_id, "developer", secret, 3600).unwrap();
        assert!(!token.is_empty());

        let claims = verify_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.tenant_id, tenant_id);
        assert_eq!(claims.role, "developer");
    }

    #[test]
    fn test_invalid_token() {
        let result = verify_token("invalid.token.here", "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_and_verify_password() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        assert!(!hash.is_empty());

        let valid = verify_password(password, &hash).unwrap();
        assert!(valid);

        let invalid = verify_password("wrong_password", &hash).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_generate_api_key() {
        let key = generate_api_key();
        assert!(key.starts_with("llms_"));
        assert_eq!(key.len(), 37); // "llms_" + 32 chars
    }

    #[test]
    fn test_hash_api_key() {
        let api_key = "llms_test1234567890";
        let hash = hash_api_key(api_key).unwrap();
        assert!(!hash.is_empty());
        assert_ne!(hash, api_key);
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_verify_api_key() {
        use crate::models::ApiKey;
        use chrono::Utc;
        use sqlx::PgPool;

        // This test would require a database connection
        // In a real test environment, you would:
        // 1. Create a test database pool
        // 2. Insert a test API key with hash
        // 3. Verify the API key
        // 4. Assert the claims are correct
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_verify_api_key_expired() {
        // Test that expired API keys are rejected
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_verify_api_key_invalid() {
        // Test that invalid API keys are rejected
    }
}
