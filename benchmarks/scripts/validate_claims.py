#!/usr/bin/env python3
"""
Performance Claims Validation Script

Validates actual benchmark results against claimed improvements in README.md

Usage:
    python validate_claims.py
    python validate_claims.py --analysis ../results/analysis.json
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from dataclasses import dataclass


@dataclass
class Claim:
    """Represents a performance claim"""
    category: str
    metric: str
    min_improvement: float
    max_improvement: float or None
    rust_target: float or None
    python_baseline: Tuple[float, float] or float or None
    unit: str


# Performance claims from README.md
PERFORMANCE_CLAIMS = {
    'latency': Claim(
        category='Latency',
        metric='End-to-end scan time',
        min_improvement=10.0,
        max_improvement=25.0,
        rust_target=20.0,  # <20ms
        python_baseline=(200.0, 500.0),
        unit='ms'
    ),
    'throughput': Claim(
        category='Throughput',
        metric='Requests per second',
        min_improvement=100.0,
        max_improvement=None,  # >=100x
        rust_target=10000.0,  # >10,000 req/sec
        python_baseline=100.0,
        unit='req/sec'
    ),
    'memory': Claim(
        category='Memory',
        metric='RAM usage under load',
        min_improvement=8.0,
        max_improvement=16.0,
        rust_target=500.0,  # <500MB
        python_baseline=(4000.0, 8000.0),
        unit='MB'
    ),
    'cold_start': Claim(
        category='Cold Start',
        metric='Application startup time',
        min_improvement=10.0,
        max_improvement=30.0,
        rust_target=1000.0,  # <1s (1000ms)
        python_baseline=(10000.0, 30000.0),
        unit='ms'
    ),
    'binary_size': Claim(
        category='Binary Size',
        metric='Deployment artifact size',
        min_improvement=60.0,
        max_improvement=100.0,
        rust_target=2.0,  # <2MB (WASM gzip)
        python_baseline=(3000.0, 5000.0),  # Docker image
        unit='MB'
    ),
    'cpu_usage': Claim(
        category='CPU Usage',
        metric='CPU efficiency',
        min_improvement=5.0,
        max_improvement=10.0,
        rust_target=None,
        python_baseline=None,
        unit='req/sec per core'
    ),
}


class ClaimValidator:
    """Validates actual results against performance claims"""

    def __init__(self, analysis_file: Path):
        self.analysis_file = Path(analysis_file)

        # Load analysis results
        try:
            with open(self.analysis_file) as f:
                self.analysis = json.load(f)
        except FileNotFoundError:
            print(f"Error: Analysis file not found: {self.analysis_file}")
            print("Please run analyze_results.py first.")
            sys.exit(1)

    def validate_claim(self, claim_key: str, claim: Claim) -> Dict:
        """Validate a single claim against actual results"""

        # Find matching comparisons
        comparisons = [
            c for c in self.analysis.get('comparisons', [])
            if c['category'].lower() == claim.category.lower()
        ]

        if not comparisons:
            return {
                'claim': claim.category,
                'status': 'NO_DATA',
                'message': 'No benchmark data available',
                'actual': None,
                'claimed': f"{claim.min_improvement}-{claim.max_improvement or claim.min_improvement}x",
                'passed': False
            }

        # Calculate average improvement across all scenarios in category
        improvements = [c['improvement_factor'] for c in comparisons]
        avg_improvement = sum(improvements) / len(improvements)

        # Calculate average Rust value
        rust_values = [c['rust_value'] for c in comparisons]
        avg_rust = sum(rust_values) / len(rust_values)

        # Check if improvement meets claim
        if claim.max_improvement:
            # Range claim (e.g., 10-25x)
            passed = claim.min_improvement <= avg_improvement <= claim.max_improvement
            tolerance = abs(claim.min_improvement - claim.max_improvement) * 0.1
        else:
            # Minimum claim (e.g., >=100x)
            passed = avg_improvement >= claim.min_improvement
            tolerance = claim.min_improvement * 0.1

        # Allow 10% tolerance
        if not passed:
            passed = avg_improvement >= (claim.min_improvement - tolerance)

        # Check target value if specified
        target_met = True
        if claim.rust_target:
            # For latency, memory, cold start (lower is better)
            if claim.category in ['Latency', 'Memory', 'Cold Start', 'Binary Size']:
                target_met = avg_rust <= claim.rust_target
            # For throughput (higher is better)
            else:
                target_met = avg_rust >= claim.rust_target

        # Detailed status
        if passed and target_met:
            status = 'PASS'
            icon = '✅'
        elif passed and not target_met:
            status = 'PARTIAL'
            icon = '⚠️'
        else:
            status = 'FAIL'
            icon = '❌'

        # Build result
        result = {
            'claim': claim.category,
            'status': status,
            'icon': icon,
            'actual_improvement': avg_improvement,
            'claimed_improvement': f"{claim.min_improvement}-{claim.max_improvement or claim.min_improvement}x",
            'actual_value': avg_rust,
            'target_value': claim.rust_target,
            'unit': claim.unit,
            'passed': passed,
            'target_met': target_met,
            'scenarios': len(comparisons),
            'message': self._generate_message(claim, avg_improvement, avg_rust, passed, target_met)
        }

        return result

    def _generate_message(self, claim: Claim, actual_improvement: float,
                         actual_value: float, passed: bool, target_met: bool) -> str:
        """Generate human-readable validation message"""

        if passed and target_met:
            return f"✓ Meets all criteria"
        elif passed and not target_met:
            return f"⚠ Improvement met ({actual_improvement:.1f}x) but target missed ({actual_value:.1f}{claim.unit})"
        elif not passed and target_met:
            return f"⚠ Target met but improvement below claim ({actual_improvement:.1f}x < {claim.min_improvement}x)"
        else:
            return f"✗ Neither improvement ({actual_improvement:.1f}x) nor target ({actual_value:.1f}{claim.unit}) met"

    def validate_all(self) -> Dict:
        """Validate all claims"""

        results = {}
        for key, claim in PERFORMANCE_CLAIMS.items():
            results[key] = self.validate_claim(key, claim)

        # Calculate overall status
        total = len(results)
        passed = sum(1 for r in results.values() if r['passed'])
        failed = total - passed

        overall = {
            'total_claims': total,
            'passed': passed,
            'failed': failed,
            'pass_rate': f"{(passed / total * 100):.1f}%",
            'overall_status': 'PASS' if failed == 0 else 'PARTIAL' if passed > 0 else 'FAIL'
        }

        return {
            'overall': overall,
            'claims': results
        }

    def print_report(self, validation: Dict):
        """Print validation report to console"""

        print("\n" + "=" * 60)
        print("PERFORMANCE CLAIMS VALIDATION REPORT")
        print("=" * 60)
        print()

        overall = validation['overall']
        print(f"Overall Status: {overall['overall_status']}")
        print(f"Claims Passed: {overall['passed']}/{overall['total_claims']} ({overall['pass_rate']})")
        print(f"Claims Failed: {overall['failed']}")
        print()
        print("=" * 60)
        print()

        # Print individual claims
        for key, result in validation['claims'].items():
            print(f"{result['icon']} {result['claim']}")
            print(f"   Claimed: {result['claimed_improvement']}", end='')
            if result['target_value']:
                print(f", Target: {result['target_value']}{result['unit']}")
            else:
                print()

            if result['status'] != 'NO_DATA':
                print(f"   Actual: {result['actual_improvement']:.1f}x", end='')
                if result['actual_value']:
                    print(f", Value: {result['actual_value']:.1f}{result['unit']}")
                else:
                    print()
                print(f"   Status: {result['message']}")
            else:
                print(f"   Status: {result['message']}")

            print()

        print("=" * 60)
        print()

        # Recommendations
        if overall['failed'] > 0:
            print("RECOMMENDATIONS:")
            print()
            for key, result in validation['claims'].items():
                if not result['passed']:
                    print(f"• {result['claim']}: {self._get_recommendation(key, result)}")
            print()

    def _get_recommendation(self, key: str, result: Dict) -> str:
        """Get improvement recommendation for failed claim"""

        recommendations = {
            'latency': "Profile hot paths with flamegraph, optimize regex compilation",
            'throughput': "Check async runtime configuration, enable connection pooling",
            'memory': "Review allocator settings (jemalloc), check for memory leaks",
            'cold_start': "Enable lazy initialization, optimize ONNX model loading",
            'binary_size': "Enable strip=true in Cargo.toml, use wasm-opt with -Oz",
            'cpu_usage': "Reduce syscalls, batch operations, optimize hot loops"
        }

        return recommendations.get(key, "Profile and optimize critical paths")

    def save_report(self, validation: Dict, output_file: Path):
        """Save validation report to JSON"""

        with open(output_file, 'w') as f:
            json.dump(validation, f, indent=2)

        print(f"Validation report saved to: {output_file}")

    def generate_readme_updates(self, validation: Dict) -> str:
        """Generate suggested README.md updates based on validation"""

        updates = []
        updates.append("## Suggested README.md Updates")
        updates.append("")

        for key, result in validation['claims'].items():
            if result['status'] == 'NO_DATA':
                continue

            actual_imp = result['actual_improvement']
            actual_val = result['actual_value']

            if result['passed']:
                # Validated - can keep or update with actual values
                updates.append(f"### {result['claim']} ✓ VALIDATED")
                updates.append(f"**Current claim:** {result['claimed_improvement']}")
                updates.append(f"**Actual result:** {actual_imp:.1f}x faster")

                if actual_val and result['target_value']:
                    updates.append(f"**Actual value:** {actual_val:.1f}{result['unit']} (target: {result['target_value']}{result['unit']})")

                updates.append(f"**Recommendation:** Keep current claim or update to `{actual_imp:.0f}x`")
            else:
                # Failed - must update
                updates.append(f"### {result['claim']} ❌ NEEDS UPDATE")
                updates.append(f"**Current claim:** {result['claimed_improvement']}")
                updates.append(f"**Actual result:** {actual_imp:.1f}x")
                updates.append(f"**Recommendation:** Update claim to `{max(1, actual_imp * 0.9):.0f}-{actual_imp * 1.1:.0f}x`")

            updates.append("")

        return "\n".join(updates)


def main():
    """Main entry point"""
    import argparse

    parser = argparse.ArgumentParser(description='Validate LLM Shield performance claims')
    parser.add_argument('--analysis', type=str, default='../results/analysis.json',
                       help='Analysis JSON file from analyze_results.py')
    parser.add_argument('--output', type=str, default='../results/validation.json',
                       help='Output file for validation report')
    parser.add_argument('--readme-updates', action='store_true',
                       help='Generate README.md update suggestions')

    args = parser.parse_args()

    # Create validator
    validator = ClaimValidator(analysis_file=Path(args.analysis))

    # Run validation
    validation = validator.validate_all()

    # Print report
    validator.print_report(validation)

    # Save report
    validator.save_report(validation, Path(args.output))

    # Generate README updates if requested
    if args.readme_updates:
        updates = validator.generate_readme_updates(validation)
        readme_file = Path(args.output).parent / 'README_UPDATES.md'
        with open(readme_file, 'w') as f:
            f.write(updates)
        print(f"README updates saved to: {readme_file}")

    # Exit with appropriate code
    if validation['overall']['overall_status'] == 'PASS':
        sys.exit(0)
    elif validation['overall']['overall_status'] == 'PARTIAL':
        sys.exit(1)
    else:
        sys.exit(2)


if __name__ == '__main__':
    main()
