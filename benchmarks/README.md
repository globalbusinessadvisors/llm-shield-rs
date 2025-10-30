# LLM Shield Performance Benchmarks

## Overview

This directory contains the comprehensive benchmarking infrastructure to validate the 6 performance claims made in the main README:

1. **Latency**: <20ms (10-25x faster than Python)
2. **Throughput**: >10,000 req/sec (100x higher)
3. **Memory**: <500MB (8-16x lower)
4. **Cold Start**: <1s (10-30x faster)
5. **Binary Size**: <2MB WASM gzip (60-100x smaller)
6. **CPU**: 5-10x more efficient

## Directory Structure

```
benchmarks/
├── README.md                   # This file
├── ARCHITECTURE.md             # System architecture and design
├── PSEUDOCODE.md               # Algorithm pseudocode
├── METHODOLOGY.md              # Detailed methodology (to be created)
├── data/                       # Test datasets
│   └── test_prompts.json      # 1000 diverse test prompts
├── scripts/                    # Automation scripts
│   ├── run_all_benchmarks.sh  # Master runner
│   ├── bench_latency.sh       # Latency tests
│   ├── bench_throughput.sh    # Throughput tests
│   ├── bench_memory.sh        # Memory tests
│   ├── bench_cold_start.sh    # Cold start tests
│   ├── bench_binary_size.sh   # Binary size measurements
│   └── bench_cpu.sh           # CPU profiling
├── python/                     # Python baseline implementations
│   ├── requirements.txt
│   ├── bench_latency.py
│   ├── bench_throughput.py
│   ├── bench_memory.py
│   └── ...
├── analysis/                   # Analysis scripts
│   ├── analyze_results.py
│   ├── generate_charts.py
│   └── validate_claims.py
├── results/                    # Benchmark results (generated)
│   ├── rust/
│   ├── python/
│   └── RESULTS.md             # Final report
└── charts/                     # Generated charts (PNG)
    ├── latency_comparison.png
    ├── throughput_comparison.png
    └── ...
```

## Quick Start

### Prerequisites

**System Tools:**
```bash
# Ubuntu/Debian
sudo apt-get install wrk hyperfine sysstat docker.io

# macOS
brew install wrk hyperfine
```

**Python Dependencies:**
```bash
cd benchmarks/python
pip install -r requirements.txt
```

**Rust:**
```bash
# Build release binaries
cd ../..
cargo build --release
```

### Running Benchmarks

**Run all benchmarks:**
```bash
cd benchmarks/scripts
./run_all_benchmarks.sh
```

**Run individual category:**
```bash
./bench_latency.sh      # Latency tests
./bench_throughput.sh   # Throughput tests
./bench_memory.sh       # Memory profiling
./bench_cold_start.sh   # Cold start measurements
./bench_binary_size.sh  # Binary size measurements
./bench_cpu.sh          # CPU profiling
```

### Running Rust Benchmarks Only

```bash
cd ../..  # Project root

# Run all Rust benchmarks
cargo bench

# Run specific category
cargo bench --bench latency
cargo bench --bench throughput
cargo bench --bench memory
```

## Benchmark Details

### 1. Latency Benchmarks

**File:** `crates/llm-shield-benches/benches/latency.rs`

**Tests:**
- Scenario 1A: BanSubstrings (simple string matching)
- Scenario 1B: Regex (10 custom patterns)
- Scenario 1C: Secrets (40+ secret patterns)
- Scenario 1D: PromptInjection (ML model inference)

**Expected Results:**
- Python: 5-500ms (varies by complexity)
- Rust: <1-20ms (varies by complexity)
- Overall: 10-25x improvement

**Run:**
```bash
cargo bench --bench latency
```

### 2. Throughput Benchmarks

**Tool:** `wrk` HTTP load tester

**Tests:**
- Scenario 2A: Single scanner, concurrent requests
- Scenario 2B: Scanner pipeline (3 scanners)

**Expected Results:**
- Python: 100-500 req/sec
- Rust: 10,000-50,000 req/sec
- Improvement: 50-100x

**Run:**
```bash
./scripts/bench_throughput.sh
```

### 3. Memory Benchmarks

**Tool:** `pidstat`, `pmap`, `valgrind`

**Tests:**
- Scenario 3A: Baseline memory (idle server)
- Scenario 3B: Under load (1000 req/sec)
- Scenario 3C: Memory growth (1 hour test)

**Expected Results:**
- Python baseline: 1-2GB
- Rust baseline: 50-100MB
- Improvement: 10-20x

**Run:**
```bash
./scripts/bench_memory.sh
```

### 4. Cold Start Benchmarks

**Tool:** `hyperfine`

**Tests:**
- Scenario 4A: Application startup time
- Scenario 4B: First request latency
- Scenario 4C: Serverless cold start

**Expected Results:**
- Python: 10-30s
- Rust: <1s
- WASM: <100ms
- Improvement: 10-300x

**Run:**
```bash
./scripts/bench_cold_start.sh
```

### 5. Binary Size Measurements

**Tests:**
- Scenario 5A: Docker image size
- Scenario 5B: Native binary (stripped + UPX)
- Scenario 5C: WASM bundle (optimized + gzip)

**Expected Results:**
- Python Docker: 3-5GB
- Rust Docker: <100MB
- Rust binary: 10-50MB
- WASM gzip: <2MB
- Improvement: 60-5000x

**Run:**
```bash
./scripts/bench_binary_size.sh
```

### 6. CPU Usage Benchmarks

**Tool:** `pidstat`, `perf`, `flamegraph`

**Tests:**
- Scenario 6A: Single request CPU time
- Scenario 6B: CPU % under sustained load
- Scenario 6C: CPU efficiency (req/sec per core)

**Expected Results:**
- Python: 350-400% CPU (4 workers, GIL)
- Rust: 100-200% CPU (efficient multi-threading)
- Efficiency: 100x more req/sec per CPU core

**Run:**
```bash
./scripts/bench_cpu.sh
```

## Test Data

### Generation

Test data is automatically generated on first run. To manually regenerate:

```bash
cargo run --release --bin generate-test-data -- \
    --count 1000 \
    --output benchmarks/data/test_prompts.json
```

### Distribution

1000 test prompts:
- 200 simple (10-50 words)
- 200 medium (50-200 words)
- 200 long (200-500 words)
- 100 with secrets (API keys, tokens)
- 100 with code snippets
- 100 with prompt injection attempts
- 100 toxic/harmful content

## Results & Reports

After running benchmarks, results are saved to:

- **CSV Data:** `benchmarks/results/{rust|python}/{test}_results.csv`
- **Charts:** `benchmarks/charts/{test}_comparison.png`
- **Final Report:** `benchmarks/results/RESULTS.md`

### Example Report Structure

```markdown
# LLM Shield Performance Benchmark Results

## Executive Summary
- Overall Validation: ✅ PASS
- Categories Passed: 6/6

## Results Summary
| Category | Rust | Python | Improvement | Claim | Status |
|----------|------|--------|-------------|-------|--------|
| Latency | 2.5ms | 45ms | 18x | 10-25x | ✅ |
| Throughput | 15,000 req/s | 120 req/s | 125x | 100x | ✅ |
| ... | ... | ... | ... | ... | ... |
```

## Validation Criteria

Each benchmark must:
1. ✅ Run successfully on both Python and Rust
2. ✅ Collect valid measurements (no errors)
3. ✅ Generate statistical summaries (p50, p95, p99)
4. ✅ Compare against expected results
5. ✅ Validate README claims (pass/fail)
6. ✅ Generate charts and reports
7. ✅ Be reproducible (documented environment)

## Environment Recommendations

### Ideal Setup (AWS EC2)
- Instance: c5.xlarge (4 vCPU, 8GB RAM)
- OS: Ubuntu 22.04 LTS
- Python: 3.11
- Rust: 1.75+

### Minimum Setup
- CPU: 4 cores
- RAM: 8GB
- OS: Linux or macOS
- Python: 3.10+
- Rust: 1.70+

## Troubleshooting

### wrk not found
```bash
# Ubuntu
sudo apt-get install wrk

# macOS
brew install wrk
```

### hyperfine not found
```bash
cargo install hyperfine
```

### Python llm-guard not installed
```bash
cd benchmarks/python
pip install -r requirements.txt
```

### Permission denied on scripts
```bash
chmod +x benchmarks/scripts/*.sh
```

## Contributing

When adding new benchmarks:

1. Follow SPARC methodology (Specification → Pseudocode → Architecture → Refinement → Completion)
2. Use London School TDD (tests first, outside-in, behavior-focused)
3. Add both Rust and Python implementations
4. Update this README with test details
5. Add to the master runner script
6. Include validation criteria

## References

- **SPARC Methodology:** Reuven Cohen's 5-phase development process
- **London School TDD:** Outside-in test-driven development
- **Criterion.rs:** https://github.com/bheisler/criterion.rs
- **wrk:** https://github.com/wg/wrk
- **Python llm-guard:** https://github.com/protectai/llm-guard

## License

MIT OR Apache-2.0 (same as parent project)
