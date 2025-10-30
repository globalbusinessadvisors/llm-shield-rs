//! LLM Shield WebAssembly Bindings
//!
//! ## Enterprise WASM Deployment
//!
//! This module provides JavaScript/TypeScript bindings for LLM Shield,
//! enabling deployment to:
//! - Browsers (Chrome, Firefox, Safari, Edge)
//! - Node.js
//! - Cloudflare Workers
//! - Fastly Compute@Edge
//! - AWS Lambda@Edge
//!
//! ## Example Usage (JavaScript)
//!
//! ```javascript
//! import init, { LlmShield, BanSubstringsScanner } from '@llm-shield/wasm';
//!
//! await init();
//!
//! const scanner = new BanSubstringsScanner(['badword', 'offensive']);
//! const result = await scanner.scan('This has badword');
//!
//! console.log(result.is_valid); // false
//! console.log(result.risk_score); // 1.0
//! ```

use wasm_bindgen::prelude::*;
use llm_shield_core::{Scanner, ScanResult as CoreScanResult, Vault};
use llm_shield_scanners::input::{BanSubstrings, BanSubstringsConfig, MatchType};
use std::sync::Arc;

// Set up panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Use wee_alloc as the global allocator for smaller binary size
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// JavaScript-compatible scan result
#[wasm_bindgen]
#[derive(Clone)]
pub struct ScanResult {
    /// Sanitized text
    pub sanitized_text: String,

    /// Whether input is valid
    pub is_valid: bool,

    /// Risk score (0.0-1.0)
    pub risk_score: f32,
}

#[wasm_bindgen]
impl ScanResult {
    /// Get metadata as JSON string
    pub fn metadata_json(&self) -> String {
        "{}".to_string() // TODO: Implement full metadata serialization
    }

    /// Get number of entities detected
    pub fn entity_count(&self) -> usize {
        0 // TODO: Implement
    }
}

impl From<CoreScanResult> for ScanResult {
    fn from(core: CoreScanResult) -> Self {
        Self {
            sanitized_text: core.sanitized_text,
            is_valid: core.is_valid,
            risk_score: core.risk_score,
        }
    }
}

/// BanSubstrings Scanner (WASM wrapper)
///
/// ## Example
///
/// ```javascript
/// const scanner = new BanSubstringsScanner(['spam', 'scam']);
/// const result = await scanner.scan('This is spam');
/// console.log(result.is_valid); // false
/// ```
#[wasm_bindgen]
pub struct BanSubstringsScanner {
    scanner: Arc<BanSubstrings>,
    vault: Vault,
}

#[wasm_bindgen]
impl BanSubstringsScanner {
    /// Create a new BanSubstrings scanner
    ///
    /// ## Parameters
    ///
    /// - `substrings`: Array of strings to ban
    /// - `case_sensitive`: Whether matching is case-sensitive (default: false)
    #[wasm_bindgen(constructor)]
    pub fn new(substrings: JsValue, case_sensitive: Option<bool>) -> Result<BanSubstringsScanner, JsValue> {
        let substrings: Vec<String> = serde_wasm_bindgen::from_value(substrings)
            .map_err(|e| JsValue::from_str(&format!("Invalid substrings array: {}", e)))?;

        let config = BanSubstringsConfig {
            substrings,
            case_sensitive: case_sensitive.unwrap_or(false),
            match_type: MatchType::Contains,
            redact: false,
        };

        let scanner = BanSubstrings::new(config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Self {
            scanner: Arc::new(scanner),
            vault: Vault::new(),
        })
    }

    /// Scan input text
    ///
    /// ## Returns
    ///
    /// Promise<ScanResult>
    #[wasm_bindgen]
    pub async fn scan(&self, input: String) -> Result<ScanResult, JsValue> {
        let result = self.scanner.scan(&input, &self.vault)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(result.into())
    }

    /// Get scanner name
    #[wasm_bindgen]
    pub fn name(&self) -> String {
        self.scanner.name().to_string()
    }

    /// Get scanner description
    #[wasm_bindgen]
    pub fn description(&self) -> String {
        self.scanner.description().to_string()
    }
}

/// LlmShield - Main API for multiple scanners
///
/// ## Example
///
/// ```javascript
/// const shield = new LlmShield();
/// await shield.add_scanner('ban_substrings', {
///   substrings: ['spam', 'scam'],
///   case_sensitive: false
/// });
///
/// const result = await shield.scan_all('Test input');
/// ```
#[wasm_bindgen]
pub struct LlmShield {
    scanners: Vec<Arc<dyn Scanner>>,
    vault: Vault,
}

#[wasm_bindgen]
impl LlmShield {
    /// Create a new LlmShield instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            scanners: Vec::new(),
            vault: Vault::new(),
        }
    }

    /// Add a BanSubstrings scanner
    #[wasm_bindgen]
    pub fn add_ban_substrings(&mut self, substrings: JsValue) -> Result<(), JsValue> {
        let substrings: Vec<String> = serde_wasm_bindgen::from_value(substrings)
            .map_err(|e| JsValue::from_str(&format!("Invalid substrings: {}", e)))?;

        let config = BanSubstringsConfig {
            substrings,
            case_sensitive: false,
            match_type: MatchType::Contains,
            redact: false,
        };

        let scanner = BanSubstrings::new(config)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        self.scanners.push(Arc::new(scanner));
        Ok(())
    }

    /// Scan input with all configured scanners
    #[wasm_bindgen]
    pub async fn scan_all(&self, input: String) -> Result<JsValue, JsValue> {
        let mut results = Vec::new();

        for scanner in &self.scanners {
            let result = scanner.scan(&input, &self.vault)
                .await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            results.push(ScanResult::from(result));
        }

        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get number of configured scanners
    #[wasm_bindgen]
    pub fn scanner_count(&self) -> usize {
        self.scanners.len()
    }
}

impl Default for LlmShield {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_ban_substrings_scanner() {
        let substrings = serde_wasm_bindgen::to_value(&vec!["badword"]).unwrap();
        let scanner = BanSubstringsScanner::new(substrings, None).unwrap();

        let result = scanner.scan("This has badword".to_string()).await.unwrap();

        assert!(!result.is_valid);
        assert_eq!(result.risk_score, 1.0);
    }

    #[wasm_bindgen_test]
    fn test_llm_shield_creation() {
        let shield = LlmShield::new();
        assert_eq!(shield.scanner_count(), 0);
    }
}
