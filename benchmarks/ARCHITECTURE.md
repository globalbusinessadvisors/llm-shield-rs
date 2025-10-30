# LLM Shield Benchmark Architecture

## Overview

This document describes the architecture of the LLM Shield performance benchmarking system designed to validate the 6 performance claims made in the README.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  Benchmark Infrastructure                        │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────┐      ┌──────────────────┐      ┌──────────────────┐
│   Test Data      │      │  Rust Benchmarks │      │ Python Baseline  │
│   Generation     │──────▶   (criterion)    │      │   (llm-guard)    │
└──────────────────┘      └────────┬─────────┘      └────────┬─────────┘
                                   │                          │
                                   ▼                          ▼
                          ┌──────────────────────────────────────┐
                          │       Result Collection              │
                          │  (CSV/JSON/Markdown)                 │
                          └────────┬─────────────────────────────┘
                                   │
                                   ▼
                          ┌──────────────────────────────────────┐
                          │    Analysis & Reporting              │
                          │  (Python/Pandas/Matplotlib)          │
                          └────────┬─────────────────────────────┘
                                   │
                                   ▼
                          ┌──────────────────────────────────────┐
                          │     Final Report + Charts            │
                          │  (benchmarks/results/)               │
                          └──────────────────────────────────────┘
```

## Component Breakdown

### 1. Test Data Generation (`benchmarks/data/`)

**Purpose:** Generate diverse, realistic test datasets for benchmarking

**Components:**
- `generate_test_data.py` - Main data generator
- `test_prompts.json` - 1000 diverse prompts (categorized)
- `secrets_dataset.json` - Prompts with various secret types
- `code_samples.json` - Code snippets for scanning
- `toxic_samples.json` - Harmful content for safety testing

**Test Data Distribution:**
```
Total: 1000 prompts
├── 200 simple (10-50 words)
├── 200 medium (50-200 words)
├── 200 long (200-500 words)
├── 100 with secrets (API keys, tokens, passwords)
├── 100 with code snippets
├── 100 with prompt injection attempts
└── 100 toxic/harmful content
```

### 2. Rust Benchmark Suite (`crates/llm-shield-benches/`)

**Purpose:** Criterion-based micro and macro benchmarks for Rust implementation

**Structure:**
```
crates/llm-shield-benches/
├── Cargo.toml
├── benches/
│   ├── latency.rs           # Scenario 1A-1D (4 tests)
│   ├── throughput.rs         # Scenario 2A-2B (2 tests)
│   ├── memory.rs             # Scenario 3A-3C (3 tests)
│   ├── cold_start.rs         # Scenario 4A-4C (3 tests)
│   ├── binary_size.rs        # Scenario 5A-5C (3 tests)
│   └── cpu_usage.rs          # Scenario 6A-6C (3 tests)
├── src/
│   ├── lib.rs                # Common benchmark utilities
│   ├── metrics.rs            # Metrics collection (p50, p95, p99)
│   ├── fixtures.rs           # Test data loading
│   └── comparison.rs         # Python vs Rust comparison helpers
└── tests/
    └── benchmark_runner_test.rs  # TDD: Test the benchmark framework
```

**Key Features:**
- Criterion for statistical analysis
- Configurable iterations (1000 for latency, 100 for ML)
- JSON/CSV output for post-processing
- Automatic p50/p95/p99 calculation
- Memory profiling with jemalloc

### 3. Python Baseline (`benchmarks/python/`)

**Purpose:** Reference implementations using llm-guard for comparison

**Structure:**
```
benchmarks/python/
├── requirements.txt          # llm-guard, fastapi, uvicorn
├── bench_latency.py          # Equivalent to Rust latency tests
├── bench_throughput.py       # FastAPI server for load testing
├── bench_memory.py           # Memory profiling
├── bench_cold_start.py       # Startup time measurement
├── bench_cpu.py              # CPU profiling
└── utils.py                  # Shared utilities
```

### 4. Automation Scripts (`benchmarks/scripts/`)

**Purpose:** Shell scripts to orchestrate benchmarks and collect system metrics

**Scripts:**
```bash
benchmarks/scripts/
├── run_all_benchmarks.sh     # Master runner
├── bench_latency.sh          # Run latency benchmarks
├── bench_throughput.sh       # Run throughput (wrk)
├── bench_memory.sh           # Monitor RSS/heap (pidstat)
├── bench_cold_start.sh       # Measure startup time (hyperfine)
├── bench_binary_size.sh      # Measure binary/docker sizes
├── bench_cpu.sh              # CPU profiling (perf, flamegraph)
├── setup_environment.sh      # Install dependencies
└── cleanup.sh                # Clean up artifacts
```

### 5. Analysis & Reporting (`benchmarks/analysis/`)

**Purpose:** Process raw benchmark data and generate reports

**Structure:**
```
benchmarks/analysis/
├── analyze_results.py        # Main analysis script
├── generate_charts.py        # Matplotlib visualizations
├── compare_performance.py    # Python vs Rust comparison
├── validate_claims.py        # Check against README claims
└── templates/
    ├── report_template.md    # Markdown template for results
    └── executive_summary.md  # High-level summary template
```

## Benchmark Categories

### Category 1: Latency Benchmarks

**File:** `crates/llm-shield-benches/benches/latency.rs`

**Tests:**
1. **Scenario 1A:** BanSubstrings (simple string matching)
   - Input: "This is a test prompt with some content"
   - Patterns: 3 banned substrings
   - Iterations: 1000
   - Metric: p50/p95/p99 latency

2. **Scenario 1B:** Regex (10 custom patterns)
   - Input: Medium text (100 words)
   - Patterns: 10 regex patterns
   - Iterations: 1000
   - Metric: p50/p95/p99 latency

3. **Scenario 1C:** Secrets (40+ secret patterns)
   - Input: Text with embedded API key
   - Patterns: 40+ secret types
   - Iterations: 1000
   - Metric: p50/p95/p99 latency

4. **Scenario 1D:** PromptInjection (ML model)
   - Input: Potential injection attempt
   - Model: ONNX transformer
   - Iterations: 100 (ML is slower)
   - Metric: p50/p95/p99 latency

**Expected Results:**
```
Python: 5-500ms (varies by test)
Rust:   <1-20ms (varies by test)
Overall: 10-25x improvement
```

### Category 2: Throughput Benchmarks

**File:** `crates/llm-shield-benches/benches/throughput.rs`
**Tool:** `wrk` HTTP load tester

**Tests:**
1. **Scenario 2A:** Single scanner, concurrent requests
   - Tool: wrk -t4 -c100 -d60s
   - Scanner: BanSubstrings
   - Metric: requests/second

2. **Scenario 2B:** Scanner pipeline (3 scanners)
   - Scanners: BanSubstrings → Secrets → PromptInjection
   - Metric: requests/second

**Expected Results:**
```
Python: 100-500 req/sec
Rust:   10,000-50,000 req/sec
Improvement: 50-100x
```

### Category 3: Memory Benchmarks

**File:** `crates/llm-shield-benches/benches/memory.rs`
**Tool:** `pidstat`, `pmap`, `valgrind`

**Tests:**
1. **Scenario 3A:** Baseline (idle server)
2. **Scenario 3B:** Under load (1000 req/sec)
3. **Scenario 3C:** Memory growth (1 hour test)

**Expected Results:**
```
Python Baseline: 1-2GB
Rust Baseline:   50-100MB
Improvement: 10-20x
```

### Category 4: Cold Start Benchmarks

**File:** `crates/llm-shield-benches/benches/cold_start.rs`
**Tool:** `hyperfine`

**Tests:**
1. **Scenario 4A:** Application startup time
2. **Scenario 4B:** First request latency
3. **Scenario 4C:** Serverless cold start (Lambda/Workers)

**Expected Results:**
```
Python: 10-30s
Rust:   <1s
WASM:   <100ms
Improvement: 10-300x
```

### Category 5: Binary Size Benchmarks

**Script:** `benchmarks/scripts/bench_binary_size.sh`

**Tests:**
1. **Scenario 5A:** Docker image size
2. **Scenario 5B:** Native binary (stripped + UPX)
3. **Scenario 5C:** WASM bundle (optimized + gzip)

**Expected Results:**
```
Python Docker: 3-5GB
Rust Docker:   <100MB
Rust Binary:   10-50MB
WASM gzip:     <2MB
Improvement: 60-5000x
```

### Category 6: CPU Usage Benchmarks

**File:** `crates/llm-shield-benches/benches/cpu_usage.rs`
**Tool:** `pidstat`, `perf`, `flamegraph`

**Tests:**
1. **Scenario 6A:** Single request CPU time
2. **Scenario 6B:** CPU % under sustained load
3. **Scenario 6C:** CPU efficiency (req/sec per core)

**Expected Results:**
```
Python: 350-400% CPU (4 workers, GIL)
Rust:   100-200% CPU (efficient multi-threading)
Efficiency: 100x more req/sec per CPU core
```

## Testing Strategy (London School TDD)

### Test-First Approach

**Step 1: Write tests for benchmark infrastructure**
```rust
// crates/llm-shield-benches/tests/benchmark_runner_test.rs

#[tokio::test]
async fn test_benchmark_runner_collects_latency_metrics() {
    let runner = BenchmarkRunner::new();
    let result = runner.run_latency_benchmark("BanSubstrings", 1000).await;

    assert!(result.p50_ms < 20.0);
    assert!(result.p95_ms < 50.0);
    assert!(result.p99_ms < 100.0);
}

#[test]
fn test_test_data_generator_creates_diverse_prompts() {
    let prompts = generate_test_prompts(1000);

    assert_eq!(prompts.len(), 1000);
    assert_eq!(prompts.iter().filter(|p| p.category == "simple").count(), 200);
    assert_eq!(prompts.iter().filter(|p| p.category == "secrets").count(), 100);
}

#[test]
fn test_metrics_calculator_computes_percentiles() {
    let latencies = vec![1.0, 2.0, 3.0, ..., 1000.0];
    let metrics = compute_metrics(&latencies);

    assert_approx_eq!(metrics.p50, 500.0);
    assert_approx_eq!(metrics.p95, 950.0);
    assert_approx_eq!(metrics.p99, 990.0);
}
```

**Step 2: Implement infrastructure to pass tests**

**Step 3: Write tests for each benchmark scenario**

**Step 4: Implement benchmarks**

**Step 5: Validate against expected results**

## Data Flow

### 1. Benchmark Execution Flow

```
1. Load test data from benchmarks/data/
2. Initialize scanner (Rust or Python)
3. Run benchmark iterations
4. Collect raw measurements
5. Calculate statistics (p50, p95, p99)
6. Write results to benchmarks/results/{test}_results.csv
7. Generate charts to benchmarks/charts/{test}_comparison.png
8. Aggregate into summary report
```

### 2. Result Collection Format

**CSV Format:**
```csv
test_name,language,iteration,latency_ms,memory_kb,cpu_percent
ban_substrings,rust,1,0.85,1024,2.5
ban_substrings,rust,2,0.92,1024,2.7
ban_substrings,python,1,12.4,51200,15.2
...
```

**Summary JSON:**
```json
{
  "test": "latency_ban_substrings",
  "timestamp": "2025-10-30T12:00:00Z",
  "rust": {
    "p50_ms": 0.87,
    "p95_ms": 1.25,
    "p99_ms": 2.10,
    "mean_ms": 0.95
  },
  "python": {
    "p50_ms": 12.1,
    "p95_ms": 18.5,
    "p99_ms": 25.3,
    "mean_ms": 13.8
  },
  "improvement": "14.5x faster",
  "claim_validated": true
}
```

## Dependencies

### Rust
```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jemalloc-ctl = "0.5"  # Memory profiling
```

### Python
```
llm-guard>=0.3.0
fastapi>=0.104.0
uvicorn>=0.24.0
matplotlib>=3.7.0
pandas>=2.0.0
numpy>=1.24.0
```

### System Tools
- `wrk` - HTTP benchmarking
- `hyperfine` - Command-line benchmarking
- `pidstat` - Process monitoring
- `perf` - Linux profiling
- `valgrind` - Memory analysis
- `flamegraph` - CPU profiling visualization

## Validation Criteria

Each benchmark must:
1. ✅ Run successfully on both Python and Rust
2. ✅ Collect valid measurements (no errors)
3. ✅ Generate statistical summaries (p50, p95, p99)
4. ✅ Compare against expected results
5. ✅ Validate README claims (pass/fail)
6. ✅ Generate charts and reports
7. ✅ Be reproducible (documented environment)

## Output Structure

```
benchmarks/
├── ARCHITECTURE.md           # This file
├── README.md                 # How to run benchmarks
├── METHODOLOGY.md            # Detailed methodology
├── data/
│   ├── test_prompts.json
│   ├── secrets_dataset.json
│   └── code_samples.json
├── scripts/
│   ├── run_all_benchmarks.sh
│   ├── bench_latency.sh
│   ├── bench_throughput.sh
│   ├── bench_memory.sh
│   ├── bench_cold_start.sh
│   ├── bench_binary_size.sh
│   └── bench_cpu.sh
├── python/
│   ├── requirements.txt
│   ├── bench_latency.py
│   └── ...
├── analysis/
│   ├── analyze_results.py
│   └── generate_charts.py
├── results/
│   ├── latency_results.csv
│   ├── throughput_results.csv
│   ├── memory_results.csv
│   ├── cold_start_results.csv
│   ├── binary_size_results.csv
│   ├── cpu_results.csv
│   └── RESULTS.md             # Final summary
└── charts/
    ├── latency_comparison.png
    ├── throughput_comparison.png
    ├── memory_usage.png
    ├── cold_start_distribution.png
    └── cpu_efficiency.png
```

## Next Steps

1. Create `crates/llm-shield-benches/` crate
2. Implement test data generator
3. Write benchmark infrastructure tests (TDD)
4. Implement benchmark runners
5. Create automation scripts
6. Execute benchmarks
7. Generate reports
8. Validate claims

## Success Criteria

**All 6 categories must validate:**
- ✅ Latency: Rust <20ms (10-25x faster)
- ✅ Throughput: Rust >10,000 req/sec (100x higher)
- ✅ Memory: Rust <500MB (8-16x lower)
- ✅ Cold Start: Rust <1s (10-30x faster)
- ✅ Binary Size: WASM <2MB (60-100x smaller)
- ✅ CPU: 5-10x more efficient

**If any fail:** Document actual results and update README claims transparently.
