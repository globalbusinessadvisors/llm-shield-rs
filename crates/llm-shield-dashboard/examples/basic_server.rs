//! Basic dashboard server example
//!
//! This example demonstrates how to start the LLM Shield Dashboard with default configuration.
//!
//! ## Running
//!
//! ```bash
//! # Set up environment variables
//! export DASHBOARD__DATABASE__URL="postgres://user:password@localhost:5432/llm_shield"
//! export DASHBOARD__AUTH__JWT_SECRET="your-super-secret-jwt-key-min-32-chars"
//!
//! # Run the example
//! cargo run --example basic_server
//! ```
//!
//! ## Testing
//!
//! Once the server is running, you can test it with:
//!
//! ```bash
//! # Health check
//! curl http://localhost:8080/health
//!
//! # GraphQL playground (requires authentication)
//! open http://localhost:8080/graphql/playground
//! ```

use llm_shield_dashboard::{
    config::{AuthConfig, CorsConfig, DashboardConfig, DatabaseConfig, LoggingConfig, ServerConfig},
    DashboardServer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    // In production, use environment variables or a config file
    let config = DashboardConfig {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            timeout_secs: 30,
            max_body_size: 10 * 1024 * 1024, // 10MB
        },
        database: DatabaseConfig {
            url: std::env::var("DASHBOARD__DATABASE__URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/llm_shield".to_string()),
            max_connections: 20,
            min_connections: 5,
            connection_timeout_secs: 30,
        },
        auth: AuthConfig {
            jwt_secret: std::env::var("DASHBOARD__AUTH__JWT_SECRET")
                .unwrap_or_else(|_| "your-super-secret-jwt-key-min-32-chars".to_string()),
            jwt_expiration_secs: 900,           // 15 minutes
            refresh_token_expiration_secs: 604800, // 7 days
            enable_api_keys: true,
            api_key_header: "X-API-Key".to_string(),
        },
        cors: CorsConfig {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
            ],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-API-Key".to_string(),
            ],
            allow_credentials: true,
        },
        logging: LoggingConfig {
            level: std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            format: "json".to_string(),
        },
    };

    println!("🚀 Starting LLM Shield Dashboard...");

    // Create and start server
    let server = DashboardServer::new(config).await?;

    // Run migrations
    println!("📦 Running database migrations...");
    server.migrate().await?;

    println!("✅ Server ready!");
    println!();
    println!("Endpoints:");
    println!("  - Health:      http://localhost:8080/health");
    println!("  - GraphQL:     http://localhost:8080/graphql");
    println!("  - Playground:  http://localhost:8080/graphql/playground");
    println!();
    println!("Press Ctrl+C to stop");

    // Start server
    server.serve().await?;

    Ok(())
}
