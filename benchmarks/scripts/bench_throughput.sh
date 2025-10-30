#!/bin/bash
# Comprehensive Throughput Benchmark Runner
# Tests: Single scanner, Pipeline, Multiple concurrency levels
# Validates: 100x improvement claim (>10,000 req/sec for Rust)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"
RESULTS_DIR="$BENCHMARK_ROOT/results"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "LLM Shield Throughput Benchmarks"
echo "=========================================="
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR/rust"
mkdir -p "$RESULTS_DIR/python"

# Function to check if command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Function to wait for server to be ready
wait_for_server() {
    local url=$1
    local max_attempts=30
    local attempt=0

    echo "Waiting for server to be ready at $url..."

    while [ $attempt -lt $max_attempts ]; do
        if curl -s -f "$url/health" > /dev/null 2>&1; then
            echo "✅ Server is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    echo "❌ Server failed to start after $max_attempts seconds"
    return 1
}

# Function to stop background server
stop_server() {
    local pid=$1
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        echo "Stopping server (PID: $pid)..."
        kill "$pid" 2>/dev/null || true
        sleep 2
        kill -9 "$pid" 2>/dev/null || true
    fi
}

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up..."
    stop_server "$RUST_SERVER_PID"
    stop_server "$PYTHON_SERVER_PID"
}

trap cleanup EXIT

echo "================================================"
echo "Step 1: Building Rust Benchmark Infrastructure"
echo "================================================"
echo ""

cd "$PROJECT_ROOT"

echo "Building benchmark server..."
cargo build --release --bin bench-server

echo "Building load test tool..."
cargo build --release --bin throughput-load-test

echo "✅ Build complete"
echo ""

echo "================================================"
echo "Step 2: Testing Rust Throughput"
echo "================================================"
echo ""

# Start Rust server
echo "Starting Rust benchmark server..."
"$PROJECT_ROOT/target/release/bench-server" > "$RESULTS_DIR/rust/server.log" 2>&1 &
RUST_SERVER_PID=$!

if ! wait_for_server "http://localhost:3000"; then
    echo "❌ Failed to start Rust server"
    cat "$RESULTS_DIR/rust/server.log"
    exit 1
fi

echo ""
echo "--- Method 1: Using Rust Load Test Tool ---"
echo ""

# Run Rust-based load test
BENCH_URL="http://localhost:3000" "$PROJECT_ROOT/target/release/throughput-load-test"

echo ""
echo "✅ Rust load test complete"
echo ""

# Check if wrk is available for additional testing
if command_exists wrk; then
    echo "--- Method 2: Using wrk (HTTP Load Tester) ---"
    echo ""

    # Create wrk script for POST requests
    WRK_SCRIPT="$RESULTS_DIR/rust/wrk_post.lua"
    cat > "$WRK_SCRIPT" <<'EOF'
wrk.method = "POST"
wrk.headers["Content-Type"] = "application/json"
wrk.body = '{"text":"This is a test prompt for benchmarking"}'

done = function(summary, latency, requests)
    io.write("----------------------------------------\n")
    io.write(string.format("Requests/sec: %.2f\n", summary.requests / summary.duration * 1e6))
    io.write(string.format("Latency p50: %.2f ms\n", latency:percentile(50.0) / 1000))
    io.write(string.format("Latency p95: %.2f ms\n", latency:percentile(95.0) / 1000))
    io.write(string.format("Latency p99: %.2f ms\n", latency:percentile(99.0) / 1000))
    io.write("----------------------------------------\n")
end
EOF

    echo "Testing single scanner with varying concurrency..."
    echo ""

    for CONCURRENCY in 10 50 100 500; do
        echo "Concurrency: $CONCURRENCY connections"
        wrk -t4 -c"$CONCURRENCY" -d30s --latency -s "$WRK_SCRIPT" http://localhost:3000/scan \
            > "$RESULTS_DIR/rust/wrk_c${CONCURRENCY}.txt" 2>&1

        echo "Results saved to: $RESULTS_DIR/rust/wrk_c${CONCURRENCY}.txt"
        echo ""
        sleep 2
    done

    echo "Testing pipeline endpoint..."
    wrk -t4 -c100 -d30s --latency -s "$WRK_SCRIPT" http://localhost:3000/scan/pipeline \
        > "$RESULTS_DIR/rust/wrk_pipeline.txt" 2>&1

    echo "Pipeline results saved to: $RESULTS_DIR/rust/wrk_pipeline.txt"
    echo ""

    echo "✅ wrk tests complete"
else
    echo "ℹ️  wrk not found - skipping wrk-based tests"
    echo "   Install with: sudo apt-get install wrk (Ubuntu) or brew install wrk (macOS)"
fi

echo ""

# Get final metrics from server
echo "--- Fetching Server Metrics ---"
curl -s http://localhost:3000/metrics | jq '.' > "$RESULTS_DIR/rust/server_metrics.json"
echo "Server metrics saved to: $RESULTS_DIR/rust/server_metrics.json"
echo ""

# Stop Rust server
stop_server "$RUST_SERVER_PID"
RUST_SERVER_PID=""

echo "================================================"
echo "Step 3: Testing Python Baseline (Optional)"
echo "================================================"
echo ""

# Check if Python benchmark is available
if [ -f "$BENCHMARK_ROOT/python/bench_throughput_server.py" ]; then
    echo "Starting Python benchmark server..."

    cd "$BENCHMARK_ROOT/python"

    # Check if virtual environment exists
    if [ -d "venv" ]; then
        source venv/bin/activate
    fi

    # Check if llm-guard is installed
    if ! python3 -c "import llm_guard" 2>/dev/null; then
        echo "⚠️  llm-guard not installed. Skipping Python baseline."
        echo "   Install with: pip install llm-guard"
    else
        python3 bench_throughput_server.py > "$RESULTS_DIR/python/server.log" 2>&1 &
        PYTHON_SERVER_PID=$!

        if wait_for_server "http://localhost:8000"; then
            echo ""
            echo "Running Python load test..."

            # Use the same load test tool, just with different URL
            BENCH_URL="http://localhost:8000" "$PROJECT_ROOT/target/release/throughput-load-test" \
                > "$RESULTS_DIR/python/load_test_output.txt" 2>&1

            mv "$RESULTS_DIR/throughput_results.csv" "$RESULTS_DIR/python/throughput_results.csv" 2>/dev/null || true

            echo "✅ Python load test complete"

            stop_server "$PYTHON_SERVER_PID"
            PYTHON_SERVER_PID=""
        else
            echo "⚠️  Python server failed to start"
            stop_server "$PYTHON_SERVER_PID"
            PYTHON_SERVER_PID=""
        fi
    fi
else
    echo "ℹ️  Python benchmark server not found at:"
    echo "   $BENCHMARK_ROOT/python/bench_throughput_server.py"
    echo "   Skipping Python baseline comparison"
fi

echo ""
echo "================================================"
echo "Step 4: Analysis and Validation"
echo "================================================"
echo ""

# Parse and display results
echo "--- Rust Throughput Results ---"
echo ""

if [ -f "$RESULTS_DIR/throughput_results.csv" ]; then
    echo "Reading results from: $RESULTS_DIR/throughput_results.csv"
    echo ""

    # Display in table format
    column -t -s',' "$RESULTS_DIR/throughput_results.csv" | head -n 20

    echo ""
    echo "--- Performance Summary ---"

    # Extract max req/sec for single scanner
    MAX_RPS=$(awk -F',' 'NR>1 && /\/scan[^\/]/ {if($7>max) max=$7} END {print max}' "$RESULTS_DIR/throughput_results.csv")

    if [ -n "$MAX_RPS" ]; then
        echo "Maximum throughput (single scanner): ${MAX_RPS} req/sec"
        echo ""

        # Validation
        TARGET=10000
        if (( $(echo "$MAX_RPS >= $TARGET" | bc -l) )); then
            echo -e "${GREEN}✅ PASS${NC}: Achieved ${MAX_RPS} req/sec (target: ${TARGET})"
        else
            PERCENTAGE=$(echo "scale=1; $MAX_RPS / $TARGET * 100" | bc)
            echo -e "${YELLOW}⚠️  WARNING${NC}: Achieved ${MAX_RPS} req/sec (${PERCENTAGE}% of target: ${TARGET})"
            echo "   Note: Results may vary based on hardware, system load, and configuration"
        fi
    else
        echo "⚠️  Could not extract throughput data"
    fi
else
    echo "⚠️  Results file not found: $RESULTS_DIR/throughput_results.csv"
fi

echo ""
echo "================================================"
echo "Throughput Benchmarks Complete"
echo "================================================"
echo ""
echo "Results saved to: $RESULTS_DIR/"
echo ""
echo "Files generated:"
echo "  - throughput_results.csv          : Detailed results in CSV format"
echo "  - rust/server_metrics.json        : Server-side metrics"
echo "  - rust/server.log                 : Server logs"
if command_exists wrk; then
    echo "  - rust/wrk_c*.txt                 : wrk test results for various concurrency levels"
    echo "  - rust/wrk_pipeline.txt           : Pipeline throughput results"
fi
echo ""

if [ -f "$RESULTS_DIR/python/throughput_results.csv" ]; then
    echo "Python baseline results available for comparison:"
    echo "  - python/throughput_results.csv"
    echo ""
fi

echo "Next steps:"
echo "  1. Review CSV files for detailed metrics"
echo "  2. Compare Rust vs Python results (if available)"
echo "  3. Validate against README claims"
echo "  4. Generate visualization charts (optional)"
echo ""
