//! LLM Shield Dashboard
//!
//! Enterprise-grade monitoring dashboard and analytics for LLM Shield.
//!
//! ## Features
//!
//! - Real-time metrics visualization
//! - Security event monitoring
//! - Scanner performance analytics
//! - Alert management
//! - Custom dashboards
//! - Multi-tenancy with RBAC
//! - SSO integration
//!
//! ## Architecture
//!
//! The dashboard consists of several key components:
//!
//! - **GraphQL API**: Flexible query interface for metrics and analytics
//! - **Authentication**: JWT-based auth with API key support
//! - **Authorization**: Role-based access control (RBAC)
//! - **Time-Series Storage**: TimescaleDB for efficient metric storage
//! - **Real-time Updates**: WebSocket support for live metrics
//!
//! ## Example
//!
//! ```rust,no_run
//! use llm_shield_dashboard::{DashboardServer, DashboardConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = DashboardConfig::from_env()?;
//!     let server = DashboardServer::new(config).await?;
//!     server.run().await?;
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod graphql;
pub mod health;
pub mod middleware;
pub mod models;
pub mod server;

// Re-exports
pub use config::DashboardConfig;
pub use error::{DashboardError, Result};
pub use server::DashboardServer;

/// Dashboard API version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
