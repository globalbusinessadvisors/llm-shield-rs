#!/bin/bash
# Binary size measurement script
# Tests: Docker images, Native binary, WASM bundle

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"

echo "Measuring Binary Sizes..."

RESULTS_FILE="$BENCHMARK_ROOT/results/binary_size_results.txt"

echo "Binary Size Measurements" > "$RESULTS_FILE"
echo "========================" >> "$RESULTS_FILE"
echo "" >> "$RESULTS_FILE"

# Measure Python Docker image
echo "Building Python Docker image..."
cd "$BENCHMARK_ROOT/python"
docker build -t python-llm-guard -f Dockerfile.python .

PYTHON_SIZE=$(docker images python-llm-guard --format "{{.Size}}")
echo "Python Docker Image: $PYTHON_SIZE" >> "$RESULTS_FILE"

# Measure Rust Docker image
echo "Building Rust Docker image..."
cd "$PROJECT_ROOT"
docker build -t rust-llm-shield -f Dockerfile .

RUST_SIZE=$(docker images rust-llm-shield --format "{{.Size}}")
echo "Rust Docker Image: $RUST_SIZE" >> "$RESULTS_FILE"

# Measure Rust native binary
echo "Measuring native binary..."
NATIVE_SIZE=$(du -h "$PROJECT_ROOT/target/release/llm-shield" | cut -f1)
echo "Rust Native Binary: $NATIVE_SIZE" >> "$RESULTS_FILE"

# Strip and measure
echo "Stripping binary..."
cp "$PROJECT_ROOT/target/release/llm-shield" "$PROJECT_ROOT/target/release/llm-shield-stripped"
strip "$PROJECT_ROOT/target/release/llm-shield-stripped"
STRIPPED_SIZE=$(du -h "$PROJECT_ROOT/target/release/llm-shield-stripped" | cut -f1)
echo "Rust Stripped Binary: $STRIPPED_SIZE" >> "$RESULTS_FILE"

# Measure WASM
echo "Building WASM bundle..."
cd "$PROJECT_ROOT/crates/llm-shield-wasm"
wasm-pack build --release --target web

WASM_SIZE=$(du -h pkg/llm_shield_wasm_bg.wasm | cut -f1)
echo "WASM Uncompressed: $WASM_SIZE" >> "$RESULTS_FILE"

# Optimize WASM
echo "Optimizing WASM..."
wasm-opt -Oz pkg/llm_shield_wasm_bg.wasm -o pkg/optimized.wasm

WASM_OPT_SIZE=$(du -h pkg/optimized.wasm | cut -f1)
echo "WASM Optimized: $WASM_OPT_SIZE" >> "$RESULTS_FILE"

# Gzip WASM
gzip -k pkg/optimized.wasm
WASM_GZIP_SIZE=$(du -h pkg/optimized.wasm.gz | cut -f1)
echo "WASM Gzipped: $WASM_GZIP_SIZE" >> "$RESULTS_FILE"

echo "" >> "$RESULTS_FILE"
cat "$RESULTS_FILE"

echo "✅ Binary size measurements complete"
