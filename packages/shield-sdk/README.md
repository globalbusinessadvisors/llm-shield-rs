# @llm-dev-ops/shield-sdk

Enterprise-grade SDK for securing Large Language Model applications.

## Overview

Shield SDK provides comprehensive security scanning for LLM applications, protecting against:

- **Prompt Injection** - Detects attempts to manipulate LLM behavior
- **Data Leakage** - Prevents exposure of secrets, API keys, and credentials
- **PII Exposure** - Identifies personally identifiable information
- **Toxic Content** - Filters harmful, offensive, or inappropriate content

## Installation

```bash
npm install @llm-dev-ops/shield-sdk
```

## Quick Start

```typescript
import { Shield } from '@llm-dev-ops/shield-sdk';

// Create a shield with standard security level
const shield = Shield.standard();

// Scan a prompt before sending to LLM
const result = await shield.scanPrompt("Hello, how are you?");

if (result.isValid) {
  console.log("Prompt is safe to send to LLM");
} else {
  console.log("Security risk detected:", result.riskFactors);
}
```

## Security Presets

### Strict

Maximum security for regulated industries (banking, healthcare):

```typescript
const shield = Shield.strict();
```

- All scanners enabled
- Low risk tolerance (short-circuit at 0.7)
- Sequential execution for deterministic results

### Standard (Recommended)

Balanced security for general-purpose applications:

```typescript
const shield = Shield.standard();
```

- Core scanners (secrets, PII, prompt injection)
- Moderate risk tolerance (short-circuit at 0.9)
- Parallel execution enabled

### Permissive

Minimal security for development/testing:

```typescript
const shield = Shield.permissive();
```

- Essential scanners only
- High risk tolerance
- Fast execution

## Custom Configuration

Use the builder pattern for fine-grained control:

```typescript
import {
  Shield,
  SecretsScanner,
  PIIScanner,
  PromptInjectionScanner
} from '@llm-dev-ops/shield-sdk';

const shield = Shield.builder()
  .addInputScanner(new PromptInjectionScanner())
  .addInputScanner(new SecretsScanner({ secretTypes: ['aws', 'github'] }))
  .addInputScanner(new PIIScanner({ piiTypes: ['email', 'ssn'] }))
  .addOutputScanner(new PIIScanner())
  .withShortCircuit(0.8)
  .withParallelExecution(true)
  .withMaxConcurrent(4)
  .build();
```

## Scanning Methods

### Scan Prompts (Before LLM)

```typescript
const result = await shield.scanPrompt("User input here");

if (result.isValid) {
  // Safe to send to LLM
  const llmResponse = await callLLM(result.sanitizedText);
} else {
  // Handle security risk
  for (const factor of result.riskFactors) {
    console.log(`Risk: ${factor.description} (${factor.severity})`);
  }
}
```

### Scan Outputs (After LLM)

```typescript
const result = await shield.scanOutput(llmResponse);

if (result.isValid) {
  // Safe to show to user
  displayToUser(result.sanitizedText);
} else {
  displayError("Response filtered for security reasons");
}
```

### Scan Both Prompt and Output

```typescript
const { promptResult, outputResult } = await shield.scanPromptAndOutput(
  userInput,
  llmResponse
);
```

### Batch Scanning

```typescript
const prompts = ["Hello", "How are you?", "What's the weather?"];
const results = await shield.scanBatch(prompts);

for (const [prompt, result] of prompts.map((p, i) => [p, results[i]])) {
  console.log(`${prompt}: ${result.isValid ? 'safe' : 'risky'}`);
}
```

## Available Scanners

### PromptInjectionScanner

Detects attempts to manipulate LLM behavior:

```typescript
import { PromptInjectionScanner } from '@llm-dev-ops/shield-sdk';

const scanner = new PromptInjectionScanner({
  customPatterns: [/my-custom-pattern/i],
  detectJailbreaks: true,
  detectRolePlay: true,
});
```

**Detects:**
- Instruction override attempts ("ignore previous instructions")
- Role manipulation ("pretend to be", "act as")
- System prompt attacks
- Jailbreak patterns (DAN mode, etc.)
- Delimiter injection

### SecretsScanner

Detects 40+ types of credentials:

```typescript
import { SecretsScanner } from '@llm-dev-ops/shield-sdk';

const scanner = new SecretsScanner({
  secretTypes: ['aws', 'github', 'stripe', 'openai'],
  redact: true,
});
```

**Detects:**
- AWS Access Keys and Secrets
- GitHub Tokens (PAT, OAuth, App)
- Stripe API Keys
- OpenAI / Anthropic API Keys
- Slack Tokens and Webhooks
- Google API Keys
- Private Keys (RSA, EC, PGP)
- JWT Tokens
- Generic API keys and passwords

### PIIScanner

Detects personally identifiable information:

```typescript
import { PIIScanner } from '@llm-dev-ops/shield-sdk';

const scanner = new PIIScanner({
  piiTypes: ['email', 'phone', 'ssn', 'credit-card'],
  redact: true,
});
```

**Detects:**
- Email addresses
- Phone numbers (US, UK, international)
- Social Security Numbers
- Credit Card numbers (with Luhn validation)
- IP addresses
- Passport numbers
- Driver's license numbers

### ToxicityScanner

Detects harmful content:

```typescript
import { ToxicityScanner } from '@llm-dev-ops/shield-sdk';

const scanner = new ToxicityScanner({
  categories: ['violence', 'hate', 'harassment', 'self-harm'],
  sensitivity: 0.5,
  customKeywords: ['banned-word'],
  allowedKeywords: ['exception-word'],
});
```

**Detects:**
- Violence-related content
- Hate speech
- Harassment
- Self-harm references
- Sexual content
- Profanity

## Result Structure

```typescript
interface ScanResult {
  isValid: boolean;        // Whether scan passed
  riskScore: number;       // 0.0-1.0
  sanitizedText: string;   // Sanitized input
  entities: Entity[];      // Detected entities
  riskFactors: RiskFactor[];
  severity: Severity;      // 'none' | 'low' | 'medium' | 'high' | 'critical'
  metadata: Record<string, string>;
  durationMs: number;      // Scan duration
}

interface Entity {
  entityType: string;      // 'email', 'ssn', 'api_key', etc.
  text: string;
  start: number;
  end: number;
  confidence: number;      // 0.0-1.0
}
```

## LangChain Integration

```typescript
import { Shield } from '@llm-dev-ops/shield-sdk';
import { ChatOpenAI } from '@langchain/openai';

const shield = Shield.standard();
const llm = new ChatOpenAI();

async function safeChat(userInput: string) {
  // Scan input
  const inputResult = await shield.scanPrompt(userInput);
  if (!inputResult.isValid) {
    throw new Error('Unsafe input detected');
  }

  // Call LLM
  const response = await llm.invoke(inputResult.sanitizedText);

  // Scan output
  const outputResult = await shield.scanOutput(response.content);
  if (!outputResult.isValid) {
    throw new Error('Unsafe output detected');
  }

  return outputResult.sanitizedText;
}
```

## OpenAI Integration

```typescript
import { Shield } from '@llm-dev-ops/shield-sdk';
import OpenAI from 'openai';

const shield = Shield.standard();
const openai = new OpenAI();

async function safeChatCompletion(userMessage: string) {
  const inputResult = await shield.scanPrompt(userMessage);
  if (!inputResult.isValid) {
    return { error: 'Input blocked', riskFactors: inputResult.riskFactors };
  }

  const completion = await openai.chat.completions.create({
    model: 'gpt-4',
    messages: [{ role: 'user', content: inputResult.sanitizedText }],
  });

  const response = completion.choices[0].message.content;
  const outputResult = await shield.scanOutput(response);

  return {
    response: outputResult.sanitizedText,
    inputScan: inputResult,
    outputScan: outputResult,
  };
}
```

## Performance

- **Sub-millisecond** scanning for most inputs
- **Parallel execution** support
- **Short-circuit evaluation** for early termination
- **Batch processing** for high-throughput scenarios

## Related Packages

- [@llm-dev-ops/shield-cli](https://www.npmjs.com/package/@llm-dev-ops/shield-cli) - Command-line interface
- [llm-shield-sdk](https://crates.io/crates/llm-shield-sdk) - Rust SDK

## License

Apache-2.0
