//! Vault wrapper for thread-safe state management.
//!
//! The Vault provides thread-safe storage for sharing state between scanners
//! during a scanning session.

use pyo3::prelude::*;
use llm_shield_core::Vault;
use std::sync::Arc;

/// Thread-safe state storage for scanners
///
/// The Vault provides a thread-safe key-value store that can be shared
/// between multiple scanners during a scanning session. This is useful
/// for maintaining context and sharing detected patterns.
///
/// # Examples
///
/// ```python
/// from llm_shield import Vault
///
/// vault = Vault()
/// vault.set("key", "value")
/// assert vault.get("key") == "value"
/// assert vault.contains("key") == True
/// ```
#[pyclass(name = "Vault")]
pub struct PyVault {
    /// Internal Arc-wrapped Vault for thread-safe sharing
    pub(crate) inner: Arc<Vault>,
}

#[pymethods]
impl PyVault {
    /// Create a new Vault
    ///
    /// # Returns
    ///
    /// A new empty Vault instance
    #[new]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Vault::new()),
        }
    }

    /// Store a value in the vault
    ///
    /// # Arguments
    ///
    /// * `key` - The key to store the value under
    /// * `value` - The value to store (must be a string)
    ///
    /// # Examples
    ///
    /// ```python
    /// vault = Vault()
    /// vault.set("pattern_detected", "true")
    /// ```
    pub fn set(&self, key: String, value: String) -> PyResult<()> {
        self.inner.set(key, value);
        Ok(())
    }

    /// Retrieve a value from the vault
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve
    ///
    /// # Returns
    ///
    /// The stored value, or None if the key doesn't exist
    ///
    /// # Examples
    ///
    /// ```python
    /// vault = Vault()
    /// vault.set("key", "value")
    /// assert vault.get("key") == "value"
    /// assert vault.get("nonexistent") is None
    /// ```
    pub fn get(&self, key: String) -> PyResult<Option<String>> {
        Ok(self.inner.get(&key))
    }

    /// Check if a key exists in the vault
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check
    ///
    /// # Returns
    ///
    /// True if the key exists, False otherwise
    ///
    /// # Examples
    ///
    /// ```python
    /// vault = Vault()
    /// vault.set("key", "value")
    /// assert vault.contains("key") == True
    /// assert vault.contains("nonexistent") == False
    /// ```
    pub fn contains(&self, key: String) -> PyResult<bool> {
        Ok(self.inner.contains(&key))
    }

    /// Remove a value from the vault
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove
    ///
    /// # Examples
    ///
    /// ```python
    /// vault = Vault()
    /// vault.set("key", "value")
    /// vault.remove("key")
    /// assert vault.contains("key") == False
    /// ```
    pub fn remove(&self, key: String) -> PyResult<()> {
        self.inner.remove(&key);
        Ok(())
    }

    /// Clear all values from the vault
    ///
    /// # Examples
    ///
    /// ```python
    /// vault = Vault()
    /// vault.set("key1", "value1")
    /// vault.set("key2", "value2")
    /// vault.clear()
    /// assert vault.contains("key1") == False
    /// ```
    pub fn clear(&self) -> PyResult<()> {
        self.inner.clear();
        Ok(())
    }

    /// Get all keys in the vault
    ///
    /// # Returns
    ///
    /// A list of all keys currently stored in the vault
    ///
    /// # Examples
    ///
    /// ```python
    /// vault = Vault()
    /// vault.set("key1", "value1")
    /// vault.set("key2", "value2")
    /// keys = vault.keys()
    /// assert "key1" in keys and "key2" in keys
    /// ```
    pub fn keys(&self) -> PyResult<Vec<String>> {
        Ok(self.inner.keys())
    }

    /// Get the number of entries in the vault
    ///
    /// # Returns
    ///
    /// The number of key-value pairs in the vault
    pub fn __len__(&self) -> PyResult<usize> {
        Ok(self.inner.len())
    }

    /// Check if key exists (supports 'in' operator)
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check
    ///
    /// # Returns
    ///
    /// True if the key exists
    pub fn __contains__(&self, key: String) -> PyResult<bool> {
        Ok(self.inner.contains(&key))
    }

    /// String representation of the vault
    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Vault(entries={})", self.inner.len()))
    }
}

/// Clone implementation for Vault (creates new Arc reference)
impl Clone for PyVault {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_basic_operations() {
        Python::with_gil(|_py| {
            let vault = PyVault::new();

            // Test set and get
            vault.set("key1".to_string(), "value1".to_string()).unwrap();
            assert_eq!(
                vault.get("key1".to_string()).unwrap(),
                Some("value1".to_string())
            );

            // Test contains
            assert!(vault.contains("key1".to_string()).unwrap());
            assert!(!vault.contains("nonexistent".to_string()).unwrap());

            // Test remove
            vault.remove("key1".to_string()).unwrap();
            assert!(!vault.contains("key1".to_string()).unwrap());
        });
    }

    #[test]
    fn test_vault_clear() {
        Python::with_gil(|_py| {
            let vault = PyVault::new();

            vault.set("key1".to_string(), "value1".to_string()).unwrap();
            vault.set("key2".to_string(), "value2".to_string()).unwrap();

            vault.clear().unwrap();

            assert!(!vault.contains("key1".to_string()).unwrap());
            assert!(!vault.contains("key2".to_string()).unwrap());
        });
    }

    #[test]
    fn test_vault_keys() {
        Python::with_gil(|_py| {
            let vault = PyVault::new();

            vault.set("key1".to_string(), "value1".to_string()).unwrap();
            vault.set("key2".to_string(), "value2".to_string()).unwrap();

            let keys = vault.keys().unwrap();
            assert!(keys.contains(&"key1".to_string()));
            assert!(keys.contains(&"key2".to_string()));
        });
    }
}
