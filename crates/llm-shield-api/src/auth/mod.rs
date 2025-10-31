//! Authentication module
//!
//! ## Overview
//!
//! API key-based authentication with:
//! - Cryptographically secure key generation
//! - Argon2id password hashing
//! - Multiple storage backends (Memory, File, Redis)
//! - Role-based access control via tiers
//!
//! ## Architecture
//!
//! ```text
//! Request → AuthMiddleware → [Extract Key] → [Validate] → Handler
//!                                   ↓             ↓
//!                              AuthService   KeyStorage
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llm_shield_api::auth::{AuthService, MemoryKeyStorage};
//! use std::sync::Arc;
//!
//! // Create service
//! let storage = Arc::new(MemoryKeyStorage::new());
//! let auth_service = AuthService::new(storage);
//!
//! // Create a key
//! let response = auth_service.create_key(
//!     "My App".to_string(),
//!     RateLimitTier::Pro,
//!     Some(365)
//! ).await?;
//!
//! println!("API Key: {}", response.key); // Only shown once!
//!
//! // Validate a key
//! let key = auth_service.validate_key(&api_key).await?;
//! ```

pub mod service;
pub mod storage;
pub mod types;

// Re-exports
pub use service::AuthService;
pub use storage::{FileKeyStorage, KeyStorage, MemoryKeyStorage};
pub use types::{ApiKey, CreateKeyRequest, CreateKeyResponse};
