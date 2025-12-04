//! # Runtime Hooks for Scanner Integration
//!
//! Runtime hooks that allow scanners to query policy engine and config
//! during message inspection.
//!
//! ## Features
//!
//! - Pre-scan hooks for policy pre-check
//! - Post-scan hooks for result adjustment
//! - Config refresh hooks
//! - Error handling with fallbacks
//!
//! ## Example
//!
//! ```rust,ignore
//! use llm_shield_sdk::integrations::{RuntimeHooks, ScanHook};
//!
//! let hooks = RuntimeHooks::new()
//!     .with_pre_scan_hook(Box::new(PolicyPreCheckHook::new()))
//!     .with_post_scan_hook(Box::new(PolicyApplyHook::new()));
//!
//! // Register with vault
//! hooks.register(&vault)?;
//!
//! // Execute hooks during scan
//! let pre_result = hooks.execute_pre_scan(content, &vault).await?;
//! // ... perform scan ...
//! let final_result = hooks.execute_post_scan(scan_result, &vault).await?;
//! ```

use llm_shield_core::{
    adapters::policy::{PolicyContext, PolicyDecision, EnforcementAction},
    adapters::config::ShieldParameters,
    Error, Result, ScanResult, Vault,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Result of a hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookResult {
    /// Continue with scanning
    Continue,
    /// Skip scanning (pre-scan approved or rejected)
    Skip { reason: String, allow: bool },
    /// Modify the result
    Modify { adjustment: f32 },
    /// Error occurred (with fallback behavior)
    Error { message: String, fallback: HookFallback },
}

impl HookResult {
    /// Create a continue result
    pub fn continue_scan() -> Self {
        Self::Continue
    }

    /// Create a skip result (approved)
    pub fn skip_approved(reason: impl Into<String>) -> Self {
        Self::Skip {
            reason: reason.into(),
            allow: true,
        }
    }

    /// Create a skip result (rejected)
    pub fn skip_rejected(reason: impl Into<String>) -> Self {
        Self::Skip {
            reason: reason.into(),
            allow: false,
        }
    }

    /// Create a modify result
    pub fn modify(adjustment: f32) -> Self {
        Self::Modify { adjustment }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>, fallback: HookFallback) -> Self {
        Self::Error {
            message: message.into(),
            fallback,
        }
    }

    /// Check if this is a continue result
    pub fn should_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }

    /// Check if scanning should be skipped
    pub fn should_skip(&self) -> bool {
        matches!(self, Self::Skip { .. })
    }
}

/// Fallback behavior when hooks fail
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookFallback {
    /// Allow the content through
    Allow,
    /// Block the content
    Block,
    /// Continue with default behavior
    Continue,
    /// Fail the operation with error
    Fail,
}

impl Default for HookFallback {
    fn default() -> Self {
        Self::Continue
    }
}

/// Trait for scan hooks
#[async_trait]
pub trait ScanHook: Send + Sync {
    /// Name of the hook
    fn name(&self) -> &str;

    /// Execute the hook on content before scanning
    async fn on_pre_scan(&self, content: &str, vault: &Vault) -> Result<HookResult> {
        let _ = (content, vault);
        Ok(HookResult::Continue)
    }

    /// Execute the hook on result after scanning
    async fn on_post_scan(&self, result: &ScanResult, vault: &Vault) -> Result<HookResult> {
        let _ = (result, vault);
        Ok(HookResult::Continue)
    }

    /// Check if the hook is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Get fallback behavior
    fn fallback(&self) -> HookFallback {
        HookFallback::Continue
    }
}

/// Container for runtime hooks
pub struct RuntimeHooks {
    /// Pre-scan hooks (run before scanning)
    pre_scan_hooks: Vec<Arc<dyn ScanHook>>,
    /// Post-scan hooks (run after scanning)
    post_scan_hooks: Vec<Arc<dyn ScanHook>>,
    /// Whether to continue on hook errors
    continue_on_error: bool,
    /// Default fallback behavior
    default_fallback: HookFallback,
}

impl RuntimeHooks {
    /// Create new runtime hooks
    pub fn new() -> Self {
        Self {
            pre_scan_hooks: Vec::new(),
            post_scan_hooks: Vec::new(),
            continue_on_error: true,
            default_fallback: HookFallback::Continue,
        }
    }

    /// Add a pre-scan hook
    pub fn with_pre_scan_hook(mut self, hook: Arc<dyn ScanHook>) -> Self {
        self.pre_scan_hooks.push(hook);
        self
    }

    /// Add a post-scan hook
    pub fn with_post_scan_hook(mut self, hook: Arc<dyn ScanHook>) -> Self {
        self.post_scan_hooks.push(hook);
        self
    }

    /// Set error handling behavior
    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Set default fallback
    pub fn with_default_fallback(mut self, fallback: HookFallback) -> Self {
        self.default_fallback = fallback;
        self
    }

    /// Execute pre-scan hooks
    pub async fn execute_pre_scan(&self, content: &str, vault: &Vault) -> Result<PreScanResult> {
        let mut result = PreScanResult::Continue;

        for hook in &self.pre_scan_hooks {
            if !hook.is_enabled() {
                continue;
            }

            match hook.on_pre_scan(content, vault).await {
                Ok(HookResult::Continue) => continue,
                Ok(HookResult::Skip { reason, allow }) => {
                    result = if allow {
                        PreScanResult::Approved { reason }
                    } else {
                        PreScanResult::Rejected { reason }
                    };
                    break;
                }
                Ok(HookResult::Modify { adjustment }) => {
                    result = PreScanResult::Modify { adjustment };
                }
                Ok(HookResult::Error { message, fallback }) => {
                    if !self.continue_on_error {
                        return Err(Error::scanner(format!("Pre-scan hook failed: {}", message)));
                    }
                    match fallback {
                        HookFallback::Block => {
                            result = PreScanResult::Rejected {
                                reason: format!("Hook error (blocked): {}", message),
                            };
                            break;
                        }
                        HookFallback::Allow => {
                            result = PreScanResult::Approved {
                                reason: format!("Hook error (allowed): {}", message),
                            };
                            break;
                        }
                        HookFallback::Fail => {
                            return Err(Error::scanner(format!("Pre-scan hook failed: {}", message)));
                        }
                        HookFallback::Continue => continue,
                    }
                }
                Err(e) if self.continue_on_error => {
                    tracing::warn!("Pre-scan hook {} failed: {}", hook.name(), e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(result)
    }

    /// Execute post-scan hooks
    pub async fn execute_post_scan(
        &self,
        mut scan_result: ScanResult,
        vault: &Vault,
    ) -> Result<ScanResult> {
        for hook in &self.post_scan_hooks {
            if !hook.is_enabled() {
                continue;
            }

            match hook.on_post_scan(&scan_result, vault).await {
                Ok(HookResult::Continue) => continue,
                Ok(HookResult::Modify { adjustment }) => {
                    // Apply risk score adjustment
                    scan_result.risk_score = (scan_result.risk_score + adjustment).clamp(0.0, 1.0);
                    scan_result = scan_result.with_metadata("hook_adjustment", adjustment);
                }
                Ok(HookResult::Skip { reason, allow }) => {
                    if !allow {
                        scan_result.is_valid = false;
                        scan_result.risk_score = 1.0;
                    }
                    scan_result = scan_result.with_metadata("hook_override", reason);
                    break;
                }
                Ok(HookResult::Error { message, fallback }) => {
                    if !self.continue_on_error {
                        return Err(Error::scanner(format!("Post-scan hook failed: {}", message)));
                    }
                    match fallback {
                        HookFallback::Block => {
                            scan_result.is_valid = false;
                            scan_result.risk_score = 1.0;
                            scan_result = scan_result.with_metadata(
                                "hook_error",
                                format!("Blocked: {}", message),
                            );
                            break;
                        }
                        HookFallback::Fail => {
                            return Err(Error::scanner(format!("Post-scan hook failed: {}", message)));
                        }
                        _ => continue,
                    }
                }
                Err(e) if self.continue_on_error => {
                    tracing::warn!("Post-scan hook {} failed: {}", hook.name(), e);
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(scan_result)
    }

    /// Register hooks metadata in vault
    pub fn register(&self, vault: &Vault) -> Result<()> {
        let pre_scan_names: Vec<_> = self.pre_scan_hooks.iter().map(|h| h.name()).collect();
        let post_scan_names: Vec<_> = self.post_scan_hooks.iter().map(|h| h.name()).collect();

        vault.set("runtime_hooks:pre_scan", &pre_scan_names)?;
        vault.set("runtime_hooks:post_scan", &post_scan_names)?;
        vault.set("runtime_hooks:continue_on_error", self.continue_on_error)?;

        Ok(())
    }

    /// Check if any hooks are registered
    pub fn has_hooks(&self) -> bool {
        !self.pre_scan_hooks.is_empty() || !self.post_scan_hooks.is_empty()
    }

    /// Get number of pre-scan hooks
    pub fn pre_scan_hook_count(&self) -> usize {
        self.pre_scan_hooks.len()
    }

    /// Get number of post-scan hooks
    pub fn post_scan_hook_count(&self) -> usize {
        self.post_scan_hooks.len()
    }
}

impl Default for RuntimeHooks {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of pre-scan hook execution
#[derive(Debug, Clone)]
pub enum PreScanResult {
    /// Continue with normal scanning
    Continue,
    /// Content is pre-approved, skip scanning
    Approved { reason: String },
    /// Content is rejected, skip scanning
    Rejected { reason: String },
    /// Apply a risk score modifier
    Modify { adjustment: f32 },
}

impl PreScanResult {
    /// Check if scanning should proceed
    pub fn should_scan(&self) -> bool {
        matches!(self, Self::Continue | Self::Modify { .. })
    }

    /// Check if content was approved
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::Approved { .. })
    }

    /// Check if content was rejected
    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected { .. })
    }

    /// Get the risk adjustment if any
    pub fn risk_adjustment(&self) -> Option<f32> {
        match self {
            Self::Modify { adjustment } => Some(*adjustment),
            _ => None,
        }
    }
}

/// A no-op scan hook for testing
pub struct NoOpScanHook {
    name: String,
}

impl NoOpScanHook {
    /// Create a new no-op hook
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl ScanHook for NoOpScanHook {
    fn name(&self) -> &str {
        &self.name
    }
}

/// A policy-based pre-scan hook
pub struct PolicyPreCheckHook {
    check_type: String,
    enabled: bool,
}

impl PolicyPreCheckHook {
    /// Create a new policy pre-check hook
    pub fn new(check_type: impl Into<String>) -> Self {
        Self {
            check_type: check_type.into(),
            enabled: true,
        }
    }

    /// Disable the hook
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[async_trait]
impl ScanHook for PolicyPreCheckHook {
    fn name(&self) -> &str {
        "policy_pre_check"
    }

    async fn on_pre_scan(&self, content: &str, vault: &Vault) -> Result<HookResult> {
        // Check for policy decision in vault
        let cache_key = format!("policy_cache:{}", self.check_type);
        if let Ok(Some(decision)) = vault.get::<_, PolicyDecision>(&cache_key) {
            if decision.should_block() {
                return Ok(HookResult::skip_rejected(decision.reason().to_string()));
            }
        }

        Ok(HookResult::Continue)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn fallback(&self) -> HookFallback {
        HookFallback::Continue
    }
}

/// A config-based threshold hook
pub struct ConfigThresholdHook {
    scanner_name: String,
    enabled: bool,
}

impl ConfigThresholdHook {
    /// Create a new config threshold hook
    pub fn new(scanner_name: impl Into<String>) -> Self {
        Self {
            scanner_name: scanner_name.into(),
            enabled: true,
        }
    }
}

#[async_trait]
impl ScanHook for ConfigThresholdHook {
    fn name(&self) -> &str {
        "config_threshold"
    }

    async fn on_post_scan(&self, result: &ScanResult, vault: &Vault) -> Result<HookResult> {
        // Check for config-driven threshold in vault
        if let Ok(Some(params)) = vault.get::<_, ShieldParameters>("shield_parameters") {
            let threshold = params.thresholds.get_scanner_threshold(&self.scanner_name);

            if result.risk_score >= threshold {
                // Ensure result is marked invalid if over threshold
                return Ok(HookResult::Modify {
                    adjustment: 0.0, // No adjustment, just ensure threshold is applied
                });
            }
        }

        Ok(HookResult::Continue)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn fallback(&self) -> HookFallback {
        HookFallback::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_result() {
        assert!(HookResult::Continue.should_continue());
        assert!(HookResult::skip_approved("test").should_skip());
        assert!(HookResult::skip_rejected("test").should_skip());
    }

    #[test]
    fn test_pre_scan_result() {
        let continue_result = PreScanResult::Continue;
        assert!(continue_result.should_scan());

        let approved = PreScanResult::Approved {
            reason: "test".to_string(),
        };
        assert!(!approved.should_scan());
        assert!(approved.is_approved());

        let rejected = PreScanResult::Rejected {
            reason: "test".to_string(),
        };
        assert!(!rejected.should_scan());
        assert!(rejected.is_rejected());

        let modify = PreScanResult::Modify { adjustment: 0.1 };
        assert!(modify.should_scan());
        assert_eq!(modify.risk_adjustment(), Some(0.1));
    }

    #[tokio::test]
    async fn test_runtime_hooks() {
        let hooks = RuntimeHooks::new()
            .with_pre_scan_hook(Arc::new(NoOpScanHook::new("test-pre")))
            .with_post_scan_hook(Arc::new(NoOpScanHook::new("test-post")));

        assert!(hooks.has_hooks());
        assert_eq!(hooks.pre_scan_hook_count(), 1);
        assert_eq!(hooks.post_scan_hook_count(), 1);

        let vault = Vault::new();
        let pre_result = hooks.execute_pre_scan("test", &vault).await.unwrap();
        assert!(pre_result.should_scan());
    }

    #[tokio::test]
    async fn test_policy_pre_check_hook() {
        let hook = PolicyPreCheckHook::new("prompt-injection");
        let vault = Vault::new();

        let result = hook.on_pre_scan("test content", &vault).await.unwrap();
        assert!(result.should_continue());
    }

    #[test]
    fn test_hook_registration() {
        let hooks = RuntimeHooks::new()
            .with_pre_scan_hook(Arc::new(NoOpScanHook::new("pre-1")))
            .with_pre_scan_hook(Arc::new(NoOpScanHook::new("pre-2")))
            .with_post_scan_hook(Arc::new(NoOpScanHook::new("post-1")));

        let vault = Vault::new();
        hooks.register(&vault).unwrap();

        let pre_hooks: Vec<String> = vault.get("runtime_hooks:pre_scan").unwrap().unwrap();
        assert_eq!(pre_hooks.len(), 2);

        let post_hooks: Vec<String> = vault.get("runtime_hooks:post_scan").unwrap().unwrap();
        assert_eq!(post_hooks.len(), 1);
    }
}
