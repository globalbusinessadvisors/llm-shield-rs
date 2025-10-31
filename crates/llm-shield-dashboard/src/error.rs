//! Error types for the dashboard

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Dashboard error type
#[derive(Debug, thiserror::Error)]
pub enum DashboardError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization failed: {0}")]
    Authorization(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Not found error
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Conflict error
    #[error("Resource conflict: {0}")]
    Conflict(String),

    /// Rate limit error
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),

    /// GraphQL error
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    /// JWT error
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for dashboard operations
pub type Result<T> = std::result::Result<T, DashboardError>;

/// Error response for API
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl IntoResponse for DashboardError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            DashboardError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            DashboardError::Authentication(_) => (StatusCode::UNAUTHORIZED, "authentication_error"),
            DashboardError::Authorization(_) => (StatusCode::FORBIDDEN, "authorization_error"),
            DashboardError::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            DashboardError::Configuration(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "configuration_error")
            }
            DashboardError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            DashboardError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            DashboardError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded")
            }
            DashboardError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            DashboardError::GraphQL(_) => (StatusCode::BAD_REQUEST, "graphql_error"),
            DashboardError::Jwt(_) => (StatusCode::UNAUTHORIZED, "jwt_error"),
            DashboardError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "io_error"),
            DashboardError::Serialization(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "serialization_error")
            }
        };

        let body = Json(ErrorResponse::new(error_type, self.to_string()));

        (status, body).into_response()
    }
}

impl From<DashboardError> for async_graphql::Error {
    fn from(err: DashboardError) -> Self {
        async_graphql::Error::new(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = DashboardError::Authentication("Invalid token".to_string());
        assert_eq!(err.to_string(), "Authentication failed: Invalid token");
    }

    #[test]
    fn test_error_response() {
        let response = ErrorResponse::new("test_error", "Test message");
        assert_eq!(response.error, "test_error");
        assert_eq!(response.message, "Test message");
        assert!(response.details.is_none());
    }

    #[test]
    fn test_error_response_with_details() {
        let details = serde_json::json!({ "field": "email" });
        let response = ErrorResponse::new("validation_error", "Invalid field").with_details(details);
        assert!(response.details.is_some());
    }

    #[test]
    fn test_error_into_response() {
        let err = DashboardError::Validation("Invalid input".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_database_error_conversion() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let err: DashboardError = sqlx_err.into();
        assert!(matches!(err, DashboardError::Database(_)));
    }

    #[test]
    fn test_not_found_error() {
        let err = DashboardError::NotFound("Dashboard not found".to_string());
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_rate_limit_error() {
        let err = DashboardError::RateLimitExceeded;
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
}
