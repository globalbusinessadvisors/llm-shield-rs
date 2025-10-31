//! Common Types for ML Model Integration
//!
//! ## SPARC Phase 1: Specification
//!
//! This module defines common types for ML model configuration and hybrid
//! detection mode that combines heuristic and ML approaches.
//!
//! ## Design Principles
//!
//! - **Flexibility**: Support multiple model variants (FP32, FP16, INT8)
//! - **Graceful Degradation**: Fallback to heuristics if ML fails
//! - **Performance**: Cache-aware configuration
//! - **Observability**: Rich metadata for monitoring

use crate::registry::ModelVariant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// ML detection configuration for scanners
///
/// ## Fields
///
/// - `enabled`: Whether ML detection is enabled
/// - `model_variant`: Model precision (FP32, FP16, INT8)
/// - `threshold`: Detection threshold (0.0 to 1.0)
/// - `fallback_to_heuristic`: Use heuristic if ML fails
/// - `cache_enabled`: Enable result caching
/// - `cache_config`: Cache configuration
///
/// ## Recommended Configurations
///
/// **Production (balanced)**:
/// ```rust,ignore
/// MLConfig {
///     enabled: true,
///     model_variant: ModelVariant::FP16,
///     threshold: 0.5,
///     fallback_to_heuristic: true,
///     cache_enabled: true,
///     cache_config: CacheSettings::default(),
/// }
/// ```
///
/// **High accuracy**:
/// ```rust,ignore
/// MLConfig {
///     enabled: true,
///     model_variant: ModelVariant::FP32,
///     threshold: 0.3,  // Lower threshold = more sensitive
///     fallback_to_heuristic: false,  // Require ML
///     cache_enabled: true,
///     cache_config: CacheSettings::default(),
/// }
/// ```
///
/// **Edge/Mobile**:
/// ```rust,ignore
/// MLConfig {
///     enabled: true,
///     model_variant: ModelVariant::INT8,
///     threshold: 0.6,
///     fallback_to_heuristic: true,
///     cache_enabled: true,
///     cache_config: CacheSettings {
///         max_size: 100,  // Smaller cache
///         ttl: Duration::from_secs(600),
///     },
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    /// Whether ML detection is enabled
    pub enabled: bool,

    /// Model variant to use (FP32, FP16, INT8)
    pub model_variant: ModelVariant,

    /// Detection threshold (0.0 to 1.0)
    /// - Higher = fewer false positives, more false negatives
    /// - Lower = fewer false negatives, more false positives
    /// - Recommended: 0.5 for balanced results
    pub threshold: f32,

    /// Use heuristic detection if ML fails or is unavailable
    pub fallback_to_heuristic: bool,

    /// Enable result caching for repeated inputs
    pub cache_enabled: bool,

    /// Cache settings
    pub cache_config: CacheSettings,

    /// Additional model-specific configuration
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Opt-in for ML
            model_variant: ModelVariant::FP16,
            threshold: 0.5,
            fallback_to_heuristic: true,
            cache_enabled: true,
            cache_config: CacheSettings::default(),
            extra: HashMap::new(),
        }
    }
}

impl MLConfig {
    /// Create a new ML configuration
    pub fn new(model_variant: ModelVariant, threshold: f32) -> Self {
        Self {
            enabled: true,
            model_variant,
            threshold,
            ..Default::default()
        }
    }

    /// Create ML configuration for production use
    ///
    /// - FP16 model (balanced speed/accuracy)
    /// - 0.5 threshold (balanced sensitivity)
    /// - Heuristic fallback enabled
    /// - Caching enabled with 1000 entries, 1 hour TTL
    pub fn production() -> Self {
        Self {
            enabled: true,
            model_variant: ModelVariant::FP16,
            threshold: 0.5,
            fallback_to_heuristic: true,
            cache_enabled: true,
            cache_config: CacheSettings::production(),
            extra: HashMap::new(),
        }
    }

    /// Create ML configuration for edge/mobile deployment
    ///
    /// - INT8 model (smallest size)
    /// - 0.6 threshold (fewer false positives)
    /// - Heuristic fallback enabled
    /// - Smaller cache (100 entries, 10 minutes TTL)
    pub fn edge() -> Self {
        Self {
            enabled: true,
            model_variant: ModelVariant::INT8,
            threshold: 0.6,
            fallback_to_heuristic: true,
            cache_enabled: true,
            cache_config: CacheSettings::edge(),
            extra: HashMap::new(),
        }
    }

    /// Create ML configuration for high accuracy
    ///
    /// - FP32 model (highest accuracy)
    /// - 0.3 threshold (very sensitive)
    /// - No heuristic fallback
    /// - Aggressive caching
    pub fn high_accuracy() -> Self {
        Self {
            enabled: true,
            model_variant: ModelVariant::FP32,
            threshold: 0.3,
            fallback_to_heuristic: false,
            cache_enabled: true,
            cache_config: CacheSettings::aggressive(),
            extra: HashMap::new(),
        }
    }

    /// Disable ML detection (heuristic-only mode)
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.threshold) {
            return Err(format!(
                "Threshold must be between 0.0 and 1.0, got {}",
                self.threshold
            ));
        }
        Ok(())
    }
}

/// Cache settings for ML result caching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Maximum number of cached entries (LRU eviction)
    pub max_size: usize,

    /// Time-to-live for cache entries
    #[serde(with = "duration_serde")]
    pub ttl: Duration,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            max_size: 1000,
            ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

impl CacheSettings {
    /// Production cache settings
    /// - 1000 entries
    /// - 1 hour TTL
    pub fn production() -> Self {
        Self {
            max_size: 1000,
            ttl: Duration::from_secs(3600),
        }
    }

    /// Edge/mobile cache settings (smaller)
    /// - 100 entries
    /// - 10 minutes TTL
    pub fn edge() -> Self {
        Self {
            max_size: 100,
            ttl: Duration::from_secs(600),
        }
    }

    /// Aggressive caching for high-traffic scenarios
    /// - 10000 entries
    /// - 2 hours TTL
    pub fn aggressive() -> Self {
        Self {
            max_size: 10000,
            ttl: Duration::from_secs(7200),
        }
    }

    /// Minimal caching (for testing or memory-constrained environments)
    /// - 10 entries
    /// - 1 minute TTL
    pub fn minimal() -> Self {
        Self {
            max_size: 10,
            ttl: Duration::from_secs(60),
        }
    }

    /// Disable caching
    pub fn disabled() -> Self {
        Self {
            max_size: 0,
            ttl: Duration::from_secs(0),
        }
    }
}

/// Hybrid detection mode
///
/// ## Specification
///
/// Hybrid mode combines fast heuristic detection with accurate ML detection:
///
/// 1. **Heuristic Pre-filter**: Fast pattern-based detection (0.01ms)
///    - If obviously safe → return safe (60-70% of inputs)
///    - If obviously malicious → return malicious (5-10% of inputs)
///    - If ambiguous → proceed to ML (20-30% of inputs)
///
/// 2. **ML Detection**: Accurate but slower (50-150ms)
///    - Only called for ambiguous cases
///    - Results are cached
///    - Falls back to heuristic on error (if enabled)
///
/// ## Performance Impact
///
/// - Pure heuristic: ~15,500 req/sec
/// - Pure ML: ~150 req/sec
/// - Hybrid: ~2,000 req/sec (10x faster than pure ML)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridMode {
    /// Only use heuristic detection (no ML)
    HeuristicOnly,

    /// Only use ML detection (no heuristic pre-filter)
    MLOnly,

    /// Use heuristic pre-filter, then ML for ambiguous cases
    Hybrid,

    /// Use both and combine results (max risk score)
    Both,
}

impl Default for HybridMode {
    fn default() -> Self {
        Self::Hybrid
    }
}

/// Detection method used for a scan result
///
/// Tracks which method(s) were used to generate the result.
/// Useful for monitoring and debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Only heuristic pattern matching was used
    #[serde(rename = "heuristic")]
    Heuristic,

    /// Only ML model inference was used
    #[serde(rename = "ml")]
    ML,

    /// Heuristic pre-filter detected safe/malicious
    #[serde(rename = "heuristic_short_circuit")]
    HeuristicShortCircuit,

    /// ML was attempted but failed, fell back to heuristic
    #[serde(rename = "ml_fallback_to_heuristic")]
    MLFallbackToHeuristic,

    /// Both heuristic and ML were used, results combined
    #[serde(rename = "hybrid_both")]
    HybridBoth,
}

/// Inference performance metrics
///
/// ## Specification
///
/// These metrics should be collected and reported for monitoring:
/// - Latency (p50, p95, p99)
/// - Throughput
/// - Cache hit rate
/// - Heuristic filter rate
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InferenceMetrics {
    /// Total inference calls
    pub total_calls: u64,

    /// ML inference calls (not cached)
    pub ml_calls: u64,

    /// Heuristic pre-filter calls
    pub heuristic_calls: u64,

    /// Cache hits
    pub cache_hits: u64,

    /// Heuristic short-circuits (didn't need ML)
    pub heuristic_short_circuits: u64,

    /// Total inference time (milliseconds)
    pub total_inference_time_ms: u64,

    /// ML inference errors
    pub ml_errors: u64,

    /// Fallback to heuristic count
    pub fallback_count: u64,
}

impl InferenceMetrics {
    /// Calculate cache hit rate (0.0 to 1.0)
    pub fn cache_hit_rate(&self) -> f32 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.cache_hits as f32 / self.total_calls as f32
        }
    }

    /// Calculate heuristic filter rate (% of inputs filtered by heuristic)
    pub fn heuristic_filter_rate(&self) -> f32 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.heuristic_short_circuits as f32 / self.total_calls as f32
        }
    }

    /// Calculate average inference time (milliseconds)
    pub fn avg_inference_time_ms(&self) -> f32 {
        if self.total_calls == 0 {
            0.0
        } else {
            self.total_inference_time_ms as f32 / self.total_calls as f32
        }
    }

    /// Calculate ML error rate
    pub fn ml_error_rate(&self) -> f32 {
        if self.ml_calls == 0 {
            0.0
        } else {
            self.ml_errors as f32 / self.ml_calls as f32
        }
    }
}

/// Serialization helper for Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_config_default() {
        let config = MLConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.model_variant, ModelVariant::FP16);
        assert_eq!(config.threshold, 0.5);
        assert!(config.fallback_to_heuristic);
        assert!(config.cache_enabled);
    }

    #[test]
    fn test_ml_config_production() {
        let config = MLConfig::production();
        assert!(config.enabled);
        assert_eq!(config.model_variant, ModelVariant::FP16);
        assert_eq!(config.threshold, 0.5);
        assert!(config.fallback_to_heuristic);
        assert!(config.cache_enabled);
    }

    #[test]
    fn test_ml_config_edge() {
        let config = MLConfig::edge();
        assert!(config.enabled);
        assert_eq!(config.model_variant, ModelVariant::INT8);
        assert_eq!(config.threshold, 0.6);
        assert_eq!(config.cache_config.max_size, 100);
    }

    #[test]
    fn test_ml_config_high_accuracy() {
        let config = MLConfig::high_accuracy();
        assert!(config.enabled);
        assert_eq!(config.model_variant, ModelVariant::FP32);
        assert_eq!(config.threshold, 0.3);
        assert!(!config.fallback_to_heuristic);
    }

    #[test]
    fn test_ml_config_disabled() {
        let config = MLConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_ml_config_validation() {
        let mut config = MLConfig::default();
        assert!(config.validate().is_ok());

        config.threshold = 1.5;
        assert!(config.validate().is_err());

        config.threshold = -0.1;
        assert!(config.validate().is_err());

        config.threshold = 0.0;
        assert!(config.validate().is_ok());

        config.threshold = 1.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cache_settings_default() {
        let settings = CacheSettings::default();
        assert_eq!(settings.max_size, 1000);
        assert_eq!(settings.ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_cache_settings_production() {
        let settings = CacheSettings::production();
        assert_eq!(settings.max_size, 1000);
        assert_eq!(settings.ttl, Duration::from_secs(3600));
    }

    #[test]
    fn test_cache_settings_edge() {
        let settings = CacheSettings::edge();
        assert_eq!(settings.max_size, 100);
        assert_eq!(settings.ttl, Duration::from_secs(600));
    }

    #[test]
    fn test_cache_settings_aggressive() {
        let settings = CacheSettings::aggressive();
        assert_eq!(settings.max_size, 10000);
        assert_eq!(settings.ttl, Duration::from_secs(7200));
    }

    #[test]
    fn test_cache_settings_minimal() {
        let settings = CacheSettings::minimal();
        assert_eq!(settings.max_size, 10);
        assert_eq!(settings.ttl, Duration::from_secs(60));
    }

    #[test]
    fn test_cache_settings_disabled() {
        let settings = CacheSettings::disabled();
        assert_eq!(settings.max_size, 0);
        assert_eq!(settings.ttl, Duration::from_secs(0));
    }

    #[test]
    fn test_hybrid_mode_default() {
        assert_eq!(HybridMode::default(), HybridMode::Hybrid);
    }

    #[test]
    fn test_inference_metrics_default() {
        let metrics = InferenceMetrics::default();
        assert_eq!(metrics.total_calls, 0);
        assert_eq!(metrics.cache_hit_rate(), 0.0);
        assert_eq!(metrics.heuristic_filter_rate(), 0.0);
        assert_eq!(metrics.avg_inference_time_ms(), 0.0);
        assert_eq!(metrics.ml_error_rate(), 0.0);
    }

    #[test]
    fn test_inference_metrics_calculations() {
        let metrics = InferenceMetrics {
            total_calls: 100,
            ml_calls: 40,
            heuristic_calls: 100,
            cache_hits: 30,
            heuristic_short_circuits: 60,
            total_inference_time_ms: 5000,
            ml_errors: 4,
            fallback_count: 4,
        };

        assert_eq!(metrics.cache_hit_rate(), 0.3);
        assert_eq!(metrics.heuristic_filter_rate(), 0.6);
        assert_eq!(metrics.avg_inference_time_ms(), 50.0);
        assert_eq!(metrics.ml_error_rate(), 0.1);
    }

    #[test]
    fn test_ml_config_serialization() {
        let config = MLConfig::production();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: MLConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.threshold, deserialized.threshold);
        assert_eq!(config.cache_config.max_size, deserialized.cache_config.max_size);
    }

    #[test]
    fn test_detection_method_serialization() {
        let method = DetectionMethod::ML;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"ml\"");

        let deserialized: DetectionMethod = serde_json::from_str(&json).unwrap();
        assert_eq!(method, deserialized);
    }

    #[test]
    fn test_hybrid_mode_serialization() {
        let mode = HybridMode::Hybrid;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"hybrid\"");

        let deserialized: HybridMode = serde_json::from_str(&json).unwrap();
        assert_eq!(mode, deserialized);
    }
}
