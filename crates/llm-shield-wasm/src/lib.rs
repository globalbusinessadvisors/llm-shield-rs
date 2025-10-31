//! WebAssembly bindings for LLM Shield
//!
//! This module provides JavaScript/TypeScript-friendly bindings for the LLM Shield library,
//! including ML infrastructure components (ModelRegistry, ResultCache, ModelLoader).
//!
//! ## Features
//!
//! - **ModelRegistry**: Manage model metadata and downloads
//! - **ResultCache**: Cache scan results with LRU eviction
//! - **ModelLoader**: Load and manage ONNX models
//! - **Type Safety**: Full type conversion between Rust and JavaScript
//! - **Async Support**: Proper async/await for downloads and inference
//!
//! ## Example Usage (JavaScript)
//!
//! ```javascript
//! import init, { ModelRegistryWasm, ResultCacheWasm, CacheConfig } from './pkg';
//!
//! await init();
//!
//! // Create a model registry
//! const registry = ModelRegistryWasm.from_file('models/registry.json');
//!
//! // Create a result cache
//! const cacheConfig = new CacheConfig(1000, 3600);
//! const cache = new ResultCacheWasm(cacheConfig);
//!
//! // Use the cache
//! cache.insert("key1", scanResult);
//! const cached = cache.get("key1");
//! ```

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use llm_shield_core::{ScanResult, Error};
use llm_shield_models::{
    ModelRegistry, ModelTask, ModelVariant, ModelLoader, ModelType,
    ResultCache, CacheConfig as RustCacheConfig, CacheStats,
};

// ============================================================================
// Panic Hook Setup
// ============================================================================

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// Error Handling
// ============================================================================

/// Convert Rust Error to JsValue for WASM
fn to_js_error(err: Error) -> JsValue {
    JsValue::from_str(&format!("Error: {}", err))
}

// ============================================================================
// Type Conversions and Wrappers
// ============================================================================

/// Cache configuration for JavaScript/TypeScript
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of cached entries
    pub max_size: usize,
    /// Time-to-live in seconds
    pub ttl_seconds: u64,
}

#[wasm_bindgen]
impl CacheConfig {
    /// Create a new cache configuration
    #[wasm_bindgen(constructor)]
    pub fn new(max_size: usize, ttl_seconds: u64) -> Self {
        Self {
            max_size,
            ttl_seconds,
        }
    }

    /// Create default configuration (1000 entries, 1 hour TTL)
    pub fn default() -> Self {
        Self {
            max_size: 1000,
            ttl_seconds: 3600,
        }
    }

    /// Create production configuration
    pub fn production() -> Self {
        Self {
            max_size: 1000,
            ttl_seconds: 3600,
        }
    }

    /// Create edge/mobile configuration (smaller cache)
    pub fn edge() -> Self {
        Self {
            max_size: 100,
            ttl_seconds: 600,
        }
    }

    /// Create aggressive caching configuration
    pub fn aggressive() -> Self {
        Self {
            max_size: 10000,
            ttl_seconds: 7200,
        }
    }
}

impl From<CacheConfig> for RustCacheConfig {
    fn from(config: CacheConfig) -> Self {
        Self {
            max_size: config.max_size,
            ttl: std::time::Duration::from_secs(config.ttl_seconds),
        }
    }
}

/// Model task types exposed to JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTaskWasm {
    PromptInjection,
    Toxicity,
    Sentiment,
}

impl From<ModelTaskWasm> for ModelTask {
    fn from(task: ModelTaskWasm) -> Self {
        match task {
            ModelTaskWasm::PromptInjection => ModelTask::PromptInjection,
            ModelTaskWasm::Toxicity => ModelTask::Toxicity,
            ModelTaskWasm::Sentiment => ModelTask::Sentiment,
        }
    }
}

impl From<ModelTask> for ModelTaskWasm {
    fn from(task: ModelTask) -> Self {
        match task {
            ModelTask::PromptInjection => ModelTaskWasm::PromptInjection,
            ModelTask::Toxicity => ModelTaskWasm::Toxicity,
            ModelTask::Sentiment => ModelTaskWasm::Sentiment,
        }
    }
}

/// Model variant types exposed to JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelVariantWasm {
    FP16,
    FP32,
    INT8,
}

impl From<ModelVariantWasm> for ModelVariant {
    fn from(variant: ModelVariantWasm) -> Self {
        match variant {
            ModelVariantWasm::FP16 => ModelVariant::FP16,
            ModelVariantWasm::FP32 => ModelVariant::FP32,
            ModelVariantWasm::INT8 => ModelVariant::INT8,
        }
    }
}

impl From<ModelVariant> for ModelVariantWasm {
    fn from(variant: ModelVariant) -> Self {
        match variant {
            ModelVariant::FP16 => ModelVariantWasm::FP16,
            ModelVariant::FP32 => ModelVariantWasm::FP32,
            ModelVariant::INT8 => ModelVariantWasm::INT8,
        }
    }
}

/// Cache statistics exposed to JavaScript
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStatsWasm {
    pub hits: u64,
    pub misses: u64,
}

#[wasm_bindgen]
impl CacheStatsWasm {
    /// Get total requests
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// Get hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get as JSON string
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl From<CacheStats> for CacheStatsWasm {
    fn from(stats: CacheStats) -> Self {
        Self {
            hits: stats.hits,
            misses: stats.misses,
        }
    }
}

// ============================================================================
// ModelRegistry WASM Bindings
// ============================================================================

/// WebAssembly wrapper for ModelRegistry
///
/// Manages model metadata, downloads, and caching.
#[wasm_bindgen]
pub struct ModelRegistryWasm {
    inner: Arc<ModelRegistry>,
}

#[wasm_bindgen]
impl ModelRegistryWasm {
    /// Create a new empty registry
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ModelRegistry::new()),
        }
    }

    /// Create a registry from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to registry.json file
    ///
    /// # Example
    ///
    /// ```javascript
    /// const registry = ModelRegistryWasm.from_file('models/registry.json');
    /// ```
    pub fn from_file(path: &str) -> Result<ModelRegistryWasm, JsValue> {
        let registry = ModelRegistry::from_file(path).map_err(to_js_error)?;
        Ok(Self {
            inner: Arc::new(registry),
        })
    }

    /// Check if a model is available in the registry
    ///
    /// # Arguments
    ///
    /// * `task` - The model task
    /// * `variant` - The model variant
    ///
    /// # Example
    ///
    /// ```javascript
    /// const hasModel = registry.has_model(ModelTaskWasm.PromptInjection, ModelVariantWasm.FP16);
    /// ```
    pub fn has_model(&self, task: ModelTaskWasm, variant: ModelVariantWasm) -> bool {
        self.inner.has_model(task.into(), variant.into())
    }

    /// Get the total number of registered models
    pub fn model_count(&self) -> usize {
        self.inner.model_count()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get model metadata as JSON
    ///
    /// Returns a JSON string containing model metadata for the specified task and variant.
    pub fn get_model_metadata_json(
        &self,
        task: ModelTaskWasm,
        variant: ModelVariantWasm,
    ) -> Result<String, JsValue> {
        let metadata = self
            .inner
            .get_model_metadata(task.into(), variant.into())
            .map_err(to_js_error)?;
        serde_json::to_string(&metadata).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// List all models as JSON
    ///
    /// Returns a JSON array of all model metadata in the registry.
    pub fn list_models_json(&self) -> Result<String, JsValue> {
        let models = self.inner.list_models();
        serde_json::to_string(&models).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Ensure a model is available locally (download if needed)
    ///
    /// This is an async operation that downloads the model if not already cached.
    ///
    /// # Arguments
    ///
    /// * `task` - The model task
    /// * `variant` - The model variant
    ///
    /// # Returns
    ///
    /// Path to the local model file
    ///
    /// # Example
    ///
    /// ```javascript
    /// const modelPath = await registry.ensure_model_available(
    ///     ModelTaskWasm.PromptInjection,
    ///     ModelVariantWasm.FP16
    /// );
    /// ```
    pub async fn ensure_model_available(
        &self,
        task: ModelTaskWasm,
        variant: ModelVariantWasm,
    ) -> Result<String, JsValue> {
        let path = self
            .inner
            .ensure_model_available(task.into(), variant.into())
            .await
            .map_err(to_js_error)?;
        Ok(path.to_string_lossy().to_string())
    }
}

// ============================================================================
// ResultCache WASM Bindings
// ============================================================================

/// WebAssembly wrapper for ResultCache
///
/// Provides LRU caching of scan results with TTL support.
#[wasm_bindgen]
pub struct ResultCacheWasm {
    inner: ResultCache,
}

#[wasm_bindgen]
impl ResultCacheWasm {
    /// Create a new result cache with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Cache configuration (max_size, ttl)
    ///
    /// # Example
    ///
    /// ```javascript
    /// const config = new CacheConfig(1000, 3600);
    /// const cache = new ResultCacheWasm(config);
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: ResultCache::new(config.into()),
        }
    }

    /// Create a cache with default configuration
    pub fn default() -> Self {
        Self {
            inner: ResultCache::new(RustCacheConfig::default()),
        }
    }

    /// Insert a scan result into the cache
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    /// * `result` - Scan result to cache (as JSON string)
    ///
    /// # Example
    ///
    /// ```javascript
    /// cache.insert("key1", JSON.stringify(scanResult));
    /// ```
    pub fn insert(&self, key: String, result_json: &str) -> Result<(), JsValue> {
        let result: ScanResult = serde_json::from_str(result_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse result: {}", e)))?;
        self.inner.insert(key, result);
        Ok(())
    }

    /// Get a cached scan result
    ///
    /// Returns None if the key doesn't exist or entry has expired.
    ///
    /// # Arguments
    ///
    /// * `key` - Cache key
    ///
    /// # Returns
    ///
    /// JSON string of the cached ScanResult, or null if not found
    ///
    /// # Example
    ///
    /// ```javascript
    /// const resultJson = cache.get("key1");
    /// if (resultJson) {
    ///     const result = JSON.parse(resultJson);
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).and_then(|result| {
            serde_json::to_string(&result).ok()
        })
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get cache statistics
    ///
    /// # Example
    ///
    /// ```javascript
    /// const stats = cache.stats();
    /// console.log(`Hit rate: ${stats.hit_rate() * 100}%`);
    /// ```
    pub fn stats(&self) -> CacheStatsWasm {
        self.inner.stats().into()
    }

    /// Reset cache statistics
    pub fn reset_stats(&self) {
        self.inner.reset_stats();
    }

    /// Generate a deterministic hash key from input text
    ///
    /// Useful for caching scan results based on input content.
    ///
    /// # Example
    ///
    /// ```javascript
    /// const key = ResultCacheWasm.hash_key("some input text");
    /// ```
    pub fn hash_key(input: &str) -> String {
        ResultCache::hash_key(input)
    }
}

// ============================================================================
// ModelLoader WASM Bindings
// ============================================================================

/// WebAssembly wrapper for ModelLoader
///
/// Note: Full ONNX Runtime support in WASM is limited. This provides the API
/// structure, but actual model loading may require ONNX.js or TensorFlow.js
/// in the browser environment.
#[wasm_bindgen]
pub struct ModelLoaderWasm {
    inner: ModelLoader,
}

#[wasm_bindgen]
impl ModelLoaderWasm {
    /// Create a new model loader with a registry
    ///
    /// # Arguments
    ///
    /// * `registry` - Model registry for metadata
    ///
    /// # Example
    ///
    /// ```javascript
    /// const registry = ModelRegistryWasm.from_file('models/registry.json');
    /// const loader = new ModelLoaderWasm(registry);
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(registry: &ModelRegistryWasm) -> Self {
        Self {
            inner: ModelLoader::new(Arc::clone(&registry.inner)),
        }
    }

    /// Check if a model is loaded
    pub fn is_loaded(&self, task: ModelTaskWasm, variant: ModelVariantWasm) -> bool {
        self.inner.is_loaded(
            ModelType::from(ModelTask::from(task)),
            variant.into(),
        )
    }

    /// Get the number of loaded models
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if no models are loaded
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Unload a specific model
    pub fn unload(&self, task: ModelTaskWasm, variant: ModelVariantWasm) {
        self.inner.unload(
            ModelType::from(ModelTask::from(task)),
            variant.into(),
        );
    }

    /// Unload all models
    pub fn unload_all(&self) {
        self.inner.unload_all();
    }

    /// Get loader statistics as JSON
    pub fn stats_json(&self) -> Result<String, JsValue> {
        let stats = self.inner.stats();
        serde_json::to_string(&stats).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// ============================================================================
// ML Configuration WASM Bindings
// ============================================================================

/// ML configuration for JavaScript/TypeScript
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfigWasm {
    pub enabled: bool,
    pub threshold: f32,
    pub fallback_to_heuristic: bool,
    pub cache_enabled: bool,
}

#[wasm_bindgen]
impl MLConfigWasm {
    /// Create a new ML configuration
    #[wasm_bindgen(constructor)]
    pub fn new(
        enabled: bool,
        _variant: ModelVariantWasm,
        threshold: f32,
        fallback_to_heuristic: bool,
        cache_enabled: bool,
    ) -> Self {
        Self {
            enabled,
            threshold,
            fallback_to_heuristic,
            cache_enabled,
        }
    }

    /// Create default configuration (ML disabled)
    pub fn default() -> Self {
        Self {
            enabled: false,
            threshold: 0.5,
            fallback_to_heuristic: true,
            cache_enabled: true,
        }
    }

    /// Create production configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            threshold: 0.5,
            fallback_to_heuristic: true,
            cache_enabled: true,
        }
    }

    /// Create edge/mobile configuration
    pub fn edge() -> Self {
        Self {
            enabled: true,
            threshold: 0.6,
            fallback_to_heuristic: true,
            cache_enabled: true,
        }
    }

    /// Create high accuracy configuration
    pub fn high_accuracy() -> Self {
        Self {
            enabled: true,
            threshold: 0.3,
            fallback_to_heuristic: false,
            cache_enabled: true,
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<MLConfigWasm, JsValue> {
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Initialize the WASM module
///
/// Call this once before using any other functions.
#[wasm_bindgen]
pub fn initialize() {
    init_panic_hook();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_creation() {
        let config = CacheConfig::new(1000, 3600);
        assert_eq!(config.max_size, 1000);
        assert_eq!(config.ttl_seconds, 3600);
    }

    #[test]
    fn test_model_task_conversion() {
        let task = ModelTaskWasm::PromptInjection;
        let rust_task: ModelTask = task.into();
        assert!(matches!(rust_task, ModelTask::PromptInjection));
    }

    #[test]
    fn test_model_variant_conversion() {
        let variant = ModelVariantWasm::FP16;
        let rust_variant: ModelVariant = variant.into();
        assert!(matches!(rust_variant, ModelVariant::FP16));
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let stats = CacheStatsWasm {
            hits: 7,
            misses: 3,
        };
        assert_eq!(stats.total_requests(), 10);
        assert!((stats.hit_rate() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_result_cache_basic_operations() {
        let cache = ResultCacheWasm::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }
}
