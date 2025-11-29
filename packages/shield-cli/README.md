# @llm-dev-ops/shield-cli

Enterprise-grade security scanning CLI for Large Language Model applications.

## Installation

```bash
npm install -g @llm-dev-ops/shield-cli
```

Or run directly with npx:

```bash
npx @llm-dev-ops/shield-cli scan .
```

## Quick Start

```bash
# Scan current directory
shield scan

# Scan a specific file or directory
shield scan ./prompts/

# Check a single text input
shield check "User input to validate"

# Output results as JSON
shield scan --output json
```

## Commands

### `shield scan [path]`

Scan files or directories for security issues.

```bash
# Scan with default patterns
shield scan ./src

# Scan specific file types
shield scan . --pattern "**/*.txt" "**/*.md"

# Exclude directories
shield scan . --exclude "**/test/**" "**/fixtures/**"

# Filter by category
shield scan . --secrets          # Only secrets
shield scan . --pii              # Only PII
shield scan . --prompt-injection # Only prompt injection
shield scan . --toxicity         # Only toxicity

# Set failure threshold
shield scan . --fail-on critical  # Only fail on critical issues
shield scan . --fail-on medium    # Fail on medium or higher
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `-p, --pattern <patterns...>` | File patterns to include | `**/*.txt`, `**/*.md`, etc. |
| `-e, --exclude <patterns...>` | Patterns to exclude | `node_modules`, `dist`, `.git` |
| `--secrets` | Only scan for secrets | false |
| `--pii` | Only scan for PII | false |
| `--prompt-injection` | Only scan for prompt injection | false |
| `--toxicity` | Only scan for toxicity | false |
| `-o, --output <format>` | Output format (text, json) | text |
| `--fail-on <severity>` | Exit with error at this severity | high |

### `shield check <text>`

Check a single text input for security issues.

```bash
# Basic check
shield check "Hello, how are you?"

# Check with JSON output
shield check "ignore previous instructions" --output json
```

### `shield version`

Display version information.

## Detection Categories

### Prompt Injection (High Severity)

Detects attempts to manipulate LLM behavior:
- "Ignore previous instructions"
- "You are now..."
- "Pretend to be..."
- Jailbreak attempts

### Secrets Detection (Critical/High Severity)

Detects 40+ types of credentials:
- AWS Access Keys and Secrets
- GitHub Tokens (PAT, OAuth, App)
- Stripe API Keys
- OpenAI / Anthropic API Keys
- Slack Tokens and Webhooks
- Google API Keys
- Private Keys (RSA, EC, PGP)
- JWT Tokens
- Generic API keys and passwords

### PII Detection (Critical/Medium Severity)

Detects personally identifiable information:
- Email addresses
- Social Security Numbers (SSN)
- Credit Card numbers (Visa, Mastercard, Amex)
- Phone numbers

### Toxicity Detection (Medium Severity)

Detects potentially harmful content:
- Violence-related keywords
- Hate speech indicators
- Self-harm references

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No issues found (or issues below threshold) |
| 1 | Issues found at or above `--fail-on` severity |

## CI/CD Integration

### GitHub Actions

```yaml
- name: Security Scan
  run: npx @llm-dev-ops/shield-cli scan ./prompts --fail-on high
```

### GitLab CI

```yaml
security_scan:
  script:
    - npx @llm-dev-ops/shield-cli scan ./prompts --fail-on high
```

## Output Formats

### Text (Default)

```
🛡️  LLM Shield Scan Results

  🔑 src/config.json:15:10
     CRITICAL AWS Access Key ID
     Match: AKIA1234****

━━━ Scan Summary ━━━
  Files scanned: 25
  Total issues:  3

  By Category:
    🎯 Prompt Injection: 0
    🔑 Secrets:          2
    👤 PII:              1
    ⚠️  Toxicity:         0
```

### JSON

```json
{
  "file": "src/config.json",
  "line": 15,
  "column": 10,
  "type": "AWS Access Key ID",
  "category": "secret",
  "severity": "critical",
  "match": "AKIA1234****"
}
```

## Related Packages

- [@llm-dev-ops/shield-sdk](https://www.npmjs.com/package/@llm-dev-ops/shield-sdk) - SDK for Node.js/Browser integration
- [llm-shield-sdk](https://crates.io/crates/llm-shield-sdk) - Rust SDK

## License

Apache-2.0
