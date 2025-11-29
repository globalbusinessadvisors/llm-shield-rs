# LLM Shield SDK

Enterprise-grade SDK for securing Large Language Model applications.

## Overview

LLM Shield SDK provides a comprehensive security toolkit for LLM applications, offering real-time scanning and protection against:

- **Prompt Injection**: Detects attempts to manipulate LLM behavior
- **Data Leakage**: Prevents exposure of sensitive information (PII, credentials)
- **Toxic Content**: Filters harmful, offensive, or inappropriate content
- **Code Injection**: Blocks malicious code in prompts
- **Jailbreak Attempts**: Identifies attempts to bypass safety measures

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-shield-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use llm_shield_sdk::prelude::*;

#[tokio::main]
async fn main() -> SdkResult<()> {
    // Create a shield with standard security level
    let shield = Shield::standard()?;

    // Scan a prompt before sending to LLM
    let result = shield.scan_prompt("Hello, how are you?").await?;

    if result.is_valid {
        println!("Prompt is safe to send to LLM");
    } else {
        println!("Security risk detected: {:?}", result.risk_factors);
    }

    Ok(())
}
```

## Security Presets

The SDK provides three built-in security presets:

### Strict
Maximum security for regulated industries (banking, healthcare):
- All 22 scanners enabled
- Low risk tolerance (short-circuit at 0.7)
- Sequential execution for deterministic results

```rust
let shield = Shield::strict()?;
```

### Standard (Recommended)
Balanced security for general-purpose applications:
- Core input scanners (secrets, toxicity, prompt injection)
- Core output scanners (sensitive data, malicious URLs)
- Moderate risk tolerance (short-circuit at 0.9)
- Parallel execution enabled

```rust
let shield = Shield::standard()?;
```

### Permissive
Minimal security for development/testing:
- Essential scanners only
- High risk tolerance
- Fast execution

```rust
let shield = Shield::permissive()?;
```

## Custom Configuration

For fine-grained control, use the builder pattern:

```rust
let shield = Shield::builder()
    // Add specific input scanners
    .add_input_scanner(BanSubstrings::with_substrings(["password", "secret"])?)
    .add_input_scanner(Secrets::default_config()?)
    .add_input_scanner(Toxicity::default_config()?)
    // Add output scanners
    .add_output_scanner(Sensitive::default_config()?)
    // Configure behavior
    .with_short_circuit(0.9) // Stop on high-risk detection
    .with_parallel_execution(true) // Run scanners in parallel
    .with_max_concurrent(8) // Max concurrent scanners
    .build()?;
```

## Scanning Methods

### Scan Prompts (Before LLM)

```rust
let result = shield.scan_prompt("User input here").await?;

if result.is_valid {
    // Safe to send to LLM
    let llm_response = call_llm(&result.sanitized_text).await?;
} else {
    // Handle security risk
    for factor in &result.risk_factors {
        println!("Risk: {} (severity: {:?})", factor.description, factor.severity);
    }
}
```

### Scan Outputs (After LLM)

```rust
let result = shield.scan_output(&llm_response).await?;

if result.is_valid {
    // Safe to show to user
    display_to_user(&result.sanitized_text);
} else {
    // Handle security risk
    display_error("Response filtered for security reasons");
}
```

### Batch Scanning

For high-throughput scenarios:

```rust
let prompts = vec!["Hello", "How are you?", "What's the weather?"];
let results = shield.scan_batch(&prompts).await?;

for (prompt, result) in prompts.iter().zip(results.iter()) {
    println!("{}: {}", prompt, if result.is_valid { "safe" } else { "risky" });
}
```

## Available Scanners

### Input Scanners (12)

| Scanner | Description |
|---------|-------------|
| `BanSubstrings` | Block specific words/phrases |
| `BanCode` | Detect programming language code |
| `BanCompetitors` | Filter competitor mentions |
| `Secrets` | Detect 40+ types of API keys/tokens |
| `Toxicity` | ML-based toxicity detection |
| `PromptInjection` | Detect injection attacks |
| `InvisibleText` | Detect hidden unicode characters |
| `Gibberish` | Detect nonsensical text |
| `Language` | Enforce language requirements |
| `TokenLimit` | Enforce token limits |
| `RegexScanner` | Custom regex patterns |
| `Sentiment` | Analyze text sentiment |

### Output Scanners (10)

| Scanner | Description |
|---------|-------------|
| `Sensitive` | Detect PII (emails, SSNs, credit cards) |
| `NoRefusal` | Detect over-cautious refusals |
| `Relevance` | Ensure response relevance |
| `BanTopics` | Filter prohibited topics |
| `Bias` | Detect biased content |
| `MaliciousURLs` | Detect phishing/malware URLs |
| `Factuality` | Assess factual confidence |
| `ReadingTime` | Validate response length |
| `URLReachability` | Verify URL accessibility |
| `RegexOutput` | Custom output patterns |

## Scanner Factory

Create scanners with convenience methods:

```rust
use llm_shield_sdk::scanner_factory::{InputScannerFactory, OutputScannerFactory};

// Input scanners
let secrets = InputScannerFactory::secrets()?;
let toxicity = InputScannerFactory::toxicity()?;
let ban_words = InputScannerFactory::ban_substrings(["banned", "words"])?;
let token_limit = InputScannerFactory::token_limit_with_max(4096)?;

// Output scanners
let sensitive = OutputScannerFactory::sensitive()?;
let malicious_urls = OutputScannerFactory::malicious_urls()?;
```

## Result Structure

```rust
pub struct ScanResult {
    pub is_valid: bool,           // Whether scan passed
    pub risk_score: f32,          // 0.0-1.0
    pub sanitized_text: String,   // Sanitized input
    pub entities: Vec<Entity>,    // Detected entities
    pub risk_factors: Vec<RiskFactor>,
    pub severity: Severity,
    pub metadata: HashMap<String, String>,
}

pub struct Entity {
    pub entity_type: String,      // "email", "ssn", "api_key", etc.
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,          // 0.0-1.0
}

pub enum Severity {
    None,
    Low,
    Medium,
    High,
    Critical,
}
```

## Performance

- **Sub-millisecond** scanning for most inputs
- **Zero-copy** text processing
- **Parallel execution** support
- **Short-circuit evaluation** for early termination

## Feature Flags

```toml
[dependencies]
llm-shield-sdk = { version = "0.1", features = ["all-scanners"] }

# Available features:
# - all-scanners (default): All scanner types
# - input-scanners: Only input scanners
# - output-scanners: Only output scanners
# - cloud-aws: AWS integration
# - cloud-gcp: GCP integration
# - cloud-azure: Azure integration
```

## Enterprise Features

- **Thread-Safe**: Safe for concurrent use across threads
- **Async-First**: Full async/await support
- **Composable**: Chain multiple scanners
- **Observable**: Built-in tracing and metrics
- **GDPR/HIPAA Compliant**: PII detection and redaction

## License

MIT OR Apache-2.0
