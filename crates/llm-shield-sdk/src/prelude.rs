//! # Prelude Module
//!
//! Convenient re-exports for common LLM Shield SDK usage.
//!
//! Import everything you need with a single use statement:
//!
//! ```rust,ignore
//! use llm_shield_sdk::prelude::*;
//! ```
//!
//! ## What's Included
//!
//! - Core types: `Shield`, `ShieldBuilder`, `Preset`
//! - Result types: `ScanResult`, `Entity`, `RiskFactor`, `Severity`
//! - Error handling: `SdkResult`, `SdkError`
//! - Configuration: `ShieldConfig`, `ScanMode`
//! - Input scanners and their configs
//! - Output scanners and their configs
//! - Async traits and utilities

// ============================================================================
// Core SDK Types
// ============================================================================

pub use crate::shield::Shield;
pub use crate::builder::ShieldBuilder;
pub use crate::preset::Preset;
pub use crate::config::{ShieldConfig, ScanMode, ParallelConfig};
pub use crate::error::{SdkError, SdkResult};

// ============================================================================
// Core Types from llm-shield-core
// ============================================================================

pub use llm_shield_core::{
    // Results
    ScanResult,
    Entity,
    RiskFactor,
    Severity,

    // Scanner traits
    Scanner,
    InputScanner,
    OutputScanner,
    ScannerType,
    ScannerPipeline,

    // State management
    Vault,

    // Configuration
    ScannerConfig,
    ScannerMetadata,
    ScannerCategory,
    PerformanceInfo,

    // Error handling
    Error as CoreError,
    Result as CoreResult,

    // Async support
    async_trait,
};

// ============================================================================
// Input Scanners
// ============================================================================

pub use llm_shield_scanners::input::{
    // Ban Substrings
    BanSubstrings,
    BanSubstringsConfig,
    MatchType,

    // Ban Code
    BanCode,
    BanCodeConfig,

    // Ban Competitors
    BanCompetitors,
    BanCompetitorsConfig,

    // Secrets Detection
    Secrets,
    SecretsConfig,
    SecretCategory,

    // Toxicity Detection
    Toxicity,
    ToxicityConfig,
    ToxicityCategory,

    // Prompt Injection Detection
    PromptInjection,
    PromptInjectionConfig,

    // Invisible Text Detection
    InvisibleText,
    InvisibleTextConfig,

    // Gibberish Detection
    Gibberish,
    GibberishConfig,

    // Language Detection
    Language,
    LanguageConfig,

    // Token Limit
    TokenLimit,
    TokenLimitConfig,

    // Regex Scanner
    RegexScanner,
    RegexConfig,
    RegexPattern,

    // Sentiment Analysis
    Sentiment,
    SentimentConfig,
};

// ============================================================================
// Output Scanners
// ============================================================================

pub use llm_shield_scanners::output::{
    // Sensitive Data Detection
    Sensitive,
    SensitiveConfig,
    SensitiveEntityType,

    // No Refusal Detection
    NoRefusal,
    NoRefusalConfig,

    // Relevance Checking
    Relevance,
    RelevanceConfig,

    // Ban Topics
    BanTopics,
    BanTopicsConfig,

    // Bias Detection
    Bias,
    BiasConfig,

    // Malicious URL Detection
    MaliciousURLs,
    MaliciousURLsConfig,

    // Factuality Checking
    Factuality,
    FactualityConfig,

    // Reading Time Validation
    ReadingTime,
    ReadingTimeConfig,

    // URL Reachability
    URLReachability,
    URLReachabilityConfig,

    // Regex Output Checking
    RegexOutput,
    RegexOutputConfig,
};

// ============================================================================
// Async Runtime Support
// ============================================================================

pub use futures::future::{join_all, try_join_all};
pub use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prelude_types_available() {
        // Test that core types are accessible
        let _severity = Severity::High;
        let _preset = Preset::Standard;
        let _scan_mode = ScanMode::Prompt;
    }

    #[test]
    fn test_prelude_result_types() {
        // Test that result types work correctly
        let result = ScanResult::pass("test".to_string());
        assert!(result.is_valid);
    }
}
