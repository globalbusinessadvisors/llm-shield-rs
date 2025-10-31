//! Input scanner Python bindings.
//!
//! Input scanners analyze user prompts before they are sent to the LLM.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use llm_shield_scanners::input::*;
use crate::{error::*, types::*, vault::PyVault, utils::*};

/// BanSubstrings scanner - Detects and removes banned substrings
///
/// This scanner checks for forbidden substrings in user input and can
/// optionally redact them from the output.
///
/// # Examples
///
/// ```python
/// from llm_shield import BanSubstrings, Vault
///
/// # Create scanner
/// scanner = BanSubstrings(substrings=["banned", "forbidden"])
///
/// # Scan text
/// vault = Vault()
/// result = scanner.scan("This text is clean", vault)
/// assert result['is_valid'] == True
///
/// # Detect banned content
/// result = scanner.scan("This contains banned word", vault)
/// assert result['is_valid'] == False
/// ```
#[pyclass(name = "BanSubstrings")]
pub struct PyBanSubstrings {
    pub(crate) inner: BanSubstrings,
}

#[pymethods]
impl PyBanSubstrings {
    /// Create a new BanSubstrings scanner
    ///
    /// # Arguments
    ///
    /// * `substrings` - List of substrings to ban
    /// * `case_sensitive` - Whether matching should be case-sensitive (default: false)
    /// * `redact` - Whether to redact banned substrings (default: true)
    /// * `match_type` - Type of matching: "str" or "word" (default: "str")
    ///
    /// # Returns
    ///
    /// A new BanSubstrings scanner instance
    ///
    /// # Raises
    ///
    /// * `ConfigError` - If configuration is invalid
    #[new]
    #[pyo3(signature = (substrings, case_sensitive=false, redact=true, match_type="str"))]
    pub fn new(
        substrings: Vec<String>,
        case_sensitive: bool,
        redact: bool,
        match_type: &str,
    ) -> PyResult<Self> {
        // Validate inputs
        if substrings.is_empty() {
            return Err(PyErr::new::<ConfigError, _>(
                "substrings list cannot be empty"
            ));
        }

        // Parse match type
        let match_type_enum = match match_type {
            "str" => MatchType::String,
            "word" => MatchType::Word,
            _ => return Err(PyErr::new::<ConfigError, _>(
                format!("Invalid match_type: {}. Must be 'str' or 'word'", match_type)
            )),
        };

        // Create config
        let config = BanSubstringsConfig {
            substrings,
            case_sensitive,
            redact,
            match_type: match_type_enum,
        };

        // Initialize scanner
        let scanner = BanSubstrings::new(config)
            .map_err(|e| convert_error(e))?;

        Ok(Self { inner: scanner })
    }

    /// Scan text for banned substrings
    ///
    /// # Arguments
    ///
    /// * `text` - The text to scan
    /// * `vault` - Optional vault for state management
    ///
    /// # Returns
    ///
    /// Dictionary with scan results:
    /// - `sanitized_input`: Redacted text (if redact=True)
    /// - `is_valid`: False if banned substring found
    /// - `risk_score`: Risk level (0.0-1.0)
    /// - `entities`: List of detected banned substrings
    pub fn scan(
        &self,
        py: Python<'_>,
        text: String,
        vault: Option<&PyVault>,
    ) -> PyResult<Py<PyDict>> {
        let vault_inner = get_or_create_vault(vault);

        // Release GIL during Rust computation
        let result = py.allow_threads(|| {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            rt.block_on(async {
                self.inner.scan(&text, &*vault_inner.inner).await
            })
        });

        // Convert result to Python
        match result {
            Ok(r) => scan_result_to_py(py, &r),
            Err(e) => Err(convert_error(e)),
        }
    }

    /// Get scanner name
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// String representation
    pub fn __repr__(&self) -> String {
        format!("BanSubstrings(substrings={})", self.inner.config().substrings.len())
    }
}

/// Secrets scanner - Detects secrets and credentials
///
/// This scanner identifies various types of secrets including API keys,
/// passwords, tokens, and other credentials using regex patterns and
/// entropy analysis.
///
/// # Examples
///
/// ```python
/// from llm_shield import Secrets, Vault
///
/// scanner = Secrets(redact=True)
/// vault = Vault()
///
/// # Detect secret
/// result = scanner.scan("API key: sk-proj-abc123", vault)
/// assert result['is_valid'] == False
/// ```
#[pyclass(name = "Secrets")]
pub struct PySecrets {
    pub(crate) inner: llm_shield_secrets::Secrets,
}

#[pymethods]
impl PySecrets {
    /// Create a new Secrets scanner
    ///
    /// # Arguments
    ///
    /// * `redact` - Whether to redact detected secrets (default: true)
    #[new]
    #[pyo3(signature = (redact=true))]
    pub fn new(redact: bool) -> PyResult<Self> {
        let config = llm_shield_secrets::SecretsConfig { redact };

        let scanner = llm_shield_secrets::Secrets::new(config)
            .map_err(|e| PyErr::new::<ConfigError, _>(e.to_string()))?;

        Ok(Self { inner: scanner })
    }

    /// Scan text for secrets
    pub fn scan(
        &self,
        py: Python<'_>,
        text: String,
        vault: Option<&PyVault>,
    ) -> PyResult<Py<PyDict>> {
        let vault_inner = get_or_create_vault(vault);

        let result = py.allow_threads(|| {
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            rt.block_on(async {
                self.inner.scan(&text, &*vault_inner.inner).await
            })
        });

        match result {
            Ok(r) => scan_result_to_py(py, &r),
            Err(e) => Err(PyErr::new::<ScannerError, _>(e.to_string())),
        }
    }

    pub fn name(&self) -> &str {
        "Secrets"
    }

    pub fn __repr__(&self) -> String {
        "Secrets()".to_string()
    }
}

// Placeholder implementations for remaining input scanners
// These will follow the same pattern as BanSubstrings and Secrets

#[pyclass(name = "PromptInjection")]
pub struct PyPromptInjection {
    pub(crate) inner: PromptInjection,
}

#[pymethods]
impl PyPromptInjection {
    #[new]
    pub fn new() -> PyResult<Self> {
        let config = PromptInjectionConfig::default();
        let scanner = PromptInjection::new(config)
            .map_err(|e| convert_error(e))?;
        Ok(Self { inner: scanner })
    }

    pub fn scan(
        &self,
        py: Python<'_>,
        text: String,
        vault: Option<&PyVault>,
    ) -> PyResult<Py<PyDict>> {
        let vault_inner = get_or_create_vault(vault);
        let result = py.allow_threads(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                self.inner.scan(&text, &*vault_inner.inner).await
            })
        });
        match result {
            Ok(r) => scan_result_to_py(py, &r),
            Err(e) => Err(convert_error(e)),
        }
    }

    pub fn name(&self) -> &str {
        "PromptInjection"
    }

    pub fn __repr__(&self) -> String {
        "PromptInjection()".to_string()
    }
}

#[pyclass(name = "Toxicity")]
pub struct PyToxicity {
    pub(crate) inner: Toxicity,
}

#[pymethods]
impl PyToxicity {
    #[new]
    pub fn new() -> PyResult<Self> {
        let config = ToxicityConfig::default();
        let scanner = Toxicity::new(config)
            .map_err(|e| convert_error(e))?;
        Ok(Self { inner: scanner })
    }

    pub fn scan(
        &self,
        py: Python<'_>,
        text: String,
        vault: Option<&PyVault>,
    ) -> PyResult<Py<PyDict>> {
        let vault_inner = get_or_create_vault(vault);
        let result = py.allow_threads(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                self.inner.scan(&text, &*vault_inner.inner).await
            })
        });
        match result {
            Ok(r) => scan_result_to_py(py, &r),
            Err(e) => Err(convert_error(e)),
        }
    }

    pub fn name(&self) -> &str {
        "Toxicity"
    }

    pub fn __repr__(&self) -> String {
        "Toxicity()".to_string()
    }
}

// Additional placeholder scanners (simplified for now)
macro_rules! simple_scanner {
    ($name:ident, $rust_type:ty, $config_type:ty) => {
        #[pyclass(name = stringify!($name))]
        pub struct $name {
            pub(crate) inner: $rust_type,
        }

        #[pymethods]
        impl $name {
            #[new]
            pub fn new() -> PyResult<Self> {
                let config = <$config_type>::default();
                let scanner = <$rust_type>::new(config)
                    .map_err(|e| convert_error(e))?;
                Ok(Self { inner: scanner })
            }

            pub fn scan(
                &self,
                py: Python<'_>,
                text: String,
                vault: Option<&PyVault>,
            ) -> PyResult<Py<PyDict>> {
                let vault_inner = get_or_create_vault(vault);
                let result = py.allow_threads(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        self.inner.scan(&text, &*vault_inner.inner).await
                    })
                });
                match result {
                    Ok(r) => scan_result_to_py(py, &r),
                    Err(e) => Err(convert_error(e)),
                }
            }

            pub fn name(&self) -> &str {
                stringify!($name)
            }

            pub fn __repr__(&self) -> String {
                format!("{}()", stringify!($name))
            }
        }
    };
}

// Generate remaining input scanners
simple_scanner!(PyGibberish, Gibberish, GibberishConfig);
simple_scanner!(PyInvisibleText, InvisibleText, InvisibleTextConfig);
simple_scanner!(PyLanguage, Language, LanguageConfig);
simple_scanner!(PyTokenLimit, TokenLimit, TokenLimitConfig);
simple_scanner!(PyBanCompetitors, BanCompetitors, BanCompetitorsConfig);
simple_scanner!(PySentiment, Sentiment, SentimentConfig);
simple_scanner!(PyBanCode, BanCode, BanCodeConfig);
simple_scanner!(PyRegex, Regex, RegexConfig);
