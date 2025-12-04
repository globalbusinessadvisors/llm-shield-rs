//! # SDK Integrations Module
//!
//! High-level integration layer for consuming upstream LLM-Dev-Ops services.
//!
//! ## Phase 2B Integration
//!
//! This module provides SDK-level wrappers around the core adapter modules,
//! offering convenient APIs for:
//! - Policy-driven scanning with LLM-Policy-Engine
//! - Config-driven parameter loading from LLM-Config-Manager
//! - Runtime hooks for dynamic behavior
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llm_shield_sdk::integrations::{PolicyIntegration, ConfigIntegration};
//!
//! // Set up policy integration
//! let policy = PolicyIntegration::new()
//!     .with_default_context("my-app")
//!     .build();
//!
//! // Set up config integration
//! let config = ConfigIntegration::from_env("SHIELD")
//!     .with_auto_refresh(300)
//!     .build();
//!
//! // Use with Shield
//! let shield = Shield::builder()
//!     .with_policy_integration(policy)
//!     .with_config_integration(config)
//!     .build()?;
//! ```

pub mod policy_integration;
pub mod config_integration;
pub mod runtime_hooks;

// Re-export main types
pub use policy_integration::{PolicyIntegration, PolicyIntegrationBuilder};
pub use config_integration::{ConfigIntegration, ConfigIntegrationBuilder};
pub use runtime_hooks::{RuntimeHooks, ScanHook, HookResult};
