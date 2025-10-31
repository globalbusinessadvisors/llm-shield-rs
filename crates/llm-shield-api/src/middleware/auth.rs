//! Authentication middleware for Axum
//!
//! ## SPARC Phase 3: Construction (TDD)
//!
//! Axum middleware that validates API keys before processing requests.

use crate::auth::AuthService;
use crate::middleware::rate_limit::ClientTier;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Authenticated user information
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// API key ID
    pub key_id: String,

    /// Key name
    pub name: String,

    /// Rate limit tier
    pub tier: crate::config::rate_limit::RateLimitTier,
}

/// Authentication middleware layer
///
/// ## Features
///
/// - Extracts API key from Authorization header
/// - Validates key using AuthService
/// - Checks expiration and active status
/// - Adds user info to request extensions
/// - Returns 401 for invalid/missing keys
///
/// ## Header Format
///
/// ```
/// Authorization: Bearer llm_shield_<40 characters>
/// ```
///
/// ## Example
///
/// ```rust,ignore
/// use axum::Router;
/// use axum::middleware;
///
/// let router = Router::new()
///     .route("/api/scan", post(scan_handler))
///     .layer(middleware::from_fn_with_state(
///         app_state.clone(),
///         auth_middleware
///     ));
/// ```
pub async fn auth_middleware(
    auth_service: Arc<AuthService>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    let api_key = match auth_header {
        Some(header) => {
            // Extract Bearer token
            if let Some(key) = header.strip_prefix("Bearer ") {
                key
            } else {
                return create_unauthorized_response("Invalid authorization header format");
            }
        }
        None => {
            return create_unauthorized_response("Missing authorization header");
        }
    };

    // Validate API key
    let key = match auth_service.validate_key(api_key).await {
        Ok(key) => key,
        Err(e) => {
            tracing::warn!("API key validation failed: {}", e);
            return create_unauthorized_response("Invalid or expired API key");
        }
    };

    // Add authenticated user to request extensions
    let user = AuthenticatedUser {
        key_id: key.id.clone(),
        name: key.name.clone(),
        tier: key.tier,
    };

    // Also add tier for rate limiting middleware
    request.extensions_mut().insert(ClientTier(key.tier));
    request.extensions_mut().insert(user);

    // Log successful authentication
    tracing::debug!(
        "Authenticated request from key: {} ({})",
        key.id,
        key.name
    );

    // Process request
    next.run(request).await
}

/// Create 401 Unauthorized response
fn create_unauthorized_response(message: &str) -> Response {
    let body = serde_json::json!({
        "error": "Unauthorized",
        "message": message,
    });

    (
        StatusCode::UNAUTHORIZED,
        serde_json::to_string(&body).unwrap(),
    )
        .into_response()
}

/// Optional authentication middleware
///
/// Similar to `auth_middleware` but allows requests without authentication.
/// If authentication is provided, it validates and adds user info.
/// If not provided, the request proceeds without user info.
///
/// Use this for endpoints that have both public and authenticated modes.
pub async fn optional_auth_middleware(
    auth_service: Arc<AuthService>,
    mut request: Request,
    next: Next,
) -> Response {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(header) = auth_header {
        // Extract Bearer token
        if let Some(api_key) = header.strip_prefix("Bearer ") {
            // Validate API key
            if let Ok(key) = auth_service.validate_key(api_key).await {
                // Add authenticated user to request extensions
                let user = AuthenticatedUser {
                    key_id: key.id.clone(),
                    name: key.name.clone(),
                    tier: key.tier,
                };

                request.extensions_mut().insert(ClientTier(key.tier));
                request.extensions_mut().insert(user);

                tracing::debug!(
                    "Authenticated optional request from key: {} ({})",
                    key.id,
                    key.name
                );
            }
        }
    }

    // Process request (with or without authentication)
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{storage::MemoryKeyStorage, AuthService};
    use crate::config::rate_limit::RateLimitTier;
    use axum::http::{Request, StatusCode};

    async fn create_test_service_and_key() -> (Arc<AuthService>, String) {
        let storage = Arc::new(MemoryKeyStorage::new());
        let service = Arc::new(AuthService::new(storage));

        let response = service
            .create_key("test-key".to_string(), RateLimitTier::Pro, None)
            .await
            .unwrap();

        (service, response.key)
    }

    #[test]
    fn test_create_unauthorized_response() {
        let response = create_unauthorized_response("Test message");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_authenticated_user_creation() {
        let user = AuthenticatedUser {
            key_id: "test-id".to_string(),
            name: "test-name".to_string(),
            tier: RateLimitTier::Pro,
        };

        assert_eq!(user.key_id, "test-id");
        assert_eq!(user.name, "test-name");
        assert_eq!(user.tier, RateLimitTier::Pro);
    }

    // Note: Full integration tests for middleware would require:
    // - Setting up an Axum test server
    // - Making actual HTTP requests
    // - These are better suited for integration tests
    // Unit tests above verify the core types and functions
}
