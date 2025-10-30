//! Vault for cross-scanner state management
//!
//! ## SPARC Specification
//!
//! Provides thread-safe state storage for:
//! - Anonymization mappings
//! - Session context
//! - Cross-scanner communication

use crate::Error;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Thread-safe state storage for scanners
///
/// ## Enterprise Design
///
/// - Thread-safe with RwLock
/// - Type-safe value storage
/// - Namespaced keys
/// - Clone-friendly (Arc<RwLock>)
#[derive(Clone)]
pub struct Vault {
    data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

impl Vault {
    /// Create a new vault
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a value in the vault
    pub fn set<K: Into<String>, V: serde::Serialize>(&self, key: K, value: V) -> Result<(), Error> {
        let json_value = serde_json::to_value(value)
            .map_err(|e| Error::vault(format!("Failed to serialize value: {}", e)))?;

        self.data
            .write()
            .map_err(|e| Error::vault(format!("Failed to acquire write lock: {}", e)))?
            .insert(key.into(), json_value);

        Ok(())
    }

    /// Get a value from the vault
    pub fn get<K: AsRef<str>, V: for<'de> serde::Deserialize<'de>>(
        &self,
        key: K,
    ) -> Result<Option<V>, Error> {
        let data = self
            .data
            .read()
            .map_err(|e| Error::vault(format!("Failed to acquire read lock: {}", e)))?;

        match data.get(key.as_ref()) {
            Some(value) => {
                let typed_value = serde_json::from_value(value.clone())
                    .map_err(|e| Error::vault(format!("Failed to deserialize value: {}", e)))?;
                Ok(Some(typed_value))
            }
            None => Ok(None),
        }
    }

    /// Check if a key exists
    pub fn contains_key<K: AsRef<str>>(&self, key: K) -> bool {
        self.data
            .read()
            .map(|data| data.contains_key(key.as_ref()))
            .unwrap_or(false)
    }

    /// Remove a value from the vault
    pub fn remove<K: AsRef<str>>(&self, key: K) -> Result<(), Error> {
        self.data
            .write()
            .map_err(|e| Error::vault(format!("Failed to acquire write lock: {}", e)))?
            .remove(key.as_ref());

        Ok(())
    }

    /// Clear all values
    pub fn clear(&self) -> Result<(), Error> {
        self.data
            .write()
            .map_err(|e| Error::vault(format!("Failed to acquire write lock: {}", e)))?
            .clear();

        Ok(())
    }

    /// Get all keys
    pub fn keys(&self) -> Result<Vec<String>, Error> {
        let data = self
            .data
            .read()
            .map_err(|e| Error::vault(format!("Failed to acquire read lock: {}", e)))?;

        Ok(data.keys().cloned().collect())
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.data.read().map(|data| data.len()).unwrap_or(0)
    }

    /// Check if vault is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_basic_operations() {
        let vault = Vault::new();

        // Test set and get
        vault.set("key1", "value1").unwrap();
        assert_eq!(vault.get::<_, String>("key1").unwrap(), Some("value1".to_string()));

        // Test contains_key
        assert!(vault.contains_key("key1"));
        assert!(!vault.contains_key("key2"));

        // Test remove
        vault.remove("key1").unwrap();
        assert!(!vault.contains_key("key1"));
    }

    #[test]
    fn test_vault_typed_values() {
        let vault = Vault::new();

        vault.set("int", 42i32).unwrap();
        vault.set("float", 3.14f64).unwrap();
        vault.set("bool", true).unwrap();
        vault.set("string", "hello").unwrap();

        assert_eq!(vault.get::<_, i32>("int").unwrap(), Some(42));
        assert_eq!(vault.get::<_, f64>("float").unwrap(), Some(3.14));
        assert_eq!(vault.get::<_, bool>("bool").unwrap(), Some(true));
        assert_eq!(vault.get::<_, String>("string").unwrap(), Some("hello".to_string()));
    }

    #[test]
    fn test_vault_clear() {
        let vault = Vault::new();

        vault.set("key1", "value1").unwrap();
        vault.set("key2", "value2").unwrap();

        assert_eq!(vault.len(), 2);

        vault.clear().unwrap();

        assert_eq!(vault.len(), 0);
        assert!(vault.is_empty());
    }

    #[test]
    fn test_vault_clone() {
        let vault1 = Vault::new();
        vault1.set("key", "value").unwrap();

        let vault2 = vault1.clone();
        assert_eq!(vault2.get::<_, String>("key").unwrap(), Some("value".to_string()));

        // Both vaults share the same underlying data
        vault2.set("key2", "value2").unwrap();
        assert!(vault1.contains_key("key2"));
    }
}
