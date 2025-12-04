//! # LLM Shield Core
//!
//! Core types, traits, and utilities for the LLM Shield security toolkit.
//!
//! ## SPARC Methodology - Specification Phase
//!
//! This module defines the core abstractions for the LLM Shield system:
//! - `Scanner`: Core trait for all security scanners
//! - `ScanResult`: Standardized result type
//! - `Risk`: Risk assessment types
//! - `Error`: Comprehensive error handling
//!
//! ## Enterprise-Grade Design Principles
//!
//! 1. **Type Safety**: Strong typing throughout
//! 2. **Async-First**: All scanners support async operations
//! 3. **Composability**: Scanners can be chained and combined
//! 4. **Observability**: Comprehensive tracing and metrics
//! 5. **Error Context**: Rich error types with context

pub mod error;
pub mod result;
pub mod scanner;
pub mod types;
pub mod vault;
pub mod adapters;

// Re-exports for convenience
pub use async_trait::async_trait;
pub use error::{Error, Result};
pub use result::{Entity, RiskFactor, ScanResult, Severity};
pub use scanner::{InputScanner, OutputScanner, Scanner, ScannerPipeline, ScannerType};
pub use types::{ScannerConfig, ScannerMetadata, ScannerCategory, PerformanceInfo};
pub use vault::Vault;

// Re-export adapter types for upstream integration
pub use adapters::{
    PolicyAdapter, PolicyDecision, PolicyContext, PolicyEvaluator,
    EnforcementAction, PolicyResult, PolicyHook,
    ConfigAdapter, ShieldParameters, ThresholdConfig, PatternConfig,
    ConfigLoader, ConfigSource, ConfigHook,
};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library initialization
pub fn init() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
