# LLM Guard Rust Conversion - Project Roadmap

## 🎯 Mission
Convert LLM Guard from Python to Rust, achieving 4-10x performance improvement while maintaining feature parity and improving type safety.

---

## 📅 Timeline Overview

```
Month 1-3:   Foundation & Simple Scanners
Month 4-6:   ONNX Integration & ML Scanners  
Month 7-9:   Complex Scanners & PII Detection
Month 10-12: Optimization & Production Readiness

Total: 8-12 months (2-3 FTE)
```

---

## Phase 1: Foundation (Months 1-3)

### Goals
- ✅ Establish core infrastructure
- ✅ Implement 7 simple scanners (no ML)
- ✅ Basic REST API
- ✅ CI/CD pipeline
- ✅ Testing framework

### Milestones

#### Month 1: Project Setup & Core Infrastructure
**Week 1-2: Project Foundation**
- [ ] Initialize Cargo workspace
- [ ] Set up GitHub Actions CI/CD
- [ ] Configure Rust toolchain (MSRV 1.75+)
- [ ] Set up pre-commit hooks
- [ ] Documentation structure (rustdoc)

**Week 3-4: Core Modules**
- [ ] `llm_guard::model` - Model configuration structs
- [ ] `llm_guard::error` - Error types (thiserror)
- [ ] `llm_guard::vault` - Anonymization storage
- [ ] `llm_guard::util` - Text utilities, logging (tracing)
- [ ] `llm_guard::evaluate` - Scanner orchestration
- [ ] Unit tests for all core modules (>80% coverage)

#### Month 2: Simple Scanners
**Week 1-2:**
- [ ] `BanSubstrings` - String/word matching with redaction
- [ ] `Regex` - Custom regex pattern matching
- [ ] `InvisibleText` - Unicode zero-width detection
- [ ] Integration tests for each scanner

**Week 3-4:**
- [ ] `TokenLimit` - Token counting via tiktoken-rs
- [ ] `JSON` (output) - JSON validation
- [ ] `ReadingTime` (output) - Reading time calculation
- [ ] `URLReachability` (output) - HTTP health checks
- [ ] Snapshot tests (insta crate)

#### Month 3: API & Documentation
**Week 1-2:**
- [ ] Axum REST API endpoints
  - [ ] POST /analyze/prompt
  - [ ] POST /analyze/output
  - [ ] GET /health
  - [ ] GET /metrics (OpenTelemetry)
- [ ] OpenAPI/Swagger documentation
- [ ] Request/Response schemas

**Week 3-4:**
- [ ] Docker configuration
  - [ ] Multi-stage build
  - [ ] Alpine-based image (<500MB)
- [ ] Integration tests for API
- [ ] README and usage examples
- [ ] Benchmarks (criterion)

**Deliverable:** Working Rust library with 7 scanners + API

---

## Phase 2: ONNX Integration (Months 4-6)

### Goals
- ✅ ONNX Runtime infrastructure
- ✅ Convert 8-10 ML models to ONNX
- ✅ Implement classification scanners
- ✅ Embeddings-based scanners

### Milestones

#### Month 4: ONNX Foundation
**Week 1-2: ONNX Infrastructure**
- [ ] Integrate `ort` crate (ONNX Runtime)
- [ ] Model loading and caching
- [ ] Tokenizer integration (tokenizers crate)
- [ ] CUDA support (optional)
- [ ] Model download (hf-hub crate)

**Week 3-4: First ML Scanner**
- [ ] Convert PromptInjection model to ONNX
  ```bash
  optimum-cli export onnx \
    --model protectai/deberta-v3-base-prompt-injection-v2 \
    --task text-classification \
    ./models/prompt-injection/
  ```
- [ ] Implement `PromptInjection` scanner
- [ ] Accuracy validation vs Python
- [ ] Benchmark performance

#### Month 5: Classification Scanners
**Week 1-2:**
- [ ] `Toxicity` - Multi-label classification
- [ ] `Sentiment` - Sentiment analysis
- [ ] `Code` - Programming language detection
- [ ] Model conversions for each

**Week 3-4:**
- [ ] `BanTopics` - Zero-shot classification
- [ ] `NoRefusal` (output) - Refusal detection
- [ ] `Language` - Language detection
- [ ] Integration tests with real models

#### Month 6: Embeddings & Advanced
**Week 1-2:**
- [ ] `Relevance` (output) - Semantic similarity
  - [ ] Embeddings model (BAAI/bge-base)
  - [ ] Cosine similarity computation
- [ ] `FactualConsistency` (output) - NLI checking
- [ ] Tensor operations optimization

**Week 3-4:**
- [ ] `Bias` (output) - Bias detection
- [ ] `Gibberish` - Perplexity-based detection
- [ ] `EmotionDetection` - Emotion classification
- [ ] Comprehensive accuracy testing

**Deliverable:** 15+ scanners working (7 simple + 8-10 ML)

---

## Phase 3: Complex Scanners (Months 7-9)

### Goals
- ✅ Secret detection (95 plugins)
- ✅ PII detection and anonymization
- ✅ Remaining edge-case scanners
- ✅ Feature parity with Python

### Milestones

#### Month 7: Secret Detection
**Week 1-2: Core Secret Detection**
- [ ] Port detect-secrets core logic
- [ ] Implement entropy analysis
- [ ] Base secret detector trait
- [ ] Redaction and hashing

**Week 3-4: Plugin System**
- [ ] Port 95 secret detection plugins
  - [ ] AWS, OpenAI, Stripe, GitHub, etc.
- [ ] TOML-based configuration
- [ ] Regex compilation and caching
- [ ] Comprehensive test suite

#### Month 8: PII Detection (Phase 1)
**Week 1-2: Regex-based PII**
- [ ] Email detection and redaction
- [ ] Phone number patterns
- [ ] SSN, credit card detection
- [ ] IP address detection
- [ ] Basic placeholder replacement

**Week 3-4: NER Integration**
- [ ] Convert PII NER model to ONNX
  - [ ] ai4privacy/pii-detection-deberta-v3-base
- [ ] Implement NER-based entity detection
- [ ] Combine regex + NER results
- [ ] Vault integration for storage

#### Month 9: Advanced PII & Remaining
**Week 1-2: Anonymization**
- [ ] `Anonymize` (input) - Full PII detection
- [ ] `Deanonymize` (output) - PII restoration
- [ ] Faker integration (fake-rs)
- [ ] Context-aware entity detection

**Week 3-4: Final Scanners**
- [ ] `BanCompetitors` - Fuzzy company matching
- [ ] `BanCode` - Code snippet blocking
- [ ] `MaliciousURLs` (output) - URL reputation
- [ ] `LanguageSame` (output) - Language consistency
- [ ] `Sensitive` (output) - Sensitive info detection

**Deliverable:** Complete feature parity with Python

---

## Phase 4: Optimization (Months 10-12)

### Goals
- ✅ Performance optimization
- ✅ Memory efficiency
- ✅ Production deployment
- ✅ Documentation and examples

### Milestones

#### Month 10: Performance Tuning
**Week 1-2: Profiling**
- [ ] CPU profiling (perf, flamegraph)
- [ ] Memory profiling (valgrind, heaptrack)
- [ ] Identify hotspots
- [ ] Benchmark suite expansion

**Week 3-4: Optimization**
- [ ] Model quantization (INT8)
- [ ] Batch inference optimization
- [ ] Parallel scanner execution (rayon)
- [ ] Memory pooling and reuse
- [ ] ONNX graph optimization

#### Month 11: Production Readiness
**Week 1-2: Deployment**
- [ ] Docker optimization (<1GB image)
- [ ] Kubernetes manifests
- [ ] Helm charts
- [ ] Health checks and graceful shutdown
- [ ] Observability (OpenTelemetry)

**Week 3-4: Reliability**
- [ ] Error recovery strategies
- [ ] Rate limiting
- [ ] Circuit breakers
- [ ] Load testing (k6, locust)
- [ ] Chaos engineering tests

#### Month 12: Documentation & Launch
**Week 1-2: Documentation**
- [ ] Complete API documentation
- [ ] Usage examples
- [ ] Migration guide (Python → Rust)
- [ ] Performance comparison blog post
- [ ] Architecture decision records

**Week 3-4: Launch Preparation**
- [ ] Security audit
- [ ] Dependency audit (cargo-audit)
- [ ] License compliance check
- [ ] Release process documentation
- [ ] v1.0.0 release

**Deliverable:** Production-ready Rust implementation

---

## Success Metrics

### Phase 1 (Month 3)
- [ ] 7 scanners passing all tests
- [ ] >80% test coverage
- [ ] API functional
- [ ] CI/CD green

### Phase 2 (Month 6)
- [ ] 15+ scanners operational
- [ ] ML models <50ms latency
- [ ] Accuracy ≥99% vs Python
- [ ] Docker image <2GB

### Phase 3 (Month 9)
- [ ] All 41 scanners working
- [ ] Feature parity confirmed
- [ ] Integration tests passing
- [ ] Documentation complete

### Phase 4 (Month 12)
- [ ] 4x faster than Python
- [ ] <2GB memory usage
- [ ] <1GB Docker image
- [ ] Production deployment successful

---

## Risk Mitigation

### High Risk: ML Model Compatibility
**Mitigation:**
- Validate ONNX conversion in Month 4
- Keep PyO3 fallback option
- Test accuracy continuously

### Medium Risk: PII Detection Accuracy
**Mitigation:**
- Phased approach (regex → NER → context)
- Extensive test dataset
- Presidio PyO3 bridge as fallback

### Low Risk: Development Timeline
**Mitigation:**
- Monthly checkpoints
- Flexible scope (nice-to-haves can slip)
- Parallel workstreams where possible

---

## Resource Allocation

### Team Structure (Recommended)
- **2 Senior Rust Engineers** (full-time)
  - 1 focused on ML/ONNX
  - 1 focused on scanners/API
- **1 DevOps Engineer** (part-time)
  - CI/CD, deployment, monitoring
- **1 Tech Lead** (oversight)
  - Architecture decisions, code review

### Budget Considerations
- **Compute:** GPU instances for ONNX conversion (~$500/month)
- **Storage:** Model hosting (~100GB, ~$10/month)
- **Testing:** Cloud CI minutes (~$200/month)
- **Total:** ~$5-10k over 12 months

---

## Dependencies (External)

### Month 1-3
- Rust ecosystem (stable)
- tiktoken-rs (stable)
- Axum/Tower (stable)

### Month 4-6
- ONNX Runtime (stable)
- HuggingFace models (stable)
- hf-hub crate (stable)

### Month 7-9
- Presidio (optional fallback)
- Fake-rs (stable)
- Custom NER implementation

### Month 10-12
- Kubernetes cluster (for deployment)
- Observability stack

---

## Go/No-Go Gates

### End of Month 3 (Phase 1)
**Decision Point:** Continue to ML integration?
- ✅ If: Core scanners working, tests passing, team confident
- ⚠️ Defer: If major technical blockers, reassess scope
- ❌ Cancel: If Rust approach fundamentally flawed

### End of Month 6 (Phase 2)
**Decision Point:** Commit to full conversion?
- ✅ If: ML inference working, performance good, accuracy validated
- ⚠️ Pivot: Consider PyO3 hybrid if ONNX problematic
- ❌ Abandon: Stick with Python if no clear benefit

### End of Month 9 (Phase 3)
**Decision Point:** Production deployment?
- ✅ If: Feature parity achieved, stability proven
- ⚠️ Limited: Beta release, gradual migration
- ❌ Internal: Keep internal-only if issues persist

---

## Parallel Workstreams

### Months 1-6
- **Workstream A:** Core + Simple scanners
- **Workstream B:** ONNX infrastructure

### Months 7-9
- **Workstream A:** Secret detection
- **Workstream B:** PII detection

### Months 10-12
- **Workstream A:** Performance optimization
- **Workstream B:** Production deployment

---

## Monthly Checkpoints

### What to Review
1. **Progress:** Completed vs planned
2. **Quality:** Test coverage, bug count
3. **Performance:** Benchmark results
4. **Risks:** Blockers and mitigations
5. **Timeline:** On track? Adjust?

### Stakeholder Updates
- Monthly demo of new scanners
- Performance comparison charts
- Risk dashboard
- Next month's plan

---

## Launch Checklist

- [ ] All scanners implemented and tested
- [ ] Accuracy ≥99% parity with Python
- [ ] Performance ≥4x faster
- [ ] Memory usage ≤50% of Python
- [ ] Docker image <1GB
- [ ] Security audit complete
- [ ] Documentation published
- [ ] Migration guide written
- [ ] v1.0.0 tagged and released
- [ ] Blog post published
- [ ] Community announcement

---

**Status:** Planning Complete - Ready to Execute  
**Last Updated:** 2025-10-30  
**Next Step:** Begin Phase 1, Month 1, Week 1
