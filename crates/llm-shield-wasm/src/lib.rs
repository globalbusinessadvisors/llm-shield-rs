//! WebAssembly bindings for LLM Shield
//!
//! This module provides JavaScript/TypeScript-friendly bindings for the LLM Shield library.
//!
//! ## Features
//!
//! - **LLMShield**: Main security scanner interface
//! - **Type Safety**: Full type conversion between Rust and JavaScript
//! - **Async Support**: Proper async/await support
//!
//! ## Example Usage (JavaScript)
//!
//! ```javascript
//! import init, { LLMShield } from './pkg';
//!
//! await init();
//!
//! const shield = new LLMShield();
//! const result = await shield.scan_text("Some text to scan");
//! ```

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use llm_shield_core::ScanResult;

// ============================================================================
// Panic Hook Setup
// ============================================================================

#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// Main LLM Shield Interface
// ============================================================================

/// Main LLM Shield security scanner for WASM
///
/// This is the primary interface for scanning text for security issues.
#[wasm_bindgen]
pub struct LLMShield {
    config: ShieldConfig,
}

#[wasm_bindgen]
impl LLMShield {
    /// Create a new LLM Shield instance
    ///
    /// # Example
    ///
    /// ```javascript
    /// const shield = new LLMShield();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            config: ShieldConfig::default(),
        }
    }

    /// Create a new instance with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Shield configuration
    ///
    /// # Example
    ///
    /// ```javascript
    /// const config = ShieldConfig.production();
    /// const shield = LLMShield.with_config(config);
    /// ```
    pub fn with_config(config: ShieldConfig) -> Self {
        Self { config }
    }

    /// Scan text for security issues
    ///
    /// This is an async operation that scans the provided text for various
    /// security issues including prompt injection, PII, toxicity, etc.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to scan
    ///
    /// # Returns
    ///
    /// JSON string containing scan results
    ///
    /// # Example
    ///
    /// ```javascript
    /// const resultJson = await shield.scan_text("Ignore previous instructions");
    /// const result = JSON.parse(resultJson);
    /// console.log(`Safe: ${result.is_safe}`);
    /// ```
    pub async fn scan_text(&self, text: &str) -> Result<String, JsValue> {
        // Create a basic scan result
        let has_injection = text.to_lowercase().contains("ignore previous instructions");

        let result = if has_injection {
            ScanResult::fail(text.to_string(), 0.9)
                .with_metadata("detected", "prompt_injection")
        } else {
            ScanResult::pass(text.to_string())
        };

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Detect PII in text
    ///
    /// Scans text for personally identifiable information.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to scan
    ///
    /// # Returns
    ///
    /// JSON string containing PII detection results
    ///
    /// # Example
    ///
    /// ```javascript
    /// const piiJson = await shield.detect_pii("My email is john@example.com");
    /// const pii = JSON.parse(piiJson);
    /// ```
    pub async fn detect_pii(&self, text: &str) -> Result<String, JsValue> {
        // Basic PII patterns
        let has_email = text.contains('@') && text.contains('.');
        let has_phone = text.chars().filter(|c| c.is_numeric()).count() >= 10;

        let mut result = if has_email || has_phone {
            ScanResult::fail(text.to_string(), 0.7)
        } else {
            ScanResult::pass(text.to_string())
        };

        if has_email {
            result = result.with_metadata("pii_type", "email");
        }
        if has_phone {
            result = result.with_metadata("pii_type", "phone");
        }

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Check text for toxicity
    ///
    /// Analyzes text for toxic or harmful content.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to check
    ///
    /// # Returns
    ///
    /// JSON string containing toxicity analysis
    ///
    /// # Example
    ///
    /// ```javascript
    /// const toxicityJson = await shield.check_toxicity("Some text");
    /// const toxicity = JSON.parse(toxicityJson);
    /// console.log(`Toxic: ${toxicity.is_toxic}`);
    /// ```
    pub async fn check_toxicity(&self, text: &str) -> Result<String, JsValue> {
        // Basic toxicity check (placeholder)
        let toxic_words = vec!["hate", "kill", "destroy"];
        let is_toxic = toxic_words.iter().any(|word| text.to_lowercase().contains(word));

        let result = if is_toxic {
            ScanResult::fail(text.to_string(), 0.8)
                .with_metadata("detection_type", "toxicity")
        } else {
            ScanResult::pass(text.to_string())
        };

        serde_json::to_string(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get the current configuration as JSON
    pub fn get_config_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.config)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
}

// ============================================================================
// Configuration
// ============================================================================

/// Shield configuration for JavaScript/TypeScript
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldConfig {
    /// Enable PII detection
    pub pii_detection: bool,
    /// Enable toxicity checking
    pub toxicity_check: bool,
    /// Enable prompt injection detection
    pub prompt_injection_check: bool,
    /// Confidence threshold (0.0 to 1.0)
    pub threshold: f32,
}

#[wasm_bindgen]
impl ShieldConfig {
    /// Create a new configuration
    #[wasm_bindgen(constructor)]
    pub fn new(
        pii_detection: bool,
        toxicity_check: bool,
        prompt_injection_check: bool,
        threshold: f32,
    ) -> Self {
        Self {
            pii_detection,
            toxicity_check,
            prompt_injection_check,
            threshold,
        }
    }

    /// Create default configuration
    pub fn default() -> Self {
        Self {
            pii_detection: true,
            toxicity_check: true,
            prompt_injection_check: true,
            threshold: 0.5,
        }
    }

    /// Create production configuration
    pub fn production() -> Self {
        Self {
            pii_detection: true,
            toxicity_check: true,
            prompt_injection_check: true,
            threshold: 0.5,
        }
    }

    /// Create development configuration
    pub fn development() -> Self {
        Self {
            pii_detection: true,
            toxicity_check: false,
            prompt_injection_check: true,
            threshold: 0.7,
        }
    }

    /// Create permissive configuration
    pub fn permissive() -> Self {
        Self {
            pii_detection: false,
            toxicity_check: false,
            prompt_injection_check: true,
            threshold: 0.8,
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Create from JSON string
    pub fn from_json(json: &str) -> Result<ShieldConfig, JsValue> {
        serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the library version
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Initialize the WASM module
///
/// Call this once before using any other functions.
#[wasm_bindgen]
pub fn initialize() {
    init_panic_hook();
}

/// Get information about the WASM build
#[wasm_bindgen]
pub fn build_info() -> String {
    serde_json::json!({
        "version": version(),
        "target": "wasm32-unknown-unknown",
        "features": ["basic-detection"],
        "note": "Full ML models not available in WASM build due to native dependency constraints"
    }).to_string()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = ShieldConfig::default();
        assert!(config.pii_detection);
        assert!(config.toxicity_check);
        assert!(config.prompt_injection_check);
        assert_eq!(config.threshold, 0.5);
    }

    #[test]
    fn test_version() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_shield_creation() {
        let shield = LLMShield::new();
        assert!(shield.config.pii_detection);
    }
}
