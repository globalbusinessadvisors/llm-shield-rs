//! # Config Manager Adapter
//!
//! Thin adapter layer for consuming configuration from LLM-Config-Manager.
//!
//! ## Purpose
//!
//! This module provides config-driven shield parameter loading, enabling
//! dynamic configuration of thresholds, patterns, and enforcement parameters.
//!
//! ## Integration Pattern
//!
//! ```text
//! LLM-Config-Manager → ConfigAdapter → Shield Parameters
//!                            ↓
//!            ThresholdConfig, PatternConfig, etc.
//!                            ↓
//!              Scanner Configuration at Runtime
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llm_shield_core::adapters::config::{ConfigAdapter, ConfigSource};
//!
//! // Create adapter from config source
//! let adapter = ConfigAdapter::from_source(ConfigSource::Remote {
//!     url: "https://config.example.com".to_string(),
//! }).await?;
//!
//! // Load shield parameters
//! let params = adapter.load_shield_parameters().await?;
//!
//! // Apply to scanner configuration
//! let threshold = params.thresholds.get("prompt_injection").unwrap_or(&0.9);
//! ```

use crate::{Error, Result, Vault};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Shield parameters loaded from config manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldParameters {
    /// Version of the configuration
    pub version: String,
    /// Threshold configurations for various scanners
    pub thresholds: ThresholdConfig,
    /// Pattern configurations (allowed/blocked)
    pub patterns: PatternConfig,
    /// Enforcement parameters
    pub enforcement: EnforcementConfig,
    /// Scanner-specific configurations
    pub scanners: HashMap<String, serde_json::Value>,
    /// Feature flags
    pub features: HashMap<String, bool>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ShieldParameters {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            thresholds: ThresholdConfig::default(),
            patterns: PatternConfig::default(),
            enforcement: EnforcementConfig::default(),
            scanners: HashMap::new(),
            features: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

impl ShieldParameters {
    /// Create new shield parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Get a scanner-specific configuration
    pub fn get_scanner_config<T: for<'de> Deserialize<'de>>(
        &self,
        scanner_name: &str,
    ) -> Option<T> {
        self.scanners
            .get(scanner_name)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }

    /// Store parameters in vault for runtime access
    pub fn store_in_vault(&self, vault: &Vault) -> Result<()> {
        vault.set("shield_parameters", self)?;
        vault.set("shield_parameters:version", &self.version)?;
        Ok(())
    }

    /// Load parameters from vault
    pub fn load_from_vault(vault: &Vault) -> Result<Option<Self>> {
        vault.get("shield_parameters")
    }
}

/// Threshold configuration for scanners
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Short-circuit threshold (stop scanning if risk exceeds this)
    pub short_circuit: f32,
    /// Per-scanner risk thresholds
    pub scanner_thresholds: HashMap<String, f32>,
    /// Severity escalation thresholds
    pub severity_thresholds: SeverityThresholds,
    /// Confidence thresholds for detections
    pub confidence_thresholds: HashMap<String, f32>,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        let mut scanner_thresholds = HashMap::new();
        scanner_thresholds.insert("prompt_injection".to_string(), 0.8);
        scanner_thresholds.insert("secrets".to_string(), 0.9);
        scanner_thresholds.insert("toxicity".to_string(), 0.7);
        scanner_thresholds.insert("pii".to_string(), 0.85);
        scanner_thresholds.insert("ban_topics".to_string(), 0.75);

        let mut confidence_thresholds = HashMap::new();
        confidence_thresholds.insert("default".to_string(), 0.5);
        confidence_thresholds.insert("high_security".to_string(), 0.3);

        Self {
            short_circuit: 0.95,
            scanner_thresholds,
            severity_thresholds: SeverityThresholds::default(),
            confidence_thresholds,
        }
    }
}

impl ThresholdConfig {
    /// Get threshold for a specific scanner
    pub fn get_scanner_threshold(&self, scanner: &str) -> f32 {
        self.scanner_thresholds
            .get(scanner)
            .copied()
            .unwrap_or(0.5)
    }

    /// Get confidence threshold
    pub fn get_confidence_threshold(&self, category: &str) -> f32 {
        self.confidence_thresholds
            .get(category)
            .or_else(|| self.confidence_thresholds.get("default"))
            .copied()
            .unwrap_or(0.5)
    }
}

/// Severity thresholds for risk scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityThresholds {
    /// Threshold for Critical severity
    pub critical: f32,
    /// Threshold for High severity
    pub high: f32,
    /// Threshold for Medium severity
    pub medium: f32,
    /// Threshold for Low severity
    pub low: f32,
}

impl Default for SeverityThresholds {
    fn default() -> Self {
        Self {
            critical: 0.9,
            high: 0.7,
            medium: 0.4,
            low: 0.01,
        }
    }
}

/// Pattern configuration for content filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Allowed patterns (whitelisted)
    pub allowed: Vec<PatternRule>,
    /// Blocked patterns (blacklisted)
    pub blocked: Vec<PatternRule>,
    /// Regex patterns for custom matching
    pub regex_patterns: Vec<RegexPatternRule>,
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            allowed: Vec::new(),
            blocked: Vec::new(),
            regex_patterns: Vec::new(),
        }
    }
}

impl PatternConfig {
    /// Add an allowed pattern
    pub fn allow(mut self, pattern: PatternRule) -> Self {
        self.allowed.push(pattern);
        self
    }

    /// Add a blocked pattern
    pub fn block(mut self, pattern: PatternRule) -> Self {
        self.blocked.push(pattern);
        self
    }

    /// Check if a string matches any blocked pattern
    pub fn is_blocked(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.blocked.iter().any(|p| {
            if p.case_sensitive {
                text.contains(&p.pattern)
            } else {
                text_lower.contains(&p.pattern.to_lowercase())
            }
        })
    }

    /// Check if a string matches any allowed pattern
    pub fn is_allowed(&self, text: &str) -> bool {
        if self.allowed.is_empty() {
            return true; // No whitelist means all allowed
        }
        let text_lower = text.to_lowercase();
        self.allowed.iter().any(|p| {
            if p.case_sensitive {
                text.contains(&p.pattern)
            } else {
                text_lower.contains(&p.pattern.to_lowercase())
            }
        })
    }
}

/// A pattern rule for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRule {
    /// The pattern to match
    pub pattern: String,
    /// Category of the pattern
    pub category: String,
    /// Whether matching is case sensitive
    pub case_sensitive: bool,
    /// Description of why this pattern is allowed/blocked
    pub description: Option<String>,
}

impl PatternRule {
    /// Create a new pattern rule
    pub fn new(pattern: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            category: category.into(),
            case_sensitive: false,
            description: None,
        }
    }

    /// Make the pattern case sensitive
    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A regex pattern rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexPatternRule {
    /// The regex pattern
    pub pattern: String,
    /// Category of the pattern
    pub category: String,
    /// Severity when matched
    pub severity: String,
    /// Action to take when matched
    pub action: String,
}

/// Enforcement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementConfig {
    /// Default action when violations are detected
    pub default_action: String,
    /// Whether to redact sensitive content
    pub enable_redaction: bool,
    /// Whether to log all decisions
    pub enable_audit_log: bool,
    /// Maximum allowed risk score before blocking
    pub max_risk_score: f32,
    /// Per-category enforcement overrides
    pub category_overrides: HashMap<String, CategoryEnforcement>,
}

impl Default for EnforcementConfig {
    fn default() -> Self {
        Self {
            default_action: "block".to_string(),
            enable_redaction: true,
            enable_audit_log: true,
            max_risk_score: 0.9,
            category_overrides: HashMap::new(),
        }
    }
}

/// Per-category enforcement settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryEnforcement {
    /// Action for this category
    pub action: String,
    /// Whether to redact
    pub redact: bool,
    /// Custom threshold for this category
    pub threshold: Option<f32>,
}

/// Source for loading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConfigSource {
    /// Load from a local file
    File { path: String },
    /// Load from a remote URL
    Remote { url: String },
    /// Load from environment variables
    Environment { prefix: String },
    /// Use inline configuration
    Inline { config: ShieldParameters },
    /// No external configuration (use defaults)
    Default,
}

impl Default for ConfigSource {
    fn default() -> Self {
        Self::Default
    }
}

/// Trait for loading configuration
#[async_trait]
pub trait ConfigLoader: Send + Sync {
    /// Load shield parameters from the source
    async fn load(&self) -> Result<ShieldParameters>;

    /// Reload configuration (for hot-reload support)
    async fn reload(&self) -> Result<ShieldParameters> {
        self.load().await
    }

    /// Watch for configuration changes
    async fn watch(&self) -> Result<()> {
        // Default implementation: no-op
        Ok(())
    }

    /// Check if the config source is available
    async fn is_available(&self) -> bool {
        true
    }
}

/// Main config adapter for Shield integration
pub struct ConfigAdapter {
    /// The configuration source
    source: ConfigSource,
    /// Cached parameters
    cached_params: Arc<tokio::sync::RwLock<Option<ShieldParameters>>>,
    /// Enable auto-refresh
    auto_refresh: bool,
    /// Refresh interval in seconds
    refresh_interval: u64,
}

impl ConfigAdapter {
    /// Create a new config adapter with default source
    pub fn new() -> Self {
        Self {
            source: ConfigSource::Default,
            cached_params: Arc::new(tokio::sync::RwLock::new(None)),
            auto_refresh: false,
            refresh_interval: 300,
        }
    }

    /// Create from a specific source
    pub fn from_source(source: ConfigSource) -> Self {
        Self {
            source,
            cached_params: Arc::new(tokio::sync::RwLock::new(None)),
            auto_refresh: false,
            refresh_interval: 300,
        }
    }

    /// Enable auto-refresh
    pub fn with_auto_refresh(mut self, interval_seconds: u64) -> Self {
        self.auto_refresh = true;
        self.refresh_interval = interval_seconds;
        self
    }

    /// Load shield parameters
    pub async fn load_parameters(&self) -> Result<ShieldParameters> {
        // Check cache first
        {
            let cache = self.cached_params.read().await;
            if let Some(params) = cache.as_ref() {
                return Ok(params.clone());
            }
        }

        // Load from source
        let params = self.load_from_source().await?;

        // Update cache
        {
            let mut cache = self.cached_params.write().await;
            *cache = Some(params.clone());
        }

        Ok(params)
    }

    /// Force reload parameters from source
    pub async fn reload_parameters(&self) -> Result<ShieldParameters> {
        let params = self.load_from_source().await?;

        // Update cache
        {
            let mut cache = self.cached_params.write().await;
            *cache = Some(params.clone());
        }

        Ok(params)
    }

    /// Load parameters and store in vault
    pub async fn load_into_vault(&self, vault: &Vault) -> Result<ShieldParameters> {
        let params = self.load_parameters().await?;
        params.store_in_vault(vault)?;
        Ok(params)
    }

    /// Internal: load from the configured source
    async fn load_from_source(&self) -> Result<ShieldParameters> {
        match &self.source {
            ConfigSource::Default => Ok(ShieldParameters::default()),
            ConfigSource::Inline { config } => Ok(config.clone()),
            ConfigSource::File { path } => self.load_from_file(path).await,
            ConfigSource::Remote { url } => self.load_from_remote(url).await,
            ConfigSource::Environment { prefix } => self.load_from_env(prefix),
        }
    }

    /// Load from a file (placeholder - actual implementation would read file)
    async fn load_from_file(&self, path: &str) -> Result<ShieldParameters> {
        // In a real implementation, this would read and parse the file
        // For now, we return defaults with the path noted
        let mut params = ShieldParameters::default();
        params.metadata.insert(
            "source".to_string(),
            serde_json::json!({ "type": "file", "path": path }),
        );
        Ok(params)
    }

    /// Load from a remote URL (placeholder - actual implementation would fetch)
    async fn load_from_remote(&self, url: &str) -> Result<ShieldParameters> {
        // In a real implementation, this would fetch from the URL
        // For now, we return defaults with the URL noted
        let mut params = ShieldParameters::default();
        params.metadata.insert(
            "source".to_string(),
            serde_json::json!({ "type": "remote", "url": url }),
        );
        Ok(params)
    }

    /// Load from environment variables
    fn load_from_env(&self, prefix: &str) -> Result<ShieldParameters> {
        let mut params = ShieldParameters::default();

        // Load thresholds from env
        if let Ok(threshold) = std::env::var(format!("{}_SHORT_CIRCUIT_THRESHOLD", prefix)) {
            if let Ok(value) = threshold.parse::<f32>() {
                params.thresholds.short_circuit = value;
            }
        }

        // Load enforcement settings
        if let Ok(action) = std::env::var(format!("{}_DEFAULT_ACTION", prefix)) {
            params.enforcement.default_action = action;
        }

        if let Ok(redact) = std::env::var(format!("{}_ENABLE_REDACTION", prefix)) {
            params.enforcement.enable_redaction = redact.to_lowercase() == "true";
        }

        params.metadata.insert(
            "source".to_string(),
            serde_json::json!({ "type": "environment", "prefix": prefix }),
        );

        Ok(params)
    }

    /// Get a specific threshold
    pub async fn get_threshold(&self, scanner: &str) -> Result<f32> {
        let params = self.load_parameters().await?;
        Ok(params.thresholds.get_scanner_threshold(scanner))
    }

    /// Check if a feature is enabled
    pub async fn is_feature_enabled(&self, feature: &str) -> Result<bool> {
        let params = self.load_parameters().await?;
        Ok(params.is_feature_enabled(feature))
    }
}

impl Default for ConfigAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Default config loader implementation
pub struct DefaultConfigLoader {
    source: ConfigSource,
}

impl DefaultConfigLoader {
    /// Create a new default config loader
    pub fn new(source: ConfigSource) -> Self {
        Self { source }
    }
}

#[async_trait]
impl ConfigLoader for DefaultConfigLoader {
    async fn load(&self) -> Result<ShieldParameters> {
        let adapter = ConfigAdapter::from_source(self.source.clone());
        adapter.load_parameters().await
    }
}

/// Runtime configuration hook for scanner integration
///
/// This struct can be stored in the Vault and accessed by scanners
/// to get configuration at runtime.
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigHook {
    /// The scanner or component this config is for
    pub component: String,
    /// Whether dynamic config loading is enabled
    pub enabled: bool,
    /// Fallback to defaults if config loading fails
    pub fallback_to_defaults: bool,
}

impl ConfigHook {
    /// Create a new config hook
    pub fn new(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            enabled: true,
            fallback_to_defaults: true,
        }
    }

    /// Disable the hook
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Disable fallback to defaults
    pub fn strict(mut self) -> Self {
        self.fallback_to_defaults = false;
        self
    }

    /// Store this hook in the vault
    pub fn register(self, vault: &Vault) -> Result<()> {
        vault.set(format!("config_hook:{}", self.component), &self)
            .map_err(|e| Error::vault(format!("Failed to register config hook: {}", e)))
    }

    /// Retrieve a hook from the vault
    pub fn get(vault: &Vault, component: &str) -> Result<Option<Self>> {
        vault.get(format!("config_hook:{}", component))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shield_parameters_default() {
        let params = ShieldParameters::default();
        assert_eq!(params.version, "1.0.0");
        assert!(!params.thresholds.scanner_thresholds.is_empty());
    }

    #[test]
    fn test_threshold_config() {
        let config = ThresholdConfig::default();
        assert_eq!(config.get_scanner_threshold("prompt_injection"), 0.8);
        assert_eq!(config.get_scanner_threshold("unknown"), 0.5);
    }

    #[test]
    fn test_pattern_config() {
        let config = PatternConfig::default()
            .block(PatternRule::new("password", "sensitive"))
            .allow(PatternRule::new("hello", "greeting"));

        assert!(config.is_blocked("Enter your password here"));
        assert!(!config.is_blocked("Just saying hi"));
        assert!(config.is_allowed("Say hello to everyone"));
    }

    #[test]
    fn test_pattern_rule() {
        let rule = PatternRule::new("test", "category")
            .case_sensitive()
            .with_description("Test pattern");

        assert!(rule.case_sensitive);
        assert_eq!(rule.description, Some("Test pattern".to_string()));
    }

    #[tokio::test]
    async fn test_config_adapter_default() {
        let adapter = ConfigAdapter::new();
        let params = adapter.load_parameters().await.unwrap();
        assert_eq!(params.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_config_adapter_inline() {
        let inline_params = ShieldParameters::new()
            .with_version("2.0.0");

        let adapter = ConfigAdapter::from_source(ConfigSource::Inline {
            config: inline_params,
        });

        let params = adapter.load_parameters().await.unwrap();
        assert_eq!(params.version, "2.0.0");
    }

    #[test]
    fn test_config_hook() {
        let hook = ConfigHook::new("secrets-scanner")
            .strict();

        assert!(hook.enabled);
        assert!(!hook.fallback_to_defaults);
    }

    #[test]
    fn test_severity_thresholds() {
        let thresholds = SeverityThresholds::default();
        assert_eq!(thresholds.critical, 0.9);
        assert_eq!(thresholds.high, 0.7);
        assert_eq!(thresholds.medium, 0.4);
        assert_eq!(thresholds.low, 0.01);
    }

    #[test]
    fn test_enforcement_config() {
        let config = EnforcementConfig::default();
        assert_eq!(config.default_action, "block");
        assert!(config.enable_redaction);
        assert!(config.enable_audit_log);
    }
}
