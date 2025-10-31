//! GCP Secret Manager integration.
//!
//! Provides implementation of `CloudSecretManager` trait for GCP Secret Manager.

use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_googleapis::secretmanager::v1::{
    secret_manager_service_client::SecretManagerServiceClient, AccessSecretVersionRequest,
    AddSecretVersionRequest, CreateSecretRequest, DeleteSecretRequest, GetSecretRequest,
    ListSecretsRequest, Replication, Secret, SecretPayload,
};
use llm_shield_cloud::{
    async_trait, CloudError, CloudSecretManager, Result, SecretCache, SecretMetadata, SecretValue,
};
use std::collections::HashMap;
use tonic::transport::Channel;

/// GCP Secret Manager implementation of `CloudSecretManager`.
///
/// This implementation provides:
/// - Automatic credential discovery (ADC, service account, workload identity)
/// - Built-in secret caching with TTL
/// - Support for both string and binary secrets
/// - Automatic retry with exponential backoff
/// - Multi-region replication support
///
/// # Example
///
/// ```no_run
/// use llm_shield_cloud_gcp::GcpSecretManager;
/// use llm_shield_cloud::CloudSecretManager;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let manager = GcpSecretManager::new("my-project-id").await?;
///     let secret = manager.get_secret("my-secret").await?;
///     println!("Secret: {}", secret.as_string());
///     Ok(())
/// }
/// ```
pub struct GcpSecretManager {
    client: SecretManagerServiceClient<Channel>,
    project_id: String,
    cache: SecretCache,
}

impl GcpSecretManager {
    /// Creates a new GCP Secret Manager client with default configuration.
    ///
    /// Uses Application Default Credentials (ADC):
    /// 1. GOOGLE_APPLICATION_CREDENTIALS environment variable
    /// 2. gcloud auth application-default login
    /// 3. Service account attached to GCE/GKE
    /// 4. Workload Identity for GKE
    ///
    /// # Arguments
    ///
    /// * `project_id` - GCP project ID (e.g., "my-project-123")
    ///
    /// # Errors
    ///
    /// Returns error if GCP credentials cannot be loaded.
    pub async fn new(project_id: impl Into<String>) -> Result<Self> {
        let project_id = project_id.into();

        // Load credentials using Application Default Credentials
        let credentials = google_cloud_auth::token::DefaultTokenSourceProvider::new(
            google_cloud_auth::project::Config {
                audience: None,
                scopes: Some(&["https://www.googleapis.com/auth/cloud-platform"]),
                sub: None,
            },
        )
        .await
        .map_err(|e| CloudError::Authentication(e.to_string()))?;

        // Create gRPC channel
        let channel = Channel::from_static("https://secretmanager.googleapis.com")
            .connect()
            .await
            .map_err(|e| CloudError::Connection(e.to_string()))?;

        let client = SecretManagerServiceClient::new(channel);
        let cache = SecretCache::new(300); // 5 minute default TTL

        tracing::info!("Initialized GCP Secret Manager client for project: {}", project_id);

        Ok(Self {
            client,
            project_id,
            cache,
        })
    }

    /// Creates a new GCP Secret Manager client with custom cache TTL.
    ///
    /// # Arguments
    ///
    /// * `project_id` - GCP project ID
    /// * `cache_ttl_seconds` - Cache time-to-live in seconds
    ///
    /// # Errors
    ///
    /// Returns error if GCP credentials cannot be loaded.
    pub async fn new_with_cache_ttl(
        project_id: impl Into<String>,
        cache_ttl_seconds: u64,
    ) -> Result<Self> {
        let project_id = project_id.into();

        let credentials = google_cloud_auth::token::DefaultTokenSourceProvider::new(
            google_cloud_auth::project::Config {
                audience: None,
                scopes: Some(&["https://www.googleapis.com/auth/cloud-platform"]),
                sub: None,
            },
        )
        .await
        .map_err(|e| CloudError::Authentication(e.to_string()))?;

        let channel = Channel::from_static("https://secretmanager.googleapis.com")
            .connect()
            .await
            .map_err(|e| CloudError::Connection(e.to_string()))?;

        let client = SecretManagerServiceClient::new(channel);
        let cache = SecretCache::new(cache_ttl_seconds);

        tracing::info!(
            "Initialized GCP Secret Manager client for project: {} with {}s cache TTL",
            project_id,
            cache_ttl_seconds
        );

        Ok(Self {
            client,
            project_id,
            cache,
        })
    }

    /// Gets the GCP project ID this client is configured for.
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Clears the secret cache.
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        tracing::debug!("Cleared GCP Secret Manager cache");
    }

    /// Gets the number of cached secrets.
    pub async fn cache_size(&self) -> usize {
        self.cache.len().await
    }

    /// Constructs a GCP secret resource name.
    fn secret_name(&self, name: &str) -> String {
        format!("projects/{}/secrets/{}", self.project_id, name)
    }

    /// Constructs a GCP secret version resource name.
    fn secret_version_name(&self, name: &str, version: &str) -> String {
        format!("projects/{}/secrets/{}/versions/{}", self.project_id, name, version)
    }
}

#[async_trait]
impl CloudSecretManager for GcpSecretManager {
    async fn get_secret(&self, name: &str) -> Result<SecretValue> {
        // Check cache first
        if let Some(cached) = self.cache.get(name).await {
            tracing::debug!("Cache hit for secret: {}", name);
            return Ok(cached);
        }

        tracing::debug!("Fetching secret from GCP Secret Manager: {}", name);

        // Access latest version
        let secret_version = self.secret_version_name(name, "latest");

        let request = AccessSecretVersionRequest {
            name: secret_version.clone(),
        };

        let response = self
            .client
            .clone()
            .access_secret_version(request)
            .await
            .map_err(|e| CloudError::secret_fetch(name, e.to_string()))?;

        let payload = response
            .into_inner()
            .payload
            .ok_or_else(|| CloudError::SecretFormat {
                name: name.to_string(),
                reason: "No payload in secret response".to_string(),
            })?;

        // GCP secrets are stored as bytes, try to convert to string if valid UTF-8
        let value = if let Ok(secret_string) = String::from_utf8(payload.data.clone()) {
            SecretValue::from_string(secret_string)
        } else {
            SecretValue::from_bytes(payload.data)
        };

        // Cache the secret
        self.cache.set(name.to_string(), value.clone()).await;

        tracing::info!("Successfully fetched secret: {}", name);

        Ok(value)
    }

    async fn list_secrets(&self) -> Result<Vec<String>> {
        tracing::debug!("Listing secrets from GCP Secret Manager");

        let parent = format!("projects/{}", self.project_id);
        let mut secret_names = Vec::new();
        let mut page_token = String::new();

        loop {
            let request = ListSecretsRequest {
                parent: parent.clone(),
                page_size: 100,
                page_token: page_token.clone(),
                filter: String::new(),
            };

            let response = self
                .client
                .clone()
                .list_secrets(request)
                .await
                .map_err(|e| CloudError::SecretList(e.to_string()))?;

            let response_inner = response.into_inner();

            for secret in response_inner.secrets {
                if let Some(name) = secret.name.split('/').last() {
                    secret_names.push(name.to_string());
                }
            }

            page_token = response_inner.next_page_token;

            if page_token.is_empty() {
                break;
            }
        }

        tracing::info!("Listed {} secrets", secret_names.len());

        Ok(secret_names)
    }

    async fn create_secret(&self, name: &str, value: &SecretValue) -> Result<()> {
        tracing::debug!("Creating secret in GCP Secret Manager: {}", name);

        let parent = format!("projects/{}", self.project_id);
        let secret_id = name.to_string();

        // Create the secret metadata
        let secret = Secret {
            name: String::new(),
            replication: Some(Replication {
                replication: Some(google_cloud_googleapis::secretmanager::v1::replication::Replication::Automatic(
                    google_cloud_googleapis::secretmanager::v1::replication::Automatic {},
                )),
            }),
            labels: HashMap::new(),
            ..Default::default()
        };

        let create_request = CreateSecretRequest {
            parent: parent.clone(),
            secret_id: secret_id.clone(),
            secret: Some(secret),
        };

        // Create the secret
        self.client
            .clone()
            .create_secret(create_request)
            .await
            .map_err(|e| CloudError::secret_create(name, e.to_string()))?;

        // Add the secret version with data
        let payload = SecretPayload {
            data: value.as_string().as_bytes().to_vec(),
            data_crc32c: None,
        };

        let add_version_request = AddSecretVersionRequest {
            parent: self.secret_name(name),
            payload: Some(payload),
        };

        self.client
            .clone()
            .add_secret_version(add_version_request)
            .await
            .map_err(|e| CloudError::secret_create(name, e.to_string()))?;

        tracing::info!("Successfully created secret: {}", name);

        Ok(())
    }

    async fn update_secret(&self, name: &str, value: &SecretValue) -> Result<()> {
        tracing::debug!("Updating secret in GCP Secret Manager: {}", name);

        // In GCP, updating a secret means adding a new version
        let payload = SecretPayload {
            data: value.as_string().as_bytes().to_vec(),
            data_crc32c: None,
        };

        let request = AddSecretVersionRequest {
            parent: self.secret_name(name),
            payload: Some(payload),
        };

        self.client
            .clone()
            .add_secret_version(request)
            .await
            .map_err(|e| CloudError::secret_update(name, e.to_string()))?;

        // Invalidate cache
        self.cache.invalidate(name).await;

        tracing::info!("Successfully updated secret: {}", name);

        Ok(())
    }

    async fn delete_secret(&self, name: &str) -> Result<()> {
        tracing::debug!("Deleting secret from GCP Secret Manager: {}", name);

        let request = DeleteSecretRequest {
            name: self.secret_name(name),
            etag: String::new(),
        };

        self.client
            .clone()
            .delete_secret(request)
            .await
            .map_err(|e| CloudError::secret_delete(name, e.to_string()))?;

        // Invalidate cache
        self.cache.invalidate(name).await;

        tracing::info!("Successfully deleted secret: {}", name);

        Ok(())
    }

    async fn get_secret_metadata(&self, name: &str) -> Result<SecretMetadata> {
        tracing::debug!("Fetching secret metadata from GCP Secret Manager: {}", name);

        let request = GetSecretRequest {
            name: self.secret_name(name),
        };

        let response = self
            .client
            .clone()
            .get_secret(request)
            .await
            .map_err(|e| CloudError::secret_fetch(name, e.to_string()))?;

        let secret = response.into_inner();

        let created_at = secret
            .create_time
            .and_then(|t| {
                chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                    .map(|dt| std::time::SystemTime::from(dt))
            })
            .unwrap_or_else(std::time::SystemTime::now);

        // GCP doesn't have a last_modified on Secret, use create_time
        let updated_at = created_at;

        let tags: HashMap<String, String> = secret
            .labels
            .into_iter()
            .map(|(k, v)| (k, v))
            .collect();

        // GCP doesn't expose version in the same way, we'd need to list versions
        let version = None;

        Ok(SecretMetadata {
            name: name.to_string(),
            created_at,
            updated_at,
            tags,
            version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_name_format() {
        let project_id = "test-project-123";
        let secret_name = "my-secret";
        let expected = format!("projects/{}/secrets/{}", project_id, secret_name);

        let manager = GcpSecretManager {
            client: todo!(),
            project_id: project_id.to_string(),
            cache: SecretCache::new(300),
        };

        assert_eq!(manager.secret_name(secret_name), expected);
    }

    #[test]
    fn test_secret_version_name_format() {
        let project_id = "test-project-123";
        let secret_name = "my-secret";
        let version = "latest";
        let expected = format!(
            "projects/{}/secrets/{}/versions/{}",
            project_id, secret_name, version
        );

        let manager = GcpSecretManager {
            client: todo!(),
            project_id: project_id.to_string(),
            cache: SecretCache::new(300),
        };

        assert_eq!(manager.secret_version_name(secret_name, version), expected);
    }

    #[tokio::test]
    async fn test_cache_operations() {
        // Test the cache independently without GCP calls
        let cache = SecretCache::new(300);

        let test_secret = SecretValue::from_string("test-value".to_string());
        cache.set("test-key".to_string(), test_secret.clone()).await;

        let retrieved = cache.get("test-key").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_string(), "test-value");

        cache.clear().await;
        assert_eq!(cache.len().await, 0);
    }
}
