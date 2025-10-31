//! Data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Tenant model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum UserRole {
    #[serde(rename = "super_admin")]
    #[sqlx(rename = "super_admin")]
    SuperAdmin,
    #[serde(rename = "tenant_admin")]
    #[sqlx(rename = "tenant_admin")]
    TenantAdmin,
    #[serde(rename = "developer")]
    #[sqlx(rename = "developer")]
    Developer,
    #[serde(rename = "viewer")]
    #[sqlx(rename = "viewer")]
    Viewer,
}

/// API Key model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    #[serde(skip_serializing)]
    pub key_hash: String,
    pub name: String,
    pub permissions: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MetricDataPoint {
    pub time: DateTime<Utc>,
    pub tenant_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub labels: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Scanner statistics
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScannerStats {
    pub time: DateTime<Utc>,
    pub tenant_id: Uuid,
    pub scanner_name: String,
    pub requests_total: i64,
    pub requests_valid: i64,
    pub requests_invalid: i64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub metadata: Option<serde_json::Value>,
}

/// Security event
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SecurityEvent {
    pub time: DateTime<Utc>,
    pub event_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_type: String,
    pub severity: Severity,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum Severity {
    #[serde(rename = "info")]
    #[sqlx(rename = "info")]
    Info,
    #[serde(rename = "warning")]
    #[sqlx(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    #[sqlx(rename = "error")]
    Error,
    #[serde(rename = "critical")]
    #[sqlx(rename = "critical")]
    Critical,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub threshold: f64,
    pub operator: String,
    pub duration_seconds: i32,
    pub severity: Severity,
    pub enabled: bool,
    pub notification_channels: Vec<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dashboard {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub config: serde_json::Value,
    pub is_default: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub result: AuditResult,
    pub error_message: Option<String>,
}

/// Audit result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum AuditResult {
    #[serde(rename = "success")]
    #[sqlx(rename = "success")]
    Success,
    #[serde(rename = "failure")]
    #[sqlx(rename = "failure")]
    Failure,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_serialization() {
        let role = UserRole::SuperAdmin;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"super_admin\"");

        let deserialized: UserRole = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, UserRole::SuperAdmin);
    }

    #[test]
    fn test_severity_serialization() {
        let severity = Severity::Critical;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"critical\"");

        let deserialized: Severity = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Severity::Critical);
    }

    #[test]
    fn test_audit_result_serialization() {
        let result = AuditResult::Success;
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(json, "\"success\"");
    }
}
