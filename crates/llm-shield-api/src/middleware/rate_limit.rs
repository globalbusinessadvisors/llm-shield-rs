//! Rate limiting middleware for Axum
//!
//! ## SPARC Phase 3: Construction (TDD)
//!
//! Axum middleware that enforces rate limits before processing requests.

use crate::config::rate_limit::RateLimitTier;
use crate::rate_limiting::{ConcurrentLimiter, MultiTierRateLimiter, RateLimiter};
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Extension to store the client's rate limit tier
#[derive(Debug, Clone)]
pub struct ClientTier(pub RateLimitTier);

/// Rate limit middleware layer
///
/// ## Features
///
/// - Enforces per-minute, per-hour, and per-day limits
/// - Limits concurrent requests per client
/// - Adds rate limit headers to response
/// - Returns 429 when limits exceeded
///
/// ## Headers Added
///
/// - `X-RateLimit-Limit`: Maximum requests per window
/// - `X-RateLimit-Remaining`: Requests remaining in window
/// - `X-RateLimit-Reset`: Unix timestamp when limit resets
/// - `Retry-After`: Seconds until retry allowed (on 429)
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
///         rate_limit_middleware
///     ));
/// ```
pub async fn rate_limit_middleware(
    rate_limiter: Arc<MultiTierRateLimiter>,
    concurrent_limiter: Arc<ConcurrentLimiter>,
    request: Request,
    next: Next,
) -> Response {
    // Extract client identifier (API key or IP address)
    let client_key = extract_client_key(&request);

    // Extract tier (from auth middleware or default to Free)
    let tier = request
        .extensions()
        .get::<ClientTier>()
        .map(|t| t.0)
        .unwrap_or(RateLimitTier::Free);

    // Check rate limit
    let decision = rate_limiter.check_rate_limit(&client_key, tier).await;

    if !decision.allowed {
        // Rate limit exceeded - return 429
        return create_rate_limit_response(decision);
    }

    // Check concurrent limit
    let limits = match tier {
        RateLimitTier::Free => 2,
        RateLimitTier::Pro => 10,
        RateLimitTier::Enterprise => 50,
    };

    let permit = concurrent_limiter.try_acquire(&client_key, limits).await;

    if permit.is_none() {
        // Too many concurrent requests
        return create_concurrent_limit_response(decision);
    }

    // Process request
    let mut response = next.run(request).await;

    // Add rate limit headers
    add_rate_limit_headers(response.headers_mut(), &decision);

    response
}

/// Extract client identifier from request
fn extract_client_key(request: &Request) -> String {
    // Try to get API key from Authorization header
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                return key.to_string();
            }
        }
    }

    // Fall back to IP address
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .unwrap_or("unknown")
        .to_string()
}

/// Add rate limit headers to response
fn add_rate_limit_headers(headers: &mut HeaderMap, decision: &crate::rate_limiting::RateLimitDecision) {
    headers.insert(
        "X-RateLimit-Limit",
        decision.limit.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        decision.remaining.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Reset",
        decision.reset_at.to_string().parse().unwrap(),
    );
}

/// Create 429 response for rate limit exceeded
fn create_rate_limit_response(decision: crate::rate_limiting::RateLimitDecision) -> Response {
    let mut headers = HeaderMap::new();
    add_rate_limit_headers(&mut headers, &decision);

    if let Some(retry_after) = decision.retry_after {
        headers.insert("Retry-After", retry_after.to_string().parse().unwrap());
    }

    let body = serde_json::json!({
        "error": "Rate limit exceeded",
        "message": format!("You have exceeded the rate limit of {} requests per minute", decision.limit),
        "retry_after": decision.retry_after,
        "reset_at": decision.reset_at,
    });

    (
        StatusCode::TOO_MANY_REQUESTS,
        headers,
        serde_json::to_string(&body).unwrap(),
    )
        .into_response()
}

/// Create 429 response for concurrent limit exceeded
fn create_concurrent_limit_response(decision: crate::rate_limiting::RateLimitDecision) -> Response {
    let mut headers = HeaderMap::new();
    add_rate_limit_headers(&mut headers, &decision);

    let body = serde_json::json!({
        "error": "Too many concurrent requests",
        "message": "You have too many concurrent requests. Please wait for some to complete.",
        "retry_after": 1,
    });

    (
        StatusCode::TOO_MANY_REQUESTS,
        headers,
        serde_json::to_string(&body).unwrap(),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for oneshot

    #[test]
    fn test_extract_client_key_from_auth_header() {
        let request = Request::builder()
            .header("authorization", "Bearer test_key_123")
            .body(Body::empty())
            .unwrap();

        let key = extract_client_key(&request);
        assert_eq!(key, "test_key_123");
    }

    #[test]
    fn test_extract_client_key_from_ip() {
        let request = Request::builder()
            .header("x-forwarded-for", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let key = extract_client_key(&request);
        assert_eq!(key, "192.168.1.1");
    }

    #[test]
    fn test_extract_client_key_fallback() {
        let request = Request::builder().body(Body::empty()).unwrap();

        let key = extract_client_key(&request);
        assert_eq!(key, "unknown");
    }

    #[test]
    fn test_add_rate_limit_headers() {
        use crate::rate_limiting::RateLimitDecision;

        let decision = RateLimitDecision::allow(100, 95, 1234567890);
        let mut headers = HeaderMap::new();

        add_rate_limit_headers(&mut headers, &decision);

        assert_eq!(headers.get("X-RateLimit-Limit").unwrap(), "100");
        assert_eq!(headers.get("X-RateLimit-Remaining").unwrap(), "95");
        assert_eq!(headers.get("X-RateLimit-Reset").unwrap(), "1234567890");
    }

    #[test]
    fn test_create_rate_limit_response() {
        use crate::rate_limiting::RateLimitDecision;

        let decision = RateLimitDecision::deny(100, 1234567890, 60);
        let response = create_rate_limit_response(decision);

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);

        let headers = response.headers();
        assert_eq!(headers.get("X-RateLimit-Limit").unwrap(), "100");
        assert_eq!(headers.get("Retry-After").unwrap(), "60");
    }
}
