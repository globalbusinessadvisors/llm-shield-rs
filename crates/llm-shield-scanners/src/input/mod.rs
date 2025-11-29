//! Input scanners - scan prompts before sending to LLM
//!
//! This module provides security scanners for validating user prompts
//! before sending them to Large Language Models.
//!
//! ## Available Scanners
//!
//! - `BanSubstrings` - Block specific words/phrases
//! - `BanCode` - Detect programming language code
//! - `BanCompetitors` - Filter competitor mentions
//! - `TokenLimit` - Enforce token limits
//! - `InvisibleText` - Detect hidden unicode characters
//! - `RegexScanner` - Custom regex patterns
//! - `Gibberish` - Detect nonsensical text
//! - `Language` - Enforce language requirements
//! - `Secrets` - Detect API keys and tokens
//! - `PromptInjection` - Detect injection attacks
//! - `Toxicity` - ML-based toxicity detection
//! - `Sentiment` - Analyze text sentiment

pub mod ban_substrings;
pub mod ban_code;
pub mod ban_competitors;
pub mod token_limit;
pub mod invisible_text;
pub mod regex_scanner;
pub mod gibberish;
pub mod language;
pub mod secrets;
pub mod prompt_injection;
pub mod toxicity;
pub mod sentiment;

// Re-exports - Scanner types
pub use ban_substrings::BanSubstrings;
pub use ban_code::BanCode;
pub use ban_competitors::BanCompetitors;
pub use token_limit::TokenLimit;
pub use invisible_text::InvisibleText;
pub use regex_scanner::RegexScanner;
pub use gibberish::Gibberish;
pub use language::Language;
pub use secrets::Secrets;
pub use prompt_injection::PromptInjection;
pub use toxicity::Toxicity;
pub use sentiment::Sentiment;

// Re-exports - Configuration types
pub use ban_substrings::{BanSubstringsConfig, MatchType};
pub use ban_code::BanCodeConfig;
pub use ban_competitors::BanCompetitorsConfig;
pub use token_limit::TokenLimitConfig;
pub use invisible_text::InvisibleTextConfig;
pub use regex_scanner::{RegexConfig, RegexPattern};
pub use gibberish::GibberishConfig;
pub use language::LanguageConfig;
pub use secrets::{SecretsConfig, SecretCategory};
pub use prompt_injection::PromptInjectionConfig;
pub use toxicity::{ToxicityConfig, ToxicityCategory};
pub use sentiment::SentimentConfig;
