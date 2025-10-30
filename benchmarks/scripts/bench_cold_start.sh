#!/bin/bash
# Cold start benchmark runner using hyperfine
# Tests: App startup, First request, Serverless

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"

echo "Running Cold Start Benchmarks..."

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo "ERROR: hyperfine is not installed. Please install hyperfine first."
    echo "Installation: cargo install hyperfine"
    exit 1
fi

# Build release binaries
cd "$PROJECT_ROOT"
cargo build --release

# Test Python cold start
echo "Testing Python cold start..."
hyperfine \
    --warmup 0 \
    --runs 100 \
    --export-json "$BENCHMARK_ROOT/results/python/cold_start.json" \
    'python3 -c "from llm_guard.input_scanners import BanSubstrings; print(\"Ready\")"'

# Test Rust cold start
echo "Testing Rust cold start..."
hyperfine \
    --warmup 0 \
    --runs 100 \
    --export-json "$BENCHMARK_ROOT/results/rust/cold_start.json" \
    "$PROJECT_ROOT/target/release/llm-shield --test-mode"

echo "✅ Cold start benchmarks complete"
