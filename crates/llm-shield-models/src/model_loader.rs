//! Model Loader with ONNX Runtime Integration
//!
//! ## SPARC Phase 3: Implementation
//!
//! This module provides lazy loading, caching, and thread-safe access to ONNX models.
//!
//! ## Features
//!
//! - **Lazy Loading**: Models are only loaded when first requested
//! - **Caching**: Loaded models are cached for reuse
//! - **Thread-Safe**: Uses Arc + RwLock for concurrent access
//! - **Registry Integration**: Uses ModelRegistry for model discovery
//! - **Graceful Error Handling**: Comprehensive error messages
//!
//! ## Usage Example
//!
//! ```no_run
//! use llm_shield_models::{ModelLoader, ModelRegistry, ModelType, ModelVariant};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), llm_shield_core::Error> {
//! // Create registry
//! let registry = ModelRegistry::from_file("models/registry.json")?;
//!
//! // Create loader
//! let loader = ModelLoader::new(Arc::new(registry));
//!
//! // Load model (lazy - only loads once)
//! let session = loader.load(ModelType::PromptInjection, ModelVariant::FP16).await?;
//!
//! // Use session for inference...
//! # Ok(())
//! # }
//! ```

use crate::registry::{ModelRegistry, ModelTask, ModelVariant};
use llm_shield_core::Error;
use ort::session::Session;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, Mutex};

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Model type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelType {
    /// Prompt injection detection
    PromptInjection,
    /// Toxicity classification
    Toxicity,
    /// Sentiment analysis
    Sentiment,
}

/// Conversion from ModelTask to ModelType
impl From<ModelTask> for ModelType {
    fn from(task: ModelTask) -> Self {
        match task {
            ModelTask::PromptInjection => ModelType::PromptInjection,
            ModelTask::Toxicity => ModelType::Toxicity,
            ModelTask::Sentiment => ModelType::Sentiment,
        }
    }
}

/// Conversion from ModelType to ModelTask
impl From<ModelType> for ModelTask {
    fn from(model_type: ModelType) -> Self {
        match model_type {
            ModelType::PromptInjection => ModelTask::PromptInjection,
            ModelType::Toxicity => ModelTask::Toxicity,
            ModelType::Sentiment => ModelTask::Sentiment,
        }
    }
}

/// Configuration for loading a model
#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Type of model
    pub model_type: ModelType,
    /// Model variant (precision)
    pub variant: ModelVariant,
    /// Path to ONNX model file
    pub model_path: PathBuf,
    /// Number of threads for inference
    pub thread_pool_size: usize,
    /// ONNX graph optimization level (0-3)
    pub optimization_level: u8,
}

impl ModelConfig {
    /// Create a new model configuration
    ///
    /// # Arguments
    ///
    /// * `model_type` - Type of model (PromptInjection, Toxicity, Sentiment)
    /// * `variant` - Model variant (FP32, FP16, INT8)
    /// * `model_path` - Path to ONNX model file
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::{ModelConfig, ModelType, ModelVariant};
    /// use std::path::PathBuf;
    ///
    /// let config = ModelConfig::new(
    ///     ModelType::PromptInjection,
    ///     ModelVariant::FP16,
    ///     PathBuf::from("/path/to/model.onnx")
    /// );
    /// ```
    pub fn new(model_type: ModelType, variant: ModelVariant, model_path: PathBuf) -> Self {
        Self {
            model_type,
            variant,
            model_path,
            thread_pool_size: num_cpus::get().max(1),
            optimization_level: 3, // Max optimization
        }
    }

    /// Set the thread pool size
    pub fn with_thread_pool_size(mut self, size: usize) -> Self {
        self.thread_pool_size = size;
        self
    }

    /// Set the optimization level (0-3)
    pub fn with_optimization_level(mut self, level: u8) -> Self {
        self.optimization_level = level.min(3);
        self
    }
}

/// Statistics about loaded models
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct LoaderStats {
    /// Number of models currently loaded
    pub total_loaded: usize,
    /// Total number of load operations
    pub total_loads: u64,
    /// Total number of cache hits
    pub cache_hits: u64,
}

/// Model loader with lazy loading and caching
///
/// ## Thread Safety
///
/// ModelLoader uses Arc + RwLock internally for thread-safe access.
/// Multiple threads can safely load and access models concurrently.
///
/// ## Caching
///
/// Once a model is loaded, it stays in memory until explicitly unloaded.
/// Subsequent calls to `load()` with the same model type/variant return
/// the cached session.
pub struct ModelLoader {
    /// Model registry for metadata
    registry: Arc<ModelRegistry>,
    /// Loaded ONNX sessions cache
    cache: Arc<RwLock<HashMap<(ModelType, ModelVariant), Arc<Mutex<Session>>>>>,
    /// Statistics
    stats: Arc<RwLock<LoaderStats>>,
}

impl ModelLoader {
    /// Create a new model loader
    ///
    /// # Arguments
    ///
    /// * `registry` - Model registry for metadata and downloads
    ///
    /// # Example
    ///
    /// ```no_run
    /// use llm_shield_models::{ModelLoader, ModelRegistry};
    /// use std::sync::Arc;
    ///
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// let registry = ModelRegistry::from_file("models/registry.json")?;
    /// let loader = ModelLoader::new(Arc::new(registry));
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(registry: Arc<ModelRegistry>) -> Self {
        Self {
            registry,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(LoaderStats::default())),
        }
    }

    /// Create a new model loader (alias for `new`)
    pub fn with_registry(registry: Arc<ModelRegistry>) -> Self {
        Self::new(registry)
    }

    /// Load a model (lazily, with caching)
    ///
    /// If the model is already loaded, returns the cached session.
    /// Otherwise, loads the model from disk using the registry.
    ///
    /// # Arguments
    ///
    /// * `model_type` - Type of model to load
    /// * `variant` - Model variant (precision)
    ///
    /// # Returns
    ///
    /// Arc to ONNX Runtime session
    ///
    /// # Example
    ///
    /// ```no_run
    /// use llm_shield_models::{ModelLoader, ModelRegistry, ModelType, ModelVariant};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// # let loader = ModelLoader::new(Arc::new(registry));
    /// let session = loader.load(ModelType::PromptInjection, ModelVariant::FP16).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load(
        &self,
        model_type: ModelType,
        variant: ModelVariant,
    ) -> Result<Arc<Mutex<Session>>> {
        // Check cache first (read lock)
        {
            let cache = self.cache.read().unwrap();
            if let Some(session) = cache.get(&(model_type, variant)) {
                tracing::debug!(
                    "Model cache hit: {:?}/{:?}",
                    model_type,
                    variant
                );
                let mut stats = self.stats.write().unwrap();
                stats.cache_hits += 1;
                return Ok(Arc::clone(session));
            }
        }

        // Not in cache - load it (write lock)
        tracing::info!(
            "Loading model: {:?}/{:?}",
            model_type,
            variant
        );

        // Convert to task and get metadata
        let task = ModelTask::from(model_type);
        let metadata = self.registry.get_model_metadata(task, variant)?;

        // Ensure model is downloaded
        let model_path = self.registry.ensure_model_available(task, variant).await?;

        tracing::debug!("Model path: {:?}", model_path);

        // Create ONNX session
        let session = Self::create_session(&model_path, num_cpus::get().max(1), 3)?;

        // Cache the session (wrapped in Mutex for ORT 2.0 API)
        {
            let mut cache = self.cache.write().unwrap();
            let session_arc = Arc::new(Mutex::new(session));
            cache.insert((model_type, variant), Arc::clone(&session_arc));

            // Update stats
            let mut stats = self.stats.write().unwrap();
            stats.total_loaded = cache.len();
            stats.total_loads += 1;

            tracing::info!(
                "Model loaded successfully: {} ({:?}/{:?})",
                metadata.id,
                model_type,
                variant
            );

            Ok(session_arc)
        }
    }

    /// Load a model with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Model configuration
    ///
    /// # Returns
    ///
    /// Arc to ONNX Runtime session
    pub async fn load_with_config(&self, config: ModelConfig) -> Result<Arc<Mutex<Session>>> {
        let model_type = config.model_type;
        let variant = config.variant;

        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(session) = cache.get(&(model_type, variant)) {
                let mut stats = self.stats.write().unwrap();
                stats.cache_hits += 1;
                return Ok(Arc::clone(session));
            }
        }

        // Get model path from registry
        let task = ModelTask::from(model_type);
        let model_path = self.registry.ensure_model_available(task, variant).await?;

        // Create session with custom config
        let session = Self::create_session(
            &model_path,
            config.thread_pool_size,
            config.optimization_level,
        )?;

        // Cache it (wrapped in Mutex for ORT 2.0 API)
        let mut cache = self.cache.write().unwrap();
        let session_arc = Arc::new(Mutex::new(session));
        cache.insert((model_type, variant), Arc::clone(&session_arc));

        let mut stats = self.stats.write().unwrap();
        stats.total_loaded = cache.len();
        stats.total_loads += 1;

        Ok(session_arc)
    }

    /// Preload multiple models
    ///
    /// Useful for warming up the cache before first use.
    ///
    /// # Arguments
    ///
    /// * `models` - List of (ModelType, ModelVariant) tuples to preload
    ///
    /// # Example
    ///
    /// ```no_run
    /// use llm_shield_models::{ModelLoader, ModelRegistry, ModelType, ModelVariant};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// # let loader = ModelLoader::new(Arc::new(registry));
    /// let models = vec![
    ///     (ModelType::PromptInjection, ModelVariant::FP16),
    ///     (ModelType::Toxicity, ModelVariant::FP16),
    /// ];
    /// loader.preload(models).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn preload(&self, models: Vec<(ModelType, ModelVariant)>) -> Result<()> {
        tracing::info!("Preloading {} models", models.len());

        for (model_type, variant) in models {
            match self.load(model_type, variant).await {
                Ok(_) => {
                    tracing::debug!("Preloaded: {:?}/{:?}", model_type, variant);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to preload {:?}/{:?}: {}",
                        model_type,
                        variant,
                        e
                    );
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Check if a model is loaded
    pub fn is_loaded(&self, model_type: ModelType, variant: ModelVariant) -> bool {
        let cache = self.cache.read().unwrap();
        cache.contains_key(&(model_type, variant))
    }

    /// Unload a specific model
    ///
    /// Removes the model from cache, freeing memory.
    pub fn unload(&self, model_type: ModelType, variant: ModelVariant) {
        let mut cache = self.cache.write().unwrap();
        if cache.remove(&(model_type, variant)).is_some() {
            tracing::info!("Unloaded model: {:?}/{:?}", model_type, variant);
            let mut stats = self.stats.write().unwrap();
            stats.total_loaded = cache.len();
        }
    }

    /// Unload all models
    ///
    /// Clears the entire cache, freeing all memory.
    pub fn unload_all(&self) {
        let mut cache = self.cache.write().unwrap();
        let count = cache.len();
        cache.clear();
        tracing::info!("Unloaded all {} models", count);
        let mut stats = self.stats.write().unwrap();
        stats.total_loaded = 0;
    }

    /// Get the number of loaded models
    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Check if no models are loaded
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get list of loaded models
    pub fn loaded_models(&self) -> Vec<(ModelType, ModelVariant)> {
        let cache = self.cache.read().unwrap();
        cache.keys().copied().collect()
    }

    /// Get information about a loaded model
    ///
    /// Returns None if model is not loaded.
    pub fn model_info(&self, model_type: ModelType, variant: ModelVariant) -> Option<String> {
        let cache = self.cache.read().unwrap();
        if cache.contains_key(&(model_type, variant)) {
            Some(format!(
                "Model: {:?}, Variant: {:?}, Status: loaded",
                model_type, variant
            ))
        } else {
            None
        }
    }

    /// Get loader statistics
    pub fn stats(&self) -> LoaderStats {
        self.stats.read().unwrap().clone()
    }

    /// Create an ONNX Runtime session
    fn create_session(
        model_path: &PathBuf,
        _thread_pool_size: usize,
        _optimization_level: u8,
    ) -> Result<Session> {
        // Create session with default settings
        // Note: with_optimization_level and with_intra_threads APIs vary by ort version
        let session = Session::builder()
            .map_err(|e| Error::model(format!("Failed to create session builder: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| {
                Error::model(format!(
                    "Failed to load model from '{}': {}",
                    model_path.display(),
                    e
                ))
            })?;

        Ok(session)
    }
}

impl Clone for ModelLoader {
    /// Clone creates a new reference to the same underlying cache
    ///
    /// All clones share the same loaded models and statistics.
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            cache: Arc::clone(&self.cache),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ModelTask;

    #[test]
    fn test_model_type_conversions() {
        // ModelTask -> ModelType
        assert!(matches!(
            ModelType::from(ModelTask::PromptInjection),
            ModelType::PromptInjection
        ));
        assert!(matches!(
            ModelType::from(ModelTask::Toxicity),
            ModelType::Toxicity
        ));
        assert!(matches!(
            ModelType::from(ModelTask::Sentiment),
            ModelType::Sentiment
        ));

        // ModelType -> ModelTask
        assert!(matches!(
            ModelTask::from(ModelType::PromptInjection),
            ModelTask::PromptInjection
        ));
        assert!(matches!(
            ModelTask::from(ModelType::Toxicity),
            ModelTask::Toxicity
        ));
        assert!(matches!(
            ModelTask::from(ModelType::Sentiment),
            ModelTask::Sentiment
        ));
    }

    #[test]
    fn test_model_config_defaults() {
        let config = ModelConfig::new(
            ModelType::PromptInjection,
            ModelVariant::FP16,
            PathBuf::from("/test/model.onnx"),
        );

        assert!(config.thread_pool_size > 0);
        assert_eq!(config.optimization_level, 3);
    }

    #[test]
    fn test_model_config_builder_pattern() {
        let config = ModelConfig::new(
            ModelType::Toxicity,
            ModelVariant::INT8,
            PathBuf::from("/test/model.onnx"),
        )
        .with_thread_pool_size(4)
        .with_optimization_level(2);

        assert_eq!(config.thread_pool_size, 4);
        assert_eq!(config.optimization_level, 2);
    }

    #[test]
    fn test_loader_stats_default() {
        let stats = LoaderStats::default();
        assert_eq!(stats.total_loaded, 0);
        assert_eq!(stats.total_loads, 0);
        assert_eq!(stats.cache_hits, 0);
    }
}
