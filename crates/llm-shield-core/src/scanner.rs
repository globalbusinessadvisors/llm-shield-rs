//! Scanner trait and types
//!
//! ## SPARC Specification - Scanner Abstraction
//!
//! Core trait that all security scanners must implement:
//! - Async-first design for scalability
//! - Composable architecture
//! - Type-safe configuration
//! - Observability built-in

use crate::{Error, Result, ScanResult, Vault};
use async_trait::async_trait;
use std::sync::Arc;

/// Core scanner trait
///
/// All security scanners implement this trait to provide consistent interface.
///
/// ## London School TDD Design
///
/// This trait is designed to be mockable for testing:
/// - Pure async interface
/// - No internal state mutations
/// - Testable with mock vault
///
/// ## Example
///
/// ```rust,ignore
/// use llm_shield_core::{Scanner, ScanResult, Vault};
/// use async_trait::async_trait;
///
/// struct MyScanner;
///
/// #[async_trait]
/// impl Scanner for MyScanner {
///     fn name(&self) -> &str {
///         "my_scanner"
///     }
///
///     async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
///         // Implement scanning logic
///         Ok(ScanResult::pass(input.to_string()))
///     }
///
///     fn scanner_type(&self) -> ScannerType {
///         ScannerType::Input
///     }
/// }
/// ```
#[async_trait]
pub trait Scanner: Send + Sync {
    /// Scanner name for identification
    fn name(&self) -> &str;

    /// Scan input text and return result
    ///
    /// ## Parameters
    ///
    /// - `input`: Text to scan
    /// - `vault`: State storage for cross-scanner communication
    ///
    /// ## Returns
    ///
    /// - `Ok(ScanResult)`: Scan completed successfully
    /// - `Err(Error)`: Scan failed
    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult>;

    /// Type of scanner (input/output/bidirectional)
    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }

    /// Scanner version
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// Scanner description
    fn description(&self) -> &str {
        "No description provided"
    }

    /// Whether this scanner requires async execution
    ///
    /// Some scanners (e.g., URL checking) must be async.
    /// Simple scanners can be sync for better performance.
    fn requires_async(&self) -> bool {
        false
    }

    /// Validate scanner configuration
    fn validate_config(&self) -> Result<()> {
        Ok(())
    }
}

/// Input scanner specialization
///
/// Scans LLM prompts/inputs before they're sent to the model
#[async_trait]
pub trait InputScanner: Scanner {
    /// Scan a prompt before sending to LLM
    async fn scan_prompt(&self, prompt: &str, vault: &Vault) -> Result<ScanResult> {
        self.scan(prompt, vault).await
    }
}

/// Output scanner specialization
///
/// Scans LLM responses/outputs before returning to user
#[async_trait]
pub trait OutputScanner: Scanner {
    /// Scan LLM output with context of original prompt
    async fn scan_output(
        &self,
        prompt: &str,
        output: &str,
        vault: &Vault,
    ) -> Result<ScanResult>;
}

/// Scanner type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScannerType {
    /// Scans inputs (prompts)
    Input,
    /// Scans outputs (responses)
    Output,
    /// Scans both inputs and outputs
    Bidirectional,
}

/// Scanner pipeline for composing multiple scanners
///
/// ## Enterprise Pattern
///
/// Provides:
/// - Sequential execution
/// - Parallel execution option
/// - Short-circuit on high risk
/// - Result aggregation
pub struct ScannerPipeline {
    scanners: Vec<Arc<dyn Scanner>>,
    short_circuit: bool,
    short_circuit_threshold: f32,
}

impl ScannerPipeline {
    /// Create a new scanner pipeline
    pub fn new() -> Self {
        Self {
            scanners: Vec::new(),
            short_circuit: false,
            short_circuit_threshold: 0.9,
        }
    }

    /// Add a scanner to the pipeline
    pub fn add(mut self, scanner: Arc<dyn Scanner>) -> Self {
        self.scanners.push(scanner);
        self
    }

    /// Enable short-circuit evaluation
    ///
    /// If any scanner returns risk >= threshold, stop execution
    pub fn with_short_circuit(mut self, threshold: f32) -> Self {
        self.short_circuit = true;
        self.short_circuit_threshold = threshold;
        self
    }

    /// Execute pipeline sequentially
    pub async fn execute(&self, input: &str, vault: &Vault) -> Result<Vec<ScanResult>> {
        let mut results = Vec::new();

        for scanner in &self.scanners {
            let result = scanner.scan(input, vault).await?;

            if self.short_circuit && result.risk_score >= self.short_circuit_threshold {
                results.push(result);
                break;
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Execute pipeline in parallel
    ///
    /// All scanners run concurrently. Useful for I/O-bound scanners.
    pub async fn execute_parallel(&self, input: &str, vault: &Vault) -> Result<Vec<ScanResult>> {
        use futures::future::join_all;

        let futures: Vec<_> = self
            .scanners
            .iter()
            .map(|scanner| {
                let input = input.to_string();
                let vault = vault.clone();
                let scanner = Arc::clone(scanner);
                async move { scanner.scan(&input, &vault).await }
            })
            .collect();

        let results: Vec<Result<ScanResult>> = join_all(futures).await;

        // Collect results, propagating first error
        results.into_iter().collect()
    }

    /// Get aggregated result from pipeline
    pub async fn execute_aggregated(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        let results = self.execute(input, vault).await?;
        Ok(ScanResult::combine(results))
    }
}

impl Default for ScannerPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock scanner for testing
    struct MockScanner {
        name: String,
        risk_score: f32,
    }

    #[async_trait]
    impl Scanner for MockScanner {
        fn name(&self) -> &str {
            &self.name
        }

        async fn scan(&self, input: &str, _vault: &Vault) -> Result<ScanResult> {
            Ok(ScanResult::new(
                input.to_string(),
                self.risk_score < 0.5,
                self.risk_score,
            ))
        }

        fn scanner_type(&self) -> ScannerType {
            ScannerType::Input
        }
    }

    #[tokio::test]
    async fn test_scanner_pipeline_sequential() {
        let vault = Vault::new();

        let scanner1 = Arc::new(MockScanner {
            name: "test1".to_string(),
            risk_score: 0.3,
        });

        let scanner2 = Arc::new(MockScanner {
            name: "test2".to_string(),
            risk_score: 0.5,
        });

        let pipeline = ScannerPipeline::new().add(scanner1).add(scanner2);

        let results = pipeline.execute("test input", &vault).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].risk_score, 0.3);
        assert_eq!(results[1].risk_score, 0.5);
    }

    #[tokio::test]
    async fn test_scanner_pipeline_short_circuit() {
        let vault = Vault::new();

        let scanner1 = Arc::new(MockScanner {
            name: "test1".to_string(),
            risk_score: 0.95,
        });

        let scanner2 = Arc::new(MockScanner {
            name: "test2".to_string(),
            risk_score: 0.2,
        });

        let pipeline = ScannerPipeline::new()
            .add(scanner1)
            .add(scanner2)
            .with_short_circuit(0.9);

        let results = pipeline.execute("test input", &vault).await.unwrap();

        // Should stop after first scanner due to high risk
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].risk_score, 0.95);
    }

    #[tokio::test]
    async fn test_scanner_pipeline_aggregated() {
        let vault = Vault::new();

        let scanner1 = Arc::new(MockScanner {
            name: "test1".to_string(),
            risk_score: 0.3,
        });

        let scanner2 = Arc::new(MockScanner {
            name: "test2".to_string(),
            risk_score: 0.7,
        });

        let pipeline = ScannerPipeline::new().add(scanner1).add(scanner2);

        let result = pipeline
            .execute_aggregated("test input", &vault)
            .await
            .unwrap();

        // Should take maximum risk score
        assert_eq!(result.risk_score, 0.7);
    }
}
