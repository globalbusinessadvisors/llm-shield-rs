//! Main Anonymizer component

use crate::config::AnonymizerConfig;
use crate::placeholder::PlaceholderGenerator;
use crate::replacer::replace_entities;
use crate::types::{EntityMapping, EntityMatch};
use crate::Result;
use std::sync::Arc;
use std::time::SystemTime;

/// Trait for entity detection
#[async_trait::async_trait]
pub trait EntityDetector: Send + Sync {
    /// Detect entities in the given text
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>>;
}

/// Trait for vault storage
#[async_trait::async_trait]
pub trait VaultStorage: Send + Sync {
    /// Store an entity mapping
    async fn store_mapping(&self, session_id: &str, mapping: EntityMapping) -> Result<()>;

    /// Retrieve an entity mapping
    async fn get_mapping(&self, session_id: &str, placeholder: &str) -> Result<Option<EntityMapping>>;

    /// Delete all mappings for a session
    async fn delete_session(&self, session_id: &str) -> Result<()>;
}

/// Trait for audit logging
pub trait AuditLogger: Send + Sync {
    /// Log an anonymization event
    fn log_anonymize(&self, session_id: &str, entity_count: usize);

    /// Log a deanonymization event
    fn log_deanonymize(&self, session_id: &str, entity_count: usize);
}

/// Result of an anonymization operation
#[derive(Debug, Clone, PartialEq)]
pub struct AnonymizeResult {
    /// Anonymized text with placeholders
    pub anonymized_text: String,
    /// Unique session ID for this anonymization
    pub session_id: String,
    /// Entities that were detected and replaced
    pub entities: Vec<EntityMatch>,
}

/// Main anonymizer component
pub struct Anonymizer {
    config: AnonymizerConfig,
    detector: Arc<dyn EntityDetector>,
    vault: Arc<dyn VaultStorage>,
    audit: Arc<dyn AuditLogger>,
}

impl Anonymizer {
    /// Create a new Anonymizer
    pub fn new(
        config: AnonymizerConfig,
        detector: Arc<dyn EntityDetector>,
        vault: Arc<dyn VaultStorage>,
        audit: Arc<dyn AuditLogger>,
    ) -> Self {
        Self {
            config,
            detector,
            vault,
            audit,
        }
    }

    /// Anonymize text by detecting and replacing PII entities
    pub async fn anonymize(&self, text: &str) -> Result<AnonymizeResult> {
        // 1. Detect entities
        let entities = self.detector.detect(text).await?;

        // Early return if no entities found
        if entities.is_empty() {
            let generator = PlaceholderGenerator::new();
            return Ok(AnonymizeResult {
                anonymized_text: text.to_string(),
                session_id: generator.session_id().to_string(),
                entities: vec![],
            });
        }

        // 2. Generate session and placeholders
        let generator = PlaceholderGenerator::new();
        let session_id = generator.session_id().to_string();
        let placeholders = generator.generate_batch(&entities);

        // 3. Replace text (reverse order to preserve indices)
        let anonymized_text = replace_entities(text, &entities, &placeholders)?;

        // 4. Store in vault
        let now = SystemTime::now();
        let expires_at = now + self.config.vault_ttl;

        for (i, entity) in entities.iter().enumerate() {
            let mapping = EntityMapping {
                entity_type: entity.entity_type,
                original_value: entity.value.clone(),
                placeholder: placeholders[i].clone(),
                confidence: entity.confidence,
                timestamp: now,
                expires_at: Some(expires_at),
            };
            self.vault.store_mapping(&session_id, mapping).await?;
        }

        // 5. Audit log
        self.audit.log_anonymize(&session_id, entities.len());

        Ok(AnonymizeResult {
            anonymized_text,
            session_id,
            entities,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EntityType;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock detector for testing
    struct MockDetector {
        entities: Vec<EntityMatch>,
    }

    impl MockDetector {
        fn new(entities: Vec<EntityMatch>) -> Self {
            Self { entities }
        }
    }

    #[async_trait::async_trait]
    impl EntityDetector for MockDetector {
        async fn detect(&self, _text: &str) -> Result<Vec<EntityMatch>> {
            Ok(self.entities.clone())
        }
    }

    // Mock vault for testing
    struct MockVault {
        storage: Arc<Mutex<HashMap<String, EntityMapping>>>,
    }

    impl MockVault {
        fn new() -> Self {
            Self {
                storage: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        fn get_stored_count(&self) -> usize {
            self.storage.lock().unwrap().len()
        }
    }

    #[async_trait::async_trait]
    impl VaultStorage for MockVault {
        async fn store_mapping(&self, session_id: &str, mapping: EntityMapping) -> Result<()> {
            let key = format!("{}:{}", session_id, mapping.placeholder);
            self.storage.lock().unwrap().insert(key, mapping);
            Ok(())
        }

        async fn get_mapping(&self, session_id: &str, placeholder: &str) -> Result<Option<EntityMapping>> {
            let key = format!("{}:{}", session_id, placeholder);
            Ok(self.storage.lock().unwrap().get(&key).cloned())
        }

        async fn delete_session(&self, _session_id: &str) -> Result<()> {
            Ok(())
        }
    }

    // Mock audit logger
    struct MockAudit {
        anonymize_calls: Arc<Mutex<Vec<(String, usize)>>>,
    }

    impl MockAudit {
        fn new() -> Self {
            Self {
                anonymize_calls: Arc::new(Mutex::new(vec![])),
            }
        }

        fn get_call_count(&self) -> usize {
            self.anonymize_calls.lock().unwrap().len()
        }
    }

    impl AuditLogger for MockAudit {
        fn log_anonymize(&self, session_id: &str, entity_count: usize) {
            self.anonymize_calls
                .lock()
                .unwrap()
                .push((session_id.to_string(), entity_count));
        }

        fn log_deanonymize(&self, _session_id: &str, _entity_count: usize) {}
    }

    fn create_entity(entity_type: EntityType, start: usize, end: usize, value: &str) -> EntityMatch {
        EntityMatch {
            entity_type,
            start,
            end,
            value: value.to_string(),
            confidence: 0.95,
        }
    }

    #[tokio::test]
    async fn test_anonymize_single_entity() {
        let entities = vec![create_entity(EntityType::Person, 0, 8, "John Doe")];
        let detector = Arc::new(MockDetector::new(entities));
        let vault = Arc::new(MockVault::new());
        let audit = Arc::new(MockAudit::new());
        let config = AnonymizerConfig::default();

        let anonymizer = Anonymizer::new(config, detector, vault.clone(), audit);
        let result = anonymizer.anonymize("John Doe").await.unwrap();

        assert!(result.anonymized_text.contains("[PERSON_1]"));
        assert_eq!(result.entities.len(), 1);
        assert!(result.session_id.starts_with("sess_"));
        assert_eq!(vault.get_stored_count(), 1);
    }

    #[tokio::test]
    async fn test_anonymize_multiple_entities() {
        let text = "John Doe at john@example.com";
        let entities = vec![
            create_entity(EntityType::Person, 0, 8, "John Doe"),
            create_entity(EntityType::Email, 12, 28, "john@example.com"),
        ];
        let detector = Arc::new(MockDetector::new(entities));
        let vault = Arc::new(MockVault::new());
        let audit = Arc::new(MockAudit::new());
        let config = AnonymizerConfig::default();

        let anonymizer = Anonymizer::new(config, detector, vault.clone(), audit);
        let result = anonymizer.anonymize(text).await.unwrap();

        assert_eq!(result.anonymized_text, "[PERSON_1] at [EMAIL_1]");
        assert_eq!(result.entities.len(), 2);
        assert_eq!(vault.get_stored_count(), 2);
    }

    #[tokio::test]
    async fn test_anonymize_no_entities() {
        let detector = Arc::new(MockDetector::new(vec![]));
        let vault = Arc::new(MockVault::new());
        let audit = Arc::new(MockAudit::new());
        let config = AnonymizerConfig::default();

        let anonymizer = Anonymizer::new(config, detector, vault.clone(), audit);
        let result = anonymizer
            .anonymize("No PII here")
            .await
            .unwrap();

        assert_eq!(result.anonymized_text, "No PII here");
        assert_eq!(result.entities.len(), 0);
        assert_eq!(vault.get_stored_count(), 0);
    }

    #[tokio::test]
    async fn test_vault_storage() {
        let text = "John Doe";
        let entities = vec![create_entity(EntityType::Person, 0, 8, "John Doe")];
        let detector = Arc::new(MockDetector::new(entities));
        let vault = Arc::new(MockVault::new());
        let audit = Arc::new(MockAudit::new());
        let config = AnonymizerConfig::default();

        let anonymizer = Anonymizer::new(config, detector, vault.clone(), audit);
        let result = anonymizer.anonymize(text).await.unwrap();

        // Verify mapping was stored
        let mapping = vault
            .get_mapping(&result.session_id, "[PERSON_1]")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(mapping.original_value, "John Doe");
        assert_eq!(mapping.placeholder, "[PERSON_1]");
        assert_eq!(mapping.entity_type, EntityType::Person);
        assert!(mapping.expires_at.is_some());
    }

    #[tokio::test]
    async fn test_audit_logging() {
        let text = "John at john@example.com";
        let entities = vec![
            create_entity(EntityType::Person, 0, 4, "John"),
            create_entity(EntityType::Email, 8, 24, "john@example.com"),
        ];
        let detector = Arc::new(MockDetector::new(entities));
        let vault = Arc::new(MockVault::new());
        let audit = Arc::new(MockAudit::new());
        let config = AnonymizerConfig::default();

        let anonymizer = Anonymizer::new(config, detector, vault, audit.clone());
        let _result = anonymizer.anonymize(text).await.unwrap();

        assert_eq!(audit.get_call_count(), 1);
    }
}
