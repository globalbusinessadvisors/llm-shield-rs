//! # Scanner Factory
//!
//! Factory methods for creating scanner instances with default or custom configurations.
//!
//! ## Design Principles
//!
//! - Easy scanner creation with sensible defaults
//! - Type-safe configuration
//! - Clear error messages for invalid configurations

use crate::error::{SdkError, SdkResult};
use crate::preset::{InputScannerType, OutputScannerType};
use llm_shield_core::Scanner;
use llm_shield_scanners::input::{
    BanCode, BanCodeConfig, BanCompetitors, BanSubstrings,
    BanSubstringsConfig, Gibberish, InvisibleText,
    Language, LanguageConfig, PromptInjection, PromptInjectionConfig, RegexScanner,
    RegexConfig, RegexPattern, Secrets, SecretsConfig, Sentiment, TokenLimit,
    TokenLimitConfig, Toxicity, ToxicityConfig,
};
use llm_shield_scanners::output::{
    BanTopics, BanTopicsConfig, Bias, Factuality, MaliciousURLs,
    NoRefusal, ReadingTime, ReadingTimeConfig, RegexOutput,
    RegexOutputConfig, Relevance, Sensitive, SensitiveConfig, URLReachability,
};
use std::sync::Arc;

/// Factory for creating input scanners
pub struct InputScannerFactory;

impl InputScannerFactory {
    /// Create a scanner from its type with default configuration
    pub fn create(scanner_type: InputScannerType) -> SdkResult<Arc<dyn Scanner>> {
        match scanner_type {
            InputScannerType::BanSubstrings => {
                // BanSubstrings requires at least one substring, use a placeholder
                Err(SdkError::scanner_init(
                    "BanSubstrings",
                    "BanSubstrings requires explicit configuration with substrings to ban",
                ))
            }
            InputScannerType::BanCode => {
                let scanner = BanCode::default_config()
                    .map_err(|e| SdkError::scanner_init("BanCode", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::BanCompetitors => {
                Err(SdkError::scanner_init(
                    "BanCompetitors",
                    "BanCompetitors requires explicit configuration with competitor names",
                ))
            }
            InputScannerType::Secrets => {
                let scanner = Secrets::default_config()
                    .map_err(|e| SdkError::scanner_init("Secrets", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::Toxicity => {
                let scanner = Toxicity::default_config()
                    .map_err(|e| SdkError::scanner_init("Toxicity", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::PromptInjection => {
                let scanner = PromptInjection::default_config()
                    .map_err(|e| SdkError::scanner_init("PromptInjection", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::InvisibleText => {
                let scanner = InvisibleText::default_config()
                    .map_err(|e| SdkError::scanner_init("InvisibleText", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::Gibberish => {
                let scanner = Gibberish::default_config()
                    .map_err(|e| SdkError::scanner_init("Gibberish", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::Language => {
                let scanner = Language::default_config()
                    .map_err(|e| SdkError::scanner_init("Language", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::TokenLimit => {
                let scanner = TokenLimit::new(TokenLimitConfig::default())
                    .map_err(|e| SdkError::scanner_init("TokenLimit", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            InputScannerType::RegexScanner => {
                Err(SdkError::scanner_init(
                    "RegexScanner",
                    "RegexScanner requires explicit configuration with patterns",
                ))
            }
            InputScannerType::Sentiment => {
                let scanner = Sentiment::default_config()
                    .map_err(|e| SdkError::scanner_init("Sentiment", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
        }
    }

    /// Create BanSubstrings scanner with substrings
    pub fn ban_substrings<I, S>(substrings: I) -> SdkResult<Arc<dyn Scanner>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let scanner = BanSubstrings::with_substrings(substrings)
            .map_err(|e| SdkError::scanner_init("BanSubstrings", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create BanSubstrings scanner with configuration
    pub fn ban_substrings_with_config(config: BanSubstringsConfig) -> SdkResult<Arc<dyn Scanner>> {
        let scanner = BanSubstrings::new(config)
            .map_err(|e| SdkError::scanner_init("BanSubstrings", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create BanCode scanner with default configuration
    pub fn ban_code() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::BanCode)
    }

    /// Create BanCode scanner with configuration
    pub fn ban_code_with_config(config: BanCodeConfig) -> SdkResult<Arc<dyn Scanner>> {
        let scanner = BanCode::new(config)
            .map_err(|e| SdkError::scanner_init("BanCode", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create BanCompetitors scanner with competitor names
    pub fn ban_competitors<I, S>(competitors: I) -> SdkResult<Arc<dyn Scanner>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let scanner = BanCompetitors::with_competitors(competitors)
            .map_err(|e| SdkError::scanner_init("BanCompetitors", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create Secrets scanner with default configuration
    pub fn secrets() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::Secrets)
    }

    /// Create Secrets scanner with configuration
    pub fn secrets_with_config(config: SecretsConfig) -> SdkResult<Arc<dyn Scanner>> {
        let scanner = Secrets::new(config)
            .map_err(|e| SdkError::scanner_init("Secrets", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create Toxicity scanner with default configuration
    pub fn toxicity() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::Toxicity)
    }

    /// Create Toxicity scanner with configuration
    pub fn toxicity_with_config(config: ToxicityConfig) -> SdkResult<Arc<dyn Scanner>> {
        let scanner = Toxicity::new(config)
            .map_err(|e| SdkError::scanner_init("Toxicity", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create PromptInjection scanner with default configuration
    pub fn prompt_injection() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::PromptInjection)
    }

    /// Create PromptInjection scanner with configuration
    pub fn prompt_injection_with_config(config: PromptInjectionConfig) -> SdkResult<Arc<dyn Scanner>> {
        let scanner = PromptInjection::new(config)
            .map_err(|e| SdkError::scanner_init("PromptInjection", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create InvisibleText scanner with default configuration
    pub fn invisible_text() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::InvisibleText)
    }

    /// Create Gibberish scanner with default configuration
    pub fn gibberish() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::Gibberish)
    }

    /// Create Language scanner with default configuration
    pub fn language() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::Language)
    }

    /// Create Language scanner with allowed languages
    pub fn language_with_allowed<I, S>(languages: I) -> SdkResult<Arc<dyn Scanner>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let config = LanguageConfig {
            allowed_languages: languages.into_iter().map(|l| l.into()).collect(),
            ..Default::default()
        };
        let scanner = Language::new(config)
            .map_err(|e| SdkError::scanner_init("Language", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create TokenLimit scanner with default configuration
    pub fn token_limit() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::TokenLimit)
    }

    /// Create TokenLimit scanner with custom limit
    pub fn token_limit_with_max(limit: usize) -> SdkResult<Arc<dyn Scanner>> {
        let config = TokenLimitConfig {
            limit,
            ..Default::default()
        };
        let scanner = TokenLimit::new(config)
            .map_err(|e| SdkError::scanner_init("TokenLimit", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create RegexScanner with patterns (name, pattern pairs)
    pub fn regex<I>(patterns: I) -> SdkResult<Arc<dyn Scanner>>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        let regex_patterns: Vec<RegexPattern> = patterns
            .into_iter()
            .map(|(name, pattern)| RegexPattern {
                name,
                pattern,
                risk_score: 1.0,
            })
            .collect();
        let config = RegexConfig {
            patterns: regex_patterns,
            redact: false,
        };
        let scanner = RegexScanner::new(config)
            .map_err(|e| SdkError::scanner_init("RegexScanner", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create Sentiment scanner with default configuration
    pub fn sentiment() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(InputScannerType::Sentiment)
    }
}

/// Factory for creating output scanners
pub struct OutputScannerFactory;

impl OutputScannerFactory {
    /// Create a scanner from its type with default configuration
    pub fn create(scanner_type: OutputScannerType) -> SdkResult<Arc<dyn Scanner>> {
        match scanner_type {
            OutputScannerType::Sensitive => {
                let scanner = Sensitive::default_config()
                    .map_err(|e| SdkError::scanner_init("Sensitive", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::NoRefusal => {
                let scanner = NoRefusal::default_config()
                    .map_err(|e| SdkError::scanner_init("NoRefusal", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::Relevance => {
                let scanner = Relevance::default_config()
                    .map_err(|e| SdkError::scanner_init("Relevance", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::BanTopics => {
                Err(SdkError::scanner_init(
                    "BanTopics",
                    "BanTopics requires explicit configuration with topics to ban",
                ))
            }
            OutputScannerType::Bias => {
                let scanner = Bias::default_config()
                    .map_err(|e| SdkError::scanner_init("Bias", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::MaliciousURLs => {
                let scanner = MaliciousURLs::default_config()
                    .map_err(|e| SdkError::scanner_init("MaliciousURLs", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::Factuality => {
                let scanner = Factuality::default_config()
                    .map_err(|e| SdkError::scanner_init("Factuality", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::ReadingTime => {
                let scanner = ReadingTime::default_config()
                    .map_err(|e| SdkError::scanner_init("ReadingTime", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::URLReachability => {
                let scanner = URLReachability::default_config()
                    .map_err(|e| SdkError::scanner_init("URLReachability", e.to_string()))?;
                Ok(Arc::new(scanner))
            }
            OutputScannerType::RegexOutput => {
                Err(SdkError::scanner_init(
                    "RegexOutput",
                    "RegexOutput requires explicit configuration with patterns",
                ))
            }
        }
    }

    /// Create Sensitive scanner with default configuration
    pub fn sensitive() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::Sensitive)
    }

    /// Create Sensitive scanner with configuration
    pub fn sensitive_with_config(config: SensitiveConfig) -> SdkResult<Arc<dyn Scanner>> {
        let scanner = Sensitive::new(config)
            .map_err(|e| SdkError::scanner_init("Sensitive", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create NoRefusal scanner with default configuration
    pub fn no_refusal() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::NoRefusal)
    }

    /// Create Relevance scanner with default configuration
    pub fn relevance() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::Relevance)
    }

    /// Create BanTopics scanner with topic names
    /// Uses default keywords for common topics, or creates simple keyword-based topics
    pub fn ban_topics<I, S>(topic_names: I) -> SdkResult<Arc<dyn Scanner>>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        use llm_shield_scanners::output::BannedTopic;
        use llm_shield_core::Severity;

        let topics: Vec<BannedTopic> = topic_names
            .into_iter()
            .map(|t| {
                let name: String = t.into();
                BannedTopic {
                    name: name.clone(),
                    keywords: vec![name.clone()],
                    severity: Severity::Medium,
                }
            })
            .collect();
        let config = BanTopicsConfig {
            topics,
            ..Default::default()
        };
        let scanner = BanTopics::new(config)
            .map_err(|e| SdkError::scanner_init("BanTopics", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create Bias scanner with default configuration
    pub fn bias() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::Bias)
    }

    /// Create MaliciousURLs scanner with default configuration
    pub fn malicious_urls() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::MaliciousURLs)
    }

    /// Create Factuality scanner with default configuration
    pub fn factuality() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::Factuality)
    }

    /// Create ReadingTime scanner with default configuration
    pub fn reading_time() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::ReadingTime)
    }

    /// Create ReadingTime scanner with custom limits
    pub fn reading_time_with_limits(min_seconds: u32, max_seconds: u32) -> SdkResult<Arc<dyn Scanner>> {
        let config = ReadingTimeConfig {
            min_time_seconds: min_seconds,
            max_time_seconds: max_seconds,
            ..Default::default()
        };
        let scanner = ReadingTime::new(config)
            .map_err(|e| SdkError::scanner_init("ReadingTime", e.to_string()))?;
        Ok(Arc::new(scanner))
    }

    /// Create URLReachability scanner with default configuration
    pub fn url_reachability() -> SdkResult<Arc<dyn Scanner>> {
        Self::create(OutputScannerType::URLReachability)
    }

    /// Create RegexOutput scanner with patterns (name, pattern pairs)
    pub fn regex<I>(patterns: I) -> SdkResult<Arc<dyn Scanner>>
    where
        I: IntoIterator<Item = (String, String)>,
    {
        use llm_shield_scanners::output::RegexPattern;
        use llm_shield_core::Severity;

        let regex_patterns: Vec<RegexPattern> = patterns
            .into_iter()
            .map(|(name, pattern)| RegexPattern {
                name,
                pattern,
                severity: Severity::Medium,
                case_insensitive: false,
            })
            .collect();
        let config = RegexOutputConfig {
            patterns: regex_patterns,
            ..Default::default()
        };
        let scanner = RegexOutput::new(config)
            .map_err(|e| SdkError::scanner_init("RegexOutput", e.to_string()))?;
        Ok(Arc::new(scanner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_scanner_factory_secrets() {
        let scanner = InputScannerFactory::secrets();
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_input_scanner_factory_ban_substrings() {
        let scanner = InputScannerFactory::ban_substrings(["test", "word"]);
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_input_scanner_factory_requires_config() {
        // BanSubstrings without config should fail
        let result = InputScannerFactory::create(InputScannerType::BanSubstrings);
        assert!(result.is_err());
    }

    #[test]
    fn test_output_scanner_factory_sensitive() {
        let scanner = OutputScannerFactory::sensitive();
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_output_scanner_factory_ban_topics() {
        let scanner = OutputScannerFactory::ban_topics(["politics", "religion"]);
        assert!(scanner.is_ok());
    }

    #[test]
    fn test_output_scanner_factory_requires_config() {
        // BanTopics without config should fail
        let result = OutputScannerFactory::create(OutputScannerType::BanTopics);
        assert!(result.is_err());
    }
}
