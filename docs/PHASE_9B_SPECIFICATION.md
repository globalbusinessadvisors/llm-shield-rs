# Phase 9B: NER-based PII Detection - SPARC Specification

**Date:** 2025-10-31
**Methodology:** SPARC + London School TDD
**Target:** Enterprise-grade, commercially viable NER implementation

---

## SPARC Phase 1: Specification

### 1.1 Business Requirements

**Objective:** Implement ML-based Named Entity Recognition for PII detection with 95-99% accuracy, replacing low-accuracy regex patterns for contextual entities (person names, organizations, addresses).

**Success Criteria:**
- ✅ Detect 15+ PII entity types with 95%+ accuracy
- ✅ Process 200+ texts/second (batch mode)
- ✅ Inference latency <5ms per text (FP16 model)
- ✅ Memory footprint <1GB (model + runtime)
- ✅ Thread-safe, production-ready API
- ✅ 100+ comprehensive tests with 80%+ coverage

### 1.2 Functional Requirements

#### FR-1: Token Classification Inference
**As a** developer
**I want** to run token-level classification on input text
**So that** each token gets assigned a BIO tag with confidence score

**Acceptance Criteria:**
```rust
// Given: A text with PII
let text = "John Smith works at Microsoft in Seattle";

// When: I run token classification
let predictions = engine.infer_token_classification(
    &input_ids,
    &attention_mask,
    &labels
).await?;

// Then: Each token has a prediction
assert_eq!(predictions.len(), input_ids.len());
assert!(predictions[0].predicted_label.starts_with("B-") ||
        predictions[0].predicted_label == "O");
assert!(predictions[0].confidence >= 0.0 && predictions[0].confidence <= 1.0);
```

#### FR-2: Token Offset Tracking
**As a** developer
**I want** to map tokens back to character positions in original text
**So that** I can extract exact entity text from input

**Acceptance Criteria:**
```rust
// Given: A text with entities
let text = "Email: john@example.com";

// When: I tokenize with offsets
let encoding = tokenizer.encode(text)?;

// Then: Each token has character positions
assert_eq!(encoding.offsets.len(), encoding.input_ids.len());
assert_eq!(&text[encoding.offsets[2].0..encoding.offsets[2].1], "john");
```

#### FR-3: BIO Tag to Entity Conversion
**As a** developer
**I want** to merge consecutive BIO tags into entities
**So that** I get complete entity spans with accurate boundaries

**Acceptance Criteria:**
```rust
// Given: BIO-tagged tokens
let tagged = vec![
    TaggedToken { tag: BioTag::Begin(Person), text: "John", start: 0, end: 4, ... },
    TaggedToken { tag: BioTag::Inside(Person), text: "Smith", start: 5, end: 10, ... },
];

// When: I decode BIO tags
let entities = detector.decode_bio_tags(text, tagged)?;

// Then: I get merged entities
assert_eq!(entities.len(), 1);
assert_eq!(entities[0].entity_type, EntityType::Person);
assert_eq!(entities[0].value, "John Smith");
assert_eq!(entities[0].start, 0);
assert_eq!(entities[0].end, 10);
```

#### FR-4: End-to-End NER Detection
**As a** developer
**I want** to detect PII entities in text
**So that** I can anonymize sensitive information

**Acceptance Criteria:**
```rust
// Given: A NerDetector with loaded model
let detector = NerDetector::new(engine, tokenizer, config);

// When: I detect entities
let entities = detector.detect(
    "John Smith (john.smith@example.com) lives at 123 Main St"
).await?;

// Then: All entities are detected
assert!(entities.iter().any(|e| e.entity_type == EntityType::Person));
assert!(entities.iter().any(|e| e.entity_type == EntityType::Email));
assert!(entities.iter().any(|e| e.entity_type == EntityType::Address));
assert!(entities.iter().all(|e| e.confidence >= 0.85));
```

### 1.3 Non-Functional Requirements

#### NFR-1: Performance
- Inference latency: <5ms per text (512 tokens, FP16)
- Throughput: 200+ texts/second (batch size 32)
- Cold start: <2s (model loading + warmup)
- Memory: <1GB total (model 220MB + runtime 100MB + overhead)

#### NFR-2: Accuracy
- Precision: ≥95% (few false positives)
- Recall: ≥95% (few false negatives)
- F1 Score: ≥95%
- Support: 15+ entity types from ai4privacy model

#### NFR-3: Reliability
- Thread-safe: Concurrent access from multiple threads
- Error handling: Graceful failures with clear error messages
- Deterministic: Same input produces same output
- Resilient: Handle edge cases (empty text, very long text, special characters)

#### NFR-4: Maintainability
- Test coverage: ≥80%
- Documentation: API docs for all public types
- Code quality: 0 clippy warnings, follows Rust idioms
- Modularity: Clear separation of concerns

### 1.4 Interface Specifications

#### Interface 1: TokenPrediction

```rust
/// Prediction for a single token in token classification
#[derive(Debug, Clone, PartialEq)]
pub struct TokenPrediction {
    /// Token ID from vocabulary
    pub token_id: u32,

    /// Predicted label (e.g., "B-PERSON", "I-EMAIL", "O")
    pub predicted_label: String,

    /// Index of predicted class
    pub predicted_class: usize,

    /// Confidence score for predicted class (0.0-1.0)
    pub confidence: f32,

    /// Probability distribution over all classes (after softmax)
    pub all_scores: Vec<f32>,
}
```

**Invariants:**
- `confidence` must be in [0.0, 1.0]
- `predicted_class` must be valid index into `all_scores`
- `all_scores.len()` must equal number of labels
- `all_scores[predicted_class]` must equal `confidence`
- Sum of `all_scores` must be ~1.0 (within floating point precision)

#### Interface 2: InferenceEngine Extensions

```rust
impl InferenceEngine {
    /// Run token-level classification inference
    ///
    /// # Arguments
    /// * `input_ids` - Token IDs from tokenizer
    /// * `attention_mask` - Attention mask (1=real token, 0=padding)
    /// * `labels` - BIO tag labels (e.g., ["O", "B-PERSON", "I-PERSON", ...])
    ///
    /// # Returns
    /// Vector of predictions, one per input token
    ///
    /// # Errors
    /// - Model inference failure
    /// - Invalid tensor shapes
    /// - Label count mismatch
    pub async fn infer_token_classification(
        &self,
        input_ids: &[u32],
        attention_mask: &[u32],
        labels: &[String],
    ) -> Result<Vec<TokenPrediction>>;
}
```

**Preconditions:**
- `input_ids.len() == attention_mask.len()`
- `input_ids.len() > 0`
- `labels.len() > 0`
- Model is loaded and session is valid

**Postconditions:**
- `result.len() == input_ids.len()`
- All predictions have valid confidence scores
- All predictions reference valid label indices

#### Interface 3: Encoding with Offsets

```rust
/// Enhanced encoding with character offsets
#[derive(Debug, Clone)]
pub struct Encoding {
    /// Token IDs (vocabulary indices)
    pub input_ids: Vec<u32>,

    /// Attention mask (1 for real tokens, 0 for padding)
    pub attention_mask: Vec<u32>,

    /// Character offsets in original text for each token
    /// (start_char, end_char) for each token
    pub offsets: Vec<(usize, usize)>,
}
```

**Invariants:**
- `input_ids.len() == attention_mask.len() == offsets.len()`
- For each offset `(start, end)`: `start <= end`
- Offsets are non-overlapping and ordered
- Special tokens (CLS, SEP, PAD) have offset (0, 0)

#### Interface 4: TaggedToken (Internal)

```rust
/// Token with BIO tag and position information
#[derive(Debug, Clone)]
struct TaggedToken {
    /// Token text
    text: String,

    /// Start character position in original text
    start: usize,

    /// End character position in original text
    end: usize,

    /// BIO tag assignment
    tag: BioTag,

    /// Confidence score for this tag
    confidence: f32,
}
```

### 1.5 Test Strategy (London School TDD)

#### Unit Tests (with mocks)
- `InferenceEngine::infer_token_classification()` - Mock ONNX session
- `TokenizerWrapper::encode()` with offsets - Mock tokenizer
- `NerDetector::decode_bio_tags()` - Pure function, no mocks needed
- `BioTag::from_str()` - Pure function

#### Integration Tests (real components)
- End-to-end NER detection with real model
- TokenPrediction -> TaggedToken -> EntityMatch pipeline
- Batch processing with multiple texts
- Error handling (invalid input, model failures)

#### Acceptance Tests (user scenarios)
- Detect person names in various formats
- Detect emails, phones, addresses
- Handle overlapping entities
- Filter by confidence threshold
- Performance benchmarks

### 1.6 Dependencies

**Internal:**
- llm-shield-core: Error, Result types
- llm-shield-models: InferenceEngine, TokenizerWrapper, ModelLoader
- ort: ONNX Runtime session
- tokenizers: HuggingFace tokenizers

**External:**
- tokio: Async runtime
- ndarray: Tensor operations
- serde: Serialization

### 1.7 Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| ONNX model shape mismatch | Medium | High | Write tests with mock tensors first |
| Token offset mapping errors | High | High | Extensive unit tests with edge cases |
| Performance below 200 TPS | Medium | Medium | Profile and optimize hot paths, use INT8 |
| Model accuracy <95% | Low | High | Validate with benchmark dataset, fine-tune if needed |
| Memory usage >1GB | Low | Medium | Monitor during tests, implement model quantization |

---

## SPARC Phase 2: Pseudocode

### 2.1 Token Classification Algorithm

```
FUNCTION infer_token_classification(input_ids, attention_mask, labels):
    // Step 1: Prepare inputs
    batch_size = 1
    seq_length = length(input_ids)

    // Step 2: Convert to i64 for ONNX
    input_ids_i64 = map(input_ids, |x| x as i64)
    attention_mask_i64 = map(attention_mask, |x| x as i64)

    // Step 3: Create 2D arrays [batch_size, seq_length]
    input_array = Array2::from_shape_vec((batch_size, seq_length), input_ids_i64)
    mask_array = Array2::from_shape_vec((batch_size, seq_length), attention_mask_i64)

    // Step 4: Run ONNX inference
    outputs = session.run({
        "input_ids": input_array,
        "attention_mask": mask_array
    })

    // Step 5: Extract logits [batch_size, seq_length, num_labels]
    logits = outputs["logits"].extract_tensor()

    // Step 6: For each token, apply softmax and get prediction
    predictions = []
    FOR i in 0..seq_length:
        token_logits = logits[0, i, :]  // Get logits for token i
        scores = softmax(token_logits)

        (predicted_class, max_score) = argmax(scores)
        predicted_label = labels[predicted_class]

        predictions.push(TokenPrediction {
            token_id: input_ids[i],
            predicted_label: predicted_label,
            predicted_class: predicted_class,
            confidence: max_score,
            all_scores: scores
        })

    RETURN predictions
END FUNCTION
```

### 2.2 NER Detection Pipeline

```
FUNCTION detect_entities(text):
    // Step 1: Tokenize with offsets
    encoding = tokenizer.encode(text)
    // encoding contains: input_ids, attention_mask, offsets

    // Step 2: Run token classification
    predictions = inference_engine.infer_token_classification(
        encoding.input_ids,
        encoding.attention_mask,
        BIO_LABELS
    )

    // Step 3: Build tagged tokens with positions
    tagged_tokens = []
    FOR i in 0..predictions.len():
        (start_char, end_char) = encoding.offsets[i]

        // Skip special tokens (CLS, SEP, PAD)
        IF start_char == end_char:
            CONTINUE

        token_text = text[start_char..end_char]
        bio_tag = BioTag::from_str(predictions[i].predicted_label)

        tagged_tokens.push(TaggedToken {
            text: token_text,
            start: start_char,
            end: end_char,
            tag: bio_tag,
            confidence: predictions[i].confidence
        })

    // Step 4: Decode BIO tags to entities
    entities = decode_bio_tags(text, tagged_tokens)

    // Step 5: Filter by confidence threshold
    entities = filter(entities, |e| e.confidence >= threshold)

    RETURN entities
END FUNCTION
```

### 2.3 BIO Decoding Algorithm

```
FUNCTION decode_bio_tags(text, tagged_tokens):
    entities = []
    current_entity = None

    FOR token in tagged_tokens:
        MATCH token.tag:
            CASE Begin(entity_type):
                // Finalize previous entity if exists
                IF current_entity is not None:
                    entity = finalize_entity(current_entity)
                    IF entity.confidence >= threshold:
                        entities.push(entity)

                // Start new entity
                current_entity = {
                    type: entity_type,
                    start: token.start,
                    end: token.end,
                    confidences: [token.confidence]
                }

            CASE Inside(entity_type):
                IF current_entity is not None AND current_entity.type == entity_type:
                    // Extend current entity
                    current_entity.end = token.end
                    current_entity.confidences.push(token.confidence)
                ELSE:
                    // Entity type mismatch - finalize and start new
                    IF current_entity is not None:
                        entity = finalize_entity(current_entity)
                        IF entity.confidence >= threshold:
                            entities.push(entity)

                    current_entity = {
                        type: entity_type,
                        start: token.start,
                        end: token.end,
                        confidences: [token.confidence]
                    }

            CASE Outside:
                // Finalize current entity if exists
                IF current_entity is not None:
                    entity = finalize_entity(current_entity)
                    IF entity.confidence >= threshold:
                        entities.push(entity)
                    current_entity = None

    // Finalize last entity
    IF current_entity is not None:
        entity = finalize_entity(current_entity)
        IF entity.confidence >= threshold:
            entities.push(entity)

    RETURN entities

FUNCTION finalize_entity(current):
    avg_confidence = mean(current.confidences)
    RETURN EntityMatch {
        entity_type: current.type,
        value: text[current.start..current.end],
        start: current.start,
        end: current.end,
        confidence: avg_confidence
    }
END FUNCTION
```

---

## SPARC Phase 3: Architecture

### 3.1 Component Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    NerDetector (Public)                  │
│  ┌───────────────────────────────────────────────────┐  │
│  │ detect(text: &str) -> Vec<EntityMatch>           │  │
│  └───────────────────────────────────────────────────┘  │
│           │                    │                         │
│           │                    │                         │
│           ▼                    ▼                         │
│  ┌─────────────────┐  ┌──────────────────────────┐      │
│  │ TokenizerWrapper│  │ InferenceEngine          │      │
│  │  + encode()     │  │  + infer_token_class()   │      │
│  │  + offsets      │  │  + TokenPrediction       │      │
│  └─────────────────┘  └──────────────────────────┘      │
│           │                    │                         │
└───────────┼────────────────────┼─────────────────────────┘
            │                    │
            ▼                    ▼
   ┌─────────────────┐  ┌─────────────────┐
   │ HuggingFace     │  │ ONNX Runtime    │
   │ Tokenizers      │  │ Session         │
   └─────────────────┘  └─────────────────┘
```

### 3.2 Data Flow

```
Input Text
    │
    ▼
┌─────────────────────────────────────────┐
│ 1. Tokenization with Offsets            │
│    text -> (input_ids, mask, offsets)   │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ 2. Token Classification Inference       │
│    input_ids -> Vec<TokenPrediction>    │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ 3. Build Tagged Tokens                  │
│    predictions + offsets -> TaggedToken │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ 4. Decode BIO Tags                      │
│    TaggedToken -> EntityMatch           │
└─────────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────────┐
│ 5. Filter by Confidence                 │
│    EntityMatch -> filtered results      │
└─────────────────────────────────────────┘
    │
    ▼
Output: Vec<EntityMatch>
```

### 3.3 Testing Architecture (London School)

```
┌────────────────────────────────────────────────┐
│           Acceptance Tests (Outside-In)        │
│  Test real user scenarios with real components │
└────────────────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────┐
│         Integration Tests (Real)               │
│  - InferenceEngine + real ONNX session         │
│  - TokenizerWrapper + real tokenizer           │
│  - Full pipeline tests                         │
└────────────────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────┐
│          Unit Tests (Mocked)                   │
│  - Mock ONNX session for InferenceEngine       │
│  - Mock tokenizer for TokenizerWrapper         │
│  - Pure function tests (BIO decoding)          │
└────────────────────────────────────────────────┘
```

---

## Acceptance Criteria Summary

✅ **Must Have (MVP):**
1. TokenPrediction struct with all required fields
2. InferenceEngine::infer_token_classification() implementation
3. Encoding.offsets field with character positions
4. NerDetector.detect() end-to-end implementation
5. BIO tag decoding with confidence averaging
6. 25+ comprehensive tests (unit + integration + acceptance)
7. 0 compiler errors, 0 test failures
8. API documentation for all public types

⚠️ **Should Have (Post-MVP):**
9. Result caching for repeated inputs
10. Batch processing optimization
11. Model quantization (INT8) for performance
12. Hybrid detector (Regex + NER)

🔄 **Could Have (Future):**
13. Custom model training pipeline
14. Online learning / fine-tuning
15. Multi-language support
16. GPU acceleration

---

**Next:** SPARC Phase 4 - Refinement (TDD Implementation)
