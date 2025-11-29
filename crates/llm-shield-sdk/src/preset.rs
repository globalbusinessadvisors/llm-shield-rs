//! # Security Presets
//!
//! Pre-configured security levels for different use cases.
//!
//! ## Available Presets
//!
//! - **Strict**: Maximum security for regulated industries
//! - **Standard**: Balanced security for general applications
//! - **Permissive**: Minimal security for development/testing
//! - **Custom**: Build your own configuration

use crate::config::{ParallelConfig, ShieldConfig, ScanMode};
use serde::{Deserialize, Serialize};

/// Security preset levels
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_sdk::prelude::*;
///
/// // Use a preset
/// let shield = Shield::builder()
///     .with_preset(Preset::Strict)
///     .build()?;
///
/// // Or use convenience methods
/// let shield = Shield::strict()?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    /// Maximum security for regulated industries (banking, healthcare)
    ///
    /// Includes all scanners with strict thresholds:
    /// - All input scanners enabled
    /// - All output scanners enabled
    /// - Low risk tolerance (short-circuit at 0.7)
    /// - No parallel execution for deterministic results
    Strict,

    /// Balanced security for general-purpose applications
    ///
    /// Includes common scanners with reasonable thresholds:
    /// - Core input scanners (secrets, toxicity, prompt injection)
    /// - Core output scanners (sensitive data, malicious URLs)
    /// - Moderate risk tolerance (short-circuit at 0.9)
    /// - Parallel execution enabled
    Standard,

    /// Minimal security for development and testing
    ///
    /// Includes only essential scanners:
    /// - Basic content filtering
    /// - High risk tolerance
    /// - Fast execution
    Permissive,

    /// Custom configuration (no preset scanners)
    Custom,
}

impl Default for Preset {
    fn default() -> Self {
        Self::Standard
    }
}

impl Preset {
    /// Get the shield configuration for this preset
    pub fn config(&self) -> ShieldConfig {
        match self {
            Preset::Strict => ShieldConfig {
                scan_mode: ScanMode::Both,
                parallel: ParallelConfig::disabled(), // Deterministic
                short_circuit_threshold: Some(0.7), // Low tolerance
                timeout_ms: Some(60_000), // 1 minute
                enable_tracing: true,
                enable_caching: false, // No caching for strict mode
                cache_ttl_seconds: 0,
                max_batch_size: 50,
            },
            Preset::Standard => ShieldConfig {
                scan_mode: ScanMode::Both,
                parallel: ParallelConfig::with_concurrency(4),
                short_circuit_threshold: Some(0.9),
                timeout_ms: Some(30_000),
                enable_tracing: true,
                enable_caching: true,
                cache_ttl_seconds: 300,
                max_batch_size: 100,
            },
            Preset::Permissive => ShieldConfig {
                scan_mode: ScanMode::Both,
                parallel: ParallelConfig::with_concurrency(8),
                short_circuit_threshold: None, // No short-circuit
                timeout_ms: Some(10_000),
                enable_tracing: false,
                enable_caching: true,
                cache_ttl_seconds: 600,
                max_batch_size: 200,
            },
            Preset::Custom => ShieldConfig::default(),
        }
    }

    /// Get input scanner types for this preset
    pub fn input_scanners(&self) -> Vec<InputScannerType> {
        match self {
            Preset::Strict => vec![
                InputScannerType::BanSubstrings,
                InputScannerType::BanCode,
                InputScannerType::BanCompetitors,
                InputScannerType::Secrets,
                InputScannerType::Toxicity,
                InputScannerType::PromptInjection,
                InputScannerType::InvisibleText,
                InputScannerType::Gibberish,
                InputScannerType::Language,
                InputScannerType::TokenLimit,
                InputScannerType::Sentiment,
            ],
            Preset::Standard => vec![
                InputScannerType::Secrets,
                InputScannerType::Toxicity,
                InputScannerType::PromptInjection,
                InputScannerType::InvisibleText,
                InputScannerType::TokenLimit,
            ],
            Preset::Permissive => vec![
                InputScannerType::Secrets,
                InputScannerType::InvisibleText,
            ],
            Preset::Custom => vec![],
        }
    }

    /// Get output scanner types for this preset
    pub fn output_scanners(&self) -> Vec<OutputScannerType> {
        match self {
            Preset::Strict => vec![
                OutputScannerType::Sensitive,
                OutputScannerType::NoRefusal,
                OutputScannerType::Relevance,
                OutputScannerType::BanTopics,
                OutputScannerType::Bias,
                OutputScannerType::MaliciousURLs,
                OutputScannerType::Factuality,
                OutputScannerType::URLReachability,
            ],
            Preset::Standard => vec![
                OutputScannerType::Sensitive,
                OutputScannerType::MaliciousURLs,
                OutputScannerType::NoRefusal,
            ],
            Preset::Permissive => vec![
                OutputScannerType::Sensitive,
            ],
            Preset::Custom => vec![],
        }
    }

    /// Get human-readable description of this preset
    pub fn description(&self) -> &'static str {
        match self {
            Preset::Strict => "Maximum security for regulated industries (banking, healthcare). All scanners enabled with strict thresholds.",
            Preset::Standard => "Balanced security for general-purpose applications. Core scanners with reasonable thresholds.",
            Preset::Permissive => "Minimal security for development and testing. Essential scanners only.",
            Preset::Custom => "Custom configuration. No preset scanners - build your own configuration.",
        }
    }
}

/// Input scanner types for preset configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputScannerType {
    BanSubstrings,
    BanCode,
    BanCompetitors,
    Secrets,
    Toxicity,
    PromptInjection,
    InvisibleText,
    Gibberish,
    Language,
    TokenLimit,
    RegexScanner,
    Sentiment,
}

/// Output scanner types for preset configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputScannerType {
    Sensitive,
    NoRefusal,
    Relevance,
    BanTopics,
    Bias,
    MaliciousURLs,
    Factuality,
    ReadingTime,
    URLReachability,
    RegexOutput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_default() {
        assert_eq!(Preset::default(), Preset::Standard);
    }

    #[test]
    fn test_strict_preset() {
        let preset = Preset::Strict;
        let config = preset.config();

        assert!(!config.parallel.enabled);
        assert_eq!(config.short_circuit_threshold, Some(0.7));
        assert!(!config.enable_caching);

        let input_scanners = preset.input_scanners();
        assert!(input_scanners.len() >= 10);

        let output_scanners = preset.output_scanners();
        assert!(output_scanners.len() >= 6);
    }

    #[test]
    fn test_standard_preset() {
        let preset = Preset::Standard;
        let config = preset.config();

        assert!(config.parallel.enabled);
        assert_eq!(config.short_circuit_threshold, Some(0.9));
        assert!(config.enable_caching);

        let input_scanners = preset.input_scanners();
        assert!(input_scanners.contains(&InputScannerType::Secrets));
        assert!(input_scanners.contains(&InputScannerType::Toxicity));

        let output_scanners = preset.output_scanners();
        assert!(output_scanners.contains(&OutputScannerType::Sensitive));
    }

    #[test]
    fn test_permissive_preset() {
        let preset = Preset::Permissive;
        let config = preset.config();

        assert!(config.parallel.enabled);
        assert!(config.short_circuit_threshold.is_none());

        let input_scanners = preset.input_scanners();
        assert!(input_scanners.len() <= 3);

        let output_scanners = preset.output_scanners();
        assert!(output_scanners.len() <= 2);
    }

    #[test]
    fn test_custom_preset() {
        let preset = Preset::Custom;

        let input_scanners = preset.input_scanners();
        assert!(input_scanners.is_empty());

        let output_scanners = preset.output_scanners();
        assert!(output_scanners.is_empty());
    }

    #[test]
    fn test_preset_descriptions() {
        assert!(!Preset::Strict.description().is_empty());
        assert!(!Preset::Standard.description().is_empty());
        assert!(!Preset::Permissive.description().is_empty());
        assert!(!Preset::Custom.description().is_empty());
    }

    #[test]
    fn test_preset_serialization() {
        let preset = Preset::Standard;
        let json = serde_json::to_string(&preset).unwrap();
        let deserialized: Preset = serde_json::from_str(&json).unwrap();
        assert_eq!(preset, deserialized);
    }
}
