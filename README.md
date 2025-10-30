# LLM Shield (Rust) - Analysis & Conversion Documentation

## 📋 Repository Contents

This repository contains comprehensive analysis and planning documentation for converting the Python-based [LLM Guard](https://github.com/protectai/llm-guard) library to Rust.

### 📚 Documents

1. **[LLM_GUARD_ANALYSIS_REPORT.md](LLM_GUARD_ANALYSIS_REPORT.md)** (Primary Document)
   - Comprehensive 1,381-line analysis report
   - Architecture deep-dive
   - Dependency mapping
   - Conversion complexity assessment
   - 8-12 month roadmap
   - Risk assessment and mitigation strategies

2. **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** (Developer Quick Start)
   - Scanner conversion priority matrix
   - Critical Rust crate dependencies
   - ONNX conversion commands
   - Code examples and patterns
   - Performance targets
   - Timeline estimates

3. **[TECHNICAL_DECISIONS.md](TECHNICAL_DECISIONS.md)** (Architecture Decisions)
   - ML inference backend selection (ONNX vs Candle vs PyO3)
   - Web framework choice (Axum)
   - Error handling strategy (thiserror + anyhow)
   - Logging approach (tracing)
   - Testing strategy
   - Migration phases

---

## 🎯 Executive Summary

**Project:** Convert LLM Guard from Python to Rust  
**Scope:** 17 input scanners + 24 output scanners + API  
**Timeline:** 8-12 months (2-3 FTE)  
**Feasibility:** HIGH ✅  
**Expected Benefits:**
- 4-10x faster inference
- 2-3x lower memory usage
- Sub-second cold starts
- Type safety and better maintainability

---

## 🔍 What is LLM Guard?

LLM Guard is a comprehensive security toolkit for Large Language Model interactions:
- **Sanitization:** Remove PII, secrets, toxic content
- **Detection:** Identify prompt injection, malicious URLs, bias
- **Prevention:** Block topics, competitors, code execution
- **Compliance:** Token limits, language detection, sentiment analysis

**Original Repository:** https://github.com/protectai/llm-guard  
**License:** MIT  
**Stars:** 1.8k+  
**Language:** Python 3.10-3.12

---

## 📊 Key Metrics (Python Version)

| Metric | Value |
|--------|-------|
| **Total Python Files** | 217 |
| **Core Module LOC** | ~9,000 |
| **Input Scanners** | 17 types |
| **Output Scanners** | 24 types |
| **Secret Plugins** | 95 custom detectors |
| **ML Models Used** | 15+ HuggingFace models |
| **Dependencies** | 15 core packages |

---

## 🏗️ Architecture Overview

```
llm_guard/
├── Core
│   ├── evaluate.py          # Scan orchestration
│   ├── model.py             # Model config
│   ├── util.py              # Utilities
│   └── transformers_helpers.py  # ML helpers
│
├── Input Scanners (17)
│   ├── Anonymize           # PII detection (NER + regex)
│   ├── PromptInjection     # DeBERTa classification
│   ├── Toxicity            # RoBERTa multi-label
│   ├── Secrets             # 95 secret detectors
│   ├── TokenLimit          # tiktoken counting
│   └── ...
│
├── Output Scanners (24)
│   ├── Deanonymize         # PII restoration
│   ├── NoRefusal           # Refusal detection
│   ├── Relevance           # Semantic similarity
│   ├── FactualConsistency  # NLI checking
│   └── ...
│
└── API (FastAPI)
    └── REST endpoints for scanning
```

---

## 🚀 Conversion Strategy

### Phase 1: Foundation (2-3 months)
- ✅ Core infrastructure (model config, errors, logging)
- ✅ Simple scanners (7 types, no ML)
  - BanSubstrings, Regex, InvisibleText
  - TokenLimit, JSON, ReadingTime, URLReachability
- ✅ Basic REST API (Axum)

**Deliverable:** Working Rust library with non-ML scanners

### Phase 2: ONNX Integration (2-3 months)
- ✅ ONNX Runtime setup (ort crate)
- ✅ Model conversion pipeline
- ✅ ML-based scanners (8-10 types)
  - PromptInjection, Toxicity, Sentiment
  - Code, BanTopics, NoRefusal, Relevance

**Deliverable:** ML inference working via ONNX

### Phase 3: Complex Scanners (3-4 months)
- ✅ Secret detection (95 plugins)
- ✅ PII detection (NER + regex)
- ✅ Remaining scanners
- ✅ Feature parity with Python

**Deliverable:** Complete port

### Phase 4: Optimization (1-2 months)
- ✅ Performance tuning
- ✅ Memory optimization
- ✅ Production deployment

**Deliverable:** Production-ready system

---

## 🔧 Tech Stack

### Core Libraries
```toml
[dependencies]
# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Async
tokio = { version = "1", features = ["full"] }
rayon = "1.10"

# ML
ort = "2.0"                  # ONNX Runtime
hf-hub = "0.3"               # Model downloads
candle-core = "0.8"          # Optional native ML

# Text
regex = "1.11"
tiktoken-rs = "0.5"
unicode-segmentation = "1.12"

# Web
axum = "0.8"
tower = "0.5"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### ML Backend Decision
**Primary:** ONNX Runtime (production-ready now)  
**Future:** Candle (HuggingFace native Rust)  
**Fallback:** PyO3 (Python interop for edge cases)

---

## 📈 Expected Performance

| Metric | Python | Rust (Target) | Improvement |
|--------|--------|---------------|-------------|
| **Latency** | 200-500ms | <50ms | 4-10x |
| **Throughput** | 100/sec | 1000+/sec | 10x |
| **Memory** | 4-8GB | <2GB | 2-4x |
| **Cold Start** | 10-30s | <5s | 2-6x |
| **Docker Image** | 3-5GB | <1GB | 3-5x |

---

## ⚠️ Critical Challenges

### 1. ML Model Inference (HIGH)
**Challenge:** Python's transformers is standard, Rust ML ecosystem is maturing  
**Solution:** ONNX Runtime initially, migrate to Candle gradually  
**Effort:** 3-6 months

### 2. Presidio PII Detection (HIGH)
**Challenge:** No Rust equivalent of Microsoft's Presidio library  
**Solution:** Custom implementation (NER via ONNX + regex)  
**Effort:** 2-4 months

### 3. Secret Detection (MEDIUM)
**Challenge:** 95 custom plugins to port  
**Solution:** Manual port with TOML config system  
**Effort:** 2-3 weeks

### 4. Model Downloading (LOW)
**Challenge:** HuggingFace Hub integration  
**Solution:** Use `hf-hub` crate (official)  
**Effort:** 1-2 weeks

---

## 📋 Scanner Conversion Checklist

### ✅ Tier 1: Simple (No ML) - Start Here
- [ ] BanSubstrings (2 days)
- [ ] Regex (1 day)
- [ ] InvisibleText (1 day)
- [ ] TokenLimit (2 days)
- [ ] JSON (1 day)
- [ ] ReadingTime (1 day)
- [ ] URLReachability (1 day)

**Total: 1-2 weeks**

### ⏳ Tier 2: ML via ONNX
- [ ] PromptInjection (5 days)
- [ ] Toxicity (4 days)
- [ ] Sentiment (3 days)
- [ ] Code (4 days)
- [ ] BanTopics (5 days)
- [ ] NoRefusal (4 days)
- [ ] Relevance (5 days)
- [ ] Language (3 days)

**Total: 2-3 months**

### 🔥 Tier 3: Complex
- [ ] Secrets (3 weeks)
- [ ] Anonymize (4 weeks)
- [ ] Deanonymize (2 weeks)
- [ ] Gibberish (2 weeks)
- [ ] FactualConsistency (2 weeks)

**Total: 3-4 months**

---

## 🎓 Learning Resources

### Rust ML
- [Candle Documentation](https://github.com/huggingface/candle)
- [ONNX Runtime Rust](https://docs.rs/ort/)
- [HuggingFace Hub Rust](https://docs.rs/hf-hub/)
- [Are We Learning Yet?](https://www.arewelearningyet.com/)

### Rust Web
- [Axum Book](https://docs.rs/axum/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Tower Guide](https://docs.rs/tower/)

### General Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)

---

## 🔗 Quick Links

- **Original Python Repo:** https://github.com/protectai/llm-guard
- **Documentation:** https://protectai.github.io/llm-guard/
- **Playground:** https://huggingface.co/spaces/ProtectAI/llm-guard-playground
- **Slack Community:** https://mlsecops.com/slack

---

## 📝 Next Steps

1. **Read the full analysis:** [LLM_GUARD_ANALYSIS_REPORT.md](LLM_GUARD_ANALYSIS_REPORT.md)
2. **Review technical decisions:** [TECHNICAL_DECISIONS.md](TECHNICAL_DECISIONS.md)
3. **Check quick reference:** [QUICK_REFERENCE.md](QUICK_REFERENCE.md)
4. **Set up development environment:**
   ```bash
   # Clone Python repo for reference
   git clone https://github.com/protectai/llm-guard /tmp/llm-guard
   
   # Create Rust project
   cargo new --lib llm-guard-rs
   cd llm-guard-rs
   
   # Add basic dependencies
   cargo add anyhow thiserror tracing tokio axum
   ```
5. **Start with simplest scanner:** Implement `BanSubstrings` first
6. **Build confidence with tests:** Achieve >80% coverage
7. **Iterate and expand:** Add more scanners incrementally

---

## 📊 Success Criteria

### Must Have
- ✅ Same accuracy as Python (±1%)
- ✅ >80% test coverage
- ✅ Type-safe API
- ✅ Production-ready logging
- ✅ Docker deployment
- ✅ API compatibility

### Nice to Have
- ⭐ 4x faster than Python
- ⭐ 100% Candle (no ONNX)
- ⭐ WebAssembly support
- ⭐ Plugin system

---

## 📄 License

This analysis and planning documentation is provided under MIT license.

The original LLM Guard project is licensed under MIT.

---

## 🤝 Contributing

This is a planning repository. For the actual implementation:
1. Review all documentation
2. Set up Rust development environment
3. Start with Phase 1 (simple scanners)
4. Submit PRs with comprehensive tests
5. Maintain documentation

---

## 📧 Contact

- **Original Project:** https://github.com/protectai/llm-guard
- **Analysis Date:** 2025-10-30
- **Analyst:** Claude Code Repository Analyst

---

**Status:** ✅ Analysis Complete - Ready for Implementation  
**Last Updated:** 2025-10-30  
**Documents:** 4 comprehensive guides (2,175 total lines)
