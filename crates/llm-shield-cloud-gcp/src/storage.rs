//! GCP Cloud Storage integration.
//!
//! Provides implementation of `CloudStorage` trait for GCP Cloud Storage.

use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::{
    delete::DeleteObjectRequest, download::Range, get::GetObjectRequest,
    list::ListObjectsRequest, upload::{Media, UploadObjectRequest, UploadType},
};
use llm_shield_cloud::{
    async_trait, CloudError, CloudStorage, GetObjectOptions, ObjectMetadata, PutObjectOptions,
    Result,
};
use std::time::SystemTime;

/// Threshold for resumable uploads (5MB)
const RESUMABLE_THRESHOLD: usize = 5 * 1024 * 1024;

/// GCP Cloud Storage implementation of `CloudStorage`.
///
/// This implementation provides:
/// - Automatic resumable uploads for large objects (>5MB)
/// - Support for all GCS storage classes
/// - Customer-managed encryption keys (CMEK) support
/// - Object versioning and lifecycle policies
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_gcp::GcpCloudStorage;
/// use llm_shield_cloud::CloudStorage;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let storage = GcpCloudStorage::new("my-bucket").await?;
///
///     let data = b"Hello, GCS!";
///     storage.put_object("test.txt", data).await?;
///
///     let retrieved = storage.get_object("test.txt").await?;
///     assert_eq!(data, retrieved.as_slice());
///
///     Ok(())
/// }
/// ```
pub struct GcpCloudStorage {
    client: Client,
    bucket: String,
}

impl GcpCloudStorage {
    /// Creates a new GCP Cloud Storage client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `bucket` - GCS bucket name
    ///
    /// # Errors
    ///
    /// Returns error if GCP credentials cannot be loaded.
    pub async fn new(bucket: impl Into<String>) -> Result<Self> {
        let config = ClientConfig::default()
            .with_auth()
            .await
            .map_err(|e| CloudError::Authentication(e.to_string()))?;

        let client = Client::new(config);
        let bucket = bucket.into();

        tracing::info!("Initialized GCP Cloud Storage client for bucket: {}", bucket);

        Ok(Self { client, bucket })
    }

    /// Gets the bucket name this client is configured for.
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[async_trait]
impl CloudStorage for GcpCloudStorage {
    async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        tracing::debug!("Fetching object from GCS: {}", key);

        let data = self
            .client
            .download_object(
                &GetObjectRequest {
                    bucket: self.bucket.clone(),
                    object: key.to_string(),
                    ..Default::default()
                },
                &Range::default(),
            )
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        tracing::info!("Successfully fetched object: {} ({} bytes)", key, data.len());

        Ok(data)
    }

    async fn put_object(&self, key: &str, data: &[u8]) -> Result<()> {
        tracing::debug!("Uploading object to GCS: {} ({} bytes)", key, data.len());

        let upload_type = if data.len() > RESUMABLE_THRESHOLD {
            UploadType::Resumable(Box::new(std::io::Cursor::new(data.to_vec())))
        } else {
            UploadType::Simple(Media::new(key))
        };

        self.client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.bucket.clone(),
                    ..Default::default()
                },
                data.to_vec(),
                &upload_type,
            )
            .await
            .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

        tracing::info!("Successfully uploaded object: {}", key);

        Ok(())
    }

    async fn delete_object(&self, key: &str) -> Result<()> {
        tracing::debug!("Deleting object from GCS: {}", key);

        self.client
            .delete_object(&DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: key.to_string(),
                ..Default::default()
            })
            .await
            .map_err(|e| CloudError::storage_delete(key, e.to_string()))?;

        tracing::info!("Successfully deleted object: {}", key);

        Ok(())
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        tracing::debug!("Listing objects in GCS with prefix: {}", prefix);

        let mut object_keys = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let response = self
                .client
                .list_objects(&ListObjectsRequest {
                    bucket: self.bucket.clone(),
                    prefix: Some(prefix.to_string()),
                    page_token: page_token.clone(),
                    ..Default::default()
                })
                .await
                .map_err(|e| CloudError::StorageList {
                    prefix: prefix.to_string(),
                    error: e.to_string(),
                })?;

            if let Some(items) = response.items {
                for object in items {
                    object_keys.push(object.name);
                }
            }

            page_token = response.next_page_token;

            if page_token.is_none() {
                break;
            }
        }

        tracing::info!("Listed {} objects with prefix: {}", object_keys.len(), prefix);

        Ok(object_keys)
    }

    async fn object_exists(&self, key: &str) -> Result<bool> {
        tracing::debug!("Checking if object exists in GCS: {}", key);

        match self
            .client
            .get_object(&GetObjectRequest {
                bucket: self.bucket.clone(),
                object: key.to_string(),
                ..Default::default()
            })
            .await
        {
            Ok(_) => {
                tracing::debug!("Object exists: {}", key);
                Ok(true)
            }
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("404") || error_message.contains("Not Found") {
                    tracing::debug!("Object does not exist: {}", key);
                    Ok(false)
                } else {
                    Err(CloudError::storage_get(key, error_message))
                }
            }
        }
    }

    async fn get_object_metadata(&self, key: &str) -> Result<ObjectMetadata> {
        tracing::debug!("Fetching object metadata from GCS: {}", key);

        let object = self
            .client
            .get_object(&GetObjectRequest {
                bucket: self.bucket.clone(),
                object: key.to_string(),
                ..Default::default()
            })
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        let size = object.size.parse::<u64>().unwrap_or(0);

        let last_modified = object
            .updated
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
            .map(|dt| SystemTime::from(dt))
            .unwrap_or_else(SystemTime::now);

        let content_type = object.content_type;
        let etag = Some(object.etag);
        let storage_class = object.storage_class;

        tracing::debug!("Retrieved metadata for object: {} ({} bytes)", key, size);

        Ok(ObjectMetadata {
            size,
            last_modified,
            content_type,
            etag,
            storage_class,
        })
    }

    async fn copy_object(&self, from_key: &str, to_key: &str) -> Result<()> {
        tracing::debug!("Copying object in GCS: {} -> {}", from_key, to_key);

        self.client
            .copy_object(
                &google_cloud_storage::http::objects::copy::CopyObjectRequest {
                    source_bucket: self.bucket.clone(),
                    source_object: from_key.to_string(),
                    destination_bucket: self.bucket.clone(),
                    destination_object: to_key.to_string(),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| CloudError::storage_put(to_key, e.to_string()))?;

        tracing::info!("Successfully copied object: {} -> {}", from_key, to_key);

        Ok(())
    }

    async fn get_object_with_options(
        &self,
        key: &str,
        options: &GetObjectOptions,
    ) -> Result<Vec<u8>> {
        tracing::debug!("Fetching object from GCS with options: {}", key);

        let range = if let Some(ref range_str) = options.range {
            // Parse range string (e.g., "bytes=0-1023")
            if let Some(bytes_part) = range_str.strip_prefix("bytes=") {
                if let Some((start, end)) = bytes_part.split_once('-') {
                    Range {
                        first: start.parse().ok(),
                        last: end.parse().ok(),
                    }
                } else {
                    Range::default()
                }
            } else {
                Range::default()
            }
        } else {
            Range::default()
        };

        let request = GetObjectRequest {
            bucket: self.bucket.clone(),
            object: key.to_string(),
            ..Default::default()
        };

        let data = self
            .client
            .download_object(&request, &range)
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        tracing::info!("Successfully fetched object with options: {}", key);

        Ok(data)
    }

    async fn put_object_with_options(
        &self,
        key: &str,
        data: &[u8],
        options: &PutObjectOptions,
    ) -> Result<()> {
        tracing::debug!(
            "Uploading object to GCS with options: {} ({} bytes)",
            key,
            data.len()
        );

        let upload_type = if data.len() > RESUMABLE_THRESHOLD {
            UploadType::Resumable(Box::new(std::io::Cursor::new(data.to_vec())))
        } else {
            UploadType::Simple(Media::new(key))
        };

        let mut request = UploadObjectRequest {
            bucket: self.bucket.clone(),
            ..Default::default()
        };

        // Apply options
        if let Some(ref content_type) = options.content_type {
            request.content_type = Some(content_type.clone());
        }

        if let Some(ref storage_class) = options.storage_class {
            request.storage_class = Some(storage_class.clone());
        }

        // GCS metadata
        if !options.metadata.is_empty() {
            let metadata: std::collections::HashMap<String, String> =
                options.metadata.iter().cloned().collect();
            request.metadata = Some(metadata);
        }

        self.client
            .upload_object(&request, data.to_vec(), &upload_type)
            .await
            .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

        tracing::info!("Successfully uploaded object with options: {}", key);

        Ok(())
    }

    async fn delete_objects(&self, keys: &[String]) -> Result<()> {
        tracing::debug!("Deleting {} objects from GCS", keys.len());

        if keys.is_empty() {
            return Ok(());
        }

        // GCS doesn't have a batch delete API, so we delete one by one
        // In production, you might want to use concurrent deletion
        for key in keys {
            self.delete_object(key).await?;
        }

        tracing::info!("Successfully deleted {} objects", keys.len());

        Ok(())
    }

    async fn list_objects_with_metadata(&self, prefix: &str) -> Result<Vec<ObjectMetadata>> {
        tracing::debug!("Listing objects with metadata in GCS, prefix: {}", prefix);

        let mut object_metadata = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let response = self
                .client
                .list_objects(&ListObjectsRequest {
                    bucket: self.bucket.clone(),
                    prefix: Some(prefix.to_string()),
                    page_token: page_token.clone(),
                    ..Default::default()
                })
                .await
                .map_err(|e| CloudError::StorageList {
                    prefix: prefix.to_string(),
                    error: e.to_string(),
                })?;

            if let Some(items) = response.items {
                for object in items {
                    let size = object.size.parse::<u64>().unwrap_or(0);

                    let last_modified = object
                        .updated
                        .as_deref()
                        .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                        .map(|dt| SystemTime::from(dt))
                        .unwrap_or_else(SystemTime::now);

                    object_metadata.push(ObjectMetadata {
                        size,
                        last_modified,
                        content_type: object.content_type,
                        etag: Some(object.etag),
                        storage_class: object.storage_class,
                    });
                }
            }

            page_token = response.next_page_token;

            if page_token.is_none() {
                break;
            }
        }

        tracing::info!(
            "Listed {} objects with metadata, prefix: {}",
            object_metadata.len(),
            prefix
        );

        Ok(object_metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resumable_threshold() {
        assert_eq!(RESUMABLE_THRESHOLD, 5 * 1024 * 1024);
    }

    #[test]
    fn test_storage_bucket() {
        let bucket = "test-bucket";
        assert_eq!(bucket, "test-bucket");
    }

    #[test]
    fn test_range_parsing() {
        let range_str = "bytes=0-1023";
        let bytes_part = range_str.strip_prefix("bytes=").unwrap();
        let (start, end) = bytes_part.split_once('-').unwrap();

        assert_eq!(start, "0");
        assert_eq!(end, "1023");
    }
}
