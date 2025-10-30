# LLM Shield Performance Benchmark & Validation Plan

## 🎯 Objective

Validate and measure the performance claims made in the README regarding LLM Shield (Rust/WASM) vs Python llm-guard:

| Metric | Python llm-guard | **LLM Shield (Rust)** | **Claimed Improvement** |
|--------|------------------|----------------------|------------------------|
| **Latency** | 200-500ms | <20ms | **10-25x faster** ⚡ |
| **Throughput** | 100 req/sec | 10,000+ req/sec | **100x higher** 📈 |
| **Memory** | 4-8GB | <500MB | **8-16x lower** 💾 |
| **Cold Start** | 10-30s | <1s | **10-30x faster** 🚀 |
| **Binary Size** | 3-5GB (Docker) | <50MB (native) / <2MB (WASM gzip) | **60-100x smaller** 📦 |
| **CPU Usage** | High (Python GIL) | Low (parallel Rust) | **5-10x lower** ⚙️ |

---

## 📋 Benchmark Categories

### 1. Latency Benchmark
### 2. Throughput Benchmark
### 3. Memory Usage Benchmark
### 4. Cold Start Benchmark
### 5. Binary Size Benchmark
### 6. CPU Usage Benchmark

---

## 🔬 1. Latency Benchmark

### Goal
Measure end-to-end latency for scanning operations (single request)

### Test Scenarios

#### Scenario 1A: Simple String Matching (BanSubstrings)
**Python llm-guard:**
```python
from llm_guard.input_scanners import BanSubstrings
import time

scanner = BanSubstrings(substrings=["banned1", "banned2", "banned3"])
prompt = "This is a test prompt with some content"

start = time.perf_counter()
result = scanner.scan(prompt)
end = time.perf_counter()
latency_ms = (end - start) * 1000
```

**Rust llm-shield:**
```rust
use llm_shield_scanners::input::BanSubstrings;
use std::time::Instant;

let config = BanSubstringsConfig {
    substrings: vec!["banned1", "banned2", "banned3"],
    ..Default::default()
};
let scanner = BanSubstrings::new(config)?;
let prompt = "This is a test prompt with some content";

let start = Instant::now();
let result = scanner.scan(prompt, &vault).await?;
let latency_ms = start.elapsed().as_millis();
```

**Expected Results:**
- Python: 5-15ms (simple regex)
- Rust: <1ms (Aho-Corasick)
- **Improvement: 10-15x**

#### Scenario 1B: Regex Scanning
**Test:** Custom regex pattern matching (10 patterns)

**Expected Results:**
- Python: 10-30ms
- Rust: 1-3ms
- **Improvement: 10x**

#### Scenario 1C: Secret Detection (40+ patterns)
**Test:** Full secret scanning with entropy validation

**Expected Results:**
- Python: 50-100ms (detect-secrets library)
- Rust: 5-10ms (compiled regex + entropy)
- **Improvement: 10x**

#### Scenario 1D: ML-Based Scanning (PromptInjection)
**Test:** Transformer model inference (ONNX vs HuggingFace)

**Expected Results:**
- Python: 200-500ms (transformers library, CPU)
- Rust: 50-150ms (ONNX Runtime, optimized)
- **Improvement: 3-5x for ML, 10x for heuristic fallback**

### Tools
- Python: `timeit`, `time.perf_counter()`
- Rust: `std::time::Instant`, `criterion` crate for micro-benchmarks
- Statistical analysis: Measure p50, p95, p99 over 1000 runs

### Environment
- AWS EC2 c5.xlarge (4 vCPU, 8GB RAM)
- Ubuntu 22.04 LTS
- Python 3.11 (no GIL if available, otherwise 3.10)
- Rust 1.75+ (release build with optimizations)

### Validation Criteria
✅ **PASS:** Rust achieves <20ms for mixed workload (average of all scenarios)
✅ **PASS:** Rust is 10-25x faster than Python for non-ML workloads
✅ **PASS:** Rust is 3-5x faster than Python for ML workloads

---

## 🚀 2. Throughput Benchmark

### Goal
Measure requests per second under sustained load

### Test Scenarios

#### Scenario 2A: Single Scanner, Concurrent Requests
**Test Setup:**
- Tool: `wrk` or `hey` HTTP load tester
- Duration: 60 seconds
- Concurrency: 10, 50, 100, 500 concurrent connections

**Python llm-guard (FastAPI):**
```python
# app.py
from fastapi import FastAPI
from llm_guard.input_scanners import BanSubstrings

app = FastAPI()
scanner = BanSubstrings(substrings=["test"])

@app.post("/scan")
async def scan(prompt: str):
    result = scanner.scan(prompt)
    return {"is_valid": result[1]}

# Run: uvicorn app:app --workers 4
```

**Rust llm-shield (Axum):**
```rust
use axum::{Router, Json};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/scan", post(scan_handler));

    // Run with tokio multi-threaded runtime
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

**Load Test Command:**
```bash
# Python
wrk -t4 -c100 -d60s --latency http://localhost:8000/scan

# Rust
wrk -t4 -c100 -d60s --latency http://localhost:3000/scan
```

**Expected Results:**
- Python (4 workers): 100-500 req/sec
- Rust (tokio): 10,000-50,000 req/sec
- **Improvement: 20-100x**

#### Scenario 2B: Scanner Pipeline (3 scanners in sequence)
**Test:** BanSubstrings → Secrets → PromptInjection

**Expected Results:**
- Python: 50-100 req/sec
- Rust: 5,000-10,000 req/sec
- **Improvement: 50-100x**

### Tools
- `wrk` - HTTP benchmarking tool
- `hey` - Alternative HTTP load generator
- `k6` - For complex scenarios
- Custom Rust benchmark using `tokio` + `criterion`

### Metrics to Collect
- Requests/second (mean)
- Latency distribution (p50, p95, p99)
- Error rate
- Connection errors
- Timeout rate

### Validation Criteria
✅ **PASS:** Rust achieves >10,000 req/sec for simple scanners
✅ **PASS:** Rust achieves >1,000 req/sec for ML scanners
✅ **PASS:** Rust is 50-100x higher throughput than Python

---

## 💾 3. Memory Usage Benchmark

### Goal
Measure resident memory (RSS) during operation

### Test Scenarios

#### Scenario 3A: Baseline Memory (Idle)
**Measurement:** Memory usage with server loaded but no requests

**Python:**
```bash
# Start FastAPI server
uvicorn app:app --workers 4 &
PID=$!
sleep 5
ps -p $PID -o rss=  # RSS in KB
pmap -x $PID | tail -1  # Detailed breakdown
```

**Rust:**
```bash
# Start Axum server
./target/release/llm-shield &
PID=$!
sleep 5
ps -p $PID -o rss=
```

**Expected Results:**
- Python (4 workers): 1-2GB baseline (transformers models loaded)
- Rust: 50-100MB baseline
- **Improvement: 10-20x lower**

#### Scenario 3B: Under Load (1000 req/sec)
**Measurement:** Peak memory during sustained load

**Expected Results:**
- Python: 4-8GB (GC pressure, model copies per worker)
- Rust: 200-500MB (single model, shared memory)
- **Improvement: 8-16x lower**

#### Scenario 3C: Memory Growth Over Time
**Test:** Run for 1 hour, measure memory growth

**Python:**
```bash
# Monitor every 10 seconds for 1 hour
for i in {1..360}; do
    ps -p $PID -o rss= >> python_memory.log
    sleep 10
done
```

**Expected Results:**
- Python: Memory grows 10-20% due to fragmentation
- Rust: Memory stays stable (no GC)
- **Validation: Rust has predictable memory usage**

### Tools
- `ps -o rss` - Resident Set Size
- `pmap` - Detailed memory map
- `valgrind --tool=massif` - Memory profiling
- `heaptrack` - Heap memory tracking
- Custom instrumentation: `jemalloc` stats for Rust

### Validation Criteria
✅ **PASS:** Rust baseline memory <500MB
✅ **PASS:** Rust under load <1GB
✅ **PASS:** Rust is 8-16x lower memory than Python
✅ **PASS:** Rust memory remains stable over time (no growth)

---

## ⚡ 4. Cold Start Benchmark

### Goal
Measure time from process start to first successful request

### Test Scenarios

#### Scenario 4A: Application Startup Time
**Python llm-guard:**
```bash
time python -c "
from llm_guard.input_scanners import PromptInjection, Secrets, Toxicity
# Models are loaded here (HuggingFace transformers)
print('Ready')
"
```

**Rust llm-shield:**
```bash
time cargo run --release -- -c "
use llm_shield_scanners::input::{PromptInjection, Secrets, Toxicity};
println!(\"Ready\");
"
```

**Expected Results:**
- Python: 10-30s (model downloads + initialization)
- Rust: <1s (compiled binary, lazy model loading)
- **Improvement: 10-30x faster**

#### Scenario 4B: First Request Latency
**Test:** Time from server start to first successful scan

**Python:**
```bash
#!/bin/bash
start=$(date +%s%3N)
uvicorn app:app --workers 1 &
PID=$!
while ! curl -s http://localhost:8000/scan; do sleep 0.1; done
end=$(date +%s%3N)
cold_start=$((end - start))
kill $PID
echo "Cold start: ${cold_start}ms"
```

**Expected Results:**
- Python: 5-15s (transformers model loading)
- Rust: 100-500ms (ONNX model loading, if needed)
- **Improvement: 10-50x faster**

#### Scenario 4C: Serverless Cold Start (AWS Lambda)
**Test:** End-to-end cold start in Lambda environment

**Python Lambda:**
- Image size: 3-5GB (base + dependencies + models)
- Cold start: 10-20s

**Rust Lambda:**
- Image size: <100MB (static binary)
- Cold start: 500ms-1s

**WASM (Cloudflare Workers):**
- Bundle size: <2MB gzip
- Cold start: <10ms

**Expected Results:**
- **Improvement: 10-40x faster cold starts**

### Tools
- `time` command
- `hyperfine` - Benchmarking tool
- AWS Lambda performance insights
- Cloudflare Workers analytics

### Validation Criteria
✅ **PASS:** Rust application starts in <1s
✅ **PASS:** Rust first request in <500ms
✅ **PASS:** WASM cold start <100ms
✅ **PASS:** 10-30x faster than Python

---

## 📦 5. Binary Size Benchmark

### Goal
Measure deployment artifact sizes

### Test Scenarios

#### Scenario 5A: Native Binary Size
**Python (Docker):**
```dockerfile
FROM python:3.11-slim
RUN pip install llm-guard fastapi uvicorn
# Result: 3-5GB image (base + deps + models)
```

```bash
docker images python-llm-guard
# REPOSITORY           SIZE
# python-llm-guard     4.2GB
```

**Rust (Docker):**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/llm-shield /usr/local/bin/
# Result: <100MB image
```

```bash
docker images rust-llm-shield
# REPOSITORY           SIZE
# rust-llm-shield      87MB
```

**Expected Results:**
- Python: 3-5GB
- Rust: <100MB
- **Improvement: 30-50x smaller**

#### Scenario 5B: Native Binary (Stripped)
```bash
# Rust
cargo build --release
strip target/release/llm-shield
ls -lh target/release/llm-shield
# Expected: 30-50MB

# With UPX compression
upx --best target/release/llm-shield
ls -lh target/release/llm-shield
# Expected: 10-20MB
```

**Expected Results:**
- Rust stripped: 30-50MB
- Rust UPX: 10-20MB
- **50-100x smaller than Python Docker**

#### Scenario 5C: WASM Binary Size
```bash
cd crates/llm-shield-wasm
wasm-pack build --release --target web

# Measure sizes
ls -lh pkg/llm_shield_wasm_bg.wasm
# Expected: 5-8MB uncompressed

# With wasm-opt
wasm-opt -Oz pkg/llm_shield_wasm_bg.wasm -o pkg/optimized.wasm
ls -lh pkg/optimized.wasm
# Expected: 3-5MB

# Gzipped
gzip -k pkg/optimized.wasm
ls -lh pkg/optimized.wasm.gz
# Expected: 1-2MB
```

**Expected Results:**
- WASM uncompressed: 5-8MB
- WASM optimized: 3-5MB
- WASM gzipped: **1-2MB**
- **1500-5000x smaller than Python Docker!**

### Validation Criteria
✅ **PASS:** Rust Docker image <100MB
✅ **PASS:** Rust stripped binary <50MB
✅ **PASS:** WASM gzipped <2MB
✅ **PASS:** 60-100x smaller than Python deployment

---

## ⚙️ 6. CPU Usage Benchmark

### Goal
Measure CPU utilization under load

### Test Scenarios

#### Scenario 6A: Single Request CPU Time
**Measurement:** CPU time per request

**Python:**
```python
import cProfile
import pstats

profiler = cProfile.Profile()
profiler.enable()
result = scanner.scan(prompt)
profiler.disable()

stats = pstats.Stats(profiler)
stats.print_stats()
# Note total CPU time
```

**Rust:**
```rust
use std::time::Instant;

let start = Instant::now();
let result = scanner.scan(prompt, &vault).await?;
let cpu_time = start.elapsed();
```

**Expected Results:**
- Python: 50-200ms CPU time (GIL locks, interpreted)
- Rust: 5-20ms CPU time (compiled, no GIL)
- **Improvement: 10x lower**

#### Scenario 6B: CPU Usage Under Sustained Load
**Test:** Monitor CPU % during throughput test

**Commands:**
```bash
# Python (4 workers)
pidstat -p $(pgrep -f uvicorn) 1 60 > python_cpu.log

# Rust
pidstat -p $(pgrep llm-shield) 1 60 > rust_cpu.log
```

**Expected Results:**
- Python: 350-400% CPU (4 workers, GIL contention)
- Rust: 100-200% CPU (efficient multi-threading)
- **Improvement: 2-4x more efficient**

#### Scenario 6C: CPU Efficiency (Requests per CPU second)
**Metric:** Throughput / CPU usage

**Expected Results:**
- Python: 25-100 req/sec per CPU core
- Rust: 2,500-10,000 req/sec per CPU core
- **Improvement: 100x higher efficiency**

### Tools
- `pidstat` - Process CPU statistics
- `perf` - Linux performance analysis
- `flamegraph` - CPU profiling visualization
- `py-spy` - Python sampling profiler
- `cargo flamegraph` - Rust profiling

### Validation Criteria
✅ **PASS:** Rust uses <50% CPU at 1000 req/sec
✅ **PASS:** Rust CPU efficiency 50-100x higher
✅ **PASS:** Rust scales linearly with cores (no GIL)

---

## 🧪 Comprehensive Test Plan

### Phase 1: Setup (Week 1)
**Tasks:**
- [ ] Set up AWS EC2 c5.xlarge instance
- [ ] Install Python 3.11 + llm-guard
- [ ] Build Rust llm-shield (release mode)
- [ ] Install benchmarking tools (wrk, hyperfine, pidstat)
- [ ] Create test datasets (1000 diverse prompts)
- [ ] Set up monitoring (Prometheus + Grafana)

### Phase 2: Latency Testing (Week 1-2)
**Tasks:**
- [ ] Run Scenario 1A: String matching (1000 iterations)
- [ ] Run Scenario 1B: Regex scanning (1000 iterations)
- [ ] Run Scenario 1C: Secret detection (1000 iterations)
- [ ] Run Scenario 1D: ML inference (100 iterations)
- [ ] Collect p50, p95, p99 latencies
- [ ] Generate comparison charts

### Phase 3: Throughput Testing (Week 2)
**Tasks:**
- [ ] Deploy Python FastAPI server (4 workers)
- [ ] Deploy Rust Axum server
- [ ] Run wrk benchmarks (10, 50, 100, 500 concurrent)
- [ ] Run pipeline benchmarks (3 scanners)
- [ ] Measure error rates and timeouts
- [ ] Document maximum sustainable throughput

### Phase 4: Memory Testing (Week 3)
**Tasks:**
- [ ] Measure baseline memory (idle)
- [ ] Measure memory under load (1000 req/sec)
- [ ] Run 1-hour stability test
- [ ] Use valgrind/heaptrack for profiling
- [ ] Analyze memory fragmentation
- [ ] Generate memory usage charts

### Phase 5: Cold Start Testing (Week 3)
**Tasks:**
- [ ] Measure application startup time (100 runs)
- [ ] Measure first request latency (100 runs)
- [ ] Deploy to AWS Lambda (both Python and Rust)
- [ ] Deploy WASM to Cloudflare Workers
- [ ] Measure serverless cold starts (50 runs each)
- [ ] Document cold start distributions

### Phase 6: Binary Size Testing (Week 4)
**Tasks:**
- [ ] Build Docker images (Python and Rust)
- [ ] Measure Docker image sizes
- [ ] Strip and compress Rust binary (UPX)
- [ ] Build and optimize WASM bundle
- [ ] Measure gzipped sizes
- [ ] Document size breakdowns

### Phase 7: CPU Usage Testing (Week 4)
**Tasks:**
- [ ] Profile single request CPU time
- [ ] Monitor CPU during sustained load (1 hour)
- [ ] Generate flame graphs (Python and Rust)
- [ ] Calculate CPU efficiency metrics
- [ ] Test scaling across cores (1, 2, 4, 8 cores)
- [ ] Document CPU utilization patterns

### Phase 8: Analysis & Reporting (Week 5)
**Tasks:**
- [ ] Aggregate all benchmark results
- [ ] Generate comparison charts/graphs
- [ ] Write executive summary
- [ ] Document methodology
- [ ] Create reproducible test scripts
- [ ] Publish results in ./benchmarks/results/

---

## 📊 Test Data

### Dataset Composition
**1000 test prompts covering:**
- 20% simple text (10-50 words)
- 20% medium text (50-200 words)
- 20% long text (200-500 words)
- 10% with secrets (API keys, tokens)
- 10% with code snippets
- 10% with prompt injection attempts
- 10% toxic/harmful content

### Generating Test Data
```python
# generate_test_data.py
import random
import json

prompts = []

# Simple prompts
for i in range(200):
    prompts.append({
        "id": f"simple_{i}",
        "text": f"This is a simple test prompt number {i}",
        "category": "simple"
    })

# Medium prompts (with potential issues)
for i in range(200):
    prompts.append({
        "id": f"medium_{i}",
        "text": generate_medium_prompt(),
        "category": "medium"
    })

# ... etc

with open("test_prompts.json", "w") as f:
    json.dump(prompts, f, indent=2)
```

---

## 🎯 Success Criteria Summary

### Must Achieve (Critical)
- ✅ Latency: Rust <20ms average (10-25x faster)
- ✅ Throughput: Rust >10,000 req/sec (100x higher)
- ✅ Memory: Rust <500MB under load (8-16x lower)
- ✅ Cold Start: Rust <1s (10-30x faster)
- ✅ Binary Size: WASM <2MB gzip (60-100x smaller)
- ✅ CPU Usage: Rust 5-10x more efficient

### Nice to Have (Stretch)
- ⭐ Latency: Rust <10ms for non-ML
- ⭐ Throughput: Rust >50,000 req/sec
- ⭐ Memory: Rust <200MB baseline
- ⭐ Cold Start: WASM <10ms
- ⭐ Binary Size: Native <20MB (UPX)

---

## 📝 Reporting Format

### Results Document Structure
```markdown
# LLM Shield Performance Benchmark Results

## Executive Summary
- Date: YYYY-MM-DD
- Environment: AWS c5.xlarge
- Python Version: 3.11
- Rust Version: 1.75
- Overall Result: PASS/FAIL

## Detailed Results

### 1. Latency
| Scenario | Python (ms) | Rust (ms) | Improvement | Status |
|----------|-------------|-----------|-------------|--------|
| String Matching | X | Y | Zx | ✅/❌ |
| ... | ... | ... | ... | ... |

### 2. Throughput
...

### Charts
[Include PNG/SVG charts]

### Raw Data
[Link to CSV files]
```

---

## 🔧 Tools & Scripts

### Benchmark Runner Script
```bash
#!/bin/bash
# run_benchmarks.sh

set -e

echo "Starting LLM Shield Benchmarks..."

# 1. Latency
echo "Running latency benchmarks..."
./scripts/bench_latency.sh

# 2. Throughput
echo "Running throughput benchmarks..."
./scripts/bench_throughput.sh

# 3. Memory
echo "Running memory benchmarks..."
./scripts/bench_memory.sh

# 4. Cold start
echo "Running cold start benchmarks..."
./scripts/bench_cold_start.sh

# 5. Binary size
echo "Measuring binary sizes..."
./scripts/bench_binary_size.sh

# 6. CPU usage
echo "Running CPU benchmarks..."
./scripts/bench_cpu.sh

echo "All benchmarks complete!"
echo "Results saved to ./benchmarks/results/"
```

### Result Analysis Script
```python
# analyze_results.py
import pandas as pd
import matplotlib.pyplot as plt

def analyze_latency(python_data, rust_data):
    """Compare latency distributions"""
    df = pd.DataFrame({
        'Python': python_data,
        'Rust': rust_data
    })

    improvement = python_data.mean() / rust_data.mean()

    # Generate comparison chart
    df.boxplot()
    plt.savefig('latency_comparison.png')

    return improvement

# ... similar for other metrics
```

---

## 📅 Timeline

### Week 1: Setup & Latency
- Days 1-2: Environment setup
- Days 3-5: Latency benchmarks

### Week 2: Throughput
- Days 1-3: Simple throughput tests
- Days 4-5: Pipeline throughput tests

### Week 3: Memory & Cold Start
- Days 1-3: Memory benchmarks
- Days 4-5: Cold start tests

### Week 4: Size & CPU
- Days 1-2: Binary size measurements
- Days 3-5: CPU profiling

### Week 5: Analysis & Reporting
- Days 1-3: Data analysis
- Days 4-5: Report writing & documentation

**Total Duration: 5 weeks (part-time) or 2-3 weeks (full-time)**

---

## 🚨 Risk Mitigation

### Potential Issues & Solutions

**Issue 1: Python llm-guard not representative**
- **Solution:** Test against latest version (0.3.x)
- **Solution:** Use recommended production configuration
- **Solution:** Include both CPU and GPU tests

**Issue 2: Unfair comparison (different hardware)**
- **Solution:** Run both on identical EC2 instances
- **Solution:** Document all environment variables
- **Solution:** Run tests multiple times, use median values

**Issue 3: Results don't match claims**
- **Solution:** Identify bottlenecks using profiling
- **Solution:** Optimize Rust implementation
- **Solution:** Update README with actual results
- **Solution:** Be transparent about limitations

---

## 📚 Deliverables

1. **Benchmark Scripts** (`./benchmarks/scripts/`)
   - `bench_latency.sh`
   - `bench_throughput.sh`
   - `bench_memory.sh`
   - `bench_cold_start.sh`
   - `bench_binary_size.sh`
   - `bench_cpu.sh`

2. **Test Data** (`./benchmarks/data/`)
   - `test_prompts.json` (1000 prompts)
   - `secrets_dataset.json`
   - `code_samples.json`

3. **Results** (`./benchmarks/results/`)
   - `latency_results.csv`
   - `throughput_results.csv`
   - `memory_results.csv`
   - `cold_start_results.csv`
   - `binary_size_results.csv`
   - `cpu_results.csv`
   - `summary_report.md`

4. **Charts** (`./benchmarks/charts/`)
   - `latency_comparison.png`
   - `throughput_comparison.png`
   - `memory_usage.png`
   - `cold_start_distribution.png`
   - `cpu_efficiency.png`

5. **Documentation** (`./benchmarks/`)
   - `README.md` - How to reproduce
   - `METHODOLOGY.md` - Detailed methodology
   - `RESULTS.md` - Final results summary

---

## ✅ Checklist

- [ ] Test environment provisioned (AWS EC2)
- [ ] Python llm-guard installed and tested
- [ ] Rust llm-shield built (release mode)
- [ ] Test dataset generated (1000 prompts)
- [ ] Benchmarking tools installed
- [ ] Latency benchmarks complete
- [ ] Throughput benchmarks complete
- [ ] Memory benchmarks complete
- [ ] Cold start benchmarks complete
- [ ] Binary size measurements complete
- [ ] CPU usage benchmarks complete
- [ ] Results analyzed and documented
- [ ] Charts generated
- [ ] Report published
- [ ] README updated with actual results

---

## 🔗 References

- **Python llm-guard:** https://github.com/protectai/llm-guard
- **Rust benchmarking:** https://doc.rust-lang.org/cargo/commands/cargo-bench.html
- **Criterion.rs:** https://github.com/bheisler/criterion.rs
- **wrk:** https://github.com/wg/wrk
- **AWS EC2 pricing:** https://aws.amazon.com/ec2/pricing/

---

**Next Steps:** Begin Phase 1 (Setup) and execute benchmarks according to this plan.

**Note:** All performance claims in README.md should be validated and updated based on actual benchmark results. Be transparent about methodology and limitations.
