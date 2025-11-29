//! # SDK Configuration
//!
//! Configuration types for the LLM Shield SDK.
//!
//! ## Design Principles
//!
//! - Sensible defaults for common use cases
//! - Easy customization through builder pattern
//! - Serializable for configuration files

use serde::{Deserialize, Serialize};

/// Main SDK configuration
///
/// ## Example
///
/// ```rust,ignore
/// let config = ShieldConfig {
///     scan_mode: ScanMode::Both,
///     parallel: ParallelConfig::default(),
///     short_circuit_threshold: Some(0.9),
///     timeout_ms: Some(5000),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldConfig {
    /// Scan mode (prompt only, output only, or both)
    pub scan_mode: ScanMode,

    /// Parallel execution configuration
    pub parallel: ParallelConfig,

    /// Short-circuit threshold (stop scanning if risk exceeds this)
    pub short_circuit_threshold: Option<f32>,

    /// Operation timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Enable tracing/observability
    pub enable_tracing: bool,

    /// Enable caching of scan results
    pub enable_caching: bool,

    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,

    /// Maximum batch size for batch operations
    pub max_batch_size: usize,
}

impl Default for ShieldConfig {
    fn default() -> Self {
        Self {
            scan_mode: ScanMode::Both,
            parallel: ParallelConfig::default(),
            short_circuit_threshold: None,
            timeout_ms: Some(30_000), // 30 seconds default
            enable_tracing: true,
            enable_caching: false,
            cache_ttl_seconds: 300, // 5 minutes
            max_batch_size: 100,
        }
    }
}

impl ShieldConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration for development/testing
    pub fn development() -> Self {
        Self {
            scan_mode: ScanMode::Both,
            parallel: ParallelConfig {
                enabled: false, // Sequential for easier debugging
                max_concurrent: 1,
            },
            short_circuit_threshold: None,
            timeout_ms: Some(60_000), // 1 minute
            enable_tracing: true,
            enable_caching: false,
            cache_ttl_seconds: 0,
            max_batch_size: 10,
        }
    }

    /// Create configuration for production
    pub fn production() -> Self {
        Self {
            scan_mode: ScanMode::Both,
            parallel: ParallelConfig {
                enabled: true,
                max_concurrent: 8,
            },
            short_circuit_threshold: Some(0.95), // Short-circuit on critical risks
            timeout_ms: Some(10_000), // 10 seconds
            enable_tracing: true,
            enable_caching: true,
            cache_ttl_seconds: 300,
            max_batch_size: 100,
        }
    }

    /// Create configuration for high-throughput scenarios
    pub fn high_throughput() -> Self {
        Self {
            scan_mode: ScanMode::Both,
            parallel: ParallelConfig {
                enabled: true,
                max_concurrent: 16,
            },
            short_circuit_threshold: Some(0.9),
            timeout_ms: Some(5_000), // 5 seconds
            enable_tracing: false, // Disable for performance
            enable_caching: true,
            cache_ttl_seconds: 600,
            max_batch_size: 500,
        }
    }
}

/// Scan mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanMode {
    /// Only scan prompts (input)
    Prompt,
    /// Only scan outputs
    Output,
    /// Scan both prompts and outputs
    Both,
}

impl Default for ScanMode {
    fn default() -> Self {
        Self::Both
    }
}

/// Parallel execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Enable parallel scanner execution
    pub enabled: bool,

    /// Maximum concurrent scanners
    pub max_concurrent: usize,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent: 4,
        }
    }
}

impl ParallelConfig {
    /// Create disabled parallel config (sequential execution)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            max_concurrent: 1,
        }
    }

    /// Create parallel config with custom concurrency
    pub fn with_concurrency(max_concurrent: usize) -> Self {
        Self {
            enabled: true,
            max_concurrent: max_concurrent.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ShieldConfig::default();
        assert_eq!(config.scan_mode, ScanMode::Both);
        assert!(config.parallel.enabled);
    }

    #[test]
    fn test_development_config() {
        let config = ShieldConfig::development();
        assert!(!config.parallel.enabled);
        assert!(!config.enable_caching);
    }

    #[test]
    fn test_production_config() {
        let config = ShieldConfig::production();
        assert!(config.parallel.enabled);
        assert!(config.enable_caching);
        assert!(config.short_circuit_threshold.is_some());
    }

    #[test]
    fn test_high_throughput_config() {
        let config = ShieldConfig::high_throughput();
        assert!(config.parallel.enabled);
        assert!(config.parallel.max_concurrent >= 16);
        assert!(!config.enable_tracing);
    }

    #[test]
    fn test_scan_mode() {
        assert_eq!(ScanMode::default(), ScanMode::Both);
    }

    #[test]
    fn test_parallel_config() {
        let disabled = ParallelConfig::disabled();
        assert!(!disabled.enabled);
        assert_eq!(disabled.max_concurrent, 1);

        let concurrent = ParallelConfig::with_concurrency(8);
        assert!(concurrent.enabled);
        assert_eq!(concurrent.max_concurrent, 8);

        // Test minimum concurrency
        let min = ParallelConfig::with_concurrency(0);
        assert_eq!(min.max_concurrent, 1);
    }

    #[test]
    fn test_config_serialization() {
        let config = ShieldConfig::production();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ShieldConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.scan_mode, deserialized.scan_mode);
        assert_eq!(config.parallel.enabled, deserialized.parallel.enabled);
    }
}
