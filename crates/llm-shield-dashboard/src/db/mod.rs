//! Database layer

pub mod migrations;
pub mod pool;

pub use pool::DatabasePool;

use crate::error::Result;
use sqlx::PgPool;

/// Initialize database with migrations
pub async fn initialize_database(pool: &PgPool) -> Result<()> {
    migrations::run_migrations(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL
    async fn test_database_initialization() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/llm_shield_dashboard_test".to_string());

        let pool = PgPool::connect(&database_url).await.unwrap();
        let result = initialize_database(&pool).await;
        assert!(result.is_ok());

        pool.close().await;
    }
}
