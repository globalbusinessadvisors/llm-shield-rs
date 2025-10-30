#!/bin/bash
# Master benchmark runner script
# Executes all LLM Shield performance benchmarks

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"

echo "=================================================="
echo "LLM Shield Performance Benchmark Suite"
echo "=================================================="
echo ""
echo "Benchmark Root: $BENCHMARK_ROOT"
echo "Project Root: $PROJECT_ROOT"
echo "Start Time: $(date)"
echo ""

# Create results and charts directories
mkdir -p "$BENCHMARK_ROOT/results"
mkdir -p "$BENCHMARK_ROOT/charts"
mkdir -p "$BENCHMARK_ROOT/data"

# Phase 1: Generate test data
echo "=================================================="
echo "Phase 1: Generating Test Data"
echo "=================================================="

if [ ! -f "$BENCHMARK_ROOT/data/test_prompts.json" ]; then
    echo "Generating 1000 test prompts..."
    cd "$PROJECT_ROOT"
    cargo run --release --bin generate-test-data -- \
        --count 1000 \
        --output "$BENCHMARK_ROOT/data/test_prompts.json"
else
    echo "Test data already exists, skipping generation."
fi

echo ""

# Phase 2: Latency Benchmarks
echo "=================================================="
echo "Phase 2: Running Latency Benchmarks"
echo "=================================================="
bash "$SCRIPT_DIR/bench_latency.sh"
echo ""

# Phase 3: Throughput Benchmarks
echo "=================================================="
echo "Phase 3: Running Throughput Benchmarks"
echo "=================================================="
bash "$SCRIPT_DIR/bench_throughput.sh"
echo ""

# Phase 4: Memory Benchmarks
echo "=================================================="
echo "Phase 4: Running Memory Benchmarks"
echo "=================================================="
bash "$SCRIPT_DIR/bench_memory.sh"
echo ""

# Phase 5: Cold Start Benchmarks
echo "=================================================="
echo "Phase 5: Running Cold Start Benchmarks"
echo "=================================================="
bash "$SCRIPT_DIR/bench_cold_start.sh"
echo ""

# Phase 6: Binary Size Measurements
echo "=================================================="
echo "Phase 6: Measuring Binary Sizes"
echo "=================================================="
bash "$SCRIPT_DIR/bench_binary_size.sh"
echo ""

# Phase 7: CPU Usage Benchmarks
echo "=================================================="
echo "Phase 7: Running CPU Usage Benchmarks"
echo "=================================================="
bash "$SCRIPT_DIR/bench_cpu.sh"
echo ""

# Phase 8: Analysis & Reporting
echo "=================================================="
echo "Phase 8: Analyzing Results & Generating Reports"
echo "=================================================="

cd "$BENCHMARK_ROOT/analysis"
python3 analyze_results.py
python3 generate_charts.py
python3 validate_claims.py

echo ""
echo "=================================================="
echo "Benchmark Suite Complete!"
echo "=================================================="
echo "End Time: $(date)"
echo ""
echo "Results saved to: $BENCHMARK_ROOT/results/"
echo "Charts saved to: $BENCHMARK_ROOT/charts/"
echo ""
echo "View the final report: $BENCHMARK_ROOT/results/RESULTS.md"
echo ""

# Check validation status
if grep -q "✅ PASS" "$BENCHMARK_ROOT/results/RESULTS.md"; then
    echo "✅ All performance claims validated!"
    exit 0
else
    echo "❌ Some performance claims not validated. See RESULTS.md for details."
    exit 1
fi
