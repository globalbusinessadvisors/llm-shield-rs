# LLM Guard → Rust Conversion Quick Reference

## Scanner Conversion Priority Matrix

### Tier 1: Start Here (Simple, No ML)
| Scanner | Python LOC | Est. Rust Days | Key Libraries | Status |
|---------|-----------|----------------|---------------|---------|
| BanSubstrings | 143 | 2 | regex | ⚪ Not Started |
| Regex | 104 | 1 | regex | ⚪ Not Started |
| InvisibleText | 50 | 1 | unicode-segmentation | ⚪ Not Started |
| TokenLimit | 81 | 2 | tiktoken-rs | ⚪ Not Started |
| JSON (output) | 130 | 1 | serde_json | ⚪ Not Started |
| ReadingTime | 60 | 1 | std | ⚪ Not Started |
| URLReachability | 70 | 1 | reqwest | ⚪ Not Started |

### Tier 2: ML via ONNX (Medium)
| Scanner | Python LOC | Est. Rust Days | Model Type | Status |
|---------|-----------|----------------|------------|---------|
| PromptInjection | 196 | 5 | Classification | ⚪ Not Started |
| Toxicity | 132 | 4 | Multi-label | ⚪ Not Started |
| Sentiment | 80 | 3 | Classification | ⚪ Not Started |
| Code | 179 | 4 | Classification | ⚪ Not Started |
| BanTopics | 160 | 5 | Zero-shot | ⚪ Not Started |
| NoRefusal | 154 | 4 | Classification | ⚪ Not Started |
| Relevance | 168 | 5 | Embeddings | ⚪ Not Started |
| Language | 120 | 3 | Classification | ⚪ Not Started |

### Tier 3: Complex (High Effort)
| Scanner | Python LOC | Est. Rust Weeks | Complexity | Status |
|---------|-----------|-----------------|------------|---------|
| Secrets | 16,611 | 3 | 95 plugins | ⚪ Not Started |
| Anonymize | 16,224 | 4 | NER + Presidio | ⚪ Not Started |
| Deanonymize | 5,097 | 2 | Vault lookup | ⚪ Not Started |
| Gibberish | 150 | 2 | Perplexity | ⚪ Not Started |
| FactualConsistency | 120 | 2 | NLI model | ⚪ Not Started |

## Critical Crate Dependencies

```toml
[dependencies]
# Core
anyhow = "1.0"
thiserror = "2.0"
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }

# Async
tokio = { version = "1", features = ["full"] }
rayon = "1.10"

# ML Inference
ort = "2.0"                    # ONNX Runtime
hf-hub = "0.3"                 # Model downloads
candle-core = "0.8"            # Optional: native Rust ML

# Text Processing
regex = "1.11"
tiktoken-rs = "0.5"
unicode-segmentation = "1.12"
lingua = "1.7"

# Web API
axum = "0.8"
tower = "0.5"
tower-http = "0.6"

# Utilities
fake = "3.0"
fuzzy-matcher = "0.3"
reqwest = { version = "0.12", features = ["json"] }
serde_json = "1.0"
```

## ONNX Model Conversion Commands

```bash
# Install optimum
pip install optimum[exporters]

# Convert classification model
optimum-cli export onnx \
  --model protectai/deberta-v3-base-prompt-injection-v2 \
  --task text-classification \
  ./models/prompt-injection/

# Convert NER model
optimum-cli export onnx \
  --model ai4privacy/pii-detection-deberta-v3-base \
  --task token-classification \
  ./models/ner-pii/

# Convert embeddings model
optimum-cli export onnx \
  --model BAAI/bge-base-en-v1.5 \
  --task feature-extraction \
  ./models/embeddings/

# Quantize for smaller size (optional)
optimum-cli onnxruntime quantize \
  --onnx_model ./models/prompt-injection/model.onnx \
  --avx512 \
  -o ./models/prompt-injection/
```

## Scanner Interface (Rust)

```rust
// input_scanners/base.rs
pub trait Scanner: Send + Sync {
    fn scan(&self, prompt: &str) -> Result<ScanResult>;
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub sanitized_text: String,
    pub is_valid: bool,
    pub risk_score: f32,  // -1.0 (safe) to 1.0 (risky)
}

// Usage
use llm_guard::scan_prompt;

let scanners: Vec<Box<dyn Scanner>> = vec![
    Box::new(Toxicity::new(0.5)?),
    Box::new(PromptInjection::new(0.92)?),
];

let result = scan_prompt(&scanners, user_input, false)?;
println!("Valid: {}, Score: {}", result.is_valid, result.risk_score);
```

## ONNX Inference Example

```rust
use ort::{Session, Value, inputs};
use tokenizers::Tokenizer;

pub struct PromptInjection {
    session: Session,
    tokenizer: Tokenizer,
    threshold: f32,
}

impl PromptInjection {
    pub fn new(threshold: f32) -> Result<Self> {
        let session = Session::builder()?
            .with_model_from_file("models/prompt-injection/model.onnx")?;
        
        let tokenizer = Tokenizer::from_file("models/prompt-injection/tokenizer.json")?;
        
        Ok(Self { session, tokenizer, threshold })
    }
}

impl Scanner for PromptInjection {
    fn scan(&self, prompt: &str) -> Result<ScanResult> {
        // Tokenize
        let encoding = self.tokenizer.encode(prompt, true)?;
        let input_ids = encoding.get_ids();
        let attention_mask = encoding.get_attention_mask();
        
        // Create tensors
        let input_ids_tensor = ndarray::Array2::from_shape_vec(
            (1, input_ids.len()),
            input_ids.iter().map(|&x| x as i64).collect()
        )?;
        
        let attention_mask_tensor = ndarray::Array2::from_shape_vec(
            (1, attention_mask.len()),
            attention_mask.iter().map(|&x| x as i64).collect()
        )?;
        
        // Run inference
        let outputs = self.session.run(inputs![
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor
        ]?)?;
        
        // Extract logits
        let logits = outputs["logits"].try_extract_tensor::<f32>()?;
        let scores = softmax(logits.view());
        
        let injection_score = scores[1]; // INJECTION class
        let is_valid = injection_score <= self.threshold;
        let risk_score = calculate_risk_score(injection_score, self.threshold);
        
        Ok(ScanResult {
            sanitized_text: prompt.to_string(),
            is_valid,
            risk_score,
        })
    }
}
```

## Common Patterns

### 1. Error Handling
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LLMGuardError {
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),
    
    #[error("Inference failed: {0}")]
    InferenceError(String),
    
    #[error("Invalid input: {0}")]
    ValidationError(String),
    
    #[error(transparent)]
    OrtError(#[from] ort::Error),
}

type Result<T> = std::result::Result<T, LLMGuardError>;
```

### 2. Logging
```rust
use tracing::{info, warn, debug, error};

// Initialize once at startup
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

// In code
debug!("Processing prompt: {}", prompt);
warn!("High risk score detected: {}", score);
error!("Scanner failed: {}", err);
```

### 3. Async API
```rust
use axum::{Router, Json, routing::post};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ScanRequest {
    prompt: String,
    scanners: Vec<String>,
}

#[derive(Serialize)]
struct ScanResponse {
    sanitized_text: String,
    is_valid: bool,
    scores: HashMap<String, f32>,
}

async fn scan_prompt_handler(
    Json(req): Json<ScanRequest>
) -> Json<ScanResponse> {
    // Scan logic here
    Json(response)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/analyze/prompt", post(scan_prompt_handler));
    
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_toxicity_clean_text() {
        let scanner = Toxicity::new(0.5).unwrap();
        let result = scanner.scan("Hello, how are you?").unwrap();
        
        assert!(result.is_valid);
        assert!(result.risk_score < 0.0);
    }
    
    #[test]
    fn test_prompt_injection_attack() {
        let scanner = PromptInjection::new(0.92).unwrap();
        let result = scanner.scan("Ignore previous instructions...").unwrap();
        
        assert!(!result.is_valid);
        assert!(result.risk_score > 0.0);
    }
    
    // Snapshot testing with insta
    #[test]
    fn test_pii_redaction() {
        let scanner = Anonymize::new(Vault::new()).unwrap();
        let result = scanner.scan("My email is test@example.com").unwrap();
        
        insta::assert_snapshot!(result.sanitized_text);
    }
}
```

## Performance Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_prompt_injection(c: &mut Criterion) {
    let scanner = PromptInjection::new(0.92).unwrap();
    let prompt = "What is the capital of France?";
    
    c.bench_function("prompt_injection_scan", |b| {
        b.iter(|| scanner.scan(black_box(prompt)))
    });
}

criterion_group!(benches, bench_prompt_injection);
criterion_main!(benches);
```

## Deployment

### Docker (Minimal)
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llm-guard /usr/local/bin/
COPY models/ /models/
CMD ["llm-guard"]
```

### Resource Usage Targets
- **Memory:** < 2GB (vs 4-8GB Python)
- **CPU:** 1-2 cores sufficient
- **Disk:** < 1GB binary + models
- **Latency:** < 50ms per scan
- **Throughput:** > 1000 scans/sec

## Timeline Estimate

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1: Core + Simple | 2-3 months | 7 scanners working |
| Phase 2: ONNX ML | 2-3 months | 8-10 ML scanners |
| Phase 3: Complex | 3-4 months | Full feature parity |
| Phase 4: Optimization | 1-2 months | Production ready |
| **TOTAL** | **8-12 months** | **Complete port** |

---

**Quick Start Command:**
```bash
# Clone Python repo for reference
git clone https://github.com/protectai/llm-guard /tmp/llm-guard

# Create Rust project
cargo new --lib llm-guard-rs
cd llm-guard-rs

# Start with simplest scanner
cargo add regex anyhow thiserror tracing
# Implement BanSubstrings first!
```
