# LLM Shield Benchmark Results

Generated: 2025-12-01 (Initial Setup)

## Summary

This is the canonical benchmark output directory for LLM Shield.

- **Total Benchmarks**: 7 registered targets
- **Successful**: Pending execution
- **Failed**: N/A

## Canonical Benchmark Interface

LLM Shield implements the canonical benchmark interface compatible with all 25 benchmark-target repositories:

### BenchmarkResult Struct

```rust
pub struct BenchmarkResult {
    pub target_id: String,
    pub metrics: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
```

### BenchTarget Trait

```rust
#[async_trait]
pub trait BenchTarget: Send + Sync {
    fn id(&self) -> String;
    async fn run(&self) -> Result<serde_json::Value, BenchTargetError>;
}
```

### Available Targets

| Target ID | Description |
|-----------|-------------|
| llm-shield/latency-simple | Measures latency for simple, short prompts |
| llm-shield/latency-complex | Measures latency for complex, long prompts with code |
| llm-shield/latency-secrets | Measures latency for secrets detection in text |
| llm-shield/throughput | Measures throughput in requests per second |
| llm-shield/memory | Measures memory usage during scanning |
| llm-shield/scanner-secrets | Benchmarks the secrets detection scanner |
| llm-shield/scanner-prompt-injection | Benchmarks the prompt injection detection scanner |

## Running Benchmarks

```bash
# Run all benchmarks
cargo run --bin llm-shield-bench -- run

# Run a specific target
cargo run --bin llm-shield-bench -- run --target llm-shield/latency-simple

# List available targets
cargo run --bin llm-shield-bench -- list

# Show previous results
cargo run --bin llm-shield-bench -- show
```

## Output Structure

```
benchmarks/output/
├── results.json      # Combined results file
├── summary.md        # This file
└── raw/              # Individual result files
    ├── llm-shield_latency-simple.json
    ├── llm-shield_latency-complex.json
    └── ...
```

---

*This benchmark interface is compatible with all 25 benchmark-target repositories.*
*LLM Shield complies with the canonical benchmark interface specification.*
