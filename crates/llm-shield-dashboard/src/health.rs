//! Health check endpoints

use crate::db::DatabasePool;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: DatabaseStatus,
}

/// Database status
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub connected: bool,
    pub connections: u32,
    pub idle_connections: u32,
}

/// Health check handler
pub async fn health_check(State(pool): State<DatabasePool>) -> impl IntoResponse {
    let db_status = check_database(&pool).await;

    let status = if db_status.connected {
        "healthy"
    } else {
        "unhealthy"
    };

    let response = HealthResponse {
        status: status.to_string(),
        version: crate::VERSION.to_string(),
        database: db_status,
    };

    let status_code = if response.database.connected {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(response))
}

/// Readiness check handler
pub async fn readiness_check(State(pool): State<DatabasePool>) -> impl IntoResponse {
    let db_status = check_database(&pool).await;

    if db_status.connected {
        (StatusCode::OK, Json(serde_json::json!({ "ready": true })))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({ "ready": false })))
    }
}

/// Liveness check handler
pub async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "alive": true })))
}

async fn check_database(pool: &DatabasePool) -> DatabaseStatus {
    if pool.is_closed() {
        return DatabaseStatus {
            connected: false,
            connections: 0,
            idle_connections: 0,
        };
    }

    // Test database connection
    let connected = sqlx::query("SELECT 1")
        .fetch_one(pool.inner())
        .await
        .is_ok();

    let stats = pool.stats();

    DatabaseStatus {
        connected,
        connections: stats.connections,
        idle_connections: stats.idle_connections,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            database: DatabaseStatus {
                connected: true,
                connections: 5,
                idle_connections: 3,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_database_status() {
        let status = DatabaseStatus {
            connected: true,
            connections: 10,
            idle_connections: 5,
        };

        assert!(status.connected);
        assert_eq!(status.connections, 10);
        assert_eq!(status.idle_connections, 5);
    }
}
