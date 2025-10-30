# LLM Shield Benchmark Pseudocode

## Overview

This document contains the pseudocode and algorithm design for the LLM Shield benchmarking infrastructure.

---

## Component 1: Test Data Generator

### Function: generate_test_prompts(count: int) → Vec<TestPrompt>

```
INPUT: count = 1000
OUTPUT: List of TestPrompt objects

ALGORITHM:
1. Initialize empty prompts list
2. Calculate distribution:
   - simple = count * 0.20 (200)
   - medium = count * 0.20 (200)
   - long = count * 0.20 (200)
   - secrets = count * 0.10 (100)
   - code = count * 0.10 (100)
   - injection = count * 0.10 (100)
   - toxic = count * 0.10 (100)

3. FOR i in 0..simple:
     prompt = TestPrompt {
       id: "simple_{i}",
       text: generate_simple_text(10, 50),  # 10-50 words
       category: "simple",
       expected_threats: []
     }
     prompts.append(prompt)

4. FOR i in 0..medium:
     prompt = TestPrompt {
       id: "medium_{i}",
       text: generate_medium_text(50, 200),
       category: "medium",
       expected_threats: []
     }
     prompts.append(prompt)

5. FOR i in 0..long:
     prompt = TestPrompt {
       id: "long_{i}",
       text: generate_long_text(200, 500),
       category: "long",
       expected_threats: []
     }
     prompts.append(prompt)

6. FOR i in 0..secrets:
     secret_type = choose_random([
       "aws_key", "stripe_key", "slack_token",
       "github_token", "jwt", "password", "api_key"
     ])
     prompt = TestPrompt {
       id: "secret_{i}",
       text: embed_secret(generate_medium_text(), secret_type),
       category: "secrets",
       expected_threats: [secret_type]
     }
     prompts.append(prompt)

7. FOR i in 0..code:
     language = choose_random(["python", "javascript", "rust", "java"])
     prompt = TestPrompt {
       id: "code_{i}",
       text: generate_code_snippet(language, 10, 50),
       category: "code",
       expected_threats: []
     }
     prompts.append(prompt)

8. FOR i in 0..injection:
     injection_type = choose_random([
       "jailbreak", "role_reversal", "system_prompt_leak",
       "instruction_override", "delimiter_injection"
     ])
     prompt = TestPrompt {
       id: "injection_{i}",
       text: generate_injection_attempt(injection_type),
       category: "injection",
       expected_threats: ["prompt_injection"]
     }
     prompts.append(prompt)

9. FOR i in 0..toxic:
     toxicity_type = choose_random([
       "violence", "hate_speech", "harassment",
       "self_harm", "sexual", "profanity"
     ])
     prompt = TestPrompt {
       id: "toxic_{i}",
       text: generate_toxic_content(toxicity_type),
       category: "toxic",
       expected_threats: ["toxicity"]
     }
     prompts.append(prompt)

10. Shuffle prompts randomly
11. Save to JSON file
12. RETURN prompts
```

---

## Component 2: Benchmark Runner

### Function: run_latency_benchmark(scanner_name: str, iterations: int) → BenchmarkResult

```
INPUT:
  - scanner_name: "BanSubstrings", "Secrets", "PromptInjection", etc.
  - iterations: 1000 (or 100 for ML)

OUTPUT: BenchmarkResult with statistics

ALGORITHM:
1. Load test data:
     prompts = load_test_prompts()
     test_set = filter_prompts_by_category(prompts, scanner_name)

2. Initialize scanner:
     scanner = create_scanner(scanner_name)
     vault = SecretVault::new()

3. Warm-up phase (avoid cold start bias):
     FOR i in 0..10:
       _ = scanner.scan(test_set[0], &vault).await

4. Initialize results storage:
     latencies = Vec::new()

5. Benchmark phase:
     FOR iteration in 0..iterations:
       # Select random prompt to avoid cache effects
       prompt_idx = random(0, test_set.length)
       prompt = test_set[prompt_idx]

       # Measure latency
       start_time = high_precision_timer()
       result = scanner.scan(prompt.text, &vault).await
       end_time = high_precision_timer()

       latency_ms = (end_time - start_time).as_millis_f64()
       latencies.push(latency_ms)

       # Verify correctness (London TDD: behavior validation)
       IF prompt.expected_threats is not empty:
         assert result.is_valid == false OR result.threats match expected

6. Calculate statistics:
     metrics = compute_metrics(latencies)

7. Create result object:
     result = BenchmarkResult {
       test_name: scanner_name,
       language: "rust",
       iterations: iterations,
       p50_ms: metrics.p50,
       p95_ms: metrics.p95,
       p99_ms: metrics.p99,
       mean_ms: metrics.mean,
       min_ms: metrics.min,
       max_ms: metrics.max,
       std_dev: metrics.std_dev,
       timestamp: current_timestamp()
     }

8. Save results to CSV
9. RETURN result
```

### Function: run_throughput_benchmark(scanner_name: str, duration_secs: int, concurrency: int) → ThroughputResult

```
INPUT:
  - scanner_name: "BanSubstrings"
  - duration_secs: 60
  - concurrency: 100

OUTPUT: ThroughputResult

ALGORITHM:
1. Start server in background:
     server_process = spawn_server(scanner_name, port=3000)
     wait_for_server_ready()

2. Prepare load test:
     wrk_command = "wrk -t4 -c{concurrency} -d{duration}s --latency http://localhost:3000/scan"

3. Execute load test:
     wrk_output = execute_command(wrk_command)

4. Parse wrk output:
     result = parse_wrk_output(wrk_output)
     # Extract: requests/sec, latency distribution, errors

5. Stop server:
     server_process.kill()

6. Create result object:
     throughput_result = ThroughputResult {
       test_name: scanner_name,
       language: "rust",
       duration_secs: duration_secs,
       concurrency: concurrency,
       requests_per_sec: result.req_per_sec,
       total_requests: result.total_requests,
       latency_p50_ms: result.latency_p50,
       latency_p95_ms: result.latency_p95,
       latency_p99_ms: result.latency_p99,
       errors: result.errors,
       timeouts: result.timeouts,
       timestamp: current_timestamp()
     }

7. Save results to CSV
8. RETURN throughput_result
```

---

## Component 3: Metrics Calculator

### Function: compute_metrics(measurements: Vec<f64>) → Metrics

```
INPUT: measurements = [1.2, 1.5, 0.8, ..., 2.1]  # latencies in ms
OUTPUT: Metrics object with statistics

ALGORITHM:
1. Sort measurements in ascending order:
     sorted = measurements.sort()
     n = sorted.length

2. Calculate mean:
     sum = 0
     FOR value in sorted:
       sum += value
     mean = sum / n

3. Calculate percentiles:
     p50_index = floor(n * 0.50)
     p95_index = floor(n * 0.95)
     p99_index = floor(n * 0.99)

     p50 = sorted[p50_index]
     p95 = sorted[p95_index]
     p99 = sorted[p99_index]

4. Calculate standard deviation:
     variance_sum = 0
     FOR value in sorted:
       variance_sum += (value - mean)^2
     variance = variance_sum / n
     std_dev = sqrt(variance)

5. Get min/max:
     min = sorted[0]
     max = sorted[n - 1]

6. Create metrics object:
     metrics = Metrics {
       mean: mean,
       median: p50,
       p50: p50,
       p95: p95,
       p99: p99,
       min: min,
       max: max,
       std_dev: std_dev,
       count: n
     }

7. RETURN metrics
```

---

## Component 4: Comparison Framework

### Function: compare_rust_vs_python(test_name: str) → ComparisonResult

```
INPUT: test_name = "latency_ban_substrings"
OUTPUT: ComparisonResult with improvement metrics

ALGORITHM:
1. Load Rust results:
     rust_data = load_csv("benchmarks/results/rust/{test_name}.csv")
     rust_metrics = compute_metrics(rust_data.latencies)

2. Load Python results:
     python_data = load_csv("benchmarks/results/python/{test_name}.csv")
     python_metrics = compute_metrics(python_data.latencies)

3. Calculate improvements:
     latency_improvement = python_metrics.mean / rust_metrics.mean
     p95_improvement = python_metrics.p95 / rust_metrics.p95
     p99_improvement = python_metrics.p99 / rust_metrics.p99

4. Determine if claim is validated:
     claimed_improvement = get_claimed_improvement(test_name)  # e.g., 10-25x

     IF latency_improvement >= claimed_improvement.min:
       claim_validated = true
     ELSE:
       claim_validated = false

5. Create comparison result:
     comparison = ComparisonResult {
       test_name: test_name,
       rust_mean_ms: rust_metrics.mean,
       python_mean_ms: python_metrics.mean,
       improvement_factor: latency_improvement,
       claimed_improvement: claimed_improvement,
       claim_validated: claim_validated,
       rust_p50: rust_metrics.p50,
       rust_p95: rust_metrics.p95,
       rust_p99: rust_metrics.p99,
       python_p50: python_metrics.p50,
       python_p95: python_metrics.p95,
       python_p99: python_metrics.p99
     }

6. RETURN comparison
```

### Function: validate_all_claims() → ValidationReport

```
INPUT: None
OUTPUT: ValidationReport for all 6 categories

ALGORITHM:
1. Initialize report:
     report = ValidationReport {
       timestamp: current_timestamp(),
       categories: []
     }

2. Define test categories:
     categories = [
       ("latency", ["ban_substrings", "regex", "secrets", "prompt_injection"]),
       ("throughput", ["single_scanner", "pipeline"]),
       ("memory", ["baseline", "under_load", "growth"]),
       ("cold_start", ["startup", "first_request", "serverless"]),
       ("binary_size", ["docker", "native", "wasm"]),
       ("cpu", ["single_request", "sustained_load", "efficiency"])
     ]

3. FOR EACH (category_name, test_names) in categories:
     category_results = []

     FOR EACH test_name in test_names:
       full_test_name = "{category_name}_{test_name}"
       comparison = compare_rust_vs_python(full_test_name)
       category_results.append(comparison)

     # Aggregate category validation
     all_passed = all(result.claim_validated for result in category_results)

     category_report = CategoryReport {
       name: category_name,
       tests: category_results,
       overall_validation: all_passed
     }

     report.categories.append(category_report)

4. Calculate overall validation:
     report.overall_validation = all(cat.overall_validation for cat in report.categories)

5. Generate summary:
     report.summary = generate_summary_text(report)

6. Save report to markdown file
7. RETURN report
```

---

## Component 5: Chart Generator

### Function: generate_latency_comparison_chart(rust_data, python_data, output_file)

```
INPUT:
  - rust_data: BenchmarkResult for Rust
  - python_data: BenchmarkResult for Python
  - output_file: "benchmarks/charts/latency_comparison.png"

ALGORITHM (Python/Matplotlib):
1. Prepare data for plotting:
     labels = ["Mean", "p50", "p95", "p99"]
     rust_values = [rust_data.mean_ms, rust_data.p50_ms, rust_data.p95_ms, rust_data.p99_ms]
     python_values = [python_data.mean_ms, python_data.p50_ms, python_data.p95_ms, python_data.p99_ms]

2. Create figure:
     fig, ax = plt.subplots(figsize=(10, 6))

3. Create grouped bar chart:
     x_positions = np.arange(len(labels))
     bar_width = 0.35

     bars1 = ax.bar(x_positions - bar_width/2, rust_values, bar_width, label='Rust', color='#f74c00')
     bars2 = ax.bar(x_positions + bar_width/2, python_values, bar_width, label='Python', color='#3776ab')

4. Add value labels on bars:
     FOR bar in bars1:
       height = bar.get_height()
       ax.text(bar.get_x() + bar.get_width()/2., height,
               f'{height:.2f}ms', ha='center', va='bottom')

     # Same for bars2

5. Add improvement annotations:
     FOR i, label in enumerate(labels):
       improvement = python_values[i] / rust_values[i]
       ax.text(i, max(rust_values[i], python_values[i]) * 1.1,
               f'{improvement:.1f}x faster', ha='center', fontweight='bold', color='green')

6. Configure chart:
     ax.set_xlabel('Metric')
     ax.set_ylabel('Latency (ms)')
     ax.set_title('LLM Shield Latency Comparison: Rust vs Python')
     ax.set_xticks(x_positions)
     ax.set_xticklabels(labels)
     ax.legend()
     ax.grid(axis='y', alpha=0.3)

7. Save chart:
     plt.tight_layout()
     plt.savefig(output_file, dpi=300)
     plt.close()
```

---

## Component 6: Memory Profiling

### Function: measure_memory_usage(process_id: int, duration_secs: int, interval_secs: int) → MemoryProfile

```
INPUT:
  - process_id: PID of server process
  - duration_secs: 3600 (1 hour)
  - interval_secs: 10 (sample every 10 seconds)

ALGORITHM:
1. Initialize storage:
     memory_samples = []
     timestamps = []

2. Calculate sample count:
     num_samples = duration_secs / interval_secs

3. Sampling loop:
     FOR i in 0..num_samples:
       # Get RSS (Resident Set Size) in KB
       rss_kb = execute_command("ps -p {process_id} -o rss=").parse_int()

       # Get detailed memory map
       memory_map = execute_command("pmap -x {process_id}").parse()

       sample = MemorySample {
         timestamp: current_timestamp(),
         rss_kb: rss_kb,
         rss_mb: rss_kb / 1024,
         heap_kb: memory_map.heap_kb,
         stack_kb: memory_map.stack_kb,
         shared_kb: memory_map.shared_kb
       }

       memory_samples.append(sample)
       timestamps.append(current_timestamp())

       sleep(interval_secs)

4. Calculate statistics:
     baseline_mb = memory_samples[0].rss_mb
     peak_mb = max(sample.rss_mb for sample in memory_samples)
     final_mb = memory_samples[-1].rss_mb
     growth_mb = final_mb - baseline_mb
     growth_percent = (growth_mb / baseline_mb) * 100

5. Create profile:
     profile = MemoryProfile {
       process_id: process_id,
       duration_secs: duration_secs,
       samples: memory_samples,
       baseline_mb: baseline_mb,
       peak_mb: peak_mb,
       final_mb: final_mb,
       growth_mb: growth_mb,
       growth_percent: growth_percent,
       mean_mb: mean(sample.rss_mb for sample in memory_samples),
       timestamps: timestamps
     }

6. Save to CSV
7. RETURN profile
```

---

## Component 7: Cold Start Measurement

### Function: measure_cold_start(binary_path: str, iterations: int) → ColdStartResult

```
INPUT:
  - binary_path: "./target/release/llm-shield"
  - iterations: 100

ALGORITHM (using hyperfine):
1. Prepare command:
     test_command = "{binary_path} --test-mode"

2. Run hyperfine benchmark:
     hyperfine_command = "hyperfine --warmup 0 --runs {iterations} --export-json /tmp/cold_start.json '{test_command}'"
     hyperfine_output = execute_command(hyperfine_command)

3. Parse JSON results:
     results = parse_json("/tmp/cold_start.json")

4. Extract statistics:
     mean_secs = results.mean
     min_secs = results.min
     max_secs = results.max
     median_secs = results.median
     std_dev_secs = results.stddev

5. Create result:
     cold_start = ColdStartResult {
       binary_path: binary_path,
       iterations: iterations,
       mean_ms: mean_secs * 1000,
       median_ms: median_secs * 1000,
       min_ms: min_secs * 1000,
       max_ms: max_secs * 1000,
       std_dev_ms: std_dev_secs * 1000,
       claim_target_ms: 1000,  # <1s
       claim_validated: median_ms < 1000
     }

6. Save to CSV
7. RETURN cold_start
```

---

## Component 8: Binary Size Measurement

### Function: measure_binary_sizes() → BinarySizeResult

```
ALGORITHM:
1. Measure Python Docker image:
     python_docker_size_bytes = execute_command("docker images python-llm-guard --format '{{.Size}}'").parse_bytes()

2. Measure Rust Docker image:
     rust_docker_size_bytes = execute_command("docker images rust-llm-shield --format '{{.Size}}'").parse_bytes()

3. Measure native Rust binary:
     native_size_bytes = get_file_size("target/release/llm-shield")

4. Strip binary and measure:
     execute_command("strip target/release/llm-shield")
     stripped_size_bytes = get_file_size("target/release/llm-shield")

5. Compress with UPX and measure:
     execute_command("upx --best target/release/llm-shield")
     upx_size_bytes = get_file_size("target/release/llm-shield")

6. Build WASM and measure:
     execute_command("cd crates/llm-shield-wasm && wasm-pack build --release")
     wasm_size_bytes = get_file_size("crates/llm-shield-wasm/pkg/llm_shield_wasm_bg.wasm")

7. Optimize WASM:
     execute_command("wasm-opt -Oz input.wasm -o optimized.wasm")
     wasm_opt_size_bytes = get_file_size("optimized.wasm")

8. Gzip WASM:
     execute_command("gzip -k optimized.wasm")
     wasm_gzip_size_bytes = get_file_size("optimized.wasm.gz")

9. Calculate improvements:
     docker_improvement = python_docker_size_bytes / rust_docker_size_bytes
     wasm_improvement = python_docker_size_bytes / wasm_gzip_size_bytes

10. Create result:
      result = BinarySizeResult {
        python_docker_mb: bytes_to_mb(python_docker_size_bytes),
        rust_docker_mb: bytes_to_mb(rust_docker_size_bytes),
        rust_native_mb: bytes_to_mb(native_size_bytes),
        rust_stripped_mb: bytes_to_mb(stripped_size_bytes),
        rust_upx_mb: bytes_to_mb(upx_size_bytes),
        wasm_uncompressed_mb: bytes_to_mb(wasm_size_bytes),
        wasm_optimized_mb: bytes_to_mb(wasm_opt_size_bytes),
        wasm_gzip_mb: bytes_to_mb(wasm_gzip_size_bytes),
        docker_improvement: docker_improvement,
        wasm_improvement: wasm_improvement,
        claim_validated: wasm_gzip_size_bytes < 2 * 1024 * 1024  # <2MB
      }

11. RETURN result
```

---

## Component 9: CPU Usage Profiling

### Function: measure_cpu_usage(process_id: int, duration_secs: int) → CPUProfile

```
INPUT:
  - process_id: PID of server
  - duration_secs: 60

ALGORITHM:
1. Start pidstat monitoring:
     pidstat_command = "pidstat -p {process_id} 1 {duration_secs}"
     pidstat_output = execute_command(pidstat_command)

2. Parse pidstat output:
     # Example line: "12:34:56  1234  25.5  0.8  ..."
     cpu_samples = []

     FOR line in pidstat_output.lines:
       IF line contains process_id:
         parts = line.split_whitespace()
         cpu_percent = float(parts[6])  # %CPU column
         cpu_samples.append(cpu_percent)

3. Calculate statistics:
     mean_cpu = mean(cpu_samples)
     max_cpu = max(cpu_samples)
     min_cpu = min(cpu_samples)

4. Calculate CPU efficiency:
     # From throughput benchmark
     requests_per_sec = load_throughput_result()
     cpu_efficiency = requests_per_sec / mean_cpu

5. Create profile:
     profile = CPUProfile {
       process_id: process_id,
       duration_secs: duration_secs,
       mean_cpu_percent: mean_cpu,
       max_cpu_percent: max_cpu,
       min_cpu_percent: min_cpu,
       samples: cpu_samples,
       requests_per_sec: requests_per_sec,
       cpu_efficiency: cpu_efficiency
     }

6. RETURN profile
```

---

## Component 10: Report Generator

### Function: generate_final_report(validation_report: ValidationReport) → str

```
INPUT: validation_report with all test results
OUTPUT: Markdown report string

ALGORITHM:
1. Initialize markdown builder:
     md = MarkdownBuilder::new()

2. Add header:
     md.h1("LLM Shield Performance Benchmark Results")
     md.add_line(f"Generated: {current_timestamp()}")
     md.add_line(f"Environment: {get_environment_info()}")

3. Add executive summary:
     md.h2("Executive Summary")

     overall_status = "✅ PASS" if validation_report.overall_validation else "❌ FAIL"
     md.add_line(f"**Overall Validation: {overall_status}**")
     md.add_line()

     passed_count = count(cat for cat in validation_report.categories if cat.overall_validation)
     total_count = len(validation_report.categories)
     md.add_line(f"**Categories Passed: {passed_count}/{total_count}**")

4. Add summary table:
     md.h2("Results Summary")
     md.table_header(["Category", "Rust", "Python", "Improvement", "Claim", "Status"])

     FOR category in validation_report.categories:
       primary_test = category.tests[0]  # Use first test as representative

       rust_value = format_metric(primary_test.rust_mean_ms, category.name)
       python_value = format_metric(primary_test.python_mean_ms, category.name)
       improvement = f"{primary_test.improvement_factor:.1f}x"
       claim = primary_test.claimed_improvement
       status = "✅" if category.overall_validation else "❌"

       md.table_row([category.name, rust_value, python_value, improvement, claim, status])

5. Add detailed results for each category:
     FOR category in validation_report.categories:
       md.h2(f"{category.name.capitalize()} Benchmarks")

       FOR test in category.tests:
         md.h3(test.test_name)
         md.add_line(f"- **Rust:** {test.rust_mean_ms:.2f}ms (p95: {test.rust_p95:.2f}ms)")
         md.add_line(f"- **Python:** {test.python_mean_ms:.2f}ms (p95: {test.python_p95:.2f}ms)")
         md.add_line(f"- **Improvement:** {test.improvement_factor:.1f}x faster")
         md.add_line(f"- **Claim Validated:** {test.claim_validated}")
         md.add_line()

6. Add charts section:
     md.h2("Performance Charts")
     FOR category in validation_report.categories:
       chart_path = f"charts/{category.name}_comparison.png"
       md.add_image(category.name.capitalize(), chart_path)

7. Add methodology:
     md.h2("Methodology")
     md.add_line(load_file("benchmarks/METHODOLOGY.md"))

8. Add raw data links:
     md.h2("Raw Data")
     FOR category in validation_report.categories:
       csv_path = f"results/{category.name}_results.csv"
       md.add_link(f"{category.name} CSV", csv_path)

9. Generate markdown string:
     report_content = md.build()

10. Save to file:
      write_file("benchmarks/results/RESULTS.md", report_content)

11. RETURN report_content
```

---

## End-to-End Workflow

### Main Orchestration Algorithm

```
FUNCTION run_complete_benchmark_suite():

  # Phase 1: Setup
  print("Phase 1: Setup")
  generate_test_prompts(1000)
  setup_environment()

  # Phase 2: Run Rust benchmarks
  print("Phase 2: Running Rust benchmarks")
  rust_results = {
    "latency": run_rust_latency_benchmarks(),
    "throughput": run_rust_throughput_benchmarks(),
    "memory": run_rust_memory_benchmarks(),
    "cold_start": run_rust_cold_start_benchmarks(),
    "binary_size": measure_binary_sizes(),
    "cpu": run_rust_cpu_benchmarks()
  }

  # Phase 3: Run Python benchmarks
  print("Phase 3: Running Python benchmarks")
  python_results = {
    "latency": run_python_latency_benchmarks(),
    "throughput": run_python_throughput_benchmarks(),
    "memory": run_python_memory_benchmarks(),
    "cold_start": run_python_cold_start_benchmarks(),
    "cpu": run_python_cpu_benchmarks()
  }

  # Phase 4: Compare and validate
  print("Phase 4: Comparing results")
  validation_report = validate_all_claims()

  # Phase 5: Generate artifacts
  print("Phase 5: Generating reports and charts")
  FOR category in ["latency", "throughput", "memory", "cold_start", "cpu"]:
    generate_comparison_chart(
      rust_results[category],
      python_results[category],
      f"charts/{category}_comparison.png"
    )

  generate_final_report(validation_report)

  # Phase 6: Display summary
  print_summary(validation_report)

  IF validation_report.overall_validation:
    print("✅ All performance claims validated!")
  ELSE:
    print("❌ Some claims not validated. See RESULTS.md for details.")

  RETURN validation_report
```

---

## Data Structures

### TestPrompt
```
struct TestPrompt {
  id: String,              # "simple_42"
  text: String,            # Actual prompt text
  category: String,        # "simple", "secrets", "injection", etc.
  expected_threats: Vec<String>  # ["aws_key"] or []
}
```

### BenchmarkResult
```
struct BenchmarkResult {
  test_name: String,
  language: String,        # "rust" or "python"
  iterations: int,
  p50_ms: f64,
  p95_ms: f64,
  p99_ms: f64,
  mean_ms: f64,
  min_ms: f64,
  max_ms: f64,
  std_dev: f64,
  timestamp: DateTime
}
```

### ComparisonResult
```
struct ComparisonResult {
  test_name: String,
  rust_mean_ms: f64,
  python_mean_ms: f64,
  improvement_factor: f64,
  claimed_improvement: String,  # "10-25x"
  claim_validated: bool,
  rust_p50: f64,
  rust_p95: f64,
  rust_p99: f64,
  python_p50: f64,
  python_p95: f64,
  python_p99: f64
}
```

### ValidationReport
```
struct ValidationReport {
  timestamp: DateTime,
  categories: Vec<CategoryReport>,
  overall_validation: bool,
  summary: String
}

struct CategoryReport {
  name: String,           # "latency", "throughput", etc.
  tests: Vec<ComparisonResult>,
  overall_validation: bool
}
```

---

## Next Steps

With pseudocode complete, we proceed to:
1. **Phase 3: Architecture** - Create actual directory structure and files
2. **Phase 4: Refinement** - Implement code with TDD
3. **Phase 5: Completion** - Execute and validate

