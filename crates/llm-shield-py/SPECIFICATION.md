# Phase 12: Python Bindings - Specification (SPARC-S)

## 1. Functional Requirements

### 1.1 Core Scanner API
```python
# Input Scanner
scanner = PromptInjection(threshold=0.7)
result = scanner.scan("user input")
assert isinstance(result, ScanResult)
assert result.is_valid in [True, False]
assert 0.0 <= result.risk_score <= 1.0

# Output Scanner
scanner = Sensitive()
result = scanner.scan_output(prompt="question", output="answer")
```

### 1.2 Required Types
- `ScanResult`: Result object with `sanitized_input`, `is_valid`, `risk_score`, `entities`
- `Vault`: Thread-safe state management
- `Entity`: Detected entities (PII, secrets, etc.)
- Custom exceptions: `LLMShieldError`, `ScannerError`, `ConfigError`, `ModelError`

### 1.3 Scanners to Implement
**Input Scanners (12):**
1. BanSubstrings
2. Secrets
3. PromptInjection
4. Toxicity
5. Gibberish
6. InvisibleText
7. Language
8. TokenLimit
9. BanCompetitors
10. Sentiment
11. BanCode
12. Regex

**Output Scanners (10):**
1. NoRefusal
2. Relevance
3. Sensitive
4. BanTopics
5. Bias
6. MaliciousURLs
7. ReadingTime
8. Factuality
9. URLReachability
10. RegexOutput

### 1.4 Async Support
```python
# Async API
result = await scanner.scan_async("user input")

# Batch processing
results = await scanner.scan_batch(["input1", "input2", "input3"])
```

## 2. Non-Functional Requirements

### 2.1 Performance
- Scanner initialization: <2ms
- Single scan latency: <1ms (heuristic), <50ms (ML)
- Throughput: >1,000 scans/sec
- Memory: <200MB baseline

### 2.2 Compatibility
- Python versions: 3.8, 3.9, 3.10, 3.11, 3.12, 3.13
- Platforms: Linux x86_64/aarch64, macOS x86_64/ARM64, Windows x64
- API compatibility: 95%+ with llm-guard

### 2.3 Quality
- Test coverage: >90%
- Type hints: 100% coverage
- Documentation: Complete API docs with examples
- Zero memory leaks

## 3. Build System Requirements

### 3.1 Maturin Configuration
- ABI3 wheels for Python 3.8+
- Stripped binaries (<25MB per wheel)
- Optimized for release builds

### 3.2 Development Workflow
```bash
# Development install
maturin develop --release

# Run tests
pytest python/tests -v

# Build wheels
maturin build --release
```

## 4. Testing Requirements

### 4.1 Test Pyramid
- Unit tests: 60+ (pytest)
- Integration tests: 30+ (Rust ↔ Python boundary)
- Property tests: 50+ (Hypothesis)
- E2E tests: 10+ (FastAPI, Django integration)

### 4.2 Test Coverage
- All scanners: 100% coverage
- Error handling: 100% coverage
- Type conversions: 100% coverage
- Async support: 100% coverage

## 5. Documentation Requirements

### 5.1 API Documentation
- Comprehensive docstrings for all public APIs
- Type hints for all functions and methods
- Usage examples for each scanner

### 5.2 Examples
- Basic usage (5+ examples)
- Async usage (3+ examples)
- FastAPI integration (1 example)
- Django integration (1 example)
- Migration guide from llm-guard

## 6. Acceptance Criteria

✅ All 22 scanners working in Python
✅ >90% test coverage
✅ <2ms scan latency (p50)
✅ All tests passing on all platforms
✅ Complete documentation
✅ Zero mypy errors with --strict
✅ Published on PyPI
