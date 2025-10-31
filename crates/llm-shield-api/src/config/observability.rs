//! Observability configuration (logging, metrics, tracing)

use super::{ConfigError, Result};
use serde::{Deserialize, Serialize};

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Logging configuration
    pub logging: LoggingConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,
}

impl ObservabilityConfig {
    /// Validate observability configuration
    pub fn validate(&self) -> Result<()> {
        self.logging.validate()?;
        self.metrics.validate()?;
        Ok(())
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json or pretty)
    #[serde(default = "default_log_format")]
    pub format: LogFormat,

    /// Enable log sampling
    #[serde(default = "default_sampling_enabled")]
    pub sampling_enabled: bool,

    /// Sampling rate (0.0 - 1.0)
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,
}

impl LoggingConfig {
    /// Validate logging configuration
    pub fn validate(&self) -> Result<()> {
        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.level.to_lowercase().as_str()) {
            return Err(ConfigError::ValidationError(format!(
                "Invalid log level: {}. Must be one of: {}",
                self.level,
                valid_levels.join(", ")
            )));
        }

        // Validate sampling rate
        if self.sampling_enabled && (self.sampling_rate < 0.0 || self.sampling_rate > 1.0) {
            return Err(ConfigError::ValidationError(format!(
                "Sampling rate must be between 0.0 and 1.0, got: {}",
                self.sampling_rate
            )));
        }

        Ok(())
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            sampling_enabled: default_sampling_enabled(),
            sampling_rate: default_sampling_rate(),
        }
    }
}

/// Log format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON format (for production)
    Json,
    /// Pretty format (for development)
    Pretty,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable Prometheus metrics
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,

    /// Metrics endpoint path
    #[serde(default = "default_metrics_path")]
    pub path: String,

    /// Include detailed labels
    #[serde(default = "default_detailed_labels")]
    pub detailed_labels: bool,
}

impl MetricsConfig {
    /// Validate metrics configuration
    pub fn validate(&self) -> Result<()> {
        if self.enabled && self.path.is_empty() {
            return Err(ConfigError::ValidationError(
                "Metrics path cannot be empty when metrics are enabled".to_string(),
            ));
        }

        if self.enabled && !self.path.starts_with('/') {
            return Err(ConfigError::ValidationError(format!(
                "Metrics path must start with '/', got: {}",
                self.path
            )));
        }

        Ok(())
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            path: default_metrics_path(),
            detailed_labels: default_detailed_labels(),
        }
    }
}

// Default value functions
fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> LogFormat {
    LogFormat::Pretty
}

fn default_sampling_enabled() -> bool {
    false
}

fn default_sampling_rate() -> f64 {
    0.1
}

fn default_metrics_enabled() -> bool {
    true
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

fn default_detailed_labels() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Pretty);
        assert!(!config.sampling_enabled);
    }

    #[test]
    fn test_logging_config_validation() {
        let config = LoggingConfig::default();
        assert!(config.validate().is_ok());

        // Invalid log level
        let mut config = LoggingConfig::default();
        config.level = "invalid".to_string();
        assert!(config.validate().is_err());

        // Valid log levels
        for level in ["trace", "debug", "info", "warn", "error"] {
            let mut config = LoggingConfig::default();
            config.level = level.to_string();
            assert!(config.validate().is_ok());
        }

        // Invalid sampling rate
        let mut config = LoggingConfig::default();
        config.sampling_enabled = true;
        config.sampling_rate = 1.5;
        assert!(config.validate().is_err());

        // Valid sampling rate
        config.sampling_rate = 0.5;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_metrics_config_defaults() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.path, "/metrics");
        assert!(!config.detailed_labels);
    }

    #[test]
    fn test_metrics_config_validation() {
        let config = MetricsConfig::default();
        assert!(config.validate().is_ok());

        // Invalid: enabled but empty path
        let mut config = MetricsConfig::default();
        config.path = String::new();
        assert!(config.validate().is_err());

        // Invalid: path doesn't start with /
        config.path = "metrics".to_string();
        assert!(config.validate().is_err());

        // Valid: proper path
        config.path = "/metrics".to_string();
        assert!(config.validate().is_ok());

        // Valid: disabled with empty path
        config.enabled = false;
        config.path = String::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_log_format_serialization() {
        assert_eq!(
            serde_json::to_string(&LogFormat::Json).unwrap(),
            "\"json\""
        );
        assert_eq!(
            serde_json::to_string(&LogFormat::Pretty).unwrap(),
            "\"pretty\""
        );
    }

    #[test]
    fn test_observability_config_default() {
        let config = ObservabilityConfig::default();
        assert!(config.validate().is_ok());
    }
}
