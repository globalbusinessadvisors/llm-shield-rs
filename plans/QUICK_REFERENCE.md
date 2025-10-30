# LLM-Guard Rust Conversion - Quick Reference

## One-Page Overview

### Project Goal
Convert llm-guard from Python to Rust/WASM for 2-5x performance improvement and browser deployment.

### Timeline: 32 Weeks (8 Months)

```
Week 1-4    | Foundation        | Core types, traits, utilities
Week 5-12   | Scanners          | Rule-based detection algorithms
Week 13-20  | ML Integration    | ONNX models, transformers
Week 21-24  | API Layer         | REST API, WASM bindings
Week 25-28  | Testing           | Validation, security audit
Week 29-32  | Optimization      | Performance tuning, deployment
```

### Team: 3-5 Engineers
- 1x Tech Lead (Rust + ML)
- 2x Rust Engineers
- 1x ML Engineer
- 1x QA Engineer (optional)

### Budget: ~$470K
- Labor: $456K
- Infrastructure: $14K

---

## Phase Checklist

### ✅ Phase 1: Foundation (Weeks 1-4)
- [ ] Core types (ScanResult, ScanError)
- [ ] Scanner traits (Scanner, InputScanner, OutputScanner)
- [ ] Configuration system
- [ ] Basic utilities (text processing, regex)
- [ ] Test infrastructure

### ✅ Phase 2: Scanners (Weeks 5-12)
- [ ] BanSubstrings, BanCode, BanCompetitors
- [ ] Regex, Secrets, InvisibleText
- [ ] TokenLimit, Gibberish
- [ ] Code detection, URL validation
- [ ] Python test suite ported

### ✅ Phase 3: ML Integration (Weeks 13-20)
- [ ] ONNX model conversion pipeline
- [ ] PromptInjection scanner
- [ ] Toxicity scanner
- [ ] Anonymize (NER-based)
- [ ] Accuracy validation (<0.5% deviation)

### ✅ Phase 4: API Layer (Weeks 21-24)
- [ ] Pipeline orchestration
- [ ] REST API (Axum)
- [ ] WASM bindings
- [ ] Configuration management
- [ ] API documentation

### ✅ Phase 5: Testing (Weeks 25-28)
- [ ] Test suite migration complete
- [ ] Performance benchmarks met
- [ ] Security audit passed
- [ ] Cross-platform testing
- [ ] WASM compatibility verified

### ✅ Phase 6: Optimization (Weeks 29-32)
- [ ] Performance optimization
- [ ] Documentation complete
- [ ] Docker images
- [ ] CI/CD pipeline
- [ ] Release v1.0.0

---

## Key Technologies

| Category | Technology |
|----------|-----------|
| Language | Rust 1.75+ |
| ML | ONNX Runtime, Candle |
| Tokenization | HuggingFace tokenizers |
| Text | regex, aho-corasick |
| Web | Axum (REST), wasm-bindgen |
| Testing | criterion, proptest |

---

## Performance Targets

| Metric | Python | Rust | Gain |
|--------|--------|------|------|
| Rule-based | 5ms | 1ms | 5x |
| ML scanners | 50ms | 25ms | 2x |
| Memory | 100MB | 10MB | 10x |
| WASM bundle | N/A | 5MB | NEW |

---

## Success Metrics

### Technical
- ✅ 2-5x performance improvement
- ✅ <0.5% accuracy deviation
- ✅ >80% test coverage
- ✅ Zero critical vulnerabilities

### Business
- ✅ 100+ downloads (Month 1)
- ✅ 5+ production deployments (Q1)
- ✅ 10+ contributors (Q1)

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| ML accuracy loss | Extensive validation suite |
| ONNX WASM issues | Candle as fallback |
| Timeline slip | Incremental delivery |
| Team expertise | Training + consultants |

---

## Quick Commands

### Setup
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install tools
cargo install wasm-pack cargo-audit cargo-tarpaulin
```

### Build
```bash
# Build library
cargo build --release

# Build WASM
wasm-pack build --target web --release

# Run tests
cargo test

# Benchmark
cargo bench
```

### Convert Models
```bash
# Python script
python scripts/convert_models.py

# Verify ONNX
python -c "import onnx; onnx.checker.check_model('model.onnx')"
```

---

## Document Navigation

| Document | Purpose |
|----------|---------|
| [README.md](./README.md) | Start here - Document index |
| [EXECUTIVE_SUMMARY.md](./EXECUTIVE_SUMMARY.md) | For stakeholders & decision makers |
| [CONVERSION_STRATEGY.md](./CONVERSION_STRATEGY.md) | Complete conversion plan |
| [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) | Code examples & patterns |
| [TECHNICAL_REFERENCE.md](./TECHNICAL_REFERENCE.md) | Deep technical details |

---

## Critical Paths

### Week 4 Decision Point
- ✅ Core framework operational?
- ✅ First scanner working?
- ✅ Test infrastructure ready?

### Week 12 Decision Point
- ✅ All scanners implemented?
- ✅ ONNX conversion tested?
- ✅ Performance targets met?

### Week 20 Decision Point
- ✅ ML accuracy validated?
- ✅ WASM ML working?
- ✅ Ready for beta release?

### Week 28 Go/No-Go
- ✅ All tests passing?
- ✅ Security audit complete?
- ✅ Documentation ready?
- ✅ Production deployment tested?

---

## Release Schedule

```
Week 12  | v0.1.0-alpha   | Core scanners
Week 20  | v0.2.0-beta    | ML support
Week 28  | v1.0.0-rc1     | Release candidate
Week 32  | v1.0.0         | Production release
```

---

## Contact & Support

| Need | Resource |
|------|----------|
| Project questions | Tech Lead |
| Technical issues | GitHub Issues |
| Implementation help | IMPLEMENTATION_GUIDE.md |
| Architecture decisions | TECHNICAL_REFERENCE.md |

---

**Version:** 1.0
**Last Updated:** 2025-01-30
**Status:** Quick Reference Guide
