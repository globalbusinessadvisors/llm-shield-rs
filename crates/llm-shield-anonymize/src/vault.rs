//! Enhanced Vault for Anonymization/Deanonymization - Phase 9A
//!
//! This module provides TTL-based session management and audit logging.

use crate::types::EntityType;
use async_trait::async_trait;
use llm_shield_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Vault-specific mapping with session and TTL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMapping {
    pub session_id: String,
    pub placeholder: String,
    pub entity_type: EntityType,
    pub original_value: String,
    pub confidence: f32,
    pub timestamp: SystemTime,
    pub expires_at: SystemTime,
}

impl EntityMapping {
    pub fn new(
        session_id: String,
        placeholder: String,
        entity_type: EntityType,
        original_value: String,
        confidence: f32,
        ttl_seconds: u64,
    ) -> Self {
        let timestamp = SystemTime::now();
        let expires_at = timestamp + std::time::Duration::from_secs(ttl_seconds);
        Self {
            session_id,
            placeholder,
            entity_type,
            original_value,
            confidence,
            timestamp,
            expires_at,
        }
    }

    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// Anonymization session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationSession {
    pub session_id: String,
    pub user_id: Option<String>,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub mappings: Vec<EntityMapping>,
}

impl AnonymizationSession {
    pub fn new(session_id: String, user_id: Option<String>, ttl_seconds: u64) -> Self {
        let created_at = SystemTime::now();
        let expires_at = created_at + std::time::Duration::from_secs(ttl_seconds);
        Self {
            session_id,
            user_id,
            created_at,
            expires_at,
            mappings: Vec::new(),
        }
    }
}

/// Vault storage trait
#[async_trait]
pub trait VaultStorage: Send + Sync {
    async fn store_mapping(&self, mapping: EntityMapping) -> Result<()>;
    async fn get_mapping(&self, session_id: &str, placeholder: &str) -> Result<Option<EntityMapping>>;
    async fn delete_mapping(&self, session_id: &str, placeholder: &str) -> Result<()>;
    async fn get_session_mappings(&self, session_id: &str) -> Result<Vec<EntityMapping>>;
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    async fn cleanup_expired(&self) -> Result<usize>;
    async fn list_session_ids(&self) -> Result<Vec<String>>;
}

/// In-memory vault
#[derive(Clone)]
pub struct MemoryVault {
    mappings: Arc<RwLock<HashMap<String, EntityMapping>>>,
}

impl MemoryVault {
    pub fn new() -> Self {
        Self {
            mappings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn make_key(session_id: &str, placeholder: &str) -> String {
        format!("{}:{}", session_id, placeholder)
    }
}

impl Default for MemoryVault {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VaultStorage for MemoryVault {
    async fn store_mapping(&self, mapping: EntityMapping) -> Result<()> {
        let key = Self::make_key(&mapping.session_id, &mapping.placeholder);
        self.mappings
            .write()
            .map_err(|e| llm_shield_core::Error::internal(format!("Lock error: {}", e)))?
            .insert(key, mapping);
        Ok(())
    }

    async fn get_mapping(&self, session_id: &str, placeholder: &str) -> Result<Option<EntityMapping>> {
        let key = Self::make_key(session_id, placeholder);
        let mappings = self
            .mappings
            .read()
            .map_err(|e| llm_shield_core::Error::internal(format!("Lock error: {}", e)))?;
        match mappings.get(&key) {
            Some(m) if !m.is_expired() => Ok(Some(m.clone())),
            _ => Ok(None),
        }
    }

    async fn delete_mapping(&self, session_id: &str, placeholder: &str) -> Result<()> {
        let key = Self::make_key(session_id, placeholder);
        self.mappings
            .write()
            .map_err(|e| llm_shield_core::Error::internal(format!("Lock error: {}", e)))?
            .remove(&key);
        Ok(())
    }

    async fn get_session_mappings(&self, session_id: &str) -> Result<Vec<EntityMapping>> {
        let prefix = format!("{}:", session_id);
        let mappings = self
            .mappings
            .read()
            .map_err(|e| llm_shield_core::Error::internal(format!("Lock error: {}", e)))?;
        Ok(mappings
            .iter()
            .filter(|(k, v)| k.starts_with(&prefix) && !v.is_expired())
            .map(|(_, v)| v.clone())
            .collect())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let prefix = format!("{}:", session_id);
        self.mappings
            .write()
            .map_err(|e| llm_shield_core::Error::internal(format!("Lock error: {}", e)))?
            .retain(|k, _| !k.starts_with(&prefix));
        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let mut mappings = self
            .mappings
            .write()
            .map_err(|e| llm_shield_core::Error::internal(format!("Lock error: {}", e)))?;
        let before = mappings.len();
        mappings.retain(|_, v| !v.is_expired());
        Ok(before - mappings.len())
    }

    async fn list_session_ids(&self) -> Result<Vec<String>> {
        use std::collections::HashSet;
        let mappings = self
            .mappings
            .read()
            .map_err(|e| llm_shield_core::Error::internal(format!("Lock error: {}", e)))?;
        Ok(mappings
            .iter()
            .filter(|(_, v)| !v.is_expired())
            .map(|(k, _)| k.split(':').next().unwrap_or("").to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect())
    }
}

/// Session manager
#[derive(Clone)]
pub struct SessionManager {
    vault: Arc<dyn VaultStorage>,
}

impl SessionManager {
    pub fn new<V: VaultStorage + 'static>(vault: V) -> Self {
        Self {
            vault: Arc::new(vault),
        }
    }

    pub async fn create_session(&self, user_id: Option<String>, ttl_seconds: u64) -> Result<AnonymizationSession> {
        let session_id = self.generate_session_id();
        Ok(AnonymizationSession::new(session_id, user_id, ttl_seconds))
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<AnonymizationSession>> {
        let mappings = self.vault.get_session_mappings(session_id).await?;
        if mappings.is_empty() {
            return Ok(None);
        }
        let first = &mappings[0];
        Ok(Some(AnonymizationSession {
            session_id: session_id.to_string(),
            user_id: None,
            created_at: first.timestamp,
            expires_at: first.expires_at,
            mappings,
        }))
    }

    pub async fn add_mapping(&self, _session_id: &str, mapping: EntityMapping) -> Result<()> {
        self.vault.store_mapping(mapping).await
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        self.vault.delete_session(session_id).await
    }

    pub async fn list_sessions(&self) -> Result<Vec<String>> {
        self.vault.list_session_ids().await
    }

    fn generate_session_id(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};
        let random_state = RandomState::new();
        let mut hasher = random_state.build_hasher();
        timestamp.hash(&mut hasher);
        std::thread::current().id().hash(&mut hasher);
        format!("sess_{}_{:x}", timestamp, hasher.finish() & 0xFFFF)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(MemoryVault::new())
    }
}

/// Audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    AnonymizeStart { session_id: String, entity_count: usize },
    AnonymizeComplete { session_id: String, duration_ms: u64 },
    VaultStore { session_id: String, entity_type: EntityType },
    VaultExpire { session_id: String, mapping_count: usize },
}

impl AuditEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            AuditEvent::AnonymizeStart { .. } => "anonymize_start",
            AuditEvent::AnonymizeComplete { .. } => "anonymize_complete",
            AuditEvent::VaultStore { .. } => "vault_store",
            AuditEvent::VaultExpire { .. } => "vault_expire",
        }
    }
}

/// Audit logger
#[derive(Debug, Clone)]
pub struct AuditLogger {
    enabled: bool,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub async fn log(&self, event: AuditEvent) {
        if !self.enabled {
            return;
        }
        let event_type = event.event_type();
        match &event {
            AuditEvent::AnonymizeStart { session_id, entity_count } => {
                info!(event_type = event_type, session_id = %Self::redact_session_id(session_id), entity_count = entity_count, "Anonymization started");
            }
            AuditEvent::AnonymizeComplete { session_id, duration_ms } => {
                info!(event_type = event_type, session_id = %Self::redact_session_id(session_id), duration_ms = duration_ms, "Anonymization completed");
            }
            AuditEvent::VaultStore { session_id, entity_type } => {
                info!(event_type = event_type, session_id = %Self::redact_session_id(session_id), entity_type = ?entity_type, "Entity stored");
            }
            AuditEvent::VaultExpire { session_id, mapping_count } => {
                info!(event_type = event_type, session_id = %Self::redact_session_id(session_id), mapping_count = mapping_count, "Session expired");
            }
        }
    }

    fn redact_session_id(session_id: &str) -> String {
        if session_id.len() <= 8 {
            format!("{}****", &session_id[..4.min(session_id.len())])
        } else {
            format!("{}****", &session_id[..8])
        }
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}
