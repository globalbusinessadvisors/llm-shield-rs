//! Utility functions and helpers for Python bindings.

use pyo3::prelude::*;
use crate::vault::PyVault;

/// Get or create a vault
///
/// If vault is None, creates a new temporary vault.
/// Otherwise returns a clone of the provided vault.
pub fn get_or_create_vault(vault: Option<&PyVault>) -> PyVault {
    match vault {
        Some(v) => v.clone(),
        None => PyVault::new(),
    }
}

/// Macro for implementing common scanner methods
///
/// This macro reduces boilerplate by implementing common methods
/// for all scanner types (scan, scan_async, etc.)
#[macro_export]
macro_rules! impl_scanner_methods {
    ($scanner_type:ty, $rust_scanner:ty) => {
        #[pymethods]
        impl $scanner_type {
            /// Scan text synchronously
            ///
            /// # Arguments
            ///
            /// * `text` - The text to scan
            /// * `vault` - Optional vault for state management
            ///
            /// # Returns
            ///
            /// A dictionary with scan results
            pub fn scan(
                &self,
                py: Python<'_>,
                text: String,
                vault: Option<&PyVault>,
            ) -> PyResult<Py<PyDict>> {
                let vault_inner = $crate::utils::get_or_create_vault(vault);

                // Release GIL during Rust computation
                let result = py.allow_threads(|| {
                    // Create tokio runtime for async scanner
                    let rt = tokio::runtime::Runtime::new()
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

                    rt.block_on(async {
                        self.inner
                            .scan(&text, &*vault_inner.inner)
                            .await
                    })
                });

                // Convert result to Python
                match result {
                    Ok(r) => $crate::types::scan_result_to_py(py, &r),
                    Err(e) => Err($crate::error::convert_error(e)),
                }
            }

            /// Get scanner name
            pub fn name(&self) -> &str {
                self.inner.name()
            }

            /// String representation
            pub fn __repr__(&self) -> String {
                format!("{}()", stringify!($scanner_type))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_create_vault() {
        Python::with_gil(|_py| {
            // Test with None
            let vault = get_or_create_vault(None);
            assert_eq!(vault.inner.len(), 0);

            // Test with Some
            let existing_vault = PyVault::new();
            existing_vault.set("key".to_string(), "value".to_string()).unwrap();
            let vault = get_or_create_vault(Some(&existing_vault));
            assert!(vault.contains("key".to_string()).unwrap());
        });
    }
}
