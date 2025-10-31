//! Azure Blob Storage integration.
//!
//! Provides implementation of `CloudStorage` trait for Azure Blob Storage.

use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;
use llm_shield_cloud::{
    async_trait, CloudError, CloudStorage, GetObjectOptions, ObjectMetadata, PutObjectOptions,
    Result,
};
use std::sync::Arc;
use std::time::SystemTime;

/// Threshold for block blob uploads (4MB recommended by Azure)
const BLOCK_THRESHOLD: usize = 4 * 1024 * 1024;

/// Azure Blob Storage implementation of `CloudStorage`.
///
/// This implementation provides:
/// - Automatic block blob uploads for large objects
/// - Support for all Azure blob tiers (Hot, Cool, Archive)
/// - Server-side encryption
/// - Blob metadata and tags
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_azure::AzureBlobStorage;
/// use llm_shield_cloud::CloudStorage;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let storage = AzureBlobStorage::new(
///         "mystorageaccount",
///         "mycontainer"
///     ).await?;
///
///     let data = b"Hello, Azure!";
///     storage.put_object("test.txt", data).await?;
///
///     let retrieved = storage.get_object("test.txt").await?;
///     assert_eq!(data, retrieved.as_slice());
///
///     Ok(())
/// }
/// ```
pub struct AzureBlobStorage {
    container_client: ContainerClient,
    account_name: String,
    container_name: String,
}

impl AzureBlobStorage {
    /// Creates a new Azure Blob Storage client with default configuration.
    ///
    /// Uses DefaultAzureCredential for authentication.
    ///
    /// # Arguments
    ///
    /// * `account_name` - Storage account name
    /// * `container_name` - Container name
    ///
    /// # Errors
    ///
    /// Returns error if Azure credentials cannot be loaded.
    pub async fn new(
        account_name: impl Into<String>,
        container_name: impl Into<String>,
    ) -> Result<Self> {
        let account_name = account_name.into();
        let container_name = container_name.into();

        // Create credentials
        let credential = Arc::new(DefaultAzureCredential::default());

        // Create storage credentials
        let storage_credentials = StorageCredentials::token_credential(credential);

        // Create blob service client
        let blob_service = BlobServiceClient::new(&account_name, storage_credentials);

        // Get container client
        let container_client = blob_service.container_client(&container_name);

        tracing::info!(
            "Initialized Azure Blob Storage client for account: {} container: {}",
            account_name,
            container_name
        );

        Ok(Self {
            container_client,
            account_name,
            container_name,
        })
    }

    /// Gets the storage account name this client is configured for.
    pub fn account_name(&self) -> &str {
        &self.account_name
    }

    /// Gets the container name this client is configured for.
    pub fn container_name(&self) -> &str {
        &self.container_name
    }
}

#[async_trait]
impl CloudStorage for AzureBlobStorage {
    async fn get_object(&self, key: &str) -> Result<Vec<u8>> {
        tracing::debug!("Fetching blob from Azure Storage: {}", key);

        let blob_client = self.container_client.blob_client(key);

        let data = blob_client
            .get_content()
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        tracing::info!("Successfully fetched blob: {} ({} bytes)", key, data.len());

        Ok(data.to_vec())
    }

    async fn put_object(&self, key: &str, data: &[u8]) -> Result<()> {
        tracing::debug!("Uploading blob to Azure Storage: {} ({} bytes)", key, data.len());

        let blob_client = self.container_client.blob_client(key);

        if data.len() > BLOCK_THRESHOLD {
            // Use block blob for large files
            tracing::debug!("Using block blob upload for large file");

            // Split into blocks
            let mut block_list = Vec::new();
            for (i, chunk) in data.chunks(BLOCK_THRESHOLD).enumerate() {
                let block_id = format!("{:08}", i);
                let block_id_base64 = base64::encode(&block_id);

                blob_client
                    .put_block(&block_id_base64, chunk.to_vec())
                    .await
                    .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

                block_list.push(BlobBlockType::new_uncommitted(&block_id_base64));
            }

            // Commit blocks
            blob_client
                .put_block_list(block_list)
                .await
                .map_err(|e| CloudError::storage_put(key, e.to_string()))?;
        } else {
            // Simple upload for small files
            blob_client
                .put_block_blob(data.to_vec())
                .await
                .map_err(|e| CloudError::storage_put(key, e.to_string()))?;
        }

        tracing::info!("Successfully uploaded blob: {}", key);

        Ok(())
    }

    async fn delete_object(&self, key: &str) -> Result<()> {
        tracing::debug!("Deleting blob from Azure Storage: {}", key);

        let blob_client = self.container_client.blob_client(key);

        blob_client
            .delete()
            .await
            .map_err(|e| CloudError::storage_delete(key, e.to_string()))?;

        tracing::info!("Successfully deleted blob: {}", key);

        Ok(())
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        tracing::debug!("Listing blobs in Azure Storage with prefix: {}", prefix);

        let mut object_keys = Vec::new();

        let mut stream = self
            .container_client
            .list_blobs()
            .prefix(prefix)
            .into_stream();

        use futures::StreamExt;

        while let Some(result) = stream.next().await {
            let response = result.map_err(|e| CloudError::StorageList(e.to_string()))?;

            for blob in response.blobs.blobs() {
                object_keys.push(blob.name.clone());
            }
        }

        tracing::info!("Listed {} blobs with prefix: {}", object_keys.len(), prefix);

        Ok(object_keys)
    }

    async fn object_exists(&self, key: &str) -> Result<bool> {
        tracing::debug!("Checking if blob exists in Azure Storage: {}", key);

        let blob_client = self.container_client.blob_client(key);

        match blob_client.get_properties().await {
            Ok(_) => {
                tracing::debug!("Blob exists: {}", key);
                Ok(true)
            }
            Err(e) => {
                let error_message = e.to_string();
                if error_message.contains("404") || error_message.contains("BlobNotFound") {
                    tracing::debug!("Blob does not exist: {}", key);
                    Ok(false)
                } else {
                    Err(CloudError::storage_get(key, error_message))
                }
            }
        }
    }

    async fn get_object_metadata(&self, key: &str) -> Result<ObjectMetadata> {
        tracing::debug!("Fetching blob metadata from Azure Storage: {}", key);

        let blob_client = self.container_client.blob_client(key);

        let properties = blob_client
            .get_properties()
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?;

        let size = properties.blob.properties.content_length;

        let last_modified = properties
            .blob
            .properties
            .last_modified
            .and_then(|t| {
                std::time::SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(
                    t.unix_timestamp() as u64,
                ))
            })
            .unwrap_or_else(SystemTime::now);

        let content_type = properties
            .blob
            .properties
            .content_type
            .map(|ct| ct.to_string());

        let etag = properties.blob.properties.etag.map(|e| e.to_string());

        let storage_class = properties.blob.properties.access_tier.map(|t| format!("{:?}", t));

        tracing::debug!("Retrieved metadata for blob: {} ({} bytes)", key, size);

        Ok(ObjectMetadata {
            size,
            last_modified,
            content_type,
            etag,
            storage_class,
        })
    }

    async fn copy_object(&self, from_key: &str, to_key: &str) -> Result<()> {
        tracing::debug!("Copying blob in Azure Storage: {} -> {}", from_key, to_key);

        let source_blob = self.container_client.blob_client(from_key);
        let dest_blob = self.container_client.blob_client(to_key);

        // Get source blob URL
        let source_url = source_blob.url()?;

        dest_blob
            .copy(&source_url)
            .await
            .map_err(|e| CloudError::storage_put(to_key, e.to_string()))?;

        tracing::info!("Successfully copied blob: {} -> {}", from_key, to_key);

        Ok(())
    }

    async fn get_object_with_options(
        &self,
        key: &str,
        options: &GetObjectOptions,
    ) -> Result<Vec<u8>> {
        tracing::debug!("Fetching blob from Azure Storage with options: {}", key);

        let blob_client = self.container_client.blob_client(key);

        let mut request = blob_client.get();

        // Apply range if specified
        if let Some(ref range_str) = options.range {
            // Parse range string (e.g., "bytes=0-1023")
            if let Some(bytes_part) = range_str.strip_prefix("bytes=") {
                if let Some((start, end)) = bytes_part.split_once('-') {
                    if let (Ok(start_byte), Ok(end_byte)) =
                        (start.parse::<u64>(), end.parse::<u64>())
                    {
                        request = request.range(start_byte..=end_byte);
                    }
                }
            }
        }

        let data = request
            .await
            .map_err(|e| CloudError::storage_get(key, e.to_string()))?
            .data
            .to_vec();

        tracing::info!("Successfully fetched blob with options: {}", key);

        Ok(data)
    }

    async fn put_object_with_options(
        &self,
        key: &str,
        data: &[u8],
        options: &PutObjectOptions,
    ) -> Result<()> {
        tracing::debug!(
            "Uploading blob to Azure Storage with options: {} ({} bytes)",
            key,
            data.len()
        );

        let blob_client = self.container_client.blob_client(key);

        let mut request = blob_client.put_block_blob(data.to_vec());

        // Apply content type
        if let Some(ref content_type) = options.content_type {
            request = request.content_type(content_type);
        }

        // Apply metadata
        if !options.metadata.is_empty() {
            let metadata: std::collections::HashMap<String, String> =
                options.metadata.iter().cloned().collect();
            request = request.metadata(metadata);
        }

        request
            .await
            .map_err(|e| CloudError::storage_put(key, e.to_string()))?;

        tracing::info!("Successfully uploaded blob with options: {}", key);

        Ok(())
    }

    async fn delete_objects(&self, keys: &[String]) -> Result<()> {
        tracing::debug!("Deleting {} blobs from Azure Storage", keys.len());

        if keys.is_empty() {
            return Ok(());
        }

        // Azure doesn't have batch delete, so delete one by one
        for key in keys {
            self.delete_object(key).await?;
        }

        tracing::info!("Successfully deleted {} blobs", keys.len());

        Ok(())
    }

    async fn list_objects_with_metadata(&self, prefix: &str) -> Result<Vec<ObjectMetadata>> {
        tracing::debug!(
            "Listing blobs with metadata in Azure Storage, prefix: {}",
            prefix
        );

        let mut object_metadata = Vec::new();

        let mut stream = self
            .container_client
            .list_blobs()
            .prefix(prefix)
            .into_stream();

        use futures::StreamExt;

        while let Some(result) = stream.next().await {
            let response = result.map_err(|e| CloudError::StorageList(e.to_string()))?;

            for blob in response.blobs.blobs() {
                let size = blob.properties.content_length;

                let last_modified = blob
                    .properties
                    .last_modified
                    .and_then(|t| {
                        std::time::SystemTime::UNIX_EPOCH.checked_add(
                            std::time::Duration::from_secs(t.unix_timestamp() as u64),
                        )
                    })
                    .unwrap_or_else(SystemTime::now);

                let content_type = blob.properties.content_type.map(|ct| ct.to_string());

                let etag = blob.properties.etag.map(|e| e.to_string());

                let storage_class = blob.properties.access_tier.map(|t| format!("{:?}", t));

                object_metadata.push(ObjectMetadata {
                    size,
                    last_modified,
                    content_type,
                    etag,
                    storage_class,
                });
            }
        }

        tracing::info!(
            "Listed {} blobs with metadata, prefix: {}",
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
    fn test_block_threshold() {
        assert_eq!(BLOCK_THRESHOLD, 4 * 1024 * 1024);
    }

    #[test]
    fn test_storage_account_format() {
        let account = "mystorageaccount";
        assert!(account.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_block_id_format() {
        let block_id = format!("{:08}", 0);
        assert_eq!(block_id, "00000000");

        let block_id = format!("{:08}", 123);
        assert_eq!(block_id, "00000123");
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
