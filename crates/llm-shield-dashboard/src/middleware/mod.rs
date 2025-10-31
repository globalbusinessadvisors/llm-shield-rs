//! Middleware

use crate::{
    auth::{verify_api_key, verify_token, Claims},
    config::DashboardConfig,
    db::DatabasePool,
    error::DashboardError,
};
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Authentication middleware state
#[derive(Clone)]
pub struct AuthState {
    pub config: Arc<DashboardConfig>,
    pub pool: DatabasePool,
}

/// Authentication middleware
pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AuthState>,
    mut request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Extract token from Authorization header
    let token = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if let Some(token) = token {
        match verify_token(token, &state.config.auth.jwt_secret) {
            Ok(claims) => {
                // Add claims to request extensions
                request.extensions_mut().insert(claims);
                return Ok(next.run(request).await);
            }
            Err(_) => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    "Invalid or expired token".to_string(),
                ))
            }
        }
    }

    // Check for API key
    if state.config.auth.enable_api_keys {
        let api_key = request
            .headers()
            .get(&state.config.auth.api_key_header)
            .and_then(|value| value.to_str().ok());

        if let Some(api_key) = api_key {
            match verify_api_key(state.pool.inner(), api_key).await {
                Ok(claims) => {
                    // Add claims to request extensions
                    request.extensions_mut().insert(claims);
                    return Ok(next.run(request).await);
                }
                Err(_) => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        "Invalid or expired API key".to_string(),
                    ))
                }
            }
        }
    }

    Err((StatusCode::UNAUTHORIZED, "Missing authentication".to_string()))
}

/// Extract claims from request
pub fn extract_claims(request: &Request) -> Result<&Claims, DashboardError> {
    request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| DashboardError::Authentication("No claims found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::generate_token;
    use axum::body::Body;
    use axum::http;
    use uuid::Uuid;

    #[test]
    fn test_extract_claims() {
        let mut request = Request::new(Body::empty());

        let claims = Claims {
            sub: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            role: "developer".to_string(),
            exp: 9999999999,
            iat: 1000000000,
        };

        request.extensions_mut().insert(claims.clone());

        let extracted = extract_claims(&request).unwrap();
        assert_eq!(extracted.sub, claims.sub);
        assert_eq!(extracted.role, claims.role);
    }

    #[test]
    fn test_extract_claims_missing() {
        let request = Request::new(Body::empty());
        let result = extract_claims(&request);
        assert!(result.is_err());
    }
}
