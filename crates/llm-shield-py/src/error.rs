//! Error handling and conversion between Rust and Python exceptions.
//!
//! This module provides comprehensive error handling that maps Rust errors
//! to appropriate Python exceptions with rich context and error messages.

use pyo3::{exceptions as exc, prelude::*};
use llm_shield_core::Error as CoreError;

/// Base exception for all LLM Shield errors
///
/// This is the parent exception for all custom exceptions in the library.
/// It inherits from Python's built-in `Exception` class.
#[pyclass(extends = exc::PyException)]
pub struct LLMShieldError;

/// Scanner-specific errors
///
/// Raised when a scanner encounters an error during initialization or execution.
///
/// # Attributes
///
/// * `scanner` - The name of the scanner that encountered the error
/// * `message` - Detailed error message
#[pyclass(extends = LLMShieldError)]
pub struct ScannerError;

/// Model loading or inference errors
///
/// Raised when ML model operations fail (loading, inference, etc.)
#[pyclass(extends = LLMShieldError)]
pub struct ModelError;

/// Configuration errors
///
/// Raised when scanner configuration is invalid
#[pyclass(extends = LLMShieldError)]
pub struct ConfigError;

/// Vault errors
///
/// Raised when vault operations fail
#[pyclass(extends = LLMShieldError)]
pub struct VaultError;

/// Operation timeout errors
///
/// Raised when an operation exceeds its time limit
///
/// # Attributes
///
/// * `duration_ms` - The timeout duration in milliseconds
#[pyclass(extends = LLMShieldError)]
pub struct TimeoutError;

/// Register all custom exception types with the Python module
pub fn register_exceptions(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Create base exception
    let base_exception = py.get_type_bound::<exc::PyException>();
    let llm_shield_error = PyClassInitializer::from(exc::PyException)
        .add_subclass(LLMShieldError);
    let llm_shield_error_type = m.py().get_type_bound::<LLMShieldError>();

    m.add("LLMShieldError", llm_shield_error_type)?;

    // Create specific exceptions
    m.add("ScannerError", py.get_type_bound::<ScannerError>())?;
    m.add("ModelError", py.get_type_bound::<ModelError>())?;
    m.add("ConfigError", py.get_type_bound::<ConfigError>())?;
    m.add("VaultError", py.get_type_bound::<VaultError>())?;
    m.add("TimeoutError", py.get_type_bound::<TimeoutError>())?;

    Ok(())
}

/// Convert a Rust CoreError to a Python exception
///
/// This function maps Rust error types to appropriate Python exceptions
/// with context-rich error messages.
pub fn convert_error(err: CoreError) -> PyErr {
    match err {
        CoreError::Scanner { scanner, message, .. } => {
            PyErr::new::<ScannerError, _>(format!("Scanner '{}': {}", scanner, message))
        }
        CoreError::Model(msg) => {
            PyErr::new::<ModelError, _>(msg)
        }
        CoreError::Config(msg) => {
            PyErr::new::<ConfigError, _>(msg)
        }
        CoreError::InvalidInput(msg) => {
            exc::PyValueError::new_err(msg)
        }
        CoreError::Io(e) => {
            exc::PyIOError::new_err(e.to_string())
        }
        CoreError::Vault(msg) => {
            PyErr::new::<VaultError, _>(msg)
        }
        CoreError::Timeout(duration) => {
            PyErr::new::<TimeoutError, _>(format!("Operation timed out after {}ms", duration))
        }
        _ => {
            exc::PyRuntimeError::new_err(err.to_string())
        }
    }
}

/// Helper trait for converting Results with Rust errors to PyResult
pub trait ToPyResult<T> {
    fn to_py_result(self) -> PyResult<T>;
}

impl<T> ToPyResult<T> for Result<T, CoreError> {
    fn to_py_result(self) -> PyResult<T> {
        self.map_err(convert_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        Python::with_gil(|py| {
            // Test scanner error
            let err = CoreError::Scanner {
                scanner: "TestScanner".to_string(),
                message: "Test error".to_string(),
                source: None,
            };
            let py_err = convert_error(err);
            assert!(py_err.to_string().contains("TestScanner"));

            // Test config error
            let err = CoreError::Config("Invalid config".to_string());
            let py_err = convert_error(err);
            assert!(py_err.to_string().contains("Invalid config"));

            // Test model error
            let err = CoreError::Model("Model not found".to_string());
            let py_err = convert_error(err);
            assert!(py_err.to_string().contains("Model not found"));
        });
    }
}
