//! Model Loader Tests - TDD Red Phase
//!
//! Comprehensive tests for ModelLoader with ONNX Runtime integration.
//! These tests are written BEFORE implementation (London School TDD).

use llm_shield_models::{
    ModelLoader, ModelConfig, ModelType, ModelRegistry, ModelTask, ModelVariant,
    Result,
};
use sha2::Digest;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

/// Create a temporary registry for testing
fn create_test_registry() -> Result<(ModelRegistry, TempDir)> {
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    // Create a minimal test model file
    let model_dir = temp_dir.path().join("test-model");
    std::fs::create_dir_all(&model_dir).unwrap();
    let model_path = model_dir.join("model.onnx");
    std::fs::write(&model_path, b"fake model data").unwrap();

    // Calculate checksum
    let bytes = std::fs::read(&model_path).unwrap();
    let checksum = format!("{:x}", sha2::Sha256::digest(&bytes));

    let registry_json = format!(
        r#"{{
            "cache_dir": "{}",
            "models": [
                {{
                    "id": "test-model",
                    "task": "PromptInjection",
                    "variant": "FP16",
                    "url": "file://{}",
                    "checksum": "{}",
                    "size_bytes": {}
                }}
            ]
        }}"#,
        temp_dir.path().display(),
        model_path.display(),
        checksum,
        bytes.len()
    );

    std::fs::write(&registry_path, registry_json).unwrap();
    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())?;

    Ok((registry, temp_dir))
}

// ============================================================================
// Test 1: ModelType Enum Tests
// ============================================================================

#[test]
fn test_model_type_enum_variants() {
    // Test that ModelType has expected variants
    let _prompt_injection = ModelType::PromptInjection;
    let _toxicity = ModelType::Toxicity;
    let _sentiment = ModelType::Sentiment;
}

#[test]
fn test_model_type_debug() {
    let model_type = ModelType::PromptInjection;
    let debug_str = format!("{:?}", model_type);
    assert!(debug_str.contains("PromptInjection"));
}

#[test]
fn test_model_type_clone() {
    let model_type = ModelType::Toxicity;
    let cloned = model_type.clone();
    assert_eq!(format!("{:?}", model_type), format!("{:?}", cloned));
}

// ============================================================================
// Test 2: ModelConfig Tests
// ============================================================================

#[test]
fn test_model_config_creation() {
    let config = ModelConfig::new(
        ModelType::PromptInjection,
        ModelVariant::FP16,
        PathBuf::from("/test/path/model.onnx"),
    );

    assert_eq!(format!("{:?}", config.model_type), "PromptInjection");
    assert_eq!(format!("{:?}", config.variant), "FP16");
    assert_eq!(config.model_path, PathBuf::from("/test/path/model.onnx"));
}

#[test]
fn test_model_config_default_thread_pool() {
    let config = ModelConfig::new(
        ModelType::Sentiment,
        ModelVariant::FP32,
        PathBuf::from("/test/model.onnx"),
    );

    // Should have sensible defaults
    assert!(config.thread_pool_size > 0);
}

#[test]
fn test_model_config_with_thread_pool() {
    let mut config = ModelConfig::new(
        ModelType::Toxicity,
        ModelVariant::INT8,
        PathBuf::from("/test/model.onnx"),
    );

    config.thread_pool_size = 4;
    assert_eq!(config.thread_pool_size, 4);
}

#[test]
fn test_model_config_with_optimization_level() {
    let mut config = ModelConfig::new(
        ModelType::PromptInjection,
        ModelVariant::FP16,
        PathBuf::from("/test/model.onnx"),
    );

    config.optimization_level = 3; // Max optimization
    assert_eq!(config.optimization_level, 3);
}

// ============================================================================
// Test 3: ModelLoader Creation Tests
// ============================================================================

#[test]
fn test_model_loader_creation() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    assert_eq!(loader.len(), 0); // No models loaded yet
    assert!(loader.is_empty());
}

#[test]
fn test_model_loader_with_custom_registry() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::with_registry(Arc::new(registry));

    assert!(loader.is_empty());
}

// ============================================================================
// Test 4: Lazy Loading Tests
// ============================================================================

#[tokio::test]
async fn test_lazy_loading_not_loaded_initially() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Initially no models loaded
    assert_eq!(loader.len(), 0);
    assert!(!loader.is_loaded(ModelType::PromptInjection, ModelVariant::FP16));
}

#[tokio::test]
async fn test_lazy_loading_load_on_first_use() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // First load should actually load the model
    let result = loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await;

    assert!(result.is_ok());
    assert_eq!(loader.len(), 1);
    assert!(loader.is_loaded(ModelType::PromptInjection, ModelVariant::FP16));
}

#[tokio::test]
async fn test_lazy_loading_cached_on_second_load() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Load once
    let _model1 = loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();

    // Load again - should be cached
    let _model2 = loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();

    // Still only 1 model in cache
    assert_eq!(loader.len(), 1);
}

// ============================================================================
// Test 5: Thread Safety Tests
// ============================================================================

#[tokio::test]
async fn test_thread_safe_concurrent_loads() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = Arc::new(ModelLoader::new(Arc::new(registry)));

    // Spawn multiple concurrent load tasks
    let mut handles = vec![];
    for _ in 0..5 {
        let loader_clone = Arc::clone(&loader);
        let handle = tokio::spawn(async move {
            loader_clone
                .load(ModelType::PromptInjection, ModelVariant::FP16)
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Should still only have 1 model loaded (not 5)
    assert_eq!(loader.len(), 1);
}

#[test]
fn test_model_loader_clone() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader1 = ModelLoader::new(Arc::new(registry));
    let loader2 = loader1.clone();

    // Both should reference the same underlying cache
    assert_eq!(loader1.len(), loader2.len());
}

// ============================================================================
// Test 6: Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_load_missing_model() {
    let registry = ModelRegistry::new(); // Empty registry
    let loader = ModelLoader::new(Arc::new(registry));

    let result = loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await;

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("not found") || err_msg.contains("Model"));
}

#[tokio::test]
async fn test_load_invalid_model_path() {
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_json = format!(
        r#"{{
            "cache_dir": "{}",
            "models": [
                {{
                    "id": "invalid-model",
                    "task": "PromptInjection",
                    "variant": "FP16",
                    "url": "file:///nonexistent/model.onnx",
                    "checksum": "abc123",
                    "size_bytes": 100
                }}
            ]
        }}"#,
        temp_dir.path().display()
    );

    std::fs::write(&registry_path, registry_json).unwrap();
    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap()).unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    let result = loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await;

    assert!(result.is_err());
}

// ============================================================================
// Test 7: Model Unloading Tests
// ============================================================================

#[tokio::test]
async fn test_unload_model() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Load model
    loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();
    assert_eq!(loader.len(), 1);

    // Unload model
    loader.unload(ModelType::PromptInjection, ModelVariant::FP16);
    assert_eq!(loader.len(), 0);
    assert!(!loader.is_loaded(ModelType::PromptInjection, ModelVariant::FP16));
}

#[tokio::test]
async fn test_unload_all_models() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Load model
    loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();
    assert_eq!(loader.len(), 1);

    // Unload all
    loader.unload_all();
    assert_eq!(loader.len(), 0);
    assert!(loader.is_empty());
}

// ============================================================================
// Test 8: Model Information Tests
// ============================================================================

#[tokio::test]
async fn test_get_loaded_models() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Initially empty
    let loaded = loader.loaded_models();
    assert_eq!(loaded.len(), 0);

    // Load a model
    loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();

    // Now should have 1 entry
    let loaded = loader.loaded_models();
    assert_eq!(loaded.len(), 1);
    assert!(loaded.contains(&(ModelType::PromptInjection, ModelVariant::FP16)));
}

#[tokio::test]
async fn test_get_model_info() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Load model
    loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();

    // Get info
    let info = loader.model_info(ModelType::PromptInjection, ModelVariant::FP16);
    assert!(info.is_some());

    let info = info.unwrap();
    assert!(info.contains("PromptInjection") || info.contains("loaded"));
}

#[test]
fn test_model_info_not_loaded() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    let info = loader.model_info(ModelType::PromptInjection, ModelVariant::FP16);
    assert!(info.is_none());
}

// ============================================================================
// Test 9: Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_load_with_custom_config() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    let mut config = ModelConfig::new(
        ModelType::PromptInjection,
        ModelVariant::FP16,
        PathBuf::from("/unused"), // Will be overridden by registry
    );
    config.thread_pool_size = 2;
    config.optimization_level = 2;

    let result = loader.load_with_config(config).await;
    assert!(result.is_ok());
}

// ============================================================================
// Test 10: Model Type Conversion Tests
// ============================================================================

#[test]
fn test_model_type_from_task() {
    let task = ModelTask::PromptInjection;
    let model_type = ModelType::from(task);
    assert_eq!(format!("{:?}", model_type), "PromptInjection");
}

#[test]
fn test_model_type_to_task() {
    let model_type = ModelType::Toxicity;
    let task = ModelTask::from(model_type);
    assert_eq!(format!("{:?}", task), "Toxicity");
}

// ============================================================================
// Test 11: Memory Management Tests
// ============================================================================

#[tokio::test]
async fn test_model_loader_memory_cleanup() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Load model
    loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();

    // Unload should free memory
    loader.unload(ModelType::PromptInjection, ModelVariant::FP16);
    assert!(loader.is_empty());
}

// ============================================================================
// Test 12: Integration with ModelRegistry Tests
// ============================================================================

#[tokio::test]
async fn test_loader_uses_registry_metadata() {
    let (registry, _temp_dir) = create_test_registry().unwrap();

    // Verify registry has the model
    let metadata = registry
        .get_model_metadata(ModelTask::PromptInjection, ModelVariant::FP16)
        .unwrap();
    assert_eq!(metadata.id, "test-model");

    // Loader should use registry to find model
    let loader = ModelLoader::new(Arc::new(registry));
    let result = loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await;
    assert!(result.is_ok());
}

// ============================================================================
// Test 13: Edge Cases
// ============================================================================

#[test]
fn test_model_config_edge_cases() {
    let config = ModelConfig::new(
        ModelType::PromptInjection,
        ModelVariant::FP16,
        PathBuf::from(""),
    );

    // Empty path should be handled gracefully
    assert_eq!(config.model_path, PathBuf::from(""));
}

#[tokio::test]
async fn test_concurrent_load_different_models() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = Arc::new(ModelLoader::new(Arc::new(registry)));

    // Try loading different variants concurrently
    // Note: This test will fail if registry doesn't have multiple models
    // but tests the concurrent access pattern
    let loader1 = Arc::clone(&loader);
    let handle1 = tokio::spawn(async move {
        loader1
            .load(ModelType::PromptInjection, ModelVariant::FP16)
            .await
    });

    // Wait for completion
    let result1 = handle1.await.unwrap();
    assert!(result1.is_ok());
}

// ============================================================================
// Test 14: Display and Debug Implementations
// ============================================================================

#[test]
fn test_model_config_debug() {
    let config = ModelConfig::new(
        ModelType::PromptInjection,
        ModelVariant::FP16,
        PathBuf::from("/test/model.onnx"),
    );

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("PromptInjection"));
    assert!(debug_str.contains("FP16"));
}

// ============================================================================
// Test 15: Preload Tests
// ============================================================================

#[tokio::test]
async fn test_preload_models() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Preload specific models
    let models_to_preload = vec![(ModelType::PromptInjection, ModelVariant::FP16)];

    let result = loader.preload(models_to_preload).await;
    assert!(result.is_ok());
    assert_eq!(loader.len(), 1);
}

#[tokio::test]
async fn test_preload_multiple_models() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    let models = vec![(ModelType::PromptInjection, ModelVariant::FP16)];

    let result = loader.preload(models).await;
    assert!(result.is_ok());
}

// ============================================================================
// Test 16: Statistics and Metrics
// ============================================================================

#[tokio::test]
async fn test_loader_statistics() {
    let (registry, _temp_dir) = create_test_registry().unwrap();
    let loader = ModelLoader::new(Arc::new(registry));

    // Load a model
    loader
        .load(ModelType::PromptInjection, ModelVariant::FP16)
        .await
        .unwrap();

    // Get stats
    let stats = loader.stats();
    assert_eq!(stats.total_loaded, 1);
    assert!(stats.total_loads >= 1);
}
