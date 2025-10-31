//! Placeholder generation for anonymization

use crate::types::{EntityMatch, EntityType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Generator for unique placeholders per session
pub struct PlaceholderGenerator {
    /// Session-scoped counters for each entity type
    counters: Arc<Mutex<HashMap<EntityType, usize>>>,
    /// Unique session identifier
    session_id: String,
}

impl PlaceholderGenerator {
    /// Create a new placeholder generator with a unique session ID
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            session_id: Self::generate_session_id(),
        }
    }

    /// Create a generator with a specific session ID (for testing)
    pub fn with_session_id(session_id: String) -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            session_id,
        }
    }

    /// Generate a unique session ID using UUID v4
    fn generate_session_id() -> String {
        format!("sess_{}", Uuid::new_v4().to_string().replace("-", "")[..12].to_string())
    }

    /// Get the session ID for this generator
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Generate a numbered placeholder for an entity
    ///
    /// Format: [ENTITY_TYPE_N] where N is an incrementing counter
    pub fn generate(&self, entity: &EntityMatch) -> String {
        let mut counters = self.counters.lock().unwrap();
        let counter = counters.entry(entity.entity_type).or_insert(0);
        *counter += 1;
        format!("[{}_{}]", entity.entity_type.as_str(), counter)
    }

    /// Generate placeholders for multiple entities
    pub fn generate_batch(&self, entities: &[EntityMatch]) -> Vec<String> {
        entities.iter().map(|e| self.generate(e)).collect()
    }

    /// Reset counters (useful for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        let mut counters = self.counters.lock().unwrap();
        counters.clear();
    }
}

impl Default for PlaceholderGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_entity(entity_type: EntityType, value: &str) -> EntityMatch {
        EntityMatch {
            entity_type,
            start: 0,
            end: value.len(),
            value: value.to_string(),
            confidence: 0.95,
        }
    }

    #[test]
    fn test_generate_numbered_placeholder() {
        let gen = PlaceholderGenerator::new();
        let entity = create_entity(EntityType::Person, "John Doe");
        let placeholder = gen.generate(&entity);
        assert_eq!(placeholder, "[PERSON_1]");
    }

    #[test]
    fn test_counter_increments() {
        let gen = PlaceholderGenerator::new();
        let entity1 = create_entity(EntityType::Person, "John Doe");
        let entity2 = create_entity(EntityType::Person, "Jane Smith");

        let placeholder1 = gen.generate(&entity1);
        let placeholder2 = gen.generate(&entity2);

        assert_eq!(placeholder1, "[PERSON_1]");
        assert_eq!(placeholder2, "[PERSON_2]");
    }

    #[test]
    fn test_multiple_entity_types() {
        let gen = PlaceholderGenerator::new();
        let person = create_entity(EntityType::Person, "John Doe");
        let email = create_entity(EntityType::Email, "john@example.com");
        let credit_card = create_entity(EntityType::CreditCard, "4111111111111111");

        let p1 = gen.generate(&person);
        let e1 = gen.generate(&email);
        let c1 = gen.generate(&credit_card);
        let p2 = gen.generate(&person);

        assert_eq!(p1, "[PERSON_1]");
        assert_eq!(e1, "[EMAIL_1]");
        assert_eq!(c1, "[CREDIT_CARD_1]");
        assert_eq!(p2, "[PERSON_2]");
    }

    #[test]
    fn test_session_scoped_counters() {
        let gen1 = PlaceholderGenerator::new();
        let gen2 = PlaceholderGenerator::new();

        let entity = create_entity(EntityType::Person, "John Doe");

        let p1 = gen1.generate(&entity);
        let p2 = gen2.generate(&entity);

        // Both should start at 1 since they have different sessions
        assert_eq!(p1, "[PERSON_1]");
        assert_eq!(p2, "[PERSON_1]");

        // Different session IDs
        assert_ne!(gen1.session_id(), gen2.session_id());
    }

    #[test]
    fn test_unique_session_ids() {
        let gen1 = PlaceholderGenerator::new();
        let gen2 = PlaceholderGenerator::new();
        let gen3 = PlaceholderGenerator::new();

        let id1 = gen1.session_id();
        let id2 = gen2.session_id();
        let id3 = gen3.session_id();

        // All should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        // Should start with "sess_"
        assert!(id1.starts_with("sess_"));
        assert!(id2.starts_with("sess_"));
        assert!(id3.starts_with("sess_"));

        // Should be reasonable length (sess_ + 12 chars)
        assert_eq!(id1.len(), 17);
    }

    #[test]
    fn test_thread_safe_generation() {
        use std::thread;

        let gen = Arc::new(PlaceholderGenerator::new());
        let mut handles = vec![];

        // Spawn 10 threads, each generating 10 placeholders
        for _ in 0..10 {
            let gen_clone = Arc::clone(&gen);
            let handle = thread::spawn(move || {
                let mut results = vec![];
                for _ in 0..10 {
                    let entity = create_entity(EntityType::Person, "Test");
                    results.push(gen_clone.generate(&entity));
                }
                results
            });
            handles.push(handle);
        }

        // Collect all results
        let mut all_placeholders = vec![];
        for handle in handles {
            all_placeholders.extend(handle.join().unwrap());
        }

        // Should have 100 unique placeholders
        assert_eq!(all_placeholders.len(), 100);

        // Extract numbers from placeholders
        let mut numbers: Vec<usize> = all_placeholders
            .iter()
            .filter_map(|p| {
                p.strip_prefix("[PERSON_")
                    .and_then(|s| s.strip_suffix("]"))
                    .and_then(|s| s.parse().ok())
            })
            .collect();

        numbers.sort();

        // Should have numbers 1-100
        assert_eq!(numbers.len(), 100);
        assert_eq!(numbers[0], 1);
        assert_eq!(numbers[99], 100);
    }

    #[test]
    fn test_generate_batch() {
        let gen = PlaceholderGenerator::new();
        let entities = vec![
            create_entity(EntityType::Person, "John"),
            create_entity(EntityType::Email, "john@example.com"),
            create_entity(EntityType::Person, "Jane"),
        ];

        let placeholders = gen.generate_batch(&entities);

        assert_eq!(placeholders.len(), 3);
        assert_eq!(placeholders[0], "[PERSON_1]");
        assert_eq!(placeholders[1], "[EMAIL_1]");
        assert_eq!(placeholders[2], "[PERSON_2]");
    }
}
