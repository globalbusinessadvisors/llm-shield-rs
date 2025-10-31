//! API routes and handlers

use crate::{
    config::DashboardConfig,
    db::DatabasePool,
    graphql::DashboardSchema,
    health::{health_check, liveness_check, readiness_check},
    middleware::{auth_middleware, AuthState},
};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    http::{header, Method, StatusCode},
    middleware,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub schema: DashboardSchema,
    pub pool: DatabasePool,
    pub config: Arc<DashboardConfig>,
}

/// GraphQL handler
async fn graphql_handler(
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

/// GraphQL playground handler
async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

/// Create the API router
pub fn create_router(state: AppState) -> Router {
    let config = state.config.clone();

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .cors
                .allowed_origins
                .iter()
                .filter_map(|origin| origin.parse().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods(
            config
                .cors
                .allowed_methods
                .iter()
                .filter_map(|method| method.parse::<Method>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_headers(
            config
                .cors
                .allowed_headers
                .iter()
                .filter_map(|header_name| header_name.parse().ok())
                .collect::<Vec<_>>(),
        )
        .allow_credentials(config.cors.allow_credentials);

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(readiness_check))
        .route("/health/live", get(liveness_check))
        .with_state(state.pool.clone());

    // Create auth state for middleware
    let auth_state = AuthState {
        config: state.config.clone(),
        pool: state.pool.clone(),
    };

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql/playground", get(graphql_playground))
        .layer(middleware::from_fn_with_state(
            auth_state,
            auth_middleware,
        ))
        .with_state(state.clone());

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
}

/// Root handler
async fn root_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        r#"{"message":"LLM Shield Dashboard API","version":"0.1.0","graphql":"/graphql","playground":"/graphql/playground"}"#,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        auth::generate_token,
        config::{AuthConfig, CorsConfig, DatabaseConfig, LoggingConfig, ServerConfig},
        graphql::create_schema,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    fn create_test_config() -> DashboardConfig {
        DashboardConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_secs: 30,
                max_body_size: 1024 * 1024,
            },
            database: DatabaseConfig {
                url: "postgres://test:test@localhost/test".to_string(),
                max_connections: 10,
                min_connections: 2,
                connection_timeout_secs: 30,
            },
            auth: AuthConfig {
                jwt_secret: "test_secret".to_string(),
                jwt_expiration_secs: 900,
                refresh_token_expiration_secs: 604800,
                enable_api_keys: true,
                api_key_header: "X-API-Key".to_string(),
            },
            cors: CorsConfig {
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string()],
                allowed_headers: vec!["*".to_string()],
                allow_credentials: false,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_public_routes_accessible() {
        let config = create_test_config();
        let pool = DatabasePool::new(&config.database).await.unwrap_or_else(|_| {
            // For testing without a real database
            panic!("Database connection required for integration tests");
        });

        let schema = create_schema(pool.clone());
        let state = AppState {
            schema,
            pool: pool.clone(),
            config: Arc::new(config),
        };

        let app = create_router(state);

        // Test health endpoint
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
    #[ignore] // Requires database
    async fn test_protected_routes_require_auth() {
        let config = create_test_config();
        let pool = DatabasePool::new(&config.database).await.unwrap();

        let schema = create_schema(pool.clone());
        let state = AppState {
            schema,
            pool: pool.clone(),
            config: Arc::new(config),
        };

        let app = create_router(state);

        // Test GraphQL endpoint without authentication
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"query":"{ health }"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_protected_routes_with_valid_token() {
        let config = create_test_config();
        let pool = DatabasePool::new(&config.database).await.unwrap();

        let schema = create_schema(pool.clone());
        let state = AppState {
            schema,
            pool: pool.clone(),
            config: Arc::new(config.clone()),
        };

        let app = create_router(state);

        // Generate valid token
        let token = generate_token(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "developer",
            &config.auth.jwt_secret,
            900,
        )
        .unwrap();

        // Test GraphQL endpoint with authentication
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::from(r#"{"query":"{ health }"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_app_state_clone() {
        // Test that AppState can be cloned
        let config = create_test_config();
        // This test just verifies the structure compiles
        assert_eq!(config.server.port, 8080);
    }
}
