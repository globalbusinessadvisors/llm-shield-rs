# LLM Shield Performance Benchmark Results

**Report Date:** 2025-10-30
**Version:** 0.1.0
**Status:** 🚧 FRAMEWORK READY - Awaiting Benchmark Execution

---

## 📋 Executive Summary

This report validates the performance claims made in the main README regarding LLM Shield (Rust/WASM) compared to the Python llm-guard implementation.

### Overall Status

| Status | Tests | Result |
|--------|-------|--------|
| ⏳ **PENDING** | 0/18 | Benchmark infrastructure complete, awaiting execution |

**Note:** This document provides the complete reporting framework. Actual benchmark results will be populated when the benchmarks are executed in an environment with Rust installed.

### Performance Claims Validation

| Category | Claimed Improvement | Target | Status |
|----------|---------------------|--------|--------|
| **Latency** | 10-25x faster | <20ms | ⏳ Pending |
| **Throughput** | 100x higher | >10,000 req/sec | ⏳ Pending |
| **Memory** | 8-16x lower | <500MB | ⏳ Pending |
| **Cold Start** | 10-30x faster | <1s | ⏳ Pending |
| **Binary Size** | 60-100x smaller | <2MB WASM | ⏳ Pending |
| **CPU Usage** | 5-10x more efficient | - | ⏳ Pending |

---

## 🎯 Test Environment

### Hardware Specifications

**Recommended (for actual benchmarking):**
- **Instance:** AWS EC2 c5.xlarge
- **vCPU:** 4 cores (Intel Xeon Platinum 8000 series)
- **RAM:** 8GB
- **Storage:** EBS gp3 (provisioned IOPS)
- **Network:** Up to 10 Gbps

**Minimum:**
- **CPU:** 4 physical cores
- **RAM:** 8GB
- **OS:** Linux/macOS with modern kernel

### Software Versions

**Rust Stack:**
- Rust: 1.75+ (stable channel)
- Cargo: 1.75+
- rustc optimization: `--release` mode with LTO

**Python Stack:**
- Python: 3.11
- llm-guard: 0.3.x
- FastAPI: 0.104+
- uvicorn: 0.24+

**Benchmark Tools:**
- criterion.rs: 0.5+
- wrk: 4.2+
- hyperfine: 1.18+
- pidstat: 12.5+ (sysstat)

---

## 📊 Detailed Results

### 1. Latency Benchmarks

**Objective:** Validate <20ms average latency (10-25x improvement)

#### Scenario 1A: Simple String Matching (BanSubstrings)

| Implementation | Mean (ms) | Median (ms) | P95 (ms) | P99 (ms) | Std Dev |
|----------------|-----------|-------------|----------|----------|---------|
| Python llm-guard | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| Rust LLM Shield | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** | - |

**Test Configuration:**
- Patterns: 3 banned substrings
- Input sizes: Short (50 words), Medium (200 words), Long (500 words)
- Iterations: 1000
- Expected: <1ms for Rust, 5-15ms for Python

#### Scenario 1B: Regex Pattern Matching

| Implementation | Mean (ms) | Median (ms) | P95 (ms) | P99 (ms) | Std Dev |
|----------------|-----------|-------------|----------|----------|---------|
| Python llm-guard | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| Rust LLM Shield | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** | - |

**Test Configuration:**
- Patterns: 10 regex patterns (SSN, email, credit card, etc.)
- Input size: 200 words
- Iterations: 1000
- Expected: 1-3ms for Rust, 10-30ms for Python

#### Scenario 1C: Secret Detection (40+ Patterns)

| Implementation | Mean (ms) | Median (ms) | P95 (ms) | P99 (ms) | Std Dev |
|----------------|-----------|-------------|----------|----------|---------|
| Python llm-guard | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| Rust LLM Shield | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** | - |

**Test Configuration:**
- Patterns: 40+ secret types with entropy validation
- Input: Mixed content with potential secrets
- Iterations: 1000
- Expected: 5-10ms for Rust, 50-100ms for Python

#### Scenario 1D: ML-Based Scanning (PromptInjection)

| Implementation | Mean (ms) | Median (ms) | P95 (ms) | P99 (ms) | Std Dev |
|----------------|-----------|-------------|----------|----------|---------|
| Python llm-guard | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| Rust LLM Shield | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** | - |

**Test Configuration:**
- Model: Transformer-based classification (ONNX)
- Input: Jailbreak attempts, role reversals, system prompt leaks
- Iterations: 100 (expensive operation)
- Expected: 50-150ms for Rust, 200-500ms for Python

#### Latency Summary

**Overall Average Latency:**
- Python: ⏳ ms
- Rust: ⏳ ms
- Improvement: ⏳x faster

**Claim Validation:** ⏳ PENDING
- Target: <20ms average
- Actual: ⏳ ms
- Status: ⏳

**Chart:** `charts/latency_comparison.png`

---

### 2. Throughput Benchmarks

**Objective:** Validate >10,000 req/sec (100x improvement)

#### Scenario 2A: Single Scanner (Concurrent Requests)

| Concurrency | Python (req/sec) | Rust (req/sec) | Improvement | P95 Latency (ms) |
|-------------|------------------|----------------|-------------|------------------|
| 10 | ⏳ | ⏳ | ⏳x | ⏳ / ⏳ |
| 50 | ⏳ | ⏳ | ⏳x | ⏳ / ⏳ |
| 100 | ⏳ | ⏳ | ⏳x | ⏳ / ⏳ |
| 500 | ⏳ | ⏳ | ⏳x | ⏳ / ⏳ |

**Test Configuration:**
- Tool: wrk HTTP load tester
- Duration: 60 seconds per test
- Scanner: BanSubstrings (simple)
- Workers: Python (4 uvicorn workers), Rust (tokio multi-threaded)

**Expected Results:**
- Python: 100-500 req/sec
- Rust: 10,000-50,000 req/sec

#### Scenario 2B: Scanner Pipeline (3 Scanners)

| Concurrency | Python (req/sec) | Rust (req/sec) | Improvement | Error Rate |
|-------------|------------------|----------------|-------------|------------|
| 10 | ⏳ | ⏳ | ⏳x | ⏳% / ⏳% |
| 50 | ⏳ | ⏳ | ⏳x | ⏳% / ⏳% |
| 100 | ⏳ | ⏳ | ⏳x | ⏳% / ⏳% |

**Test Configuration:**
- Pipeline: BanSubstrings → Secrets → Toxicity
- Duration: 60 seconds per test

**Expected Results:**
- Python: 50-100 req/sec
- Rust: 5,000-10,000 req/sec

#### Throughput Summary

**Peak Throughput:**
- Python: ⏳ req/sec
- Rust: ⏳ req/sec
- Improvement: ⏳x higher

**Claim Validation:** ⏳ PENDING
- Target: >10,000 req/sec
- Actual: ⏳ req/sec
- Status: ⏳

**Chart:** `charts/throughput_comparison.png`

---

### 3. Memory Usage Benchmarks

**Objective:** Validate <500MB under load (8-16x improvement)

#### Scenario 3A: Baseline Memory (Idle)

| Implementation | RSS (MB) | Heap (MB) | Stack (MB) | Total (MB) |
|----------------|----------|-----------|------------|------------|
| Python (4 workers) | ⏳ | ⏳ | ⏳ | ⏳ |
| Rust | ⏳ | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** |

**Test Configuration:**
- Measurement: After server startup, no requests
- Tool: `ps`, `pmap`, `valgrind massif`

**Expected Results:**
- Python: 1-2GB (models loaded per worker)
- Rust: 50-100MB

#### Scenario 3B: Under Load (1000 req/sec)

| Implementation | RSS (MB) | Peak (MB) | Growth (MB/min) |
|----------------|----------|-----------|-----------------|
| Python | ⏳ | ⏳ | ⏳ |
| Rust | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** |

**Test Configuration:**
- Load: 1000 req/sec sustained for 5 minutes
- Measurement: RSS every 10 seconds

**Expected Results:**
- Python: 4-8GB (GC pressure)
- Rust: 200-500MB (shared memory, no GC)

#### Scenario 3C: Memory Stability (1 hour)

| Implementation | Start (MB) | End (MB) | Growth | Stability |
|----------------|------------|----------|--------|-----------|
| Python | ⏳ | ⏳ | ⏳% | ⏳ |
| Rust | ⏳ | ⏳ | ⏳% | ⏳ |

**Test Configuration:**
- Duration: 1 hour at moderate load (100 req/sec)
- Measurement: Memory sampled every 60 seconds

**Expected Results:**
- Python: 10-20% growth (fragmentation)
- Rust: <5% growth (predictable allocations)

#### Memory Summary

**Average Memory Usage:**
- Python: ⏳ MB
- Rust: ⏳ MB
- Improvement: ⏳x lower

**Claim Validation:** ⏳ PENDING
- Target: <500MB under load
- Actual: ⏳ MB
- Status: ⏳

**Chart:** `charts/memory_usage.png`

---

### 4. Cold Start Benchmarks

**Objective:** Validate <1s startup (10-30x improvement)

#### Scenario 4A: Application Startup Time

| Implementation | Mean (ms) | Median (ms) | P95 (ms) | Fastest (ms) | Slowest (ms) |
|----------------|-----------|-------------|----------|--------------|--------------|
| Python | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| Rust | ⏳ | ⏳ | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** | **⏳x** |

**Test Configuration:**
- Tool: hyperfine
- Runs: 100 cold starts
- Measurement: Process start to "ready" state

**Expected Results:**
- Python: 10-30s (HuggingFace transformers loading)
- Rust: <1s (compiled binary, lazy ONNX loading)

#### Scenario 4B: First Request Latency

| Implementation | Mean (ms) | Median (ms) | P95 (ms) |
|----------------|-----------|-------------|----------|
| Python | ⏳ | ⏳ | ⏳ |
| Rust | ⏳ | ⏳ | ⏳ |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** |

**Test Configuration:**
- Measurement: Server start to first successful request
- Runs: 100

**Expected Results:**
- Python: 5-15s
- Rust: 100-500ms

#### Scenario 4C: Serverless Cold Start Simulation

| Platform | Mean (ms) | P95 (ms) | Notes |
|----------|-----------|----------|-------|
| Python (AWS Lambda) | ⏳ | ⏳ | 3-5GB image |
| Rust (AWS Lambda) | ⏳ | ⏳ | <100MB image |
| WASM (Cloudflare Workers) | ⏳ | ⏳ | <2MB bundle |

**Test Configuration:**
- Simulation: 100 rapid init/destroy cycles
- Measurement: Full initialization time

**Expected Results:**
- Python Lambda: 10-20s
- Rust Lambda: 500ms-1s
- WASM: <100ms

#### Cold Start Summary

**Average Cold Start:**
- Python: ⏳ ms
- Rust: ⏳ ms
- Improvement: ⏳x faster

**Claim Validation:** ⏳ PENDING
- Target: <1s (Rust), <100ms (WASM)
- Actual: ⏳ ms
- Status: ⏳

**Chart:** `charts/cold_start_comparison.png`

---

### 5. Binary Size Benchmarks

**Objective:** Validate <2MB WASM (60-100x improvement)

#### Measurements

| Format | Python | Rust | Improvement |
|--------|--------|------|-------------|
| **Docker Image** | ⏳ MB | ⏳ MB | ⏳x |
| **Native Binary** | N/A | ⏳ MB | - |
| **Native (stripped)** | N/A | ⏳ MB | - |
| **Native (UPX)** | N/A | ⏳ MB | - |
| **WASM (uncompressed)** | N/A | ⏳ MB | - |
| **WASM (wasm-opt -Oz)** | N/A | ⏳ MB | - |
| **WASM (gzip)** | N/A | ⏳ MB | ⏳x |

#### Detailed Breakdown

**Python Docker Image:**
```
Base image: python:3.11-slim (⏳ MB)
+ Dependencies: ⏳ MB
+ llm-guard: ⏳ MB
+ ML models: ⏳ MB
= Total: ⏳ MB
```

**Rust Docker Image:**
```
Base image: debian:bookworm-slim (⏳ MB)
+ Rust binary: ⏳ MB
+ Runtime deps: ⏳ MB
= Total: ⏳ MB
```

**WASM Bundle:**
```
Core logic: ⏳ MB
+ WASM overhead: ⏳ MB
= Uncompressed: ⏳ MB
After wasm-opt: ⏳ MB
After gzip: ⏳ MB
```

#### Binary Size Summary

**Production Deployment Sizes:**
- Python: ⏳ MB
- Rust (native): ⏳ MB
- Rust (WASM gzip): ⏳ MB

**Claim Validation:** ⏳ PENDING
- Target: <50MB (native), <2MB (WASM gzip)
- Actual: ⏳ MB / ⏳ MB
- Status: ⏳

**Chart:** `charts/binary_size_comparison.png`

---

### 6. CPU Usage Benchmarks

**Objective:** Validate 5-10x CPU efficiency improvement

#### Scenario 6A: Single Request CPU Time

| Implementation | Mean (ms) | CPU % | Instructions | Cycles |
|----------------|-----------|-------|--------------|--------|
| Python | ⏳ | ⏳ | ⏳ M | ⏳ M |
| Rust | ⏳ | ⏳ | ⏳ M | ⏳ M |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** |

**Test Configuration:**
- Tool: `perf stat`
- Scanner: BanSubstrings (simple)
- Runs: 1000

**Expected Results:**
- Python: 50-200ms CPU time (GIL, interpreted)
- Rust: 5-20ms CPU time (compiled, no GIL)

#### Scenario 6B: CPU Under Sustained Load

| Implementation | Avg CPU % | Peak CPU % | Cores Used | Efficiency |
|----------------|-----------|------------|------------|------------|
| Python (4 workers) | ⏳ | ⏳ | ⏳ | ⏳ req/sec per % |
| Rust | ⏳ | ⏳ | ⏳ | ⏳ req/sec per % |
| **Improvement** | **⏳x** | **⏳x** | **⏳x** | **⏳x** |

**Test Configuration:**
- Load: 1000 req/sec for 5 minutes
- Tool: `pidstat` (1 second intervals)

**Expected Results:**
- Python: 350-400% CPU (4 workers, GIL contention)
- Rust: 100-200% CPU (efficient multi-threading)

#### Scenario 6C: CPU Efficiency (Requests per Core)

| Implementation | Throughput (req/sec) | CPU Cores | Efficiency (req/sec/core) |
|----------------|----------------------|-----------|---------------------------|
| Python | ⏳ | 4 | ⏳ |
| Rust | ⏳ | 4 | ⏳ |
| **Improvement** | - | - | **⏳x** |

**Test Configuration:**
- Fixed 4 cores per implementation
- Measure maximum sustainable throughput

**Expected Results:**
- Python: 25-100 req/sec per core (GIL bottleneck)
- Rust: 2,500-10,000 req/sec per core (true parallelism)

#### CPU Summary

**CPU Efficiency:**
- Python: ⏳ req/sec per CPU %
- Rust: ⏳ req/sec per CPU %
- Improvement: ⏳x more efficient

**Claim Validation:** ⏳ PENDING
- Target: 5-10x more efficient
- Actual: ⏳x
- Status: ⏳

**Chart:** `charts/cpu_efficiency.png`

---

## 📈 Overall Performance Summary

### Improvement Factors

| Category | Claimed | Actual | Status | Notes |
|----------|---------|--------|--------|-------|
| **Latency** | 10-25x | ⏳x | ⏳ | Target: <20ms avg |
| **Throughput** | 100x | ⏳x | ⏳ | Target: >10,000 req/sec |
| **Memory** | 8-16x | ⏳x | ⏳ | Target: <500MB |
| **Cold Start** | 10-30x | ⏳x | ⏳ | Target: <1s |
| **Binary Size** | 60-100x | ⏳x | ⏳ | Target: <2MB WASM |
| **CPU** | 5-10x | ⏳x | ⏳ | Efficiency metric |

### Pass/Fail Summary

**Total Tests:** 18 scenarios across 6 categories

| Status | Count | Percentage |
|--------|-------|------------|
| ✅ Passed | ⏳ | ⏳% |
| ❌ Failed | ⏳ | ⏳% |
| ⏳ Pending | 18 | 100% |

**Overall Validation:** ⏳ PENDING

---

## 🎯 Performance Highlights

### 🏆 Top Performers

**Once benchmarks are executed, this section will highlight:**

1. **Biggest Improvement:** ⏳
2. **Closest to Target:** ⏳
3. **Most Consistent:** ⏳
4. **Biggest Surprise:** ⏳

### ⚠️ Areas of Concern

**Potential issues to investigate:**

1. ⏳ (To be determined after execution)
2. ⏳
3. ⏳

---

## 🔍 Methodology

### Test Data

**1000 Test Prompts:**
- 200 simple (10-50 words)
- 200 medium (50-200 words)
- 200 long (200-500 words)
- 100 with secrets (API keys, tokens)
- 100 with code snippets
- 100 with prompt injection attempts
- 100 toxic/harmful content

**Generation:** Automated via `generate_test_prompts()` in `llm-shield-benches`

### Statistical Analysis

**Metrics Collected:**
- Mean (average)
- Median (50th percentile)
- P95 (95th percentile)
- P99 (99th percentile)
- Standard deviation
- Min/Max values

**Sample Sizes:**
- Fast operations: 1000 iterations
- ML operations: 100 iterations (expensive)
- Cold starts: 100 runs
- Throughput: 60 second sustained tests

### Benchmark Tools

**Rust:**
- Framework: Criterion.rs 0.5
- Async: Tokio multi-threaded runtime
- Optimizations: `--release` with LTO
- Profiling: `perf`, `flamegraph`

**Python:**
- Framework: timeit, custom benchmarks
- Server: FastAPI + uvicorn (4 workers)
- Profiling: cProfile, py-spy

**System:**
- Load testing: wrk 4.2
- Cold start: hyperfine 1.18
- Memory: pidstat, pmap, valgrind
- Binary: docker images, ls -lh

### Comparison Logic

**Improvement Factor Calculation:**
```
For latency, memory, cold start (lower is better):
  improvement = python_value / rust_value

For throughput, CPU efficiency (higher is better):
  improvement = rust_value / python_value

For binary size (lower is better, different baselines):
  improvement = python_docker_size / rust_wasm_size
```

**Pass/Fail Criteria:**
- ✅ **PASS:** Actual improvement within claimed range (±10% tolerance)
- ⚠️ **PARTIAL:** Actual improvement within 50% of claimed minimum
- ❌ **FAIL:** Actual improvement below 50% of claimed minimum

### Reproducibility

**All tests are reproducible via:**

```bash
# 1. Setup environment
cd /workspaces/llm-shield-rs/benchmarks

# 2. Install dependencies
pip install -r python/requirements.txt
sudo apt-get install wrk hyperfine sysstat

# 3. Build Rust (release mode)
cd ../..
cargo build --release

# 4. Run all benchmarks
cd benchmarks/scripts
./run_all_benchmarks.sh

# 5. Analyze results
python analyze_results.py

# 6. Generate charts
python generate_charts.py
```

**Environment Variables:**
- `RUST_LOG=info` - Rust logging
- `PYTHONUNBUFFERED=1` - Python logging
- `CRITERION_HOME=target/criterion` - Criterion output

---

## 📌 Limitations & Caveats

### Known Limitations

1. **Hardware Dependency:** Results vary by CPU architecture, core count, and memory speed
2. **Python GIL:** Multi-worker Python may show better results than single-process
3. **ML Models:** ONNX vs PyTorch performance depends on model optimization
4. **Network I/O:** Throughput tests affected by network stack and OS tuning
5. **Cold Start Variance:** First run may include one-time initialization costs

### Fair Comparison Considerations

**What Makes This Fair:**
- ✅ Same test data for both implementations
- ✅ Same hardware and OS
- ✅ Both running in production-optimized mode
- ✅ Multiple runs with statistical analysis
- ✅ Real-world workload scenarios

**Potential Biases:**
- ⚠️ Rust benefits from compile-time optimizations
- ⚠️ Python may benefit from warm interpreter caches
- ⚠️ WASM has additional overhead vs native Rust
- ⚠️ Comparison is against baseline Python, not optimized variants

### Interpretation Guidelines

**How to Read These Results:**

1. **Ranges vs Single Values:** Claims with ranges (10-25x) accommodate different scenarios
2. **Statistical Variance:** Focus on median/p95, not just mean
3. **Scenario Context:** ML workloads differ from simple string matching
4. **Production Reality:** Benchmark results approximate real-world performance

---

## 🎓 Recommendations

### For README Updates

**Based on benchmark results, the following README updates are recommended:**

⏳ (To be determined after benchmark execution)

**Suggested changes:**

1. ⏳
2. ⏳
3. ⏳

### For Future Optimization

**Performance improvement opportunities:**

1. ⏳
2. ⏳
3. ⏳

### For Documentation

**Additional documentation needed:**

1. ⏳
2. ⏳
3. ⏳

---

## 📚 Appendix

### A. Benchmark Infrastructure

**Files Created:**
- `crates/llm-shield-benches/` - Rust benchmark crate
- `benches/latency.rs` - Latency scenarios (350 lines)
- `benches/throughput.rs` - Throughput scenarios (100 lines)
- `benches/memory.rs` - Memory scenarios (90 lines)
- `benches/cold_start.rs` - Cold start scenarios (110 lines)
- `benches/binary_size.rs` - Binary size measurements (100 lines)
- `benches/cpu_usage.rs` - CPU scenarios (110 lines)
- `scripts/analyze_results.py` - Analysis script
- `scripts/generate_charts.py` - Chart generation
- Total: ~1,260 lines of benchmark code + 600 lines of analysis

### B. Raw Data Files

**When benchmarks are executed, results will be saved to:**

- `results/rust/latency_results.csv`
- `results/rust/throughput_results.csv`
- `results/rust/memory_results.csv`
- `results/rust/cold_start_results.csv`
- `results/rust/binary_size_results.csv`
- `results/rust/cpu_results.csv`
- `results/python/latency_results.csv`
- `results/python/throughput_results.csv`
- `results/python/memory_results.csv`
- `results/analysis.json` - Aggregated analysis

### C. Chart Files

**Generated visualizations:**

- `charts/latency_comparison.png`
- `charts/throughput_comparison.png`
- `charts/memory_usage.png`
- `charts/cold_start_comparison.png`
- `charts/binary_size_comparison.png`
- `charts/cpu_efficiency.png`
- `charts/improvement_summary.png`

### D. Benchmark Execution Commands

**To execute benchmarks when Rust is available:**

```bash
# Latency
cargo bench --bench latency

# Throughput
cargo bench --bench throughput

# Memory
cargo bench --bench memory

# Cold Start
cargo bench --bench cold_start

# Binary Size
cargo bench --bench binary_size

# CPU Usage
cargo bench --bench cpu_usage

# All benchmarks
cargo bench

# View Criterion HTML reports
open target/criterion/report/index.html
```

---

## 📄 Document History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 0.1.0 | 2025-10-30 | Initial framework created | Results Analysis Agent |

---

## 🔗 References

1. **Benchmark Plan:** `/workspaces/llm-shield-rs/plans/PERFORMANCE_BENCHMARK_PLAN.md`
2. **Architecture:** `/workspaces/llm-shield-rs/benchmarks/ARCHITECTURE.md`
3. **Phase 4 Summary:** `/workspaces/llm-shield-rs/benchmarks/PHASE_4_COMPLETE.md`
4. **Criterion.rs:** https://github.com/bheisler/criterion.rs
5. **Python llm-guard:** https://github.com/protectai/llm-guard
6. **wrk:** https://github.com/wg/wrk
7. **hyperfine:** https://github.com/sharkdp/hyperfine

---

**Status:** 🚧 Framework Complete - Ready for Benchmark Execution

**Next Steps:**
1. Execute benchmarks in Rust-enabled environment
2. Run Python baselines
3. Execute analysis scripts
4. Generate charts
5. Populate this report with actual results
6. Update README.md with validated claims
