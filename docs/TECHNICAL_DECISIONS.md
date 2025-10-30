# Technical Decisions for Rust Conversion

## Decision Matrix: Key Architecture Choices

### Decision 1: ML Inference Backend

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **ONNX Runtime** | ✅ Mature, fast<br>✅ All models convertible<br>✅ Official Rust bindings<br>✅ Production-ready | ⚠️ Requires ONNX conversion<br>⚠️ Less flexible than PyTorch | **PRIMARY CHOICE** |
| **Candle (HuggingFace)** | ✅ Native Rust<br>✅ Growing ecosystem<br>✅ SafeTensors support | ⚠️ Still maturing<br>⚠️ Fewer models<br>⚠️ Less documentation | **FUTURE MIGRATION** |
| **PyO3 (Python bridge)** | ✅ Full compatibility<br>✅ Easy migration | ❌ Not pure Rust<br>❌ Python runtime required<br>❌ Deployment complexity | **FALLBACK ONLY** |
| **Burn** | ✅ Pure Rust<br>✅ Flexible | ⚠️ Very early stage<br>⚠️ Limited models | **NOT RECOMMENDED** |

**Decision:** Start with ONNX Runtime, migrate to Candle incrementally

**Rationale:**
- ONNX provides immediate path to production
- Candle is improving rapidly - good for future
- PyO3 as escape hatch for complex cases
- Can run both in parallel during migration

---

### Decision 2: Web Framework

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **Axum** | ✅ Modern, ergonomic<br>✅ Tower ecosystem<br>✅ Tokio native<br>✅ Type-safe extractors | ⚠️ Newer than Actix | **RECOMMENDED** |
| **Actix-web** | ✅ Very mature<br>✅ Battle-tested<br>✅ Excellent performance | ⚠️ Older macro-based API<br>⚠️ Less ergonomic | **ALTERNATIVE** |
| **Rocket** | ✅ Developer-friendly<br>✅ Good docs | ⚠️ Slower development<br>⚠️ Custom async runtime | **NOT RECOMMENDED** |

**Decision:** Axum

**Rationale:**
- Better integration with Tower middleware
- More type-safe than Actix
- Growing community, HuggingFace uses it
- Excellent async performance

---

### Decision 3: Error Handling

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **thiserror + anyhow** | ✅ Standard approach<br>✅ Ergonomic<br>✅ Good error context | ⚠️ Two crates needed | **RECOMMENDED** |
| **snafu** | ✅ Feature-rich<br>✅ Context handling | ⚠️ More complex<br>⚠️ Heavier | **ALTERNATIVE** |
| **Custom only** | ✅ Full control | ❌ Reinventing wheel<br>❌ More code | **NOT RECOMMENDED** |

**Decision:** thiserror for library errors, anyhow for application

**Pattern:**
```rust
// Library (llm-guard crate)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLMGuardError {
    #[error("model error: {0}")]
    ModelError(String),
}

// Application (API server)
use anyhow::{Context, Result};

fn load_model() -> Result<Model> {
    Model::load().context("Failed to load model")
}
```

---

### Decision 4: Logging/Tracing

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **tracing** | ✅ Standard Rust<br>✅ Structured logging<br>✅ OpenTelemetry support<br>✅ Async-aware | ⚠️ Learning curve | **RECOMMENDED** |
| **log + env_logger** | ✅ Simple<br>✅ Lightweight | ❌ Less features<br>❌ No structured logging | **NOT RECOMMENDED** |
| **slog** | ✅ Structured<br>✅ Flexible | ⚠️ More verbose<br>⚠️ Older API | **ALTERNATIVE** |

**Decision:** tracing + tracing-subscriber

**Configuration:**
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .with_target(false)
    .json()  // For production
    .init();
```

---

### Decision 5: Tokenization Strategy

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **tiktoken-rs** | ✅ OpenAI compatible<br>✅ Maintained<br>✅ Fast | ⚠️ OpenAI encodings only | **PRIMARY (TokenLimit)** |
| **tokenizers** | ✅ HuggingFace standard<br>✅ All models<br>✅ Python parity | ⚠️ Larger dependency | **PRIMARY (ML)** |
| **Custom** | ✅ Full control | ❌ Complex<br>❌ Maintenance burden | **NOT RECOMMENDED** |

**Decision:** Both - tiktoken-rs for token counting, tokenizers for ML

**Usage:**
```rust
// TokenLimit scanner
use tiktoken_rs::get_bpe_from_model;
let bpe = get_bpe_from_model("gpt-4")?;
let tokens = bpe.encode_with_special_tokens(text);

// ML scanners
use tokenizers::Tokenizer;
let tokenizer = Tokenizer::from_file("tokenizer.json")?;
let encoding = tokenizer.encode(text, true)?;
```

---

### Decision 6: Configuration Management

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **config crate** | ✅ Multiple formats<br>✅ Environment vars<br>✅ Layered configs | ⚠️ Complex API | **RECOMMENDED** |
| **figment** | ✅ Type-safe<br>✅ Provider system | ⚠️ Less documentation | **ALTERNATIVE** |
| **Manual (serde)** | ✅ Simple<br>✅ Lightweight | ❌ Limited features | **FOR SIMPLE CASES** |

**Decision:** config crate for API, serde for library

**Example:**
```rust
use config::{Config, Environment, File};

let settings = Config::builder()
    .add_source(File::with_name("config/default"))
    .add_source(Environment::with_prefix("LLM_GUARD"))
    .build()?;

let threshold: f32 = settings.get("toxicity.threshold")?;
```

---

### Decision 7: Async Runtime

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **Tokio** | ✅ Most popular<br>✅ Best docs<br>✅ Rich ecosystem | ⚠️ Heavier than async-std | **RECOMMENDED** |
| **async-std** | ✅ Lighter<br>✅ std-like API | ⚠️ Smaller ecosystem | **ALTERNATIVE** |
| **smol** | ✅ Minimal<br>✅ Fast | ⚠️ Less mature<br>⚠️ Small ecosystem | **NOT RECOMMENDED** |

**Decision:** Tokio (full feature set)

**Rationale:**
- Best integration with Axum, Tower, tonic
- Most ML crates use Tokio
- Better debugging tools
- Larger community

---

### Decision 8: Testing Strategy

| Component | Approach | Tools |
|-----------|----------|-------|
| **Unit Tests** | Per-scanner | Built-in `#[test]` |
| **Integration** | Full pipeline | `#[tokio::test]` |
| **Snapshots** | Output validation | `insta` |
| **Property-based** | Edge cases | `proptest` |
| **Benchmarks** | Performance | `criterion` |
| **Coverage** | CI enforcement | `cargo-tarpaulin` |

**Test Organization:**
```
tests/
├── unit/
│   ├── scanners/
│   │   ├── toxicity.rs
│   │   └── prompt_injection.rs
│   └── utils/
├── integration/
│   ├── api_tests.rs
│   └── pipeline_tests.rs
└── benchmarks/
    └── scanner_benchmarks.rs
```

---

### Decision 9: Model Storage & Loading

| Option | Pros | Cons | Recommendation |
|--------|------|------|----------------|
| **hf-hub crate** | ✅ Official client<br>✅ Auto caching<br>✅ Auth support | ⚠️ Network dependency | **RECOMMENDED** |
| **Bundled in binary** | ✅ No network<br>✅ Faster startup | ❌ Huge binary<br>❌ Update complexity | **NOT RECOMMENDED** |
| **Docker volumes** | ✅ Separation of concerns<br>✅ Easy updates | ⚠️ Deployment complexity | **FOR PRODUCTION** |

**Decision:** hf-hub for development, Docker volumes for production

**Implementation:**
```rust
use hf_hub::{api::sync::Api, Repo, RepoType};

let api = Api::new()?;
let repo = api.repo(Repo::new("protectai/model".to_string(), RepoType::Model));
let model_file = repo.get("model.onnx")?;
```

**Production Docker:**
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llm-guard /usr/local/bin/
VOLUME /models  # Mount pre-downloaded models here
ENV MODEL_PATH=/models
CMD ["llm-guard"]
```

---

### Decision 10: Parallelization Strategy

| Use Case | Solution | Rationale |
|----------|----------|-----------|
| **Multiple Scanners** | `rayon::par_iter()` | CPU-bound, embarrassingly parallel |
| **Batch Requests** | `tokio::spawn` | I/O-bound, async concurrency |
| **Sentence Scanning** | `rayon::par_iter()` | Independent compute tasks |
| **API Endpoints** | Tokio runtime | Network I/O |
| **Model Inference** | Thread pool | Blocking operations |

**Example:**
```rust
use rayon::prelude::*;

// Parallel scanner execution
let results: Vec<_> = scanners
    .par_iter()
    .map(|scanner| scanner.scan(prompt))
    .collect();

// Async API
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/scan", post(scan_handler));
    
    tokio::spawn(async {
        // Background model loading
    });
    
    serve(app).await;
}
```

---

### Decision 11: Secret Detection Implementation

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Port Python plugins** | ✅ Feature parity<br>✅ Validated patterns | ⚠️ 95 plugins to port<br>⚠️ Manual work | **RECOMMENDED** |
| **Use gitleaks** | ✅ Maintained<br>✅ Active community | ❌ Go binary<br>❌ External dependency | **NOT RECOMMENDED** |
| **Use trufflehog** | ✅ Entropy detection | ❌ Go binary<br>❌ Integration complexity | **NOT RECOMMENDED** |

**Decision:** Port all 95 plugins manually

**Implementation Pattern:**
```rust
pub struct SecretDetector {
    patterns: Vec<CompiledPattern>,
}

#[derive(Debug)]
struct CompiledPattern {
    name: &'static str,
    regex: Regex,
    entropy_threshold: Option<f32>,
}

// Load from TOML config
impl SecretDetector {
    pub fn from_config(path: &str) -> Result<Self> {
        let config: SecretConfig = toml::from_str(&fs::read_to_string(path)?)?;
        // ...
    }
}
```

**Config Format (TOML):**
```toml
[[secrets]]
name = "AWS Access Key"
pattern = "AKIA[0-9A-Z]{16}"
entropy_threshold = 4.5

[[secrets]]
name = "OpenAI API Key"
pattern = "sk-[a-zA-Z0-9]{48}"
```

---

### Decision 12: PII Detection (Presidio Alternative)

| Approach | Pros | Cons | Recommendation |
|----------|------|------|----------------|
| **Custom NER + Regex** | ✅ Rust-native<br>✅ Controllable | ⚠️ Less accurate than Presidio<br>⚠️ More work | **RECOMMENDED** |
| **Port Presidio** | ✅ Feature parity | ❌ Huge effort<br>❌ Complex codebase | **NOT RECOMMENDED** |
| **PyO3 to Presidio** | ✅ Easy<br>✅ Accurate | ❌ Python runtime<br>❌ Not pure Rust | **FALLBACK** |

**Decision:** Custom implementation with phases

**Phase 1: Regex-based (60% accuracy, 2 weeks)**
```rust
pub struct PiiDetector {
    email_regex: Regex,
    phone_regex: Regex,
    ssn_regex: Regex,
    credit_card_regex: Regex,
    // ... more patterns
}
```

**Phase 2: Add NER via ONNX (85% accuracy, 4 weeks)**
```rust
pub struct NerDetector {
    session: Session,
    tokenizer: Tokenizer,
}

impl NerDetector {
    pub fn detect_entities(&self, text: &str) -> Vec<Entity> {
        // ONNX inference for PERSON, ORG, LOC
    }
}
```

**Phase 3: Context-aware rules (95% accuracy, 6 weeks)**
```rust
pub struct ContextualPii {
    ner: NerDetector,
    regex: RegexDetector,
    rules: Vec<ValidationRule>,
}

// e.g., "John" only PII if near "name:", "called", etc.
```

---

## Summary of Decisions

| Decision | Choice | Why |
|----------|--------|-----|
| ML Backend | ONNX → Candle | Production-ready now, future-proof |
| Web Framework | Axum | Modern, type-safe, ergonomic |
| Error Handling | thiserror + anyhow | Standard, best practices |
| Logging | tracing | Structured, async-aware, standard |
| Tokenization | tiktoken-rs + tokenizers | Best of both worlds |
| Config | config crate | Flexible, standard |
| Async | Tokio (full) | Ecosystem compatibility |
| Testing | Built-in + insta + criterion | Comprehensive coverage |
| Model Loading | hf-hub + Docker volumes | Dev convenience + prod control |
| Parallelism | Rayon + Tokio | CPU + I/O parallel |
| Secrets | Manual port | Control + maintenance |
| PII | Phased custom impl | Pragmatic approach |

---

## Migration Strategy

### Phase 1: ONNX-First (Months 1-6)
- All ML scanners use ONNX Runtime
- Focus on getting working, not perfect
- Accept ONNX conversion overhead
- Build confidence and infrastructure

### Phase 2: Candle Exploration (Months 7-9)
- Prototype simple scanners in Candle
- Compare accuracy and performance
- Identify blockers
- Start gradual migration

### Phase 3: Optimization (Months 10-12)
- Replace ONNX with Candle where possible
- Keep ONNX for complex models
- Profile and optimize hotspots
- Production hardening

---

## Non-Negotiables

✅ **MUST HAVE:**
- Same accuracy as Python (±1%)
- Type-safe API
- Comprehensive tests (>80% coverage)
- Production-ready logging
- OpenAPI-compatible API
- Docker deployment

⚠️ **NICE TO HAVE:**
- 100% Candle (no ONNX)
- WebAssembly support
- Native plugins system
- Real-time streaming

❌ **EXPLICITLY AVOID:**
- Python runtime in production (PyO3)
- Bundling models in binary
- Blocking async runtime
- Unsafe code without justification
- Custom ML implementations

---

**Last Updated:** 2025-10-30  
**Status:** Architecture Decisions Finalized  
**Next:** Begin Phase 1 implementation
