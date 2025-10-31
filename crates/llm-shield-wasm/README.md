# @llm-shield/wasm

Enterprise-grade LLM security toolkit compiled to WebAssembly.

Converted from [llm-guard](https://github.com/protectai/llm-guard) Python library to Rust for:
- 🚀 10-25x faster performance
- 🔒 Memory safety guarantees
- 🌐 Universal deployment (browsers, Node.js, edge)
- 📦 Small bundle size (<5MB optimized)

## Installation

```bash
npm install @llm-shield/wasm
```

## Quick Start

### Browser (ES Modules)

```javascript
import init, { BanSubstringsScanner } from '@llm-shield/wasm';

// Initialize WASM module
await init();

// Create a scanner
const scanner = new BanSubstringsScanner(['spam', 'scam', 'phishing']);

// Scan input
const result = await scanner.scan('This is a spam message');

if (!result.is_valid) {
  console.log(`Blocked! Risk score: ${result.risk_score}`);
}
```

### Node.js

```javascript
const { BanSubstringsScanner } = require('@llm-shield/wasm');

const scanner = new BanSubstringsScanner(['badword']);
const result = await scanner.scan('test badword');
console.log(result.is_valid); // false
```

### Multiple Scanners

```javascript
import init, { LlmShield } from '@llm-shield/wasm';

await init();

const shield = new LlmShield();

// Add scanners
shield.add_ban_substrings(['spam', 'scam']);

// Scan with all configured scanners
const results = await shield.scan_all('User input here');

// Check if any scanner flagged the input
const blocked = results.some(r => !r.is_valid);
```

## Available Scanners

### BanSubstrings
Detects and blocks specific substrings in input text.

```javascript
const scanner = new BanSubstringsScanner(
  ['badword1', 'badword2'],  // substrings to ban
  false                       // case_sensitive
);
```

**Use Cases:**
- Content moderation
- Brand protection
- Compliance enforcement

## API Reference

### `BanSubstringsScanner`

Constructor:
- `substrings: string[]` - Array of strings to detect
- `case_sensitive?: boolean` - Whether matching is case-sensitive (default: false)

Methods:
- `scan(input: string): Promise<ScanResult>` - Scan input text
- `name(): string` - Get scanner name
- `description(): string` - Get scanner description

### `ScanResult`

Properties:
- `sanitized_text: string` - Sanitized/redacted text
- `is_valid: boolean` - Whether input passed validation
- `risk_score: number` - Risk score from 0.0 (safe) to 1.0 (high risk)

Methods:
- `metadata_json(): string` - Get metadata as JSON
- `entity_count(): number` - Number of detected entities

### `LlmShield`

Main API for managing multiple scanners.

Methods:
- `add_ban_substrings(substrings: string[]): void` - Add BanSubstrings scanner
- `scan_all(input: string): Promise<ScanResult[]>` - Run all scanners
- `scanner_count(): number` - Get number of configured scanners

## Performance

Benchmarks vs Python llm-guard:

| Scanner | Python | Rust/WASM | Speedup |
|---------|--------|-----------|---------|
| BanSubstrings | 50µs | 5µs | 10x |
| Regex | 80µs | 8µs | 10x |
| TokenLimit | 100µs | 10µs | 10x |

## Browser Compatibility

- ✅ Chrome 87+
- ✅ Firefox 89+
- ✅ Safari 15.2+
- ✅ Edge 88+
- ✅ Node.js 14+

## Bundle Size

- Uncompressed: ~2MB
- Gzipped: ~500KB
- Brotli: ~400KB

## ML Infrastructure (Phase 8)

New in Phase 8: WASM bindings for ML infrastructure components.

### ModelRegistry

Manage model metadata and downloads:

```javascript
import { ModelRegistryWasm, ModelTaskWasm, ModelVariantWasm } from '@llm-shield/wasm';

const registry = ModelRegistryWasm.from_file('models/registry.json');

// Check model availability
const hasModel = registry.has_model(
    ModelTaskWasm.PromptInjection,
    ModelVariantWasm.FP16
);

// Get model metadata
const metadata = JSON.parse(
    registry.get_model_metadata_json(ModelTaskWasm.PromptInjection, ModelVariantWasm.FP16)
);

// Download model if needed
const modelPath = await registry.ensure_model_available(
    ModelTaskWasm.PromptInjection,
    ModelVariantWasm.FP16
);
```

### ResultCache

LRU caching with TTL support:

```javascript
import { ResultCacheWasm, CacheConfig } from '@llm-shield/wasm';

// Create cache with configuration
const cache = new ResultCacheWasm(CacheConfig.production());
// Production: 1000 entries, 1 hour TTL

// Or use other presets
const edgeCache = new ResultCacheWasm(CacheConfig.edge());
// Edge: 100 entries, 10 minutes TTL

// Cache a result
const key = ResultCacheWasm.hash_key("input text");
cache.insert(key, JSON.stringify(scanResult));

// Get from cache
const cached = cache.get(key);
if (cached) {
    const result = JSON.parse(cached);
}

// Get statistics
const stats = cache.stats();
console.log(`Hit rate: ${(stats.hit_rate() * 100).toFixed(2)}%`);
console.log(`Total requests: ${stats.total_requests()}`);
```

### Configuration Presets

```javascript
// Production: balanced performance
CacheConfig.production(); // 1000 entries, 1h TTL

// Edge/Mobile: memory-optimized
CacheConfig.edge(); // 100 entries, 10min TTL

// High-traffic: aggressive caching
CacheConfig.aggressive(); // 10000 entries, 2h TTL
```

### Documentation

- **[WASM_ML_INTEGRATION.md](./WASM_ML_INTEGRATION.md)**: Complete ML infrastructure guide
- **[Phase 8 Report](../../PHASE_8_WASM_INTEGRATION_REPORT.md)**: Implementation details

## License

MIT

## Contributing

Contributions welcome! See [GitHub repository](https://github.com/llm-shield/llm-shield-rs).

## Support

- 📖 [Documentation](https://llm-shield.dev)
- 💬 [Discord](https://discord.gg/llm-shield)
- 🐛 [Issues](https://github.com/llm-shield/llm-shield-rs/issues)
