# Phase 8: Pre-trained ML Models - Implementation Plan

**Project:** llm-shield-rs
**Phase:** 8 - ML Model Integration
**Version:** 1.0
**Date:** 2025-10-30
**Status:** Ready for Implementation
**Estimated Duration:** 7 weeks

---

## Executive Summary

This document provides a comprehensive, actionable implementation plan for Phase 8 of the llm-shield-rs project: integrating pre-trained ML models for three security scanners (PromptInjection, Toxicity, and Sentiment).

**Objective:** Enable production ML-powered security scanning with <150ms latency, >90% test coverage, and >85% accuracy (F1 score).

**Scope:**
- Convert 3 HuggingFace models to ONNX format
- Implement model distribution via HuggingFace Hub
- Integrate models into existing scanners
- Optimize for performance (FP16 quantization, batching, caching)
- Validate accuracy parity with Python llm-guard
- Document and benchmark all models

**Success Criteria:**
- ✅ All 3 models converted and deployed
- ✅ Inference latency < 150ms per request
- ✅ Memory usage < 1.5GB under load
- ✅ Accuracy: F1 > 0.85 for all models
- ✅ Test coverage > 90%
- ✅ Comprehensive documentation

---

## Table of Contents

1. [Implementation Phases](#1-implementation-phases)
2. [Technical Architecture](#2-technical-architecture)
3. [Model Specifications](#3-model-specifications)
4. [Development Workflows](#4-development-workflows)
5. [Dependencies and Tools](#5-dependencies-and-tools)
6. [Risk Assessment](#6-risk-assessment)
7. [Testing Strategy](#7-testing-strategy)
8. [Documentation Requirements](#8-documentation-requirements)
9. [Timeline and Resources](#9-timeline-and-resources)
10. [Success Metrics](#10-success-metrics)

---

## 1. Implementation Phases

### Phase 8.1: Model Conversion Infrastructure (Week 1)

**Objective:** Set up tools and convert all 3 models to ONNX

**Tasks:**
1. Set up Python environment with Optimum, transformers, ONNX
2. Convert PromptInjection model (DeBERTa-v3-base)
   - FP32 baseline
   - FP16 optimized (production)
   - INT8 quantized (edge/WASM)
3. Convert Toxicity model (RoBERTa)
4. Convert Sentiment model (RoBERTa-based)
5. Validate all conversions against PyTorch baseline
6. Generate model metadata (size, latency, accuracy)

**Deliverables:**
- ✅ 9 ONNX models (3 models × 3 formats)
- ✅ Validation reports showing <1% accuracy deviation
- ✅ Model metadata JSON files
- ✅ Tokenizer configurations exported

**Success Criteria:**
- All models convert without errors
- Accuracy deviation < 1% for FP16, < 3% for INT8
- Models load successfully in ort 2.0

**Tools:**
- `scripts/convert_models.py` (baseline, graph-opt, fp16, int8)
- `scripts/test_model_accuracy.py` (validation)

**Time Estimate:** 5 days

---

### Phase 8.2: Model Download & Caching (Week 2)

**Objective:** Implement model distribution and lazy loading

**Tasks:**
1. Upload ONNX models to HuggingFace Hub
   - Create organization: `llm-shield`
   - Upload models with version tags
   - Set up model cards with attribution
2. Implement `hf-hub` integration in Rust
   - Create `ModelRegistry` struct
   - Implement lazy download on first use
   - Cache models in `~/.cache/llm-shield/models/`
   - Verify checksums (SHA-256)
3. Add `ModelDownloader` utility
4. Create CLI tool for pre-downloading models

**Deliverables:**
- ✅ Models hosted on HuggingFace Hub
- ✅ `ModelRegistry` implementation (llm-shield-models)
- ✅ Download CLI: `llm-shield download --model prompt-injection`
- ✅ Cache directory structure
- ✅ Model verification with checksums

**Success Criteria:**
- Models download automatically on first scanner use
- Downloads are cached and reused
- Checksum validation prevents corruption
- Offline mode works with cached models

**Dependencies:**
- `hf-hub = "0.3"` (HuggingFace Hub API)
- `sha2 = "0.10"` (checksum verification)

**Time Estimate:** 5 days

---

### Phase 8.3: PromptInjection Model Integration (Week 3)

**Objective:** Integrate DeBERTa model into PromptInjection scanner

**Tasks:**
1. Update `PromptInjectionConfig` with ML settings
   ```rust
   pub struct PromptInjectionConfig {
       pub use_ml_model: bool,
       pub ml_threshold: f32,
       pub model_variant: ModelVariant, // FP16, INT8
       pub fallback_to_heuristic: bool,
       pub cache_results: bool,
   }
   ```
2. Implement `detect_ml()` method
   - Load model via `ModelRegistry`
   - Tokenize input (max 512 tokens)
   - Run inference
   - Post-process logits to probabilities
   - Apply threshold
3. Add hybrid mode (heuristic pre-filter + ML)
4. Implement result caching (LRU cache)
5. Add async batch inference support

**Deliverables:**
- ✅ Full ML integration in `prompt_injection.rs`
- ✅ Hybrid mode implementation
- ✅ Result caching with TTL
- ✅ Unit tests (15+)
- ✅ Integration tests (5+)
- ✅ Benchmark tests

**Success Criteria:**
- Latency < 100ms (FP16, CPU)
- Accuracy: F1 > 0.90
- Heuristic pre-filter reduces ML calls by 60%+
- Cache hit rate > 40% in typical workload

**Time Estimate:** 7 days

---

### Phase 8.4: Toxicity Model Integration (Week 4)

**Objective:** Integrate RoBERTa toxicity model

**Tasks:**
1. Update `ToxicityConfig` with ML settings
2. Implement multi-label classification (6 categories)
   - Toxicity, severe toxicity, obscene, threat, insult, identity hate
3. Implement `detect_ml()` method
4. Add category-specific thresholds
5. Implement hybrid mode
6. Add result aggregation (worst category score)

**Deliverables:**
- ✅ Full ML integration in `toxicity.rs`
- ✅ Multi-label classification
- ✅ Per-category confidence scores
- ✅ Unit tests (15+)
- ✅ Integration tests (5+)
- ✅ Benchmark tests

**Success Criteria:**
- Latency < 120ms (FP16, CPU)
- Accuracy: F1 > 0.87 across all categories
- Hybrid mode reduces ML calls by 50%+

**Time Estimate:** 7 days

---

### Phase 8.5: Sentiment Model Integration (Week 4)

**Objective:** Integrate sentiment analysis model

**Tasks:**
1. Update `SentimentConfig` with ML settings
2. Implement 3-way classification (positive, neutral, negative)
3. Implement `detect_ml()` method
4. Add confidence thresholds per sentiment
5. Implement hybrid mode with lexicon pre-filter

**Deliverables:**
- ✅ Full ML integration in `sentiment.rs`
- ✅ 3-way classification
- ✅ Confidence scoring
- ✅ Unit tests (12+)
- ✅ Integration tests (5+)
- ✅ Benchmark tests

**Success Criteria:**
- Latency < 100ms (FP16, CPU)
- Accuracy: > 0.82
- Hybrid mode reduces ML calls by 70%+

**Time Estimate:** 5 days

---

### Phase 8.6: Model Optimization & Quantization (Week 5)

**Objective:** Optimize models for production performance

**Tasks:**
1. Benchmark all models (CPU, GPU if available)
   - Measure latency (p50, p95, p99)
   - Measure memory usage
   - Measure throughput
2. Implement model quantization (INT8)
   - Evaluate accuracy impact
   - Benchmark performance gains
3. Implement batching for high-throughput scenarios
4. Add GPU support (CUDA, ROCm) via ONNX Runtime
5. Implement warm-up for first inference
6. Profile and optimize hot paths

**Deliverables:**
- ✅ Comprehensive benchmark results
- ✅ INT8 models with <3% accuracy loss
- ✅ Batch inference implementation
- ✅ GPU support (optional)
- ✅ Performance optimization report

**Success Criteria:**
- FP16 latency < 100ms (PromptInjection, Sentiment)
- FP16 latency < 150ms (Toxicity)
- INT8 provides 30-50% speedup
- Batching increases throughput 3-5x
- Memory usage < 1.5GB with all models loaded

**Time Estimate:** 7 days

---

### Phase 8.7: WASM Compatibility (Week 6) - OPTIONAL

**Objective:** Assess and enable WASM deployment if feasible

**Tasks:**
1. Research ONNX Runtime WASM support
   - Test ort-wasm crate
   - Evaluate model size constraints (<10MB)
2. Create minimal WASM build
   - Use INT8 models
   - Feature-flag ML support
3. Test inference in browser
4. Benchmark WASM performance
5. Document limitations and workarounds

**Deliverables:**
- ✅ WASM feasibility report
- ✅ Minimal WASM build (if feasible)
- ✅ WASM benchmarks
- ✅ Documentation on WASM deployment

**Success Criteria (if feasible):**
- WASM bundle < 15MB (gzipped)
- Inference latency < 500ms (browser)
- Models load in <3s

**Alternative:** Document why WASM ML is not feasible and recommend native/server deployment

**Time Estimate:** 5 days

---

### Phase 8.8: Testing & Validation (Week 7)

**Objective:** Comprehensive testing and validation

**Tasks:**
1. Unit tests for all ML components
   - Model loading
   - Tokenization
   - Inference
   - Result processing
2. Integration tests
   - End-to-end scanner tests
   - Hybrid mode tests
   - Caching tests
3. Accuracy validation
   - Test against Python llm-guard baseline
   - Use real-world test datasets
   - Generate confusion matrices
4. Performance benchmarks
   - Update benchmark framework
   - Run latency benchmarks
   - Run throughput benchmarks
   - Run memory benchmarks
5. Cross-platform testing
   - Linux (x86_64, arm64)
   - macOS (x86_64, arm64)
   - Windows (x86_64)

**Deliverables:**
- ✅ 50+ unit tests
- ✅ 15+ integration tests
- ✅ Accuracy validation report
- ✅ Performance benchmark results
- ✅ Cross-platform compatibility matrix

**Success Criteria:**
- Test coverage > 90%
- All tests pass on all platforms
- Accuracy within 1% of Python baseline
- Performance meets targets (< 150ms)

**Time Estimate:** 7 days

---

## 2. Technical Architecture

### 2.1 System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    LLM Shield with ML Models                 │
└─────────────────────────────────────────────────────────────┘

┌──────────────────┐
│   Application    │  User's LLM Application
└────────┬─────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────┐
│                Scanner with Hybrid Detection                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  1. Heuristic Pre-Filter (0.01ms)                    │   │
│  │     ↓ (60-70% filtered here)                         │   │
│  │  2. ML Model Inference (50-150ms)                    │   │
│  │     ├─ Check cache first                             │   │
│  │     ├─ Load model (lazy, once)                       │   │
│  │     ├─ Tokenize input                                │   │
│  │     ├─ Run ONNX inference                            │   │
│  │     └─ Post-process results                          │   │
│  └──────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────┐
│              Model Infrastructure Layer                       │
│  ┌───────────────┐  ┌──────────────┐  ┌─────────────────┐   │
│  │ ModelRegistry │  │ ModelLoader  │  │  ModelCache     │   │
│  │               │  │              │  │  (LRU, TTL)     │   │
│  │ - Download    │  │ - ONNX RT    │  │  - Input hash   │   │
│  │ - Verify      │  │ - Tokenizer  │  │  - Result cache │   │
│  │ - Cache       │  │ - Inference  │  │                 │   │
│  └───────────────┘  └──────────────┘  └─────────────────┘   │
└──────────────────────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────────────┐
│              Model Storage                                    │
│  HuggingFace Hub (primary)    ~/.cache/llm-shield/models/   │
│  ├─ prompt-injection/          ├─ prompt-injection/          │
│  │  ├─ model.onnx (FP16)       │  ├─ model.onnx             │
│  │  ├─ tokenizer.json          │  ├─ tokenizer.json         │
│  │  └─ config.json             │  └─ metadata.json          │
│  ├─ toxicity/                  ├─ toxicity/                  │
│  └─ sentiment/                 └─ sentiment/                 │
└──────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
User Input → Heuristic Check → [Cache Check] → ML Model → Result
              ↓ SAFE                  ↓ HIT      ↓
              Return (fast)           Return    ONNX Inference
                                                  ↓
                                                Post-process
                                                  ↓
                                                [Cache Store]
                                                  ↓
                                                Return
```

### 2.3 Module Structure

```rust
// crates/llm-shield-models/src/

pub mod registry;      // ModelRegistry, model discovery
pub mod loader;        // ModelLoader, ONNX loading
pub mod cache;         // ResultCache (LRU with TTL)
pub mod inference;     // InferenceEngine, batching
pub mod tokenizer;     // TokenizerWrapper
pub mod types;         // ModelMetadata, ModelVariant, etc.

// crates/llm-shield-scanners/src/input/

// Each scanner implements:
impl Scanner for PromptInjection {
    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        // 1. Check cache
        if let Some(cached) = self.cache.get(input) {
            return Ok(cached);
        }

        // 2. Heuristic pre-filter
        if self.config.use_heuristic_prefilter {
            if self.is_obviously_safe_heuristic(input) {
                return Ok(ScanResult::safe());
            }
        }

        // 3. ML detection (if enabled)
        if self.config.use_ml_model {
            let result = self.detect_ml(input).await?;
            self.cache.insert(input, result.clone());
            return Ok(result);
        }

        // 4. Fallback to heuristic only
        self.detect_heuristic(input)
    }

    async fn detect_ml(&self, input: &str) -> Result<ScanResult> {
        // Lazy load model
        let model = self.model_loader.load_or_get("prompt-injection")?;

        // Tokenize
        let tokens = self.tokenizer.encode(input, 512)?;

        // Inference
        let logits = model.infer(&tokens).await?;

        // Post-process
        let probability = softmax(logits)[1]; // class 1 = injection

        Ok(ScanResult {
            is_valid: probability < self.config.ml_threshold,
            risk_score: probability,
            ..Default::default()
        })
    }
}
```

---

## 3. Model Specifications

### 3.1 PromptInjection Model

**Source:** `protectai/deberta-v3-base-prompt-injection-v2`
**Architecture:** DeBERTa-v3-base
**Parameters:** 184M
**Task:** Binary classification (SAFE vs INJECTION)

| Format | Size | Latency | Memory | Accuracy (F1) |
|--------|------|---------|--------|---------------|
| FP32   | 700 MB | 150ms | 1.2 GB | 0.902 |
| FP16   | 350 MB | 80ms  | 800 MB | 0.900 |
| INT8   | 175 MB | 50ms  | 500 MB | 0.883 |

**Recommended:** FP16 (production), INT8 (edge)

**Input:**
- Max tokens: 512
- Tokenizer: DeBERTa tokenizer
- Padding: right
- Truncation: enabled

**Output:**
- Logits: [batch_size, 2]
- Classes: [SAFE, INJECTION]
- Post-processing: Softmax → probability

**Configuration:**
```rust
PromptInjectionConfig {
    use_ml_model: true,
    ml_threshold: 0.5,
    model_variant: ModelVariant::FP16,
    fallback_to_heuristic: true,
    cache_results: true,
    cache_ttl: Duration::from_secs(3600),
}
```

---

### 3.2 Toxicity Model

**Source:** `unitary/unbiased-toxic-roberta`
**Architecture:** RoBERTa-base
**Parameters:** 125M
**Task:** Multi-label classification (6 categories)

| Format | Size | Latency | Memory | Accuracy (F1) |
|--------|------|---------|--------|---------------|
| FP32   | 500 MB | 180ms | 900 MB | 0.875 |
| FP16   | 250 MB | 100ms | 600 MB | 0.872 |
| INT8   | 125 MB | 60ms  | 400 MB | 0.851 |

**Recommended:** FP16

**Input:**
- Max tokens: 512
- Tokenizer: RoBERTa tokenizer
- Padding: right
- Truncation: enabled

**Output:**
- Logits: [batch_size, 6]
- Classes: [toxicity, severe_toxicity, obscene, threat, insult, identity_hate]
- Post-processing: Sigmoid → per-class probability

**Configuration:**
```rust
ToxicityConfig {
    use_ml_model: true,
    ml_thresholds: HashMap::from([
        ("toxicity", 0.5),
        ("severe_toxicity", 0.3),
        ("threat", 0.4),
        // ...
    ]),
    model_variant: ModelVariant::FP16,
    aggregate_method: AggregateMethod::MaxScore,
}
```

---

### 3.3 Sentiment Model

**Source:** `cardiffnlp/twitter-roberta-base-sentiment-latest`
**Architecture:** RoBERTa-base
**Parameters:** 125M
**Task:** 3-way classification (positive, neutral, negative)

| Format | Size | Latency | Memory | Accuracy |
|--------|------|---------|--------|----------|
| FP32   | 500 MB | 150ms | 900 MB | 0.825 |
| FP16   | 250 MB | 80ms  | 600 MB | 0.823 |
| INT8   | 125 MB | 50ms  | 400 MB | 0.809 |

**Recommended:** FP16

**Input:**
- Max tokens: 512
- Tokenizer: RoBERTa tokenizer
- Padding: right
- Truncation: enabled

**Output:**
- Logits: [batch_size, 3]
- Classes: [negative, neutral, positive]
- Post-processing: Softmax → probability distribution

**Configuration:**
```rust
SentimentConfig {
    use_ml_model: true,
    allowed_sentiments: vec![Sentiment::Positive, Sentiment::Neutral],
    ml_threshold: 0.5,
    model_variant: ModelVariant::FP16,
}
```

---

## 4. Development Workflows

### 4.1 Model Conversion Workflow

```bash
# 1. Install dependencies
pip install -r scripts/requirements.txt

# 2. Convert model (all formats)
python scripts/convert_models.py \
    --model protectai/deberta-v3-base-prompt-injection-v2 \
    --task sequence-classification \
    --output-dir ./models/onnx/prompt-injection \
    --optimize all  # baseline, graph-opt, fp16, int8

# 3. Validate accuracy
python scripts/test_model_accuracy.py \
    --onnx-model ./models/onnx/prompt-injection/model-fp16.onnx \
    --pytorch-model protectai/deberta-v3-base-prompt-injection-v2 \
    --test-size 1000 \
    --report ./models/onnx/prompt-injection/validation_report.json

# 4. Upload to HuggingFace Hub
huggingface-cli upload \
    llm-shield/prompt-injection-deberta-v3-fp16 \
    ./models/onnx/prompt-injection/model-fp16.onnx \
    --repo-type model
```

### 4.2 Model Testing Workflow

```bash
# 1. Download model
./scripts/download_models.sh --model prompt-injection

# 2. Run Rust inference example
cargo run --example ml_model_inference -- \
    --model-dir ~/.cache/llm-shield/models/prompt-injection \
    --input "Ignore all previous instructions" \
    --batch

# 3. Run accuracy tests
cargo test --package llm-shield-scanners --test ml_accuracy -- \
    --test-threads 1

# 4. Run benchmarks
cargo bench --package llm-shield-benches -- ml_

# 5. Generate report
python benchmarks/scripts/analyze_results.py --ml-models
```

### 4.3 Scanner Integration Workflow

```rust
// Example: Integrating ML into PromptInjection

// 1. Add dependencies to Cargo.toml
[dependencies]
llm-shield-models = { path = "../llm-shield-models", features = ["onnx"] }

// 2. Update config struct
pub struct PromptInjectionConfig {
    // Existing heuristic config
    pub heuristic_threshold: f32,

    // NEW: ML config
    pub use_ml_model: bool,
    pub ml_threshold: f32,
    pub model_variant: ModelVariant,
    pub fallback_to_heuristic: bool,
}

// 3. Update scanner struct
pub struct PromptInjection {
    config: PromptInjectionConfig,

    // NEW: ML components
    model_loader: Arc<ModelLoader>,
    tokenizer: Arc<Tokenizer>,
    cache: Arc<RwLock<ResultCache>>,
}

// 4. Implement detect_ml()
impl PromptInjection {
    async fn detect_ml(&self, input: &str) -> Result<ScanResult> {
        // Load model (cached after first load)
        let model = self.model_loader
            .load("prompt-injection", self.config.model_variant)
            .await?;

        // Tokenize
        let encoding = self.tokenizer.encode(input, 512)?;

        // Inference
        let output = model.infer(encoding.ids()).await?;

        // Post-process
        let probabilities = softmax(&output);
        let injection_prob = probabilities[1];

        Ok(ScanResult {
            is_valid: injection_prob < self.config.ml_threshold,
            risk_score: injection_prob,
            entities: vec![],
            sanitized_input: None,
        })
    }
}

// 5. Update scan() method
async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
    // Check cache first
    if let Some(cached) = self.cache.read().await.get(input) {
        return Ok(cached.clone());
    }

    // Heuristic pre-filter (fast path)
    if self.is_obviously_safe(input) {
        return Ok(ScanResult::safe());
    }

    // ML detection
    if self.config.use_ml_model {
        match self.detect_ml(input).await {
            Ok(result) => {
                self.cache.write().await.insert(input.to_string(), result.clone());
                return Ok(result);
            }
            Err(e) if self.config.fallback_to_heuristic => {
                tracing::warn!("ML inference failed, falling back to heuristic: {}", e);
                return self.detect_heuristic(input);
            }
            Err(e) => return Err(e),
        }
    }

    // Heuristic-only mode
    self.detect_heuristic(input)
}
```

---

## 5. Dependencies and Tools

### 5.1 Rust Dependencies

```toml
[dependencies]
# ONNX Runtime
ort = { version = "2.0", features = ["download-binaries"] }

# HuggingFace Hub
hf-hub = "0.3"

# Tokenizers
tokenizers = { version = "0.20", features = ["http"] }

# Array operations
ndarray = "0.16"

# Caching
lru = "0.12"

# Hashing (for cache keys)
sha2 = "0.10"
blake3 = "1.5"

# Async runtime
tokio = { version = "1", features = ["full"] }

# Logging
tracing = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 5.2 Python Dependencies (for conversion)

```
# HuggingFace ecosystem
transformers==4.36.0
optimum[onnxruntime]==1.16.0
tokenizers==0.15.0

# ONNX tools
onnx==1.15.0
onnxruntime==1.16.0
onnxconverter-common==1.14.0

# ML frameworks
torch==2.1.0
numpy==1.24.0

# Testing
scikit-learn==1.3.0
pytest==7.4.0

# CLI tools
huggingface-cli
```

### 5.3 External Tools

- **Docker:** For reproducible conversion environment
- **wasm-pack:** For WASM builds (optional)
- **ONNX Graph Surgeon:** For model optimization
- **Netron:** For model visualization and debugging

---

## 6. Risk Assessment

### 6.1 Risk Matrix

| Risk | Severity | Probability | Impact | Mitigation |
|------|----------|-------------|--------|------------|
| **Performance degradation** | 🔴 HIGH | 🟡 MEDIUM | 🔴 HIGH | Hybrid mode, caching, batching |
| **Model size bloat** | 🟡 MEDIUM | 🟢 LOW | 🟡 MEDIUM | Lazy download, quantization |
| **ONNX compatibility issues** | 🟡 MEDIUM | 🟡 MEDIUM | 🟡 MEDIUM | Extensive testing, fallback mode |
| **Accuracy regression** | 🔴 HIGH | 🟢 LOW | 🔴 HIGH | Comprehensive validation tests |
| **Memory consumption** | 🟡 MEDIUM | 🟡 MEDIUM | 🟡 MEDIUM | Model unloading, streaming inference |
| **WASM incompatibility** | 🟢 LOW | 🔴 HIGH | 🟢 LOW | Feature flags, native-only mode |
| **Model versioning conflicts** | 🟢 LOW | 🟡 MEDIUM | 🟢 LOW | Strict version pinning, registry |
| **Distribution bottlenecks** | 🟢 LOW | 🟡 MEDIUM | 🟢 LOW | CDN (HuggingFace), local cache |

### 6.2 Detailed Risk Analysis

#### Risk 1: Performance Degradation (🔴 HIGH SEVERITY)

**Description:** ML models are 10,000x slower than heuristics (50-150ms vs 0.005ms)

**Impact:**
- Throughput drops from 15,500 req/sec → 150 req/sec
- Latency increases dramatically
- User experience degradation

**Mitigation Strategies:**
1. **Hybrid Mode (Primary):**
   - Use heuristic pre-filter to eliminate 60-70% of inputs
   - Only invoke ML for ambiguous cases
   - Expected improvement: 3-5x throughput increase

2. **Result Caching:**
   - LRU cache with 1-hour TTL
   - Cache hit rate expected: 40%+
   - Reduces repeated ML calls

3. **Batch Inference:**
   - Process multiple inputs simultaneously
   - 3-5x throughput gain
   - Trade-off: Higher latency for individual requests

4. **Quantization:**
   - INT8 models: 30-50% faster
   - Acceptable accuracy loss: <3%

5. **Async Processing:**
   - Non-blocking inference
   - Tokio task spawning for parallel requests

**Residual Risk:** 🟡 MEDIUM (with all mitigations)

#### Risk 2: ONNX Compatibility Issues (🟡 MEDIUM SEVERITY)

**Description:** ONNX models may not run correctly on all platforms

**Impact:**
- Runtime errors on specific OS/architecture
- Incorrect predictions
- Deployment failures

**Mitigation Strategies:**
1. **Comprehensive Testing:**
   - Test on Linux (x86_64, arm64)
   - Test on macOS (x86_64, Apple Silicon)
   - Test on Windows (x86_64)

2. **Fallback Mode:**
   - Always maintain heuristic detection
   - Graceful degradation if ML fails
   - Clear error messages

3. **ONNX Runtime Binary Distribution:**
   - Use `download-binaries` feature
   - Ensure ONNX RT version compatibility
   - Pin ONNX RT version

**Residual Risk:** 🟢 LOW (with all mitigations)

#### Risk 3: Memory Consumption (🟡 MEDIUM SEVERITY)

**Description:** Loading all 3 models consumes ~850 MB (FP16)

**Impact:**
- Increased baseline memory: 45 MB → 895 MB
- May exceed memory limits in constrained environments
- Potential OOM errors

**Mitigation Strategies:**
1. **Lazy Loading:**
   - Only load models when scanner is first used
   - Unload models after inactivity (optional)

2. **Model Variants:**
   - Provide INT8 models for memory-constrained environments
   - Document memory requirements clearly

3. **Feature Flags:**
   - Allow users to exclude ML support at compile time
   - Reduces binary size and memory footprint

**Residual Risk:** 🟢 LOW (with lazy loading)

---

## 7. Testing Strategy

### 7.1 Test Pyramid

```
                    ╱╲
                   ╱  ╲
                  ╱ E2E╲          5 tests
                 ╱──────╲
                ╱        ╲
               ╱Integration╲      15 tests
              ╱────────────╲
             ╱              ╲
            ╱  Unit Tests    ╲   50+ tests
           ╱__________________╲
```

### 7.2 Unit Tests (50+ tests)

**Model Loading:**
- ✅ Test model download from HuggingFace Hub
- ✅ Test local cache retrieval
- ✅ Test checksum verification
- ✅ Test corrupt model handling
- ✅ Test missing model handling

**Tokenization:**
- ✅ Test tokenization with various inputs
- ✅ Test truncation at 512 tokens
- ✅ Test padding
- ✅ Test special tokens handling
- ✅ Test Unicode handling

**Inference:**
- ✅ Test single input inference
- ✅ Test batch inference
- ✅ Test output shape validation
- ✅ Test logits to probabilities conversion
- ✅ Test threshold application

**Caching:**
- ✅ Test cache insert/retrieve
- ✅ Test cache expiration (TTL)
- ✅ Test LRU eviction
- ✅ Test cache hit rate

**Hybrid Mode:**
- ✅ Test heuristic pre-filter
- ✅ Test ML fallback when heuristic is ambiguous
- ✅ Test heuristic-only mode

### 7.3 Integration Tests (15+ tests)

**Scanner Integration:**
- ✅ Test PromptInjection with ML enabled
- ✅ Test Toxicity with ML enabled
- ✅ Test Sentiment with ML enabled
- ✅ Test hybrid mode across all scanners
- ✅ Test fallback to heuristic on ML error

**Performance:**
- ✅ Test latency meets targets (<150ms)
- ✅ Test memory usage meets targets (<1.5GB)
- ✅ Test cache effectiveness (>40% hit rate)

**Cross-Platform:**
- ✅ Test on Linux x86_64
- ✅ Test on macOS x86_64
- ✅ Test on macOS arm64
- ✅ Test on Windows x86_64

### 7.4 Accuracy Validation Tests

**Methodology:**
1. Create test dataset of 1,000 examples per scanner
2. Run both Python llm-guard and Rust llm-shield
3. Compare predictions
4. Calculate metrics: precision, recall, F1, accuracy

**Acceptance Criteria:**
- **F1 score:** > 0.85 for all scanners
- **Accuracy deviation:** < 1% from Python baseline
- **False positive rate:** < 5%
- **False negative rate:** < 5%

**Test Datasets:**
- PromptInjection: 500 safe + 500 injection attempts
- Toxicity: 600 safe + 400 toxic (various categories)
- Sentiment: 333 positive + 334 neutral + 333 negative

### 7.5 Performance Benchmarks

**Metrics to Measure:**
- Latency (p50, p95, p99)
- Throughput (requests/sec)
- Memory usage (baseline, under load, peak)
- CPU usage (%)
- Cache hit rate (%)

**Scenarios:**
- Single request (cold start)
- Single request (warm)
- 100 concurrent requests
- 1000 requests (batch)
- Sustained load (10 minutes)

**Tools:**
- `benchmarks/scripts/bench_latency.sh`
- `benchmarks/scripts/bench_throughput.sh`
- `benchmarks/scripts/bench_memory.sh`
- `cargo bench`

---

## 8. Documentation Requirements

### 8.1 User Documentation

**1. User Guide: ML Model Setup**
- Prerequisites
- Model download instructions
- Configuration options
- Troubleshooting common issues
- Performance tuning tips

**2. Configuration Guide**
- `use_ml_model` flag
- `ml_threshold` tuning
- `model_variant` selection (FP16 vs INT8)
- Hybrid mode configuration
- Cache configuration

**3. Performance Guide**
- Expected latency/throughput
- Memory requirements
- GPU support (optional)
- Optimization recommendations

### 8.2 Developer Documentation

**1. Model Conversion Guide**
- Step-by-step conversion process
- Validation procedures
- Optimization techniques
- Troubleshooting

**2. Adding New Models**
- How to add a new model
- Integration checklist
- Testing requirements
- Documentation requirements

**3. API Documentation**
- `ModelRegistry` API
- `ModelLoader` API
- `InferenceEngine` API
- Scanner ML API extensions

### 8.3 Operational Documentation

**1. Deployment Guide**
- Model hosting setup (HuggingFace)
- Cache directory configuration
- Environment variables
- Resource requirements

**2. Monitoring Guide**
- Metrics to track
- Alerting thresholds
- Performance baselines
- Troubleshooting runbooks

**3. Upgrade Guide**
- Model version upgrades
- Backward compatibility
- Migration paths

---

## 9. Timeline and Resources

### 9.1 Gantt Chart (ASCII)

```
Week  │ 1      │ 2      │ 3      │ 4      │ 5      │ 6      │ 7      │
──────┼────────┼────────┼────────┼────────┼────────┼────────┼────────┤
8.1   │████████│        │        │        │        │        │        │ Model Conversion
8.2   │        │████████│        │        │        │        │        │ Download & Cache
8.3   │        │        │████████│        │        │        │        │ PromptInjection
8.4   │        │        │        │████████│        │        │        │ Toxicity
8.5   │        │        │        │████████│        │        │        │ Sentiment
8.6   │        │        │        │        │████████│        │        │ Optimization
8.7   │        │        │        │        │        │████████│        │ WASM (optional)
8.8   │        │        │        │        │        │        │████████│ Testing
──────┴────────┴────────┴────────┴────────┴────────┴────────┴────────┘
Doc   │    ██  │    ██  │    ██  │    ██  │    ██  │    ██  │    ██  │ Documentation
Review│        │    ✓   │        │    ✓   │        │    ✓   │    ✓   │ Checkpoints
```

### 9.2 Resource Requirements

**Team Composition:**
- **ML Engineer (1):** Model conversion, validation, optimization
- **Backend Engineer (1):** Rust integration, scanner implementation
- **DevOps Engineer (0.5):** HuggingFace setup, CI/CD, model distribution
- **QA Engineer (0.5):** Testing, validation, benchmarking
- **Total:** 3 FTE

**Skills Required:**
- Python (transformers, ONNX)
- Rust (async, ONNX Runtime)
- ML/DL fundamentals
- Performance optimization
- Testing and validation

**Infrastructure:**
- GPU instance for model conversion (optional, speeds up conversion)
- HuggingFace Hub account (free tier sufficient)
- CI/CD runners with sufficient memory (8GB+)
- Storage for models (~2GB)

### 9.3 Budget Estimate

| Item | Cost | Notes |
|------|------|-------|
| **Personnel** (7 weeks × 3 FTE × $150/hr × 40hr) | $126,000 | Primary cost |
| **GPU instance** (optional, 7 weeks × $100/week) | $700 | Speed up conversion |
| **HuggingFace Hub** | $0 | Free tier |
| **CI/CD compute** (7 weeks × $50/week) | $350 | GitHub Actions |
| **Model storage** (S3 backup) | $50 | 2GB × $0.023/GB |
| **Total** | **~$127,100** | Estimate |

**Cost Optimization:**
- Use local GPU for conversion (saves $700)
- Use free HuggingFace Hub (no cost)
- Optimize CI/CD usage (reduce cost)

---

## 10. Success Metrics

### 10.1 Quantitative Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Latency** | < 150ms (p95) | Benchmark framework |
| **Throughput** | > 100 req/sec (ML mode) | Load testing |
| **Memory** | < 1.5 GB (3 models loaded) | Process monitoring |
| **Accuracy (F1)** | > 0.85 | Validation tests |
| **Test Coverage** | > 90% | `cargo tarpaulin` |
| **Documentation** | 100% API coverage | Manual review |
| **Cache Hit Rate** | > 40% | Runtime metrics |
| **Heuristic Filter Rate** | > 60% | Runtime metrics |

### 10.2 Qualitative Metrics

- ✅ **User Satisfaction:** Positive feedback on ML accuracy
- ✅ **Developer Experience:** Easy to configure and use
- ✅ **Operational Stability:** No production incidents
- ✅ **Community Adoption:** Increased GitHub stars/downloads

### 10.3 Go/No-Go Criteria

**Week 2 Checkpoint:**
- ✅ All 3 models converted successfully
- ✅ Accuracy validation shows <1% deviation
- ✅ Models load in ort without errors

**Week 4 Checkpoint:**
- ✅ At least 2 scanners integrated with ML
- ✅ Latency < 200ms (initial, before optimization)
- ✅ No critical bugs

**Week 6 Checkpoint:**
- ✅ All 3 scanners integrated
- ✅ Latency < 150ms (optimized)
- ✅ Test coverage > 80%

**Week 7 Final:**
- ✅ All metrics met
- ✅ Documentation complete
- ✅ Ready for production deployment

---

## 11. Appendix

### 11.1 Model Card Template

```markdown
# Model Card: {Model Name}

## Model Details
- **Model Name:** {name}
- **Model Type:** {architecture}
- **Task:** {task}
- **Parameters:** {count}
- **License:** {license}

## Intended Use
- **Primary Use:** {use case}
- **Out of Scope:** {limitations}

## Training Data
- **Dataset:** {dataset name}
- **Size:** {size}
- **Preprocessing:** {preprocessing steps}

## Performance
- **Accuracy:** {metrics}
- **Latency:** {latency}
- **Memory:** {memory}

## Ethical Considerations
- **Bias:** {bias analysis}
- **Fairness:** {fairness considerations}

## Citation
```bibtex
{citation}
```
```

### 11.2 Example Model Registry Entry

```json
{
  "models": [
    {
      "id": "prompt-injection-deberta-v3",
      "name": "PromptInjection DeBERTa-v3 FP16",
      "task": "prompt-injection",
      "architecture": "deberta-v3-base",
      "source": "protectai/deberta-v3-base-prompt-injection-v2",
      "version": "1.0.0",
      "format": "onnx",
      "precision": "fp16",
      "size_mb": 350,
      "url": "https://huggingface.co/llm-shield/prompt-injection-deberta-v3-fp16/resolve/main/model.onnx",
      "checksum": "sha256:abc123...",
      "performance": {
        "latency_ms": 80,
        "memory_mb": 800,
        "accuracy": {
          "f1": 0.90,
          "precision": 0.91,
          "recall": 0.89
        }
      },
      "config": {
        "max_length": 512,
        "num_labels": 2,
        "tokenizer_url": "https://huggingface.co/llm-shield/prompt-injection-deberta-v3-fp16/resolve/main/tokenizer.json"
      }
    }
  ]
}
```

### 11.3 References

- [ONNX Runtime Documentation](https://onnxruntime.ai/docs/)
- [HuggingFace Optimum](https://huggingface.co/docs/optimum/)
- [Model Quantization Guide](https://onnxruntime.ai/docs/performance/quantization.html)
- [Python llm-guard Repository](https://github.com/protectai/llm-guard)
- [DeBERTa Paper](https://arxiv.org/abs/2006.03654)
- [RoBERTa Paper](https://arxiv.org/abs/1907.11692)

---

## 12. Conclusion

This implementation plan provides a comprehensive, actionable roadmap for Phase 8: Pre-trained ML Models. The plan is structured to minimize risk through:

1. **Incremental delivery** - 8 sub-phases with clear deliverables
2. **Hybrid approach** - Combining ML with heuristics for optimal performance
3. **Extensive testing** - 50+ unit tests, 15+ integration tests, accuracy validation
4. **Clear metrics** - Quantitative success criteria for every aspect
5. **Risk mitigation** - Identified risks with specific mitigation strategies

**Next Steps:**
1. ✅ Review and approve this plan
2. Allocate resources (3 FTE for 7 weeks)
3. Set up HuggingFace organization account
4. Begin Phase 8.1: Model Conversion Infrastructure

**Expected Outcome:**
By the end of Phase 8, llm-shield-rs will have production-ready ML models that improve accuracy by 30-50% while maintaining acceptable performance through hybrid detection, caching, and optimization.

---

**Document Status:** ✅ Ready for Implementation
**Last Updated:** 2025-10-30
**Version:** 1.0
**Approved By:** [Pending]
