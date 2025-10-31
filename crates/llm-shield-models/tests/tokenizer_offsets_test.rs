//! Acceptance tests for tokenizer with character offsets
//!
//! Following London School TDD - test the public API first.

use llm_shield_models::{TokenizerWrapper, TokenizerConfig};

#[test]
#[ignore] // Requires real tokenizer - enable when tokenizer files are available
fn test_tokenizer_produces_offsets() {
    // Given: A text with known character positions
    let text = "Email: john@example.com";

    // When: I tokenize with offsets
    // let tokenizer = TokenizerWrapper::from_pretrained(...).unwrap();
    // let encoding = tokenizer.encode(text).unwrap();

    // Then: Each token has character offsets
    // assert_eq!(encoding.offsets.len(), encoding.input_ids.len());

    // And: Offsets are valid
    // for (i, (start, end)) in encoding.offsets.iter().enumerate() {
    //     assert!(start <= end, "Invalid offset at token {}: {} > {}", i, start, end);
    //     if *start < *end {
    //         let token_text = &text[*start..*end];
    //         assert!(!token_text.is_empty());
    //     }
    // }

    // And: Special tokens have zero-length offsets
    // // CLS token (usually first) should have offset (0, 0)
    // assert_eq!(encoding.offsets[0], (0, 0), "CLS token should have (0,0) offset");
}

#[test]
fn test_encoding_has_offsets_field() {
    // This test verifies the Encoding struct has an offsets field
    // It will fail to compile until we add the field

    // Given: Mock encoding data
    let input_ids = vec![101, 2003, 102]; // [CLS] john [SEP]
    let attention_mask = vec![1, 1, 1];
    let offsets = vec![(0, 0), (0, 4), (0, 0)]; // CLS, "john", SEP

    // When: I create an Encoding
    let encoding = llm_shield_models::Encoding {
        input_ids,
        attention_mask,
        offsets,  // This line will fail to compile until we add the field
    };

    // Then: Offsets are accessible
    assert_eq!(encoding.offsets.len(), 3);
    assert_eq!(encoding.offsets[1], (0, 4));
}

#[test]
fn test_encoding_offset_invariants() {
    // Given: An encoding with offsets
    let encoding = llm_shield_models::Encoding {
        input_ids: vec![101, 2003, 2004, 102],
        attention_mask: vec![1, 1, 1, 1],
        offsets: vec![(0, 0), (0, 4), (5, 10), (0, 0)],
    };

    // Then: All arrays have same length
    assert_eq!(encoding.input_ids.len(), encoding.attention_mask.len());
    assert_eq!(encoding.input_ids.len(), encoding.offsets.len());

    // And: Offsets are ordered and non-overlapping
    for i in 0..encoding.offsets.len() {
        let (start, end) = encoding.offsets[i];
        assert!(start <= end, "start > end at index {}", i);
    }

    // And: Token ranges don't overlap (except special tokens at (0,0))
    for i in 1..encoding.offsets.len() {
        let (prev_start, prev_end) = encoding.offsets[i - 1];
        let (curr_start, _curr_end) = encoding.offsets[i];

        // Skip special tokens (0,0)
        if prev_start != prev_end && curr_start != 0 {
            assert!(
                curr_start >= prev_end,
                "Overlapping offsets: [{}, {}) and [{}, {})",
                prev_start, prev_end, curr_start, _curr_end
            );
        }
    }
}
