//! Configuration for anonymization

use crate::types::EntityType;
use std::time::Duration;

/// Configuration for the Anonymizer component
#[derive(Debug, Clone)]
pub struct AnonymizerConfig {
    /// Entity types to detect and anonymize
    pub entity_types: Vec<EntityType>,
    /// Placeholder format to use
    pub placeholder_format: PlaceholderFormat,
    /// Time-to-live for vault mappings
    pub vault_ttl: Duration,
}

impl Default for AnonymizerConfig {
    fn default() -> Self {
        Self {
            entity_types: vec![
                EntityType::Person,
                EntityType::Email,
                EntityType::CreditCard,
                EntityType::SSN,
                EntityType::PhoneNumber,
            ],
            placeholder_format: PlaceholderFormat::Numbered,
            vault_ttl: Duration::from_secs(3600), // 1 hour
        }
    }
}

/// Format for generated placeholders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceholderFormat {
    /// Numbered format: [PERSON_1], [EMAIL_1]
    Numbered,
    /// UUID format: [PERSON_uuid]
    Uuid,
    /// Hashed format: [PERSON_hash]
    Hashed,
}
