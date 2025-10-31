//! Entity Detector Performance and Accuracy Tests

use llm_shield_anonymize::detector::{regex::RegexDetector, EntityDetector};
use llm_shield_anonymize::types::EntityType;
use std::time::Instant;

#[tokio::test]
async fn test_all_entity_types_detection() {
    let detector = RegexDetector::new();

    // Test text with multiple PII types
    let text = "Contact John Doe at john@example.com or call 555-123-4567. \
                SSN: 123-45-6789. Card: 4111-1111-1111-1111. \
                IP: 192.168.1.1. Visit https://example.com. \
                Born: 01/15/1990. Account: 12345678901. \
                License: A1234567. Passport: B12345678. \
                MRN-654321. ZIP: 12345. \
                Address: 123 Main Street. Works at Acme Corp.";

    let matches = detector.detect(text).await.unwrap();

    println!("\n=== Entity Detection Results ===");
    println!("Total entities detected: {}", matches.len());

    let mut entity_counts = std::collections::HashMap::new();
    for m in &matches {
        *entity_counts.entry(m.entity_type).or_insert(0) += 1;
        println!("{:?}: '{}' (conf: {:.2})", m.entity_type, m.value, m.confidence);
    }

    println!("\n=== Entity Type Summary ===");
    for (entity_type, count) in &entity_counts {
        println!("{:?}: {}", entity_type, count);
    }

    // Verify we detect at least 10 different entities
    assert!(matches.len() >= 10, "Should detect at least 10 entities, found {}", matches.len());

    // Verify we have diverse entity types
    assert!(entity_counts.len() >= 8, "Should detect at least 8 different types, found {}", entity_counts.len());
}

#[tokio::test]
async fn test_entity_detection_accuracy() {
    let detector = RegexDetector::new();

    // Test specific entities
    let test_cases = vec![
        ("john@example.com", EntityType::Email),
        ("555-123-4567", EntityType::PhoneNumber),
        ("123-45-6789", EntityType::SSN),
        ("192.168.1.1", EntityType::IpAddress),
        ("https://example.com", EntityType::Url),
        ("01/15/1990", EntityType::DateOfBirth),
        ("12345", EntityType::PostalCode),
    ];

    for (text, expected_type) in test_cases {
        let matches = detector.detect(text).await.unwrap();
        assert!(!matches.is_empty(), "Should detect entity in '{}'", text);
        assert_eq!(matches[0].entity_type, expected_type,
                   "Wrong entity type for '{}': expected {:?}, got {:?}",
                   text, expected_type, matches[0].entity_type);
    }
}

#[tokio::test]
async fn test_detection_performance_benchmark() {
    let detector = RegexDetector::new();

    let text = "Contact John Doe at john@example.com or 555-123-4567. \
                SSN: 123-45-6789, Card: 4111-1111-1111-1111, IP: 192.168.1.1";

    // Warmup
    let _ = detector.detect(text).await.unwrap();

    // Benchmark
    let iterations = 100;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = detector.detect(text).await.unwrap();
    }

    let duration = start.elapsed();
    let avg_micros = duration.as_micros() / iterations;

    println!("\n=== Performance Metrics ===");
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", duration);
    println!("Average per detection: {} µs", avg_micros);
    println!("Throughput: {:.0} detections/sec", 1_000_000.0 / avg_micros as f64);

    // Should be fast (< 10ms average)
    assert!(avg_micros < 10_000, "Detection too slow: {} µs (should be < 10ms)", avg_micros);
}

#[tokio::test]
async fn test_luhn_credit_card_validation() {
    let detector = RegexDetector::new();

    // Valid Visa card (passes Luhn) - this is a test card number
    let valid = detector.detect("4111-1111-1111-1111").await.unwrap();
    assert_eq!(valid.len(), 1, "Should detect valid credit card");
    assert_eq!(valid[0].entity_type, EntityType::CreditCard);

    // Invalid card (fails Luhn)
    let invalid = detector.detect("1234-5678-9012-3456").await.unwrap();
    assert_eq!(invalid.len(), 0, "Should reject invalid credit card");
}

#[tokio::test]
async fn test_ssn_validation() {
    let detector = RegexDetector::new();

    // Valid SSN
    let valid = detector.detect("123-45-6789").await.unwrap();
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0].entity_type, EntityType::SSN);

    // Invalid SSN (area = 000)
    let invalid = detector.detect("000-45-6789").await.unwrap();
    assert_eq!(invalid.len(), 0, "Should reject SSN with area=000");
}

#[tokio::test]
async fn test_overlap_handling() {
    let detector = RegexDetector::new();

    let text = "John Doe at john@example.com";
    let matches = detector.detect(text).await.unwrap();

    // Should handle overlaps correctly
    for i in 0..matches.len().saturating_sub(1) {
        assert!(matches[i].end <= matches[i + 1].start,
                "Entities should not overlap");
    }
}
