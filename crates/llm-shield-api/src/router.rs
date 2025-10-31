//! Route configuration

use axum::{routing::get, Router};

use crate::handlers;

/// Create the application router
///
/// ## Routes
/// - GET /health - Basic health check
/// - GET /health/ready - Readiness probe
/// - GET /health/live - Liveness probe
/// - GET /version - Version information
pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/health/ready", get(handlers::ready))
        .route("/health/live", get(handlers::live))
        .route("/version", get(handlers::version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // For `oneshot`

    #[tokio::test]
    async fn test_health_route() {
        let app = create_router();

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_route() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_live_route() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_version_route() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/version")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_not_found() {
        let app = create_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/notfound")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
