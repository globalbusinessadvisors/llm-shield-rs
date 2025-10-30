#!/bin/bash
# CPU usage benchmark runner using pidstat
# Tests: Single request, Sustained load, Efficiency

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BENCHMARK_ROOT="$(dirname "$SCRIPT_DIR")"
PROJECT_ROOT="$(dirname "$BENCHMARK_ROOT")"

echo "Running CPU Usage Benchmarks..."

# Check if pidstat is installed
if ! command -v pidstat &> /dev/null; then
    echo "ERROR: pidstat is not installed. Please install sysstat first."
    echo "Ubuntu: sudo apt-get install sysstat"
    echo "macOS: Not available, using alternative..."
fi

# Test Python CPU usage
echo "Testing Python CPU usage..."
cd "$BENCHMARK_ROOT/python"
python3 bench_throughput.py &
PYTHON_PID=$!
sleep 5

pidstat -p $PYTHON_PID 1 60 > "$BENCHMARK_ROOT/results/python/cpu_usage.txt"

kill $PYTHON_PID

# Test Rust CPU usage
echo "Testing Rust CPU usage..."
"$PROJECT_ROOT/target/release/bench-server" &
RUST_PID=$!
sleep 2

pidstat -p $RUST_PID 1 60 > "$BENCHMARK_ROOT/results/rust/cpu_usage.txt"

kill $RUST_PID

echo "✅ CPU usage benchmarks complete"
