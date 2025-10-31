//! API key storage backends
//!
//! ## SPARC Phase 3: Construction (TDD)
//!
//! Storage trait with multiple backend implementations.

use super::types::{ApiKey, Result};
use async_trait::async_trait;
use llm_shield_core::Error;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API key storage trait
///
/// ## Implementations
///
/// - `MemoryKeyStorage`: In-memory HashMap (testing/development)
/// - `FileKeyStorage`: JSON file persistence (production)
/// - `RedisKeyStorage`: Redis backend (optional, high-scale)
#[async_trait]
pub trait KeyStorage: Send + Sync {
    /// Store a new API key
    async fn store(&self, key: &ApiKey) -> Result<()>;

    /// Retrieve an API key by its hashed value
    async fn get_by_hash(&self, hashed_value: &str) -> Result<Option<ApiKey>>;

    /// Retrieve an API key by its ID
    async fn get_by_id(&self, id: &str) -> Result<Option<ApiKey>>;

    /// Delete an API key by ID
    async fn delete(&self, id: &str) -> Result<()>;

    /// List all keys
    async fn list(&self) -> Result<Vec<ApiKey>>;

    /// Update an existing key
    async fn update(&self, key: &ApiKey) -> Result<()>;
}

/// In-memory key storage (for testing/development)
///
/// ## Thread Safety
///
/// Uses `Arc<RwLock<HashMap>>` for thread-safe access.
///
/// ## Limitations
///
/// - Data lost on restart
/// - Not suitable for multi-instance deployments
/// - Use for testing or single-instance development only
pub struct MemoryKeyStorage {
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl MemoryKeyStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryKeyStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyStorage for MemoryKeyStorage {
    async fn store(&self, key: &ApiKey) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.insert(key.id.clone(), key.clone());
        Ok(())
    }

    async fn get_by_hash(&self, hashed_value: &str) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys
            .values()
            .find(|k| k.hashed_value == hashed_value)
            .cloned())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.get(id).cloned())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.remove(id);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().cloned().collect())
    }

    async fn update(&self, key: &ApiKey) -> Result<()> {
        let mut keys = self.keys.write().await;
        keys.insert(key.id.clone(), key.clone());
        Ok(())
    }
}

/// File-based key storage (JSON persistence)
///
/// ## Features
///
/// - Persists keys to JSON file
/// - Atomic writes with temp file + rename
/// - Thread-safe access
///
/// ## File Format
///
/// ```json
/// {
///   "keys": [
///     {
///       "id": "...",
///       "name": "...",
///       "hashed_value": "...",
///       ...
///     }
///   ]
/// }
/// ```
pub struct FileKeyStorage {
    file_path: PathBuf,
    keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct KeyFile {
    keys: Vec<ApiKey>,
}

impl FileKeyStorage {
    /// Create a new file-based storage
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the JSON file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let storage = FileKeyStorage::new("config/api_keys.json").await?;
    /// ```
    pub async fn new<P: Into<PathBuf>>(file_path: P) -> Result<Self> {
        let file_path = file_path.into();

        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| Error::auth(format!("Failed to create key directory: {}", e)))?;
        }

        let mut storage = Self {
            file_path,
            keys: Arc::new(RwLock::new(HashMap::new())),
        };

        // Load existing keys
        storage.load().await?;

        Ok(storage)
    }

    /// Load keys from file
    async fn load(&mut self) -> Result<()> {
        if !tokio::fs::try_exists(&self.file_path)
            .await
            .unwrap_or(false)
        {
            // File doesn't exist yet - start with empty storage
            return Ok(());
        }

        let contents = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| Error::auth(format!("Failed to read keys file: {}", e)))?;

        let key_file: KeyFile = serde_json::from_str(&contents)
            .map_err(|e| Error::auth(format!("Failed to parse keys file: {}", e)))?;

        let mut keys = self.keys.write().await;
        keys.clear();

        for key in key_file.keys {
            keys.insert(key.id.clone(), key);
        }

        Ok(())
    }

    /// Save keys to file
    async fn save(&self) -> Result<()> {
        let keys = self.keys.read().await;
        let key_file = KeyFile {
            keys: keys.values().cloned().collect(),
        };

        let contents = serde_json::to_string_pretty(&key_file)
            .map_err(|e| Error::auth(format!("Failed to serialize keys: {}", e)))?;

        // Write to temp file first
        let temp_path = self.file_path.with_extension("tmp");
        tokio::fs::write(&temp_path, contents)
            .await
            .map_err(|e| Error::auth(format!("Failed to write temp file: {}", e)))?;

        // Atomic rename
        tokio::fs::rename(&temp_path, &self.file_path)
            .await
            .map_err(|e| Error::auth(format!("Failed to rename temp file: {}", e)))?;

        Ok(())
    }
}

#[async_trait]
impl KeyStorage for FileKeyStorage {
    async fn store(&self, key: &ApiKey) -> Result<()> {
        {
            let mut keys = self.keys.write().await;
            keys.insert(key.id.clone(), key.clone());
        }
        self.save().await
    }

    async fn get_by_hash(&self, hashed_value: &str) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys
            .values()
            .find(|k| k.hashed_value == hashed_value)
            .cloned())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.get(id).cloned())
    }

    async fn delete(&self, id: &str) -> Result<()> {
        {
            let mut keys = self.keys.write().await;
            keys.remove(id);
        }
        self.save().await
    }

    async fn list(&self) -> Result<Vec<ApiKey>> {
        let keys = self.keys.read().await;
        Ok(keys.values().cloned().collect())
    }

    async fn update(&self, key: &ApiKey) -> Result<()> {
        {
            let mut keys = self.keys.write().await;
            keys.insert(key.id.clone(), key.clone());
        }
        self.save().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::rate_limit::RateLimitTier;

    fn create_test_key(name: &str) -> ApiKey {
        ApiKey::new(name.to_string(), RateLimitTier::Free, None).unwrap()
    }

    #[tokio::test]
    async fn test_memory_storage_new() {
        let storage = MemoryKeyStorage::new();
        let keys = storage.list().await.unwrap();
        assert_eq!(keys.len(), 0);
    }

    #[tokio::test]
    async fn test_memory_storage_store_and_get() {
        let storage = MemoryKeyStorage::new();
        let key = create_test_key("test-key");

        storage.store(&key).await.unwrap();

        let retrieved = storage.get_by_id(&key.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, key.id);
    }

    #[tokio::test]
    async fn test_memory_storage_get_by_hash() {
        let storage = MemoryKeyStorage::new();
        let key = create_test_key("test-key");
        let hash = key.hashed_value.clone();

        storage.store(&key).await.unwrap();

        let retrieved = storage.get_by_hash(&hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().hashed_value, hash);
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = MemoryKeyStorage::new();
        let key = create_test_key("test-key");

        storage.store(&key).await.unwrap();
        assert!(storage.get_by_id(&key.id).await.unwrap().is_some());

        storage.delete(&key.id).await.unwrap();
        assert!(storage.get_by_id(&key.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_memory_storage_list() {
        let storage = MemoryKeyStorage::new();

        storage.store(&create_test_key("key1")).await.unwrap();
        storage.store(&create_test_key("key2")).await.unwrap();
        storage.store(&create_test_key("key3")).await.unwrap();

        let keys = storage.list().await.unwrap();
        assert_eq!(keys.len(), 3);
    }

    #[tokio::test]
    async fn test_memory_storage_update() {
        let storage = MemoryKeyStorage::new();
        let mut key = create_test_key("test-key");

        storage.store(&key).await.unwrap();

        // Update key
        key.active = false;
        storage.update(&key).await.unwrap();

        let retrieved = storage.get_by_id(&key.id).await.unwrap().unwrap();
        assert!(!retrieved.active);
    }

    #[tokio::test]
    async fn test_file_storage_new() {
        let temp_file = std::env::temp_dir().join("test_keys_new.json");

        // Clean up if exists
        let _ = tokio::fs::remove_file(&temp_file).await;

        let storage = FileKeyStorage::new(&temp_file).await.unwrap();
        let keys = storage.list().await.unwrap();
        assert_eq!(keys.len(), 0);

        // Clean up
        let _ = tokio::fs::remove_file(&temp_file).await;
    }

    #[tokio::test]
    async fn test_file_storage_persistence() {
        let temp_file = std::env::temp_dir().join("test_keys_persist.json");

        // Clean up if exists
        let _ = tokio::fs::remove_file(&temp_file).await;

        // Create storage and add key
        {
            let storage = FileKeyStorage::new(&temp_file).await.unwrap();
            let key = create_test_key("test-key");
            storage.store(&key).await.unwrap();
        }

        // Load storage again and verify key exists
        {
            let storage = FileKeyStorage::new(&temp_file).await.unwrap();
            let keys = storage.list().await.unwrap();
            assert_eq!(keys.len(), 1);
            assert_eq!(keys[0].name, "test-key");
        }

        // Clean up
        let _ = tokio::fs::remove_file(&temp_file).await;
    }

    #[tokio::test]
    async fn test_file_storage_crud() {
        let temp_file = std::env::temp_dir().join("test_keys_crud.json");

        // Clean up if exists
        let _ = tokio::fs::remove_file(&temp_file).await;

        let storage = FileKeyStorage::new(&temp_file).await.unwrap();

        // Create
        let key = create_test_key("test-key");
        storage.store(&key).await.unwrap();

        // Read
        let retrieved = storage.get_by_id(&key.id).await.unwrap();
        assert!(retrieved.is_some());

        // Update
        let mut updated_key = retrieved.unwrap();
        updated_key.active = false;
        storage.update(&updated_key).await.unwrap();

        let retrieved = storage.get_by_id(&key.id).await.unwrap().unwrap();
        assert!(!retrieved.active);

        // Delete
        storage.delete(&key.id).await.unwrap();
        assert!(storage.get_by_id(&key.id).await.unwrap().is_none());

        // Clean up
        let _ = tokio::fs::remove_file(&temp_file).await;
    }
}
