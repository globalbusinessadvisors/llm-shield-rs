//! Tokenizer Integration Tests
//!
//! ## SPARC Phase 3: TDD Red Phase
//!
//! Comprehensive tests for TokenizerWrapper and TokenizerConfig.
//! These tests validate:
//! - Tokenizer creation from pretrained models
//! - Text encoding with truncation
//! - Padding support
//! - Special tokens handling
//! - Thread safety
//! - Error handling

use llm_shield_models::{TokenizerWrapper, TokenizerConfig};

// ============================================================================
// Test 1: TokenizerConfig Creation and Defaults
// ============================================================================

#[test]
fn test_tokenizer_config_default() {
    let config = TokenizerConfig::default();

    assert_eq!(config.max_length, 512);
    assert!(config.padding);
    assert!(config.truncation);
    assert!(config.add_special_tokens);
}

#[test]
fn test_tokenizer_config_custom() {
    let config = TokenizerConfig {
        max_length: 256,
        padding: false,
        truncation: false,
        add_special_tokens: false,
    };

    assert_eq!(config.max_length, 256);
    assert!(!config.padding);
    assert!(!config.truncation);
    assert!(!config.add_special_tokens);
}

// ============================================================================
// Test 2: TokenizerWrapper Creation from HuggingFace
// ============================================================================

#[test]
fn test_tokenizer_from_pretrained_deberta() {
    // DeBERTa tokenizer for PromptInjection model
    let result = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    );

    assert!(result.is_ok(), "Failed to load DeBERTa tokenizer: {:?}", result.err());
}

#[test]
fn test_tokenizer_from_pretrained_roberta() {
    // RoBERTa tokenizer for Toxicity/Sentiment models
    let result = TokenizerWrapper::from_pretrained(
        "roberta-base",
        TokenizerConfig::default(),
    );

    assert!(result.is_ok(), "Failed to load RoBERTa tokenizer: {:?}", result.err());
}

#[test]
fn test_tokenizer_from_invalid_model() {
    let result = TokenizerWrapper::from_pretrained(
        "invalid/nonexistent-model",
        TokenizerConfig::default(),
    );

    assert!(result.is_err());
}

// ============================================================================
// Test 3: Basic Text Encoding
// ============================================================================

#[test]
fn test_encode_simple_text() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let text = "Hello, world!";
    let encoding = tokenizer.encode(text).unwrap();

    // Should have input_ids
    assert!(!encoding.input_ids.is_empty());

    // Should have attention_mask
    assert_eq!(encoding.input_ids.len(), encoding.attention_mask.len());

    // Length should be reasonable for short text
    assert!(encoding.input_ids.len() < 20);
}

#[test]
fn test_encode_empty_string() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let encoding = tokenizer.encode("").unwrap();

    // Empty text should still produce tokens (special tokens)
    assert!(!encoding.input_ids.is_empty());
}

// ============================================================================
// Test 4: Truncation at 512 Tokens
// ============================================================================

#[test]
fn test_truncation_at_max_length() {
    let config = TokenizerConfig {
        max_length: 512,
        truncation: true,
        ..Default::default()
    };

    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        config,
    ).unwrap();

    // Create a very long text (repeat 1000 times)
    let long_text = "This is a test sentence. ".repeat(1000);
    let encoding = tokenizer.encode(&long_text).unwrap();

    // Should be truncated to max_length
    assert!(encoding.input_ids.len() <= 512);
}

#[test]
fn test_truncation_disabled() {
    let config = TokenizerConfig {
        max_length: 512,
        truncation: false,
        ..Default::default()
    };

    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        config,
    ).unwrap();

    // Create a moderately long text
    let text = "This is a test sentence. ".repeat(100);
    let encoding = tokenizer.encode(&text).unwrap();

    // Without truncation, may exceed max_length
    // (exact behavior depends on tokenizer implementation)
    assert!(!encoding.input_ids.is_empty());
}

// ============================================================================
// Test 5: Padding Support
// ============================================================================

#[test]
fn test_padding_to_max_length() {
    let config = TokenizerConfig {
        max_length: 512,
        padding: true,
        truncation: true,
        ..Default::default()
    };

    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        config,
    ).unwrap();

    let short_text = "Hello";
    let encoding = tokenizer.encode(short_text).unwrap();

    // Should be padded to max_length
    assert_eq!(encoding.input_ids.len(), 512);
    assert_eq!(encoding.attention_mask.len(), 512);

    // Attention mask should have 0s for padding
    let padding_count = encoding.attention_mask.iter().filter(|&&x| x == 0).count();
    assert!(padding_count > 0, "Expected padding tokens");
}

#[test]
fn test_padding_disabled() {
    let config = TokenizerConfig {
        max_length: 512,
        padding: false,
        truncation: true,
        ..Default::default()
    };

    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        config,
    ).unwrap();

    let short_text = "Hello";
    let encoding = tokenizer.encode(short_text).unwrap();

    // Should NOT be padded
    assert!(encoding.input_ids.len() < 512);
}

// ============================================================================
// Test 6: Special Tokens Handling
// ============================================================================

#[test]
fn test_special_tokens_added() {
    let config = TokenizerConfig {
        add_special_tokens: true,
        padding: false,
        ..Default::default()
    };

    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        config,
    ).unwrap();

    let encoding = tokenizer.encode("test").unwrap();

    // With special tokens, should have more than just the word tokens
    // (e.g., [CLS], text, [SEP] for BERT-style models)
    assert!(encoding.input_ids.len() >= 3);
}

#[test]
fn test_special_tokens_disabled() {
    let config = TokenizerConfig {
        add_special_tokens: false,
        padding: false,
        truncation: true,
        max_length: 512,
    };

    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        config,
    ).unwrap();

    let encoding = tokenizer.encode("test").unwrap();

    // Without special tokens, should be minimal
    assert!(!encoding.input_ids.is_empty());
}

// ============================================================================
// Test 7: Batch Encoding
// ============================================================================

#[test]
fn test_encode_batch() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let texts = vec![
        "First sentence",
        "Second sentence is longer",
        "Third",
    ];

    let encodings = tokenizer.encode_batch(&texts).unwrap();

    assert_eq!(encodings.len(), 3);

    // All encodings should have the same length (due to padding)
    let first_len = encodings[0].input_ids.len();
    for encoding in &encodings {
        assert_eq!(encoding.input_ids.len(), first_len);
    }
}

#[test]
fn test_encode_batch_empty() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let texts: Vec<&str> = vec![];
    let encodings = tokenizer.encode_batch(&texts).unwrap();

    assert_eq!(encodings.len(), 0);
}

// ============================================================================
// Test 8: Thread Safety
// ============================================================================

#[test]
fn test_tokenizer_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let tokenizer = Arc::new(
        TokenizerWrapper::from_pretrained(
            "microsoft/deberta-v3-base",
            TokenizerConfig::default(),
        ).unwrap()
    );

    let mut handles = vec![];

    for i in 0..4 {
        let tokenizer_clone = Arc::clone(&tokenizer);
        let handle = thread::spawn(move || {
            let text = format!("Test sentence number {}", i);
            tokenizer_clone.encode(&text).unwrap()
        });
        handles.push(handle);
    }

    for handle in handles {
        let encoding = handle.join().unwrap();
        assert!(!encoding.input_ids.is_empty());
    }
}

// ============================================================================
// Test 9: Unicode and Special Characters
// ============================================================================

#[test]
fn test_encode_unicode() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let texts = vec![
        "Hello 世界",  // Chinese
        "Привет мир",  // Russian
        "مرحبا",       // Arabic
        "🚀 Emoji test 🎉",
    ];

    for text in texts {
        let encoding = tokenizer.encode(text).unwrap();
        assert!(!encoding.input_ids.is_empty(), "Failed to encode: {}", text);
    }
}

#[test]
fn test_encode_special_characters() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let text = "Test with\nnewlines\tand\ttabs";
    let encoding = tokenizer.encode(text).unwrap();

    assert!(!encoding.input_ids.is_empty());
}

// ============================================================================
// Test 10: Encoding Properties
// ============================================================================

#[test]
fn test_encoding_properties() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let text = "This is a test sentence";
    let encoding = tokenizer.encode(text).unwrap();

    // Verify all required fields are present
    assert!(!encoding.input_ids.is_empty());
    assert!(!encoding.attention_mask.is_empty());
    assert_eq!(encoding.input_ids.len(), encoding.attention_mask.len());
}

// ============================================================================
// Test 11: Different Model Types
// ============================================================================

#[test]
fn test_different_tokenizer_types() {
    // Test with multiple model types
    let models = vec![
        "microsoft/deberta-v3-base",
        "roberta-base",
    ];

    for model_name in models {
        let result = TokenizerWrapper::from_pretrained(
            model_name,
            TokenizerConfig::default(),
        );

        assert!(result.is_ok(), "Failed to load tokenizer: {}", model_name);

        let tokenizer = result.unwrap();
        let encoding = tokenizer.encode("test").unwrap();
        assert!(!encoding.input_ids.is_empty());
    }
}

// ============================================================================
// Test 12: Config Validation
// ============================================================================

#[test]
fn test_config_validation() {
    // Test that various config combinations work
    let configs = vec![
        TokenizerConfig {
            max_length: 128,
            padding: true,
            truncation: true,
            add_special_tokens: true,
        },
        TokenizerConfig {
            max_length: 256,
            padding: false,
            truncation: true,
            add_special_tokens: true,
        },
        TokenizerConfig {
            max_length: 512,
            padding: true,
            truncation: false,
            add_special_tokens: false,
        },
    ];

    for config in configs {
        let tokenizer = TokenizerWrapper::from_pretrained(
            "microsoft/deberta-v3-base",
            config,
        ).unwrap();

        let encoding = tokenizer.encode("test").unwrap();
        assert!(!encoding.input_ids.is_empty());
    }
}

// ============================================================================
// Test 13: Very Long Text Performance
// ============================================================================

#[test]
fn test_very_long_text() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    // Create very long text (5000 words)
    let long_text = "word ".repeat(5000);
    let encoding = tokenizer.encode(&long_text).unwrap();

    // Should be truncated to max_length (512)
    assert_eq!(encoding.input_ids.len(), 512);
}

// ============================================================================
// Test 14: Encoding Information
// ============================================================================

#[test]
fn test_encoding_to_vec() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    let encoding = tokenizer.encode("test").unwrap();

    // Test conversion to various formats
    let ids_vec = encoding.input_ids.clone();
    let mask_vec = encoding.attention_mask.clone();

    assert!(!ids_vec.is_empty());
    assert_eq!(ids_vec.len(), mask_vec.len());
}

// ============================================================================
// Test 15: Edge Cases
// ============================================================================

#[test]
fn test_edge_cases() {
    let tokenizer = TokenizerWrapper::from_pretrained(
        "microsoft/deberta-v3-base",
        TokenizerConfig::default(),
    ).unwrap();

    // Single character
    let encoding = tokenizer.encode("a").unwrap();
    assert!(!encoding.input_ids.is_empty());

    // Whitespace only
    let encoding = tokenizer.encode("   ").unwrap();
    assert!(!encoding.input_ids.is_empty());

    // Special characters
    let encoding = tokenizer.encode("!@#$%^&*()").unwrap();
    assert!(!encoding.input_ids.is_empty());

    // Numbers
    let encoding = tokenizer.encode("1234567890").unwrap();
    assert!(!encoding.input_ids.is_empty());
}
