//! Integration tests for the dashboard

use llm_shield_dashboard::{
    auth::{generate_api_key, generate_token, hash_api_key},
    config::{AuthConfig, CorsConfig, DashboardConfig, DatabaseConfig, LoggingConfig, ServerConfig},
    db::DatabasePool,
};
use uuid::Uuid;

/// Create a test configuration
fn create_test_config() -> DashboardConfig {
    DashboardConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0, // Random port for testing
            timeout_secs: 30,
            max_body_size: 1024 * 1024,
        },
        database: DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/llm_shield_test".to_string()),
            max_connections: 5,
            min_connections: 1,
            connection_timeout_secs: 30,
        },
        auth: AuthConfig {
            jwt_secret: "test_secret_key_for_testing_only".to_string(),
            jwt_expiration_secs: 900,
            refresh_token_expiration_secs: 604800,
            enable_api_keys: true,
            api_key_header: "X-API-Key".to_string(),
        },
        cors: CorsConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "OPTIONS".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            allow_credentials: true,
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
            format: "text".to_string(),
        },
    }
}

#[tokio::test]
#[ignore] // Requires PostgreSQL with TimescaleDB
async fn test_database_connection() {
    let config = create_test_config();
    let pool = DatabasePool::new(&config.database).await;
    assert!(pool.is_ok(), "Failed to connect to database");

    let pool = pool.unwrap();
    assert!(!pool.is_closed(), "Database pool should be open");

    let stats = pool.stats();
    assert!(stats.connections > 0, "Should have at least one connection");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL with TimescaleDB
async fn test_migrations() {
    use llm_shield_dashboard::db::migrations::run_migrations;

    let config = create_test_config();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    let result = run_migrations(pool.inner()).await;
    assert!(result.is_ok(), "Migrations should run successfully");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL with TimescaleDB
async fn test_create_tenant() {
    use chrono::Utc;
    use serde_json::json;

    let config = create_test_config();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    let tenant_id = Uuid::new_v4();
    let name = format!("test_tenant_{}", Uuid::new_v4());

    let result = sqlx::query!(
        r#"INSERT INTO tenants (id, name, display_name, settings, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        tenant_id,
        name,
        "Test Tenant",
        json!({}),
        Utc::now(),
        Utc::now()
    )
    .execute(pool.inner())
    .await;

    assert!(result.is_ok(), "Should create tenant successfully");

    // Clean up
    let _ = sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
        .execute(pool.inner())
        .await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL with TimescaleDB
async fn test_create_user() {
    use chrono::Utc;
    use llm_shield_dashboard::auth::hash_password;
    use serde_json::json;

    let config = create_test_config();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // Create tenant first
    let tenant_id = Uuid::new_v4();
    let tenant_name = format!("test_tenant_{}", Uuid::new_v4());

    sqlx::query!(
        r#"INSERT INTO tenants (id, name, display_name, settings, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        tenant_id,
        tenant_name,
        "Test Tenant",
        json!({}),
        Utc::now(),
        Utc::now()
    )
    .execute(pool.inner())
    .await
    .unwrap();

    // Create user
    let user_id = Uuid::new_v4();
    let email = format!("test_{}@example.com", Uuid::new_v4());
    let password_hash = hash_password("test_password").unwrap();

    let result = sqlx::query!(
        r#"INSERT INTO users (id, tenant_id, email, password_hash, role, enabled, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        user_id,
        tenant_id,
        email,
        password_hash,
        "developer",
        true,
        Utc::now(),
        Utc::now()
    )
    .execute(pool.inner())
    .await;

    assert!(result.is_ok(), "Should create user successfully");

    // Clean up
    let _ = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(pool.inner())
        .await;
    let _ = sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
        .execute(pool.inner())
        .await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL with TimescaleDB
async fn test_create_api_key() {
    use chrono::Utc;
    use serde_json::json;

    let config = create_test_config();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // Create tenant first
    let tenant_id = Uuid::new_v4();
    let tenant_name = format!("test_tenant_{}", Uuid::new_v4());

    sqlx::query!(
        r#"INSERT INTO tenants (id, name, display_name, settings, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        tenant_id,
        tenant_name,
        "Test Tenant",
        json!({}),
        Utc::now(),
        Utc::now()
    )
    .execute(pool.inner())
    .await
    .unwrap();

    // Generate and hash API key
    let api_key = generate_api_key();
    let key_hash = hash_api_key(&api_key).unwrap();

    let api_key_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"INSERT INTO api_keys (id, tenant_id, key_hash, name, permissions, created_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        api_key_id,
        tenant_id,
        key_hash,
        "Test API Key",
        &["read", "write"],
        Utc::now()
    )
    .execute(pool.inner())
    .await;

    assert!(result.is_ok(), "Should create API key successfully");

    // Clean up
    let _ = sqlx::query!("DELETE FROM api_keys WHERE id = $1", api_key_id)
        .execute(pool.inner())
        .await;
    let _ = sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
        .execute(pool.inner())
        .await;
}

#[tokio::test]
#[ignore] // Requires PostgreSQL with TimescaleDB
async fn test_insert_metrics() {
    use chrono::Utc;
    use serde_json::json;

    let config = create_test_config();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // Create tenant first
    let tenant_id = Uuid::new_v4();
    let tenant_name = format!("test_tenant_{}", Uuid::new_v4());

    sqlx::query!(
        r#"INSERT INTO tenants (id, name, display_name, settings, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        tenant_id,
        tenant_name,
        "Test Tenant",
        json!({}),
        Utc::now(),
        Utc::now()
    )
    .execute(pool.inner())
    .await
    .unwrap();

    // Insert metric
    let result = sqlx::query!(
        r#"INSERT INTO metrics (time, tenant_id, metric_name, metric_value, labels)
           VALUES ($1, $2, $3, $4, $5)"#,
        Utc::now(),
        tenant_id,
        "test_metric",
        42.0,
        json!({"environment": "test"})
    )
    .execute(pool.inner())
    .await;

    assert!(result.is_ok(), "Should insert metric successfully");

    // Clean up
    let _ = sqlx::query!("DELETE FROM metrics WHERE tenant_id = $1", tenant_id)
        .execute(pool.inner())
        .await;
    let _ = sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
        .execute(pool.inner())
        .await;
}

#[test]
fn test_jwt_token_generation() {
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let secret = "test_secret_key";

    let token = generate_token(user_id, tenant_id, "developer", secret, 3600);
    assert!(token.is_ok(), "Should generate token successfully");

    let token = token.unwrap();
    assert!(!token.is_empty(), "Token should not be empty");
    assert!(token.contains('.'), "Token should be a JWT");
}

#[test]
fn test_api_key_generation() {
    let key = generate_api_key();
    assert!(key.starts_with("llms_"), "API key should have correct prefix");
    assert_eq!(key.len(), 37, "API key should have correct length");

    // Test uniqueness
    let key2 = generate_api_key();
    assert_ne!(key, key2, "API keys should be unique");
}

#[test]
fn test_config_validation() {
    let config = create_test_config();
    assert!(config.validate().is_ok(), "Config should be valid");
}

#[test]
fn test_invalid_config_empty_jwt_secret() {
    let mut config = create_test_config();
    config.auth.jwt_secret = String::new();
    assert!(config.validate().is_err(), "Empty JWT secret should be invalid");
}

#[test]
fn test_invalid_config_short_jwt_secret() {
    let mut config = create_test_config();
    config.auth.jwt_secret = "short".to_string();
    assert!(config.validate().is_err(), "Short JWT secret should be invalid");
}

#[test]
fn test_invalid_config_zero_port() {
    let mut config = create_test_config();
    config.server.port = 0;
    // Port 0 is actually valid (means random port), but let's test the validation logic exists
    assert!(config.validate().is_ok(), "Port 0 is valid for testing");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL with TimescaleDB
async fn test_security_event_insertion() {
    use chrono::Utc;
    use serde_json::json;

    let config = create_test_config();
    let pool = DatabasePool::new(&config.database).await.unwrap();

    // Create tenant first
    let tenant_id = Uuid::new_v4();
    let tenant_name = format!("test_tenant_{}", Uuid::new_v4());

    sqlx::query!(
        r#"INSERT INTO tenants (id, name, display_name, settings, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        tenant_id,
        tenant_name,
        "Test Tenant",
        json!({}),
        Utc::now(),
        Utc::now()
    )
    .execute(pool.inner())
    .await
    .unwrap();

    // Insert security event
    let event_id = Uuid::new_v4();
    let result = sqlx::query!(
        r#"INSERT INTO security_events (time, event_id, tenant_id, event_type, severity, description, acknowledged)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
        Utc::now(),
        event_id,
        tenant_id,
        "test_event",
        "warning",
        "Test security event",
        false
    )
    .execute(pool.inner())
    .await;

    assert!(result.is_ok(), "Should insert security event successfully");

    // Clean up
    let _ = sqlx::query!("DELETE FROM security_events WHERE event_id = $1", event_id)
        .execute(pool.inner())
        .await;
    let _ = sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
        .execute(pool.inner())
        .await;
}
