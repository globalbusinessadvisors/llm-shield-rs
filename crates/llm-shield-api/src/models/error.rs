//! Error types for the API

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Invalid request (400 Bad Request)
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Unauthorized (401 Unauthorized)
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden (403 Forbidden)
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// Not found (404 Not Found)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Rate limit exceeded (429 Too Many Requests)
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Internal server error (500 Internal Server Error)
    #[error("Internal server error: {0}")]
    InternalError(String),

    /// Service unavailable (503 Service Unavailable)
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Validation error (422 Unprocessable Entity)
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Scanner error
    #[error("Scanner error: {0}")]
    ScannerError(String),

    /// Model error
    #[error("Model error: {0}")]
    ModelError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl ApiError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::ValidationError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ApiError::ScannerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ModelError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::InvalidRequest(_) => "INVALID_REQUEST",
            ApiError::Unauthorized(_) => "UNAUTHORIZED",
            ApiError::Forbidden(_) => "FORBIDDEN",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::RateLimitExceeded(_) => "RATE_LIMIT_EXCEEDED",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
            ApiError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::ScannerError(_) => "SCANNER_ERROR",
            ApiError::ModelError(_) => "MODEL_ERROR",
            ApiError::ConfigError(_) => "CONFIG_ERROR",
        }
    }

    /// Convert to ErrorResponse
    pub fn to_error_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.error_code().to_string(),
            message: self.to_string(),
            details: None,
        }
    }
}

/// Error response body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code (e.g., "INVALID_REQUEST")
    pub code: String,

    /// Human-readable error message
    pub message: String,

    /// Additional error details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// Implement IntoResponse for ApiError to convert errors to HTTP responses
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = self.to_error_response();

        let body = Json(serde_json::json!({
            "success": false,
            "error": error_response,
        }));

        (status, body).into_response()
    }
}

/// Convert llm_shield_core::Error to ApiError
impl From<llm_shield_core::Error> for ApiError {
    fn from(err: llm_shield_core::Error) -> Self {
        ApiError::ScannerError(err.to_string())
    }
}

/// Convert config::ConfigError to ApiError
impl From<config::ConfigError> for ApiError {
    fn from(err: config::ConfigError) -> Self {
        ApiError::ConfigError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            ApiError::InvalidRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::RateLimitExceeded("test".to_string()).status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            ApiError::InternalError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            ApiError::InvalidRequest("test".to_string()).error_code(),
            "INVALID_REQUEST"
        );
        assert_eq!(
            ApiError::Unauthorized("test".to_string()).error_code(),
            "UNAUTHORIZED"
        );
        assert_eq!(
            ApiError::RateLimitExceeded("test".to_string()).error_code(),
            "RATE_LIMIT_EXCEEDED"
        );
    }

    #[test]
    fn test_error_response_creation() {
        let error = ApiError::InvalidRequest("Missing field".to_string());
        let response = error.to_error_response();

        assert_eq!(response.code, "INVALID_REQUEST");
        assert!(response.message.contains("Missing field"));
        assert!(response.details.is_none());
    }

    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            code: "TEST_ERROR".to_string(),
            message: "Test message".to_string(),
            details: None,
        };

        let json = serde_json::to_string(&error_response).unwrap();
        assert!(json.contains("TEST_ERROR"));
        assert!(json.contains("Test message"));
    }

    #[test]
    fn test_error_display() {
        let error = ApiError::InvalidRequest("test error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Invalid request"));
        assert!(display.contains("test error"));
    }
}
