//! # Policy Integration for SDK
//!
//! SDK-level integration with LLM-Policy-Engine for policy-driven scanning.
//!
//! ## Features
//!
//! - Automatic policy evaluation during scans
//! - Context-aware policy decisions
//! - Fallback handling when policy engine is unavailable
//! - Caching of policy decisions
//!
//! ## Example
//!
//! ```rust,ignore
//! use llm_shield_sdk::integrations::PolicyIntegration;
//!
//! let integration = PolicyIntegration::builder()
//!     .with_app_id("my-app")
//!     .with_tenant_id("tenant-123")
//!     .with_fallback_action(EnforcementAction::Allow)
//!     .with_cache_ttl(300)
//!     .build();
//!
//! // Apply during scan
//! let result = integration.evaluate_and_apply(context, content, scan_result).await?;
//! ```

use llm_shield_core::{
    adapters::policy::{
        PolicyAdapter, PolicyContext, PolicyDecision, PolicyEvaluator,
        EnforcementAction, PolicyResult, NoOpPolicyEvaluator, PolicyHook,
    },
    Error, Result, ScanResult, Vault,
};
use std::sync::Arc;

/// High-level policy integration for Shield SDK
pub struct PolicyIntegration {
    /// The underlying policy adapter
    adapter: Arc<dyn PolicyIntegrationAdapter>,
    /// Application ID for policy context
    app_id: Option<String>,
    /// Default tenant ID
    tenant_id: Option<String>,
    /// Fallback action when policy engine is unavailable
    fallback_action: EnforcementAction,
    /// Enable caching of decisions
    enable_caching: bool,
    /// Cache TTL in seconds
    cache_ttl: u64,
}

impl PolicyIntegration {
    /// Create a new policy integration with default settings
    pub fn new() -> Self {
        Self {
            adapter: Arc::new(DefaultPolicyAdapter::new()),
            app_id: None,
            tenant_id: None,
            fallback_action: EnforcementAction::Allow,
            enable_caching: true,
            cache_ttl: 300,
        }
    }

    /// Create a builder for policy integration
    pub fn builder() -> PolicyIntegrationBuilder {
        PolicyIntegrationBuilder::new()
    }

    /// Create a policy context for scanning
    pub fn create_context(&self, check_type: &str) -> PolicyContext {
        let mut context = PolicyContext::new(check_type);

        if let Some(ref app_id) = self.app_id {
            context = context.with_attribute("app_id", app_id);
        }

        if let Some(ref tenant_id) = self.tenant_id {
            context = context.with_tenant(tenant_id);
        }

        context
    }

    /// Evaluate policy for content
    pub async fn evaluate(
        &self,
        check_type: &str,
        content: &str,
    ) -> PolicyResult<PolicyDecision> {
        let context = self.create_context(check_type);
        self.adapter.evaluate(&context, content).await
    }

    /// Evaluate policy and apply to scan result
    pub async fn evaluate_and_apply(
        &self,
        check_type: &str,
        content: &str,
        scan_result: ScanResult,
    ) -> Result<ScanResult> {
        let context = self.create_context(check_type);
        self.adapter.apply_to_result(&context, content, scan_result).await
    }

    /// Register policy hooks in the vault
    pub fn register_hooks(&self, vault: &Vault, check_types: &[&str]) -> Result<()> {
        for check_type in check_types {
            let hook = PolicyHook::new(*check_type)
                .with_fallback(self.fallback_action);
            hook.register(vault)?;
        }
        Ok(())
    }

    /// Check if policy engine is healthy
    pub async fn is_healthy(&self) -> bool {
        self.adapter.health_check().await
    }

    /// Get the fallback action
    pub fn fallback_action(&self) -> EnforcementAction {
        self.fallback_action
    }
}

impl Default for PolicyIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for PolicyIntegration
pub struct PolicyIntegrationBuilder {
    app_id: Option<String>,
    tenant_id: Option<String>,
    fallback_action: EnforcementAction,
    enable_caching: bool,
    cache_ttl: u64,
}

impl PolicyIntegrationBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            app_id: None,
            tenant_id: None,
            fallback_action: EnforcementAction::Allow,
            enable_caching: true,
            cache_ttl: 300,
        }
    }

    /// Set the application ID
    pub fn with_app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    /// Set the tenant ID
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Set the fallback action
    pub fn with_fallback_action(mut self, action: EnforcementAction) -> Self {
        self.fallback_action = action;
        self
    }

    /// Enable caching with TTL
    pub fn with_cache_ttl(mut self, ttl_seconds: u64) -> Self {
        self.enable_caching = true;
        self.cache_ttl = ttl_seconds;
        self
    }

    /// Disable caching
    pub fn without_caching(mut self) -> Self {
        self.enable_caching = false;
        self
    }

    /// Build the PolicyIntegration
    pub fn build(self) -> PolicyIntegration {
        PolicyIntegration {
            adapter: Arc::new(DefaultPolicyAdapter::new()),
            app_id: self.app_id,
            tenant_id: self.tenant_id,
            fallback_action: self.fallback_action,
            enable_caching: self.enable_caching,
            cache_ttl: self.cache_ttl,
        }
    }
}

impl Default for PolicyIntegrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for policy integration adapters
#[async_trait::async_trait]
pub trait PolicyIntegrationAdapter: Send + Sync {
    /// Evaluate policy
    async fn evaluate(
        &self,
        context: &PolicyContext,
        content: &str,
    ) -> PolicyResult<PolicyDecision>;

    /// Apply policy to scan result
    async fn apply_to_result(
        &self,
        context: &PolicyContext,
        content: &str,
        scan_result: ScanResult,
    ) -> Result<ScanResult>;

    /// Health check
    async fn health_check(&self) -> bool;
}

/// Default policy adapter using NoOp evaluator
struct DefaultPolicyAdapter {
    inner: PolicyAdapter<NoOpPolicyEvaluator>,
}

impl DefaultPolicyAdapter {
    fn new() -> Self {
        Self {
            inner: PolicyAdapter::new(NoOpPolicyEvaluator),
        }
    }
}

#[async_trait::async_trait]
impl PolicyIntegrationAdapter for DefaultPolicyAdapter {
    async fn evaluate(
        &self,
        context: &PolicyContext,
        content: &str,
    ) -> PolicyResult<PolicyDecision> {
        self.inner.evaluate(context, content).await
    }

    async fn apply_to_result(
        &self,
        context: &PolicyContext,
        content: &str,
        scan_result: ScanResult,
    ) -> Result<ScanResult> {
        self.inner.apply_policy(context, content, scan_result).await
    }

    async fn health_check(&self) -> bool {
        self.inner.is_healthy().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_integration_builder() {
        let integration = PolicyIntegration::builder()
            .with_app_id("test-app")
            .with_tenant_id("tenant-1")
            .with_fallback_action(EnforcementAction::Block)
            .with_cache_ttl(600)
            .build();

        assert_eq!(integration.app_id, Some("test-app".to_string()));
        assert_eq!(integration.tenant_id, Some("tenant-1".to_string()));
        assert_eq!(integration.fallback_action, EnforcementAction::Block);
        assert_eq!(integration.cache_ttl, 600);
    }

    #[test]
    fn test_create_context() {
        let integration = PolicyIntegration::builder()
            .with_app_id("my-app")
            .with_tenant_id("tenant-123")
            .build();

        let context = integration.create_context("prompt-injection");
        assert_eq!(context.check_type, "prompt-injection");
        assert_eq!(context.tenant_id, Some("tenant-123".to_string()));
    }

    #[tokio::test]
    async fn test_policy_evaluation() {
        let integration = PolicyIntegration::new();
        let decision = integration.evaluate("test-check", "test content").await.unwrap();

        // Default adapter allows everything
        assert!(decision.allowed);
        assert!(!decision.should_block());
    }

    #[tokio::test]
    async fn test_health_check() {
        let integration = PolicyIntegration::new();
        assert!(integration.is_healthy().await);
    }
}
