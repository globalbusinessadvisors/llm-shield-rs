//! LLM Shield REST API
//!
//! Production-grade REST API exposing LLM Shield scanners and anonymization
//! capabilities via HTTP with enterprise-grade security, observability, and performance.

pub mod auth;
#[cfg(feature = "cloud")]
pub mod cloud_init;
pub mod config;
pub mod extractors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod observability;
pub mod rate_limiting;
pub mod router;
pub mod server;
pub mod services;
pub mod state;

// Re-exports
pub use config::AppConfig;
pub use models::{ApiError, ApiResponse};

/// Result type for API operations
pub type Result<T> = std::result::Result<T, ApiError>;
