#!/bin/bash
# Memory usage benchmark runner
# Tests: Baseline, Under load, Growth over time

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"

echo "Running Memory Benchmarks..."

# Test Python memory
echo "Testing Python memory usage..."
cd "$BENCHMARK_ROOT/python"
python3 bench_memory.py \
    --duration 3600 \
    --interval 10 \
    --output "$BENCHMARK_ROOT/results/python/memory_results.csv"

# Test Rust memory
echo "Testing Rust memory usage..."
cd "$PROJECT_ROOT"
cargo run --release --bin bench-memory -- \
    --duration 3600 \
    --interval 10 \
    --output "$BENCHMARK_ROOT/results/rust/memory_results.csv"

echo "✅ Memory benchmarks complete"
