//! Dashboard server

use crate::{
    api::{create_router, AppState},
    config::DashboardConfig,
    db::{migrations::run_migrations, DatabasePool},
    error::Result,
    graphql::create_schema,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tracing::{info, warn};

/// Dashboard server
pub struct DashboardServer {
    config: Arc<DashboardConfig>,
    pool: DatabasePool,
}

impl DashboardServer {
    /// Create a new dashboard server
    pub async fn new(config: DashboardConfig) -> Result<Self> {
        // Initialize logging
        Self::init_logging(&config);

        info!("Initializing LLM Shield Dashboard v{}", crate::VERSION);

        // Validate configuration
        config.validate()?;

        // Create database pool
        info!("Connecting to database...");
        let pool = DatabasePool::new(&config.database).await?;
        info!(
            "Database connection established (max_connections: {})",
            config.database.max_connections
        );

        Ok(Self {
            config: Arc::new(config),
            pool,
        })
    }

    /// Initialize logging based on configuration
    fn init_logging(config: &DashboardConfig) {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

        match config.logging.format.as_str() {
            "json" => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer().json())
                    .init();
            }
            _ => {
                tracing_subscriber::registry()
                    .with(env_filter)
                    .with(tracing_subscriber::fmt::layer())
                    .init();
            }
        }

        info!("Logging initialized (level: {}, format: {})",
              config.logging.level, config.logging.format);
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        info!("Running database migrations...");
        run_migrations(self.pool.inner()).await?;
        info!("Database migrations completed successfully");
        Ok(())
    }

    /// Start the server
    pub async fn serve(self) -> Result<()> {
        // Create GraphQL schema
        info!("Creating GraphQL schema...");
        let schema = create_schema(self.pool.clone());

        // Create application state
        let state = AppState {
            schema,
            pool: self.pool.clone(),
            config: self.config.clone(),
        };

        // Create router
        let app = create_router(state);

        // Create socket address
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let socket_addr: SocketAddr = addr.parse()
            .map_err(|e| crate::error::DashboardError::Configuration(
                format!("Invalid server address: {}", e)
            ))?;

        info!("Starting server on {}", socket_addr);
        info!("GraphQL endpoint: http://{}/graphql", socket_addr);
        info!("GraphQL playground: http://{}/graphql/playground", socket_addr);
        info!("Health check: http://{}/health", socket_addr);

        // Create TCP listener
        let listener = TcpListener::bind(socket_addr).await
            .map_err(|e| crate::error::DashboardError::Io(e))?;

        // Serve with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| crate::error::DashboardError::Server(e.to_string()))?;

        info!("Server stopped gracefully");
        Ok(())
    }

    /// Get database pool
    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    /// Get configuration
    pub fn config(&self) -> &DashboardConfig {
        &self.config
    }
}

/// Shutdown signal handler
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }

    info!("Starting graceful shutdown...");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AuthConfig, CorsConfig, DatabaseConfig, LoggingConfig, ServerConfig};

    fn create_test_config() -> DashboardConfig {
        DashboardConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 0, // Use port 0 for testing to get a random available port
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
                jwt_secret: "test_secret_that_is_long_enough".to_string(),
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
                format: "text".to_string(),
            },
        }
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_server_creation() {
        let config = create_test_config();
        let server = DashboardServer::new(config).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires database
    async fn test_server_migration() {
        let config = create_test_config();
        let server = DashboardServer::new(config).await.unwrap();
        let result = server.migrate().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config() {
        let mut config = create_test_config();
        config.auth.jwt_secret = String::new(); // Empty secret should fail validation
        assert!(config.validate().is_err());
    }
}
