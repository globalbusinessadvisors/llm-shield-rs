//! Model Registry for LLM Shield
//!
//! Manages model metadata, downloads, caching, and verification.
//!
//! ## Features
//!
//! - Model catalog management
//! - Automatic downloading with caching
//! - Checksum verification
//! - Support for multiple model tasks and variants
//!
//! ## Example
//!
//! ```no_run
//! use llm_shield_models::{ModelRegistry, ModelTask, ModelVariant};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = ModelRegistry::from_file("models/registry.json")?;
//! let model_path = registry.ensure_model_available(
//!     ModelTask::PromptInjection,
//!     ModelVariant::FP16
//! ).await?;
//! # Ok(())
//! # }
//! ```

use llm_shield_core::Error;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

/// Model task type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTask {
    /// Prompt injection detection
    PromptInjection,
    /// Toxicity classification
    Toxicity,
    /// Sentiment analysis
    Sentiment,
}

/// Model variant (precision/quantization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelVariant {
    /// 16-bit floating point
    FP16,
    /// 32-bit floating point
    FP32,
    /// 8-bit integer quantization
    INT8,
}

/// Model metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Unique model identifier
    pub id: String,
    /// Task this model performs
    pub task: ModelTask,
    /// Model variant (precision)
    pub variant: ModelVariant,
    /// Download URL
    pub url: String,
    /// SHA-256 checksum
    pub checksum: String,
    /// Model size in bytes
    pub size_bytes: usize,
}

/// Registry data structure (for deserialization)
#[derive(Debug, Serialize, Deserialize)]
struct RegistryData {
    /// Cache directory path
    cache_dir: Option<String>,
    /// List of available models
    models: Vec<ModelMetadata>,
}

/// Model registry for managing model lifecycle
///
/// ## Thread Safety
///
/// ModelRegistry uses Arc internally for efficient cloning and sharing
/// across threads. The internal HashMap is immutable after creation,
/// making concurrent reads safe without locks.
#[derive(Debug, Clone)]
pub struct ModelRegistry {
    /// Model metadata by key (task/variant) - immutable after creation
    models: Arc<HashMap<String, ModelMetadata>>,
    /// Local cache directory
    cache_dir: Arc<PathBuf>,
}

impl ModelRegistry {
    /// Create a new registry with default cache directory
    pub fn new() -> Self {
        let cache_dir = Self::default_cache_dir();

        Self {
            models: Arc::new(HashMap::new()),
            cache_dir: Arc::new(cache_dir),
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
    /// ```no_run
    /// # use llm_shield_models::ModelRegistry;
    /// let registry = ModelRegistry::from_file("models/registry.json")?;
    /// # Ok::<(), llm_shield_core::Error>(())
    /// ```
    pub fn from_file(path: &str) -> Result<Self> {
        tracing::info!("Loading model registry from: {}", path);

        let json = std::fs::read_to_string(path).map_err(|e| {
            Error::model(format!("Failed to read registry file '{}': {}", path, e))
        })?;

        let data: RegistryData = serde_json::from_str(&json).map_err(|e| {
            Error::model(format!("Failed to parse registry JSON: {}", e))
        })?;

        let mut models = HashMap::new();
        for model in data.models {
            let key = Self::model_key(&model.task, &model.variant);
            tracing::debug!(
                "Registered model: {} ({:?}/{:?})",
                model.id,
                model.task,
                model.variant
            );
            models.insert(key, model);
        }

        let cache_dir = if let Some(dir) = data.cache_dir {
            PathBuf::from(shellexpand::tilde(&dir).to_string())
        } else {
            Self::default_cache_dir()
        };

        tracing::info!(
            "Registry loaded with {} models, cache_dir: {}",
            models.len(),
            cache_dir.display()
        );

        Ok(Self {
            models: Arc::new(models),
            cache_dir: Arc::new(cache_dir)
        })
    }

    /// Get metadata for a specific model
    ///
    /// # Arguments
    ///
    /// * `task` - The model task
    /// * `variant` - The model variant
    ///
    /// # Returns
    ///
    /// Reference to model metadata, or Error if not found
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llm_shield_models::{ModelRegistry, ModelTask, ModelVariant};
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// let metadata = registry.get_model_metadata(
    ///     ModelTask::PromptInjection,
    ///     ModelVariant::FP16
    /// )?;
    /// println!("Model ID: {}", metadata.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_model_metadata(
        &self,
        task: ModelTask,
        variant: ModelVariant,
    ) -> Result<&ModelMetadata> {
        let key = Self::model_key(&task, &variant);
        self.models.get(&key).ok_or_else(|| {
            Error::model(format!(
                "Model not found in registry: {:?}/{:?}",
                task, variant
            ))
        })
    }

    /// List all available models in the registry
    ///
    /// # Returns
    ///
    /// Vector of references to all model metadata
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llm_shield_models::ModelRegistry;
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// let all_models = registry.list_models();
    /// for model in all_models {
    ///     println!("Model: {} ({:?}/{:?})", model.id, model.task, model.variant);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_models(&self) -> Vec<&ModelMetadata> {
        self.models.values().collect()
    }

    /// List all models for a specific task
    ///
    /// # Arguments
    ///
    /// * `task` - The model task to filter by
    ///
    /// # Returns
    ///
    /// Vector of references to model metadata matching the task
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llm_shield_models::{ModelRegistry, ModelTask};
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// let models = registry.list_models_for_task(ModelTask::PromptInjection);
    /// println!("Found {} PromptInjection models", models.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_models_for_task(&self, task: ModelTask) -> Vec<&ModelMetadata> {
        self.models
            .values()
            .filter(|m| m.task == task)
            .collect()
    }

    /// Get all available variants for a specific task
    ///
    /// # Arguments
    ///
    /// * `task` - The model task
    ///
    /// # Returns
    ///
    /// Vector of all available model variants for the task
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llm_shield_models::{ModelRegistry, ModelTask};
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// let variants = registry.get_available_variants(ModelTask::PromptInjection);
    /// for variant in variants {
    ///     println!("Available variant: {:?}", variant);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_available_variants(&self, task: ModelTask) -> Vec<ModelVariant> {
        self.models
            .values()
            .filter(|m| m.task == task)
            .map(|m| m.variant)
            .collect()
    }

    /// Check if a specific model is available in the registry
    ///
    /// # Arguments
    ///
    /// * `task` - The model task
    /// * `variant` - The model variant
    ///
    /// # Returns
    ///
    /// `true` if the model is registered, `false` otherwise
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llm_shield_models::{ModelRegistry, ModelTask, ModelVariant};
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// if registry.has_model(ModelTask::PromptInjection, ModelVariant::FP16) {
    ///     println!("Model is available!");
    /// } else {
    ///     println!("Model not found");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_model(&self, task: ModelTask, variant: ModelVariant) -> bool {
        let key = Self::model_key(&task, &variant);
        self.models.contains_key(&key)
    }

    /// Get the total number of registered models
    ///
    /// # Returns
    ///
    /// Count of all models in the registry
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llm_shield_models::ModelRegistry;
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// println!("Registry contains {} models", registry.model_count());
    /// # Ok(())
    /// # }
    /// ```
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// Check if the registry is empty
    ///
    /// # Returns
    ///
    /// `true` if no models are registered, `false` otherwise
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use llm_shield_models::ModelRegistry;
    /// # fn example() -> Result<(), llm_shield_core::Error> {
    /// # let registry = ModelRegistry::new();
    /// if registry.is_empty() {
    ///     println!("No models registered");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    /// Ensure a model is available locally (download if needed)
    ///
    /// This method:
    /// 1. Checks if model is already cached
    /// 2. Verifies checksum if cached
    /// 3. Downloads if not cached or verification fails
    /// 4. Verifies checksum after download
    ///
    /// # Arguments
    ///
    /// * `task` - The model task
    /// * `variant` - The model variant
    ///
    /// # Returns
    ///
    /// Path to the local model file
    pub async fn ensure_model_available(
        &self,
        task: ModelTask,
        variant: ModelVariant,
    ) -> Result<PathBuf> {
        let metadata = self.get_model_metadata(task, variant)?;
        let model_path = self.cache_dir.join(&metadata.id).join("model.onnx");

        // Check if already cached and valid
        if model_path.exists() {
            tracing::debug!("Model found in cache: {:?}", model_path);

            if self.verify_checksum(&model_path, &metadata.checksum)? {
                tracing::debug!("Checksum verified, using cached model");
                return Ok(model_path);
            } else {
                tracing::warn!("Cached model checksum mismatch, re-downloading");
            }
        }

        // Download model
        tracing::info!(
            "Downloading model: {} from {}",
            metadata.id,
            metadata.url
        );
        self.download_model(metadata, &model_path).await?;

        // Verify checksum
        if !self.verify_checksum(&model_path, &metadata.checksum)? {
            // Clean up failed download
            let _ = std::fs::remove_file(&model_path);
            return Err(Error::model(format!(
                "Checksum verification failed for model: {}",
                metadata.id
            )));
        }

        tracing::info!("Model downloaded and verified: {:?}", model_path);
        Ok(model_path)
    }

    /// Download a model from URL to local path
    async fn download_model(&self, metadata: &ModelMetadata, dest: &Path) -> Result<()> {
        // Create parent directory
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::model(format!(
                    "Failed to create cache directory '{}': {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        // Handle file:// URLs for testing
        if metadata.url.starts_with("file://") {
            let src_path = metadata.url.strip_prefix("file://").unwrap();
            std::fs::copy(src_path, dest).map_err(|e| {
                Error::model(format!(
                    "Failed to copy model from '{}' to '{}': {}",
                    src_path,
                    dest.display(),
                    e
                ))
            })?;
            return Ok(());
        }

        // Download using reqwest for HTTP(S) URLs
        let response = reqwest::get(&metadata.url).await.map_err(|e| {
            Error::model(format!(
                "Failed to download model from '{}': {}",
                metadata.url, e
            ))
        })?;

        if !response.status().is_success() {
            return Err(Error::model(format!(
                "HTTP error downloading model: {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await.map_err(|e| {
            Error::model(format!("Failed to read response body: {}", e))
        })?;

        // Write to file
        std::fs::write(dest, bytes).map_err(|e| {
            Error::model(format!("Failed to write model to '{}': {}", dest.display(), e))
        })?;

        Ok(())
    }

    /// Verify SHA-256 checksum of a file
    fn verify_checksum(&self, path: &Path, expected: &str) -> Result<bool> {
        let bytes = std::fs::read(path).map_err(|e| {
            Error::model(format!("Failed to read file '{}' for checksum: {}", path.display(), e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());

        Ok(hash == expected)
    }

    /// Generate a key for model lookup
    fn model_key(task: &ModelTask, variant: &ModelVariant) -> String {
        format!("{:?}/{:?}", task, variant)
    }

    /// Get the default cache directory
    fn default_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from(".cache"))
            .join("llm-shield")
            .join("models")
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_model_key_generation() {
        let key1 = ModelRegistry::model_key(&ModelTask::PromptInjection, &ModelVariant::FP16);
        let key2 = ModelRegistry::model_key(&ModelTask::Toxicity, &ModelVariant::FP32);

        assert_eq!(key1, "PromptInjection/FP16");
        assert_eq!(key2, "Toxicity/FP32");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_default_cache_dir() {
        let cache_dir = ModelRegistry::default_cache_dir();
        assert!(cache_dir.to_string_lossy().contains("llm-shield"));
        assert!(cache_dir.to_string_lossy().contains("models"));
    }

    #[test]
    fn test_registry_creation() {
        let registry = ModelRegistry::new();
        assert_eq!(registry.models.len(), 0);
        assert!(registry.cache_dir.to_string_lossy().contains("llm-shield"));
    }

    #[test]
    fn test_registry_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("registry.json");

        let content = r#"{
            "cache_dir": "/tmp/test-cache",
            "models": [
                {
                    "id": "test-model",
                    "task": "PromptInjection",
                    "variant": "FP16",
                    "url": "https://example.com/model.onnx",
                    "checksum": "abc123",
                    "size_bytes": 1024
                }
            ]
        }"#;

        std::fs::write(&registry_path, content).unwrap();

        let registry = ModelRegistry::from_file(registry_path.to_str().unwrap()).unwrap();
        assert_eq!(registry.models.len(), 1);

        let metadata = registry
            .get_model_metadata(ModelTask::PromptInjection, ModelVariant::FP16)
            .unwrap();
        assert_eq!(metadata.id, "test-model");
        assert_eq!(metadata.url, "https://example.com/model.onnx");
    }

    #[test]
    fn test_get_missing_model() {
        let registry = ModelRegistry::new();
        let result = registry.get_model_metadata(ModelTask::PromptInjection, ModelVariant::FP16);
        assert!(result.is_err());
    }

    #[test]
    fn test_checksum_verification() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = b"Hello, World!";
        std::fs::write(&test_file, content).unwrap();

        // Calculate correct checksum
        let mut hasher = Sha256::new();
        hasher.update(content);
        let correct_checksum = format!("{:x}", hasher.finalize());

        let registry = ModelRegistry::new();

        // Test correct checksum
        assert!(registry
            .verify_checksum(&test_file, &correct_checksum)
            .unwrap());

        // Test incorrect checksum
        assert!(!registry
            .verify_checksum(&test_file, "wrong_checksum")
            .unwrap());
    }

    #[tokio::test]
    async fn test_download_local_file() {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("source.onnx");
        let content = b"fake model data";
        std::fs::write(&src_file, content).unwrap();

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(content);
        let checksum = format!("{:x}", hasher.finalize());

        let metadata = ModelMetadata {
            id: "test".to_string(),
            task: ModelTask::PromptInjection,
            variant: ModelVariant::FP16,
            url: format!("file://{}", src_file.display()),
            checksum,
            size_bytes: content.len(),
        };

        let dest_file = temp_dir.path().join("dest.onnx");
        let registry = ModelRegistry::new();

        registry.download_model(&metadata, &dest_file).await.unwrap();
        assert!(dest_file.exists());

        let downloaded = std::fs::read(&dest_file).unwrap();
        assert_eq!(downloaded, content);
    }

    #[test]
    fn test_model_task_serialization() {
        let task = ModelTask::PromptInjection;
        let json = serde_json::to_string(&task).unwrap();
        let deserialized: ModelTask = serde_json::from_str(&json).unwrap();
        assert_eq!(task, deserialized);
    }

    #[test]
    fn test_model_variant_serialization() {
        let variant = ModelVariant::FP16;
        let json = serde_json::to_string(&variant).unwrap();
        let deserialized: ModelVariant = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, deserialized);
    }
}
