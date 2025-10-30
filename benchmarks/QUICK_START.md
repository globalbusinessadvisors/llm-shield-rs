# 🚀 LLM Shield Benchmarks - Quick Start

**One-page guide to running and analyzing benchmarks**

---

## ⚡ TL;DR - 3 Commands

```bash
# 1. Run benchmarks (2-4 hours, automated)
cd /workspaces/llm-shield-rs/benchmarks/scripts
./run_all_benchmarks.sh

# 2. Analyze results (2 minutes)
python analyze_results.py && python generate_charts.py

# 3. Validate claims (30 seconds)
python validate_claims.py --readme-updates
```

**Done!** Check `results/` and `charts/` directories for output.

---

## 📁 File Organization

### Documents You Need
| File | Purpose | When to Read |
|------|---------|--------------|
| **QUICK_START.md** | This file | First |
| **REPRODUCIBILITY.md** | Full setup guide | Before running |
| **RESULTS.md** | Final report | After execution |

### Scripts You'll Use
| Script | Purpose | Runtime |
|--------|---------|---------|
| `run_all_benchmarks.sh` | Run everything | 2-4 hours |
| `analyze_results.py` | Process results | 2 minutes |
| `generate_charts.py` | Create visuals | 1 minute |
| `validate_claims.py` | Check claims | 30 seconds |

### Output Locations
```
benchmarks/
├── results/
│   ├── rust/*.csv              # Rust benchmark data
│   ├── python/*.csv            # Python baseline data
│   ├── analysis.json           # Processed results
│   └── validation.json         # Pass/fail report
│
└── charts/
    ├── latency_comparison.png  # 7 comparison charts
    └── ... (6 more)
```

---

## 🔧 Prerequisites Checklist

**Required:**
- [ ] Rust 1.75+ (`cargo --version`)
- [ ] Python 3.11+ (`python3.11 --version`)
- [ ] wrk (`wrk --version`)
- [ ] hyperfine (`hyperfine --version`)

**Install missing tools:**
```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ubuntu/Debian
sudo apt-get install wrk sysstat python3.11 python3-pip

# macOS
brew install wrk hyperfine python@3.11

# Python dependencies
cd benchmarks/python && pip install -r requirements.txt
```

---

## 📊 What Gets Benchmarked

### 6 Categories, 18 Scenarios

1. **Latency** (4 scenarios)
   - String matching, regex, secrets, ML inference
   - **Claim:** 10-25x faster, <20ms

2. **Throughput** (2 scenarios)
   - Single scanner, pipeline
   - **Claim:** 100x higher, >10,000 req/sec

3. **Memory** (3 scenarios)
   - Baseline, under load, stability
   - **Claim:** 8-16x lower, <500MB

4. **Cold Start** (3 scenarios)
   - Startup, first request, serverless
   - **Claim:** 10-30x faster, <1s

5. **Binary Size** (3 measurements)
   - Docker, native, WASM
   - **Claim:** 60-100x smaller, <2MB WASM

6. **CPU Usage** (3 scenarios)
   - Single request, sustained load, efficiency
   - **Claim:** 5-10x more efficient

---

## 🎯 Execution Options

### Option 1: Full Automated (Recommended)

```bash
cd /workspaces/llm-shield-rs/benchmarks/scripts
./run_all_benchmarks.sh

# Runs:
# 1. All Rust benchmarks (cargo bench)
# 2. All Python baselines
# 3. Collects results
# 4. Shows summary

# Time: 2-4 hours (unattended)
```

### Option 2: Category by Category

```bash
# Latency (15 min)
./bench_latency.sh

# Throughput (30 min)
./bench_throughput.sh

# Memory (45 min)
./bench_memory.sh

# Cold Start (20 min)
./bench_cold_start.sh

# Binary Size (5 min)
./bench_binary_size.sh

# CPU (30 min)
./bench_cpu.sh
```

### Option 3: Rust Only (Fast)

```bash
cd /workspaces/llm-shield-rs
cargo bench

# Time: 30 minutes
# Output: target/criterion/
# Note: No Python comparison
```

### Option 4: Quick Test

```bash
./bench_latency.sh --quick

# Time: 2 minutes
# Runs subset of tests
# Good for smoke testing
```

---

## 📈 Analysis Workflow

### Step 1: Process Results

```bash
python analyze_results.py \
    --rust-dir ../results/rust \
    --python-dir ../results/python \
    --output ../results/analysis.json
```

**Output:**
- `analysis.json` - Structured data
- Console summary with pass/fail

**What it does:**
- Loads Rust + Python results
- Calculates improvement factors
- Compares to claimed values
- Generates pass/fail status

### Step 2: Generate Charts

```bash
python generate_charts.py \
    --input ../results/analysis.json \
    --output-dir ../charts
```

**Output:**
- 7 PNG charts (300 DPI)
- Professional styling
- Ready for presentations

**Charts created:**
1. Latency comparison (bar chart)
2. Throughput comparison (line chart)
3. Memory usage (stacked bars)
4. Cold start times (log scale)
5. Binary size (log scale)
6. CPU efficiency (bars)
7. Overall summary (claimed vs actual)

### Step 3: Validate Claims

```bash
python validate_claims.py \
    --analysis ../results/analysis.json \
    --readme-updates
```

**Output:**
- Pass/fail report (console + JSON)
- README update suggestions
- Optimization recommendations

**Pass criteria:**
- Actual improvement within claimed range (±10%)
- Target values met (e.g., <20ms, >10,000 req/sec)

---

## 🔍 Interpreting Results

### Example Output

```
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
  ✗ FAIL scenario_1d: 3.2x (claimed: 10-25x)  # ML varies

THROUGHPUT:
  ✓ PASS single_scanner: 125x (claimed: 100x)
  ...
```

### Status Indicators

- ✅ **PASS** - Meets all criteria (improvement + target)
- ⚠️ **PARTIAL** - Meets some criteria
- ❌ **FAIL** - Below expectations
- ⏳ **PENDING** - Not yet executed

### Key Metrics

- **Mean:** Average performance
- **Median (p50):** Middle value (less affected by outliers)
- **P95:** 95th percentile (worst case for most users)
- **P99:** 99th percentile (tail latency)

**Focus on:** Median and P95 for realistic expectations

---

## 🐛 Common Issues

### "cargo: command not found"
```bash
source $HOME/.cargo/env
# Or install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### "wrk: command not found"
```bash
sudo apt-get install wrk  # Ubuntu
brew install wrk          # macOS
```

### "No module named 'llm_guard'"
```bash
cd benchmarks/python
pip install -r requirements.txt
```

### Benchmarks hang or crash
```bash
# Check resources
free -h  # Memory
df -h    # Disk

# Kill stuck processes
pkill -f "cargo bench"
pkill -f uvicorn

# Resume
./run_all_benchmarks.sh --resume
```

### Results differ from claims
**Common causes:**
- Different CPU (check: `lscpu | grep "Model name"`)
- Other processes running (check: `htop`)
- Different Python/Rust versions
- Non-default configuration

**Solution:** Re-run with verbose logging:
```bash
./run_all_benchmarks.sh --verbose
```

---

## 📚 Need More Details?

### Detailed Guides

1. **Full setup:** Read `REPRODUCIBILITY.md`
   - Step-by-step installation
   - Environment verification
   - Troubleshooting guide
   - Advanced topics

2. **Understanding results:** Read `RESULTS.md`
   - Detailed methodology
   - Statistical analysis
   - Interpretation guide
   - Limitations

3. **Architecture:** Read `ARCHITECTURE.md`
   - System design
   - Component overview
   - Data flow

### Script Help

```bash
# All scripts have --help
python analyze_results.py --help
python generate_charts.py --help
python validate_claims.py --help
```

---

## ✅ Success Checklist

After running benchmarks, you should have:

### Files
- [ ] `results/rust/*.csv` (6 files)
- [ ] `results/python/*.csv` (3-6 files)
- [ ] `results/analysis.json`
- [ ] `results/validation.json`
- [ ] `charts/*.png` (7 files)

### Console Output
- [ ] Benchmark summaries (p50, p95, p99)
- [ ] Pass/fail status per category
- [ ] Overall validation result
- [ ] README update suggestions (if any)

### Next Steps
- [ ] Review charts visually
- [ ] Check validation.json for failures
- [ ] Read README_UPDATES.md (if generated)
- [ ] Update README.md if claims need adjustment
- [ ] Commit results to repository

---

## 🎓 Key Commands Reference

```bash
# BUILD
cargo build --release                    # Build Rust
cargo bench --no-run                     # Build benchmarks

# EXECUTE
./run_all_benchmarks.sh                  # All categories
./bench_latency.sh                       # One category
cargo bench --bench latency              # Rust only

# ANALYZE
python analyze_results.py                # Process data
python generate_charts.py                # Create charts
python validate_claims.py                # Check claims

# VIEW
cat ../results/analysis.json | jq .      # JSON output
open ../charts/*.png                     # Charts (macOS)
cat ../RESULTS.md                        # Full report
```

---

## 📊 Expected Timeline

| Phase | Duration | Automated? |
|-------|----------|------------|
| Setup | 30-60 min | Manual |
| Build | 10-15 min | Automated |
| Benchmarks | 2-4 hours | ✅ Automated |
| Analysis | 2-3 min | ✅ Automated |
| Charts | 1 min | ✅ Automated |
| Validation | 30 sec | ✅ Automated |
| Review | 15-30 min | Manual |

**Total:** ~3-5 hours (mostly unattended)

---

## 🏆 Performance Targets

Quick reference for validation:

| Metric | Target | Improvement |
|--------|--------|-------------|
| Latency | <20ms | 10-25x |
| Throughput | >10,000/s | 100x |
| Memory | <500MB | 8-16x |
| Cold Start | <1s | 10-30x |
| Binary (WASM) | <2MB | 60-100x |
| CPU Efficiency | - | 5-10x |

---

## 🔗 Quick Links

- **Main README:** `/workspaces/llm-shield-rs/README.md`
- **Benchmark Plan:** `/workspaces/llm-shield-rs/plans/PERFORMANCE_BENCHMARK_PLAN.md`
- **Phase 4 Summary:** `/workspaces/llm-shield-rs/benchmarks/PHASE_4_COMPLETE.md`
- **This Guide:** `/workspaces/llm-shield-rs/benchmarks/QUICK_START.md`

---

**Last Updated:** 2025-10-30

**Questions?** See `REPRODUCIBILITY.md` for detailed troubleshooting

**Ready to start?** Run: `./run_all_benchmarks.sh`
