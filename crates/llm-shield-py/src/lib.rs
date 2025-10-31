//! LLM Shield Python Bindings
//!
//! Enterprise-grade LLM security toolkit - Python bindings for Rust implementation.
//!
//! This crate provides Python bindings for the LLM Shield security scanners,
//! enabling Python developers to leverage high-performance Rust implementations
//! with 10-100x performance improvements over pure Python.

use pyo3::prelude::*;

mod error;
mod types;
mod vault;
mod scanners;
mod utils;

use error::*;
use types::*;
use vault::PyVault;
use scanners::input::*;
use scanners::output::*;

/// LLM Shield - Enterprise-grade LLM security toolkit
///
/// This module provides high-performance security scanners for Large Language Models,
/// implemented in Rust with Python bindings.
///
/// # Examples
///
/// ```python
/// from llm_shield import BanSubstrings, Vault
///
/// # Create a scanner
/// scanner = BanSubstrings(substrings=["banned", "forbidden"])
///
/// # Scan text
/// vault = Vault()
/// result = scanner.scan("This text is clean", vault)
///
/// print(f"Valid: {result['is_valid']}, Risk: {result['risk_score']}")
/// ```
#[pymodule]
fn llm_shield(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "Enterprise-grade LLM security toolkit")?;

    // Register custom exceptions
    register_exceptions(py, m)?;

    // Register core types
    m.add_class::<PyVault>()?;

    // Register input scanners
    m.add_class::<PyBanSubstrings>()?;
    m.add_class::<PySecrets>()?;
    m.add_class::<PyPromptInjection>()?;
    m.add_class::<PyToxicity>()?;
    m.add_class::<PyGibberish>()?;
    m.add_class::<PyInvisibleText>()?;
    m.add_class::<PyLanguage>()?;
    m.add_class::<PyTokenLimit>()?;
    m.add_class::<PyBanCompetitors>()?;
    m.add_class::<PySentiment>()?;
    m.add_class::<PyBanCode>()?;
    m.add_class::<PyRegex>()?;

    // Register output scanners
    m.add_class::<PyNoRefusal>()?;
    m.add_class::<PyRelevance>()?;
    m.add_class::<PySensitive>()?;
    m.add_class::<PyBanTopics>()?;
    m.add_class::<PyBias>()?;
    m.add_class::<PyMaliciousURLs>()?;
    m.add_class::<PyReadingTime>()?;
    m.add_class::<PyFactuality>()?;
    m.add_class::<PyURLReachability>()?;
    m.add_class::<PyRegexOutput>()?;

    // Register utility functions
    m.add_function(wrap_pyfunction!(create_vault, m)?)?;

    Ok(())
}

/// Create a new Vault for state management
///
/// # Examples
///
/// ```python
/// from llm_shield import create_vault
///
/// vault = create_vault()
/// vault.set("key", "value")
/// ```
#[pyfunction]
fn create_vault() -> PyResult<PyVault> {
    Ok(PyVault::new())
}
