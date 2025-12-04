//! # Config Integration for SDK
//!
//! SDK-level integration with LLM-Config-Manager for config-driven parameters.
//!
//! ## Features
//!
//! - Load shield parameters from various sources
//! - Auto-refresh configuration at runtime
//! - Environment variable support
//! - Fallback to defaults when config unavailable
//!
//! ## Example
//!
//! ```rust,ignore
//! use llm_shield_sdk::integrations::ConfigIntegration;
//!
//! let integration = ConfigIntegration::builder()
//!     .from_env("SHIELD")
//!     .with_auto_refresh(300)
//!     .with_fallback_to_defaults(true)
//!     .build();
//!
//! // Load parameters
//! let params = integration.load_parameters().await?;
//! let threshold = params.thresholds.get_scanner_threshold("toxicity");
//! ```

use llm_shield_core::{
    adapters::config::{
        ConfigAdapter, ConfigSource, ShieldParameters, ThresholdConfig,
        PatternConfig, ConfigHook,
    },
    Error, Result, Vault,
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// High-level config integration for Shield SDK
pub struct ConfigIntegration {
    /// The underlying config adapter
    adapter: ConfigAdapter,
    /// Whether to auto-refresh configuration
    auto_refresh: bool,
    /// Refresh interval in seconds
    refresh_interval: u64,
    /// Fallback to defaults on error
    fallback_to_defaults: bool,
    /// Cached parameters
    cached: Arc<RwLock<Option<ShieldParameters>>>,
}

impl ConfigIntegration {
    /// Create a new config integration with default settings
    pub fn new() -> Self {
        Self {
            adapter: ConfigAdapter::new(),
            auto_refresh: false,
            refresh_interval: 300,
            fallback_to_defaults: true,
            cached: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a builder for config integration
    pub fn builder() -> ConfigIntegrationBuilder {
        ConfigIntegrationBuilder::new()
    }

    /// Create from environment variables with a prefix
    pub fn from_env(prefix: &str) -> Self {
        Self {
            adapter: ConfigAdapter::from_source(ConfigSource::Environment {
                prefix: prefix.to_string(),
            }),
            auto_refresh: false,
            refresh_interval: 300,
            fallback_to_defaults: true,
            cached: Arc::new(RwLock::new(None)),
        }
    }

    /// Load shield parameters
    pub async fn load_parameters(&self) -> Result<ShieldParameters> {
        // Check cache first
        {
            let cache = self.cached.read().await;
            if let Some(ref params) = *cache {
                return Ok(params.clone());
            }
        }

        // Load from adapter
        match self.adapter.load_parameters().await {
            Ok(params) => {
                // Update cache
                let mut cache = self.cached.write().await;
                *cache = Some(params.clone());
                Ok(params)
            }
            Err(e) if self.fallback_to_defaults => {
                tracing::warn!("Config load failed, using defaults: {}", e);
                Ok(ShieldParameters::default())
            }
            Err(e) => Err(e),
        }
    }

    /// Force reload parameters from source
    pub async fn reload_parameters(&self) -> Result<ShieldParameters> {
        let params = self.adapter.reload_parameters().await?;

        // Update cache
        let mut cache = self.cached.write().await;
        *cache = Some(params.clone());

        Ok(params)
    }

    /// Load parameters into vault for runtime access
    pub async fn load_into_vault(&self, vault: &Vault) -> Result<ShieldParameters> {
        let params = self.load_parameters().await?;
        params.store_in_vault(vault)?;
        Ok(params)
    }

    /// Get threshold configuration
    pub async fn get_thresholds(&self) -> Result<ThresholdConfig> {
        let params = self.load_parameters().await?;
        Ok(params.thresholds)
    }

    /// Get pattern configuration
    pub async fn get_patterns(&self) -> Result<PatternConfig> {
        let params = self.load_parameters().await?;
        Ok(params.patterns)
    }

    /// Get a specific scanner threshold
    pub async fn get_scanner_threshold(&self, scanner: &str) -> Result<f32> {
        let params = self.load_parameters().await?;
        Ok(params.thresholds.get_scanner_threshold(scanner))
    }

    /// Get short-circuit threshold
    pub async fn get_short_circuit_threshold(&self) -> Result<f32> {
        let params = self.load_parameters().await?;
        Ok(params.thresholds.short_circuit)
    }

    /// Check if a feature is enabled
    pub async fn is_feature_enabled(&self, feature: &str) -> Result<bool> {
        let params = self.load_parameters().await?;
        Ok(params.is_feature_enabled(feature))
    }

    /// Register config hooks in the vault
    pub fn register_hooks(&self, vault: &Vault, components: &[&str]) -> Result<()> {
        for component in components {
            let hook = ConfigHook::new(*component);
            hook.register(vault)?;
        }
        Ok(())
    }

    /// Clear cached configuration
    pub async fn clear_cache(&self) {
        let mut cache = self.cached.write().await;
        *cache = None;
    }

    /// Check if auto-refresh is enabled
    pub fn is_auto_refresh_enabled(&self) -> bool {
        self.auto_refresh
    }

    /// Get refresh interval
    pub fn refresh_interval(&self) -> u64 {
        self.refresh_interval
    }
}

impl Default for ConfigIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for ConfigIntegration
pub struct ConfigIntegrationBuilder {
    source: ConfigSource,
    auto_refresh: bool,
    refresh_interval: u64,
    fallback_to_defaults: bool,
}

impl ConfigIntegrationBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            source: ConfigSource::Default,
            auto_refresh: false,
            refresh_interval: 300,
            fallback_to_defaults: true,
        }
    }

    /// Load from environment variables
    pub fn from_env(mut self, prefix: impl Into<String>) -> Self {
        self.source = ConfigSource::Environment {
            prefix: prefix.into(),
        };
        self
    }

    /// Load from a file
    pub fn from_file(mut self, path: impl Into<String>) -> Self {
        self.source = ConfigSource::File { path: path.into() };
        self
    }

    /// Load from a remote URL
    pub fn from_remote(mut self, url: impl Into<String>) -> Self {
        self.source = ConfigSource::Remote { url: url.into() };
        self
    }

    /// Use inline configuration
    pub fn with_inline(mut self, params: ShieldParameters) -> Self {
        self.source = ConfigSource::Inline { config: params };
        self
    }

    /// Enable auto-refresh with interval
    pub fn with_auto_refresh(mut self, interval_seconds: u64) -> Self {
        self.auto_refresh = true;
        self.refresh_interval = interval_seconds;
        self
    }

    /// Disable auto-refresh
    pub fn without_auto_refresh(mut self) -> Self {
        self.auto_refresh = false;
        self
    }

    /// Enable fallback to defaults on error
    pub fn with_fallback_to_defaults(mut self, enabled: bool) -> Self {
        self.fallback_to_defaults = enabled;
        self
    }

    /// Build the ConfigIntegration
    pub fn build(self) -> ConfigIntegration {
        ConfigIntegration {
            adapter: ConfigAdapter::from_source(self.source),
            auto_refresh: self.auto_refresh,
            refresh_interval: self.refresh_interval,
            fallback_to_defaults: self.fallback_to_defaults,
            cached: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for ConfigIntegrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_integration_builder() {
        let integration = ConfigIntegration::builder()
            .from_env("SHIELD")
            .with_auto_refresh(600)
            .with_fallback_to_defaults(true)
            .build();

        assert!(integration.auto_refresh);
        assert_eq!(integration.refresh_interval, 600);
        assert!(integration.fallback_to_defaults);
    }

    #[tokio::test]
    async fn test_load_parameters() {
        let integration = ConfigIntegration::new();
        let params = integration.load_parameters().await.unwrap();

        assert_eq!(params.version, "1.0.0");
        assert!(!params.thresholds.scanner_thresholds.is_empty());
    }

    #[tokio::test]
    async fn test_get_scanner_threshold() {
        let integration = ConfigIntegration::new();
        let threshold = integration.get_scanner_threshold("prompt_injection").await.unwrap();

        assert!(threshold > 0.0 && threshold <= 1.0);
    }

    #[tokio::test]
    async fn test_get_short_circuit_threshold() {
        let integration = ConfigIntegration::new();
        let threshold = integration.get_short_circuit_threshold().await.unwrap();

        assert!(threshold > 0.0 && threshold <= 1.0);
    }

    #[tokio::test]
    async fn test_caching() {
        let integration = ConfigIntegration::new();

        // First load
        let params1 = integration.load_parameters().await.unwrap();

        // Second load should use cache
        let params2 = integration.load_parameters().await.unwrap();

        assert_eq!(params1.version, params2.version);

        // Clear cache and reload
        integration.clear_cache().await;
        let params3 = integration.load_parameters().await.unwrap();

        assert_eq!(params1.version, params3.version);
    }

    #[test]
    fn test_from_env() {
        let integration = ConfigIntegration::from_env("TEST_SHIELD");
        assert!(integration.fallback_to_defaults);
    }

    #[tokio::test]
    async fn test_inline_config() {
        let custom_params = ShieldParameters::new()
            .with_version("2.0.0");

        let integration = ConfigIntegration::builder()
            .with_inline(custom_params)
            .build();

        let params = integration.load_parameters().await.unwrap();
        assert_eq!(params.version, "2.0.0");
    }
}
