//! Authentication configuration

use super::{ConfigError, Result};
use serde::{Deserialize, Serialize};

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Enable authentication
    #[serde(default = "default_auth_enabled")]
    pub enabled: bool,

    /// API key storage backend
    #[serde(default = "default_storage_backend")]
    pub storage_backend: StorageBackend,

    /// Path to API keys file (for file backend)
    #[serde(default = "default_keys_file")]
    pub keys_file: String,
}

impl AuthConfig {
    /// Validate authentication configuration
    pub fn validate(&self) -> Result<()> {
        if self.enabled {
            match self.storage_backend {
                StorageBackend::File => {
                    if self.keys_file.is_empty() {
                        return Err(ConfigError::ValidationError(
                            "Keys file path cannot be empty when using file backend".to_string(),
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: default_auth_enabled(),
            storage_backend: default_storage_backend(),
            keys_file: default_keys_file(),
        }
    }
}

/// API key storage backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    /// In-memory storage (for testing/development)
    Memory,
    /// File-based storage
    File,
    /// Redis storage (optional feature)
    #[cfg(feature = "redis")]
    Redis,
}

fn default_auth_enabled() -> bool {
    true
}

fn default_storage_backend() -> StorageBackend {
    StorageBackend::Memory
}

fn default_keys_file() -> String {
    "config/api_keys.json".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_defaults() {
        let config = AuthConfig::default();
        assert!(config.enabled);
        assert_eq!(config.storage_backend, StorageBackend::Memory);
    }

    #[test]
    fn test_auth_config_validation() {
        let config = AuthConfig::default();
        assert!(config.validate().is_ok());

        // File backend without keys_file
        let mut config = AuthConfig {
            enabled: true,
            storage_backend: StorageBackend::File,
            keys_file: String::new(),
        };
        assert!(config.validate().is_err());

        // File backend with keys_file
        config.keys_file = "config/keys.json".to_string();
        assert!(config.validate().is_ok());

        // Disabled auth
        config.enabled = false;
        config.keys_file = String::new();
        assert!(config.validate().is_ok());
    }
}
