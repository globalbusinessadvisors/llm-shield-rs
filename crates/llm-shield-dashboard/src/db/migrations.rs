//! Database migrations

use crate::error::Result;
use sqlx::PgPool;
use tracing::{info, warn};

/// Run all database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    info!("Running database migrations...");

    // Create TimescaleDB extension if not exists
    create_timescale_extension(pool).await?;

    // Create tables
    create_tenants_table(pool).await?;
    create_users_table(pool).await?;
    create_api_keys_table(pool).await?;
    create_metrics_table(pool).await?;
    create_scanner_stats_table(pool).await?;
    create_security_events_table(pool).await?;
    create_alert_rules_table(pool).await?;
    create_dashboards_table(pool).await?;
    create_audit_log_table(pool).await?;

    // Create hypertables
    create_hypertables(pool).await?;

    // Create continuous aggregates
    create_continuous_aggregates(pool).await?;

    // Create retention policies
    create_retention_policies(pool).await?;

    // Create indexes
    create_indexes(pool).await?;

    info!("Database migrations completed successfully");
    Ok(())
}

async fn create_timescale_extension(pool: &PgPool) -> Result<()> {
    sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE")
        .execute(pool)
        .await?;
    info!("TimescaleDB extension created");
    Ok(())
}

async fn create_tenants_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            settings JSONB DEFAULT '{}',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Tenants table created");
    Ok(())
}

async fn create_users_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
            email TEXT NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL CHECK (role IN ('super_admin', 'tenant_admin', 'developer', 'viewer')),
            enabled BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(tenant_id, email)
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Users table created");
    Ok(())
}

async fn create_api_keys_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
            user_id UUID REFERENCES users(id) ON DELETE CASCADE,
            key_hash TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            permissions TEXT[] NOT NULL DEFAULT '{}',
            last_used_at TIMESTAMPTZ,
            expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("API keys table created");
    Ok(())
}

async fn create_metrics_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS metrics (
            time TIMESTAMPTZ NOT NULL,
            tenant_id UUID NOT NULL,
            metric_name TEXT NOT NULL,
            metric_value DOUBLE PRECISION NOT NULL,
            labels JSONB,
            metadata JSONB
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Metrics table created");
    Ok(())
}

async fn create_scanner_stats_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scanner_stats (
            time TIMESTAMPTZ NOT NULL,
            tenant_id UUID NOT NULL,
            scanner_name TEXT NOT NULL,
            requests_total BIGINT NOT NULL,
            requests_valid BIGINT NOT NULL,
            requests_invalid BIGINT NOT NULL,
            avg_latency_ms DOUBLE PRECISION NOT NULL,
            p95_latency_ms DOUBLE PRECISION NOT NULL,
            p99_latency_ms DOUBLE PRECISION NOT NULL,
            metadata JSONB
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Scanner stats table created");
    Ok(())
}

async fn create_security_events_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS security_events (
            time TIMESTAMPTZ NOT NULL,
            event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            user_id UUID,
            event_type TEXT NOT NULL,
            severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'error', 'critical')),
            description TEXT NOT NULL,
            metadata JSONB,
            acknowledged BOOLEAN DEFAULT false,
            acknowledged_by UUID,
            acknowledged_at TIMESTAMPTZ
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Security events table created");
    Ok(())
}

async fn create_alert_rules_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS alert_rules (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT,
            query TEXT NOT NULL,
            threshold DOUBLE PRECISION NOT NULL,
            operator TEXT NOT NULL CHECK (operator IN ('>', '<', '>=', '<=', '=', '!=')),
            duration_seconds INTEGER NOT NULL,
            severity TEXT NOT NULL CHECK (severity IN ('info', 'warning', 'error', 'critical')),
            enabled BOOLEAN DEFAULT true,
            notification_channels TEXT[],
            created_by UUID NOT NULL REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(tenant_id, name)
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Alert rules table created");
    Ok(())
}

async fn create_dashboards_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS dashboards (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT,
            config JSONB NOT NULL,
            is_default BOOLEAN DEFAULT false,
            created_by UUID NOT NULL REFERENCES users(id),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(tenant_id, name)
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Dashboards table created");
    Ok(())
}

async fn create_audit_log_table(pool: &PgPool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_log (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            tenant_id UUID NOT NULL,
            user_id UUID NOT NULL,
            action TEXT NOT NULL,
            resource_type TEXT NOT NULL,
            resource_id TEXT,
            changes JSONB,
            ip_address INET,
            user_agent TEXT,
            result TEXT NOT NULL CHECK (result IN ('success', 'failure')),
            error_message TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;
    info!("Audit log table created");
    Ok(())
}

async fn create_hypertables(pool: &PgPool) -> Result<()> {
    // Create hypertable for metrics
    match sqlx::query(
        "SELECT create_hypertable('metrics', 'time', if_not_exists => TRUE, migrate_data => TRUE)"
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Metrics hypertable created"),
        Err(e) => warn!("Metrics hypertable may already exist: {}", e),
    }

    // Create hypertable for scanner_stats
    match sqlx::query(
        "SELECT create_hypertable('scanner_stats', 'time', if_not_exists => TRUE, migrate_data => TRUE)"
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Scanner stats hypertable created"),
        Err(e) => warn!("Scanner stats hypertable may already exist: {}", e),
    }

    // Create hypertable for security_events
    match sqlx::query(
        "SELECT create_hypertable('security_events', 'time', if_not_exists => TRUE, migrate_data => TRUE)"
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Security events hypertable created"),
        Err(e) => warn!("Security events hypertable may already exist: {}", e),
    }

    Ok(())
}

async fn create_continuous_aggregates(pool: &PgPool) -> Result<()> {
    // Create 1-minute aggregate for metrics
    match sqlx::query(
        r#"
        CREATE MATERIALIZED VIEW IF NOT EXISTS metrics_1min
        WITH (timescaledb.continuous) AS
        SELECT
            time_bucket('1 minute', time) AS bucket,
            tenant_id,
            metric_name,
            labels->>'scanner' AS scanner,
            AVG(metric_value) AS avg_value,
            MAX(metric_value) AS max_value,
            MIN(metric_value) AS min_value,
            COUNT(*) AS count
        FROM metrics
        GROUP BY bucket, tenant_id, metric_name, scanner
        WITH NO DATA
        "#,
    )
    .execute(pool)
    .await
    {
        Ok(_) => {
            info!("Metrics 1-minute continuous aggregate created");
            // Refresh policy
            let _ = sqlx::query(
                r#"
                SELECT add_continuous_aggregate_policy('metrics_1min',
                    start_offset => INTERVAL '1 hour',
                    end_offset => INTERVAL '1 minute',
                    schedule_interval => INTERVAL '1 minute',
                    if_not_exists => TRUE)
                "#,
            )
            .execute(pool)
            .await;
        }
        Err(e) => warn!("Metrics continuous aggregate may already exist: {}", e),
    }

    Ok(())
}

async fn create_retention_policies(pool: &PgPool) -> Result<()> {
    // 90 days for metrics
    match sqlx::query(
        "SELECT add_retention_policy('metrics', INTERVAL '90 days', if_not_exists => TRUE)"
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Metrics retention policy created (90 days)"),
        Err(e) => warn!("Metrics retention policy may already exist: {}", e),
    }

    // 1 year for scanner stats
    match sqlx::query(
        "SELECT add_retention_policy('scanner_stats', INTERVAL '1 year', if_not_exists => TRUE)"
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Scanner stats retention policy created (1 year)"),
        Err(e) => warn!("Scanner stats retention policy may already exist: {}", e),
    }

    // 2 years for security events
    match sqlx::query(
        "SELECT add_retention_policy('security_events', INTERVAL '2 years', if_not_exists => TRUE)"
    )
    .execute(pool)
    .await
    {
        Ok(_) => info!("Security events retention policy created (2 years)"),
        Err(e) => warn!("Security events retention policy may already exist: {}", e),
    }

    Ok(())
}

async fn create_indexes(pool: &PgPool) -> Result<()> {
    // Metrics indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_metrics_tenant_time ON metrics(tenant_id, time DESC)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_metrics_name ON metrics(metric_name, time DESC)")
        .execute(pool)
        .await;

    // Scanner stats indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_scanner_stats_tenant_time ON scanner_stats(tenant_id, time DESC)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_scanner_stats_scanner ON scanner_stats(scanner_name, time DESC)")
        .execute(pool)
        .await;

    // Security events indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_security_events_tenant_time ON security_events(tenant_id, time DESC)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_security_events_severity ON security_events(severity, time DESC)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_security_events_type ON security_events(event_type, time DESC)")
        .execute(pool)
        .await;

    // Users indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_tenant ON users(tenant_id)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(pool)
        .await;

    // API keys indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_tenant ON api_keys(tenant_id)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id)")
        .execute(pool)
        .await;

    // Audit log indexes
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp DESC)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id, timestamp DESC)")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_tenant ON audit_log(tenant_id, timestamp DESC)")
        .execute(pool)
        .await;

    info!("Database indexes created");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL with TimescaleDB
    async fn test_run_migrations() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/llm_shield_dashboard_test".to_string());

        let pool = PgPool::connect(&database_url).await.unwrap();

        // Run migrations
        let result = run_migrations(&pool).await;
        assert!(result.is_ok());

        // Verify tables exist
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'"
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let table_names: Vec<String> = tables.into_iter().map(|(name,)| name).collect();
        assert!(table_names.contains(&"tenants".to_string()));
        assert!(table_names.contains(&"users".to_string()));
        assert!(table_names.contains(&"metrics".to_string()));
        assert!(table_names.contains(&"security_events".to_string()));

        pool.close().await;
    }
}
