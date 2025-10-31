//! Acceptance tests for ModelRegistry
//!
//! Following London School TDD - these tests drive the implementation.

use llm_shield_models::{ModelRegistry, ModelTask, ModelVariant};
use std::sync::Arc;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_registry_loads_model_metadata() {
    // Given: A model registry with test catalog
    let test_dir = TempDir::new().unwrap();
    let registry_path = test_dir.path().join("registry.json");

    // Create a test registry file
    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "prompt-injection-fp16",
                "task": "PromptInjection",
                "variant": "FP16",
                "url": "https://example.com/models/prompt-injection-fp16.onnx",
                "checksum": "abc123",
                "size_bytes": 1024000
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When: We query for a model
    let metadata = registry
        .get_model_metadata(ModelTask::PromptInjection, ModelVariant::FP16)
        .expect("Failed to get model metadata");

    // Then: We get correct metadata
    assert_eq!(metadata.task, ModelTask::PromptInjection);
    assert_eq!(metadata.variant, ModelVariant::FP16);
    assert!(!metadata.url.is_empty());
    assert_eq!(metadata.id, "prompt-injection-fp16");
}

#[tokio::test]
async fn test_registry_downloads_and_caches_model() {
    // Given: A registry with mock download capability
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    // Create a mock registry with a local "model" file to simulate download
    let mock_model_content = b"fake onnx model data";
    let mock_model_path = temp_dir.path().join("mock_model.onnx");
    std::fs::write(&mock_model_path, mock_model_content).unwrap();

    // Calculate checksum of mock model
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(mock_model_content);
    let checksum = format!("{:x}", hasher.finalize());

    let registry_content = format!(
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
        cache_dir.display(),
        mock_model_path.display(),
        checksum,
        mock_model_content.len()
    );

    let registry_path = temp_dir.path().join("registry.json");
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When: We request a model for the first time
    let model_path = registry
        .ensure_model_available(ModelTask::PromptInjection, ModelVariant::FP16)
        .await
        .expect("Failed to download model");

    // Then: Model is downloaded and cached
    assert!(model_path.exists(), "Model should be cached at {:?}", model_path);

    // When: We request the same model again
    let cached_path = registry
        .ensure_model_available(ModelTask::PromptInjection, ModelVariant::FP16)
        .await
        .expect("Failed to get cached model");

    // Then: Same path is returned (cached)
    assert_eq!(model_path, cached_path);
}

#[tokio::test]
async fn test_registry_verifies_checksums() {
    // Given: A registry with incorrect checksum
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    let mock_model_content = b"fake onnx model data";
    let mock_model_path = temp_dir.path().join("mock_model.onnx");
    std::fs::write(&mock_model_path, mock_model_content).unwrap();

    let registry_content = format!(
        r#"{{
            "cache_dir": "{}",
            "models": [
                {{
                    "id": "bad-checksum-model",
                    "task": "Toxicity",
                    "variant": "FP32",
                    "url": "file://{}",
                    "checksum": "incorrect_checksum_value",
                    "size_bytes": {}
                }}
            ]
        }}"#,
        cache_dir.display(),
        mock_model_path.display(),
        mock_model_content.len()
    );

    let registry_path = temp_dir.path().join("registry.json");
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When/Then: Requesting model with bad checksum should fail
    let result = registry
        .ensure_model_available(ModelTask::Toxicity, ModelVariant::FP32)
        .await;

    assert!(result.is_err(), "Should fail checksum verification");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("checksum") || err.to_string().contains("Model error"),
        "Error should mention checksum mismatch: {}",
        err
    );
}

#[tokio::test]
async fn test_registry_handles_missing_model() {
    // Given: A registry without the requested model
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": []
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When/Then: Querying for non-existent model should fail
    let result = registry.get_model_metadata(ModelTask::PromptInjection, ModelVariant::FP16);

    assert!(result.is_err(), "Should fail for missing model");
}

#[test]
fn test_model_task_variants() {
    // Test that all model tasks can be created
    let tasks = vec![
        ModelTask::PromptInjection,
        ModelTask::Toxicity,
        ModelTask::Sentiment,
    ];

    for task in tasks {
        assert!(format!("{:?}", task).len() > 0);
    }
}

#[test]
fn test_model_variant_types() {
    // Test that all model variants can be created
    let variants = vec![
        ModelVariant::FP16,
        ModelVariant::FP32,
        ModelVariant::INT8,
    ];

    for variant in variants {
        assert!(format!("{:?}", variant).len() > 0);
    }
}

// ============================================================================
// Thread Safety Tests (London School TDD)
// ============================================================================

#[tokio::test]
async fn test_registry_thread_safe_concurrent_reads() {
    // Given: A registry with multiple models
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "prompt-injection-fp16",
                "task": "PromptInjection",
                "variant": "FP16",
                "url": "https://example.com/models/prompt-injection-fp16.onnx",
                "checksum": "abc123",
                "size_bytes": 1024000
            },
            {
                "id": "toxicity-fp32",
                "task": "Toxicity",
                "variant": "FP32",
                "url": "https://example.com/models/toxicity-fp32.onnx",
                "checksum": "def456",
                "size_bytes": 2048000
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = Arc::new(
        ModelRegistry::from_file(registry_path.to_str().unwrap())
            .expect("Failed to load registry")
    );

    // When: Multiple threads read concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = tokio::spawn(async move {
            let task = if i % 2 == 0 {
                ModelTask::PromptInjection
            } else {
                ModelTask::Toxicity
            };
            let variant = if i % 2 == 0 {
                ModelVariant::FP16
            } else {
                ModelVariant::FP32
            };

            // Then: Each thread can read metadata successfully
            let metadata = registry_clone
                .get_model_metadata(task, variant)
                .expect("Should read metadata concurrently");
            metadata.id.clone()
        });
        handles.push(handle);
    }

    // Then: All reads complete successfully
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_registry_clone_shares_data() {
    // Given: A registry
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
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
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry1 = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When: We clone the registry
    let registry2 = registry1.clone();

    // Then: Both registries can access the same data
    let metadata1 = registry1.get_model_metadata(ModelTask::PromptInjection, ModelVariant::FP16);
    let metadata2 = registry2.get_model_metadata(ModelTask::PromptInjection, ModelVariant::FP16);

    assert!(metadata1.is_ok());
    assert!(metadata2.is_ok());
    assert_eq!(metadata1.unwrap().id, metadata2.unwrap().id);
}

// ============================================================================
// Model Discovery and Capability Querying Tests
// ============================================================================

#[tokio::test]
async fn test_registry_list_available_models() {
    // Given: A registry with multiple models
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "prompt-injection-fp16",
                "task": "PromptInjection",
                "variant": "FP16",
                "url": "https://example.com/model1.onnx",
                "checksum": "abc123",
                "size_bytes": 1024
            },
            {
                "id": "prompt-injection-fp32",
                "task": "PromptInjection",
                "variant": "FP32",
                "url": "https://example.com/model2.onnx",
                "checksum": "def456",
                "size_bytes": 2048
            },
            {
                "id": "toxicity-int8",
                "task": "Toxicity",
                "variant": "INT8",
                "url": "https://example.com/model3.onnx",
                "checksum": "ghi789",
                "size_bytes": 512
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When: We list all models
    let models = registry.list_models();

    // Then: All 3 models are returned
    assert_eq!(models.len(), 3);
}

#[tokio::test]
async fn test_registry_list_models_by_task() {
    // Given: A registry with multiple models
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "prompt-injection-fp16",
                "task": "PromptInjection",
                "variant": "FP16",
                "url": "https://example.com/model1.onnx",
                "checksum": "abc123",
                "size_bytes": 1024
            },
            {
                "id": "prompt-injection-fp32",
                "task": "PromptInjection",
                "variant": "FP32",
                "url": "https://example.com/model2.onnx",
                "checksum": "def456",
                "size_bytes": 2048
            },
            {
                "id": "toxicity-int8",
                "task": "Toxicity",
                "variant": "INT8",
                "url": "https://example.com/model3.onnx",
                "checksum": "ghi789",
                "size_bytes": 512
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When: We list models for PromptInjection task
    let models = registry.list_models_for_task(ModelTask::PromptInjection);

    // Then: Only 2 PromptInjection models are returned
    assert_eq!(models.len(), 2);
    assert!(models.iter().all(|m| m.task == ModelTask::PromptInjection));
}

#[tokio::test]
async fn test_registry_get_available_variants() {
    // Given: A registry with multiple variants of same task
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "prompt-injection-fp16",
                "task": "PromptInjection",
                "variant": "FP16",
                "url": "https://example.com/model1.onnx",
                "checksum": "abc123",
                "size_bytes": 1024
            },
            {
                "id": "prompt-injection-fp32",
                "task": "PromptInjection",
                "variant": "FP32",
                "url": "https://example.com/model2.onnx",
                "checksum": "def456",
                "size_bytes": 2048
            },
            {
                "id": "prompt-injection-int8",
                "task": "PromptInjection",
                "variant": "INT8",
                "url": "https://example.com/model3.onnx",
                "checksum": "ghi789",
                "size_bytes": 512
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When: We get available variants for PromptInjection
    let variants = registry.get_available_variants(ModelTask::PromptInjection);

    // Then: All 3 variants are returned
    assert_eq!(variants.len(), 3);
    assert!(variants.contains(&ModelVariant::FP16));
    assert!(variants.contains(&ModelVariant::FP32));
    assert!(variants.contains(&ModelVariant::INT8));
}

#[tokio::test]
async fn test_registry_has_model() {
    // Given: A registry with specific models
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "prompt-injection-fp16",
                "task": "PromptInjection",
                "variant": "FP16",
                "url": "https://example.com/model.onnx",
                "checksum": "abc123",
                "size_bytes": 1024
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When/Then: Check if models exist
    assert!(registry.has_model(ModelTask::PromptInjection, ModelVariant::FP16));
    assert!(!registry.has_model(ModelTask::PromptInjection, ModelVariant::FP32));
    assert!(!registry.has_model(ModelTask::Toxicity, ModelVariant::FP16));
}

// ============================================================================
// Registry Statistics and Info Tests
// ============================================================================

#[tokio::test]
async fn test_registry_model_count() {
    // Given: A registry with known number of models
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "model1",
                "task": "PromptInjection",
                "variant": "FP16",
                "url": "https://example.com/model1.onnx",
                "checksum": "abc123",
                "size_bytes": 1024
            },
            {
                "id": "model2",
                "task": "Toxicity",
                "variant": "FP32",
                "url": "https://example.com/model2.onnx",
                "checksum": "def456",
                "size_bytes": 2048
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When/Then: Count matches expected
    assert_eq!(registry.model_count(), 2);
}

#[tokio::test]
async fn test_registry_is_empty() {
    // Given: An empty registry
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": []
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = ModelRegistry::from_file(registry_path.to_str().unwrap())
        .expect("Failed to load registry");

    // When/Then: Registry is empty
    assert!(registry.is_empty());
    assert_eq!(registry.model_count(), 0);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_registry_invalid_json() {
    // Given: Invalid JSON file
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    std::fs::write(&registry_path, "{ invalid json }").unwrap();

    // When/Then: Loading fails with clear error
    let result = ModelRegistry::from_file(registry_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("parse"));
}

#[tokio::test]
async fn test_registry_missing_file() {
    // When/Then: Loading non-existent file fails
    let result = ModelRegistry::from_file("/nonexistent/registry.json");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("read"));
}

// ============================================================================
// Integration Tests with ModelLoader
// ============================================================================

#[tokio::test]
async fn test_registry_integration_with_model_loader() {
    use llm_shield_models::{ModelLoader, ModelType};

    // Given: A registry with a test model
    let temp_dir = TempDir::new().unwrap();
    let cache_dir = temp_dir.path().join("cache");

    // Create a mock model file
    let mock_model_content = b"fake onnx model data for loader test";
    let mock_model_path = temp_dir.path().join("mock_model.onnx");
    std::fs::write(&mock_model_path, mock_model_content).unwrap();

    // Calculate checksum
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(mock_model_content);
    let checksum = format!("{:x}", hasher.finalize());

    let registry_content = format!(
        r#"{{
            "cache_dir": "{}",
            "models": [
                {{
                    "id": "loader-test-model",
                    "task": "PromptInjection",
                    "variant": "FP16",
                    "url": "file://{}",
                    "checksum": "{}",
                    "size_bytes": {}
                }}
            ]
        }}"#,
        cache_dir.display(),
        mock_model_path.display(),
        checksum,
        mock_model_content.len()
    );

    let registry_path = temp_dir.path().join("registry.json");
    std::fs::write(&registry_path, registry_content).unwrap();

    // When: We create a ModelLoader with the registry
    let registry = Arc::new(
        ModelRegistry::from_file(registry_path.to_str().unwrap())
            .expect("Failed to load registry")
    );

    let loader = ModelLoader::new(Arc::clone(&registry));

    // Then: Registry and loader work together
    assert!(registry.has_model(ModelTask::PromptInjection, ModelVariant::FP16));
    assert_eq!(registry.model_count(), 1);

    // Verify the loader can see the registry
    assert!(!loader.is_loaded(ModelType::PromptInjection, ModelVariant::FP16));
}

#[tokio::test]
async fn test_registry_provides_metadata_to_loader() {
    // Given: A registry with model metadata
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "metadata-test-model",
                "task": "Toxicity",
                "variant": "FP32",
                "url": "https://example.com/model.onnx",
                "checksum": "abc123def456",
                "size_bytes": 5242880
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = Arc::new(
        ModelRegistry::from_file(registry_path.to_str().unwrap())
            .expect("Failed to load registry")
    );

    // When: We query metadata
    let metadata = registry
        .get_model_metadata(ModelTask::Toxicity, ModelVariant::FP32)
        .expect("Should get metadata");

    // Then: Metadata is accessible and correct
    assert_eq!(metadata.id, "metadata-test-model");
    assert_eq!(metadata.task, ModelTask::Toxicity);
    assert_eq!(metadata.variant, ModelVariant::FP32);
    assert_eq!(metadata.size_bytes, 5242880);
    assert_eq!(metadata.checksum, "abc123def456");
    assert_eq!(metadata.url, "https://example.com/model.onnx");
}

#[tokio::test]
async fn test_registry_supports_multiple_model_loader_instances() {
    use llm_shield_models::ModelLoader;

    // Given: A registry shared by multiple loaders
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");

    let registry_content = r#"{
        "cache_dir": "/tmp/llm-shield/models",
        "models": [
            {
                "id": "shared-model",
                "task": "Sentiment",
                "variant": "INT8",
                "url": "https://example.com/model.onnx",
                "checksum": "abc123",
                "size_bytes": 1024
            }
        ]
    }"#;
    std::fs::write(&registry_path, registry_content).unwrap();

    let registry = Arc::new(
        ModelRegistry::from_file(registry_path.to_str().unwrap())
            .expect("Failed to load registry")
    );

    // When: We create multiple loaders with the same registry
    let loader1 = ModelLoader::new(Arc::clone(&registry));
    let loader2 = ModelLoader::new(Arc::clone(&registry));
    let loader3 = ModelLoader::new(registry);

    // Then: All loaders start empty (no models loaded yet)
    // but they all have access to the same registry
    assert!(loader1.is_empty());
    assert!(loader2.is_empty());
    assert!(loader3.is_empty());

    // Verify loaders are independent instances
    assert_eq!(loader1.len(), 0);
    assert_eq!(loader2.len(), 0);
    assert_eq!(loader3.len(), 0);
}
