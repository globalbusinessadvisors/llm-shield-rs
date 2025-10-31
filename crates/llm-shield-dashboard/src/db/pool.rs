//! Database connection pool

use crate::{
    config::DatabaseConfig,
    error::{DashboardError, Result},
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Database pool wrapper
#[derive(Clone)]
pub struct DatabasePool {
    pool: PgPool,
}

impl DatabasePool {
    /// Create a new database pool
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connection_timeout_secs))
            .connect(&config.url)
            .await
            .map_err(|e| {
                DashboardError::Database(e)
            })?;

        Ok(Self { pool })
    }

    /// Get the inner pool
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Check if the pool is closed
    pub fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            connections: self.pool.size() as u32,
            idle_connections: self.pool.num_idle() as u32,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub connections: u32,
    pub idle_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DatabaseConfig;

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL
    async fn test_database_pool_creation() {
        let config = DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/llm_shield_dashboard_test".to_string()),
            max_connections: 5,
            min_connections: 1,
            connection_timeout_secs: 5,
            enable_logging: false,
        };

        let pool = DatabasePool::new(&config).await;
        assert!(pool.is_ok());

        let pool = pool.unwrap();
        assert!(!pool.is_closed());

        let stats = pool.stats();
        assert!(stats.connections > 0);

        pool.close().await;
    }

    #[tokio::test]
    async fn test_database_pool_invalid_url() {
        let config = DatabaseConfig {
            url: "postgres://invalid:5432/nonexistent".to_string(),
            max_connections: 5,
            min_connections: 1,
            connection_timeout_secs: 1, // Short timeout for test
            enable_logging: false,
        };

        let result = DatabasePool::new(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL
    async fn test_pool_stats() {
        let config = DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/llm_shield_dashboard_test".to_string()),
            max_connections: 10,
            min_connections: 2,
            connection_timeout_secs: 5,
            enable_logging: false,
        };

        let pool = DatabasePool::new(&config).await.unwrap();
        let stats = pool.stats();

        assert!(stats.connections >= config.min_connections);
        assert!(stats.connections <= config.max_connections);

        pool.close().await;
    }
}
