//! Integration tests for AWS Secrets Manager.
//!
//! These tests require:
//! - AWS credentials configured (environment, file, or IAM role)
//! - Permissions to create, read, update, and delete secrets
//! - Test secrets with prefix `llm-shield-test/`
//!
//! Run with: cargo test --test integration_secrets -- --ignored

use llm_shield_cloud::{CloudSecretManager, SecretValue};
use llm_shield_cloud_aws::AwsSecretsManager;
use std::collections::HashMap;

const TEST_SECRET_PREFIX: &str = "llm-shield-test";

/// Helper to create a test secret name with unique ID
fn test_secret_name(suffix: &str) -> String {
    format!("{}/{}-{}", TEST_SECRET_PREFIX, suffix, uuid::Uuid::new_v4())
}

#[tokio::test]
#[ignore] // Requires AWS credentials
async fn test_create_and_get_secret() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("create-get");
    let secret_value = SecretValue::from_string("test-secret-value".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &secret_value)
        .await
        .expect("Failed to create secret");

    // Get secret
    let retrieved = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get secret");

    assert_eq!(retrieved.as_string(), "test-secret-value");

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_secret_caching() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("caching");
    let secret_value = SecretValue::from_string("cached-value".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &secret_value)
        .await
        .expect("Failed to create secret");

    // First fetch (cache miss)
    let start = std::time::Instant::now();
    let _value1 = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get secret");
    let first_duration = start.elapsed();

    // Second fetch (cache hit - should be faster)
    let start = std::time::Instant::now();
    let _value2 = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get secret from cache");
    let second_duration = start.elapsed();

    // Cached fetch should be significantly faster
    assert!(second_duration < first_duration / 2);

    // Cache size should be at least 1
    assert!(manager.cache_size().await >= 1);

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_update_secret() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("update");
    let initial_value = SecretValue::from_string("initial-value".to_string());
    let updated_value = SecretValue::from_string("updated-value".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &initial_value)
        .await
        .expect("Failed to create secret");

    // Update secret
    manager
        .update_secret(&secret_name, &updated_value)
        .await
        .expect("Failed to update secret");

    // Get updated secret
    let retrieved = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get updated secret");

    assert_eq!(retrieved.as_string(), "updated-value");

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_list_secrets() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret1_name = test_secret_name("list-1");
    let secret2_name = test_secret_name("list-2");
    let secret_value = SecretValue::from_string("list-test-value".to_string());

    // Create two secrets
    manager
        .create_secret(&secret1_name, &secret_value)
        .await
        .expect("Failed to create secret 1");
    manager
        .create_secret(&secret2_name, &secret_value)
        .await
        .expect("Failed to create secret 2");

    // List secrets
    let secrets = manager
        .list_secrets()
        .await
        .expect("Failed to list secrets");

    // Should contain our test secrets
    assert!(secrets.iter().any(|s| s.contains(&secret1_name)));
    assert!(secrets.iter().any(|s| s.contains(&secret2_name)));

    // Cleanup
    let _ = manager.delete_secret(&secret1_name).await;
    let _ = manager.delete_secret(&secret2_name).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_secret() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("delete");
    let secret_value = SecretValue::from_string("to-be-deleted".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &secret_value)
        .await
        .expect("Failed to create secret");

    // Delete secret
    manager
        .delete_secret(&secret_name)
        .await
        .expect("Failed to delete secret");

    // Try to get deleted secret (should fail after recovery window)
    // Note: AWS Secrets Manager has a 30-day recovery window by default
    // So the secret still exists but is marked for deletion
    let result = manager.get_secret(&secret_name).await;

    // In recovery window, we can still get the secret
    // After recovery window, this would return an error
    println!("Get after delete result: {:?}", result);
}

#[tokio::test]
#[ignore]
async fn test_rotate_secret() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("rotate");
    let initial_value = SecretValue::from_string("initial-value".to_string());
    let rotated_value = SecretValue::from_string("rotated-value".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &initial_value)
        .await
        .expect("Failed to create secret");

    // Rotate secret
    manager
        .rotate_secret(&secret_name, &rotated_value)
        .await
        .expect("Failed to rotate secret");

    // Get rotated secret
    let retrieved = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get rotated secret");

    assert_eq!(retrieved.as_string(), "rotated-value");

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_get_secret_metadata() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("metadata");
    let secret_value = SecretValue::from_string("metadata-test".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &secret_value)
        .await
        .expect("Failed to create secret");

    // Get metadata
    let metadata = manager
        .get_secret_metadata(&secret_name)
        .await
        .expect("Failed to get secret metadata");

    assert_eq!(metadata.name, secret_name);
    assert!(metadata.created_at <= std::time::SystemTime::now());
    assert!(metadata.updated_at >= metadata.created_at);

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_json_secret() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("json");

    // Create JSON secret
    let json_data = serde_json::json!({
        "api_key": "test-key-123",
        "api_secret": "test-secret-456",
        "endpoint": "https://api.example.com"
    });

    let secret_value = SecretValue::from_string(json_data.to_string());

    manager
        .create_secret(&secret_name, &secret_value)
        .await
        .expect("Failed to create JSON secret");

    // Get and parse JSON secret
    let retrieved = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get JSON secret");

    let parsed: serde_json::Value = serde_json::from_str(retrieved.as_string())
        .expect("Failed to parse JSON secret");

    assert_eq!(parsed["api_key"], "test-key-123");
    assert_eq!(parsed["api_secret"], "test-secret-456");
    assert_eq!(parsed["endpoint"], "https://api.example.com");

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_cache_invalidation_on_update() {
    let manager = AwsSecretsManager::new()
        .await
        .expect("Failed to initialize AwsSecretsManager");

    let secret_name = test_secret_name("cache-invalidation");
    let initial_value = SecretValue::from_string("cached-value-1".to_string());
    let updated_value = SecretValue::from_string("cached-value-2".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &initial_value)
        .await
        .expect("Failed to create secret");

    // Get secret (cache it)
    let value1 = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get secret");
    assert_eq!(value1.as_string(), "cached-value-1");

    // Update secret (should invalidate cache)
    manager
        .update_secret(&secret_name, &updated_value)
        .await
        .expect("Failed to update secret");

    // Get secret again (should fetch from AWS, not cache)
    let value2 = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get updated secret");
    assert_eq!(value2.as_string(), "cached-value-2");

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_region_configuration() {
    // Test with specific region
    let manager = AwsSecretsManager::new_with_region("us-west-2")
        .await
        .expect("Failed to initialize AwsSecretsManager with region");

    assert_eq!(manager.region(), "us-west-2");

    // Test operations work with specified region
    let secret_name = test_secret_name("region-test");
    let secret_value = SecretValue::from_string("region-test-value".to_string());

    manager
        .create_secret(&secret_name, &secret_value)
        .await
        .expect("Failed to create secret in us-west-2");

    let retrieved = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get secret from us-west-2");

    assert_eq!(retrieved.as_string(), "region-test-value");

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}

#[tokio::test]
#[ignore]
async fn test_cache_ttl_configuration() {
    // Test with short cache TTL (10 seconds)
    let manager = AwsSecretsManager::new_with_cache_ttl("us-east-1", 10)
        .await
        .expect("Failed to initialize AwsSecretsManager with custom TTL");

    let secret_name = test_secret_name("cache-ttl");
    let secret_value = SecretValue::from_string("ttl-test-value".to_string());

    // Create secret
    manager
        .create_secret(&secret_name, &secret_value)
        .await
        .expect("Failed to create secret");

    // Get secret (cache it)
    let _value1 = manager
        .get_secret(&secret_name)
        .await
        .expect("Failed to get secret");

    // Cache should contain the secret
    assert!(manager.cache_size().await >= 1);

    // Wait for cache to expire (11 seconds)
    tokio::time::sleep(tokio::time::Duration::from_secs(11)).await;

    // Cache should be expired (note: automatic cleanup may not have run yet)
    // Getting the secret again should fetch from AWS

    // Cleanup
    let _ = manager.delete_secret(&secret_name).await;
}
