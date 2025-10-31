# Phase 9B: Complete Anonymization/Deanonymization Implementation Plan

**Project:** LLM Shield Rust/WASM
**Phase:** 9B - Complete Anonymization Implementation
**Duration:** 5-6 weeks (120-150 hours)
**Status:** 📋 Planning
**Methodology:** SPARC + London School TDD
**Target Date:** Q1 2025

---

## Executive Summary

Phase 9B completes the enterprise-grade anonymization/deanonymization system for LLM Shield, building upon the foundation established in Phase 9A. This phase focuses on ML-based entity detection with 95-99% accuracy, advanced vault backends, enterprise features, and production-ready deployment.

### Key Objectives

1. **ML-Based NER Detection** - 95-99% accuracy for person names, addresses, organizations
2. **Advanced Vault Backends** - Redis, PostgreSQL, encrypted storage
3. **Enterprise Features** - Format-preserving encryption, consistency management, masking strategies
4. **Scanner Integration** - Production-ready input/output scanners
5. **Compliance Ready** - GDPR, HIPAA, PCI-DSS compliance features
6. **Performance Optimized** - <5ms latency with caching, <100MB memory

### Success Criteria

- ✅ 100+ tests passing (40+ Phase 9A + 60+ Phase 9B)
- ✅ 95-99% detection accuracy (up from 85-95%)
- ✅ <5ms anonymization latency (including NER)
- ✅ Redis/PostgreSQL vault backends operational
- ✅ Format-preserving encryption implemented
- ✅ Scanner integration complete
- ✅ Production deployment guide
- ✅ Compliance documentation

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Phase 9B Architecture](#2-phase-9b-architecture)
3. [Component Specifications](#3-component-specifications)
4. [Implementation Roadmap](#4-implementation-roadmap)
5. [Testing Strategy](#5-testing-strategy)
6. [Performance Optimization](#6-performance-optimization)
7. [Compliance Requirements](#7-compliance-requirements)
8. [Deployment Strategy](#8-deployment-strategy)
9. [Risk Management](#9-risk-management)
10. [Success Metrics](#10-success-metrics)

---

## 1. Current State Analysis

### 1.1 Phase 9A Achievements

**Foundation Complete (Oct 2024):**
- ✅ 52 tests passing (130% of target)
- ✅ 15+ entity types with regex detection
- ✅ 85-95% accuracy for validated entities
- ✅ 30-65µs anonymization latency (150x faster than target)
- ✅ Thread-safe vault with TTL support
- ✅ Session management and audit logging
- ✅ Graceful error handling

**Current Capabilities:**

```rust
// Regex-Based Detection (Phase 9A)
Entity Types: 15+
  - EMAIL: 95% accuracy
  - PHONE_NUMBER: 90% accuracy
  - SSN: 95% accuracy
  - CREDIT_CARD: 95% (with Luhn)
  - IP_ADDRESS: 90% accuracy
  - URL: 85% accuracy
  - PERSON: 60% accuracy ⚠️ (heuristic only)
  - ADDRESS: 65% accuracy ⚠️ (heuristic only)
  - ORGANIZATION: 65% accuracy ⚠️ (heuristic only)

Performance:
  - Detection: 337µs per text
  - Anonymization: 30-65µs
  - Vault storage: <10µs per mapping
```

### 1.2 Gaps for Enterprise Grade

**Detection Limitations:**
- ❌ Person names: Only 60% accuracy (heuristic-based)
- ❌ Addresses: Only 65% accuracy (pattern matching)
- ❌ Organizations: Only 65% accuracy (pattern matching)
- ❌ Complex PII: Missing context-aware detection
- ❌ Multi-lingual: English only

**Vault Limitations:**
- ❌ Memory-only storage (no persistence)
- ❌ No encryption at rest
- ❌ No distributed sessions
- ❌ No backup/recovery

**Feature Gaps:**
- ❌ Format-preserving encryption (FPE)
- ❌ Partial masking (show last 4)
- ❌ Consistency management (same entity → same placeholder)
- ❌ Custom masking strategies
- ❌ Multi-tenant support

**Integration Gaps:**
- ❌ Scanner pipeline integration
- ❌ REST API endpoints
- ❌ WASM bindings
- ❌ Prometheus metrics

---

## 2. Phase 9B Architecture

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  Anonymization System (Phase 9B)                 │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                        Detection Layer                            │
├──────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐  │
│  │ Regex        │  │ NER          │  │ Hybrid                │  │
│  │ Detector     │→ │ Detector     │→ │ Detector              │  │
│  │ (Phase 9A)   │  │ (Phase 9B)   │  │ (Phase 9B)            │  │
│  │              │  │              │  │ - Merge results       │  │
│  │ - Patterns   │  │ - ONNX Model │  │ - Resolve conflicts   │  │
│  │ - Validators │  │ - Tokenizer  │  │ - Score entities      │  │
│  │ - 85-95% acc │  │ - 95-99% acc │  │ - 95-99% final        │  │
│  └──────────────┘  └──────────────┘  └───────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Anonymization Engine                          │
├──────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ Strategy Selection                                         │  │
│  │ - Full Replacement: [PERSON_1]                            │  │
│  │ - Partial Masking: John D*** (show 4)                     │  │
│  │ - FPE: Encrypt with format preservation                   │  │
│  │ - Synthetic: Generate realistic fake data                 │  │
│  │ - Custom: User-defined strategies                         │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                    │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ Consistency Manager                                        │  │
│  │ - Same entity → same placeholder across sessions          │  │
│  │ - Cross-document consistency                              │  │
│  │ - Cache mappings for performance                          │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                         Vault Layer                               │
├──────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Memory       │  │ Redis        │  │ PostgreSQL          │   │
│  │ Vault        │  │ Vault        │  │ Vault               │   │
│  │ (Phase 9A)   │  │ (Phase 9B)   │  │ (Phase 9B)          │   │
│  │              │  │              │  │                     │   │
│  │ - Fast       │  │ - Distributed│  │ - Persistent        │   │
│  │ - Dev/Test   │  │ - TTL native │  │ - ACID guarantees   │   │
│  │ - No persist │  │ - Cluster    │  │ - Encrypted at rest │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
│                                                                    │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ Encryption Layer (AES-256-GCM)                            │  │
│  │ - Encrypt PII before storage                              │  │
│  │ - Key management (AWS KMS, HashiCorp Vault)              │  │
│  │ - Envelope encryption                                     │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│                     Scanner Integration                           │
├──────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────┐        ┌────────────────────────────┐  │
│  │ AnonymizeScanner     │        │ DeanonymizeScanner        │  │
│  │ (Input)              │        │ (Output)                  │  │
│  │                      │        │                           │  │
│  │ User Prompt          │        │ LLM Response              │  │
│  │       ↓              │        │       ↓                   │  │
│  │ Detect PII           │        │ Find Placeholders         │  │
│  │       ↓              │        │       ↓                   │  │
│  │ Anonymize            │        │ Lookup Vault              │  │
│  │       ↓              │        │       ↓                   │  │
│  │ Store in Vault       │        │ Restore PII               │  │
│  │       ↓              │        │       ↓                   │  │
│  │ Return Safe Text     │        │ Return Personalized       │  │
│  └──────────────────────┘        └────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Dependencies

```
Phase 9B Components:
│
├── NER Detection
│   ├── ONNX Model (ai4privacy/pii-detection-deberta-v3-base)
│   ├── Tokenizer (from Phase 8 ✅)
│   ├── InferenceEngine (from Phase 8 ✅)
│   └── ResultCache (from Phase 8 ✅)
│
├── Hybrid Detection
│   ├── RegexDetector (from Phase 9A ✅)
│   ├── NerDetector (Phase 9B)
│   └── Merge Strategy (Phase 9B)
│
├── Advanced Masking
│   ├── Format-Preserving Encryption (FPE)
│   ├── Partial Masking
│   ├── Synthetic Data Generation
│   └── Custom Strategies
│
├── Vault Backends
│   ├── MemoryVault (from Phase 9A ✅)
│   ├── RedisVault (Phase 9B)
│   ├── PostgreSQLVault (Phase 9B)
│   └── EncryptedVault (Phase 9B)
│
├── Consistency Management
│   ├── EntityIndex (cross-session lookup)
│   ├── Cache Layer (fast lookup)
│   └── Conflict Resolution
│
└── Scanner Integration
    ├── AnonymizeScanner (Input)
    ├── DeanonymizeScanner (Output)
    └── Metrics/Monitoring
```

---

## 3. Component Specifications

### 3.1 NER Detector

**Purpose:** ML-based entity detection with 95-99% accuracy

**Model:** `ai4privacy/pii-detection-deberta-v3-base`
- **Architecture:** DeBERTa-v3 (Microsoft)
- **Size:** 440MB (FP32), 220MB (FP16), 110MB (INT8)
- **Task:** Token classification (NER)
- **Labels:** 43 PII entity types
- **Performance:** ~50ms inference (GPU), ~200ms (CPU)
- **License:** MIT

**Alternative Models:**
- `dslim/bert-base-NER` - General NER (86% F1)
- `flair/ner-english-large` - High accuracy (94% F1)
- `StanfordAIMI/stanford-deidentifier-base` - Medical PII (HIPAA)

**Implementation:**

```rust
// crates/llm-shield-anonymize/src/detector/ner.rs

use llm_shield_models::{InferenceEngine, TokenizerWrapper, ResultCache};
use std::sync::Arc;

/// NER-based entity detector using ONNX models
pub struct NerDetector {
    /// Inference engine from Phase 8
    engine: Arc<InferenceEngine>,

    /// Tokenizer from Phase 8
    tokenizer: Arc<TokenizerWrapper>,

    /// Result cache from Phase 8
    cache: Arc<ResultCache>,

    /// Model configuration
    config: NerConfig,

    /// BIO tag to entity type mapping
    label_map: HashMap<String, EntityType>,
}

#[derive(Debug, Clone)]
pub struct NerConfig {
    /// Model identifier in registry
    pub model_id: String,

    /// Confidence threshold (0.0-1.0)
    pub threshold: f32,

    /// Maximum sequence length (tokens)
    pub max_length: usize,

    /// Enable result caching
    pub cache_enabled: bool,

    /// Cache TTL (seconds)
    pub cache_ttl: u64,
}

impl Default for NerConfig {
    fn default() -> Self {
        Self {
            model_id: "ai4privacy/pii-detection-deberta-v3-base".to_string(),
            threshold: 0.85,
            max_length: 512,
            cache_enabled: true,
            cache_ttl: 3600,
        }
    }
}

impl NerDetector {
    /// Create new NER detector with Phase 8 components
    pub async fn new(
        engine: Arc<InferenceEngine>,
        tokenizer: Arc<TokenizerWrapper>,
        cache: Arc<ResultCache>,
        config: NerConfig,
    ) -> Result<Self> {
        // Validate model is loaded in registry
        engine.validate_model(&config.model_id).await?;

        // Load label mapping from model metadata
        let label_map = Self::load_label_mapping(&config.model_id)?;

        Ok(Self {
            engine,
            tokenizer,
            cache,
            config,
            label_map,
        })
    }

    /// Detect entities using NER model
    pub async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        // Check cache first
        if self.config.cache_enabled {
            let cache_key = format!("ner:{}", hash_text(text));
            if let Some(cached) = self.cache.get(&cache_key).await? {
                return Ok(cached);
            }
        }

        // Tokenize input
        let tokens = self.tokenizer.encode(text, self.config.max_length)?;

        // Run inference
        let logits = self.engine.infer(&self.config.model_id, &tokens).await?;

        // Decode BIO tags
        let entities = self.decode_bio_tags(text, &tokens, &logits)?;

        // Filter by confidence
        let filtered: Vec<EntityMatch> = entities
            .into_iter()
            .filter(|e| e.confidence >= self.config.threshold)
            .collect();

        // Cache results
        if self.config.cache_enabled {
            let cache_key = format!("ner:{}", hash_text(text));
            self.cache.set(&cache_key, &filtered, self.config.cache_ttl).await?;
        }

        Ok(filtered)
    }

    /// Decode BIO tags to entity matches
    fn decode_bio_tags(
        &self,
        text: &str,
        tokens: &[Token],
        logits: &[Vec<f32>],
    ) -> Result<Vec<EntityMatch>> {
        let mut entities = Vec::new();
        let mut current_entity: Option<(EntityType, usize, usize, f32)> = None;

        for (idx, (token, logit)) in tokens.iter().zip(logits.iter()).enumerate() {
            // Get predicted label
            let (label_idx, confidence) = argmax(logit);
            let label = self.label_map.get(&label_idx.to_string())
                .ok_or_else(|| Error::InvalidLabel(label_idx))?;

            match label {
                BioTag::Begin(entity_type) => {
                    // Save previous entity
                    if let Some((etype, start, end, conf)) = current_entity {
                        entities.push(EntityMatch {
                            entity_type: etype,
                            start,
                            end,
                            text: text[start..end].to_string(),
                            confidence: conf,
                        });
                    }

                    // Start new entity
                    current_entity = Some((
                        entity_type.clone(),
                        token.start,
                        token.end,
                        confidence,
                    ));
                }

                BioTag::Inside(entity_type) => {
                    // Continue current entity
                    if let Some((ref mut etype, _, ref mut end, ref mut conf)) = current_entity {
                        if etype == entity_type {
                            *end = token.end;
                            *conf = (*conf + confidence) / 2.0; // Average confidence
                        } else {
                            // Type mismatch, start new
                            entities.push(EntityMatch {
                                entity_type: etype.clone(),
                                start: token.start,
                                end: *end,
                                text: text[token.start..*end].to_string(),
                                confidence: *conf,
                            });
                            current_entity = Some((
                                entity_type.clone(),
                                token.start,
                                token.end,
                                confidence,
                            ));
                        }
                    }
                }

                BioTag::Outside => {
                    // End current entity
                    if let Some((etype, start, end, conf)) = current_entity {
                        entities.push(EntityMatch {
                            entity_type: etype,
                            start,
                            end,
                            text: text[start..end].to_string(),
                            confidence: conf,
                        });
                        current_entity = None;
                    }
                }
            }
        }

        // Save last entity
        if let Some((etype, start, end, conf)) = current_entity {
            entities.push(EntityMatch {
                entity_type: etype,
                start,
                end,
                text: text[start..end].to_string(),
                confidence: conf,
            });
        }

        Ok(entities)
    }

    /// Load label mapping from model metadata
    fn load_label_mapping(model_id: &str) -> Result<HashMap<String, EntityType>> {
        // Model labels (BIO format)
        // B-PER, I-PER, O, B-ORG, I-ORG, B-LOC, I-LOC, etc.
        let labels = vec![
            ("0", BioTag::Outside),
            ("1", BioTag::Begin(EntityType::Person)),
            ("2", BioTag::Inside(EntityType::Person)),
            ("3", BioTag::Begin(EntityType::Organization)),
            ("4", BioTag::Inside(EntityType::Organization)),
            ("5", BioTag::Begin(EntityType::Location)),
            ("6", BioTag::Inside(EntityType::Location)),
            ("7", BioTag::Begin(EntityType::Email)),
            ("8", BioTag::Inside(EntityType::Email)),
            ("9", BioTag::Begin(EntityType::PhoneNumber)),
            ("10", BioTag::Inside(EntityType::PhoneNumber)),
            // ... 43 total labels
        ];

        Ok(labels.into_iter().collect())
    }
}

/// BIO tagging scheme
#[derive(Debug, Clone, PartialEq)]
pub enum BioTag {
    Begin(EntityType),
    Inside(EntityType),
    Outside,
}

impl EntityDetector for NerDetector {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        self.detect(text).await
    }
}
```

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ner_person_detection() {
        let detector = setup_ner_detector().await;
        let text = "John Smith works at Acme Corp in New York.";

        let entities = detector.detect(text).await.unwrap();

        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].entity_type, EntityType::Person);
        assert_eq!(entities[0].text, "John Smith");
        assert!(entities[0].confidence >= 0.95);

        assert_eq!(entities[1].entity_type, EntityType::Organization);
        assert_eq!(entities[1].text, "Acme Corp");

        assert_eq!(entities[2].entity_type, EntityType::Location);
        assert_eq!(entities[2].text, "New York");
    }

    #[tokio::test]
    async fn test_ner_caching() {
        let detector = setup_ner_detector().await;
        let text = "Contact Jane Doe at jane@example.com";

        // First call (cache miss)
        let start = Instant::now();
        let entities1 = detector.detect(text).await.unwrap();
        let duration1 = start.elapsed();

        // Second call (cache hit)
        let start = Instant::now();
        let entities2 = detector.detect(text).await.unwrap();
        let duration2 = start.elapsed();

        assert_eq!(entities1, entities2);
        assert!(duration2 < duration1 / 10); // 10x faster from cache
    }

    #[tokio::test]
    async fn test_ner_confidence_threshold() {
        let mut config = NerConfig::default();
        config.threshold = 0.95; // High threshold

        let detector = setup_ner_detector_with_config(config).await;
        let text = "Maybe John works somewhere?"; // Ambiguous

        let entities = detector.detect(text).await.unwrap();

        // Should filter out low-confidence entities
        assert!(entities.is_empty() || entities[0].confidence >= 0.95);
    }
}
```

**Performance Targets:**
- Inference: <200ms (CPU), <50ms (GPU)
- With caching: <1ms (99% cache hit rate)
- Throughput: 100+ texts/second (batch processing)

---

### 3.2 Hybrid Detector

**Purpose:** Combine regex and NER for best accuracy and performance

**Strategy:**
1. Run both detectors in parallel
2. Merge results with conflict resolution
3. Choose best match based on confidence

**Implementation:**

```rust
// crates/llm-shield-anonymize/src/detector/hybrid.rs

pub struct HybridDetector {
    regex: RegexDetector,
    ner: NerDetector,
    config: HybridConfig,
}

#[derive(Debug, Clone)]
pub struct HybridConfig {
    /// Enable parallel execution
    pub parallel: bool,

    /// Confidence threshold for final results
    pub min_confidence: f32,

    /// Conflict resolution strategy
    pub resolution: ConflictResolution,

    /// Entity type preferences
    pub type_preferences: HashMap<EntityType, DetectorPreference>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictResolution {
    /// Choose highest confidence
    HighestConfidence,

    /// Prefer regex for validated types (email, phone)
    PreferValidated,

    /// Prefer NER for context-aware types (person, org)
    PreferNer,

    /// Use both (mark as duplicate)
    KeepBoth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DetectorPreference {
    Regex,
    Ner,
    Either,
}

impl HybridDetector {
    pub async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        // Run both detectors
        let (regex_entities, ner_entities) = if self.config.parallel {
            // Parallel execution
            let (regex_result, ner_result) = tokio::join!(
                self.regex.detect(text),
                self.ner.detect(text)
            );
            (regex_result?, ner_result?)
        } else {
            // Sequential execution
            let regex_entities = self.regex.detect(text).await?;
            let ner_entities = self.ner.detect(text).await?;
            (regex_entities, ner_entities)
        };

        // Merge results
        let merged = self.merge_entities(regex_entities, ner_entities)?;

        // Filter by confidence
        let filtered: Vec<EntityMatch> = merged
            .into_iter()
            .filter(|e| e.confidence >= self.config.min_confidence)
            .collect();

        Ok(filtered)
    }

    /// Merge regex and NER results with conflict resolution
    fn merge_entities(
        &self,
        regex: Vec<EntityMatch>,
        ner: Vec<EntityMatch>,
    ) -> Result<Vec<EntityMatch>> {
        let mut result = Vec::new();
        let mut used_ner = vec![false; ner.len()];

        // Process regex entities
        for regex_match in regex {
            // Find overlapping NER matches
            let overlaps: Vec<(usize, &EntityMatch)> = ner
                .iter()
                .enumerate()
                .filter(|(_, n)| self.overlaps(&regex_match, n))
                .collect();

            if overlaps.is_empty() {
                // No overlap, use regex match
                result.push(regex_match);
            } else {
                // Resolve conflict
                let winner = self.resolve_conflict(&regex_match, &overlaps)?;

                // Mark NER matches as used
                for (idx, _) in &overlaps {
                    used_ner[*idx] = true;
                }

                result.push(winner);
            }
        }

        // Add non-overlapping NER matches
        for (idx, ner_match) in ner.into_iter().enumerate() {
            if !used_ner[idx] {
                result.push(ner_match);
            }
        }

        // Sort by start position
        result.sort_by_key(|e| e.start);

        Ok(result)
    }

    /// Resolve conflict between regex and NER matches
    fn resolve_conflict(
        &self,
        regex_match: &EntityMatch,
        ner_overlaps: &[(usize, &EntityMatch)],
    ) -> Result<EntityMatch> {
        match self.config.resolution {
            ConflictResolution::HighestConfidence => {
                // Find highest confidence
                let mut best = regex_match.clone();
                for (_, ner_match) in ner_overlaps {
                    if ner_match.confidence > best.confidence {
                        best = (*ner_match).clone();
                    }
                }
                Ok(best)
            }

            ConflictResolution::PreferValidated => {
                // Prefer regex for validated types
                if self.is_validated_type(&regex_match.entity_type) {
                    Ok(regex_match.clone())
                } else {
                    // Use NER with highest confidence
                    let best_ner = ner_overlaps
                        .iter()
                        .max_by(|a, b| {
                            a.1.confidence.partial_cmp(&b.1.confidence).unwrap()
                        })
                        .map(|(_, e)| *e)
                        .unwrap();
                    Ok(best_ner.clone())
                }
            }

            ConflictResolution::PreferNer => {
                // Check type preference
                let preference = self.config.type_preferences
                    .get(&regex_match.entity_type)
                    .copied()
                    .unwrap_or(DetectorPreference::Either);

                match preference {
                    DetectorPreference::Regex => Ok(regex_match.clone()),
                    DetectorPreference::Ner => {
                        let best_ner = ner_overlaps
                            .iter()
                            .max_by(|a, b| {
                                a.1.confidence.partial_cmp(&b.1.confidence).unwrap()
                            })
                            .map(|(_, e)| *e)
                            .unwrap();
                        Ok(best_ner.clone())
                    }
                    DetectorPreference::Either => {
                        // Use highest confidence
                        let mut best = regex_match.clone();
                        for (_, ner_match) in ner_overlaps {
                            if ner_match.confidence > best.confidence {
                                best = (*ner_match).clone();
                            }
                        }
                        Ok(best)
                    }
                }
            }

            ConflictResolution::KeepBoth => {
                // Mark as potential duplicate
                // Return higher confidence one with flag
                let mut best = regex_match.clone();
                for (_, ner_match) in ner_overlaps {
                    if ner_match.confidence > best.confidence {
                        best = (*ner_match).clone();
                    }
                }
                best.metadata.insert("has_duplicate".to_string(), "true".to_string());
                Ok(best)
            }
        }
    }

    /// Check if two entities overlap
    fn overlaps(&self, a: &EntityMatch, b: &EntityMatch) -> bool {
        !(a.end <= b.start || b.end <= a.start)
    }

    /// Check if entity type uses validation (Luhn, format checks)
    fn is_validated_type(&self, entity_type: &EntityType) -> bool {
        matches!(
            entity_type,
            EntityType::CreditCard |
            EntityType::Email |
            EntityType::PhoneNumber |
            EntityType::Ssn |
            EntityType::IpAddress
        )
    }
}

impl Default for HybridConfig {
    fn default() -> Self {
        let mut type_preferences = HashMap::new();

        // Prefer regex for validated types
        type_preferences.insert(EntityType::CreditCard, DetectorPreference::Regex);
        type_preferences.insert(EntityType::Email, DetectorPreference::Regex);
        type_preferences.insert(EntityType::PhoneNumber, DetectorPreference::Regex);
        type_preferences.insert(EntityType::Ssn, DetectorPreference::Regex);
        type_preferences.insert(EntityType::IpAddress, DetectorPreference::Regex);

        // Prefer NER for context-aware types
        type_preferences.insert(EntityType::Person, DetectorPreference::Ner);
        type_preferences.insert(EntityType::Organization, DetectorPreference::Ner);
        type_preferences.insert(EntityType::Location, DetectorPreference::Ner);

        Self {
            parallel: true,
            min_confidence: 0.75,
            resolution: ConflictResolution::PreferValidated,
            type_preferences,
        }
    }
}

impl EntityDetector for HybridDetector {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        self.detect(text).await
    }
}
```

**Tests:**

```rust
#[tokio::test]
async fn test_hybrid_merge_no_conflict() {
    let detector = setup_hybrid_detector().await;
    let text = "John Smith (john@example.com) at 555-1234";

    let entities = detector.detect(text).await.unwrap();

    // Should detect all 3: person (NER), email (regex), phone (regex)
    assert_eq!(entities.len(), 3);
    assert!(entities.iter().any(|e| e.entity_type == EntityType::Person));
    assert!(entities.iter().any(|e| e.entity_type == EntityType::Email));
    assert!(entities.iter().any(|e| e.entity_type == EntityType::PhoneNumber));
}

#[tokio::test]
async fn test_hybrid_conflict_resolution() {
    let detector = setup_hybrid_detector().await;

    // Both detect "John" but differently
    let text = "John sent email john@example.com";

    let entities = detector.detect(text).await.unwrap();

    // Should choose NER for person name (higher confidence)
    let person = entities.iter().find(|e| e.text.contains("John")).unwrap();
    assert_eq!(person.entity_type, EntityType::Person);
    assert!(person.confidence >= 0.90);
}

#[tokio::test]
async fn test_hybrid_performance() {
    let detector = setup_hybrid_detector().await;
    let text = "Complex text with John Smith, jane@example.com, 555-1234, 123-45-6789";

    let start = Instant::now();
    let entities = detector.detect(text).await.unwrap();
    let duration = start.elapsed();

    assert!(!entities.is_empty());
    assert!(duration < Duration::from_millis(300)); // <300ms for hybrid
}
```

---

### 3.3 Advanced Vault Backends

**3.3.1 Redis Vault**

**Purpose:** Distributed, high-performance storage with native TTL

**Implementation:**

```rust
// crates/llm-shield-anonymize/src/vault/redis.rs

use redis::{Client, AsyncCommands, aio::ConnectionManager};
use serde::{Serialize, Deserialize};

pub struct RedisVault {
    client: ConnectionManager,
    config: RedisVaultConfig,
    key_prefix: String,
}

#[derive(Debug, Clone)]
pub struct RedisVaultConfig {
    pub url: String,
    pub pool_size: usize,
    pub timeout_ms: u64,
    pub default_ttl: u64,
    pub enable_encryption: bool,
}

impl RedisVault {
    pub async fn new(config: RedisVaultConfig) -> Result<Self> {
        let client = Client::open(config.url.as_str())?;
        let conn = client.get_tokio_connection_manager().await?;

        Ok(Self {
            client: conn,
            config,
            key_prefix: "llm_shield:anon:".to_string(),
        })
    }

    /// Generate Redis key
    fn make_key(&self, session_id: &str, placeholder: &str) -> String {
        format!("{}{}:{}", self.key_prefix, session_id, placeholder)
    }

    /// Serialize mapping with optional encryption
    fn serialize_mapping(&self, mapping: &EntityMapping) -> Result<Vec<u8>> {
        let json = serde_json::to_vec(mapping)?;

        if self.config.enable_encryption {
            // Encrypt PII data
            let encrypted = self.encrypt(&json)?;
            Ok(encrypted)
        } else {
            Ok(json)
        }
    }

    /// Deserialize mapping with optional decryption
    fn deserialize_mapping(&self, data: &[u8]) -> Result<EntityMapping> {
        let json = if self.config.enable_encryption {
            self.decrypt(data)?
        } else {
            data.to_vec()
        };

        let mapping = serde_json::from_slice(&json)?;
        Ok(mapping)
    }
}

#[async_trait::async_trait]
impl VaultStorage for RedisVault {
    async fn store_mapping(
        &self,
        session_id: &str,
        mapping: EntityMapping,
    ) -> Result<()> {
        let key = self.make_key(session_id, &mapping.placeholder);
        let data = self.serialize_mapping(&mapping)?;

        // Calculate TTL
        let ttl_secs = mapping.expires_at
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        // Store with TTL
        let mut conn = self.client.clone();
        conn.set_ex(key, data, ttl_secs as usize).await?;

        Ok(())
    }

    async fn get_mapping(
        &self,
        session_id: &str,
        placeholder: &str,
    ) -> Result<Option<EntityMapping>> {
        let key = self.make_key(session_id, placeholder);

        let mut conn = self.client.clone();
        let data: Option<Vec<u8>> = conn.get(&key).await?;

        match data {
            Some(bytes) => {
                let mapping = self.deserialize_mapping(&bytes)?;
                Ok(Some(mapping))
            }
            None => Ok(None),
        }
    }

    async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Option<AnonymizationSession>> {
        let pattern = format!("{}{}:*", self.key_prefix, session_id);

        let mut conn = self.client.clone();
        let keys: Vec<String> = conn.keys(&pattern).await?;

        if keys.is_empty() {
            return Ok(None);
        }

        // Fetch all mappings for session
        let mut mappings = Vec::new();
        for key in keys {
            if let Some(data): Option<Vec<u8>> = conn.get(&key).await? {
                let mapping = self.deserialize_mapping(&data)?;
                mappings.push(mapping);
            }
        }

        if mappings.is_empty() {
            return Ok(None);
        }

        // Construct session from mappings
        let session = AnonymizationSession {
            session_id: session_id.to_string(),
            user_id: None, // Not stored in Redis
            created_at: mappings[0].timestamp,
            expires_at: mappings[0].expires_at,
            mappings,
        };

        Ok(Some(session))
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        let pattern = format!("{}{}:*", self.key_prefix, session_id);

        let mut conn = self.client.clone();
        let keys: Vec<String> = conn.keys(&pattern).await?;

        if !keys.is_empty() {
            conn.del(keys).await?;
        }

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        // Redis handles TTL automatically
        // This is a no-op for Redis
        Ok(0)
    }
}
```

**Configuration:**

```toml
# config/anonymization.toml

[vault.redis]
url = "redis://localhost:6379"
pool_size = 10
timeout_ms = 5000
default_ttl = 3600
enable_encryption = true

# Redis Cluster (for production)
[vault.redis_cluster]
nodes = [
    "redis://node1:6379",
    "redis://node2:6379",
    "redis://node3:6379",
]
```

**Tests:**

```rust
#[tokio::test]
async fn test_redis_vault_store_get() {
    let vault = setup_redis_vault().await;

    let mapping = EntityMapping {
        session_id: "sess_test123".to_string(),
        placeholder: "[PERSON_1]".to_string(),
        entity_type: EntityType::Person,
        original_value: "John Doe".to_string(),
        confidence: 0.95,
        timestamp: SystemTime::now(),
        expires_at: SystemTime::now() + Duration::from_secs(3600),
    };

    vault.store_mapping("sess_test123", mapping.clone()).await.unwrap();

    let retrieved = vault.get_mapping("sess_test123", "[PERSON_1]")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.original_value, "John Doe");
}

#[tokio::test]
async fn test_redis_vault_ttl_expiration() {
    let vault = setup_redis_vault().await;

    let mapping = EntityMapping {
        // ... with 1 second TTL
        expires_at: SystemTime::now() + Duration::from_secs(1),
    };

    vault.store_mapping("sess_test", mapping).await.unwrap();

    // Immediately retrieve
    let retrieved1 = vault.get_mapping("sess_test", "[PERSON_1]").await.unwrap();
    assert!(retrieved1.is_some());

    // Wait for expiration
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should be expired
    let retrieved2 = vault.get_mapping("sess_test", "[PERSON_1]").await.unwrap();
    assert!(retrieved2.is_none());
}

#[tokio::test]
async fn test_redis_vault_concurrent_access() {
    let vault = Arc::new(setup_redis_vault().await);

    // Spawn 100 concurrent writes
    let tasks: Vec<_> = (0..100)
        .map(|i| {
            let vault = Arc::clone(&vault);
            tokio::spawn(async move {
                let mapping = create_test_mapping(i);
                vault.store_mapping(&format!("sess_{}", i), mapping).await
            })
        })
        .collect();

    // Wait for all
    for task in tasks {
        task.await.unwrap().unwrap();
    }

    // Verify all stored
    for i in 0..100 {
        let retrieved = vault.get_mapping(&format!("sess_{}", i), "[PERSON_1]")
            .await
            .unwrap();
        assert!(retrieved.is_some());
    }
}
```

**3.3.2 PostgreSQL Vault**

**Purpose:** Persistent, ACID-compliant storage with encryption at rest

**Schema:**

```sql
-- database/schema.sql

CREATE TABLE anonymization_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(64) UNIQUE NOT NULL,
    user_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    metadata JSONB,
    INDEX idx_session_id (session_id),
    INDEX idx_user_id (user_id),
    INDEX idx_expires_at (expires_at)
);

CREATE TABLE entity_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(64) NOT NULL,
    placeholder VARCHAR(255) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    original_value_encrypted BYTEA NOT NULL, -- AES-256-GCM encrypted
    confidence REAL NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    metadata JSONB,
    FOREIGN KEY (session_id) REFERENCES anonymization_sessions(session_id) ON DELETE CASCADE,
    UNIQUE (session_id, placeholder),
    INDEX idx_session_placeholder (session_id, placeholder),
    INDEX idx_expires_at (expires_at)
);

-- Auto-cleanup expired entries
CREATE OR REPLACE FUNCTION cleanup_expired_mappings()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM entity_mappings WHERE expires_at < NOW();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;

    DELETE FROM anonymization_sessions WHERE expires_at < NOW();

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Schedule cleanup (run every hour)
CREATE EXTENSION IF NOT EXISTS pg_cron;
SELECT cron.schedule('cleanup-expired-anonymization', '0 * * * *',
    'SELECT cleanup_expired_mappings()');
```

**Implementation:**

```rust
// crates/llm-shield-anonymize/src/vault/postgres.rs

use sqlx::{PgPool, Row};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct PostgresVault {
    pool: PgPool,
    config: PostgresVaultConfig,
    cipher: Aes256Gcm,
}

#[derive(Debug, Clone)]
pub struct PostgresVaultConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub encryption_key: Vec<u8>, // 32 bytes for AES-256
    pub enable_encryption: bool,
}

impl PostgresVault {
    pub async fn new(config: PostgresVaultConfig) -> Result<Self> {
        let pool = PgPool::connect_with(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(config.max_connections)
                .connect(&config.database_url)
                .await?
        );

        // Initialize cipher
        let key = Key::from_slice(&config.encryption_key);
        let cipher = Aes256Gcm::new(key);

        Ok(Self {
            pool,
            config,
            cipher,
        })
    }

    /// Encrypt PII data
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enable_encryption {
            return Ok(data.to_vec());
        }

        // Generate random nonce
        let nonce = Nonce::from_slice(&generate_nonce());

        // Encrypt
        let ciphertext = self.cipher
            .encrypt(nonce, data)
            .map_err(|e| Error::EncryptionFailed(e.to_string()))?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt PII data
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enable_encryption {
            return Ok(data.to_vec());
        }

        if data.len() < 12 {
            return Err(Error::InvalidCiphertext);
        }

        // Extract nonce
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        // Decrypt
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }
}

#[async_trait::async_trait]
impl VaultStorage for PostgresVault {
    async fn store_mapping(
        &self,
        session_id: &str,
        mapping: EntityMapping,
    ) -> Result<()> {
        // Ensure session exists
        sqlx::query(
            r#"
            INSERT INTO anonymization_sessions (session_id, expires_at)
            VALUES ($1, $2)
            ON CONFLICT (session_id) DO NOTHING
            "#
        )
        .bind(session_id)
        .bind(mapping.expires_at)
        .execute(&self.pool)
        .await?;

        // Encrypt original value
        let encrypted = self.encrypt(mapping.original_value.as_bytes())?;

        // Store mapping
        sqlx::query(
            r#"
            INSERT INTO entity_mappings
            (session_id, placeholder, entity_type, original_value_encrypted,
             confidence, timestamp, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (session_id, placeholder) DO UPDATE
            SET original_value_encrypted = EXCLUDED.original_value_encrypted,
                confidence = EXCLUDED.confidence
            "#
        )
        .bind(session_id)
        .bind(&mapping.placeholder)
        .bind(mapping.entity_type.to_string())
        .bind(encrypted)
        .bind(mapping.confidence)
        .bind(mapping.timestamp)
        .bind(mapping.expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_mapping(
        &self,
        session_id: &str,
        placeholder: &str,
    ) -> Result<Option<EntityMapping>> {
        let row = sqlx::query(
            r#"
            SELECT placeholder, entity_type, original_value_encrypted,
                   confidence, timestamp, expires_at
            FROM entity_mappings
            WHERE session_id = $1 AND placeholder = $2 AND expires_at > NOW()
            "#
        )
        .bind(session_id)
        .bind(placeholder)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let encrypted: Vec<u8> = row.get("original_value_encrypted");
                let decrypted = self.decrypt(&encrypted)?;
                let original_value = String::from_utf8(decrypted)?;

                let mapping = EntityMapping {
                    session_id: session_id.to_string(),
                    placeholder: row.get("placeholder"),
                    entity_type: EntityType::from_str(row.get("entity_type"))?,
                    original_value,
                    confidence: row.get("confidence"),
                    timestamp: row.get("timestamp"),
                    expires_at: row.get("expires_at"),
                };

                Ok(Some(mapping))
            }
            None => Ok(None),
        }
    }

    async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Option<AnonymizationSession>> {
        // Get session metadata
        let session_row = sqlx::query(
            r#"
            SELECT user_id, created_at, expires_at
            FROM anonymization_sessions
            WHERE session_id = $1 AND expires_at > NOW()
            "#
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        let session_row = match session_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // Get all mappings
        let mapping_rows = sqlx::query(
            r#"
            SELECT placeholder, entity_type, original_value_encrypted,
                   confidence, timestamp, expires_at
            FROM entity_mappings
            WHERE session_id = $1 AND expires_at > NOW()
            "#
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        let mut mappings = Vec::new();
        for row in mapping_rows {
            let encrypted: Vec<u8> = row.get("original_value_encrypted");
            let decrypted = self.decrypt(&encrypted)?;
            let original_value = String::from_utf8(decrypted)?;

            mappings.push(EntityMapping {
                session_id: session_id.to_string(),
                placeholder: row.get("placeholder"),
                entity_type: EntityType::from_str(row.get("entity_type"))?,
                original_value,
                confidence: row.get("confidence"),
                timestamp: row.get("timestamp"),
                expires_at: row.get("expires_at"),
            });
        }

        let session = AnonymizationSession {
            session_id: session_id.to_string(),
            user_id: session_row.get("user_id"),
            created_at: session_row.get("created_at"),
            expires_at: session_row.get("expires_at"),
            mappings,
        };

        Ok(Some(session))
    }

    async fn delete_session(&self, session_id: &str) -> Result<()> {
        // Cascade delete will remove mappings
        sqlx::query(
            "DELETE FROM anonymization_sessions WHERE session_id = $1"
        )
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let result = sqlx::query("SELECT cleanup_expired_mappings()")
            .fetch_one(&self.pool)
            .await?;

        let count: i32 = result.get(0);
        Ok(count as usize)
    }
}
```

---

### 3.4 Format-Preserving Encryption (FPE)

**Purpose:** Encrypt PII while maintaining format (useful for testing, QA)

**Use Cases:**
- Test data generation
- QA environments
- Debugging with realistic data
- Compliance (data masking)

**Implementation:**

```rust
// crates/llm-shield-anonymize/src/fpe.rs

use ff1::{FF1, FlexibleNumeralString};

/// Format-preserving encryption for PII
pub struct FormatPreservingEncryption {
    cipher: FF1<Aes256>,
    key: Vec<u8>,
}

impl FormatPreservingEncryption {
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(Error::InvalidKeyLength);
        }

        let cipher = FF1::new(key)?;

        Ok(Self {
            cipher,
            key: key.to_vec(),
        })
    }

    /// Encrypt SSN with format preservation (123-45-6789 → 987-65-4321)
    pub fn encrypt_ssn(&self, ssn: &str) -> Result<String> {
        // Extract digits only
        let digits: String = ssn.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() != 9 {
            return Err(Error::InvalidSsnFormat);
        }

        // Encrypt digits
        let encrypted = self.cipher.encrypt(&[], &FlexibleNumeralString::from(&digits))?;

        // Restore format
        let encrypted_str = encrypted.to_string();
        let formatted = format!(
            "{}-{}-{}",
            &encrypted_str[0..3],
            &encrypted_str[3..5],
            &encrypted_str[5..9]
        );

        Ok(formatted)
    }

    /// Decrypt SSN
    pub fn decrypt_ssn(&self, encrypted: &str) -> Result<String> {
        let digits: String = encrypted.chars().filter(|c| c.is_ascii_digit()).collect();
        let decrypted = self.cipher.decrypt(&[], &FlexibleNumeralString::from(&digits))?;

        let decrypted_str = decrypted.to_string();
        let formatted = format!(
            "{}-{}-{}",
            &decrypted_str[0..3],
            &decrypted_str[3..5],
            &decrypted_str[5..9]
        );

        Ok(formatted)
    }

    /// Encrypt credit card with format preservation
    pub fn encrypt_credit_card(&self, card: &str) -> Result<String> {
        let digits: String = card.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 13 || digits.len() > 19 {
            return Err(Error::InvalidCardFormat);
        }

        let encrypted = self.cipher.encrypt(&[], &FlexibleNumeralString::from(&digits))?;

        // Restore spacing every 4 digits
        let encrypted_str = encrypted.to_string();
        let formatted: String = encrypted_str
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && i % 4 == 0 {
                    vec![' ', c]
                } else {
                    vec![c]
                }
            })
            .collect();

        Ok(formatted)
    }

    /// Encrypt phone number with format preservation
    pub fn encrypt_phone(&self, phone: &str) -> Result<String> {
        // Preserve format: (123) 456-7890 → (987) 654-3210
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() != 10 {
            return Err(Error::InvalidPhoneFormat);
        }

        let encrypted = self.cipher.encrypt(&[], &FlexibleNumeralString::from(&digits))?;
        let encrypted_str = encrypted.to_string();

        let formatted = format!(
            "({}) {}-{}",
            &encrypted_str[0..3],
            &encrypted_str[3..6],
            &encrypted_str[6..10]
        );

        Ok(formatted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fpe_ssn_roundtrip() {
        let fpe = FormatPreservingEncryption::new(&[0u8; 32]).unwrap();

        let original = "123-45-6789";
        let encrypted = fpe.encrypt_ssn(original).unwrap();
        let decrypted = fpe.decrypt_ssn(&encrypted).unwrap();

        assert_ne!(encrypted, original); // Should be different
        assert_eq!(decrypted, original); // Should roundtrip
        assert_eq!(encrypted.len(), original.len()); // Same format
        assert!(encrypted.contains('-')); // Preserves dashes
    }

    #[test]
    fn test_fpe_credit_card() {
        let fpe = FormatPreservingEncryption::new(&[0u8; 32]).unwrap();

        let original = "4111 1111 1111 1111";
        let encrypted = fpe.encrypt_credit_card(original).unwrap();

        assert_ne!(encrypted, original);
        assert_eq!(encrypted.len(), original.len());
        assert_eq!(encrypted.matches(' ').count(), 3); // 3 spaces
    }
}
```

---

## 4. Implementation Roadmap

### Week 1: NER Model Integration

**Days 1-2: Model Preparation**
- Download ai4privacy/pii-detection-deberta-v3-base
- Convert to ONNX (FP16 and INT8)
- Register in ModelRegistry (Phase 8)
- Create benchmark dataset (1000 texts)
- Test inference with sample data

**Days 3-4: NER Detector Implementation**
- Implement NerDetector component
- BIO tag decoding
- Integration with InferenceEngine
- Confidence threshold tuning
- Tests (20+ tests)

**Days 5: Caching & Optimization**
- Integrate ResultCache
- Batch processing support
- Performance benchmarking
- Memory profiling

**Deliverables:**
- ✅ NerDetector fully functional
- ✅ 20+ tests passing
- ✅ <200ms inference latency
- ✅ 95%+ accuracy on test dataset

### Week 2: Hybrid Detection

**Days 1-2: Hybrid Detector**
- Implement HybridDetector
- Parallel execution (tokio::join!)
- Conflict resolution strategies
- Tests (15+ tests)

**Days 3-4: Conflict Resolution**
- Merge algorithm
- Confidence-based selection
- Type-based preferences
- Overlap detection

**Day 5: Benchmarking**
- Accuracy comparison (regex vs NER vs hybrid)
- Performance testing
- Memory usage analysis
- Documentation

**Deliverables:**
- ✅ HybridDetector operational
- ✅ 15+ tests passing
- ✅ 95-99% accuracy
- ✅ <5ms latency

### Week 3: Advanced Vault Backends

**Days 1-2: Redis Vault**
- Redis client integration
- TTL implementation
- Connection pooling
- Tests (15+ tests)

**Days 3-4: PostgreSQL Vault**
- Database schema
- SQL queries
- Connection pooling
- Migration scripts
- Tests (15+ tests)

**Day 5: Encrypted Vault**
- AES-256-GCM encryption
- Key management
- Envelope encryption
- Tests (10+ tests)

**Deliverables:**
- ✅ RedisVault functional
- ✅ PostgresVault functional
- ✅ Encryption layer complete
- ✅ 40+ tests passing

### Week 4: Format-Preserving Encryption & Masking

**Days 1-2: FPE Implementation**
- FF1 algorithm (NIST)
- SSN encryption
- Credit card encryption
- Phone number encryption
- Tests (10+ tests)

**Days 3-4: Masking Strategies**
- Partial masking (show last 4)
- Synthetic data generation
- Custom strategies
- Tests (10+ tests)

**Day 5: Consistency Management**
- EntityIndex for cross-session lookup
- Same entity → same placeholder
- Cache layer
- Tests (10+ tests)

**Deliverables:**
- ✅ FPE operational
- ✅ 4 masking strategies
- ✅ Consistency manager
- ✅ 30+ tests passing

### Week 5: Scanner Integration

**Days 1-2: AnonymizeScanner**
- Input scanner implementation
- Vault integration
- Session management
- Tests (10+ tests)

**Days 3-4: DeanonymizeScanner**
- Output scanner implementation
- Placeholder restoration
- Error handling
- Tests (10+ tests)

**Day 5: End-to-End Testing**
- Full pipeline tests
- Performance testing
- Load testing
- Documentation

**Deliverables:**
- ✅ Scanners operational
- ✅ 20+ tests passing
- ✅ Full pipeline functional
- ✅ Documentation complete

### Week 6: Polish & Production Ready

**Days 1-2: Performance Optimization**
- Profile hot paths
- Optimize memory usage
- Reduce latency
- Batch processing improvements

**Days 3-4: Documentation**
- API documentation
- Deployment guide
- Configuration guide
- Compliance documentation

**Day 5: Final Validation**
- All tests passing (100+)
- Performance targets met
- Security audit
- Code review

**Deliverables:**
- ✅ Production-ready code
- ✅ Complete documentation
- ✅ Deployment guide
- ✅ Compliance docs

---

## 5. Testing Strategy

### 5.1 Test Categories

**Unit Tests (60+)**
- NER detector (20 tests)
- Hybrid detector (15 tests)
- Redis vault (15 tests)
- PostgreSQL vault (15 tests)
- FPE (10 tests)
- Masking strategies (10 tests)
- Consistency manager (10 tests)

**Integration Tests (20+)**
- Scanner pipeline (10 tests)
- Vault backends (5 tests)
- End-to-end flows (5 tests)

**Performance Tests (10+)**
- Latency benchmarks
- Throughput tests
- Memory profiling
- Concurrent load

**Compliance Tests (10+)**
- GDPR compliance
- HIPAA compliance
- PCI-DSS compliance

### 5.2 Test Data

```rust
// Test dataset structure
pub struct TestDataset {
    pub texts: Vec<TestCase>,
    pub total: usize,
    pub categories: HashMap<String, usize>,
}

pub struct TestCase {
    pub text: String,
    pub entities: Vec<GroundTruth>,
    pub category: String,
}

pub struct GroundTruth {
    pub entity_type: EntityType,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

// Load test dataset
impl TestDataset {
    pub fn load() -> Result<Self> {
        let texts = vec![
            TestCase {
                text: "John Smith at john@example.com".to_string(),
                entities: vec![
                    GroundTruth {
                        entity_type: EntityType::Person,
                        start: 0,
                        end: 10,
                        text: "John Smith".to_string(),
                    },
                    GroundTruth {
                        entity_type: EntityType::Email,
                        start: 14,
                        end: 31,
                        text: "john@example.com".to_string(),
                    },
                ],
                category: "simple".to_string(),
            },
            // ... 1000 total test cases
        ];

        Ok(Self {
            total: texts.len(),
            categories: Self::count_categories(&texts),
            texts,
        })
    }
}
```

### 5.3 Accuracy Metrics

```rust
pub struct AccuracyMetrics {
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub true_positives: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
}

impl AccuracyMetrics {
    pub fn calculate(
        predicted: &[EntityMatch],
        ground_truth: &[GroundTruth],
    ) -> Self {
        let tp = count_true_positives(predicted, ground_truth);
        let fp = predicted.len() - tp;
        let fn_count = ground_truth.len() - tp;

        let precision = tp as f64 / (tp + fp) as f64;
        let recall = tp as f64 / (tp + fn_count) as f64;
        let f1 = 2.0 * (precision * recall) / (precision + recall);

        Self {
            precision,
            recall,
            f1_score: f1,
            true_positives: tp,
            false_positives: fp,
            false_negatives: fn_count,
        }
    }
}
```

---

## 6. Performance Optimization

### 6.1 Caching Strategy

**Three-Level Cache:**

1. **L1: Local Cache (In-Memory)**
   - Hot entities (recently used)
   - TTL: 5 minutes
   - Size: 1000 entries
   - Hit rate: 80-90%

2. **L2: Redis Cache**
   - Distributed cache
   - TTL: 1 hour
   - Size: 100K entries
   - Hit rate: 50-60%

3. **L3: PostgreSQL**
   - Persistent storage
   - TTL: 24 hours
   - Size: Unlimited
   - Hit rate: 10-20%

```rust
pub struct MultiLevelCache {
    l1: Arc<RwLock<LruCache<String, EntityMapping>>>,
    l2: Arc<RedisVault>,
    l3: Arc<PostgresVault>,
    stats: Arc<CacheStats>,
}

impl MultiLevelCache {
    pub async fn get(&self, key: &str) -> Result<Option<EntityMapping>> {
        // Check L1
        if let Some(value) = self.l1.read().await.get(key) {
            self.stats.record_hit(CacheLevel::L1).await;
            return Ok(Some(value.clone()));
        }

        // Check L2
        if let Some(value) = self.l2.get(key).await? {
            // Promote to L1
            self.l1.write().await.put(key.to_string(), value.clone());
            self.stats.record_hit(CacheLevel::L2).await;
            return Ok(Some(value));
        }

        // Check L3
        if let Some(value) = self.l3.get(key).await? {
            // Promote to L2 and L1
            self.l2.put(key, &value).await?;
            self.l1.write().await.put(key.to_string(), value.clone());
            self.stats.record_hit(CacheLevel::L3).await;
            return Ok(Some(value));
        }

        self.stats.record_miss().await;
        Ok(None)
    }
}
```

### 6.2 Batch Processing

```rust
pub struct BatchProcessor {
    detector: Arc<HybridDetector>,
    vault: Arc<dyn VaultStorage>,
    batch_size: usize,
    concurrency: usize,
}

impl BatchProcessor {
    pub async fn process_batch(&self, texts: Vec<String>) -> Result<Vec<AnonymizeResult>> {
        let chunks: Vec<_> = texts.chunks(self.batch_size).collect();

        let mut results = Vec::new();

        for chunk in chunks {
            // Process in parallel with concurrency limit
            let tasks: Vec<_> = chunk
                .iter()
                .map(|text| {
                    let detector = Arc::clone(&self.detector);
                    let vault = Arc::clone(&self.vault);

                    tokio::spawn(async move {
                        let entities = detector.detect(text).await?;
                        self.anonymize_internal(text, entities, &vault).await
                    })
                })
                .collect();

            // Wait for batch
            for task in tasks {
                results.push(task.await??);
            }
        }

        Ok(results)
    }
}
```

### 6.3 Performance Targets

| Metric | Target | Phase 9A | Phase 9B Goal |
|--------|--------|----------|---------------|
| **Detection Latency** | <5ms | 0.337ms | <5ms (with NER) |
| **Anonymization** | <10ms | 0.03-0.065ms | <2ms |
| **Deanonymization** | <5ms | <5ms | <3ms |
| **Vault Get** | <1ms | <0.01ms | <1ms |
| **Cache Hit Rate** | >80% | N/A | >85% |
| **Throughput** | 1000 req/s | 2967 req/s | 1500 req/s |
| **Memory (10K sessions)** | <500MB | <100MB | <200MB |
| **Accuracy** | >95% | 85-95% | 95-99% |

---

## 7. Compliance Requirements

### 7.1 GDPR Compliance

**Requirements:**

1. **Data Minimization** ✅
   - Only store necessary PII
   - TTL-based expiration
   - Delete on request

2. **Right to Erasure** ✅
   - `delete_session()` API
   - Cascade delete in PostgreSQL
   - Audit trail

3. **Encryption** ✅ (Phase 9B)
   - AES-256-GCM
   - Encryption at rest
   - Key management

4. **Audit Trail** ✅
   - All operations logged
   - PII redacted in logs
   - Immutable audit log

5. **Data Portability** ✅
   - Export session data
   - JSON format
   - Complete history

**Implementation:**

```rust
pub struct GdprCompliance {
    vault: Arc<dyn VaultStorage>,
    audit: Arc<AuditLogger>,
}

impl GdprCompliance {
    /// Export user data (Right to Data Portability)
    pub async fn export_user_data(&self, user_id: &str) -> Result<UserDataExport> {
        let sessions = self.vault.list_sessions_for_user(user_id).await?;

        let mut export = UserDataExport {
            user_id: user_id.to_string(),
            exported_at: SystemTime::now(),
            sessions: Vec::new(),
        };

        for session in sessions {
            export.sessions.push(SessionExport {
                session_id: session.session_id,
                created_at: session.created_at,
                entity_count: session.mappings.len(),
                // PII excluded for security
            });
        }

        Ok(export)
    }

    /// Delete user data (Right to Erasure)
    pub async fn delete_user_data(&self, user_id: &str) -> Result<DeletionReport> {
        let sessions = self.vault.list_sessions_for_user(user_id).await?;

        let mut deleted_count = 0;
        for session in &sessions {
            self.vault.delete_session(&session.session_id).await?;
            deleted_count += 1;
        }

        self.audit.log_event(AuditEvent::UserDataDeleted {
            user_id: user_id.to_string(),
            session_count: deleted_count,
            timestamp: SystemTime::now(),
        }).await?;

        Ok(DeletionReport {
            user_id: user_id.to_string(),
            sessions_deleted: deleted_count,
            deleted_at: SystemTime::now(),
        })
    }
}
```

### 7.2 HIPAA Compliance

**18 Identifiers Required:**

1. Names ✅
2. Geographic subdivisions ✅
3. Dates ✅
4. Phone numbers ✅
5. Fax numbers ✅
6. Email addresses ✅
7. SSN ✅
8. Medical record numbers ✅
9. Health plan numbers ✅
10. Account numbers ✅
11. Certificate/license numbers ✅
12. Vehicle identifiers ✅
13. Device identifiers ✅
14. URLs ✅
15. IP addresses ✅
16. Biometric identifiers ✅ (with NER)
17. Face photos ⚠️ (out of scope)
18. Other unique identifiers ✅

**Implementation:**

```rust
pub struct HipaaCompliance {
    detector: Arc<HybridDetector>,
    vault: Arc<dyn VaultStorage>,
}

impl HipaaCompliance {
    /// Verify all 18 identifiers are detected
    pub async fn verify_deidentification(&self, text: &str) -> Result<HipaaVerification> {
        let entities = self.detector.detect(text).await?;

        let mut detected_types = HashSet::new();
        for entity in &entities {
            detected_types.insert(entity.entity_type.clone());
        }

        let required = vec![
            EntityType::Person,
            EntityType::Location,
            EntityType::Date,
            EntityType::PhoneNumber,
            EntityType::Email,
            EntityType::Ssn,
            EntityType::MedicalRecord,
            // ... all 18 types
        ];

        let missing: Vec<_> = required
            .iter()
            .filter(|t| !detected_types.contains(t))
            .collect();

        Ok(HipaaVerification {
            compliant: missing.is_empty(),
            detected: entities.len(),
            missing_types: missing,
        })
    }
}
```

### 7.3 PCI-DSS Compliance

**Requirements:**

1. **Mask PAN** ✅
   - Show only last 4 digits
   - "[CARD]...1234"

2. **No Storage** ✅
   - Never store full PAN
   - Tokenization only

3. **Encryption** ✅
   - AES-256 at rest
   - TLS in transit

4. **Audit Trail** ✅
   - All access logged
   - Immutable logs

**Implementation:**

```rust
pub struct PciCompliance {
    masking: PartialMasking,
}

impl PciCompliance {
    /// Mask credit card (show last 4)
    pub fn mask_credit_card(&self, card: &str) -> String {
        let digits: String = card.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.len() < 4 {
            return "[CARD]".to_string();
        }

        let last4 = &digits[digits.len()-4..];
        format!("[CARD]...{}", last4)
    }
}
```

---

## 8. Deployment Strategy

### 8.1 Configuration Management

```toml
# config/anonymization.toml

[anonymization]
# Entity detection
detection_mode = "hybrid"  # "regex", "ner", "hybrid"
confidence_threshold = 0.85
parallel_detection = true

# Vault backend
vault_backend = "redis"  # "memory", "redis", "postgres"
session_ttl_seconds = 3600
cleanup_interval_seconds = 300

# NER model
[anonymization.ner]
model_id = "ai4privacy/pii-detection-deberta-v3-base"
max_length = 512
cache_enabled = true
cache_ttl_seconds = 3600

# Redis configuration
[anonymization.redis]
url = "redis://localhost:6379"
pool_size = 10
enable_encryption = true

# PostgreSQL configuration
[anonymization.postgres]
database_url = "postgresql://user:pass@localhost:5432/llm_shield"
max_connections = 20
enable_encryption = true
encryption_key = "${ENCRYPTION_KEY}"  # From env var

# Masking strategies
[anonymization.masking]
default_strategy = "full_replacement"  # "full_replacement", "partial", "fpe", "synthetic"
credit_card_strategy = "partial"  # Show last 4
ssn_strategy = "full_replacement"

# Performance
[anonymization.performance]
cache_levels = 3  # L1 (memory) + L2 (redis) + L3 (postgres)
l1_size = 1000
l1_ttl_seconds = 300
batch_size = 100
max_concurrency = 10
```

### 8.2 Docker Deployment

```dockerfile
# docker/Dockerfile.anonymization

FROM rust:1.75 AS builder

WORKDIR /app
COPY . .

# Build with optimizations
RUN cargo build --release --package llm-shield-anonymize

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/llm-shield-anonymize /usr/local/bin/

# Copy ONNX models
COPY models/ /app/models/

# Configuration
COPY config/anonymization.toml /app/config/

ENV CONFIG_PATH=/app/config/anonymization.toml
ENV MODEL_PATH=/app/models

EXPOSE 8080

CMD ["llm-shield-anonymize"]
```

```yaml
# docker-compose.yml

version: '3.8'

services:
  anonymization:
    build:
      context: .
      dockerfile: docker/Dockerfile.anonymization
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - REDIS_URL=redis://redis:6379
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/llm_shield
      - ENCRYPTION_KEY=${ENCRYPTION_KEY}
    depends_on:
      - redis
      - postgres
    volumes:
      - ./config:/app/config
      - ./models:/app/models

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=llm_shield
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./database/schema.sql:/docker-entrypoint-initdb.d/schema.sql

volumes:
  redis_data:
  postgres_data:
```

### 8.3 Kubernetes Deployment

```yaml
# k8s/anonymization-deployment.yaml

apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-shield-anonymization
  namespace: llm-shield
spec:
  replicas: 3
  selector:
    matchLabels:
      app: anonymization
  template:
    metadata:
      labels:
        app: anonymization
    spec:
      containers:
      - name: anonymization
        image: llm-shield/anonymization:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: url
        - name: ENCRYPTION_KEY
          valueFrom:
            secretKeyRef:
              name: encryption-secret
              key: key
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2000m"
        volumeMounts:
        - name: config
          mountPath: /app/config
        - name: models
          mountPath: /app/models
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: anonymization-config
      - name: models
        persistentVolumeClaim:
          claimName: models-pvc

---
apiVersion: v1
kind: Service
metadata:
  name: anonymization-service
  namespace: llm-shield
spec:
  selector:
    app: anonymization
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: ClusterIP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: anonymization-hpa
  namespace: llm-shield
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: llm-shield-anonymization
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## 9. Risk Management

### 9.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **NER model accuracy <95%** | Medium | High | • Use multiple models<br>• Hybrid detection<br>• Fallback to regex |
| **Performance degradation** | Low | Medium | • Caching strategy<br>• Batch processing<br>• Load testing |
| **Memory leaks** | Low | High | • Profiling<br>• Memory limits<br>• Regular monitoring |
| **Data corruption** | Low | Critical | • PostgreSQL ACID<br>• Backup/restore<br>• Data validation |
| **Encryption key loss** | Low | Critical | • Key backup<br>• HSM integration<br>• Disaster recovery |

### 9.2 Integration Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Phase 8 API changes** | Low | Medium | • Trait-based design<br>• Version pinning<br>• Integration tests |
| **WASM compatibility** | Low | Medium | • Pure Rust code<br>• No native dependencies<br>• CI testing |
| **Scanner pipeline** | Low | Medium | • Clear interfaces<br>• Mock testing<br>• Integration tests |

### 9.3 Compliance Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **GDPR non-compliance** | Low | Critical | • Legal review<br>• Audit trail<br>• Data minimization |
| **HIPAA violation** | Low | Critical | • 18 identifier coverage<br>• Encryption<br>• Access controls |
| **Data breach** | Low | Critical | • Encryption at rest<br>• TLS in transit<br>• Key management |

---

## 10. Success Metrics

### 10.1 Functional Metrics

- ✅ **Tests Passing**: 100+ (Phase 9A: 52, Phase 9B: 60+)
- ✅ **Entity Types**: 18+ (HIPAA coverage)
- ✅ **Detection Accuracy**: 95-99% (up from 85-95%)
- ✅ **Vault Backends**: 3 (Memory, Redis, PostgreSQL)
- ✅ **Masking Strategies**: 4 (Full, Partial, FPE, Synthetic)
- ✅ **Scanner Integration**: Complete
- ✅ **Compliance**: GDPR, HIPAA, PCI-DSS

### 10.2 Performance Metrics

| Metric | Phase 9A | Phase 9B Target | Must Beat |
|--------|----------|-----------------|-----------|
| **Detection** | 0.337ms | <5ms | Phase 9A ✅ |
| **Anonymization** | 0.03-0.065ms | <2ms | Phase 9A ✅ |
| **Deanonymization** | <5ms | <3ms | Phase 9A ✅ |
| **Vault Get** | <0.01ms | <1ms | Phase 9A ✅ |
| **Cache Hit Rate** | N/A | >85% | New ✅ |
| **Throughput** | 2967 req/s | 1500 req/s | Acceptable ✅ |
| **Memory** | <100MB | <200MB | Acceptable ✅ |
| **Accuracy** | 85-95% | 95-99% | Phase 9A ✅ |

### 10.3 Quality Metrics

- ✅ **Code Coverage**: >90%
- ✅ **Compilation**: Zero warnings
- ✅ **Documentation**: Complete rustdoc
- ✅ **Deployment Guide**: Complete
- ✅ **Security Audit**: Passed

### 10.4 Production Readiness

- ✅ **Docker Images**: Multi-stage, optimized
- ✅ **Kubernetes Manifests**: Complete with HPA
- ✅ **Monitoring**: Prometheus metrics
- ✅ **Logging**: Structured with tracing
- ✅ **Alerting**: Critical metrics
- ✅ **Disaster Recovery**: Backup/restore procedures

---

## Conclusion

Phase 9B represents the completion of the enterprise-grade anonymization/deanonymization system for LLM Shield. The implementation delivers:

**Core Features:**
- ML-based NER detection with 95-99% accuracy
- Hybrid detection combining regex + NER
- Three vault backends (Memory, Redis, PostgreSQL)
- Format-preserving encryption
- Multiple masking strategies
- Complete scanner integration

**Production Ready:**
- 100+ comprehensive tests
- Docker and Kubernetes deployment
- Multi-level caching
- Compliance documentation
- Performance optimization

**Timeline:**
- 5-6 weeks implementation
- 120-150 hours effort
- Q1 2025 completion

This plan provides a comprehensive roadmap for delivering a commercially viable, enterprise-grade anonymization system that meets all compliance requirements while maintaining high performance.

---

**Plan Date:** 2025-10-31
**Status:** 📋 Ready for Implementation
**Next Step:** Begin Week 1 - NER Model Integration

