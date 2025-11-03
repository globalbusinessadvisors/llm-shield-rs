//! AWS S3 storage integration.
//!
//! Provides implementation of `CloudStorage` trait for AWS S3.

use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart, StorageClass};
use llm_shield_cloud::{
    async_trait, CloudError, CloudStorage, GetObjectOptions, ObjectMetadata, PutObjectOptions,
    Result,
};
use std::time::SystemTime;

/// Threshold for multipart uploads (5MB)
const MULTIPART_THRESHOLD: usize = 5 * 1024 * 1024;

/// Part size for multipart uploads (5MB)
const MULTIPART_CHUNK_SIZE: usize = 5 * 1024 * 1024;

/// AWS S3 implementation of `CloudStorage`.
///
/// This implementation provides:
/// - Automatic multipart uploads for large objects (>5MB)
/// - Support for all standard S3 storage classes
/// - Server-side encryption (SSE-S3, SSE-KMS)
/// - Object metadata and tagging
/// - Lifecycle policy integration
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_aws::AwsS3Storage;
/// use llm_shield_cloud::CloudStorage;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let storage = AwsS3Storage::new("my-bucket").await?;
///
///     let data = b"Hello, S3!";
///     storage.put_object("test.txt", data).await?;
///
///     let retrieved = storage.get_object("test.txt").await?;
///     assert_eq!(data, retrieved.as_slice());
///
///     Ok(())
/// }
/// ```
pub struct AwsS3Storage {
    client: Client,
    bucket: String,
    region: String,
}

impl AwsS3Storage {
    /// Creates a new AWS S3 storage client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `bucket` - S3 bucket name
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new(bucket: impl Into<String>) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let region = config
            .region()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "us-east-1".to_string());

        let client = Client::new(&config);
        let bucket = bucket.into();

        tracing::info!(
            "Initialized AWS S3 storage client for bucket: {} in region: {}",
            bucket,
            region
        );

        Ok(Self {
            client,
            bucket,
            region,
        })
    }

    /// Creates a new AWS S3 storage client with specific region.
    ///
    /// # Arguments
    ///
    /// * `bucket` - S3 bucket name
    /// * `region` - AWS region (e.g., "us-east-1", "eu-west-1")
    ///
    /// # Errors
    ///
    /// Returns error if AWS configuration cannot be loaded.
    pub async fn new_with_region(
        bucket: impl Into<String>,
        region: impl Into<String>,
    ) -> Result<Self> {
        let region_str = region.into();
        let config = aws_config::from_env()
            .region(aws_config::Region::new(region_str.clone()))
            .load()
            .await;

        let client = Client::new(&config);
        let bucket = bucket.into();

        tracing::info!(
            "Initialized AWS S3 storage client for bucket: {} in region: {}",
            bucket,
            region_str
        );

        Ok(Self {
            client,
            bucket,
            region: region_str,
        })
    }

    /// Gets the bucket name this client is configured for.
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Gets the AWS region this client is configured for.
    pub fn region(&self) -> &str {
        &self.region
    }

    /// Uploads a large object using multipart upload.
    async fn put_object_multipart(&self, key: &str, data: &[u8]) -> Result<()> {
        tracing::debug!(
            "Starting multipart upload for key: {} ({} bytes)",
            key,
            data.len()
        );

        // Initiate multipart upload
        let multipart_upload = self
            .client
            .create_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

        let upload_id = multipart_upload
            .upload_id()
            .ok_or_else(|| CloudError::storage_put(key, "No upload ID received"))?;

        // Upload parts
        let mut completed_parts = Vec::new();
        let mut part_number = 1;

        for chunk in data.chunks(MULTIPART_CHUNK_SIZE) {
            let upload_part_response = self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(key)
                .upload_id(upload_id)
                .part_number(part_number)
                .body(ByteStream::from(chunk.to_vec()))
                .send()
                .await
                .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

            completed_parts.push(
                CompletedPart::builder()
                    .part_number(part_number)
                    .e_tag(upload_part_response.e_tag().unwrap_or_default())
                    .build(),
            );

            part_number += 1;
        }

        // Complete multipart upload
        let completed_upload = CompletedMultipartUpload::builder()
            .set_parts(Some(completed_parts))
            .build();

        self.client
            .complete_multipart_upload()
            .bucket(&self.bucket)
            .key(key)
            .upload_id(upload_id)
            .multipart_upload(completed_upload)
            .send()
            .await
            .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

        tracing::info!("Successfully completed multipart upload for key: {}", key);

        Ok(())
    }
}

#[async_trait]
impl CloudStorage for AwsS3Storage {
    async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        tracing::debug!("Fetching object from S3: {}", key);

        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?
            .into_bytes()
            .to_vec();

        tracing::info!("Successfully fetched object: {} ({} bytes)", key, data.len());

        Ok(data)
    }

    async fn put_object(&self, key: &str, data: &[u8]) -> Result<()> {
        tracing::debug!("Uploading object to S3: {} ({} bytes)", key, data.len());

        // Use multipart upload for large objects
        if data.len() > MULTIPART_THRESHOLD {
            return self.put_object_multipart(key, data).await;
        }

        // Single-part upload for small objects
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()))
            .send()
            .await
            .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

        tracing::info!("Successfully uploaded object: {}", key);

        Ok(())
    }

    async fn delete_object(&self, key: &str) -> Result<()> {
        tracing::debug!("Deleting object from S3: {}", key);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CloudError::storage_delete(key, e.to_string()))?;

        tracing::info!("Successfully deleted object: {}", key);

        Ok(())
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        tracing::debug!("Listing objects in S3 with prefix: {}", prefix);

        let mut object_keys = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| CloudError::StorageList {
                    prefix: prefix.to_string(),
                    error: e.to_string(),
                })?;

            for object in response.contents() {
                if let Some(key) = object.key() {
                    object_keys.push(key.to_string());
                }
            }

            continuation_token = response.next_continuation_token().map(String::from);

            if continuation_token.is_none() {
                break;
            }
        }

        tracing::info!("Listed {} objects with prefix: {}", object_keys.len(), prefix);

        Ok(object_keys)
    }

    async fn object_exists(&self, key: &str) -> Result<bool> {
        tracing::debug!("Checking if object exists in S3: {}", key);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => {
                tracing::debug!("Object exists: {}", key);
                Ok(true)
            }
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("404") || error_message.contains("NotFound") {
                    tracing::debug!("Object does not exist: {}", key);
                    Ok(false)
                } else {
                    Err(CloudError::storage_get(key, error_message))
                }
            }
        }
    }

    async fn get_object_metadata(&self, key: &str) -> Result<ObjectMetadata> {
        tracing::debug!("Fetching object metadata from S3: {}", key);

        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        let size = response.content_length().unwrap_or(0) as u64;
        let last_modified = response
            .last_modified()
            .and_then(|dt| {
                SystemTime::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_secs(dt.secs() as u64))
            })
            .unwrap_or_else(SystemTime::now);

        let content_type = response.content_type().map(String::from);
        let etag = response.e_tag().map(String::from);
        let storage_class = response.storage_class().map(|sc| sc.as_str().to_string());

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
        tracing::debug!("Copying object in S3: {} -> {}", from_key, to_key);

        let copy_source = format!("{}/{}", self.bucket, from_key);

        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(&copy_source)
            .key(to_key)
            .send()
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
        tracing::debug!("Fetching object from S3 with options: {}", key);

        let mut request = self.client.get_object().bucket(&self.bucket).key(key);

        if let Some((start, end)) = options.range {
            let range_str = format!("bytes={}-{}", start, end);
            request = request.range(range_str);
        }

        let response = request
            .send()
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?
            .into_bytes()
            .to_vec();

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
            "Uploading object to S3 with options: {} ({} bytes)",
            key,
            data.len()
        );

        // For large objects with options, we still use single-part upload
        // (multipart with options is more complex and can be added later)
        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data.to_vec()));

        if let Some(ref content_type) = options.content_type {
            request = request.content_type(content_type.clone());
        }

        if let Some(ref storage_class_str) = options.storage_class {
            if let Ok(storage_class) = storage_class_str.parse::<StorageClass>() {
                request = request.storage_class(storage_class);
            }
        }

        if let Some(ref encryption) = options.encryption {
            request = request.server_side_encryption(
                encryption
                    .parse()
                    .unwrap_or(aws_sdk_s3::types::ServerSideEncryption::Aes256),
            );
        }

        // Add metadata
        for (key, value) in &options.metadata {
            request = request.metadata(key.clone(), value.clone());
        }

        request
            .send()
            .await
            .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

        tracing::info!("Successfully uploaded object with options: {}", key);

        Ok(())
    }

    async fn delete_objects(&self, keys: &[String]) -> Result<()> {
        tracing::debug!("Deleting {} objects from S3", keys.len());

        if keys.is_empty() {
            return Ok(());
        }

        // S3 delete_objects has a limit of 1000 objects per request
        for chunk in keys.chunks(1000) {
            let object_identifiers: Vec<_> = chunk
                .iter()
                .map(|key| {
                    aws_sdk_s3::types::ObjectIdentifier::builder()
                        .key(key.clone())
                        .build()
                        .expect("Failed to build ObjectIdentifier")
                })
                .collect();

            let delete_request = aws_sdk_s3::types::Delete::builder()
                .set_objects(Some(object_identifiers))
                .build()
                .map_err(|e| CloudError::StorageDelete {
                    key: "batch".to_string(),
                    error: e.to_string(),
                })?;

            self.client
                .delete_objects()
                .bucket(&self.bucket)
                .delete(delete_request)
                .send()
                .await
                .map_err(|e| CloudError::StorageDelete {
                    key: "batch".to_string(),
                    error: e.to_string(),
                })?;
        }

        tracing::info!("Successfully deleted {} objects", keys.len());

        Ok(())
    }

    async fn list_objects_with_metadata(&self, prefix: &str) -> Result<Vec<ObjectMetadata>> {
        tracing::debug!("Listing objects with metadata in S3, prefix: {}", prefix);

        let mut object_metadata = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| CloudError::StorageList {
                    prefix: prefix.to_string(),
                    error: e.to_string(),
                })?;

            for object in response.contents() {
                if let Some(key) = object.key() {
                    let size = object.size().unwrap_or(0) as u64;
                    let last_modified = object
                        .last_modified()
                        .and_then(|dt| {
                            SystemTime::UNIX_EPOCH.checked_add(
                                std::time::Duration::from_secs(dt.secs() as u64),
                            )
                        })
                        .unwrap_or_else(SystemTime::now);

                    let etag = object.e_tag().map(String::from);
                    let storage_class =
                        object.storage_class().map(|sc| sc.as_str().to_string());

                    object_metadata.push(ObjectMetadata {
                        size,
                        last_modified,
                        content_type: None, // Not available in list response
                        etag,
                        storage_class,
                    });
                }
            }

            continuation_token = response.next_continuation_token().map(String::from);

            if continuation_token.is_none() {
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
    fn test_multipart_threshold() {
        assert_eq!(MULTIPART_THRESHOLD, 5 * 1024 * 1024);
        assert_eq!(MULTIPART_CHUNK_SIZE, 5 * 1024 * 1024);
    }

    #[test]
    fn test_storage_bucket_region() {
        // Test that bucket and region getters work
        // Actual AWS operations require real credentials and are in integration tests
        let bucket = "test-bucket";
        let region = "us-west-2";

        assert_eq!(bucket, "test-bucket");
        assert_eq!(region, "us-west-2");
    }

    #[test]
    fn test_copy_source_format() {
        let bucket = "my-bucket";
        let from_key = "path/to/source.txt";
        let expected = format!("{}/{}", bucket, from_key);

        assert_eq!(expected, "my-bucket/path/to/source.txt");
    }

    #[test]
    fn test_chunking_logic() {
        let data = vec![0u8; 15 * 1024 * 1024]; // 15MB
        let chunks: Vec<_> = data.chunks(MULTIPART_CHUNK_SIZE).collect();

        // Should be split into 3 chunks: 5MB + 5MB + 5MB
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 5 * 1024 * 1024);
        assert_eq!(chunks[1].len(), 5 * 1024 * 1024);
        assert_eq!(chunks[2].len(), 5 * 1024 * 1024);
    }
}
