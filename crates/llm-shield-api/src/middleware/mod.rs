//! Middleware layers
//!
//! ## Available Middleware
//!
//! - `auth`: API key authentication
//! - `rate_limit`: Rate limiting and concurrent request limiting

pub mod auth;
pub mod rate_limit;

// Re-exports
pub use auth::{auth_middleware, optional_auth_middleware, AuthenticatedUser};
pub use rate_limit::{rate_limit_middleware, ClientTier};
