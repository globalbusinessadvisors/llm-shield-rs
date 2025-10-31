//! GraphQL API

use crate::{db::DatabasePool, models::*};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema};
use uuid::Uuid;

pub type DashboardSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// GraphQL Query root
pub struct Query;

#[Object]
impl Query {
    /// Get dashboard version
    async fn version(&self) -> &str {
        crate::VERSION
    }

    /// Health check
    async fn health(&self) -> bool {
        true
    }

    /// Get tenant by ID
    async fn tenant(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<Tenant>> {
        let pool = ctx.data::<DatabasePool>()?;
        let tenant = sqlx::query_as!(
            Tenant,
            "SELECT * FROM tenants WHERE id = $1",
            id
        )
        .fetch_optional(pool.inner())
        .await?;
        Ok(tenant)
    }

    /// Get user by ID
    async fn user(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<User>> {
        let pool = ctx.data::<DatabasePool>()?;
        let user = sqlx::query_as!(
            User,
            r#"SELECT
                id, tenant_id, email, password_hash,
                role as "role: UserRole",
                enabled, created_at, updated_at
            FROM users WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;
        Ok(user)
    }
}

/// Create GraphQL schema
pub fn create_schema(pool: DatabasePool) -> DashboardSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(pool)
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        // This test just verifies the schema can be created
        // Actual functionality testing requires a database
        let sdl = DashboardSchema::sdl();
        assert!(sdl.contains("type Query"));
        assert!(sdl.contains("version"));
        assert!(sdl.contains("health"));
    }
}
