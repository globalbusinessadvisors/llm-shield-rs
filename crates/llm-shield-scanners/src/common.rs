//! Common utilities for scanners

use regex::Regex;
use std::sync::OnceLock;

/// Get a static regex pattern (cached)
pub fn get_regex(pattern: &str) -> Result<&'static Regex, regex::Error> {
    static CACHE: OnceLock<std::collections::HashMap<String, Regex>> = OnceLock::new();

    let cache = CACHE.get_or_init(|| std::collections::HashMap::new());

    // This is a simplified version - in production use a proper cache
    // For now, we'll just compile each time
    // TODO: Implement proper caching with dashmap or similar
    let re = Regex::new(pattern)?;
    Ok(Box::leak(Box::new(re)))
}

/// Normalize text for comparison
pub fn normalize_text(text: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    }
}

/// Calculate text entropy (for gibberish detection)
pub fn calculate_entropy(text: &str) -> f32 {
    use std::collections::HashMap;

    if text.is_empty() {
        return 0.0;
    }

    let mut char_counts = HashMap::new();
    let total_chars = text.len() as f32;

    for c in text.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    let mut entropy = 0.0;
    for count in char_counts.values() {
        let probability = *count as f32 / total_chars;
        if probability > 0.0 {
            entropy -= probability * probability.log2();
        }
    }

    entropy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("Hello World", false), "hello world");
        assert_eq!(normalize_text("Hello World", true), "Hello World");
    }

    #[test]
    fn test_calculate_entropy() {
        // Repeating text has low entropy
        assert!(calculate_entropy("aaaaaaa") < 1.0);

        // Random-looking text has higher entropy
        assert!(calculate_entropy("abcdefghijk") > 2.0);
    }
}
