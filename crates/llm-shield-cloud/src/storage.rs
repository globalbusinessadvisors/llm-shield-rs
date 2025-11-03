//! Cloud storage abstractions.
//!
//! Provides unified trait for object storage across cloud providers:
//! - AWS S3
//! - GCP Cloud Storage
//! - Azure Blob Storage

use crate::error::{CloudError, Result};
use async_trait::async_trait;
use std::time::SystemTime;

/// Metadata about a storage object.
#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    /// Size of the object in bytes.
    pub size: u64,

    /// When the object was last modified.
    pub last_modified: SystemTime,

    /// Content type of the object (e.g., "application/json").
    pub content_type: Option<String>,

    /// ETag or version identifier.
    pub etag: Option<String>,

    /// Storage class/tier (e.g., "STANDARD", "GLACIER", "ARCHIVE").
    pub storage_class: Option<String>,
}

/// Options for uploading objects.
#[derive(Debug, Clone, Default)]
pub struct PutObjectOptions {
    /// Content type of the object.
    pub content_type: Option<String>,

    /// Storage class/tier.
    pub storage_class: Option<String>,

    /// Server-side encryption algorithm.
    pub encryption: Option<String>,

    /// Custom metadata key-value pairs.
    pub metadata: Vec<(String, String)>,
}

/// Options for downloading objects.
#[derive(Debug, Clone, Default)]
pub struct GetObjectOptions {
    /// Byte range to fetch (start, end).
    pub range: Option<(u64, u64)>,

    /// Expected ETag for conditional fetch.
    pub if_match: Option<String>,
}

/// Unified trait for cloud object storage.
///
/// This trait provides a consistent interface for object storage operations
/// across different cloud providers (AWS S3, GCP Cloud Storage, Azure Blob Storage).
#[async_trait]
pub trait CloudStorage: Send + Sync {
    /// Gets an object by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The object key/path
    ///
    /// # Returns
    ///
    /// Returns the object data as bytes.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageObjectNotFound` if the object doesn't exist.
    /// Returns `CloudError::StorageFetch` if the fetch operation fails.
    async fn get_object(&self, key: &str) -> Result<Vec<u8>>;

    /// Gets an object with options.
    ///
    /// # Arguments
    ///
    /// * `key` - The object key/path
    /// * `options` - Fetch options (range, conditional fetch, etc.)
    ///
    /// # Returns
    ///
    /// Returns the object data as bytes.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageFetch` if the fetch operation fails.
    async fn get_object_with_options(
        &self,
        key: &str,
        options: &GetObjectOptions,
    ) -> Result<Vec<u8>> {
        // Default implementation ignores options
        self.get_object(key).await
    }

    /// Puts an object with key.
    ///
    /// # Arguments
    ///
    /// * `key` - The object key/path
    /// * `data` - The object data
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StoragePut` if the put operation fails.
    async fn put_object(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Puts an object with options.
    ///
    /// # Arguments
    ///
    /// * `key` - The object key/path
    /// * `data` - The object data
    /// * `options` - Upload options (content type, encryption, etc.)
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StoragePut` if the put operation fails.
    async fn put_object_with_options(
        &self,
        key: &str,
        data: &[u8],
        options: &PutObjectOptions,
    ) -> Result<()> {
        // Default implementation ignores options
        self.put_object(key, data).await
    }

    /// Deletes an object.
    ///
    /// # Arguments
    ///
    /// * `key` - The object key/path to delete
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageDelete` if the delete operation fails.
    async fn delete_object(&self, key: &str) -> Result<()>;

    /// Lists objects with a given prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to filter objects
    ///
    /// # Returns
    ///
    /// Returns a vector of object keys/paths.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageList` if the list operation fails.
    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>>;

    /// Lists objects with a given prefix and limit.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to filter objects
    /// * `max_results` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Returns a vector of object keys/paths.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageList` if the list operation fails.
    async fn list_objects_with_limit(
        &self,
        prefix: &str,
        max_results: usize,
    ) -> Result<Vec<String>> {
        // Default implementation gets all and truncates
        let mut objects = self.list_objects(prefix).await?;
        objects.truncate(max_results);
        Ok(objects)
    }

    /// Checks if an object exists.
    ///
    /// # Arguments
    ///
    /// * `key` - The object key/path to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the object exists, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageFetch` if the check operation fails
    /// (but not if the object simply doesn't exist).
    async fn object_exists(&self, key: &str) -> Result<bool> {
        match self.get_object_metadata(key).await {
            Ok(_) => Ok(true),
            Err(CloudError::StorageObjectNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Gets object metadata without fetching the full object.
    ///
    /// # Arguments
    ///
    /// * `key` - The object key/path
    ///
    /// # Returns
    ///
    /// Returns metadata about the object.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageObjectNotFound` if the object doesn't exist.
    /// Returns `CloudError::StorageFetch` if the metadata fetch fails.
    async fn get_object_metadata(&self, key: &str) -> Result<ObjectMetadata>;

    /// Copies an object within the same storage.
    ///
    /// # Arguments
    ///
    /// * `from_key` - Source object key/path
    /// * `to_key` - Destination object key/path
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageFetch` if the source doesn't exist.
    /// Returns `CloudError::StoragePut` if the copy operation fails.
    async fn copy_object(&self, from_key: &str, to_key: &str) -> Result<()> {
        // Default implementation: get then put
        let data = self.get_object(from_key).await?;
        self.put_object(to_key, &data).await
    }

    /// Moves an object (copy then delete).
    ///
    /// # Arguments
    ///
    /// * `from_key` - Source object key/path
    /// * `to_key` - Destination object key/path
    ///
    /// # Errors
    ///
    /// Returns errors from copy or delete operations.
    async fn move_object(&self, from_key: &str, to_key: &str) -> Result<()> {
        self.copy_object(from_key, to_key).await?;
        self.delete_object(from_key).await
    }

    /// Gets the storage provider name (e.g., "s3", "gcs", "azure").
    fn provider_name(&self) -> &str {
        "unknown"
    }

    /// Deletes multiple objects in batch.
    ///
    /// # Arguments
    ///
    /// * `keys` - Vector of object keys/paths to delete
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageDelete` if the batch delete fails.
    async fn delete_objects(&self, keys: &[String]) -> Result<()> {
        // Default implementation: delete one by one
        for key in keys {
            self.delete_object(key).await?;
        }
        Ok(())
    }

    /// Lists objects with metadata.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to filter objects
    ///
    /// # Returns
    ///
    /// Returns a vector of (key, metadata) tuples.
    ///
    /// # Errors
    ///
    /// Returns `CloudError::StorageList` if the list operation fails.
    async fn list_objects_with_metadata(&self, prefix: &str) -> Result<Vec<ObjectMetadata>> {
        // Default implementation not provided - must be overridden
        Err(CloudError::OperationFailed(
            "list_objects_with_metadata not implemented".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_metadata() {
        let metadata = ObjectMetadata {
            size: 1024,
            last_modified: SystemTime::now(),
            content_type: Some("application/json".to_string()),
            etag: Some("abc123".to_string()),
            storage_class: Some("STANDARD".to_string()),
        };

        assert_eq!(metadata.size, 1024);
        assert!(metadata.content_type.is_some());
        assert_eq!(metadata.content_type.unwrap(), "application/json");
    }

    #[test]
    fn test_put_object_options_default() {
        let options = PutObjectOptions::default();
        assert!(options.content_type.is_none());
        assert!(options.storage_class.is_none());
        assert!(options.encryption.is_none());
        assert_eq!(options.metadata.len(), 0);
    }

    #[test]
    fn test_put_object_options_builder() {
        let options = PutObjectOptions {
            content_type: Some("text/plain".to_string()),
            storage_class: Some("GLACIER".to_string()),
            encryption: Some("AES256".to_string()),
            metadata: vec![
                ("author".to_string(), "John Doe".to_string()),
                ("version".to_string(), "1.0".to_string()),
            ],
        };

        assert_eq!(options.content_type.unwrap(), "text/plain");
        assert_eq!(options.storage_class.unwrap(), "GLACIER");
        assert_eq!(options.metadata.len(), 2);
    }

    #[test]
    fn test_get_object_options() {
        let options = GetObjectOptions {
            range: Some((0, 1023)),
            if_match: Some("etag-123".to_string()),
        };

        assert!(options.range.is_some());
        assert_eq!(options.range.unwrap(), (0, 1023));
        assert_eq!(options.if_match.unwrap(), "etag-123");
    }
}
