//! Output scanner Python bindings.
//!
//! Output scanners analyze LLM responses before they are shown to users.

use pyo3::prelude::*;
use pyo3::types::PyDict;
use llm_shield_scanners::output::*;
use crate::{error::*, types::*, vault::PyVault, utils::*};

/// Macro for output scanners that need both prompt and output
macro_rules! output_scanner {
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

            /// Scan LLM output
            ///
            /// # Arguments
            ///
            /// * `prompt` - The user prompt
            /// * `output` - The LLM response
            /// * `vault` - Optional vault for state management
            pub fn scan_output(
                &self,
                py: Python<'_>,
                prompt: String,
                output: String,
                vault: Option<&PyVault>,
            ) -> PyResult<Py<PyDict>> {
                let vault_inner = get_or_create_vault(vault);
                let result = py.allow_threads(|| {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        self.inner.scan_output(&prompt, &output, &*vault_inner.inner).await
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

/// NoRefusal scanner - Detects refusal responses
///
/// This scanner identifies when an LLM refuses to answer a prompt,
/// which can indicate over-cautious safety measures.
///
/// # Examples
///
/// ```python
/// from llm_shield import NoRefusal, Vault
///
/// scanner = NoRefusal()
/// vault = Vault()
///
/// # Detect refusal
/// result = scanner.scan_output(
///     prompt="What is the weather?",
///     output="I cannot answer that question.",
///     vault=vault
/// )
/// assert result['is_valid'] == False
/// ```
output_scanner!(PyNoRefusal, NoRefusal, NoRefusalConfig);

/// Relevance scanner - Checks output relevance to prompt
output_scanner!(PyRelevance, Relevance, RelevanceConfig);

/// Sensitive scanner - Detects PII in outputs
#[pyclass(name = "Sensitive")]
pub struct PySensitive {
    pub(crate) inner: llm_shield_anonymize::Anonymizer,
}

#[pymethods]
impl PySensitive {
    #[new]
    pub fn new() -> PyResult<Self> {
        let config = llm_shield_anonymize::AnonymizerConfig::default();
        let scanner = llm_shield_anonymize::Anonymizer::new(config)
            .map_err(|e| PyErr::new::<ConfigError, _>(e.to_string()))?;
        Ok(Self { inner: scanner })
    }

    pub fn scan_output(
        &self,
        py: Python<'_>,
        prompt: String,
        output: String,
        vault: Option<&PyVault>,
    ) -> PyResult<Py<PyDict>> {
        let vault_inner = get_or_create_vault(vault);
        let result = py.allow_threads(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                self.inner.anonymize(&output).await
            })
        });

        // Convert anonymizer result to scan result format
        match result {
            Ok((sanitized, entities)) => {
                let is_valid = entities.is_empty();
                let risk_score = if is_valid { 0.0 } else { 0.9 };

                // Create scan result
                let scan_result = llm_shield_core::ScanResult {
                    sanitized_input: sanitized,
                    is_valid,
                    risk_score,
                    entities: entities.into_iter().map(|e| {
                        llm_shield_core::Entity {
                            entity_type: e.entity_type,
                            text: e.text,
                            start: e.start,
                            end: e.end,
                            score: e.score,
                            metadata: std::collections::HashMap::new(),
                        }
                    }).collect(),
                    risk_factors: vec![],
                };

                scan_result_to_py(py, &scan_result)
            }
            Err(e) => Err(PyErr::new::<ScannerError, _>(e.to_string())),
        }
    }

    pub fn name(&self) -> &str {
        "Sensitive"
    }

    pub fn __repr__(&self) -> String {
        "Sensitive()".to_string()
    }
}

/// BanTopics scanner - Detects banned topics in outputs
output_scanner!(PyBanTopics, BanTopics, BanTopicsConfig);

/// Bias scanner - Detects biased language
output_scanner!(PyBias, Bias, BiasConfig);

/// MaliciousURLs scanner - Detects malicious URLs
output_scanner!(PyMaliciousURLs, MaliciousURLs, MaliciousURLsConfig);

/// ReadingTime scanner - Estimates reading time
output_scanner!(PyReadingTime, ReadingTime, ReadingTimeConfig);

/// Factuality scanner - Checks factual accuracy
output_scanner!(PyFactuality, Factuality, FactualityConfig);

/// URLReachability scanner - Checks if URLs are reachable
output_scanner!(PyURLReachability, URLReachability, URLReachabilityConfig);

/// RegexOutput scanner - Pattern matching on outputs
output_scanner!(PyRegexOutput, RegexOutput, RegexOutputConfig);
