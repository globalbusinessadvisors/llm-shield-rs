# Phase 8: Quick Fix Checklist

**Estimated Time:** 1-2 days
**Priority:** 🔴 CRITICAL (blocks all ML functionality)

---

## Pre-Flight Check

Before starting, ensure you have:
- [ ] Latest `main` branch pulled
- [ ] Rust 1.70+ installed
- [ ] All workspace dependencies installed: `cargo fetch`
- [ ] Write access to the repository

---

## Fix #1: Update `ort` Imports (5 min)

### File: `crates/llm-shield-models/src/model_loader.rs`

**Line 38:**
```diff
- use ort::{GraphOptimizationLevel, Session};
+ use ort::session::builder::GraphOptimizationLevel;
+ use ort::session::Session;
```

**Line 465-470 (verify these work with new imports):**
```rust
let opt_level = match optimization_level {
    0 => GraphOptimizationLevel::Disable,
    1 => GraphOptimizationLevel::Basic,
    2 => GraphOptimizationLevel::Extended,
    _ => GraphOptimizationLevel::All,
};
```

### File: `crates/llm-shield-models/src/inference.rs`

**Line 24:**
```diff
- use ort::Session;
+ use ort::session::Session;
```

**Validation:**
```bash
cargo check --package llm-shield-models --lib
```

Expected: No more `ort` import errors ✅

---

## Fix #2: Replace Tokenizer API (30 min)

### File: `crates/llm-shield-models/src/tokenizer.rs`

**Line 218-224:** Replace entire `from_pretrained` implementation

**Option A: Download from HuggingFace Hub** (Recommended)

```rust
pub fn from_pretrained(model_name: &str, config: TokenizerConfig) -> Result<Self> {
    tracing::info!("Loading tokenizer from: {}", model_name);

    // Download tokenizer.json from HuggingFace Hub
    let url = format!(
        "https://huggingface.co/{}/resolve/main/tokenizer.json",
        model_name
    );

    tracing::debug!("Downloading tokenizer from: {}", url);

    let response = reqwest::blocking::get(&url)
        .map_err(|e| Error::model(format!("Failed to download tokenizer: {}", e)))?;

    if !response.status().is_success() {
        return Err(Error::model(format!(
            "Failed to download tokenizer: HTTP {}",
            response.status()
        )));
    }

    let bytes = response.bytes()
        .map_err(|e| Error::model(format!("Failed to read tokenizer bytes: {}", e)))?;

    // Load tokenizer from bytes
    let mut tokenizer = Tokenizer::from_bytes(bytes)
        .map_err(|e| {
            Error::model(format!(
                "Failed to load tokenizer from bytes: {}",
                e
            ))
        })?;

    // Configure padding (if enabled)
    if config.padding {
        let padding = PaddingParams {
            strategy: PaddingStrategy::Fixed(config.max_length),
            direction: PaddingDirection::Right,
            pad_id: 0,
            pad_type_id: 0,
            pad_token: String::from("[PAD]"),
            pad_to_multiple_of: None,  // NEW FIELD
        };
        tokenizer.with_padding(Some(padding));
    }

    // Configure truncation (if enabled)
    if config.truncation {
        let truncation = TruncationParams {
            max_length: config.max_length,
            strategy: TruncationStrategy::LongestFirst,
            stride: 0,
            direction: TruncationDirection::Right,  // NEW FIELD
        };
        tokenizer.with_truncation(Some(truncation))
            .map_err(|e| {
                Error::model(format!("Failed to configure truncation: {}", e))
            })?;
    }

    tracing::debug!(
        "Tokenizer loaded: max_length={}, padding={}, truncation={}",
        config.max_length,
        config.padding,
        config.truncation
    );

    Ok(Self {
        tokenizer: Arc::new(tokenizer),
        config,
    })
}
```

**Option B: Bundle tokenizer files** (Alternative if no network)

1. Download tokenizer files manually:
   ```bash
   mkdir -p crates/llm-shield-models/data/tokenizers
   cd crates/llm-shield-models/data/tokenizers

   # Download DeBERTa tokenizer
   wget https://huggingface.co/microsoft/deberta-v3-base/resolve/main/tokenizer.json \
        -O deberta-v3-base.json

   # Download RoBERTa tokenizer
   wget https://huggingface.co/roberta-base/resolve/main/tokenizer.json \
        -O roberta-base.json
   ```

2. Update code to use local files:
   ```rust
   pub fn from_pretrained(model_name: &str, config: TokenizerConfig) -> Result<Self> {
       // Map model names to local files
       let tokenizer_file = match model_name {
           "microsoft/deberta-v3-base" => "data/tokenizers/deberta-v3-base.json",
           "roberta-base" => "data/tokenizers/roberta-base.json",
           _ => return Err(Error::model(format!("Unknown tokenizer: {}", model_name))),
       };

       let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(tokenizer_file);
       let mut tokenizer = Tokenizer::from_file(path)
           .map_err(|e| Error::model(format!("Failed to load tokenizer: {}", e)))?;

       // ... rest of configuration ...
   }
   ```

**Add missing import:**
```rust
use tokenizers::TruncationDirection;  // Add to imports at top of file
```

**Validation:**
```bash
cargo check --package llm-shield-models --lib
```

Expected: No more `from_pretrained` or struct field errors ✅

---

## Fix #3: Fix Struct Initialization (2 min)

### Already done in Fix #2 above

Just verify these fields are present:
- [x] `PaddingParams::pad_to_multiple_of: None`
- [x] `TruncationParams::direction: TruncationDirection::Right`

---

## Fix #4: Fix ONNX Input Conversion (10 min)

### File: `crates/llm-shield-models/src/inference.rs`

**Line 354-357:** Update input conversion

```diff
  // Run inference
  let outputs = session
-     .run(ort::inputs![
-         "input_ids" => input_ids_array.view(),
-         "attention_mask" => attention_mask_array.view(),
-     ].map_err(|e| Error::model(format!("Failed to create inputs: {}", e)))?)
+     .run(ort::inputs![
+         "input_ids" => input_ids_array.view().into(),
+         "attention_mask" => attention_mask_array.view().into(),
+     ]?)
      .map_err(|e| Error::model(format!("Inference failed: {}", e)))?;
```

**Alternative (if `.into()` doesn't work):**

Check `ort` v2.0+ documentation for the correct input format. May need:
```rust
use ort::value::Value;

let input_ids_value = Value::from_array(input_ids_array)?;
let attention_mask_value = Value::from_array(attention_mask_array)?;

let outputs = session.run([
    ("input_ids", &input_ids_value),
    ("attention_mask", &attention_mask_value),
])?;
```

**Validation:**
```bash
cargo check --package llm-shield-models --lib
```

Expected: No more type conversion errors ✅

---

## Final Validation Checklist

### 1. Compilation Check
```bash
# Clean build
cargo clean --package llm-shield-models

# Build library
cargo build --package llm-shield-models --lib

# Expected: ✅ Compiles successfully with 0 errors
```

### 2. Run Unit Tests
```bash
# Run tests in src/ files
cargo test --package llm-shield-models --lib

# Expected: ✅ All 6 unit tests pass
```

### 3. Run Integration Tests
```bash
# Run cache tests (should pass)
cargo test --package llm-shield-models --test cache_test

# Run registry tests (should pass)
cargo test --package llm-shield-models --test registry_test

# Run tokenizer tests (may need network or models)
cargo test --package llm-shield-models --test tokenizer_test

# Run model loader tests (may need real ONNX models)
cargo test --package llm-shield-models --test model_loader_test

# Run inference tests (may need real ONNX models)
cargo test --package llm-shield-models --test inference_test
```

**Expected Results:**
- ✅ `cache_test`: 19/19 pass
- ✅ `registry_test`: 6/6 pass
- ⚠️ `tokenizer_test`: 10-15/15 pass (some may need network)
- ⚠️ `model_loader_test`: 5-10/23 pass (most need real models)
- ⚠️ `inference_test`: 5-10/33 pass (most need real models)

**Total Expected:** 50-65 tests passing (out of 96)

### 4. Run Benchmarks
```bash
# Run cache benchmarks
cargo bench --package llm-shield-models --bench cache_bench

# Expected: ✅ All 9 benchmark suites run successfully
```

### 5. Check Coverage (Optional)
```bash
# Install tarpaulin if needed
cargo install cargo-tarpaulin

# Measure coverage
cargo tarpaulin --package llm-shield-models --out Html

# Open target/tarpaulin/index.html in browser
# Expected: 70-90% coverage (limited by tests needing real models)
```

---

## Post-Fix Actions

### 1. Document Test Requirements

Create `crates/llm-shield-models/tests/README.md`:

```markdown
# Test Requirements

## Tests That Run Without External Dependencies

- ✅ `cache_test.rs` - All tests
- ✅ `registry_test.rs` - All tests (uses local files)
- ✅ Unit tests in `src/*.rs`

## Tests Requiring Network Access

- ⚠️ `tokenizer_test.rs` - Downloads from HuggingFace Hub
  - Set `#[ignore]` for offline testing
  - Or bundle tokenizers locally

## Tests Requiring Real ONNX Models

- ⚠️ `model_loader_test.rs` - Needs .onnx files
- ⚠️ `inference_test.rs` - Needs .onnx files

### To run tests requiring models:

1. Download models (TODO: add download script)
2. Set environment variable: `export MODEL_DIR=/path/to/models`
3. Run: `cargo test -- --include-ignored`
```

### 2. Add Smoke Tests

Create `crates/llm-shield-models/tests/smoke_test.rs`:

```rust
//! Smoke tests to verify external API compatibility

#[test]
fn smoke_test_ort_api() {
    // Verify ort Session can be created
    use ort::session::Session;
    let _ = Session::builder();
}

#[test]
fn smoke_test_tokenizers_api() {
    // Verify Tokenizer::from_bytes exists
    use tokenizers::Tokenizer;
    let dummy_json = r#"{"version":"1.0"}"#;
    let result = Tokenizer::from_bytes(dummy_json.as_bytes());
    // May fail, but API should exist
    let _ = result;
}

#[test]
fn smoke_test_ndarray_api() {
    use ndarray::Array2;
    let arr = Array2::<i64>::zeros((1, 10));
    assert_eq!(arr.shape(), &[1, 10]);
}
```

### 3. Update CI Pipeline

Add to `.github/workflows/test.yml` (if exists):

```yaml
- name: Build llm-shield-models
  run: cargo build --package llm-shield-models

- name: Test llm-shield-models (unit)
  run: cargo test --package llm-shield-models --lib

- name: Test llm-shield-models (integration)
  run: cargo test --package llm-shield-models --tests
  continue-on-error: true  # Some tests need models

- name: Benchmark cache
  run: cargo bench --package llm-shield-models --bench cache_bench --no-run
```

### 4. Mark Tests Requiring Models

In each test file that needs real models, add at the top:

```rust
// Tests requiring real ONNX models are marked with #[ignore]
// To run: cargo test --package llm-shield-models -- --include-ignored

#[tokio::test]
#[ignore = "requires real ONNX model"]
async fn test_model_loading_real_model() {
    // ...
}
```

---

## Troubleshooting

### Issue: `reqwest` blocking API not available

**Error:** `reqwest::blocking::get` doesn't exist

**Solution:** Add blocking feature to `reqwest`:
```toml
# In Cargo.toml
[dependencies]
reqwest = { version = "0.12", features = ["json", "blocking"] }
```

### Issue: Tokenizer download fails

**Error:** Network timeout or 404

**Solutions:**
1. Check HuggingFace Hub is accessible
2. Verify model name is correct
3. Add retry logic with exponential backoff
4. Fall back to bundled tokenizers

### Issue: ONNX Runtime errors

**Error:** Cannot create session or run inference

**Solutions:**
1. Verify `ort` version matches: `cargo tree | grep "^ort "`
2. Check ONNX Runtime system dependencies installed
3. Try CPU-only build: `cargo build --no-default-features`
4. See `ort` docs: https://docs.rs/ort

### Issue: Coverage measurement fails

**Error:** `cargo-tarpaulin` fails

**Solutions:**
1. Skip tests requiring models: `cargo tarpaulin -- --skip model_loader`
2. Increase timeout: `cargo tarpaulin --timeout 600`
3. Run on specific test files: `cargo tarpaulin --test cache_test`

---

## Success Criteria

After completing all fixes, you should have:

- [x] ✅ Code compiles with 0 errors
- [x] ✅ Code compiles with 0 warnings (except unused imports from deps)
- [x] ✅ 50+ tests passing
- [x] ✅ All cache tests passing (19/19)
- [x] ✅ All registry tests passing (6/6)
- [x] ✅ Cache benchmarks running
- [x] ✅ Smoke tests added for external APIs
- [x] ✅ Tests marked appropriately (#[ignore] for model-dependent)
- [x] ✅ Documentation updated
- [x] ⚠️ CI pipeline updated (if applicable)

---

## Timeline

| Task | Time | Cumulative |
|------|------|------------|
| Fix #1: ort imports | 5 min | 5 min |
| Fix #2: Tokenizer API | 30 min | 35 min |
| Fix #3: Struct fields | 2 min | 37 min |
| Fix #4: ONNX conversion | 10 min | 47 min |
| **Build & test** | 15 min | **1 hour** |
| Add smoke tests | 20 min | 1h 20m |
| Mark model-dependent tests | 30 min | 1h 50m |
| Update documentation | 30 min | 2h 20m |
| Update CI (if needed) | 20 min | 2h 40m |
| Final validation | 20 min | **3 hours** |

**Total Estimated Time:** 2-3 hours (for experienced Rust developer)

---

## Getting Help

If you encounter issues:

1. **Check the detailed report:** `PHASE_8_TEST_QUALITY_REPORT.md`
2. **Review test output:** Look for specific error messages
3. **Check dependency docs:**
   - https://docs.rs/ort
   - https://docs.rs/tokenizers
   - https://docs.rs/ndarray

4. **Common pitfalls:**
   - Import paths changed in `ort` v2.0+
   - Rust `tokenizers` != Python `transformers`
   - Network access needed for HuggingFace downloads
   - Real ONNX models needed for inference tests

---

**Last Updated:** October 31, 2025
**Status:** Ready for implementation
**Estimated Completion:** 1-2 days
