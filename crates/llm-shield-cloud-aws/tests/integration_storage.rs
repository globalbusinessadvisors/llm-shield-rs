//! Integration tests for AWS S3 Storage.
//!
//! These tests require:
//! - AWS credentials configured (environment, file, or IAM role)
//! - An S3 bucket for testing (e.g., `llm-shield-test-ACCOUNT_ID`)
//! - Permissions to create, read, update, and delete objects
//!
//! Set environment variable `TEST_S3_BUCKET` to your test bucket name.
//!
//! Run with: cargo test --test integration_storage -- --ignored

use llm_shield_cloud::{CloudStorage, PutObjectOptions};
use llm_shield_cloud_aws::AwsS3Storage;
use std::env;

fn test_bucket() -> String {
    env::var("TEST_S3_BUCKET").unwrap_or_else(|_| {
        panic!("TEST_S3_BUCKET environment variable not set");
    })
}

/// Helper to create a test object key with unique ID
fn test_object_key(prefix: &str) -> String {
    format!("test/{}/{}", prefix, uuid::Uuid::new_v4())
}

#[tokio::test]
#[ignore] // Requires AWS credentials and TEST_S3_BUCKET
async fn test_put_and_get_object() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("put-get");
    let data = b"Hello, S3!";

    // Put object
    storage
        .put_object(&key, data)
        .await
        .expect("Failed to put object");

    // Get object
    let retrieved = storage.get_object(&key).await.expect("Failed to get object");

    assert_eq!(retrieved, data);

    // Cleanup
    let _ = storage.delete_object(&key).await;
}

#[tokio::test]
#[ignore]
async fn test_multipart_upload() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("multipart");

    // Create 10MB of data (exceeds 5MB threshold for multipart)
    let data = vec![0u8; 10 * 1024 * 1024];

    // Upload (should trigger multipart upload)
    storage
        .put_object(&key, &data)
        .await
        .expect("Failed to put large object");

    // Verify upload
    let metadata = storage
        .get_object_metadata(&key)
        .await
        .expect("Failed to get metadata");

    assert_eq!(metadata.size, data.len() as u64);

    // Cleanup
    let _ = storage.delete_object(&key).await;
}

#[tokio::test]
#[ignore]
async fn test_object_exists() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("exists");
    let data = b"existence test";

    // Object should not exist
    let exists_before = storage
        .object_exists(&key)
        .await
        .expect("Failed to check existence");
    assert!(!exists_before);

    // Create object
    storage
        .put_object(&key, data)
        .await
        .expect("Failed to put object");

    // Object should exist
    let exists_after = storage
        .object_exists(&key)
        .await
        .expect("Failed to check existence");
    assert!(exists_after);

    // Cleanup
    let _ = storage.delete_object(&key).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_object() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("delete");
    let data = b"to be deleted";

    // Create object
    storage
        .put_object(&key, data)
        .await
        .expect("Failed to put object");

    // Verify it exists
    assert!(storage.object_exists(&key).await.unwrap());

    // Delete object
    storage
        .delete_object(&key)
        .await
        .expect("Failed to delete object");

    // Verify it no longer exists
    assert!(!storage.object_exists(&key).await.unwrap());
}

#[tokio::test]
#[ignore]
async fn test_list_objects() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let prefix = format!("test/list-{}/", uuid::Uuid::new_v4());
    let key1 = format!("{}file1.txt", prefix);
    let key2 = format!("{}file2.txt", prefix);
    let key3 = format!("{}file3.txt", prefix);

    // Create objects
    storage
        .put_object(&key1, b"content1")
        .await
        .expect("Failed to put object 1");
    storage
        .put_object(&key2, b"content2")
        .await
        .expect("Failed to put object 2");
    storage
        .put_object(&key3, b"content3")
        .await
        .expect("Failed to put object 3");

    // List objects
    let objects = storage
        .list_objects(&prefix)
        .await
        .expect("Failed to list objects");

    assert_eq!(objects.len(), 3);
    assert!(objects.contains(&key1));
    assert!(objects.contains(&key2));
    assert!(objects.contains(&key3));

    // Cleanup
    let _ = storage.delete_object(&key1).await;
    let _ = storage.delete_object(&key2).await;
    let _ = storage.delete_object(&key3).await;
}

#[tokio::test]
#[ignore]
async fn test_get_object_metadata() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("metadata");
    let data = b"metadata test content";

    // Upload with content type
    let options = PutObjectOptions {
        content_type: Some("text/plain".to_string()),
        storage_class: Some("STANDARD".to_string()),
        ..Default::default()
    };

    storage
        .put_object_with_options(&key, data, &options)
        .await
        .expect("Failed to put object with options");

    // Get metadata
    let metadata = storage
        .get_object_metadata(&key)
        .await
        .expect("Failed to get metadata");

    assert_eq!(metadata.size, data.len() as u64);
    assert_eq!(metadata.content_type, Some("text/plain".to_string()));
    assert!(metadata.etag.is_some());

    // Cleanup
    let _ = storage.delete_object(&key).await;
}

#[tokio::test]
#[ignore]
async fn test_copy_object() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let source_key = test_object_key("copy-source");
    let dest_key = test_object_key("copy-dest");
    let data = b"content to copy";

    // Create source object
    storage
        .put_object(&source_key, data)
        .await
        .expect("Failed to put source object");

    // Copy object
    storage
        .copy_object(&source_key, &dest_key)
        .await
        .expect("Failed to copy object");

    // Verify destination
    let retrieved = storage
        .get_object(&dest_key)
        .await
        .expect("Failed to get copied object");

    assert_eq!(retrieved, data);

    // Cleanup
    let _ = storage.delete_object(&source_key).await;
    let _ = storage.delete_object(&dest_key).await;
}

#[tokio::test]
#[ignore]
async fn test_put_object_with_options() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("options");
    let data = b"content with options";

    let mut metadata = vec![];
    metadata.push(("purpose".to_string(), "integration-test".to_string()));
    metadata.push(("environment".to_string(), "test".to_string()));

    let options = PutObjectOptions {
        content_type: Some("application/octet-stream".to_string()),
        storage_class: Some("STANDARD".to_string()),
        encryption: Some("AES256".to_string()),
        metadata,
    };

    // Upload with options
    storage
        .put_object_with_options(&key, data, &options)
        .await
        .expect("Failed to put object with options");

    // Verify metadata
    let object_metadata = storage
        .get_object_metadata(&key)
        .await
        .expect("Failed to get metadata");

    assert_eq!(
        object_metadata.content_type,
        Some("application/octet-stream".to_string())
    );

    // Cleanup
    let _ = storage.delete_object(&key).await;
}

#[tokio::test]
#[ignore]
async fn test_delete_objects_batch() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let prefix = format!("test/batch-delete-{}/", uuid::Uuid::new_v4());
    let keys: Vec<String> = (0..10)
        .map(|i| format!("{}file{}.txt", prefix, i))
        .collect();

    // Create objects
    for key in &keys {
        storage
            .put_object(key, b"batch delete test")
            .await
            .expect("Failed to put object");
    }

    // Verify all exist
    for key in &keys {
        assert!(storage.object_exists(key).await.unwrap());
    }

    // Delete all at once
    storage
        .delete_objects(&keys)
        .await
        .expect("Failed to delete objects batch");

    // Verify all deleted
    for key in &keys {
        assert!(!storage.object_exists(key).await.unwrap());
    }
}

#[tokio::test]
#[ignore]
async fn test_list_objects_with_metadata() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let prefix = format!("test/list-metadata-{}/", uuid::Uuid::new_v4());
    let key1 = format!("{}small.txt", prefix);
    let key2 = format!("{}large.txt", prefix);

    // Create objects of different sizes
    storage
        .put_object(&key1, b"small")
        .await
        .expect("Failed to put small object");
    storage
        .put_object(&key2, &vec![0u8; 1024 * 1024])
        .await
        .expect("Failed to put large object");

    // List with metadata
    let objects_metadata = storage
        .list_objects_with_metadata(&prefix)
        .await
        .expect("Failed to list objects with metadata");

    assert_eq!(objects_metadata.len(), 2);

    // Check sizes
    let small_metadata = objects_metadata.iter().find(|m| m.size == 5).unwrap();
    let large_metadata = objects_metadata
        .iter()
        .find(|m| m.size == 1024 * 1024)
        .unwrap();

    assert_eq!(small_metadata.size, 5);
    assert_eq!(large_metadata.size, 1024 * 1024);

    // Cleanup
    let _ = storage.delete_object(&key1).await;
    let _ = storage.delete_object(&key2).await;
}

#[tokio::test]
#[ignore]
async fn test_region_configuration() {
    // Test with specific region
    let storage = AwsS3Storage::new_with_region(&test_bucket(), "us-west-2")
        .await
        .expect("Failed to initialize AwsS3Storage with region");

    assert_eq!(storage.region(), "us-west-2");
    assert_eq!(storage.bucket(), test_bucket());

    // Test operations work with specified region
    let key = test_object_key("region-test");
    let data = b"region test content";

    storage
        .put_object(&key, data)
        .await
        .expect("Failed to put object in us-west-2");

    let retrieved = storage
        .get_object(&key)
        .await
        .expect("Failed to get object from us-west-2");

    assert_eq!(retrieved, data);

    // Cleanup
    let _ = storage.delete_object(&key).await;
}

#[tokio::test]
#[ignore]
async fn test_large_file_operations() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("large-file");

    // Create 50MB file (will use multipart upload)
    let size = 50 * 1024 * 1024;
    let data = vec![0xAB; size];

    // Upload
    let start = std::time::Instant::now();
    storage
        .put_object(&key, &data)
        .await
        .expect("Failed to upload large file");
    let upload_duration = start.elapsed();

    println!("Uploaded 50MB in {:?}", upload_duration);

    // Download
    let start = std::time::Instant::now();
    let retrieved = storage
        .get_object(&key)
        .await
        .expect("Failed to download large file");
    let download_duration = start.elapsed();

    println!("Downloaded 50MB in {:?}", download_duration);

    // Verify
    assert_eq!(retrieved.len(), size);
    assert_eq!(retrieved, data);

    // Cleanup
    let _ = storage.delete_object(&key).await;
}

#[tokio::test]
#[ignore]
async fn test_storage_class() {
    let storage = AwsS3Storage::new(&test_bucket())
        .await
        .expect("Failed to initialize AwsS3Storage");

    let key = test_object_key("storage-class");
    let data = b"storage class test";

    // Upload with INTELLIGENT_TIERING storage class
    let options = PutObjectOptions {
        storage_class: Some("INTELLIGENT_TIERING".to_string()),
        ..Default::default()
    };

    storage
        .put_object_with_options(&key, data, &options)
        .await
        .expect("Failed to put object with storage class");

    // Verify (note: storage class might take time to reflect)
    let metadata = storage
        .get_object_metadata(&key)
        .await
        .expect("Failed to get metadata");

    println!("Storage class: {:?}", metadata.storage_class);

    // Cleanup
    let _ = storage.delete_object(&key).await;
}
