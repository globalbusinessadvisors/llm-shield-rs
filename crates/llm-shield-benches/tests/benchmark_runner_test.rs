//! Integration tests for benchmark runner infrastructure
//!
//! Following London School TDD: Tests first, outside-in, behavior-focused

use llm_shield_benches::{
    fixtures::{generate_test_prompts, load_test_prompts, save_test_prompts},
    metrics::{compute_metrics, metrics_to_benchmark_result},
    comparison::{compare_rust_vs_python, validate_improvement_claim, get_claimed_improvement},
    BenchmarkConfig, TestPrompt, BenchmarkResult,
};
use std::fs;
use std::path::PathBuf;
use chrono::Utc;

/// Test that we can generate the required number of test prompts
#[test]
fn test_generate_1000_test_prompts() {
    let prompts = generate_test_prompts(1000);

    assert_eq!(prompts.len(), 1000, "Should generate exactly 1000 prompts");

    // Verify distribution (approximate due to rounding)
    let categories = ["simple", "medium", "long", "secrets", "code", "injection", "toxic"];
    for category in &categories {
        let count = prompts.iter().filter(|p| p.category == category).count();
        assert!(count > 0, "Category {} should have prompts", category);
    }
}

/// Test that test data can be saved and loaded
#[test]
fn test_save_and_load_test_prompts() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_prompts_integration.json");

    // Clean up if exists
    let _ = fs::remove_file(&test_file);

    // Generate and save
    let original_prompts = generate_test_prompts(100);
    save_test_prompts(&original_prompts, &test_file)
        .expect("Should save prompts successfully");

    // Verify file exists
    assert!(test_file.exists(), "JSON file should be created");

    // Load and verify
    let content = fs::read_to_string(&test_file)
        .expect("Should read file");
    let loaded_prompts: Vec<TestPrompt> = serde_json::from_str(&content)
        .expect("Should parse JSON");

    assert_eq!(loaded_prompts.len(), 100, "Should load all prompts");
    assert_eq!(loaded_prompts[0].id, original_prompts[0].id, "First prompt should match");

    // Cleanup
    fs::remove_file(&test_file).ok();
}

/// Test that benchmark config has sensible defaults
#[test]
fn test_benchmark_config_defaults() {
    let config = BenchmarkConfig::default();

    assert_eq!(config.latency_iterations, 1000, "Should have 1000 latency iterations");
    assert_eq!(config.ml_iterations, 100, "Should have 100 ML iterations (slower)");
    assert_eq!(config.throughput_duration_secs, 60, "Should run throughput for 60s");
    assert!(config.throughput_concurrency.contains(&100), "Should test 100 concurrent connections");
}

/// Test metrics calculation with known data
#[test]
fn test_metrics_calculation_accuracy() {
    // Create known distribution: 1-100ms
    let mut measurements = Vec::new();
    for i in 1..=100 {
        measurements.push(i as f64);
    }

    let metrics = compute_metrics(&measurements);

    assert_eq!(metrics.count, 100);
    assert_eq!(metrics.min, 1.0);
    assert_eq!(metrics.max, 100.0);
    assert_eq!(metrics.p50, 50.0, "Median should be 50");
    assert_eq!(metrics.p95, 95.0, "p95 should be 95");
    assert_eq!(metrics.p99, 99.0, "p99 should be 99");

    // Mean should be 50.5 ((1+100)/2)
    assert!((metrics.mean - 50.5).abs() < 0.1, "Mean should be approximately 50.5");
}

/// Test conversion from metrics to benchmark result
#[test]
fn test_metrics_to_benchmark_result_conversion() {
    let measurements = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let metrics = compute_metrics(&measurements);

    let result = metrics_to_benchmark_result("test_scanner", "rust", &metrics);

    assert_eq!(result.test_name, "test_scanner");
    assert_eq!(result.language, "rust");
    assert_eq!(result.iterations, 5);
    assert_eq!(result.p50_ms, 3.0);
    assert!(result.timestamp <= Utc::now(), "Timestamp should be in the past or now");
}

/// Test comparison between Rust and Python results
#[test]
fn test_rust_vs_python_comparison() {
    let rust_result = BenchmarkResult {
        test_name: "ban_substrings".to_string(),
        language: "rust".to_string(),
        iterations: 1000,
        p50_ms: 0.8,
        p95_ms: 1.2,
        p99_ms: 1.8,
        mean_ms: 0.9,
        min_ms: 0.5,
        max_ms: 2.0,
        std_dev: 0.2,
        timestamp: Utc::now(),
    };

    let python_result = BenchmarkResult {
        test_name: "ban_substrings".to_string(),
        language: "python".to_string(),
        iterations: 1000,
        p50_ms: 12.0,
        p95_ms: 18.0,
        p99_ms: 25.0,
        mean_ms: 13.5,
        min_ms: 8.0,
        max_ms: 30.0,
        std_dev: 3.5,
        timestamp: Utc::now(),
    };

    let comparison = compare_rust_vs_python(
        "ban_substrings",
        &rust_result,
        &python_result,
        "10-25x"
    );

    assert_eq!(comparison.test_name, "ban_substrings");
    assert_eq!(comparison.rust_mean_ms, 0.9);
    assert_eq!(comparison.python_mean_ms, 13.5);
    assert_eq!(comparison.improvement_factor, 13.5 / 0.9);
    assert!(comparison.claim_validated, "Should validate 10-25x claim with 15x improvement");
}

/// Test improvement claim validation logic
#[test]
fn test_improvement_claim_validation() {
    // Range validation
    assert!(validate_improvement_claim(15.0, "10-25x"), "15x should pass 10-25x");
    assert!(validate_improvement_claim(10.0, "10-25x"), "10x should pass 10-25x (lower bound)");
    assert!(validate_improvement_claim(25.0, "10-25x"), "25x should pass 10-25x (upper bound)");
    assert!(!validate_improvement_claim(5.0, "10-25x"), "5x should fail 10-25x");
    assert!(!validate_improvement_claim(30.0, "10-25x"), "30x should fail 10-25x (too high)");

    // Single value validation
    assert!(validate_improvement_claim(10.0, "10x"), "10x should pass >=10x");
    assert!(validate_improvement_claim(15.0, "10x"), "15x should pass >=10x");
    assert!(!validate_improvement_claim(9.0, "10x"), "9x should fail >=10x");

    // Greater than validation
    assert!(validate_improvement_claim(11.0, ">10x"), "11x should pass >10x");
    assert!(!validate_improvement_claim(10.0, ">10x"), "10x should fail >10x (not greater)");
}

/// Test that claimed improvements are correctly defined
#[test]
fn test_claimed_improvements_defined() {
    assert_eq!(get_claimed_improvement("latency"), "10-25x");
    assert_eq!(get_claimed_improvement("throughput"), "100x");
    assert_eq!(get_claimed_improvement("memory"), "8-16x");
    assert_eq!(get_claimed_improvement("cold_start"), "10-30x");
    assert_eq!(get_claimed_improvement("binary_size"), "60-100x");
    assert_eq!(get_claimed_improvement("cpu"), "5-10x");
}

/// Test that test prompts have expected threat annotations
#[test]
fn test_prompts_with_threats_are_annotated() {
    let prompts = generate_test_prompts(1000);

    // Filter secrets category
    let secret_prompts: Vec<_> = prompts.iter()
        .filter(|p| p.category == "secrets")
        .collect();

    assert!(!secret_prompts.is_empty(), "Should have secret prompts");

    // All secret prompts should have expected_threats
    for prompt in secret_prompts {
        assert!(!prompt.expected_threats.is_empty(),
            "Secret prompts should have expected threats: {}", prompt.id);
    }

    // Filter injection category
    let injection_prompts: Vec<_> = prompts.iter()
        .filter(|p| p.category == "injection")
        .collect();

    for prompt in injection_prompts {
        assert!(prompt.expected_threats.contains(&"prompt_injection".to_string()),
            "Injection prompts should have 'prompt_injection' threat: {}", prompt.id);
    }
}

/// Test that prompts have realistic word counts
#[test]
fn test_prompt_word_counts_are_realistic() {
    let prompts = generate_test_prompts(1000);

    // Simple prompts: 10-50 words
    let simple: Vec<_> = prompts.iter().filter(|p| p.category == "simple").collect();
    for prompt in simple {
        assert!(prompt.word_count >= 10 && prompt.word_count <= 50,
            "Simple prompt {} should have 10-50 words, got {}", prompt.id, prompt.word_count);
    }

    // Medium prompts: 50-200 words
    let medium: Vec<_> = prompts.iter().filter(|p| p.category == "medium").collect();
    for prompt in medium {
        assert!(prompt.word_count >= 50 && prompt.word_count <= 200,
            "Medium prompt {} should have 50-200 words, got {}", prompt.id, prompt.word_count);
    }

    // Long prompts: 200-500 words
    let long: Vec<_> = prompts.iter().filter(|p| p.category == "long").collect();
    for prompt in long {
        assert!(prompt.word_count >= 200 && prompt.word_count <= 500,
            "Long prompt {} should have 200-500 words, got {}", prompt.id, prompt.word_count);
    }
}

/// Test that benchmark results can be serialized to JSON
#[test]
fn test_benchmark_result_serialization() {
    let result = BenchmarkResult {
        test_name: "test".to_string(),
        language: "rust".to_string(),
        iterations: 1000,
        p50_ms: 1.0,
        p95_ms: 2.0,
        p99_ms: 3.0,
        mean_ms: 1.5,
        min_ms: 0.5,
        max_ms: 5.0,
        std_dev: 0.8,
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&result)
        .expect("Should serialize to JSON");

    assert!(json.contains("\"test_name\":\"test\""));
    assert!(json.contains("\"language\":\"rust\""));

    // Should be able to deserialize back
    let deserialized: BenchmarkResult = serde_json::from_str(&json)
        .expect("Should deserialize from JSON");

    assert_eq!(deserialized.test_name, result.test_name);
    assert_eq!(deserialized.iterations, result.iterations);
}

/// Test edge case: empty measurements
#[test]
fn test_metrics_with_empty_measurements() {
    let measurements = Vec::new();
    let metrics = compute_metrics(&measurements);

    assert_eq!(metrics.count, 0);
    assert_eq!(metrics.mean, 0.0);
    assert_eq!(metrics.min, 0.0);
    assert_eq!(metrics.max, 0.0);
}

/// Test edge case: single measurement
#[test]
fn test_metrics_with_single_measurement() {
    let measurements = vec![42.0];
    let metrics = compute_metrics(&measurements);

    assert_eq!(metrics.count, 1);
    assert_eq!(metrics.mean, 42.0);
    assert_eq!(metrics.median, 42.0);
    assert_eq!(metrics.min, 42.0);
    assert_eq!(metrics.max, 42.0);
    assert_eq!(metrics.std_dev, 0.0);
}

/// Test that test data directory structure can be created
#[test]
fn test_benchmark_config_directory_creation() {
    let config = BenchmarkConfig::default();

    // These paths should be defined
    assert!(config.data_dir.to_str().is_some());
    assert!(config.results_dir.to_str().is_some());

    // Default paths should be under benchmarks/
    assert!(config.data_dir.to_str().unwrap().contains("benchmarks"));
    assert!(config.results_dir.to_str().unwrap().contains("benchmarks"));
}
