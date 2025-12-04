//! # LLM Shield SDK
//!
//! Enterprise-grade SDK for securing Large Language Model applications.
//!
//! ## Overview
//!
//! LLM Shield SDK provides a comprehensive security toolkit for LLM applications,
//! offering real-time scanning and protection against:
//!
//! - **Prompt Injection**: Detects attempts to manipulate LLM behavior
//! - **Data Leakage**: Prevents exposure of sensitive information (PII, credentials)
//! - **Toxic Content**: Filters harmful, offensive, or inappropriate content
//! - **Code Injection**: Blocks malicious code in prompts
//! - **Jailbreak Attempts**: Identifies attempts to bypass safety measures
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use llm_shield_sdk::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Create a shield with standard security level
//!     let shield = Shield::builder()
//!         .with_preset(Preset::Standard)
//!         .build()?;
//!
//!     // Scan a prompt before sending to LLM
//!     let result = shield.scan_prompt("Hello, how are you?").await?;
//!
//!     if result.is_valid {
//!         println!("Prompt is safe to send to LLM");
//!     } else {
//!         println!("Security risk detected: {:?}", result.risk_factors);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Security Presets
//!
//! The SDK provides three built-in security presets:
//!
//! - **Strict**: Maximum security for sensitive applications (banking, healthcare)
//! - **Standard**: Balanced security for general-purpose applications
//! - **Permissive**: Minimal security for development/testing
//!
//! ```rust,ignore
//! // For high-security applications
//! let shield = Shield::strict()?;
//!
//! // For general-purpose applications
//! let shield = Shield::standard()?;
//!
//! // For development/testing
//! let shield = Shield::permissive()?;
//! ```
//!
//! ## Custom Configuration
//!
//! For fine-grained control, use the builder pattern:
//!
//! ```rust,ignore
//! let shield = Shield::builder()
//!     // Add specific input scanners
//!     .add_input_scanner(BanSubstrings::with_substrings(["password", "secret"])?)
//!     .add_input_scanner(Secrets::default_config()?)
//!     .add_input_scanner(Toxicity::default_config()?)
//!     // Add output scanners
//!     .add_output_scanner(Sensitive::default_config()?)
//!     // Configure behavior
//!     .with_short_circuit(0.9) // Stop on high-risk detection
//!     .with_parallel_execution(true) // Run scanners in parallel
//!     .build()?;
//! ```
//!
//! ## Batch Scanning
//!
//! For high-throughput applications:
//!
//! ```rust,ignore
//! let prompts = vec!["Hello", "How are you?", "What's the weather?"];
//! let results = shield.scan_batch(&prompts).await?;
//!
//! for (prompt, result) in prompts.iter().zip(results.iter()) {
//!     println!("{}: {}", prompt, if result.is_valid { "✓" } else { "✗" });
//! }
//! ```
//!
//! ## Available Scanners
//!
//! ### Input Scanners (scan prompts before LLM)
//!
//! | Scanner | Description |
//! |---------|-------------|
//! | `BanSubstrings` | Block specific words/phrases |
//! | `BanCode` | Detect programming language code |
//! | `BanCompetitors` | Filter competitor mentions |
//! | `Secrets` | Detect 40+ types of API keys/tokens |
//! | `Toxicity` | ML-based toxicity detection |
//! | `PromptInjection` | Detect injection attacks |
//! | `InvisibleText` | Detect hidden unicode characters |
//! | `Gibberish` | Detect nonsensical text |
//! | `Language` | Enforce language requirements |
//! | `TokenLimit` | Enforce token limits |
//! | `RegexScanner` | Custom regex patterns |
//! | `Sentiment` | Analyze text sentiment |
//!
//! ### Output Scanners (scan LLM responses)
//!
//! | Scanner | Description |
//! |---------|-------------|
//! | `Sensitive` | Detect PII (emails, SSNs, etc.) |
//! | `NoRefusal` | Detect over-cautious refusals |
//! | `Relevance` | Ensure response relevance |
//! | `BanTopics` | Filter prohibited topics |
//! | `Bias` | Detect biased content |
//! | `MaliciousURLs` | Detect phishing/malware URLs |
//! | `Factuality` | Assess factual confidence |
//! | `ReadingTime` | Validate response length |
//! | `URLReachability` | Verify URL accessibility |
//! | `RegexOutput` | Custom output patterns |
//!
//! ## Enterprise Features
//!
//! - **High Performance**: Sub-millisecond scanning with zero-copy processing
//! - **Thread-Safe**: Safe for concurrent use across threads
//! - **Async-First**: Full async/await support for scalability
//! - **Composable**: Chain multiple scanners with short-circuit evaluation
//! - **Observable**: Built-in tracing and metrics support
//!
//! ## Feature Flags
//!
//! - `all-scanners` (default): Include all scanner types
//! - `input-scanners`: Only input scanners
//! - `output-scanners`: Only output scanners
//! - `ml-models`: Enable ML-based detection
//! - `cloud-aws`: AWS integration
//! - `cloud-gcp`: GCP integration
//! - `cloud-azure`: Azure integration

pub mod builder;
pub mod config;
pub mod error;
pub mod prelude;
pub mod preset;
pub mod scanner_factory;
pub mod shield;
pub mod integrations;

// Re-export main types for convenience
pub use builder::ShieldBuilder;
pub use config::{ShieldConfig, ScanMode, ParallelConfig};
pub use error::{SdkError, SdkResult};
pub use preset::Preset;
pub use shield::Shield;

// Re-export integration types
pub use integrations::{
    PolicyIntegration, PolicyIntegrationBuilder,
    ConfigIntegration, ConfigIntegrationBuilder,
    RuntimeHooks, ScanHook, HookResult,
};

// Re-export core types
pub use llm_shield_core::{
    Entity, Error as CoreError, Result as CoreResult, RiskFactor, ScanResult, Scanner,
    ScannerType, Severity, Vault, ScannerPipeline,
};

// Re-export core adapter types for upstream integration (Phase 2B)
pub use llm_shield_core::{
    PolicyAdapter, PolicyDecision, PolicyContext, PolicyEvaluator,
    EnforcementAction, PolicyResult, PolicyHook,
    ConfigAdapter, ShieldParameters, ThresholdConfig, PatternConfig,
    ConfigLoader, ConfigSource, ConfigHook,
};

// Re-export all scanners
pub use llm_shield_scanners::input::{
    BanSubstrings, BanSubstringsConfig, MatchType,
    BanCode, BanCodeConfig,
    BanCompetitors, BanCompetitorsConfig,
    Secrets, SecretsConfig, SecretCategory,
    Toxicity, ToxicityConfig, ToxicityCategory,
    PromptInjection, PromptInjectionConfig,
    InvisibleText, InvisibleTextConfig,
    Gibberish, GibberishConfig,
    Language, LanguageConfig,
    TokenLimit, TokenLimitConfig,
    RegexScanner, RegexConfig, RegexPattern,
    Sentiment, SentimentConfig,
};

pub use llm_shield_scanners::output::{
    Sensitive, SensitiveConfig, SensitiveEntityType,
    NoRefusal, NoRefusalConfig,
    Relevance, RelevanceConfig,
    BanTopics, BanTopicsConfig,
    Bias, BiasConfig,
    MaliciousURLs, MaliciousURLsConfig,
    Factuality, FactualityConfig,
    ReadingTime, ReadingTimeConfig,
    URLReachability, URLReachabilityConfig,
    RegexOutput, RegexOutputConfig,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the SDK with default settings
///
/// This sets up logging and performs any necessary initialization.
/// Call this once at application startup.
///
/// # Example
///
/// ```rust,ignore
/// llm_shield_sdk::init();
/// ```
pub fn init() {
    llm_shield_core::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_prelude_exports() {
        // Verify prelude contains expected types
        use prelude::*;

        // These should compile if exports are correct
        let _ = Preset::Standard;
        let _ = Severity::High;
    }
}
