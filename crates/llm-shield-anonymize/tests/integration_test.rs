//! Integration tests for anonymization

use llm_shield_anonymize::{
    anonymizer::{Anonymizer, AuditLogger, EntityDetector, VaultStorage},
    config::AnonymizerConfig,
    types::{EntityMapping, EntityMatch, EntityType},
    AnonymizeResult, Result,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

// Mock implementations for testing

struct MockDetector {
    entities: Vec<EntityMatch>,
}

impl MockDetector {
    fn new(entities: Vec<EntityMatch>) -> Self {
        Self { entities }
    }

    fn regex_based() -> Self {
        // Simulates regex-based detection for realistic entities
        Self { entities: vec![] }
    }
}

#[async_trait::async_trait]
impl EntityDetector for MockDetector {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        // If we have pre-defined entities, return them
        if !self.entities.is_empty() {
            return Ok(self.entities.clone());
        }

        // Otherwise, do simple pattern matching for demo
        let mut entities = vec![];

        // Detect email pattern
        if let Some(pos) = text.find('@') {
            let start = text[..pos].rfind(' ').map(|p| p + 1).unwrap_or(0);
            let end = text[pos..]
                .find(' ')
                .map(|p| pos + p)
                .unwrap_or(text.len());
            entities.push(EntityMatch {
                entity_type: EntityType::Email,
                start,
                end,
                value: text[start..end].to_string(),
                confidence: 0.98,
            });
        }

        // Detect credit card pattern (simplified)
        if let Some(pos) = text.find("4111") {
            if pos + 19 <= text.len() && text[pos..pos + 19].chars().all(|c| c.is_digit(10) || c == '-') {
                entities.push(EntityMatch {
                    entity_type: EntityType::CreditCard,
                    start: pos,
                    end: pos + 19,
                    value: text[pos..pos + 19].to_string(),
                    confidence: 0.99,
                });
            }
        }

        // Sort by start position
        entities.sort_by_key(|e| e.start);
        Ok(entities)
    }
}

struct MockVault {
    storage: Arc<Mutex<HashMap<String, EntityMapping>>>,
}

impl MockVault {
    fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
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

    async fn get_mapping(
        &self,
        session_id: &str,
        placeholder: &str,
    ) -> Result<Option<EntityMapping>> {
        let key = format!("{}:{}", session_id, placeholder);
        Ok(self.storage.lock().unwrap().get(&key).cloned())
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut storage = self.storage.lock().unwrap();
        storage.retain(|k, _| !k.starts_with(session_id));
        Ok(())
    }
}

struct MockAudit {
    anonymize_calls: Arc<Mutex<Vec<(String, usize, Instant)>>>,
}

impl MockAudit {
    fn new() -> Self {
        Self {
            anonymize_calls: Arc::new(Mutex::new(vec![])),
        }
    }

    #[allow(dead_code)]
    fn get_call_count(&self) -> usize {
        self.anonymize_calls.lock().unwrap().len()
    }

    fn get_latencies(&self) -> Vec<Duration> {
        let calls = self.anonymize_calls.lock().unwrap();
        let first_time = calls.first().map(|(_, _, t)| *t).unwrap_or_else(Instant::now);
        calls
            .iter()
            .map(|(_, _, t)| t.duration_since(first_time))
            .collect()
    }
}

impl AuditLogger for MockAudit {
    fn log_anonymize(&self, session_id: &str, entity_count: usize) {
        self.anonymize_calls
            .lock()
            .unwrap()
            .push((session_id.to_string(), entity_count, Instant::now()));
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
async fn test_end_to_end_anonymization() {
    let text = "Contact John Doe at john@example.com or call 555-1234";
    // Text length is 53 chars. Phone number starts at 45 (after "call "), ends at 53
    let entities = vec![
        create_entity(EntityType::Person, 8, 16, "John Doe"),
        create_entity(EntityType::Email, 20, 36, "john@example.com"),
        create_entity(EntityType::PhoneNumber, 45, 53, "555-1234"),
    ];

    let detector = Arc::new(MockDetector::new(entities));
    let vault = Arc::new(MockVault::new());
    let audit = Arc::new(MockAudit::new());
    let config = AnonymizerConfig::default();

    let anonymizer = Anonymizer::new(config, detector, vault.clone(), audit);
    let result = anonymizer.anonymize(text).await.unwrap();

    assert_eq!(
        result.anonymized_text,
        "Contact [PERSON_1] at [EMAIL_1] or call [PHONE_1]"
    );
    assert_eq!(result.entities.len(), 3);
    assert!(result.session_id.starts_with("sess_"));
}

#[tokio::test]
async fn test_performance_single_anonymization() {
    let text = "John Doe lives at john@example.com";
    let entities = vec![
        create_entity(EntityType::Person, 0, 8, "John Doe"),
        create_entity(EntityType::Email, 18, 34, "john@example.com"),
    ];

    let detector = Arc::new(MockDetector::new(entities));
    let vault = Arc::new(MockVault::new());
    let audit = Arc::new(MockAudit::new());
    let config = AnonymizerConfig::default();

    let anonymizer = Anonymizer::new(config, detector, vault, audit);

    let start = Instant::now();
    let _result = anonymizer.anonymize(text).await.unwrap();
    let elapsed = start.elapsed();

    // Should be very fast (< 10ms for regex-only)
    println!("Single anonymization took: {:?}", elapsed);
    assert!(elapsed.as_millis() < 100); // Very lenient upper bound
}

#[tokio::test]
async fn test_performance_batch_anonymization() {
    let config = AnonymizerConfig::default();
    let detector = Arc::new(MockDetector::regex_based());
    let vault = Arc::new(MockVault::new());
    let audit = Arc::new(MockAudit::new());

    let anonymizer = Anonymizer::new(config, detector, vault, audit.clone());

    let test_cases = vec![
        "Contact me at john@example.com",
        "Card: 4111-1111-1111-1111",
        "Email: alice@test.com",
        "My credit card is 4111-2222-3333-4444",
        "Reach out at bob@company.com",
    ];

    let start = Instant::now();
    for text in test_cases {
        let _result = anonymizer.anonymize(text).await.unwrap();
    }
    let elapsed = start.elapsed();

    println!("Batch of 5 anonymizations took: {:?}", elapsed);
    println!("Average per anonymization: {:?}", elapsed / 5);

    // Should process 5 texts quickly
    assert!(elapsed.as_millis() < 500);
}

#[tokio::test]
async fn test_vault_ttl_configuration() {
    let text = "John at john@example.com";
    let entities = vec![
        create_entity(EntityType::Person, 0, 4, "John"),
        create_entity(EntityType::Email, 8, 24, "john@example.com"),
    ];

    let detector = Arc::new(MockDetector::new(entities));
    let vault = Arc::new(MockVault::new());
    let audit = Arc::new(MockAudit::new());

    let mut config = AnonymizerConfig::default();
    config.vault_ttl = Duration::from_secs(1800); // 30 minutes

    let anonymizer = Anonymizer::new(config, detector, vault.clone(), audit);
    let result = anonymizer.anonymize(text).await.unwrap();

    // Verify TTL is set correctly
    let mapping = vault
        .get_mapping(&result.session_id, "[PERSON_1]")
        .await
        .unwrap()
        .unwrap();

    let expires_at = mapping.expires_at.unwrap();
    let now = SystemTime::now();
    let ttl = expires_at.duration_since(now).unwrap();

    // Should be approximately 30 minutes (with some tolerance)
    assert!(ttl.as_secs() >= 1795); // 29:55
    assert!(ttl.as_secs() <= 1805); // 30:05
}

#[tokio::test]
async fn test_concurrent_anonymization() {
    let config = AnonymizerConfig::default();
    let detector = Arc::new(MockDetector::regex_based());
    let vault = Arc::new(MockVault::new());
    let audit = Arc::new(MockAudit::new());

    let anonymizer = Arc::new(Anonymizer::new(config, detector, vault, audit));

    let mut handles = vec![];

    // Spawn 10 concurrent anonymization tasks
    for i in 0..10 {
        let anonymizer = Arc::clone(&anonymizer);
        let handle = tokio::spawn(async move {
            let text = format!("User {} at user{}@example.com", i, i);
            anonymizer.anonymize(&text).await.unwrap()
        });
        handles.push(handle);
    }

    // Collect results
    let mut results = vec![];
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // All should succeed
    assert_eq!(results.len(), 10);

    // Each should have unique session ID
    let session_ids: std::collections::HashSet<_> =
        results.iter().map(|r| r.session_id.clone()).collect();
    assert_eq!(session_ids.len(), 10);
}

#[tokio::test]
async fn test_accuracy_report() {
    let text = "John Doe (john@example.com) works at Acme Corp. His card is 4111-1111-1111-1111.";

    let entities = vec![
        create_entity(EntityType::Person, 0, 8, "John Doe"),
        create_entity(EntityType::Email, 10, 26, "john@example.com"),
        create_entity(EntityType::Organization, 38, 48, "Acme Corp."),
        create_entity(EntityType::CreditCard, 61, 80, "4111-1111-1111-1111"),
    ];

    let detector = Arc::new(MockDetector::new(entities));
    let vault = Arc::new(MockVault::new());
    let audit = Arc::new(MockAudit::new());
    let config = AnonymizerConfig::default();

    let anonymizer = Anonymizer::new(config, detector, vault, audit);

    let start = Instant::now();
    let result = anonymizer.anonymize(text).await.unwrap();
    let elapsed = start.elapsed();

    println!("\n=== Anonymization Accuracy Report ===");
    println!("Original: {}", text);
    println!("Anonymized: {}", result.anonymized_text);
    println!("Session ID: {}", result.session_id);
    println!("Entities detected: {}", result.entities.len());
    println!("Latency: {:?}", elapsed);
    println!("\nDetected entities:");
    for (i, entity) in result.entities.iter().enumerate() {
        println!(
            "  {}. {:?}: '{}' (confidence: {:.2})",
            i + 1,
            entity.entity_type,
            entity.value,
            entity.confidence
        );
    }
    println!("=====================================\n");

    // Verify all entities were detected
    assert_eq!(result.entities.len(), 4);

    // Verify placeholders are in the output
    assert!(result.anonymized_text.contains("[PERSON_1]"));
    assert!(result.anonymized_text.contains("[EMAIL_1]"));
    assert!(result.anonymized_text.contains("[ORGANIZATION_1]"));
    assert!(result.anonymized_text.contains("[CREDIT_CARD_1]"));
}
