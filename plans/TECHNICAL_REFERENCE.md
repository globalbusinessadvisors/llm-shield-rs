# LLM-Guard Rust Technical Reference

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Dependency Deep Dive](#dependency-deep-dive)
3. [Performance Characteristics](#performance-characteristics)
4. [Memory Management](#memory-management)
5. [WASM Compatibility](#wasm-compatibility)
6. [Security Considerations](#security-considerations)
7. [Benchmarking Guide](#benchmarking-guide)
8. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Application Layer                     │
│  ┌────────────┐  ┌────────────┐  ┌─────────────┐            │
│  │  REST API  │  │  WASM Pkg  │  │  CLI Tool   │            │
│  └────────────┘  └────────────┘  └─────────────┘            │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                      Pipeline Layer                          │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  GuardPipeline                                       │   │
│  │  - Scanner orchestration                             │   │
│  │  - Result aggregation                                │   │
│  │  - Configuration management                          │   │
│  └──────────────────────────────────────────────────────┘   │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                      Scanner Layer                           │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │ Input Scanners  │  │ Output Scanners │                   │
│  │                 │  │                 │                   │
│  │ - BanSubstrings │  │ - Sensitive     │                   │
│  │ - PromptInjection│ │ - Relevance     │                   │
│  │ - Toxicity      │  │ - Bias          │                   │
│  │ - Anonymize     │  │ - FactCheck     │                   │
│  └─────────────────┘  └─────────────────┘                   │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                      Core Layer                              │
│  ┌────────────┐  ┌────────────┐  ┌─────────────┐           │
│  │   Traits   │  │   Types    │  │  Utilities  │           │
│  │  Scanner   │  │ ScanResult │  │  Text Proc  │           │
│  │  Input     │  │ ScanError  │  │  Regex      │           │
│  │  Output    │  │ Entity     │  │  Config     │           │
│  └────────────┘  └────────────┘  └─────────────┘           │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                   Infrastructure Layer                       │
│  ┌────────────┐  ┌────────────┐  ┌─────────────┐           │
│  │ ONNX RT    │  │  Tokenizer │  │   Storage   │           │
│  │ (ML)       │  │  (HF)      │  │   (Cache)   │           │
│  └────────────┘  └────────────┘  └─────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**Core Layer:**
- Define scanner interfaces and contracts
- Provide common types (Result, Error, Entity)
- Implement shared utilities (text processing, regex)
- No dependencies on specific scanner implementations

**Scanner Layer:**
- Implement specific detection algorithms
- Manage ML model lifecycle
- Handle scanner-specific configuration
- Return standardized ScanResult

**Pipeline Layer:**
- Orchestrate multiple scanners
- Aggregate results
- Handle timeouts and errors
- Provide builder pattern for easy configuration

**Application Layer:**
- Expose functionality via different interfaces (REST, WASM, CLI)
- Handle serialization/deserialization
- Manage authentication/authorization (API)
- Provide user-friendly error messages

---

## Dependency Deep Dive

### Core Dependencies Analysis

#### 1. Regex Processing

**Primary: `regex` crate**
```toml
regex = { version = "1.10", features = ["unicode", "perf"] }
```

**Capabilities:**
- Fast DFA-based matching
- Unicode support
- Capture groups
- Look-ahead/look-behind via `fancy-regex`

**Limitations:**
- No built-in fuzzy matching
- Regex must be compiled (but can be cached)

**Python Equivalent:**
```python
# Python
import re
pattern = re.compile(r"\d{3}-\d{3}-\d{4}")
```

**Rust:**
```rust
use regex::Regex;
use once_cell::sync::Lazy;

static PHONE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d{3}-\d{3}-\d{4}").unwrap()
});
```

**Performance:**
- Compilation: ~1-10μs (cached)
- Matching: ~100ns - 10μs depending on pattern
- 2-10x faster than Python `re` module

#### 2. Multi-Pattern Matching

**Primary: `aho-corasick` crate**
```toml
aho-corasick = { version = "1.1", features = ["std"] }
```

**Use Cases:**
- BanSubstrings scanner
- Secret detection
- Any multi-pattern matching

**Example:**
```rust
use aho_corasick::{AhoCorasick, MatchKind};

let patterns = vec!["banned", "forbidden", "restricted"];
let matcher = AhoCorasick::builder()
    .match_kind(MatchKind::LeftmostLongest)
    .build(&patterns)
    .unwrap();

let text = "This contains banned and forbidden words";
for mat in matcher.find_iter(text) {
    println!("Found: {}", &text[mat.start()..mat.end()]);
}
```

**Performance:**
- Construction: O(m) where m = total pattern length
- Search: O(n) where n = text length
- 10-100x faster than sequential regex matching

#### 3. ONNX Runtime

**Primary: `ort` (ONNX Runtime)**
```toml
ort = { version = "1.16", features = ["download-binaries"] }
```

**Capabilities:**
- Load ONNX models
- CPU and GPU execution
- Graph optimization
- Quantization support

**Model Loading:**
```rust
use ort::{Environment, Session, SessionBuilder};

let environment = Environment::builder()
    .with_name("llm-guard")
    .build()?;

let session = SessionBuilder::new(&environment)?
    .with_optimization_level(GraphOptimizationLevel::Level3)?
    .with_intra_threads(4)?
    .with_model_from_file("model.onnx")?;
```

**Inference:**
```rust
use ort::Value;

let input_tensor = Value::from_array(session.allocator(), &input_array)?;
let outputs = session.run(vec![input_tensor])?;
let output_tensor = outputs[0].try_extract::<f32>()?;
```

**Performance:**
- Model loading: 100ms - 2s (depending on size)
- Inference (DeBERTa-base): 20-50ms (CPU), 5-10ms (GPU)
- Memory: 200-500MB per loaded model

**WASM Support:**
- Experimental: `ort` has limited WASM support
- Alternative: Use `tract` or `candle` for pure Rust inference

#### 4. Tokenizers

**Primary: `tokenizers` (HuggingFace)**
```toml
tokenizers = { version = "0.15", features = ["http"] }
```

**Capabilities:**
- WordPiece, BPE, Unigram tokenization
- Pre-trained tokenizer loading
- Fast Rust implementation

**Example:**
```rust
use tokenizers::Tokenizer;

let tokenizer = Tokenizer::from_file("tokenizer.json")?;
let encoding = tokenizer.encode("Hello, world!", true)?;

let input_ids = encoding.get_ids();
let attention_mask = encoding.get_attention_mask();
```

**Performance:**
- Loading: 10-50ms
- Tokenization: 100μs - 1ms (for typical prompts)
- 5-10x faster than Python HuggingFace tokenizers

#### 5. Web Framework (API)

**Primary: `axum`**
```toml
axum = { version = "0.7", features = ["macros", "ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace", "compression"] }
```

**Why Axum:**
- Built on `tokio` and `tower`
- Type-safe extractors
- Excellent ergonomics
- Strong ecosystem

**Alternative: `actix-web`**
- More mature
- Slightly better raw performance
- Less type-safe

**Example:**
```rust
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};

async fn scan_handler(
    State(pipeline): State<Arc<GuardPipeline>>,
    Json(request): Json<ScanRequest>,
) -> Json<ScanResponse> {
    // Handle request
}

let app = Router::new()
    .route("/scan", post(scan_handler))
    .with_state(pipeline);
```

#### 6. Serialization

**Primary: `serde` + `serde_json`**
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Capabilities:**
- Zero-copy deserialization
- Custom serialization logic
- JSON, YAML, TOML, MessagePack support

**Example:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ScanRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
}
```

**Performance:**
- Serialization: 1-10μs (small objects)
- Deserialization: 2-20μs (small objects)
- 2-5x faster than Python `json` module

#### 7. Unicode Processing

**Primary: `unicode-segmentation` + `unicode-normalization`**
```toml
unicode-segmentation = "1.10"
unicode-normalization = "0.1"
```

**Use Cases:**
- Grapheme cluster iteration
- Word boundaries
- Sentence boundaries
- Unicode normalization (NFC, NFD, NFKC, NFKD)

**Example:**
```rust
use unicode_segmentation::UnicodeSegmentation;

let text = "Hello, 世界! 👋";

// Grapheme clusters
for grapheme in text.graphemes(true) {
    println!("{}", grapheme);
}

// Words
for word in text.split_word_bounds() {
    println!("{}", word);
}
```

#### 8. Fuzzy Matching

**Primary: `strsim` or `fuzzy-matcher`**
```toml
strsim = "0.10"  # String similarity metrics
# OR
fuzzy-matcher = "0.3"  # Fuzzy string matching
```

**Algorithms:**
- Levenshtein distance
- Jaro-Winkler
- Hamming distance
- Damerau-Levenshtein

**Example:**
```rust
use strsim::levenshtein;

let distance = levenshtein("kitten", "sitting");
assert_eq!(distance, 3);
```

### Dependency Comparison Matrix

| Feature | Python Library | Rust Crate | Performance Gain | WASM Support |
|---------|---------------|------------|------------------|--------------|
| Regex | `re` | `regex` | 2-10x | ✅ |
| Multi-pattern | - | `aho-corasick` | 10-100x | ✅ |
| ML Inference | `torch` | `ort` (ONNX) | 1.5-2x | ⚠️ Experimental |
| Tokenization | `transformers` | `tokenizers` | 5-10x | ✅ |
| Web Server | `fastapi` | `axum` | 3-5x | ❌ |
| JSON | `json` | `serde_json` | 2-5x | ✅ |
| Unicode | `unicodedata` | `unicode-*` | 2-4x | ✅ |
| Fuzzy Match | `fuzzywuzzy` | `strsim` | 5-10x | ✅ |

---

## Performance Characteristics

### Latency Targets

**Rule-Based Scanners:**
```
BanSubstrings:     <1ms (p50), <2ms (p99)
BanCode:           <2ms (p50), <5ms (p99)
Regex:             <1ms (p50), <3ms (p99)
Secrets:           <3ms (p50), <10ms (p99)
```

**ML-Based Scanners (CPU):**
```
PromptInjection:   <25ms (p50), <50ms (p99)
Toxicity:          <20ms (p50), <45ms (p99)
Anonymize:         <40ms (p50), <80ms (p99)
Relevance:         <30ms (p50), <60ms (p99)
```

**Pipeline (3 scanners):**
```
Sequential:        <50ms (p50), <100ms (p99)
Parallel:          <30ms (p50), <70ms (p99)
```

### Throughput Targets

**Single-threaded:**
```
Rule-based:        10,000-50,000 requests/sec
ML-based:          50-200 requests/sec
```

**Multi-threaded (8 cores):**
```
Rule-based:        50,000-200,000 requests/sec
ML-based:          200-800 requests/sec
```

### Memory Usage

**Base Runtime:**
```
Rust process:      5-10 MB
Per scanner:       1-5 MB (rule-based)
Per ML model:      200-500 MB (loaded)
```

**Python Comparison:**
```
Python process:    50-100 MB
Per scanner:       10-20 MB (rule-based)
Per ML model:      500-1000 MB (loaded)
```

### Benchmark Example

```rust
// benches/scanner_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use llm_guard_scanners::BanSubstringsScanner;

fn bench_ban_substrings(c: &mut Criterion) {
    let mut group = c.benchmark_group("ban_substrings");

    let scanner = BanSubstringsScanner::new(
        vec!["banned".to_string(), "forbidden".to_string()],
        0.5,
        false,
    ).unwrap();

    for size in [10, 100, 1000, 10000].iter() {
        let text = "a ".repeat(*size / 2) + "clean text";

        group.bench_with_input(
            BenchmarkId::new("clean_text", size),
            size,
            |b, _| {
                b.iter(|| scanner.scan_prompt(black_box(&text)));
            },
        );

        let text = "a ".repeat(*size / 2) + "banned text";

        group.bench_with_input(
            BenchmarkId::new("banned_text", size),
            size,
            |b, _| {
                b.iter(|| scanner.scan_prompt(black_box(&text)));
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_ban_substrings);
criterion_main!(benches);
```

**Running Benchmarks:**
```bash
cargo bench

# With flamegraph
cargo install flamegraph
cargo flamegraph --bench scanner_bench

# With perf
perf record cargo bench
perf report
```

---

## Memory Management

### Ownership Patterns

**Scanner Ownership:**
```rust
// ❌ BAD: Cloning scanners
fn process(scanner: BanSubstringsScanner, text: &str) {
    // Takes ownership, scanner dropped after function
}

// ✅ GOOD: Reference
fn process(scanner: &BanSubstringsScanner, text: &str) {
    // Borrows scanner
}

// ✅ GOOD: Arc for shared ownership
fn process(scanner: Arc<BanSubstringsScanner>, text: &str) {
    // Thread-safe shared ownership
}
```

**String Handling:**
```rust
// ❌ BAD: Unnecessary allocation
fn sanitize(text: &str) -> String {
    if is_clean(text) {
        text.to_string() // Unnecessary clone
    } else {
        redact(text)
    }
}

// ✅ GOOD: Cow (Clone on Write)
use std::borrow::Cow;

fn sanitize(text: &str) -> Cow<str> {
    if is_clean(text) {
        Cow::Borrowed(text) // No allocation
    } else {
        Cow::Owned(redact(text))
    }
}
```

### Arena Allocation

For scanners that process many short-lived objects:

```rust
use bumpalo::Bump;

pub struct ArenaScanner {
    arena: Bump,
    // ... other fields
}

impl ArenaScanner {
    pub fn scan(&mut self, text: &str) -> ScanResult {
        // Allocate temporary objects in arena
        let temp_data = self.arena.alloc_slice_fill_copy(1000, 0u8);

        // ... processing ...

        // Arena is cleared at end of scope or explicitly
        self.arena.reset();

        result
    }
}
```

### Memory Profiling

**Tools:**
- `valgrind --tool=massif`
- `heaptrack`
- `cargo-flamegraph`
- `dhat` (Rust DHAT)

**Example with DHAT:**
```rust
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();

    // Your code here

    // Profile output at end
}
```

---

## WASM Compatibility

### WASM-Specific Considerations

**1. No Threading**
```rust
// ❌ WON'T WORK in WASM
use std::thread;
thread::spawn(|| { /* ... */ });

// ✅ USE: Web Workers (via wasm-bindgen)
#[wasm_bindgen]
pub fn spawn_worker() -> Worker {
    // Use web_sys::Worker
}
```

**2. No Filesystem**
```rust
// ❌ WON'T WORK in WASM
use std::fs;
let content = fs::read_to_string("file.txt")?;

// ✅ USE: Fetch API
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;

pub async fn load_file(url: &str) -> Result<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(url)
    }

    #[cfg(target_arch = "wasm32")]
    {
        let resp = Request::get(url).send().await?;
        resp.text().await
    }
}
```

**3. Async Runtime**
```rust
// ❌ WON'T WORK: tokio in WASM
use tokio::runtime::Runtime;

// ✅ USE: wasm-bindgen-futures
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub async fn async_function() {
    // Async code here
}
```

**4. Random Numbers**
```rust
// Must use getrandom with "js" feature
[dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

### WASM Bundle Optimization

**Cargo.toml:**
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Single codegen unit
panic = "abort"     # Smaller panic handler
strip = true        # Strip symbols

[profile.release.package."*"]
opt-level = "z"
```

**Build script:**
```bash
#!/bin/bash

# Build with wasm-pack
wasm-pack build --target web --release

# Optimize with wasm-opt
wasm-opt -Oz \
  --enable-simd \
  --enable-bulk-memory \
  --enable-threads \
  pkg/llm_guard_bg.wasm \
  -o pkg/llm_guard_bg.wasm

# Compress with brotli
brotli -f pkg/llm_guard_bg.wasm

# Check size
ls -lh pkg/*.wasm*
```

**Size Targets:**
- Unoptimized: 10-20 MB
- Optimized: 2-5 MB
- Compressed (br): 500KB - 2MB

---

## Security Considerations

### Input Validation

**Always validate scanner inputs:**
```rust
pub fn scan_prompt(&self, prompt: &str) -> Result<ScanResult> {
    // Length check
    if prompt.len() > MAX_PROMPT_LENGTH {
        return Err(ScanError::InvalidInput(
            format!("Prompt too long: {} bytes", prompt.len())
        ));
    }

    // UTF-8 validation (automatic in Rust &str)
    // No null bytes
    if prompt.contains('\0') {
        return Err(ScanError::InvalidInput("Null byte in prompt".into()));
    }

    // Continue with scanning...
}
```

### Regex DoS Prevention

**Set limits on regex complexity:**
```rust
use regex::RegexBuilder;

let regex = RegexBuilder::new(pattern)
    .size_limit(10 * (1 << 20)) // 10 MB
    .dfa_size_limit(2 * (1 << 20)) // 2 MB
    .build()?;
```

**Timeout for regex matching:**
```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_millis(100),
    async { regex.is_match(text) }
).await?;
```

### Secret Handling

**Never log secrets:**
```rust
#[derive(Debug)]
pub struct DetectedEntity {
    pub entity_type: String,

    // Use custom Debug impl to redact
    #[debug(skip)]
    pub text: String,

    // ... other fields
}

impl std::fmt::Debug for DetectedEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("DetectedEntity")
            .field("entity_type", &self.entity_type)
            .field("text", &"[REDACTED]")
            .finish()
    }
}
```

### Dependency Auditing

**Regular security audits:**
```bash
# Install cargo-audit
cargo install cargo-audit

# Run audit
cargo audit

# Fix known vulnerabilities
cargo audit fix
```

**Use cargo-deny:**
```toml
# deny.toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
unsound = "deny"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]

[bans]
multiple-versions = "warn"
```

---

## Benchmarking Guide

### Setting Up Benchmarks

**1. Create benchmark file:**
```bash
mkdir -p benches
touch benches/scanner_bench.rs
```

**2. Add to Cargo.toml:**
```toml
[[bench]]
name = "scanner_bench"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

**3. Write benchmarks:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_function(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_group");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("input", size), size, |b, &size| {
            b.iter(|| {
                // Code to benchmark
                black_box(size * 2)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

**4. Run benchmarks:**
```bash
cargo bench

# Generate flamegraph
cargo bench --bench scanner_bench -- --profile-time=5

# Compare with baseline
cargo bench -- --save-baseline main
git checkout feature-branch
cargo bench -- --baseline main
```

### Interpreting Results

**Criterion Output:**
```
BanSubstrings/clean_text/10
                        time:   [450.23 ns 452.67 ns 455.89 ns]
                        change: [-2.3421% -1.8912% -1.4203%] (p = 0.00 < 0.05)
                        Performance has improved.

BanSubstrings/banned_text/10
                        time:   [789.45 ns 795.12 ns 802.34 ns]
                        change: [+0.3421% +1.2912% +2.1203%] (p = 0.03 < 0.05)
                        Performance has regressed.
```

**What to look for:**
- p-value < 0.05: statistically significant
- change < 5%: probably noise
- change > 10%: investigate

---

## Troubleshooting

### Common Issues

**1. ONNX Model Loading Fails**

**Error:**
```
Error: Model load failed: invalid ONNX file
```

**Solution:**
```rust
// Check model file exists
assert!(std::path::Path::new("model.onnx").exists());

// Verify ONNX version
// Use onnx Python package to check:
// python -c "import onnx; model = onnx.load('model.onnx'); print(model.opset_import)"

// Try with different optimization level
let session = SessionBuilder::new(&environment)?
    .with_optimization_level(GraphOptimizationLevel::Level1)? // Instead of Level3
    .with_model_from_file("model.onnx")?;
```

**2. WASM Build Fails**

**Error:**
```
error: linking with `rust-lld` failed
```

**Solution:**
```bash
# Update wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Clear cache
cargo clean

# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Check for incompatible dependencies
# (e.g., using tokio without wasm features)
```

**3. Performance Regression**

**Steps:**
1. Run benchmarks with baseline
2. Profile with flamegraph
3. Check for:
   - Unnecessary allocations
   - Regex recompilation
   - Blocking operations in async code
   - Lock contention

**4. Memory Leak**

**Tools:**
```bash
# Valgrind
valgrind --leak-check=full ./target/release/llm-guard-api

# Heaptrack
heaptrack ./target/release/llm-guard-api
heaptrack_gui heaptrack.llm-guard-api.*.gz
```

### Debug Build Performance

Debug builds are MUCH slower. Always benchmark release builds:

```bash
# ❌ WRONG
cargo bench

# ✅ CORRECT
cargo bench --release
```

### Logging Performance Impact

Logging can significantly impact performance:

```rust
// ❌ SLOW: Always formats
log::debug!("Processing: {}", expensive_function());

// ✅ FAST: Only formats if enabled
log::debug!("Processing: {}", {
    if log::log_enabled!(log::Level::Debug) {
        expensive_function()
    } else {
        ""
    }
});

// ✅ BEST: Use lazy evaluation
use tracing::debug;
debug!(value = ?expensive_function(), "Processing");
```

---

## Additional Resources

### Recommended Reading

**Rust Books:**
- The Rust Programming Language (https://doc.rust-lang.org/book/)
- Rust for Rustaceans (https://rust-for-rustaceans.com/)
- Zero To Production In Rust (https://www.zero2prod.com/)

**Performance:**
- The Rust Performance Book (https://nnethercote.github.io/perf-book/)
- Rust Async Book (https://rust-lang.github.io/async-book/)

**WASM:**
- Rust and WebAssembly (https://rustwasm.github.io/docs/book/)

### Tools

**Development:**
- rust-analyzer (LSP)
- cargo-watch (auto-rebuild)
- cargo-expand (macro expansion)

**Performance:**
- cargo-flamegraph
- perf / instruments
- criterion

**Testing:**
- cargo-tarpaulin (coverage)
- cargo-fuzz (fuzzing)
- miri (undefined behavior detection)

**Security:**
- cargo-audit
- cargo-deny
- cargo-geiger (unsafe code detection)

---

*Document Version: 1.0*
*Last Updated: 2025-01-30*
*Status: Technical Reference*
