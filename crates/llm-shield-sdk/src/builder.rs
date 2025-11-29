//! # Shield Builder
//!
//! Fluent builder pattern for constructing Shield instances.
//!
//! ## Design Principles
//!
//! - Fluent, chainable API
//! - Compile-time configuration validation where possible
//! - Clear error messages for invalid configurations
//! - Support for presets and custom configurations

use crate::config::{ParallelConfig, ScanMode, ShieldConfig};
use crate::error::{SdkError, SdkResult};
use crate::preset::Preset;
use crate::scanner_factory::{InputScannerFactory, OutputScannerFactory};
use crate::shield::Shield;
use llm_shield_core::Scanner;
use std::sync::Arc;

/// Builder for constructing Shield instances
///
/// ## Example
///
/// ```rust,ignore
/// let shield = ShieldBuilder::new()
///     .with_preset(Preset::Standard)
///     .with_short_circuit(0.9)
///     .with_parallel_execution(true)
///     .build()?;
/// ```
#[derive(Default)]
pub struct ShieldBuilder {
    config: ShieldConfig,
    input_scanners: Vec<Arc<dyn Scanner>>,
    output_scanners: Vec<Arc<dyn Scanner>>,
    preset: Option<Preset>,
}

impl ShieldBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            config: ShieldConfig::default(),
            input_scanners: Vec::new(),
            output_scanners: Vec::new(),
            preset: None,
        }
    }

    // ========================================================================
    // Preset Configuration
    // ========================================================================

    /// Use a security preset
    ///
    /// This will configure the shield with preset scanners and settings.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let shield = Shield::builder()
    ///     .with_preset(Preset::Strict)
    ///     .build()?;
    /// ```
    pub fn with_preset(mut self, preset: Preset) -> Self {
        self.preset = Some(preset);
        self.config = preset.config();
        self
    }

    // ========================================================================
    // Scanner Configuration
    // ========================================================================

    /// Add an input scanner
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let shield = Shield::builder()
    ///     .add_input_scanner(Secrets::default_config()?)
    ///     .add_input_scanner(Toxicity::default_config()?)
    ///     .build()?;
    /// ```
    pub fn add_input_scanner<S: Scanner + 'static>(mut self, scanner: S) -> Self {
        self.input_scanners.push(Arc::new(scanner));
        self
    }

    /// Add an input scanner from an Arc
    pub fn add_input_scanner_arc(mut self, scanner: Arc<dyn Scanner>) -> Self {
        self.input_scanners.push(scanner);
        self
    }

    /// Add multiple input scanners
    pub fn add_input_scanners<I>(mut self, scanners: I) -> Self
    where
        I: IntoIterator<Item = Arc<dyn Scanner>>,
    {
        self.input_scanners.extend(scanners);
        self
    }

    /// Add an output scanner
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let shield = Shield::builder()
    ///     .add_output_scanner(Sensitive::default_config()?)
    ///     .build()?;
    /// ```
    pub fn add_output_scanner<S: Scanner + 'static>(mut self, scanner: S) -> Self {
        self.output_scanners.push(Arc::new(scanner));
        self
    }

    /// Add an output scanner from an Arc
    pub fn add_output_scanner_arc(mut self, scanner: Arc<dyn Scanner>) -> Self {
        self.output_scanners.push(scanner);
        self
    }

    /// Add multiple output scanners
    pub fn add_output_scanners<I>(mut self, scanners: I) -> Self
    where
        I: IntoIterator<Item = Arc<dyn Scanner>>,
    {
        self.output_scanners.extend(scanners);
        self
    }

    // ========================================================================
    // Execution Configuration
    // ========================================================================

    /// Set scan mode (prompt, output, or both)
    pub fn with_scan_mode(mut self, mode: ScanMode) -> Self {
        self.config.scan_mode = mode;
        self
    }

    /// Enable/disable parallel scanner execution
    pub fn with_parallel_execution(mut self, enabled: bool) -> Self {
        self.config.parallel.enabled = enabled;
        self
    }

    /// Set maximum concurrent scanners
    pub fn with_max_concurrent(mut self, max: usize) -> Self {
        self.config.parallel = ParallelConfig::with_concurrency(max);
        self
    }

    /// Set parallel configuration
    pub fn with_parallel_config(mut self, config: ParallelConfig) -> Self {
        self.config.parallel = config;
        self
    }

    /// Enable short-circuit evaluation
    ///
    /// If any scanner returns a risk score >= threshold, stop scanning.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let shield = Shield::builder()
    ///     .with_short_circuit(0.9)
    ///     .build()?;
    /// ```
    pub fn with_short_circuit(mut self, threshold: f32) -> Self {
        self.config.short_circuit_threshold = Some(threshold.clamp(0.0, 1.0));
        self
    }

    /// Disable short-circuit evaluation
    pub fn without_short_circuit(mut self) -> Self {
        self.config.short_circuit_threshold = None;
        self
    }

    /// Set operation timeout in milliseconds
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.config.timeout_ms = Some(timeout_ms);
        self
    }

    /// Disable operation timeout
    pub fn without_timeout(mut self) -> Self {
        self.config.timeout_ms = None;
        self
    }

    // ========================================================================
    // Observability Configuration
    // ========================================================================

    /// Enable/disable tracing
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.config.enable_tracing = enabled;
        self
    }

    // ========================================================================
    // Caching Configuration
    // ========================================================================

    /// Enable caching with TTL
    pub fn with_caching(mut self, ttl_seconds: u64) -> Self {
        self.config.enable_caching = true;
        self.config.cache_ttl_seconds = ttl_seconds;
        self
    }

    /// Disable caching
    pub fn without_caching(mut self) -> Self {
        self.config.enable_caching = false;
        self
    }

    // ========================================================================
    // Batch Configuration
    // ========================================================================

    /// Set maximum batch size
    pub fn with_max_batch_size(mut self, max: usize) -> Self {
        self.config.max_batch_size = max;
        self
    }

    // ========================================================================
    // Full Configuration
    // ========================================================================

    /// Set the full configuration
    pub fn with_config(mut self, config: ShieldConfig) -> Self {
        self.config = config;
        self
    }

    // ========================================================================
    // Build
    // ========================================================================

    /// Build the Shield instance
    ///
    /// ## Errors
    ///
    /// Returns an error if:
    /// - No scanners are configured and no preset is selected
    /// - Scanner initialization fails
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let shield = Shield::builder()
    ///     .with_preset(Preset::Standard)
    ///     .build()?;
    /// ```
    pub fn build(mut self) -> SdkResult<Shield> {
        // If a preset was selected and no custom scanners were added, use preset scanners
        if let Some(preset) = self.preset {
            if self.input_scanners.is_empty() && self.output_scanners.is_empty() {
                self = self.load_preset_scanners(preset)?;
            }
        }

        // Validate configuration
        self.validate()?;

        Ok(Shield::new(
            self.config,
            self.input_scanners,
            self.output_scanners,
        ))
    }

    /// Load scanners from a preset
    fn load_preset_scanners(mut self, preset: Preset) -> SdkResult<Self> {
        // Load input scanners
        for scanner_type in preset.input_scanners() {
            match InputScannerFactory::create(scanner_type) {
                Ok(scanner) => {
                    self.input_scanners.push(scanner);
                }
                Err(e) => {
                    // Skip scanners that require configuration
                    tracing::debug!("Skipping scanner {:?}: {}", scanner_type, e);
                }
            }
        }

        // Load output scanners
        for scanner_type in preset.output_scanners() {
            match OutputScannerFactory::create(scanner_type) {
                Ok(scanner) => {
                    self.output_scanners.push(scanner);
                }
                Err(e) => {
                    // Skip scanners that require configuration
                    tracing::debug!("Skipping scanner {:?}: {}", scanner_type, e);
                }
            }
        }

        Ok(self)
    }

    /// Validate the builder configuration
    fn validate(&self) -> SdkResult<()> {
        // Check that we have at least some scanners for the configured scan mode
        match self.config.scan_mode {
            ScanMode::Prompt if self.input_scanners.is_empty() => {
                return Err(SdkError::builder(
                    "Scan mode is Prompt but no input scanners configured",
                ));
            }
            ScanMode::Output if self.output_scanners.is_empty() => {
                return Err(SdkError::builder(
                    "Scan mode is Output but no output scanners configured",
                ));
            }
            ScanMode::Both if self.input_scanners.is_empty() && self.output_scanners.is_empty() => {
                return Err(SdkError::builder(
                    "No scanners configured. Use a preset or add scanners manually.",
                ));
            }
            _ => {}
        }

        // Validate short-circuit threshold
        if let Some(threshold) = self.config.short_circuit_threshold {
            if !(0.0..=1.0).contains(&threshold) {
                return Err(SdkError::builder(
                    "Short-circuit threshold must be between 0.0 and 1.0",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llm_shield_scanners::input::Secrets;

    #[test]
    fn test_builder_default() {
        let builder = ShieldBuilder::new();
        assert!(builder.input_scanners.is_empty());
        assert!(builder.output_scanners.is_empty());
    }

    #[test]
    fn test_builder_with_preset() {
        let builder = ShieldBuilder::new().with_preset(Preset::Standard);
        assert_eq!(builder.preset, Some(Preset::Standard));
    }

    #[test]
    fn test_builder_add_scanner() {
        let scanner = Secrets::default_config().unwrap();
        let builder = ShieldBuilder::new().add_input_scanner(scanner);
        assert_eq!(builder.input_scanners.len(), 1);
    }

    #[test]
    fn test_builder_with_short_circuit() {
        let builder = ShieldBuilder::new().with_short_circuit(0.9);
        assert_eq!(builder.config.short_circuit_threshold, Some(0.9));
    }

    #[test]
    fn test_builder_with_parallel() {
        let builder = ShieldBuilder::new()
            .with_parallel_execution(true)
            .with_max_concurrent(8);
        assert!(builder.config.parallel.enabled);
        assert_eq!(builder.config.parallel.max_concurrent, 8);
    }

    #[test]
    fn test_builder_build_with_preset() {
        let shield = ShieldBuilder::new()
            .with_preset(Preset::Permissive)
            .build();
        assert!(shield.is_ok());
    }

    #[test]
    fn test_builder_build_without_scanners_fails() {
        let result = ShieldBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_validates_threshold() {
        let builder = ShieldBuilder::new()
            .with_preset(Preset::Standard)
            .with_short_circuit(0.5);
        let shield = builder.build();
        assert!(shield.is_ok());
    }
}
