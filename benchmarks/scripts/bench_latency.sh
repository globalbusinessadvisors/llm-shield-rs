#!/bin/bash
# Latency benchmark runner
# Tests: BanSubstrings, Regex, Secrets, PromptInjection

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"

echo "Running Latency Benchmarks..."

# Run Rust benchmarks using criterion
cd "$PROJECT_ROOT"
cargo bench --bench latency -- --save-baseline rust_latency

# Copy criterion results to benchmarks/results
echo "Copying Rust results..."
mkdir -p "$BENCHMARK_ROOT/results/rust"
cp -r target/criterion "$BENCHMARK_ROOT/results/rust/latency_criterion"

# Run Python benchmarks
echo "Running Python benchmarks..."
cd "$BENCHMARK_ROOT/python"
python3 bench_latency.py --output "$BENCHMARK_ROOT/results/python/latency_results.csv"

echo "✅ Latency benchmarks complete"
