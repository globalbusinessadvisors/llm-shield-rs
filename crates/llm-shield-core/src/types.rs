//! Common types for configuration and metadata

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Scanner configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    /// Scanner name
    pub name: String,

    /// Whether scanner is enabled
    pub enabled: bool,

    /// Scanner-specific parameters
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

impl ScannerConfig {
    /// Create a new scanner configuration
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            params: HashMap::new(),
        }
    }

    /// Add a parameter to the configuration
    pub fn with_param<K: Into<String>, V: Serialize>(mut self, key: K, value: V) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.params.insert(key.into(), json_value);
        }
        self
    }

    /// Get a parameter from the configuration
    pub fn get_param<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.params
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Scanner metadata for discovery and documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerMetadata {
    /// Scanner name
    pub name: String,

    /// Scanner version
    pub version: String,

    /// Human-readable description
    pub description: String,

    /// Scanner category
    pub category: ScannerCategory,

    /// Required parameters
    pub required_params: Vec<String>,

    /// Optional parameters with defaults
    pub optional_params: HashMap<String, serde_json::Value>,

    /// Performance characteristics
    pub performance: PerformanceInfo,
}

/// Scanner categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScannerCategory {
    /// Content filtering (bans, patterns)
    ContentFilter,
    /// Security detection (injection, exploits)
    Security,
    /// Privacy protection (PII, secrets)
    Privacy,
    /// Quality assessment (toxicity, bias)
    Quality,
    /// Semantic analysis (relevance, factuality)
    Semantic,
}

/// Performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInfo {
    /// Typical latency in milliseconds
    pub typical_latency_ms: u64,

    /// Whether scanner requires network access
    pub requires_network: bool,

    /// Whether scanner uses ML models
    pub uses_ml_models: bool,

    /// Whether scanner can run in parallel
    pub parallel_safe: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_config() {
        let config = ScannerConfig::new("test_scanner")
            .with_param("threshold", 0.7)
            .with_param("enabled", true);

        assert_eq!(config.name, "test_scanner");
        assert_eq!(config.get_param::<f64>("threshold"), Some(0.7));
        assert_eq!(config.get_param::<bool>("enabled"), Some(true));
    }

    #[test]
    fn test_scanner_config_serialization() {
        let config = ScannerConfig::new("test")
            .with_param("key", "value");

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ScannerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(
            config.get_param::<String>("key"),
            deserialized.get_param::<String>("key")
        );
    }
}
