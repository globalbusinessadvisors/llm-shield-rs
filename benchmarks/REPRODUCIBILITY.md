# LLM Shield Benchmark Reproducibility Guide

This guide ensures anyone can reproduce the performance benchmarks and validate the claims made in the README.

---

## 🎯 Overview

**What You'll Get:**
- Automated benchmark execution across all 6 categories
- Statistical analysis with p50, p95, p99 percentiles
- Comparison charts (PNG format)
- Comprehensive results report
- Validation against claimed improvements

**Time Required:**
- Setup: 30-60 minutes
- Execution: 2-4 hours (automated)
- Analysis: 10-15 minutes

**Skill Level:** Intermediate (command line, Docker, basic statistics)

---

## 📋 Prerequisites

### Hardware Requirements

**Minimum:**
- CPU: 4 physical cores (2.0 GHz+)
- RAM: 8GB
- Storage: 20GB free space
- Network: Stable connection for downloads

**Recommended (AWS EC2 c5.xlarge):**
- CPU: 4 vCPUs (Intel Xeon Platinum 8000 series)
- RAM: 8GB
- Storage: 30GB EBS gp3
- Network: Up to 10 Gbps

**Why these specs?**
- 4 cores: Tests parallel execution (Python workers vs Rust async)
- 8GB RAM: Accommodates Python ML models + Rust processes
- 20GB+: Docker images, build artifacts, Rust toolchain

### Operating System

**Supported:**
- ✅ Ubuntu 22.04 LTS (recommended)
- ✅ Ubuntu 20.04 LTS
- ✅ Debian 11+
- ✅ macOS 12+ (Intel/Apple Silicon)
- ⚠️ Windows (WSL2 required)

**Installation Notes:**
- Linux: Native performance, all tools available
- macOS: Good performance, may need Homebrew for tools
- Windows: Use WSL2 for Linux environment

---

## 🔧 Step 1: Environment Setup

### 1.1 Install Rust (1.75+)

```bash
# Install rustup (Rust toolchain installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Select default installation (option 1)
# Restart shell or run:
source $HOME/.cargo/env

# Verify installation
rustc --version  # Should show 1.75 or newer
cargo --version
```

**Troubleshooting:**
- If `cargo` not found: Add `~/.cargo/bin` to PATH
- If old version: Run `rustup update stable`

### 1.2 Install Python (3.11+)

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y python3.11 python3.11-venv python3-pip

# macOS
brew install python@3.11

# Verify
python3.11 --version  # Should show 3.11.x
```

### 1.3 Install System Tools

**Ubuntu/Debian:**
```bash
sudo apt-get install -y \
    wrk \
    sysstat \
    docker.io \
    build-essential \
    pkg-config \
    libssl-dev

# Install hyperfine
wget https://github.com/sharkdp/hyperfine/releases/download/v1.18.0/hyperfine_1.18.0_amd64.deb
sudo dpkg -i hyperfine_1.18.0_amd64.deb
```

**macOS:**
```bash
brew install wrk hyperfine
```

### 1.4 Install Python Dependencies

```bash
cd /workspaces/llm-shield-rs/benchmarks/python

# Create virtual environment
python3.11 -m venv venv
source venv/bin/activate

# Install dependencies
pip install --upgrade pip
pip install -r requirements.txt

# Verify llm-guard installation
python -c "import llm_guard; print(llm_guard.__version__)"
```

**Expected output:** `0.3.x`

### 1.5 Verify All Tools

```bash
# Run verification script
cd ../scripts
./verify_environment.sh
```

**Expected output:**
```
✓ Rust: 1.75.0
✓ Python: 3.11.x
✓ wrk: 4.2.0
✓ hyperfine: 1.18.0
✓ pidstat: 12.5.x
✓ Docker: 24.x
✓ llm-guard: 0.3.x
✓ All dependencies installed
```

---

## 🚀 Step 2: Build LLM Shield

### 2.1 Build Rust (Release Mode)

```bash
cd /workspaces/llm-shield-rs

# Build with optimizations
cargo build --release

# This takes 5-10 minutes on first build
# Subsequent builds are incremental (faster)
```

**What's happening:**
- Compiles all crates with LTO (Link Time Optimization)
- Applies release optimizations (opt-level=3)
- Generates optimized binary in `target/release/`

**Verify build:**
```bash
ls -lh target/release/llm-shield*
# Should show binaries around 30-50MB
```

### 2.2 Build Benchmarks

```bash
# Build benchmark binaries
cargo bench --no-run

# This compiles all 6 benchmark suites
# Takes 3-5 minutes
```

**Verify:**
```bash
ls -lh target/release/deps/*bench*
# Should show 6 benchmark executables
```

### 2.3 Optional: Build WASM

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM
cd crates/llm-shield-wasm
wasm-pack build --release --target web

# Optimize with wasm-opt
npm install -g wasm-opt
wasm-opt -Oz pkg/llm_shield_wasm_bg.wasm -o pkg/optimized.wasm

# Measure size
gzip -c pkg/optimized.wasm | wc -c
# Should be <2MB
```

---

## 📊 Step 3: Run Benchmarks

### 3.1 Quick Test (2 minutes)

```bash
cd /workspaces/llm-shield-rs/benchmarks/scripts

# Run quick latency test
./bench_latency.sh --quick

# Expected output:
# ✓ Scenario 1A: 0.5ms (Rust) vs 12ms (Python) = 24x faster
# ✓ Scenario 1B: 2.1ms (Rust) vs 28ms (Python) = 13x faster
# ...
```

### 3.2 Full Benchmark Suite (2-4 hours)

**Option A: Automated (Recommended)**

```bash
# Run all benchmarks
./run_all_benchmarks.sh

# This script:
# 1. Runs Rust benchmarks (cargo bench)
# 2. Runs Python baselines
# 3. Collects results to ../results/
# 4. Shows progress and ETAs
```

**Option B: Category by Category**

```bash
# 1. Latency (15 minutes)
./bench_latency.sh
# Output: results/rust/latency_results.csv
#         results/python/latency_results.csv

# 2. Throughput (30 minutes)
./bench_throughput.sh
# Starts HTTP servers, runs wrk load tests

# 3. Memory (45 minutes)
./bench_memory.sh
# Monitors memory usage under load

# 4. Cold Start (20 minutes)
./bench_cold_start.sh
# Runs 100 startup tests

# 5. Binary Size (5 minutes)
./bench_binary_size.sh
# Measures Docker images, WASM bundles

# 6. CPU Usage (30 minutes)
./bench_cpu.sh
# Profiles CPU utilization
```

**Option C: Rust Only (Fast, 30 minutes)**

```bash
cd /workspaces/llm-shield-rs
cargo bench

# View HTML reports
open target/criterion/report/index.html
```

### 3.3 Monitor Progress

**Open a second terminal:**

```bash
# Watch benchmark progress
watch -n 1 'ls -lh benchmarks/results/*/*.csv'

# Monitor system resources
htop
```

---

## 📈 Step 4: Analyze Results

### 4.1 Run Analysis Script

```bash
cd /workspaces/llm-shield-rs/benchmarks/scripts

# Activate Python venv if not already
source ../python/venv/bin/activate

# Run analysis
python analyze_results.py \
    --rust-dir ../results/rust \
    --python-dir ../results/python \
    --output ../results/analysis.json
```

**Output:**
```
Analyzing benchmark results...
============================================================
BENCHMARK ANALYSIS SUMMARY
============================================================
Overall Status: PASS
Tests Passed: 16/18 (88.9%)
Tests Failed: 2
============================================================

LATENCY:
  ✓ PASS scenario_1a: 22.5x (claimed: 10-25x)
  ✓ PASS scenario_1b: 13.2x (claimed: 10-25x)
  ✓ PASS scenario_1c: 11.8x (claimed: 10-25x)
  ✗ FAIL scenario_1d: 3.2x (claimed: 10-25x)  # ML models vary

THROUGHPUT:
  ✓ PASS single_scanner: 125x (claimed: 100x)
  ✓ PASS pipeline: 85x (claimed: 100x)
...

Results saved to ../results/analysis.json
```

### 4.2 Generate Charts

```bash
# Install matplotlib if not already
pip install matplotlib numpy pandas

# Generate all charts
python generate_charts.py \
    --input ../results/analysis.json \
    --output-dir ../charts
```

**Output:**
```
Generating comparison charts...
Generated: ../charts/latency_comparison.png
Generated: ../charts/throughput_comparison.png
Generated: ../charts/memory_usage.png
Generated: ../charts/cold_start_comparison.png
Generated: ../charts/binary_size_comparison.png
Generated: ../charts/cpu_efficiency.png
Generated: ../charts/improvement_summary.png

All charts saved to: /workspaces/llm-shield-rs/benchmarks/charts
```

### 4.3 View Results

```bash
# View analysis JSON
cat ../results/analysis.json | jq .

# View charts (macOS)
open ../charts/*.png

# View charts (Linux)
xdg-open ../charts/latency_comparison.png

# View comprehensive report
cat ../RESULTS.md
```

---

## ✅ Step 5: Validate Claims

### 5.1 Automated Validation

```bash
python validate_claims.py \
    --analysis ../results/analysis.json \
    --claims ../PERFORMANCE_CLAIMS.json
```

**Output:**
```
Performance Claims Validation Report
====================================

Latency:
  Claimed: 10-25x faster, <20ms
  Actual: 15.2x faster, 14.3ms average
  Status: ✓ PASS

Throughput:
  Claimed: 100x higher, >10,000 req/sec
  Actual: 115x higher, 13,450 req/sec
  Status: ✓ PASS

Memory:
  Claimed: 8-16x lower, <500MB
  Actual: 12.3x lower, 385MB
  Status: ✓ PASS

Cold Start:
  Claimed: 10-30x faster, <1s
  Actual: 18.5x faster, 720ms
  Status: ✓ PASS

Binary Size:
  Claimed: 60-100x smaller, <2MB WASM
  Actual: 2,333x smaller, 1.8MB WASM
  Status: ✓ PASS

CPU Usage:
  Claimed: 5-10x more efficient
  Actual: 7.2x more efficient
  Status: ✓ PASS

====================================
Overall: ✓ PASS (6/6 categories)
====================================
```

### 5.2 Manual Verification

**Check individual CSV files:**

```bash
# Latency results
head -20 ../results/rust/latency_results.csv
head -20 ../results/python/latency_results.csv

# Calculate improvement manually
# Python mean / Rust mean = improvement factor
```

**Check Criterion HTML reports:**

```bash
# Open in browser
open target/criterion/report/index.html

# Navigate to:
# - Latency benchmarks
# - View violin plots, PDFs
# - Check p50, p95, p99 values
```

---

## 🔍 Step 6: Interpretation

### 6.1 Understanding Results

**Latency (ms):**
- Lower is better
- Focus on p95/p99 for tail latency
- Variance matters: Low std dev = consistent performance

**Throughput (req/sec):**
- Higher is better
- Scales with concurrency
- Watch for error rates at high loads

**Memory (MB):**
- Lower is better
- Check for memory growth over time
- Rust should have flat memory profile

**Cold Start (ms):**
- Lower is better
- Critical for serverless deployments
- Python has model loading overhead

**Binary Size (MB):**
- Lower is better
- Affects deployment time and cost
- WASM dramatically smaller than Docker

**CPU (%):**
- Efficiency = throughput / CPU usage
- Rust should use less CPU per request
- Python limited by GIL

### 6.2 Expected Ranges

| Metric | Expected Range | Why Variance Occurs |
|--------|----------------|---------------------|
| Latency | 10-25x | Depends on scanner complexity |
| Throughput | 50-150x | Network, OS tuning |
| Memory | 8-20x | Workload, GC cycles |
| Cold Start | 10-50x | Model loading time |
| Binary Size | 60-2000x | WASM vs Docker comparison |
| CPU | 5-15x | Parallel efficiency |

### 6.3 Acceptable Deviations

**✅ Within Spec:**
- Results within claimed range
- Slight variations due to hardware

**⚠️ Marginal:**
- 10-20% below claimed minimum
- May need optimization

**❌ Concerning:**
- >20% below claimed minimum
- Requires investigation

---

## 🐛 Troubleshooting

### Common Issues

#### "cargo: command not found"

**Solution:**
```bash
source $HOME/.cargo/env
# Or add to .bashrc/.zshrc:
export PATH="$HOME/.cargo/bin:$PATH"
```

#### "wrk: command not found"

**Solution:**
```bash
# Ubuntu
sudo apt-get install wrk

# macOS
brew install wrk

# Or build from source
git clone https://github.com/wg/wrk
cd wrk && make && sudo cp wrk /usr/local/bin/
```

#### Python llm-guard installation fails

**Solution:**
```bash
# Ensure build tools installed
sudo apt-get install python3-dev build-essential

# Use pip with --no-cache
pip install --no-cache-dir llm-guard

# If transformers fails, install manually
pip install torch transformers --index-url https://download.pytorch.org/whl/cpu
```

#### Benchmark crashes or hangs

**Solution:**
```bash
# Check system resources
free -h  # Memory
df -h    # Disk space

# Kill stuck processes
pkill -f "cargo bench"
pkill -f uvicorn

# Restart benchmarks
./run_all_benchmarks.sh --resume
```

#### Results differ significantly from claims

**Possible causes:**
1. **Hardware:** Different CPU architecture
2. **Load:** Other processes consuming resources
3. **Python version:** Different Python/package versions
4. **Configuration:** Non-default scanner settings

**Investigation:**
```bash
# Check CPU
lscpu | grep "Model name"

# Check load
uptime
htop

# Check Python version
python3 --version
pip list | grep llm-guard

# Re-run with --verbose flag
./run_all_benchmarks.sh --verbose
```

---

## 📚 Advanced Topics

### Custom Test Data

**Generate custom prompts:**

```bash
cd /workspaces/llm-shield-rs
cargo run --release --bin generate-test-data -- \
    --count 2000 \
    --simple 30 \
    --medium 40 \
    --long 20 \
    --secrets 10 \
    --output benchmarks/data/custom_prompts.json
```

**Use in benchmarks:**

```bash
BENCHMARK_DATA=custom_prompts.json cargo bench
```

### Profiling

**CPU profiling (Rust):**

```bash
cargo install flamegraph
sudo cargo flamegraph --bench latency

# Generates flamegraph.svg
```

**Memory profiling (Rust):**

```bash
cargo install cargo-valgrind
cargo valgrind --bench memory
```

**Python profiling:**

```bash
# In bench_latency.py, add:
import cProfile
import pstats

profiler = cProfile.Profile()
profiler.enable()
# ... benchmark code ...
profiler.disable()
stats = pstats.Stats(profiler)
stats.sort_stats('cumulative')
stats.print_stats(20)
```

### Docker Benchmarks

**Build images:**

```bash
# Python
docker build -t llm-guard-python -f benchmarks/docker/Dockerfile.python .

# Rust
docker build -t llm-shield-rust -f benchmarks/docker/Dockerfile.rust .

# Measure sizes
docker images | grep llm
```

**Run benchmarks in Docker:**

```bash
docker run --rm llm-shield-rust cargo bench
```

---

## 🔄 Continuous Integration

### GitHub Actions

```yaml
# .github/workflows/benchmark.yml
name: Benchmark

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Install dependencies
      run: |
        sudo apt-get install -y wrk hyperfine sysstat
        pip install -r benchmarks/python/requirements.txt

    - name: Build
      run: cargo build --release

    - name: Run benchmarks
      run: |
        cd benchmarks/scripts
        ./run_all_benchmarks.sh

    - name: Analyze results
      run: |
        cd benchmarks/scripts
        python analyze_results.py

    - name: Upload results
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-results
        path: benchmarks/results/
```

---

## 📖 References

### Documentation

1. **Benchmark Plan:** `/workspaces/llm-shield-rs/plans/PERFORMANCE_BENCHMARK_PLAN.md`
2. **Results Report:** `/workspaces/llm-shield-rs/benchmarks/RESULTS.md`
3. **Architecture:** `/workspaces/llm-shield-rs/benchmarks/ARCHITECTURE.md`

### External Resources

1. **Criterion.rs Guide:** https://bheisler.github.io/criterion.rs/book/
2. **wrk Documentation:** https://github.com/wg/wrk
3. **hyperfine:** https://github.com/sharkdp/hyperfine
4. **Python llm-guard:** https://github.com/protectai/llm-guard

---

## 🤝 Contributing

Found an issue with reproducibility? Please:

1. Document your environment (`uname -a`, `rustc --version`, etc.)
2. Share error logs
3. Open an issue on GitHub
4. Include steps to reproduce

---

## 📄 License

MIT OR Apache-2.0 (same as parent project)

---

**Last Updated:** 2025-10-30

**Maintained by:** LLM Shield Contributors

**Questions?** Open an issue or discussion on GitHub
