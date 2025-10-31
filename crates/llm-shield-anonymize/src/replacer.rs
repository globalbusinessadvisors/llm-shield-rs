//! Text replacement algorithm for anonymization

use crate::types::EntityMatch;
use crate::{AnonymizationError, Result};

/// Replace entities in text with placeholders using reverse-order algorithm
///
/// This function preserves text indices by replacing from end to start.
/// It handles overlapping entities, preserves whitespace and punctuation,
/// and supports Unicode text.
///
/// # Arguments
/// * `text` - Original text
/// * `entities` - Detected entities (sorted by start position)
/// * `placeholders` - Replacement placeholders (one per entity)
///
/// # Returns
/// Anonymized text with entities replaced by placeholders
pub fn replace_entities(
    text: &str,
    entities: &[EntityMatch],
    placeholders: &[String],
) -> Result<String> {
    // Validate inputs
    if entities.len() != placeholders.len() {
        return Err(AnonymizationError::InvalidRange(format!(
            "Entity count ({}) must match placeholder count ({})",
            entities.len(),
            placeholders.len()
        )));
    }

    if entities.is_empty() {
        return Ok(text.to_string());
    }

    // Validate entity ranges
    for entity in entities {
        if entity.start >= entity.end {
            return Err(AnonymizationError::InvalidRange(format!(
                "Invalid entity range: {}..{}",
                entity.start, entity.end
            )));
        }
        if entity.end > text.len() {
            return Err(AnonymizationError::InvalidRange(format!(
                "Entity end ({}) exceeds text length ({})",
                entity.end,
                text.len()
            )));
        }
    }

    // Clone text as String for in-place modification
    let mut result = text.to_string();

    // Replace in reverse order to preserve indices
    for i in (0..entities.len()).rev() {
        let entity = &entities[i];
        let placeholder = &placeholders[i];

        // Replace the range
        result.replace_range(entity.start..entity.end, placeholder);
    }

    Ok(result)
}

/// Check if two entities overlap
fn entities_overlap(e1: &EntityMatch, e2: &EntityMatch) -> bool {
    e1.start < e2.end && e2.start < e1.end
}

/// Resolve overlapping entities by choosing the highest confidence
pub fn resolve_overlaps(entities: Vec<EntityMatch>) -> Vec<EntityMatch> {
    if entities.is_empty() {
        return entities;
    }

    let mut sorted = entities;
    sorted.sort_by_key(|e| e.start);

    let mut result = vec![];
    let mut current = sorted[0].clone();

    for next in sorted.into_iter().skip(1) {
        if entities_overlap(&current, &next) {
            // Choose entity with higher confidence
            if next.confidence > current.confidence {
                current = next;
            }
            // else keep current
        } else {
            result.push(current);
            current = next;
        }
    }
    result.push(current);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EntityType;

    fn create_entity(entity_type: EntityType, start: usize, end: usize, value: &str) -> EntityMatch {
        EntityMatch {
            entity_type,
            start,
            end,
            value: value.to_string(),
            confidence: 0.95,
        }
    }

    #[test]
    fn test_replace_single_entity() {
        let text = "John at john@example.com";
        let entities = vec![
            create_entity(EntityType::Person, 0, 4, "John"),
            create_entity(EntityType::Email, 8, 24, "john@example.com"),
        ];
        let placeholders = vec!["[PERSON_1]".to_string(), "[EMAIL_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "[PERSON_1] at [EMAIL_1]");
    }

    #[test]
    fn test_reverse_order_replacement() {
        // This test verifies that reverse-order replacement preserves indices
        let text = "Contact John Doe at john@example.com or jane@example.com";
        let entities = vec![
            create_entity(EntityType::Person, 8, 16, "John Doe"),
            create_entity(EntityType::Email, 20, 36, "john@example.com"),
            create_entity(EntityType::Email, 40, 56, "jane@example.com"),
        ];
        let placeholders = vec![
            "[PERSON_1]".to_string(),
            "[EMAIL_1]".to_string(),
            "[EMAIL_2]".to_string(),
        ];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "Contact [PERSON_1] at [EMAIL_1] or [EMAIL_2]");
    }

    #[test]
    fn test_preserve_whitespace() {
        let text = "  John   at   john@example.com  ";
        let entities = vec![
            create_entity(EntityType::Person, 2, 6, "John"),
            create_entity(EntityType::Email, 14, 30, "john@example.com"),
        ];
        let placeholders = vec!["[PERSON_1]".to_string(), "[EMAIL_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "  [PERSON_1]   at   [EMAIL_1]  ");
    }

    #[test]
    fn test_overlapping_entities() {
        // Test with overlapping entities
        let entities = vec![
            create_entity(EntityType::Person, 0, 10, "John Smith"),
            create_entity(EntityType::Person, 5, 10, "Smith"), // Overlaps with above
        ];

        let resolved = resolve_overlaps(entities);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].value, "John Smith");
    }

    #[test]
    fn test_unicode_handling() {
        let text = "こんにちは John さん at john@example.com";
        // Japanese chars are 3 bytes each: "こんにちは " = 5*3 = 15 bytes + 1 space = 16 bytes
        // "John" starts at byte 16, ends at 20
        // " さん at " = 1 + 3 + 3 + 4 = 11 bytes, so email starts at 20 + 11 = 31
        // "john@example.com" is 16 bytes, ends at 31 + 16 = 47
        let entities = vec![
            create_entity(EntityType::Person, 16, 20, "John"),
            create_entity(EntityType::Email, 31, 47, "john@example.com"),
        ];
        let placeholders = vec!["[PERSON_1]".to_string(), "[EMAIL_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "こんにちは [PERSON_1] さん at [EMAIL_1]");
    }

    #[test]
    fn test_empty_entities() {
        let text = "No entities here";
        let entities = vec![];
        let placeholders = vec![];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, text);
    }

    #[test]
    fn test_entity_at_boundaries() {
        // Test entity at start of text
        let text = "john@example.com is the email";
        let entities = vec![create_entity(EntityType::Email, 0, 16, "john@example.com")];
        let placeholders = vec!["[EMAIL_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "[EMAIL_1] is the email");

        // Test entity at end of text
        let text = "Email: john@example.com";
        let entities = vec![create_entity(EntityType::Email, 7, 23, "john@example.com")];
        let placeholders = vec!["[EMAIL_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "Email: [EMAIL_1]");

        // Test entity is entire text
        let text = "john@example.com";
        let entities = vec![create_entity(EntityType::Email, 0, 16, "john@example.com")];
        let placeholders = vec!["[EMAIL_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "[EMAIL_1]");
    }

    #[test]
    fn test_special_characters() {
        let text = "Call John @ +1-555-1234 or email: john@example.com!";
        let entities = vec![
            create_entity(EntityType::Person, 5, 9, "John"),
            create_entity(EntityType::PhoneNumber, 12, 23, "+1-555-1234"),
            create_entity(EntityType::Email, 34, 50, "john@example.com"),
        ];
        let placeholders = vec![
            "[PERSON_1]".to_string(),
            "[PHONE_1]".to_string(),
            "[EMAIL_1]".to_string(),
        ];

        let result = replace_entities(text, &entities, &placeholders).unwrap();
        assert_eq!(result, "Call [PERSON_1] @ [PHONE_1] or email: [EMAIL_1]!");
    }

    #[test]
    fn test_mismatched_counts() {
        let text = "John Doe";
        let entities = vec![create_entity(EntityType::Person, 0, 8, "John Doe")];
        let placeholders = vec!["[PERSON_1]".to_string(), "[PERSON_2]".to_string()];

        let result = replace_entities(text, &entities, &placeholders);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must match"));
    }

    #[test]
    fn test_invalid_range() {
        let text = "John Doe";
        let entities = vec![create_entity(EntityType::Person, 5, 3, "invalid")]; // start > end
        let placeholders = vec!["[PERSON_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders);
        assert!(result.is_err());
    }

    #[test]
    fn test_range_out_of_bounds() {
        let text = "John";
        let entities = vec![create_entity(EntityType::Person, 0, 10, "invalid")]; // end > text.len()
        let placeholders = vec!["[PERSON_1]".to_string()];

        let result = replace_entities(text, &entities, &placeholders);
        assert!(result.is_err());
    }
}
