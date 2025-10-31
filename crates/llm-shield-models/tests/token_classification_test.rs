//! Acceptance tests for token classification
//!
//! Following London School TDD - these tests drive the implementation.
//! Tests are written from outside-in, starting with the public API.

use llm_shield_models::{
    InferenceEngine, TokenPrediction, TokenizerWrapper, TokenizerConfig,
};
use std::sync::{Arc, Mutex};

#[tokio::test]
#[ignore] // Requires real model - enable after model is loaded
async fn test_token_classification_basic() {
    // Given: A text with PII entities
    let text = "John Smith works at Microsoft";

    // And: A loaded model and tokenizer
    // TODO: Load real model for integration test
    // For now, this test is ignored until we have a real model

    // When: I tokenize the text
    // let tokenizer = TokenizerWrapper::from_pretrained("...", TokenizerConfig::default()).unwrap();
    // let encoding = tokenizer.encode(text).unwrap();

    // And: I run token classification
    // let engine = InferenceEngine::new(session);
    // let labels = vec!["O", "B-PERSON", "I-PERSON", "B-ORG", "I-ORG"];
    // let predictions = engine
    //     .infer_token_classification(&encoding.input_ids, &encoding.attention_mask, &labels)
    //     .await
    //     .unwrap();

    // Then: Each token has a prediction
    // assert_eq!(predictions.len(), encoding.input_ids.len());

    // And: All predictions are valid
    // for pred in &predictions {
    //     assert!(pred.validate().is_ok());
    //     assert!(pred.confidence >= 0.0 && pred.confidence <= 1.0);
    //     assert!(pred.all_scores.len() == labels.len());
    // }

    // And: Some tokens should be classified as PERSON or ORG
    // let person_tokens: Vec<_> = predictions
    //     .iter()
    //     .filter(|p| p.predicted_label.contains("PERSON"))
    //     .collect();
    // let org_tokens: Vec<_> = predictions
    //     .iter()
    //     .filter(|p| p.predicted_label.contains("ORG"))
    //     .collect();

    // assert!(!person_tokens.is_empty(), "Should detect person names");
    // assert!(!org_tokens.is_empty(), "Should detect organizations");
}

#[test]
fn test_token_prediction_invariants() {
    // Given: A valid token prediction
    let prediction = TokenPrediction::new(
        101, // token_id
        "B-PERSON".to_string(),
        1, // predicted_class
        0.95,
        vec![0.03, 0.95, 0.02], // softmax scores sum to 1.0
    );

    // Then: Validation passes
    assert!(prediction.validate().is_ok());
}

#[test]
fn test_token_prediction_invalid_confidence() {
    // Given: A prediction with invalid confidence
    let prediction = TokenPrediction::new(
        101,
        "B-PERSON".to_string(),
        1,
        1.5, // Invalid: >1.0
        vec![0.03, 0.95, 0.02],
    );

    // Then: Validation fails
    assert!(prediction.validate().is_err());
}

#[test]
fn test_token_prediction_confidence_mismatch() {
    // Given: A prediction where confidence doesn't match all_scores
    let prediction = TokenPrediction::new(
        101,
        "B-PERSON".to_string(),
        1,
        0.80, // Doesn't match all_scores[1] = 0.95
        vec![0.03, 0.95, 0.02],
    );

    // Then: Validation fails
    assert!(prediction.validate().is_err());
}

#[test]
fn test_token_prediction_scores_dont_sum_to_one() {
    // Given: A prediction where scores don't sum to 1.0
    let prediction = TokenPrediction::new(
        101,
        "B-PERSON".to_string(),
        1,
        0.95,
        vec![0.05, 0.95, 0.05], // Sum = 1.05 (invalid)
    );

    // Then: Validation fails
    assert!(prediction.validate().is_err());
}

#[test]
fn test_token_prediction_invalid_class_index() {
    // Given: A prediction with out-of-bounds predicted_class
    let prediction = TokenPrediction::new(
        101,
        "B-PERSON".to_string(),
        10, // Invalid: >= all_scores.len()
        0.95,
        vec![0.03, 0.95, 0.02],
    );

    // Then: Validation fails
    assert!(prediction.validate().is_err());
}
