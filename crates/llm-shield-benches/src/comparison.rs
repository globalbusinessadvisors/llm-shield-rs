//! Comparison utilities for Rust vs Python benchmarks

use crate::{BenchmarkResult, CategoryReport, ComparisonResult, ValidationReport};
use chrono::Utc;
use std::collections::HashMap;

/// Compare Rust and Python benchmark results
///
/// # Arguments
///
/// * `test_name` - Name of the test
/// * `rust_result` - Rust benchmark result
/// * `python_result` - Python benchmark result
/// * `claimed_improvement` - Expected improvement factor (e.g., "10-25x")
pub fn compare_rust_vs_python(
    test_name: &str,
    rust_result: &BenchmarkResult,
    python_result: &BenchmarkResult,
    claimed_improvement: &str,
) -> ComparisonResult {
    // Calculate improvement factor
    let improvement_factor = python_result.mean_ms / rust_result.mean_ms;

    // Parse claimed improvement range
    let claim_validated = validate_improvement_claim(improvement_factor, claimed_improvement);

    ComparisonResult {
        test_name: test_name.to_string(),
        rust_mean_ms: rust_result.mean_ms,
        python_mean_ms: python_result.mean_ms,
        improvement_factor,
        claimed_improvement: claimed_improvement.to_string(),
        claim_validated,
        rust_p50: rust_result.p50_ms,
        rust_p95: rust_result.p95_ms,
        rust_p99: rust_result.p99_ms,
        python_p50: python_result.p50_ms,
        python_p95: python_result.p95_ms,
        python_p99: python_result.p99_ms,
    }
}

/// Validate if actual improvement meets claimed improvement
///
/// Claimed improvement formats:
/// - "10x" -> at least 10x
/// - "10-25x" -> between 10x and 25x
/// - ">10x" -> greater than 10x
fn validate_improvement_claim(actual_improvement: f64, claimed: &str) -> bool {
    // Parse claimed improvement
    let claimed_clean = claimed.replace('x', "").replace('X', "");

    if claimed_clean.contains('-') {
        // Range format: "10-25"
        let parts: Vec<&str> = claimed_clean.split('-').collect();
        if parts.len() == 2 {
            let min = parts[0].trim().parse::<f64>().unwrap_or(0.0);
            let max = parts[1].trim().parse::<f64>().unwrap_or(f64::MAX);
            return actual_improvement >= min && actual_improvement <= max;
        }
    } else if claimed_clean.starts_with('>') {
        // Greater than format: ">10"
        let min = claimed_clean[1..].trim().parse::<f64>().unwrap_or(0.0);
        return actual_improvement > min;
    } else {
        // Single value format: "10" means >= 10
        let min = claimed_clean.trim().parse::<f64>().unwrap_or(0.0);
        return actual_improvement >= min;
    }

    false
}

/// Get claimed improvement for a test category
///
/// Returns the improvement claim from README
pub fn get_claimed_improvement(category: &str) -> &'static str {
    match category {
        "latency" => "10-25x",
        "throughput" => "100x",
        "memory" => "8-16x",
        "cold_start" => "10-30x",
        "binary_size" => "60-100x",
        "cpu" => "5-10x",
        _ => "1x",
    }
}

/// Validate all benchmark claims
///
/// # Arguments
///
/// * `rust_results` - HashMap of category -> Vec<BenchmarkResult>
/// * `python_results` - HashMap of category -> Vec<BenchmarkResult>
pub fn validate_all_claims(
    rust_results: &HashMap<String, Vec<BenchmarkResult>>,
    python_results: &HashMap<String, Vec<BenchmarkResult>>,
) -> ValidationReport {
    let mut categories = Vec::new();

    let category_names = vec![
        "latency",
        "throughput",
        "memory",
        "cold_start",
        "binary_size",
        "cpu",
    ];

    for category_name in &category_names {
        if let (Some(rust_tests), Some(python_tests)) = (
            rust_results.get(*category_name),
            python_results.get(*category_name),
        ) {
            let mut category_comparisons = Vec::new();
            let claimed = get_claimed_improvement(category_name);

            for (rust_test, python_test) in rust_tests.iter().zip(python_tests.iter()) {
                let comparison =
                    compare_rust_vs_python(&rust_test.test_name, rust_test, python_test, claimed);
                category_comparisons.push(comparison);
            }

            let overall_validation = category_comparisons
                .iter()
                .all(|c| c.claim_validated);

            categories.push(CategoryReport {
                name: category_name.to_string(),
                tests: category_comparisons,
                overall_validation,
            });
        }
    }

    let overall_validation = categories.iter().all(|c| c.overall_validation);

    let summary = generate_summary(&categories, overall_validation);

    ValidationReport {
        timestamp: Utc::now(),
        categories,
        overall_validation,
        summary,
    }
}

/// Generate summary text for validation report
fn generate_summary(categories: &[CategoryReport], overall_validation: bool) -> String {
    let passed_count = categories.iter().filter(|c| c.overall_validation).count();
    let total_count = categories.len();

    let status = if overall_validation {
        "✅ PASS - All performance claims validated!"
    } else {
        "❌ FAIL - Some performance claims not met"
    };

    format!(
        "{}\n\nCategories Passed: {}/{}\n\nDetailed Results:\n{}",
        status,
        passed_count,
        total_count,
        categories
            .iter()
            .map(|c| format!(
                "- {}: {}",
                c.name,
                if c.overall_validation {
                    "✅ PASS"
                } else {
                    "❌ FAIL"
                }
            ))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Format comparison result as markdown table row
pub fn format_comparison_row(comparison: &ComparisonResult) -> String {
    format!(
        "| {} | {:.2}ms | {:.2}ms | {:.1f}x | {} | {} |",
        comparison.test_name,
        comparison.rust_mean_ms,
        comparison.python_mean_ms,
        comparison.improvement_factor,
        comparison.claimed_improvement,
        if comparison.claim_validated {
            "✅"
        } else {
            "❌"
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_result(name: &str, language: &str, mean_ms: f64) -> BenchmarkResult {
        BenchmarkResult {
            test_name: name.to_string(),
            language: language.to_string(),
            iterations: 1000,
            p50_ms: mean_ms,
            p95_ms: mean_ms * 1.5,
            p99_ms: mean_ms * 2.0,
            mean_ms,
            min_ms: mean_ms * 0.5,
            max_ms: mean_ms * 3.0,
            std_dev: mean_ms * 0.1,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_compare_rust_vs_python() {
        let rust_result = create_test_result("test", "rust", 1.0);
        let python_result = create_test_result("test", "python", 15.0);

        let comparison = compare_rust_vs_python("test", &rust_result, &python_result, "10-25x");

        assert_eq!(comparison.improvement_factor, 15.0);
        assert!(comparison.claim_validated);
    }

    #[test]
    fn test_validate_improvement_claim_range() {
        assert!(validate_improvement_claim(15.0, "10-25x"));
        assert!(validate_improvement_claim(10.0, "10-25x"));
        assert!(validate_improvement_claim(25.0, "10-25x"));
        assert!(!validate_improvement_claim(5.0, "10-25x"));
        assert!(!validate_improvement_claim(30.0, "10-25x"));
    }

    #[test]
    fn test_validate_improvement_claim_single() {
        assert!(validate_improvement_claim(10.0, "10x"));
        assert!(validate_improvement_claim(15.0, "10x"));
        assert!(!validate_improvement_claim(9.0, "10x"));
    }

    #[test]
    fn test_validate_improvement_claim_greater_than() {
        assert!(validate_improvement_claim(11.0, ">10x"));
        assert!(!validate_improvement_claim(10.0, ">10x"));
        assert!(!validate_improvement_claim(9.0, ">10x"));
    }

    #[test]
    fn test_get_claimed_improvement() {
        assert_eq!(get_claimed_improvement("latency"), "10-25x");
        assert_eq!(get_claimed_improvement("throughput"), "100x");
        assert_eq!(get_claimed_improvement("memory"), "8-16x");
        assert_eq!(get_claimed_improvement("cold_start"), "10-30x");
    }

    #[test]
    fn test_format_comparison_row() {
        let comparison = ComparisonResult {
            test_name: "test_scanner".to_string(),
            rust_mean_ms: 1.5,
            python_mean_ms: 18.0,
            improvement_factor: 12.0,
            claimed_improvement: "10-25x".to_string(),
            claim_validated: true,
            rust_p50: 1.5,
            rust_p95: 2.0,
            rust_p99: 2.5,
            python_p50: 18.0,
            python_p95: 25.0,
            python_p99: 30.0,
        };

        let row = format_comparison_row(&comparison);
        assert!(row.contains("test_scanner"));
        assert!(row.contains("1.50ms"));
        assert!(row.contains("18.00ms"));
        assert!(row.contains("12.0x"));
        assert!(row.contains("✅"));
    }
}
