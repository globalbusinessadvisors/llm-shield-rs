//! Main application configuration

use super::{AuthConfig, CloudConfig, ConfigError, ObservabilityConfig, RateLimitConfig, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Observability configuration
    pub observability: ObservabilityConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Models configuration
    pub models: ModelsConfig,

    /// Cloud integration configuration
    #[serde(default)]
    pub cloud: CloudConfig,
}

impl AppConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        self.server.validate()?;
        self.auth.validate()?;
        self.rate_limit.validate()?;
        self.observability.validate()?;
        self.cache.validate()?;
        self.models.validate()?;
        self.cloud.validate()?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            auth: AuthConfig::default(),
            rate_limit: RateLimitConfig::default(),
            observability: ObservabilityConfig::default(),
            cache: CacheConfig::default(),
            models: ModelsConfig::default(),
            cloud: CloudConfig::default(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Maximum request body size in bytes
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,

    /// Number of worker threads
    #[serde(default = "default_workers")]
    pub workers: usize,
}

impl ServerConfig {
    /// Get bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get request timeout
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Validate server configuration
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port cannot be 0".to_string(),
            ));
        }

        // Note: port is u16, so it's always <= 65535. Check removed.

        if self.timeout_secs == 0 {
            return Err(ConfigError::ValidationError(
                "Timeout must be greater than 0".to_string(),
            ));
        }

        if self.max_body_size == 0 {
            return Err(ConfigError::ValidationError(
                "Max body size must be greater than 0".to_string(),
            ));
        }

        if self.workers == 0 {
            return Err(ConfigError::ValidationError(
                "Workers must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            timeout_secs: default_timeout(),
            max_body_size: default_max_body_size(),
            workers: default_workers(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable result caching
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,

    /// Maximum cache size
    #[serde(default = "default_cache_max_size")]
    pub max_size: usize,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_secs: u64,
}

impl CacheConfig {
    /// Get cache TTL as Duration
    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl_secs)
    }

    /// Validate cache configuration
    pub fn validate(&self) -> Result<()> {
        if self.enabled && self.max_size == 0 {
            return Err(ConfigError::ValidationError(
                "Cache max size must be greater than 0 when enabled".to_string(),
            ));
        }

        if self.enabled && self.ttl_secs == 0 {
            return Err(ConfigError::ValidationError(
                "Cache TTL must be greater than 0 when enabled".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            max_size: default_cache_max_size(),
            ttl_secs: default_cache_ttl(),
        }
    }
}

/// Models configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    /// Path to model registry file
    #[serde(default = "default_model_registry")]
    pub registry_path: String,

    /// Preload models on startup
    #[serde(default = "default_preload_models")]
    pub preload: bool,

    /// Models to preload (empty = all)
    #[serde(default)]
    pub preload_list: Vec<String>,
}

impl ModelsConfig {
    /// Validate models configuration
    pub fn validate(&self) -> Result<()> {
        if self.registry_path.is_empty() {
            return Err(ConfigError::ValidationError(
                "Model registry path cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            registry_path: default_model_registry(),
            preload: default_preload_models(),
            preload_list: vec![],
        }
    }
}

// Default value functions
fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_timeout() -> u64 {
    30
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10 MB
}

fn default_workers() -> usize {
    num_cpus::get()
}

fn default_cache_enabled() -> bool {
    true
}

fn default_cache_max_size() -> usize {
    10000
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

fn default_model_registry() -> String {
    "models/registry.json".to_string()
}

fn default_preload_models() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_server_config_bind_address() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            ..Default::default()
        };
        assert_eq!(config.bind_address(), "0.0.0.0:8080");
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Invalid port (0)
        config.port = 0;
        assert!(config.validate().is_err());

        // Valid port
        config.port = 8080;
        assert!(config.validate().is_ok());

        // Port 65535 is valid (max u16)
        config.port = 65535;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_size, 10000);
        assert_eq!(config.ttl_secs, 300);
    }

    #[test]
    fn test_cache_config_validation() {
        let mut config = CacheConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Invalid: enabled but max_size is 0
        config.max_size = 0;
        assert!(config.validate().is_err());

        // Valid: disabled with max_size 0
        config.enabled = false;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_models_config_defaults() {
        let config = ModelsConfig::default();
        assert_eq!(config.registry_path, "models/registry.json");
        assert!(config.preload);
        assert!(config.preload_list.is_empty());
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }
}
