#!/bin/bash
# Throughput benchmark runner using wrk
# Tests: Single scanner, Pipeline

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"

echo "Running Throughput Benchmarks..."

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "ERROR: wrk is not installed. Please install wrk first."
    echo "Ubuntu: sudo apt-get install wrk"
    echo "macOS: brew install wrk"
    exit 1
fi

# Build Rust server
echo "Building Rust benchmark server..."
cd "$PROJECT_ROOT"
cargo build --release --bin bench-server

# Test Python throughput
echo "Testing Python throughput..."
cd "$BENCHMARK_ROOT/python"
python3 bench_throughput.py &
PYTHON_PID=$!
sleep 5

wrk -t4 -c100 -d60s --latency http://localhost:8000/scan \
    > "$BENCHMARK_ROOT/results/python/throughput_results.txt"

kill $PYTHON_PID

# Test Rust throughput
echo "Testing Rust throughput..."
"$PROJECT_ROOT/target/release/bench-server" &
RUST_PID=$!
sleep 2

wrk -t4 -c100 -d60s --latency http://localhost:3000/scan \
    > "$BENCHMARK_ROOT/results/rust/throughput_results.txt"

kill $RUST_PID

echo "✅ Throughput benchmarks complete"
