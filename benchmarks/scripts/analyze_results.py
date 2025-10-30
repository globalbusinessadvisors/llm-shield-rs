#!/usr/bin/env python3
"""
Results Analysis Script for LLM Shield Benchmarks

This script processes benchmark results from both Rust and Python implementations,
calculates improvement factors, and validates performance claims.

Usage:
    python analyze_results.py
    python analyze_results.py --rust-dir results/rust --python-dir results/python
"""

import json
import csv
import sys
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass
import statistics


@dataclass
class BenchmarkResult:
    """Represents a single benchmark result"""
    name: str
    mean: float
    median: float
    p95: float
    p99: float
    std_dev: float
    unit: str


@dataclass
class ComparisonResult:
    """Comparison between Rust and Python implementations"""
    category: str
    scenario: str
    rust_value: float
    python_value: float
    improvement_factor: float
    claimed_min: float
    claimed_max: Optional[float]
    passed: bool
    unit: str


class PerformanceClaims:
    """Defined performance claims from README"""

    LATENCY = {
        'rust_target': 20.0,  # ms
        'python_baseline': (200.0, 500.0),  # ms
        'improvement': (10.0, 25.0),  # x faster
        'unit': 'ms'
    }

    THROUGHPUT = {
        'rust_target': 10000.0,  # req/sec
        'python_baseline': 100.0,  # req/sec
        'improvement': 100.0,  # x higher
        'unit': 'req/sec'
    }

    MEMORY = {
        'rust_target': 500.0,  # MB
        'python_baseline': (4000.0, 8000.0),  # MB
        'improvement': (8.0, 16.0),  # x lower
        'unit': 'MB'
    }

    COLD_START = {
        'rust_target': 1000.0,  # ms
        'python_baseline': (10000.0, 30000.0),  # ms
        'improvement': (10.0, 30.0),  # x faster
        'unit': 'ms'
    }

    BINARY_SIZE = {
        'rust_target': 2.0,  # MB (WASM gzip)
        'python_baseline': (3000.0, 5000.0),  # MB (Docker)
        'improvement': (60.0, 100.0),  # x smaller
        'unit': 'MB'
    }

    CPU_USAGE = {
        'rust_target': None,  # Relative
        'python_baseline': None,  # Relative
        'improvement': (5.0, 10.0),  # x more efficient
        'unit': 'req/sec per core'
    }


class ResultsAnalyzer:
    """Main analyzer for benchmark results"""

    def __init__(self, rust_dir: Path, python_dir: Path):
        self.rust_dir = Path(rust_dir)
        self.python_dir = Path(python_dir)
        self.comparisons: List[ComparisonResult] = []

    def load_criterion_results(self, benchmark_dir: Path, category: str) -> Dict[str, BenchmarkResult]:
        """Load results from Criterion.rs JSON output"""
        results = {}

        # Criterion saves results in target/criterion/<bench_name>/*/estimates.json
        criterion_base = benchmark_dir / "target" / "criterion"

        if not criterion_base.exists():
            print(f"Warning: Criterion results not found at {criterion_base}")
            return results

        for bench_dir in criterion_base.iterdir():
            if not bench_dir.is_dir():
                continue

            # Look for estimates.json in subdirectories
            for subdir in bench_dir.iterdir():
                if not subdir.is_dir():
                    continue

                estimates_file = subdir / "base" / "estimates.json"
                if estimates_file.exists():
                    try:
                        with open(estimates_file) as f:
                            data = json.load(f)

                        # Extract mean estimate (in nanoseconds)
                        mean_ns = data.get('mean', {}).get('point_estimate', 0)
                        median_ns = data.get('median', {}).get('point_estimate', 0)
                        std_dev_ns = data.get('std_dev', {}).get('point_estimate', 0)

                        # Convert to appropriate units based on category
                        unit_factor, unit = self._get_unit_conversion(category)

                        results[bench_dir.name] = BenchmarkResult(
                            name=bench_dir.name,
                            mean=mean_ns / unit_factor,
                            median=median_ns / unit_factor,
                            p95=0,  # Would need to calculate from raw data
                            p99=0,
                            std_dev=std_dev_ns / unit_factor,
                            unit=unit
                        )
                    except Exception as e:
                        print(f"Error loading {estimates_file}: {e}")

        return results

    def load_csv_results(self, csv_file: Path) -> List[BenchmarkResult]:
        """Load results from CSV file"""
        results = []

        if not csv_file.exists():
            print(f"Warning: CSV file not found: {csv_file}")
            return results

        try:
            with open(csv_file) as f:
                reader = csv.DictReader(f)
                for row in reader:
                    results.append(BenchmarkResult(
                        name=row.get('name', ''),
                        mean=float(row.get('mean', 0)),
                        median=float(row.get('median', 0)),
                        p95=float(row.get('p95', 0)),
                        p99=float(row.get('p99', 0)),
                        std_dev=float(row.get('std_dev', 0)),
                        unit=row.get('unit', 'unknown')
                    ))
        except Exception as e:
            print(f"Error loading CSV {csv_file}: {e}")

        return results

    def _get_unit_conversion(self, category: str) -> Tuple[float, str]:
        """Get conversion factor from nanoseconds to appropriate unit"""
        conversions = {
            'latency': (1_000_000, 'ms'),  # ns to ms
            'throughput': (1, 'req/sec'),  # Already in req/sec
            'memory': (1, 'MB'),  # Already in MB
            'cold_start': (1_000_000, 'ms'),  # ns to ms
            'binary_size': (1, 'MB'),  # Already in MB
            'cpu_usage': (1, 'req/sec per core')  # Already in proper unit
        }
        return conversions.get(category, (1, 'unknown'))

    def analyze_latency(self) -> List[ComparisonResult]:
        """Analyze latency benchmarks"""
        results = []
        claims = PerformanceClaims.LATENCY

        # Try to load Rust results
        rust_results = self.load_criterion_results(self.rust_dir.parent.parent, 'latency')

        # Try to load Python results
        python_csv = self.python_dir / 'latency_results.csv'
        python_results = self.load_csv_results(python_csv)

        # Compare results
        for rust_bench_name, rust_result in rust_results.items():
            # Find matching Python result
            python_result = None
            for py_res in python_results:
                if py_res.name == rust_bench_name:
                    python_result = py_res
                    break

            if python_result:
                improvement = python_result.mean / rust_result.mean

                # Check if passes claim
                min_improvement, max_improvement = claims['improvement']
                passed = min_improvement <= improvement <= max_improvement

                results.append(ComparisonResult(
                    category='Latency',
                    scenario=rust_bench_name,
                    rust_value=rust_result.mean,
                    python_value=python_result.mean,
                    improvement_factor=improvement,
                    claimed_min=min_improvement,
                    claimed_max=max_improvement,
                    passed=passed,
                    unit=claims['unit']
                ))

        return results

    def analyze_throughput(self) -> List[ComparisonResult]:
        """Analyze throughput benchmarks"""
        results = []
        claims = PerformanceClaims.THROUGHPUT

        rust_csv = self.rust_dir / 'throughput_results.csv'
        python_csv = self.python_dir / 'throughput_results.csv'

        rust_results = self.load_csv_results(rust_csv)
        python_results = self.load_csv_results(python_csv)

        for rust_res in rust_results:
            python_res = next((p for p in python_results if p.name == rust_res.name), None)

            if python_res:
                improvement = rust_res.mean / python_res.mean
                passed = improvement >= claims['improvement']

                results.append(ComparisonResult(
                    category='Throughput',
                    scenario=rust_res.name,
                    rust_value=rust_res.mean,
                    python_value=python_res.mean,
                    improvement_factor=improvement,
                    claimed_min=claims['improvement'],
                    claimed_max=None,
                    passed=passed,
                    unit=claims['unit']
                ))

        return results

    def analyze_memory(self) -> List[ComparisonResult]:
        """Analyze memory usage benchmarks"""
        results = []
        claims = PerformanceClaims.MEMORY

        rust_csv = self.rust_dir / 'memory_results.csv'
        python_csv = self.python_dir / 'memory_results.csv'

        rust_results = self.load_csv_results(rust_csv)
        python_results = self.load_csv_results(python_csv)

        for rust_res in rust_results:
            python_res = next((p for p in python_results if p.name == rust_res.name), None)

            if python_res:
                improvement = python_res.mean / rust_res.mean  # Lower is better
                min_improvement, max_improvement = claims['improvement']
                passed = min_improvement <= improvement <= max_improvement

                results.append(ComparisonResult(
                    category='Memory',
                    scenario=rust_res.name,
                    rust_value=rust_res.mean,
                    python_value=python_res.mean,
                    improvement_factor=improvement,
                    claimed_min=min_improvement,
                    claimed_max=max_improvement,
                    passed=passed,
                    unit=claims['unit']
                ))

        return results

    def analyze_all(self) -> Dict[str, List[ComparisonResult]]:
        """Run all analysis categories"""
        results = {
            'latency': self.analyze_latency(),
            'throughput': self.analyze_throughput(),
            'memory': self.analyze_memory(),
            # Add other categories as needed
        }

        self.comparisons = sum(results.values(), [])
        return results

    def generate_summary(self) -> Dict:
        """Generate executive summary"""
        total_tests = len(self.comparisons)
        passed_tests = sum(1 for c in self.comparisons if c.passed)
        failed_tests = total_tests - passed_tests

        overall_status = "PASS" if failed_tests == 0 else "PARTIAL" if passed_tests > 0 else "FAIL"

        return {
            'overall_status': overall_status,
            'total_tests': total_tests,
            'passed': passed_tests,
            'failed': failed_tests,
            'pass_rate': f"{(passed_tests / total_tests * 100):.1f}%" if total_tests > 0 else "N/A"
        }

    def save_results(self, output_file: Path):
        """Save analysis results to JSON"""
        data = {
            'summary': self.generate_summary(),
            'comparisons': [
                {
                    'category': c.category,
                    'scenario': c.scenario,
                    'rust_value': c.rust_value,
                    'python_value': c.python_value,
                    'improvement_factor': c.improvement_factor,
                    'claimed_min': c.claimed_min,
                    'claimed_max': c.claimed_max,
                    'passed': c.passed,
                    'unit': c.unit
                }
                for c in self.comparisons
            ]
        }

        with open(output_file, 'w') as f:
            json.dump(data, f, indent=2)

        print(f"Results saved to {output_file}")


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description='Analyze LLM Shield benchmark results')
    parser.add_argument('--rust-dir', type=str, default='../results/rust',
                       help='Directory containing Rust benchmark results')
    parser.add_argument('--python-dir', type=str, default='../results/python',
                       help='Directory containing Python benchmark results')
    parser.add_argument('--output', type=str, default='../results/analysis.json',
                       help='Output file for analysis results')

    args = parser.parse_args()

    # Create analyzer
    analyzer = ResultsAnalyzer(
        rust_dir=Path(args.rust_dir),
        python_dir=Path(args.python_dir)
    )

    # Run analysis
    print("Analyzing benchmark results...")
    results = analyzer.analyze_all()

    # Print summary
    summary = analyzer.generate_summary()
    print("\n" + "=" * 60)
    print("BENCHMARK ANALYSIS SUMMARY")
    print("=" * 60)
    print(f"Overall Status: {summary['overall_status']}")
    print(f"Tests Passed: {summary['passed']}/{summary['total_tests']} ({summary['pass_rate']})")
    print(f"Tests Failed: {summary['failed']}")
    print("=" * 60)

    # Print category breakdown
    for category, comparisons in results.items():
        if comparisons:
            print(f"\n{category.upper()}:")
            for comp in comparisons:
                status = "✓ PASS" if comp.passed else "✗ FAIL"
                print(f"  {status} {comp.scenario}: {comp.improvement_factor:.1f}x "
                      f"(claimed: {comp.claimed_min:.0f}-{comp.claimed_max or comp.claimed_min:.0f}x)")

    # Save results
    analyzer.save_results(Path(args.output))


if __name__ == '__main__':
    main()
