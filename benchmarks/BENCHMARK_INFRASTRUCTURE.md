# LLM Shield Benchmark Infrastructure

## Overview

This directory contains the complete performance benchmarking infrastructure for validating LLM Shield's performance claims against Python llm-guard.

## Directory Structure

```
benchmarks/
├── scripts/           # Benchmark execution scripts
│   ├── run_all_benchmarks.sh      # Master orchestrator
│   ├── bench_latency.sh            # Latency benchmarks
│   ├── bench_throughput.sh         # Throughput benchmarks
│   ├── bench_memory.sh             # Memory usage benchmarks
│   ├── bench_cold_start.sh         # Cold start benchmarks
│   ├── bench_binary_size.sh        # Binary size measurements
│   ├── bench_cpu.sh                # CPU usage benchmarks
│   └── generate_test_data.py       # Test data generator
├── data/              # Test datasets
│   └── test_prompts.json           # 1000 diverse test prompts
├── results/           # Benchmark results (CSV format)
│   ├── latency_results.csv
│   ├── throughput_results.csv
│   ├── memory_results.csv
│   ├── cold_start_results.csv
│   ├── binary_size_results.csv
│   └── cpu_results.csv
├── charts/            # Generated visualizations
├── analysis/          # Analysis scripts
└── python/            # Python baseline benchmarks
    ├── bench_latency.py
    └── requirements.txt
```

## Test Dataset Composition

The `test_prompts.json` file contains **1000 diverse prompts**:

| Category | Count | Description |
|----------|-------|-------------|
| Simple | 200 | 10-50 words, basic queries |
| Medium | 200 | 50-200 words, complex scenarios |
| Long | 200 | 200-500 words, detailed prompts |
| Secrets | 100 | Contains API keys, tokens, passwords |
| Code | 100 | Contains code snippets (Python, JS, Rust) |
| Injection | 100 | Prompt injection attempts |
| Toxic | 100 | Toxic/harmful content (sanitized) |

### Dataset Statistics
- **Total prompts**: 1000
- **Total size**: ~748 KB
- **Format**: JSON with metadata
- **Reproducibility**: Seeded random generation (seed=42)

## Benchmark Categories

### 1. Latency Benchmarks
**Goal**: Measure end-to-end latency for single requests

**Scenarios**:
- Simple string matching (BanSubstrings)
- Regex scanning (10 patterns)
- Secret detection (40+ patterns)
- ML-based scanning (PromptInjection)
- Mixed workload (all scanners)

**Metrics**: mean, median, p95, p99, min, max, stddev

**Target**: Rust <20ms average (10-25x faster than Python)

### 2. Throughput Benchmarks
**Goal**: Measure requests per second under sustained load

**Scenarios**:
- Single scanner with varying concurrency (10, 50, 100, 500)
- Scanner pipeline (3 scanners in sequence)
- Mixed scanners

**Metrics**: req/sec, latency distribution, error rate

**Target**: Rust >10,000 req/sec (100x higher than Python)

### 3. Memory Usage Benchmarks
**Goal**: Measure resident memory (RSS) during operation

**Scenarios**:
- Baseline memory (idle)
- Memory under load (1000 req/sec)
- Memory growth over time (1 hour)
- Memory stability (10K iterations)

**Metrics**: baseline MB, under load MB, peak MB, growth rate

**Target**: Rust <500MB under load (8-16x lower than Python)

### 4. Cold Start Benchmarks
**Goal**: Measure startup time and first request latency

**Scenarios**:
- Application startup time
- First request latency
- AWS Lambda cold start
- Cloudflare Workers (WASM)
- Model loading time

**Metrics**: mean, median, min, max, stddev, p95, p99

**Target**: Rust <1s startup, WASM <100ms (10-30x faster)

### 5. Binary Size Benchmarks
**Goal**: Measure deployment artifact sizes

**Scenarios**:
- Docker image size (Python vs Rust)
- Native binary (stripped, UPX compressed)
- WASM bundle (uncompressed, optimized, gzipped)

**Metrics**: Size in MB, compression ratios

**Target**: Rust <100MB Docker, WASM <2MB gzipped (60-100x smaller)

### 6. CPU Usage Benchmarks
**Goal**: Measure CPU utilization under load

**Scenarios**:
- Single request CPU time
- CPU usage under sustained load
- CPU efficiency (req/sec per core)
- Scaling across cores (1, 2, 4, 8)

**Metrics**: mean CPU %, peak CPU %, req/sec per core

**Target**: Rust 5-10x more CPU efficient, linear scaling

## Running Benchmarks

### Prerequisites

**System Requirements**:
- Ubuntu 22.04 LTS (or macOS)
- 4+ CPU cores, 8GB+ RAM
- 20GB+ disk space

**Tools**:
```bash
# Ubuntu
sudo apt-get update
sudo apt-get install -y build-essential python3 python3-pip docker.io sysstat

# Install wrk (HTTP benchmarking)
sudo apt-get install -y wrk

# Install hyperfine (cold start benchmarking)
cargo install hyperfine

# Install wasm-pack (WASM benchmarks)
cargo install wasm-pack

# Install wasm-opt (WASM optimization)
sudo apt-get install -y binaryen
```

**Python Dependencies**:
```bash
cd benchmarks/python
pip3 install -r requirements.txt
```

**Rust Build**:
```bash
cd /workspaces/llm-shield-rs
cargo build --release
```

### Quick Start

Run all benchmarks:
```bash
cd /workspaces/llm-shield-rs/benchmarks
./scripts/run_all_benchmarks.sh
```

Run individual benchmarks:
```bash
# Latency
./scripts/bench_latency.sh

# Throughput
./scripts/bench_throughput.sh

# Memory
./scripts/bench_memory.sh

# Cold start
./scripts/bench_cold_start.sh

# Binary size
./scripts/bench_binary_size.sh

# CPU usage
./scripts/bench_cpu.sh
```

### Generate Test Data

Regenerate the test dataset:
```bash
cd /workspaces/llm-shield-rs/benchmarks
python3 scripts/generate_test_data.py
```

## Result Collection Templates

All benchmark results are stored in CSV format for easy analysis:

### latency_results.csv
```csv
scenario,scanner_type,language,iterations,mean_ms,median_ms,p95_ms,p99_ms,min_ms,max_ms,stddev_ms,claim_target_ms,claim_validated
```

### throughput_results.csv
```csv
scenario,scanner_type,language,concurrency,duration_secs,total_requests,successful_requests,failed_requests,requests_per_sec,mean_latency_ms,p50_latency_ms,p95_latency_ms,p99_latency_ms,errors_per_sec,claim_target_rps,claim_validated
```

### memory_results.csv
```csv
scenario,language,baseline_mb,under_load_mb,peak_mb,growth_mb,growth_rate_percent,duration_secs,claim_target_mb,claim_validated
```

### cold_start_results.csv
```csv
scenario,language,platform,runs,mean_ms,median_ms,min_ms,max_ms,stddev_ms,p95_ms,p99_ms,claim_target_ms,claim_validated
```

### binary_size_results.csv
```csv
artifact_type,language,uncompressed_mb,compressed_mb,optimization_applied,claim_target_mb,claim_validated
```

### cpu_results.csv
```csv
scenario,language,workers,duration_secs,mean_cpu_percent,peak_cpu_percent,min_cpu_percent,requests_per_sec,requests_per_cpu_core,cpu_efficiency_score,claim_target,claim_validated
```

## Validation Criteria

### Performance Claims to Validate

| Metric | Python | Rust | Improvement | Status |
|--------|--------|------|-------------|--------|
| Latency | 200-500ms | <20ms | 10-25x faster | PENDING |
| Throughput | 100 req/sec | 10,000+ req/sec | 100x higher | PENDING |
| Memory | 4-8GB | <500MB | 8-16x lower | PENDING |
| Cold Start | 10-30s | <1s | 10-30x faster | PENDING |
| Binary Size | 3-5GB | <50MB native / <2MB WASM | 60-100x smaller | PENDING |
| CPU Usage | High (GIL) | Low (parallel) | 5-10x lower | PENDING |

### Validation Status
- ✅ **PASS**: Claim validated, meets or exceeds target
- ❌ **FAIL**: Claim not validated, does not meet target
- ⏳ **PENDING**: Benchmark not yet executed

## Analysis & Reporting

After running benchmarks, results are analyzed and visualized:

### Analysis Scripts
```bash
cd /workspaces/llm-shield-rs/benchmarks/analysis

# Analyze results
python3 analyze_results.py

# Generate charts
python3 generate_charts.py

# Validate claims
python3 validate_claims.py
```

### Generated Outputs
- **results/RESULTS.md**: Comprehensive results report
- **charts/*.png**: Comparison visualizations
- **charts/*.svg**: High-quality vector graphics

## Troubleshooting

### Common Issues

**Issue**: wrk not found
```bash
sudo apt-get install wrk
```

**Issue**: hyperfine not found
```bash
cargo install hyperfine
```

**Issue**: Python llm-guard not installed
```bash
cd benchmarks/python
pip3 install -r requirements.txt
```

**Issue**: Rust binary not built
```bash
cargo build --release
```

**Issue**: Test data missing
```bash
python3 scripts/generate_test_data.py
```

## Next Steps

1. **Execute Benchmarks**: Run `./scripts/run_all_benchmarks.sh`
2. **Review Results**: Check `results/*.csv` files
3. **Analyze Data**: Run analysis scripts
4. **Generate Report**: Review `results/RESULTS.md`
5. **Update README**: Update main README with validated claims

## Contributing

When adding new benchmarks:
1. Add benchmark script to `scripts/`
2. Create CSV template in `results/`
3. Update `run_all_benchmarks.sh`
4. Document in this file
5. Add analysis script if needed

## References

- **Benchmark Plan**: `/workspaces/llm-shield-rs/plans/PERFORMANCE_BENCHMARK_PLAN.md`
- **Python llm-guard**: https://github.com/protectai/llm-guard
- **Criterion.rs**: https://github.com/bheisler/criterion.rs
- **wrk**: https://github.com/wg/wrk
- **hyperfine**: https://github.com/sharkdp/hyperfine

---

**Last Updated**: 2025-10-30
**Infrastructure Version**: 1.0.0
**Status**: Ready for benchmark execution
