# PHASE 9: ANONYMIZATION/DEANONYMIZATION - IMPLEMENTATION PLAN

**Project:** LLM Shield Rust/WASM
**Phase:** 9 - PII Anonymization & Restoration
**Date:** 2025-10-31
**Status:** Planning Complete - Ready for Implementation
**Methodology:** SPARC + London School TDD
**Estimated Duration:** 10 weeks

---

## TABLE OF CONTENTS

1. [Executive Summary](#1-executive-summary)
2. [Business Case & Compliance](#2-business-case--compliance)
3. [Requirements Analysis](#3-requirements-analysis)
4. [Architecture Design](#4-architecture-design)
5. [SPARC Specification](#5-sparc-specification)
6. [Pseudocode Design](#6-pseudocode-design)
7. [Implementation Roadmap](#7-implementation-roadmap)
8. [Testing Strategy](#8-testing-strategy)
9. [Integration Plan](#9-integration-plan)
10. [Risk Management](#10-risk-management)
11. [Success Metrics](#11-success-metrics)
12. [Appendices](#12-appendices)

---

## 1. EXECUTIVE SUMMARY

### 1.1 Project Overview

**Phase 9** implements enterprise-grade **PII anonymization** and **deanonymization** capabilities for LLM Shield, enabling compliant processing of sensitive data through Large Language Models while maintaining data privacy and regulatory compliance.

### 1.2 Key Objectives

1. **Anonymize** user prompts by detecting and replacing PII with placeholder tokens
2. **Deanonymize** LLM responses by restoring original PII values
3. **Ensure compliance** with GDPR, HIPAA, and PCI-DSS regulations
4. **Maintain performance** with <50ms anonymization latency
5. **Achieve accuracy** of 95%+ entity detection with hybrid approach

### 1.3 Value Proposition

**Problem Solved:**
- LLM providers log user prompts containing PII
- Sharing PII with third-party APIs creates compliance risks
- Organizations need to process sensitive data without exposure

**Solution:**
```
User Input: "John Doe lives at john@example.com, SSN: 123-45-6789"
     ↓ [Anonymize]
LLM Input:  "[PERSON_1] lives at [EMAIL_1], SSN: [SSN_1]"
     ↓ [LLM Processing]
LLM Output: "Hello [PERSON_1], we'll contact you at [EMAIL_1]"
     ↓ [Deanonymize]
User Output: "Hello John Doe, we'll contact you at john@example.com"
```

**Benefits:**
- ✅ Zero PII in LLM provider logs
- ✅ GDPR/HIPAA/PCI compliance
- ✅ Maintain natural conversation flow
- ✅ Audit trail for regulatory reporting
- ✅ Enterprise-ready security

### 1.4 Success Criteria

| Metric | Target | Status |
|--------|--------|--------|
| Entity Detection Accuracy | ≥95% | TBD |
| Anonymization Latency | <50ms | TBD |
| Deanonymization Accuracy | 100% | TBD |
| Test Coverage | ≥90% | TBD |
| GDPR Compliance | Full | TBD |
| HIPAA Compliance | Full | TBD |

---

## 2. BUSINESS CASE & COMPLIANCE

### 2.1 Market Drivers

**Enterprise Requirements:**
- Financial services: Credit card data, account numbers
- Healthcare: Patient names, medical record IDs, diagnoses
- Government: SSNs, passport numbers, classified info
- SaaS: Customer PII, usage data, contact info

**Regulatory Landscape:**
- **GDPR** (EU): €20M or 4% revenue penalties
- **HIPAA** (US Healthcare): $50K per violation
- **PCI-DSS** (Payment Cards): $5K-$100K monthly fines
- **CCPA** (California): $7,500 per intentional violation

### 2.2 GDPR Compliance (EU General Data Protection Regulation)

**Article 5: Principles**
- Data minimization: Collect only necessary data
- Purpose limitation: Use data only for stated purpose
- Storage limitation: Delete data when no longer needed

**Article 25: Data Protection by Design**
- Anonymization by default
- Pseudonymization where possible
- Minimize PII exposure

**Article 32: Security of Processing**
- Encryption of personal data
- Ability to restore data availability
- Regular testing of security measures

**Implementation Requirements:**
```rust
pub struct GdprConfig {
    /// Anonymize all PII by default
    pub anonymize_by_default: bool,     // true

    /// Maximum retention period (seconds)
    pub max_retention_seconds: u64,     // 3600 (1 hour)

    /// Enable right to erasure
    pub enable_erasure: bool,           // true

    /// Audit all PII processing
    pub audit_all_operations: bool,     // true

    /// Encrypt vault data
    pub encrypt_at_rest: bool,          // true
}
```

### 2.3 HIPAA Compliance (US Health Insurance Portability)

**Safe Harbor Method: Remove 18 Identifiers**

1. Names
2. Geographic subdivisions (smaller than state)
3. Dates (except year)
4. Telephone numbers
5. Fax numbers
6. Email addresses
7. Social Security numbers
8. Medical record numbers
9. Health plan beneficiary numbers
10. Account numbers
11. Certificate/license numbers
12. Vehicle identifiers
13. Device identifiers/serial numbers
14. URLs
15. IP addresses
16. Biometric identifiers
17. Full-face photographs
18. Any other unique identifying number

**Implementation Requirements:**
```rust
pub struct HipaaConfig {
    /// All 18 HIPAA identifiers
    pub identifiers: Vec<HipaaIdentifier>,

    /// Require encryption (164.312(a)(2)(iv))
    pub require_encryption: bool,       // true

    /// Audit trail (164.312(b))
    pub audit_enabled: bool,            // true

    /// Access controls (164.312(a)(1))
    pub access_control_enabled: bool,   // true
}

pub enum HipaaIdentifier {
    Name, GeographicSubdivision, Dates, Telephone,
    Fax, Email, SSN, MedicalRecordNumber,
    HealthPlanNumber, AccountNumber, CertificateNumber,
    VehicleIdentifier, DeviceIdentifier, URL, IPAddress,
    BiometricIdentifier, Photograph, UniqueIdentifier,
}
```

### 2.4 PCI-DSS Compliance (Payment Card Industry)

**Requirement 3: Protect Stored Cardholder Data**
- Mask PAN (Primary Account Number) when displayed
- Show only last 4 digits (e.g., ****-****-****-1234)
- No storage of sensitive authentication data after authorization

**Requirement 10: Track and Monitor Access**
- Log all access to cardholder data
- Record user identification
- Timestamp all events
- Include success/failure indication

**Implementation Requirements:**
```rust
pub struct PciConfig {
    /// Masking strategy for PANs
    pub mask_strategy: MaskStrategy,    // ShowLast4

    /// Prohibit PAN in logs
    pub redact_from_logs: bool,         // true

    /// Audit trail for card data
    pub audit_card_access: bool,        // true
}

pub enum MaskStrategy {
    ShowLast4,      // ****-****-****-1234
    FullRedact,     // [CREDIT_CARD_1]
    TokenizeOnly,   // External tokenization
}
```

---

## 3. REQUIREMENTS ANALYSIS

### 3.1 Functional Requirements

#### FR-1: Entity Detection
**Priority:** HIGH
**Description:** Detect sensitive PII entities in text using hybrid detection (regex + NER)

**Entity Types (15+):**
| Category | Entity Type | Example | Detection Method |
|----------|-------------|---------|------------------|
| **Person** | PERSON | "John Doe" | NER + Context |
| **Contact** | EMAIL | john@example.com | Regex |
| | PHONE_NUMBER | +1-555-123-4567 | Regex |
| | URL | https://example.com | Regex |
| | IP_ADDRESS | 192.168.1.1 | Regex + Validation |
| **Financial** | CREDIT_CARD | 4532-1488-0343-6467 | Regex + Luhn |
| | BANK_ACCOUNT | 12345678901234 | Regex + Context |
| **Government** | SSN | 123-45-6789 | Regex |
| | PASSPORT | AB1234567 | Pattern + Context |
| | DRIVER_LICENSE | DL-123456 | Pattern + Context |
| **Healthcare** | MEDICAL_RECORD | MRN-789012 | Pattern + Context |
| **Location** | ADDRESS | "123 Main St, NYC" | NER |
| | POSTAL_CODE | 12345 | Pattern |
| **Temporal** | DATE_OF_BIRTH | 01/15/1990 | Regex + Context |
| **Organization** | ORGANIZATION | "Acme Corp" | NER |
| **Custom** | CUSTOM | User-defined | User Pattern |

**Acceptance Criteria:**
- ✅ Detect all 15+ entity types
- ✅ 95%+ accuracy with NER
- ✅ Configurable entity selection
- ✅ Confidence scoring (0.0-1.0)
- ✅ Position tracking (start/end indices)

#### FR-2: Anonymization
**Priority:** HIGH
**Description:** Replace detected entities with placeholder tokens and store mappings

**Placeholder Formats:**
```rust
pub enum PlaceholderFormat {
    Numbered,    // [PERSON_1], [EMAIL_1], [CREDIT_CARD_1]
    UUID,        // [PERSON_a1b2c3d4-...]
    Hashed,      // [PERSON_abc123]
    Custom(fn), // User-provided function
}
```

**Example:**
```
Input:  "John Doe lives at john@example.com, call 555-1234"
Output: "[PERSON_1] lives at [EMAIL_1], call [PHONE_1]"
```

**Acceptance Criteria:**
- ✅ Generate unique placeholders per session
- ✅ Preserve text structure (whitespace, punctuation)
- ✅ Handle overlapping entities
- ✅ Support multiple placeholder formats
- ✅ Thread-safe token generation

#### FR-3: Vault Storage
**Priority:** HIGH
**Description:** Store entity mappings with session management and TTL

**Vault Schema:**
```rust
pub struct EntityMapping {
    session_id: String,          // "sess_abc123"
    placeholder: String,         // "[PERSON_1]"
    entity_type: EntityType,     // PERSON
    original_value: String,      // "John Doe"
    confidence: f32,             // 0.95
    timestamp: SystemTime,       // Creation time
    expires_at: SystemTime,      // TTL expiration
}

pub struct AnonymizationSession {
    session_id: String,
    user_id: Option<String>,
    created_at: SystemTime,
    expires_at: SystemTime,
    mappings: Vec<EntityMapping>,
}
```

**Acceptance Criteria:**
- ✅ Store mappings with TTL (default: 1 hour)
- ✅ Thread-safe concurrent access
- ✅ Session-scoped mappings
- ✅ Automatic cleanup of expired entries
- ✅ Support for multiple vault backends (memory, Redis, SQLite)

#### FR-4: Deanonymization
**Priority:** HIGH
**Description:** Restore original PII values from placeholders in LLM responses

**Process:**
1. Parse LLM response for placeholder tokens
2. Lookup mappings in vault by session ID
3. Replace placeholders with original values
4. Return restored text

**Graceful Degradation:**
- If mapping not found → Keep placeholder (don't crash)
- If session expired → Log warning, keep placeholders
- If vault unavailable → Return text with placeholders

**Acceptance Criteria:**
- ✅ 100% accuracy (when mapping exists)
- ✅ Preserve text structure
- ✅ Handle partial mappings (some found, some not)
- ✅ Performance <5ms for typical response

#### FR-5: Audit Logging
**Priority:** MEDIUM
**Description:** Log all PII operations for compliance and debugging

**Audit Events:**
```rust
pub enum AuditEvent {
    AnonymizeStart { session_id: String, entity_count: usize },
    AnonymizeComplete { session_id: String, duration_ms: u64 },
    DeanonymizeStart { session_id: String },
    DeanonymizeComplete { session_id: String, duration_ms: u64 },
    VaultStore { session_id: String, entity_type: EntityType },
    VaultRetrieve { session_id: String, placeholder: String },
    VaultExpire { session_id: String, mapping_count: usize },
    VaultDelete { session_id: String, reason: String },
}
```

**Acceptance Criteria:**
- ✅ Log all anonymization operations
- ✅ Include session ID, timestamp, user ID
- ✅ Record success/failure status
- ✅ Structured logging (JSON format)
- ✅ No PII in logs (redact original values)

### 3.2 Non-Functional Requirements

#### NFR-1: Performance
- **Anonymization Latency:** <50ms (p95) for typical prompt (100-500 tokens)
- **Deanonymization Latency:** <5ms (p95) for typical response
- **Entity Detection (Regex):** <1ms per entity
- **Entity Detection (NER):** <50ms per text block (CPU)
- **Vault Operations:** <0.1ms for get/set
- **Memory Usage:** <100MB for vault with 10K sessions

#### NFR-2: Accuracy
- **Entity Detection (Regex):** 85-90% accuracy
- **Entity Detection (NER):** 90-95% accuracy
- **Entity Detection (Hybrid):** 95-99% accuracy
- **Deanonymization:** 100% accuracy (when mapping exists)
- **False Positive Rate:** <5%

#### NFR-3: Scalability
- **Concurrent Sessions:** 10,000+ simultaneous
- **Vault Size:** Support 1M+ mappings
- **Throughput:** 1,000+ anonymizations/sec
- **Horizontal Scaling:** Support distributed vault (Redis)

#### NFR-4: Security
- **Encryption:** AES-256 for vault data at rest
- **Access Control:** Session-scoped access only
- **Audit Trail:** Immutable logs
- **No PII Leakage:** Zero PII in application logs
- **TTL Enforcement:** Automatic expiration

#### NFR-5: Reliability
- **Availability:** 99.9% uptime
- **Fault Tolerance:** Graceful degradation on vault failure
- **Data Durability:** No data loss with persistent vault
- **Recovery:** Automatic session recovery after restart

---

## 4. ARCHITECTURE DESIGN

### 4.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     ANONYMIZATION PIPELINE                       │
└─────────────────────────────────────────────────────────────────┘

User Input: "John Doe at john@example.com, SSN: 123-45-6789"
    │
    ↓
┌───────────────────────────────────────────────────────────────┐
│  1. ENTITY DETECTION (Hybrid)                                 │
│                                                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ Regex        │  │ NER Model    │  │ Context      │       │
│  │ Detector     │  │ (ONNX)       │  │ Rules        │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│         │                  │                  │               │
│         └──────────────────┴──────────────────┘               │
│                            │                                   │
│                    ┌───────▼────────┐                         │
│                    │ Entity Merger  │  (De-duplicate)         │
│                    └───────┬────────┘                         │
│                            │                                   │
│  Entities Found:                                              │
│  - PERSON: "John Doe" (0.95)                                  │
│  - EMAIL: "john@example.com" (0.98)                           │
│  - SSN: "123-45-6789" (0.99)                                  │
└───────────────────────────────────────────────────────────────┘
    │
    ↓
┌───────────────────────────────────────────────────────────────┐
│  2. PLACEHOLDER GENERATION                                     │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ PlaceholderGenerator::new_session()                    │  │
│  │   → session_id: "sess_abc123"                          │  │
│  │   → counters: {PERSON: 1, EMAIL: 1, SSN: 1}           │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                                │
│  Placeholders:                                                │
│  - "John Doe"          → [PERSON_1]                           │
│  - "john@example.com"  → [EMAIL_1]                            │
│  - "123-45-6789"       → [SSN_1]                              │
└───────────────────────────────────────────────────────────────┘
    │
    ↓
┌───────────────────────────────────────────────────────────────┐
│  3. TEXT REPLACEMENT                                           │
│                                                                │
│  TextReplacer::replace(text, entities, placeholders)          │
│                                                                │
│  Strategy: Reverse order (end → start) to preserve indices    │
└───────────────────────────────────────────────────────────────┘
    │
    ↓
┌───────────────────────────────────────────────────────────────┐
│  4. VAULT STORAGE                                              │
│                                                                │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ Vault::store_mapping(session_id, mapping)             │  │
│  │                                                        │  │
│  │ Key: "sess_abc123:[PERSON_1]"                         │  │
│  │ Value: EntityMapping {                                │  │
│  │   entity_type: PERSON,                                │  │
│  │   original_value: "John Doe",                         │  │
│  │   confidence: 0.95,                                   │  │
│  │   timestamp: 2025-10-31T12:00:00Z,                    │  │
│  │   expires_at: 2025-10-31T13:00:00Z (TTL: 1h)         │  │
│  │ }                                                      │  │
│  └────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────┘
    │
    ↓
Anonymized Output: "[PERSON_1] at [EMAIL_1], SSN: [SSN_1]"
Session ID: "sess_abc123"


┌─────────────────────────────────────────────────────────────────┐
│                   DEANONYMIZATION PIPELINE                       │
└─────────────────────────────────────────────────────────────────┘

LLM Response: "Hello [PERSON_1], contact at [EMAIL_1]"
Session ID: "sess_abc123"
    │
    ↓
┌───────────────────────────────────────────────────────────────┐
│  1. PLACEHOLDER DETECTION                                      │
│                                                                │
│  PlaceholderParser::find_placeholders(text)                   │
│  → Regex: \[(PERSON|EMAIL|SSN|...)_\d+\]                      │
│                                                                │
│  Found: ["[PERSON_1]", "[EMAIL_1]"]                           │
└───────────────────────────────────────────────────────────────┘
    │
    ↓
┌───────────────────────────────────────────────────────────────┐
│  2. VAULT LOOKUP                                               │
│                                                                │
│  For each placeholder:                                        │
│    key = f"{session_id}:{placeholder}"                       │
│    mapping = vault.get(key)                                   │
│                                                                │
│  Results:                                                     │
│  - [PERSON_1] → "John Doe"                                    │
│  - [EMAIL_1]  → "john@example.com"                            │
└───────────────────────────────────────────────────────────────┘
    │
    ↓
┌───────────────────────────────────────────────────────────────┐
│  3. TEXT RESTORATION                                           │
│                                                                │
│  TextReplacer::replace(text, placeholders, original_values)   │
│                                                                │
│  Strategy: Reverse order to preserve indices                  │
└───────────────────────────────────────────────────────────────┘
    │
    ↓
Restored Output: "Hello John Doe, contact at john@example.com"
```

### 4.2 Component Architecture

```
crates/llm-shield-anonymize/
│
├── Anonymizer (Main Component)
│   ├── EntityDetector (Trait)
│   │   ├── RegexDetector        (Fast, 85% accuracy)
│   │   ├── NerDetector           (Slow, 95% accuracy)
│   │   ├── HybridDetector        (Best, 97% accuracy) ← Default
│   │   └── CustomDetector        (User-defined)
│   │
│   ├── PlaceholderGenerator
│   │   ├── NumberedGenerator     ([PERSON_1])
│   │   ├── UuidGenerator         ([PERSON_uuid])
│   │   └── HashedGenerator       ([PERSON_hash])
│   │
│   ├── TextReplacer
│   │   ├── replace()             (Core algorithm)
│   │   ├── preserve_structure()  (Whitespace, punctuation)
│   │   └── handle_overlaps()     (Nested entities)
│   │
│   └── EntityMerger
│       ├── merge()               (Combine regex + NER)
│       ├── deduplicate()         (Remove duplicates)
│       └── resolve_conflicts()   (Choose best detection)
│
├── Deanonymizer (Main Component)
│   ├── PlaceholderParser
│   │   ├── find_placeholders()   (Regex search)
│   │   └── validate_format()     (Check placeholder validity)
│   │
│   └── TextRestorer
│       ├── restore()             (Replace placeholders)
│       └── handle_missing()      (Graceful degradation)
│
├── Vault (Storage Layer)
│   ├── VaultStorage (Trait)
│   │   ├── MemoryVault           (In-memory, fast) ← Default
│   │   ├── RedisVault            (Distributed, optional)
│   │   └── SqliteVault           (Persistent, optional)
│   │
│   ├── SessionManager
│   │   ├── create_session()
│   │   ├── get_session()
│   │   ├── cleanup_expired()     (TTL enforcement)
│   │   └── delete_session()      (Right to erasure)
│   │
│   └── EncryptedVault (Wrapper)
│       ├── encrypt()             (AES-256)
│       ├── decrypt()
│       └── wrap(VaultStorage)    (Decorator pattern)
│
├── Audit (Logging Layer)
│   ├── AuditLogger
│   │   ├── log_event()
│   │   ├── format_json()
│   │   └── redact_pii()          (No PII in logs)
│   │
│   └── AuditEvent (Types)
│       ├── AnonymizeEvent
│       ├── DeanonymizeEvent
│       └── VaultEvent
│
└── Config (Configuration)
    ├── AnonymizerConfig
    ├── DeanonymizerConfig
    ├── VaultConfig
    ├── ComplianceConfig          (GDPR, HIPAA, PCI)
    └── Presets
        ├── production()
        ├── gdpr_compliant()
        ├── hipaa_compliant()
        └── pci_compliant()
```

### 4.3 Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                    END-TO-END DATA FLOW                           │
└──────────────────────────────────────────────────────────────────┘

1. USER INPUT
   │
   │  "John Doe at john@example.com wants to book appointment"
   │
   ↓
2. INPUT SCANNER: Anonymizer
   │
   ├─→ [RegexDetector]  → EMAIL: john@example.com (0.98)
   │
   ├─→ [NerDetector]    → PERSON: John Doe (0.95)
   │
   └─→ [EntityMerger]   → 2 entities found
   │
   ├─→ [PlaceholderGen] → [PERSON_1], [EMAIL_1]
   │
   ├─→ [TextReplacer]   → "[PERSON_1] at [EMAIL_1] wants..."
   │
   └─→ [Vault::store]   → Save mappings (session: sess_123)
   │
   ↓
3. ANONYMIZED PROMPT
   │
   │  "[PERSON_1] at [EMAIL_1] wants to book appointment"
   │  Session: sess_123
   │
   ↓
4. LLM PROVIDER
   │
   │  (No PII stored in logs!)
   │
   ↓
5. LLM RESPONSE
   │
   │  "Hello [PERSON_1], we'll send confirmation to [EMAIL_1]"
   │
   ↓
6. OUTPUT SCANNER: Deanonymizer
   │
   ├─→ [PlaceholderParser] → Find: [PERSON_1], [EMAIL_1]
   │
   ├─→ [Vault::lookup]     → sess_123:[PERSON_1] → "John Doe"
   │                         sess_123:[EMAIL_1] → "john@example.com"
   │
   └─→ [TextRestorer]      → "Hello John Doe, we'll send..."
   │
   ↓
7. USER OUTPUT
   │
   │  "Hello John Doe, we'll send confirmation to john@example.com"
   │
   ✓ Natural language preserved
   ✓ No PII exposed to LLM provider
   ✓ Compliance maintained
```

### 4.4 Class Diagram (Rust)

```rust
// ============================================================================
// CORE TYPES
// ============================================================================

pub enum EntityType {
    Person, Email, PhoneNumber, SSN, CreditCard,
    IPAddress, URL, DateOfBirth, BankAccount,
    DriverLicense, Passport, MedicalRecord,
    Address, PostalCode, Organization, Custom(String),
}

pub struct EntityMatch {
    pub entity_type: EntityType,
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f32,
    pub metadata: HashMap<String, String>,
}

pub struct EntityMapping {
    pub session_id: String,
    pub placeholder: String,
    pub entity_type: EntityType,
    pub original_value: String,
    pub confidence: f32,
    pub timestamp: SystemTime,
    pub expires_at: SystemTime,
}

// ============================================================================
// ANONYMIZER
// ============================================================================

pub struct Anonymizer {
    config: AnonymizerConfig,
    detector: Box<dyn EntityDetector>,
    generator: Box<dyn PlaceholderGenerator>,
    vault: Arc<dyn VaultStorage>,
    audit: Arc<AuditLogger>,
}

impl Anonymizer {
    pub async fn anonymize(&self, text: &str) -> Result<AnonymizeResult> {
        // 1. Detect entities
        let entities = self.detector.detect(text).await?;

        // 2. Generate session and placeholders
        let session_id = self.generator.new_session();
        let placeholders = self.generator.generate(&entities)?;

        // 3. Replace text
        let anonymized = self.replace_entities(text, &entities, &placeholders)?;

        // 4. Store in vault
        self.vault.store_mappings(&session_id, &entities, &placeholders).await?;

        // 5. Audit log
        self.audit.log(AuditEvent::Anonymize { session_id, entity_count: entities.len() })?;

        Ok(AnonymizeResult { anonymized_text: anonymized, session_id, entities })
    }
}

pub struct AnonymizeResult {
    pub anonymized_text: String,
    pub session_id: String,
    pub entities: Vec<EntityMatch>,
}

// ============================================================================
// ENTITY DETECTOR (Strategy Pattern)
// ============================================================================

#[async_trait]
pub trait EntityDetector: Send + Sync {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>>;
}

pub struct RegexDetector {
    patterns: HashMap<EntityType, Vec<Regex>>,
    validators: HashMap<EntityType, Box<dyn Validator>>,
}

#[async_trait]
impl EntityDetector for RegexDetector {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        // Use regex patterns + validation (Luhn, etc.)
    }
}

pub struct NerDetector {
    model: Arc<InferenceEngine>,
    tokenizer: Arc<TokenizerWrapper>,
    cache: Arc<ResultCache>,
}

#[async_trait]
impl EntityDetector for NerDetector {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        // Use ONNX NER model
    }
}

pub struct HybridDetector {
    regex: RegexDetector,
    ner: NerDetector,
    merger: EntityMerger,
}

#[async_trait]
impl EntityDetector for HybridDetector {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        // Combine regex + NER, merge results
        let regex_entities = self.regex.detect(text).await?;
        let ner_entities = self.ner.detect(text).await?;
        self.merger.merge(regex_entities, ner_entities)
    }
}

// ============================================================================
// PLACEHOLDER GENERATOR (Strategy Pattern)
// ============================================================================

pub trait PlaceholderGenerator: Send + Sync {
    fn new_session(&self) -> String;
    fn generate(&self, entities: &[EntityMatch]) -> Result<Vec<String>>;
}

pub struct NumberedGenerator {
    counters: Arc<Mutex<HashMap<String, usize>>>,
}

impl PlaceholderGenerator for NumberedGenerator {
    fn new_session(&self) -> String {
        format!("sess_{}", Uuid::new_v4().to_string())
    }

    fn generate(&self, entities: &[EntityMatch]) -> Result<Vec<String>> {
        // Generate [PERSON_1], [EMAIL_1], etc.
    }
}

// ============================================================================
// DEANONYMIZER
// ============================================================================

pub struct Deanonymizer {
    vault: Arc<dyn VaultStorage>,
    parser: PlaceholderParser,
    audit: Arc<AuditLogger>,
}

impl Deanonymizer {
    pub async fn deanonymize(&self, text: &str, session_id: &str) -> Result<String> {
        // 1. Find placeholders
        let placeholders = self.parser.find_placeholders(text)?;

        // 2. Lookup in vault
        let mappings = self.vault.get_mappings(session_id, &placeholders).await?;

        // 3. Replace
        let restored = self.restore_text(text, &placeholders, &mappings)?;

        // 4. Audit log
        self.audit.log(AuditEvent::Deanonymize { session_id, placeholder_count: placeholders.len() })?;

        Ok(restored)
    }
}

// ============================================================================
// VAULT STORAGE (Strategy Pattern)
// ============================================================================

#[async_trait]
pub trait VaultStorage: Send + Sync {
    async fn store_mapping(&self, session_id: &str, mapping: EntityMapping) -> Result<()>;
    async fn get_mapping(&self, session_id: &str, placeholder: &str) -> Result<Option<EntityMapping>>;
    async fn get_session(&self, session_id: &str) -> Result<Option<AnonymizationSession>>;
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    async fn cleanup_expired(&self) -> Result<usize>;
}

pub struct MemoryVault {
    sessions: Arc<RwLock<HashMap<String, AnonymizationSession>>>,
    ttl: Duration,
}

#[async_trait]
impl VaultStorage for MemoryVault { /* ... */ }

pub struct RedisVault {
    client: redis::Client,
    ttl: Duration,
}

#[async_trait]
impl VaultStorage for RedisVault { /* ... */ }

// ============================================================================
// CONFIGURATION
// ============================================================================

pub struct AnonymizerConfig {
    pub entity_types: Vec<EntityType>,
    pub detection_strategy: DetectionStrategy,
    pub placeholder_format: PlaceholderFormat,
    pub vault_backend: VaultBackend,
    pub vault_ttl: Duration,
    pub compliance_mode: ComplianceMode,
}

pub enum DetectionStrategy {
    RegexOnly,  // Fast, 85% accuracy
    NerOnly,    // Slow, 95% accuracy
    Hybrid,     // Balanced, 97% accuracy (default)
}

pub enum PlaceholderFormat {
    Numbered,   // [PERSON_1]
    UUID,       // [PERSON_uuid]
    Hashed,     // [PERSON_hash]
}

pub enum VaultBackend {
    Memory,
    Redis(String),
    SQLite(String),
}

pub enum ComplianceMode {
    None,
    GDPR,
    HIPAA,
    PCI,
    All,
}
```

---

## 5. SPARC SPECIFICATION

### 5.1 S - Specification

**Phase 9 Goal:** Implement PII anonymization and deanonymization with 95%+ accuracy, <50ms latency, and full compliance support.

**Core Requirements:**
1. Detect 15+ entity types using hybrid approach (regex + NER)
2. Replace entities with numbered placeholders ([PERSON_1], [EMAIL_1])
3. Store mappings in vault with TTL (1 hour default)
4. Restore original values from placeholders
5. Support GDPR, HIPAA, PCI-DSS compliance modes
6. Audit all operations
7. Thread-safe concurrent access
8. Graceful degradation on vault failure

**Out of Scope:**
- Multi-language support (Phase 10)
- Fake data generation (Phase 10)
- External tokenization services (Phase 11)
- Blockchain-based audit trail (Phase 12)

### 5.2 P - Pseudocode

#### 5.2.1 Anonymizer Pseudocode

```
FUNCTION anonymize(text: String) -> Result<AnonymizeResult>
    // Input validation
    IF text.is_empty() THEN
        RETURN Error("Empty text")
    END IF

    // 1. DETECT ENTITIES (Hybrid)
    entities_regex = regex_detector.detect(text)
    entities_ner = ner_detector.detect(text)
    entities = entity_merger.merge(entities_regex, entities_ner)

    IF entities.is_empty() THEN
        RETURN Ok(AnonymizeResult {
            anonymized_text: text,
            session_id: generate_session_id(),
            entities: []
        })
    END IF

    // 2. GENERATE PLACEHOLDERS
    session_id = generate_session_id()  // "sess_abc123"
    placeholders = []
    counters = HashMap<EntityType, usize>::new()

    FOR EACH entity IN entities DO
        counter = counters.get_or_insert(entity.entity_type, 0)
        counter += 1
        placeholder = format!("[{}_{}]", entity.entity_type, counter)
        placeholders.push(placeholder)
    END FOR

    // 3. REPLACE TEXT (Reverse order to preserve indices)
    anonymized = text.clone()
    FOR i FROM entities.len() - 1 DOWN TO 0 DO
        entity = entities[i]
        placeholder = placeholders[i]
        anonymized.replace_range(entity.start..entity.end, &placeholder)
    END FOR

    // 4. STORE IN VAULT
    FOR i IN 0..entities.len() DO
        mapping = EntityMapping {
            session_id: session_id.clone(),
            placeholder: placeholders[i].clone(),
            entity_type: entities[i].entity_type,
            original_value: entities[i].text.clone(),
            confidence: entities[i].confidence,
            timestamp: now(),
            expires_at: now() + ttl,
        }
        vault.store_mapping(session_id, mapping)?
    END FOR

    // 5. AUDIT LOG
    audit.log(AuditEvent::Anonymize {
        session_id: session_id.clone(),
        entity_count: entities.len(),
        timestamp: now(),
    })

    RETURN Ok(AnonymizeResult {
        anonymized_text: anonymized,
        session_id: session_id,
        entities: entities,
    })
END FUNCTION
```

#### 5.2.2 Deanonymizer Pseudocode

```
FUNCTION deanonymize(text: String, session_id: String) -> Result<String>
    // Input validation
    IF text.is_empty() THEN
        RETURN Ok(text)
    END IF

    IF session_id.is_empty() THEN
        RETURN Error("Missing session ID")
    END IF

    // 1. FIND PLACEHOLDERS
    // Regex: \[(PERSON|EMAIL|SSN|...)_\d+\]
    placeholder_pattern = Regex::new(r"\[([A-Z_]+)_(\d+)\]")
    placeholders = []

    FOR match IN placeholder_pattern.find_iter(text) DO
        placeholders.push(Placeholder {
            text: match.as_str(),
            start: match.start(),
            end: match.end(),
        })
    END FOR

    IF placeholders.is_empty() THEN
        RETURN Ok(text)  // No placeholders found
    END IF

    // 2. LOOKUP IN VAULT
    mappings = []
    FOR placeholder IN placeholders DO
        key = format!("{}:{}", session_id, placeholder.text)
        mapping = vault.get_mapping(session_id, placeholder.text)?

        IF mapping.is_some() THEN
            mappings.push((placeholder, mapping.unwrap()))
        ELSE
            // Graceful degradation: keep placeholder if not found
            warn!("Mapping not found: {}", placeholder.text)
            mappings.push((placeholder, None))
        END IF
    END FOR

    // 3. REPLACE TEXT (Reverse order)
    restored = text.clone()
    FOR i FROM mappings.len() - 1 DOWN TO 0 DO
        (placeholder, mapping_opt) = mappings[i]

        IF mapping_opt.is_some() THEN
            mapping = mapping_opt.unwrap()
            restored.replace_range(
                placeholder.start..placeholder.end,
                &mapping.original_value
            )
        END IF
    END FOR

    // 4. AUDIT LOG
    audit.log(AuditEvent::Deanonymize {
        session_id: session_id.clone(),
        placeholder_count: placeholders.len(),
        restored_count: mappings.iter().filter(|(_, m)| m.is_some()).count(),
        timestamp: now(),
    })

    RETURN Ok(restored)
END FUNCTION
```

#### 5.2.3 Hybrid Entity Detection Pseudocode

```
FUNCTION hybrid_detect(text: String) -> Result<Vec<EntityMatch>>
    // 1. RUN REGEX DETECTION (Fast)
    regex_entities = []
    FOR pattern IN regex_patterns DO
        matches = pattern.find_iter(text)
        FOR match IN matches DO
            IF validate(match) THEN  // E.g., Luhn for credit cards
                regex_entities.push(EntityMatch {
                    entity_type: pattern.entity_type,
                    text: match.as_str(),
                    start: match.start(),
                    end: match.end(),
                    confidence: 0.90,  // High confidence for validated patterns
                    source: "regex",
                })
            END IF
        END FOR
    END FOR

    // 2. RUN NER DETECTION (Accurate)
    // Check cache first
    cache_key = hash(text)
    IF cache.contains(cache_key) THEN
        ner_entities = cache.get(cache_key)
    ELSE
        // Tokenize
        encoding = tokenizer.encode(text)?

        // Run inference
        logits = ner_model.infer(encoding.input_ids, encoding.attention_mask)?

        // Post-process (BIO tagging)
        ner_entities = []
        current_entity = None

        FOR i IN 0..logits.len() DO
            label_id = argmax(logits[i])
            label = labels[label_id]
            token_text = encoding.tokens[i]

            IF label.starts_with("B-") THEN  // Begin entity
                IF current_entity.is_some() THEN
                    ner_entities.push(current_entity.unwrap())
                END IF
                current_entity = Some(EntityMatch {
                    entity_type: label.strip_prefix("B-"),
                    text: token_text,
                    start: encoding.offsets[i].0,
                    end: encoding.offsets[i].1,
                    confidence: softmax(logits[i])[label_id],
                    source: "ner",
                })
            ELSE IF label.starts_with("I-") THEN  // Inside entity
                IF current_entity.is_some() THEN
                    current_entity.text += token_text
                    current_entity.end = encoding.offsets[i].1
                END IF
            ELSE  // Outside entity (O)
                IF current_entity.is_some() THEN
                    ner_entities.push(current_entity.unwrap())
                    current_entity = None
                END IF
            END IF
        END FOR

        IF current_entity.is_some() THEN
            ner_entities.push(current_entity.unwrap())
        END IF

        // Cache result
        cache.insert(cache_key, ner_entities.clone())
    END IF

    // 3. MERGE RESULTS
    merged = entity_merger.merge(regex_entities, ner_entities)

    // Merge strategy:
    // - If regex and NER agree (overlap): Choose NER (higher confidence)
    // - If regex unique: Keep regex
    // - If NER unique: Keep NER
    // - Deduplicate exact matches

    // 4. SORT BY POSITION
    merged.sort_by_key(|e| e.start)

    RETURN Ok(merged)
END FUNCTION
```

#### 5.2.4 Vault TTL Cleanup Pseudocode

```
FUNCTION cleanup_expired_sessions() -> Result<usize>
    // Run periodically (e.g., every 5 minutes)

    now = SystemTime::now()
    expired_count = 0

    // Get all sessions
    sessions = vault.get_all_sessions()?

    FOR session IN sessions DO
        IF session.expires_at <= now THEN
            // Session expired, delete it
            vault.delete_session(session.session_id)?

            // Audit log
            audit.log(AuditEvent::VaultExpire {
                session_id: session.session_id.clone(),
                mapping_count: session.mappings.len(),
                timestamp: now,
            })

            expired_count += 1
        END IF
    END FOR

    info!("Cleaned up {} expired sessions", expired_count)
    RETURN Ok(expired_count)
END FUNCTION

// Background task
ASYNC FUNCTION start_cleanup_task(interval: Duration)
    LOOP
        sleep(interval).await

        MATCH cleanup_expired_sessions() DO
            Ok(count) => debug!("Cleanup: {} sessions", count),
            Err(e) => error!("Cleanup failed: {}", e),
        END MATCH
    END LOOP
END FUNCTION
```

### 5.3 A - Architecture (See Section 4)

### 5.4 R - Refinement

**Refinement Opportunities:**

1. **Performance Optimization:**
   - Profile entity detection to identify bottlenecks
   - Benchmark regex vs NER for different entity types
   - Optimize text replacement algorithm
   - Cache NER results aggressively

2. **Accuracy Improvement:**
   - Fine-tune NER model on domain-specific data
   - Add context-aware rules (e.g., "Mr. John Doe" vs "john@example.com")
   - Implement entity validation (checksums, format checks)
   - Handle edge cases (unicode, special characters)

3. **Usability Enhancement:**
   - Provide configuration presets (GDPR, HIPAA, PCI)
   - Add CLI tool for testing anonymization
   - Create interactive docs with examples
   - Build dashboard for vault monitoring

4. **Feature Extension:**
   - Support custom entity recognizers
   - Add fake data generation (Faker integration)
   - Implement partial anonymization (mask last 4 digits)
   - Support multi-language detection

### 5.5 C - Completion

**Definition of Done:**

- ✅ All 15+ entity types detectable
- ✅ Anonymization latency <50ms (p95)
- ✅ Deanonymization latency <5ms (p95)
- ✅ Entity detection accuracy ≥95% (hybrid)
- ✅ 100% deanonymization accuracy (mapping exists)
- ✅ 90%+ test coverage
- ✅ All tests passing (unit + integration)
- ✅ GDPR compliance validated
- ✅ HIPAA compliance validated
- ✅ PCI compliance validated
- ✅ Documentation complete (API + guides)
- ✅ Benchmarks executed and documented
- ✅ Code review approved
- ✅ Integration with Phase 8 ML infrastructure
- ✅ Production deployment ready

---

## 6. PSEUDOCODE DESIGN

### 6.1 Module: anonymizer.rs

```rust
//! Anonymization module
//!
//! Detects PII entities and replaces them with placeholder tokens.

use crate::{
    detector::{EntityDetector, HybridDetector},
    vault::VaultStorage,
    config::AnonymizerConfig,
    types::{EntityMatch, EntityMapping, AnonymizeResult},
};

/// Main anonymizer component
pub struct Anonymizer {
    config: AnonymizerConfig,
    detector: Box<dyn EntityDetector>,
    vault: Arc<dyn VaultStorage>,
    audit: Arc<AuditLogger>,
    session_generator: SessionIdGenerator,
}

impl Anonymizer {
    /// Create new anonymizer
    pub fn new(
        config: AnonymizerConfig,
        detector: Box<dyn EntityDetector>,
        vault: Arc<dyn VaultStorage>,
    ) -> Self {
        Self {
            config,
            detector,
            vault,
            audit: Arc::new(AuditLogger::new()),
            session_generator: SessionIdGenerator::default(),
        }
    }

    /// Anonymize text
    ///
    /// # Algorithm
    /// 1. Detect entities using configured detector
    /// 2. Generate session ID and placeholders
    /// 3. Replace entities with placeholders (reverse order)
    /// 4. Store mappings in vault with TTL
    /// 5. Log audit event
    ///
    /// # Example
    /// ```
    /// let result = anonymizer.anonymize("John Doe at john@example.com").await?;
    /// assert_eq!(result.anonymized_text, "[PERSON_1] at [EMAIL_1]");
    /// ```
    pub async fn anonymize(&self, text: &str) -> Result<AnonymizeResult> {
        // Validate input
        if text.is_empty() {
            return Err(Error::invalid_input("Empty text"));
        }

        // 1. Detect entities
        let entities = self.detector.detect(text).await?;

        // Early return if no entities found
        if entities.is_empty() {
            return Ok(AnonymizeResult {
                anonymized_text: text.to_string(),
                session_id: self.session_generator.generate(),
                entities: vec![],
            });
        }

        // 2. Generate session and placeholders
        let session_id = self.session_generator.generate();
        let placeholders = self.generate_placeholders(&entities)?;

        // 3. Replace text (reverse order to preserve indices)
        let anonymized = self.replace_entities(text, &entities, &placeholders)?;

        // 4. Store in vault
        self.store_mappings(&session_id, &entities, &placeholders).await?;

        // 5. Audit log
        self.audit.log(AuditEvent::Anonymize {
            session_id: session_id.clone(),
            entity_count: entities.len(),
            timestamp: SystemTime::now(),
        })?;

        Ok(AnonymizeResult {
            anonymized_text: anonymized,
            session_id,
            entities,
        })
    }

    /// Generate placeholders for entities
    ///
    /// Uses numbered format: [TYPE_INDEX]
    /// Example: [PERSON_1], [EMAIL_1], [PERSON_2]
    fn generate_placeholders(&self, entities: &[EntityMatch]) -> Result<Vec<String>> {
        let mut counters: HashMap<EntityType, usize> = HashMap::new();
        let mut placeholders = Vec::with_capacity(entities.len());

        for entity in entities {
            let counter = counters.entry(entity.entity_type).or_insert(0);
            *counter += 1;

            let placeholder = match self.config.placeholder_format {
                PlaceholderFormat::Numbered => {
                    format!("[{}_{}]", entity.entity_type.as_str().to_uppercase(), counter)
                }
                PlaceholderFormat::UUID => {
                    format!("[{}_{%s}]", entity.entity_type.as_str().to_uppercase(), Uuid::new_v4())
                }
                PlaceholderFormat::Hashed => {
                    let hash = hash_string(&entity.text);
                    format!("[{}_{}]", entity.entity_type.as_str().to_uppercase(), hash)
                }
            };

            placeholders.push(placeholder);
        }

        Ok(placeholders)
    }

    /// Replace entities with placeholders
    ///
    /// Uses reverse order to preserve string indices.
    fn replace_entities(
        &self,
        text: &str,
        entities: &[EntityMatch],
        placeholders: &[String],
    ) -> Result<String> {
        let mut result = text.to_string();

        // Reverse order: replace from end to start
        for i in (0..entities.len()).rev() {
            let entity = &entities[i];
            let placeholder = &placeholders[i];

            // Validate indices
            if entity.end > result.len() {
                return Err(Error::invalid_entity("Entity end index out of bounds"));
            }

            // Replace
            result.replace_range(entity.start..entity.end, placeholder);
        }

        Ok(result)
    }

    /// Store mappings in vault
    async fn store_mappings(
        &self,
        session_id: &str,
        entities: &[EntityMatch],
        placeholders: &[String],
    ) -> Result<()> {
        for i in 0..entities.len() {
            let mapping = EntityMapping {
                session_id: session_id.to_string(),
                placeholder: placeholders[i].clone(),
                entity_type: entities[i].entity_type,
                original_value: entities[i].text.clone(),
                confidence: entities[i].confidence,
                timestamp: SystemTime::now(),
                expires_at: SystemTime::now() + self.config.vault_ttl,
            };

            self.vault.store_mapping(session_id, mapping).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_anonymize_email_and_person() {
        let config = AnonymizerConfig::default();
        let detector = Box::new(HybridDetector::default());
        let vault = Arc::new(MemoryVault::new());
        let anonymizer = Anonymizer::new(config, detector, vault);

        let text = "Contact John Doe at john@example.com";
        let result = anonymizer.anonymize(text).await.unwrap();

        assert_eq!(result.anonymized_text, "Contact [PERSON_1] at [EMAIL_1]");
        assert_eq!(result.entities.len(), 2);
        assert!(!result.session_id.is_empty());
    }

    #[tokio::test]
    async fn test_anonymize_no_entities() {
        let anonymizer = setup_anonymizer();

        let text = "This is a clean sentence with no PII.";
        let result = anonymizer.anonymize(text).await.unwrap();

        assert_eq!(result.anonymized_text, text);
        assert_eq!(result.entities.len(), 0);
    }

    #[tokio::test]
    async fn test_anonymize_multiple_same_type() {
        let anonymizer = setup_anonymizer();

        let text = "John Doe and Jane Smith are friends.";
        let result = anonymizer.anonymize(text).await.unwrap();

        assert_eq!(result.anonymized_text, "[PERSON_1] and [PERSON_2] are friends.");
        assert_eq!(result.entities.len(), 2);
    }
}
```

### 6.2 Module: deanonymizer.rs

```rust
//! Deanonymization module
//!
//! Restores original PII from placeholder tokens.

use crate::{
    vault::VaultStorage,
    types::{Placeholder},
};
use regex::Regex;

/// Main deanonymizer component
pub struct Deanonymizer {
    vault: Arc<dyn VaultStorage>,
    parser: PlaceholderParser,
    audit: Arc<AuditLogger>,
}

impl Deanonymizer {
    /// Create new deanonymizer
    pub fn new(vault: Arc<dyn VaultStorage>) -> Self {
        Self {
            vault,
            parser: PlaceholderParser::new(),
            audit: Arc::new(AuditLogger::new()),
        }
    }

    /// Deanonymize text
    ///
    /// # Algorithm
    /// 1. Find placeholder tokens using regex
    /// 2. Lookup mappings in vault by session ID
    /// 3. Replace placeholders with original values (reverse order)
    /// 4. Handle missing mappings gracefully
    /// 5. Log audit event
    ///
    /// # Example
    /// ```
    /// let text = "Hello [PERSON_1], email sent to [EMAIL_1]";
    /// let result = deanonymizer.deanonymize(text, "sess_123").await?;
    /// assert_eq!(result, "Hello John Doe, email sent to john@example.com");
    /// ```
    pub async fn deanonymize(&self, text: &str, session_id: &str) -> Result<String> {
        // Validate input
        if text.is_empty() {
            return Ok(text.to_string());
        }

        if session_id.is_empty() {
            return Err(Error::invalid_input("Missing session ID"));
        }

        // 1. Find placeholders
        let placeholders = self.parser.find_placeholders(text)?;

        // Early return if no placeholders
        if placeholders.is_empty() {
            return Ok(text.to_string());
        }

        // 2. Lookup mappings in vault
        let mappings = self.lookup_mappings(session_id, &placeholders).await?;

        // 3. Replace placeholders (reverse order)
        let restored = self.restore_text(text, &placeholders, &mappings)?;

        // 4. Audit log
        let restored_count = mappings.iter().filter(|m| m.is_some()).count();
        self.audit.log(AuditEvent::Deanonymize {
            session_id: session_id.to_string(),
            placeholder_count: placeholders.len(),
            restored_count,
            timestamp: SystemTime::now(),
        })?;

        // Warn if some mappings not found
        if restored_count < placeholders.len() {
            warn!(
                "Incomplete deanonymization: {} of {} mappings found",
                restored_count,
                placeholders.len()
            );
        }

        Ok(restored)
    }

    /// Lookup mappings for placeholders
    async fn lookup_mappings(
        &self,
        session_id: &str,
        placeholders: &[Placeholder],
    ) -> Result<Vec<Option<EntityMapping>>> {
        let mut mappings = Vec::with_capacity(placeholders.len());

        for placeholder in placeholders {
            match self.vault.get_mapping(session_id, &placeholder.text).await {
                Ok(Some(mapping)) => mappings.push(Some(mapping)),
                Ok(None) => {
                    warn!("Mapping not found for: {}", placeholder.text);
                    mappings.push(None);
                }
                Err(e) => {
                    error!("Vault lookup failed for {}: {}", placeholder.text, e);
                    mappings.push(None);
                }
            }
        }

        Ok(mappings)
    }

    /// Restore text by replacing placeholders
    ///
    /// Uses reverse order to preserve string indices.
    /// Gracefully handles missing mappings (keeps placeholder).
    fn restore_text(
        &self,
        text: &str,
        placeholders: &[Placeholder],
        mappings: &[Option<EntityMapping>],
    ) -> Result<String> {
        let mut result = text.to_string();

        // Reverse order
        for i in (0..placeholders.len()).rev() {
            let placeholder = &placeholders[i];

            if let Some(mapping) = &mappings[i] {
                // Replace with original value
                result.replace_range(
                    placeholder.start..placeholder.end,
                    &mapping.original_value,
                );
            }
            // If mapping not found, keep placeholder (graceful degradation)
        }

        Ok(result)
    }
}

/// Placeholder parser
struct PlaceholderParser {
    pattern: Regex,
}

impl PlaceholderParser {
    fn new() -> Self {
        // Regex: [TYPE_INDEX] where TYPE is uppercase letters/underscores, INDEX is digits
        let pattern = Regex::new(r"\[([A-Z_]+)_(\d+)\]").unwrap();
        Self { pattern }
    }

    /// Find all placeholders in text
    fn find_placeholders(&self, text: &str) -> Result<Vec<Placeholder>> {
        let mut placeholders = Vec::new();

        for cap in self.pattern.captures_iter(text) {
            let matched = cap.get(0).unwrap();
            placeholders.push(Placeholder {
                text: matched.as_str().to_string(),
                start: matched.start(),
                end: matched.end(),
            });
        }

        Ok(placeholders)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deanonymize_with_mappings() {
        let vault = setup_vault_with_mappings();
        let deanonymizer = Deanonymizer::new(Arc::new(vault));

        let text = "Hello [PERSON_1], contact at [EMAIL_1]";
        let result = deanonymizer.deanonymize(text, "sess_123").await.unwrap();

        assert_eq!(result, "Hello John Doe, contact at john@example.com");
    }

    #[tokio::test]
    async fn test_deanonymize_missing_mapping() {
        let vault = Arc::new(MemoryVault::new()); // Empty vault
        let deanonymizer = Deanonymizer::new(vault);

        let text = "Hello [PERSON_1]";
        let result = deanonymizer.deanonymize(text, "sess_123").await.unwrap();

        // Graceful degradation: keep placeholder
        assert_eq!(result, "Hello [PERSON_1]");
    }

    #[tokio::test]
    async fn test_deanonymize_no_placeholders() {
        let deanonymizer = setup_deanonymizer();

        let text = "Hello, this is a regular message";
        let result = deanonymizer.deanonymize(text, "sess_123").await.unwrap();

        assert_eq!(result, text);
    }
}
```

*(Continue with more modules: detector/regex.rs, detector/ner.rs, vault/memory.rs, etc.)*

---

## 7. IMPLEMENTATION ROADMAP

### 7.1 Timeline Overview

**Total Duration:** 10 weeks
**Methodology:** London School TDD (Tests First)
**Team Size:** 1-2 developers

```
Week 1-2:  Phase 9A - Foundation
Week 3-5:  Phase 9B - NER Integration
Week 6-7:  Phase 9C - Advanced Features
Week 8-9:  Phase 9D - Compliance & Hardening
Week 10:   Phase 9E - Production Prep
```

### 7.2 Phase 9A: Foundation (Weeks 1-2)

**Goal:** Build core anonymization infrastructure without ML

**Deliverables:**
- ✅ Enhanced Vault with TTL and session management
- ✅ Regex-based entity detection (15+ types)
- ✅ Placeholder generation (numbered format)
- ✅ Text replacement algorithm
- ✅ Basic Anonymizer (regex-only)
- ✅ Deanonymizer with graceful degradation
- ✅ 40+ unit tests

**Week 1: Vault Enhancement**

*Day 1-2: Vault TTL Support*
- [ ] Add `expires_at` field to EntityMapping
- [ ] Implement TTL checking in `get_mapping()`
- [ ] Create cleanup task for expired sessions
- [ ] Write 10 tests (concurrent access, expiration)

*Day 3-4: Session Management*
- [ ] Create `AnonymizationSession` struct
- [ ] Implement `SessionManager` component
- [ ] Add session CRUD operations
- [ ] Write 8 tests (create, get, delete, list)

*Day 5: Audit Logging*
- [ ] Define `AuditEvent` types
- [ ] Implement `AuditLogger` with structured logging
- [ ] Add PII redaction in logs
- [ ] Write 5 tests (log formats, redaction)

**Week 2: Core Anonymization**

*Day 1-2: Regex Entity Detection*
- [ ] Port patterns from Sensitive scanner
- [ ] Add validation (Luhn, IP ranges, etc.)
- [ ] Implement `RegexDetector` component
- [ ] Write 15 tests (all entity types)

*Day 3: Placeholder Generation*
- [ ] Implement `NumberedGenerator`
- [ ] Add counter management per session
- [ ] Create session ID generator
- [ ] Write 6 tests (uniqueness, formats)

*Day 4: Text Replacement*
- [ ] Implement reverse-order replacement
- [ ] Handle overlapping entities
- [ ] Preserve text structure
- [ ] Write 8 tests (edge cases, unicode)

*Day 5: Integration*
- [ ] Wire up Anonymizer component
- [ ] Wire up Deanonymizer component
- [ ] End-to-end tests (5 tests)
- [ ] Documentation (rustdoc)

**Phase 9A Success Criteria:**
- ✅ 40+ tests passing
- ✅ Vault TTL working
- ✅ Anonymization <10ms (regex only)
- ✅ Deanonymization <5ms
- ✅ Graceful degradation tested

### 7.3 Phase 9B: NER Integration (Weeks 3-5)

**Goal:** Add ML-based entity detection for 95%+ accuracy

**Deliverables:**
- ✅ NER model integrated (ai4privacy/pii-detection-deberta-v3-base)
- ✅ ONNX conversion and optimization
- ✅ Hybrid detector (regex + NER)
- ✅ Entity merger with conflict resolution
- ✅ Result caching for performance
- ✅ 30+ integration tests

**Week 3: NER Model Preparation**

*Day 1-2: Model Conversion*
- [ ] Download ai4privacy/pii-detection-deberta-v3-base
- [ ] Convert to ONNX format
- [ ] Optimize with FP16 quantization
- [ ] Test ONNX inference (Python)

*Day 3: Model Registry*
- [ ] Add NER model to `registry.json`
- [ ] Test model loading with ModelLoader
- [ ] Verify tokenizer compatibility
- [ ] Write 3 tests (loading, caching)

*Day 4-5: NER Detector Implementation*
- [ ] Implement `NerDetector` component
- [ ] Integrate with InferenceEngine
- [ ] Handle BIO tagging (Begin, Inside, Outside)
- [ ] Write 10 tests (all entity types)

**Week 4: Hybrid Detection**

*Day 1-2: Entity Merger*
- [ ] Implement merge algorithm
- [ ] Handle overlapping entities (choose best)
- [ ] Deduplicate exact matches
- [ ] Write 8 tests (conflict resolution)

*Day 3-4: Hybrid Detector*
- [ ] Implement `HybridDetector`
- [ ] Run regex + NER concurrently
- [ ] Merge results with confidence weighting
- [ ] Write 10 tests (accuracy validation)

*Day 5: Result Caching*
- [ ] Integrate ResultCache from Phase 8
- [ ] Cache NER results (expensive)
- [ ] Add cache statistics
- [ ] Write 5 tests (hit/miss rates)

**Week 5: Performance Optimization**

*Day 1-2: Benchmarking*
- [ ] Create benchmark suite
- [ ] Measure regex vs NER vs hybrid
- [ ] Profile hot paths
- [ ] Document baseline metrics

*Day 3-4: Optimization*
- [ ] Optimize text preprocessing
- [ ] Batch NER inference if possible
- [ ] Reduce allocations
- [ ] Re-run benchmarks

*Day 5: Integration Testing*
- [ ] End-to-end tests with real prompts
- [ ] Accuracy validation (test dataset)
- [ ] Performance validation (<50ms p95)
- [ ] Documentation updates

**Phase 9B Success Criteria:**
- ✅ 70+ tests passing (40 + 30 new)
- ✅ NER accuracy 90-95%
- ✅ Hybrid accuracy 95-99%
- ✅ Anonymization <50ms (p95)
- ✅ Cache hit rate >70%

### 7.4 Phase 9C: Advanced Features (Weeks 6-7)

**Goal:** Add enterprise features and multi-backend support

**Deliverables:**
- ✅ Redis vault backend (optional)
- ✅ SQLite vault backend (optional)
- ✅ Encrypted vault wrapper
- ✅ Custom entity recognizers
- ✅ Configuration presets
- ✅ 20+ additional tests

**Week 6: Vault Backends**

*Day 1-2: Redis Vault*
- [ ] Implement `RedisVault`
- [ ] Add connection pooling
- [ ] Handle TTL with Redis EXPIRE
- [ ] Write 8 tests (persistence, failover)

*Day 3-4: SQLite Vault*
- [ ] Implement `SqliteVault`
- [ ] Design schema (sessions, mappings)
- [ ] Add indexes for performance
- [ ] Write 8 tests (CRUD, migrations)

*Day 5: Encrypted Vault*
- [ ] Implement AES-256 encryption wrapper
- [ ] Add key management (KMS integration)
- [ ] Write 6 tests (encrypt/decrypt)

**Week 7: Advanced Detection**

*Day 1-2: Custom Recognizers*
- [ ] Define `CustomRecognizer` trait
- [ ] Allow user-provided patterns
- [ ] Support regex + validation functions
- [ ] Write 6 tests (custom entities)

*Day 3-4: Configuration Presets*
- [ ] Create GDPR preset
- [ ] Create HIPAA preset
- [ ] Create PCI preset
- [ ] Write 5 tests (preset validation)

*Day 5: Documentation*
- [ ] Write integration guides
- [ ] Add configuration examples
- [ ] Create troubleshooting guide
- [ ] Update API docs

**Phase 9C Success Criteria:**
- ✅ 90+ tests passing (70 + 20 new)
- ✅ Redis vault working
- ✅ SQLite vault working
- ✅ Encryption validated
- ✅ Custom recognizers functional

### 7.5 Phase 9D: Compliance & Hardening (Weeks 8-9)

**Goal:** Ensure full regulatory compliance and production readiness

**Deliverables:**
- ✅ GDPR compliance validation
- ✅ HIPAA compliance validation
- ✅ PCI compliance validation
- ✅ Compliance test suite (15+ tests)
- ✅ Security audit
- ✅ Performance tuning

**Week 8: Compliance Implementation**

*Day 1: GDPR Compliance*
- [ ] Implement `GdprConfig`
- [ ] Add right to erasure (`delete_session`)
- [ ] Enforce data minimization
- [ ] Write 5 tests (GDPR requirements)

*Day 2: HIPAA Compliance*
- [ ] Implement `HipaaConfig`
- [ ] Support all 18 identifiers
- [ ] Add access control logging
- [ ] Write 5 tests (HIPAA requirements)

*Day 3: PCI Compliance*
- [ ] Implement `PciConfig`
- [ ] Add credit card masking
- [ ] Prohibit PAN in logs
- [ ] Write 5 tests (PCI requirements)

*Day 4-5: Compliance Documentation*
- [ ] Create GDPR compliance report
- [ ] Create HIPAA compliance report
- [ ] Create PCI compliance report
- [ ] Add audit trail examples

**Week 9: Security & Performance**

*Day 1-2: Security Audit*
- [ ] Review vault encryption
- [ ] Check PII redaction in logs
- [ ] Validate access controls
- [ ] Test TTL enforcement
- [ ] Fix any vulnerabilities found

*Day 3-4: Performance Tuning*
- [ ] Profile with production-like workload
- [ ] Optimize identified bottlenecks
- [ ] Reduce memory allocations
- [ ] Re-run benchmarks

*Day 5: Load Testing*
- [ ] Create load test scripts
- [ ] Test with 10K concurrent sessions
- [ ] Measure throughput (req/sec)
- [ ] Document performance characteristics

**Phase 9D Success Criteria:**
- ✅ 105+ tests passing (90 + 15 compliance)
- ✅ GDPR compliant (validated)
- ✅ HIPAA compliant (validated)
- ✅ PCI compliant (validated)
- ✅ Security audit passed
- ✅ Performance targets met

### 7.6 Phase 9E: Production Prep (Week 10)

**Goal:** Finalize documentation, examples, and deployment

**Deliverables:**
- ✅ Complete API documentation
- ✅ Integration examples
- ✅ Deployment guides
- ✅ Migration guide (from Phase 8)
- ✅ Production monitoring setup
- ✅ Release notes

*Day 1-2: Documentation*
- [ ] Write comprehensive API docs
- [ ] Add usage examples (10+ examples)
- [ ] Create integration guide
- [ ] Write troubleshooting guide

*Day 3: Examples*
- [ ] Create CLI example tool
- [ ] Add REST API example
- [ ] Add WASM example
- [ ] Test all examples

*Day 4: Deployment*
- [ ] Write deployment guide
- [ ] Create Docker example
- [ ] Add Kubernetes manifests
- [ ] Write monitoring guide

*Day 5: Release*
- [ ] Final code review
- [ ] Update CHANGELOG
- [ ] Write release notes
- [ ] Tag release version
- [ ] Merge to main

**Phase 9E Success Criteria:**
- ✅ 105+ tests passing
- ✅ Documentation complete
- ✅ Examples working
- ✅ Deployment tested
- ✅ Release approved

---

## 8. TESTING STRATEGY

### 8.1 Test Pyramid

```
             /\
            /  \
           /    \
          / E2E  \     5 tests (2%)
         /--------\
        /          \
       / Integration\   20 tests (19%)
      /--------------\
     /                \
    /   Unit Tests     \  80 tests (76%)
   /--------------------\
  /      Foundation      \
 --------------------------

Total: ~105 tests (90%+ coverage)
```

### 8.2 Unit Tests (80 tests)

**Vault Tests (15 tests):**
```rust
mod vault_tests {
    #[tokio::test]
    async fn test_store_and_retrieve_mapping() { }

    #[tokio::test]
    async fn test_ttl_expiration() { }

    #[tokio::test]
    async fn test_concurrent_access() { }

    #[tokio::test]
    async fn test_session_cleanup() { }

    #[tokio::test]
    async fn test_missing_mapping_returns_none() { }

    // ... 10 more tests
}
```

**Entity Detection Tests (30 tests):**
```rust
mod regex_detector_tests {
    #[tokio::test]
    async fn test_detect_email() { }

    #[tokio::test]
    async fn test_detect_phone_number() { }

    #[tokio::test]
    async fn test_detect_credit_card_with_luhn() { }

    #[tokio::test]
    async fn test_detect_ssn() { }

    #[tokio::test]
    async fn test_detect_ip_address() { }

    // ... 10 more for regex
}

mod ner_detector_tests {
    #[tokio::test]
    async fn test_detect_person_name() { }

    #[tokio::test]
    async fn test_detect_organization() { }

    #[tokio::test]
    async fn test_detect_location() { }

    #[tokio::test]
    async fn test_handle_bio_tagging() { }

    // ... 8 more for NER
}

mod hybrid_detector_tests {
    #[tokio::test]
    async fn test_merge_regex_and_ner() { }

    #[tokio::test]
    async fn test_conflict_resolution() { }

    #[tokio::test]
    async fn test_deduplication() { }

    // ... 4 more for hybrid
}
```

**Anonymization Tests (20 tests):**
```rust
mod anonymizer_tests {
    #[tokio::test]
    async fn test_anonymize_single_entity() { }

    #[tokio::test]
    async fn test_anonymize_multiple_entities() { }

    #[tokio::test]
    async fn test_anonymize_no_entities() { }

    #[tokio::test]
    async fn test_placeholder_generation() { }

    #[tokio::test]
    async fn test_text_replacement_preserves_structure() { }

    #[tokio::test]
    async fn test_overlapping_entities() { }

    #[tokio::test]
    async fn test_unicode_handling() { }

    // ... 13 more tests
}
```

**Deanonymization Tests (15 tests):**
```rust
mod deanonymizer_tests {
    #[tokio::test]
    async fn test_deanonymize_with_mappings() { }

    #[tokio::test]
    async fn test_deanonymize_missing_mapping() { }

    #[tokio::test]
    async fn test_deanonymize_partial_mappings() { }

    #[tokio::test]
    async fn test_deanonymize_expired_session() { }

    #[tokio::test]
    async fn test_placeholder_parsing() { }

    // ... 10 more tests
}
```

### 8.3 Integration Tests (20 tests)

**End-to-End Tests (10 tests):**
```rust
mod integration_tests {
    #[tokio::test]
    async fn test_anonymize_deanonymize_roundtrip() {
        let vault = Arc::new(MemoryVault::new());
        let anonymizer = Anonymizer::new(config(), detector(), vault.clone());
        let deanonymizer = Deanonymizer::new(vault.clone());

        let original = "John Doe at john@example.com, SSN: 123-45-6789";

        // Anonymize
        let result = anonymizer.anonymize(original).await.unwrap();
        assert!(result.anonymized_text.contains("[PERSON_1]"));
        assert!(result.anonymized_text.contains("[EMAIL_1]"));
        assert!(result.anonymized_text.contains("[SSN_1]"));

        // Simulate LLM response
        let llm_response = format!(
            "Hello {}, we'll contact you at {}",
            "[PERSON_1]",
            "[EMAIL_1]"
        );

        // Deanonymize
        let restored = deanonymizer.deanonymize(&llm_response, &result.session_id).await.unwrap();
        assert!(restored.contains("John Doe"));
        assert!(restored.contains("john@example.com"));
    }

    #[tokio::test]
    async fn test_concurrent_anonymization() {
        // Test 100 concurrent anonymization operations
    }

    #[tokio::test]
    async fn test_session_expiration_cleanup() {
        // Test TTL expiration and cleanup
    }

    #[tokio::test]
    async fn test_vault_backend_switching() {
        // Test Memory -> Redis -> SQLite
    }

    // ... 6 more integration tests
}
```

**Scanner Integration Tests (10 tests):**
```rust
mod scanner_integration_tests {
    #[tokio::test]
    async fn test_anonymizer_as_input_scanner() {
        let anonymizer = Anonymizer::default();
        let vault = Vault::new();

        let result = anonymizer.scan("John at john@example.com", &vault).await.unwrap();

        assert!(result.is_valid);
        assert!(result.sanitized_input.contains("[PERSON_1]"));
        assert!(result.sanitized_input.contains("[EMAIL_1]"));
    }

    #[tokio::test]
    async fn test_deanonymizer_as_output_scanner() {
        // Test as output scanner
    }

    #[tokio::test]
    async fn test_full_pipeline_with_scanners() {
        // Test: Input Scanners -> Anonymize -> LLM -> Deanonymize -> Output Scanners
    }

    // ... 7 more scanner integration tests
}
```

### 8.4 Compliance Tests (15 tests)

**GDPR Tests (5 tests):**
```rust
mod gdpr_tests {
    #[tokio::test]
    async fn test_gdpr_data_minimization() {
        let config = AnonymizerConfig::gdpr_compliant();
        // Test that only necessary data is stored
    }

    #[tokio::test]
    async fn test_gdpr_right_to_erasure() {
        // Test vault.delete_session()
    }

    #[tokio::test]
    async fn test_gdpr_retention_limits() {
        // Test TTL enforcement
    }

    #[tokio::test]
    async fn test_gdpr_audit_trail() {
        // Test all operations are logged
    }

    #[tokio::test]
    async fn test_gdpr_encryption() {
        // Test vault encryption
    }
}
```

**HIPAA Tests (5 tests):**
```rust
mod hipaa_tests {
    #[tokio::test]
    async fn test_hipaa_18_identifiers() {
        // Test all 18 HIPAA identifiers are detected
    }

    #[tokio::test]
    async fn test_hipaa_audit_logging() {
        // Test access logging
    }

    #[tokio::test]
    async fn test_hipaa_encryption_required() {
        // Test encryption is enforced
    }

    #[tokio::test]
    async fn test_hipaa_access_controls() {
        // Test session-scoped access
    }

    #[tokio::test]
    async fn test_hipaa_safe_harbor_method() {
        // Validate de-identification
    }
}
```

**PCI Tests (5 tests):**
```rust
mod pci_tests {
    #[tokio::test]
    async fn test_pci_pan_masking() {
        // Test credit card masking (show last 4)
    }

    #[tokio::test]
    async fn test_pci_no_pan_in_logs() {
        // Test PAN is redacted from logs
    }

    #[tokio::test]
    async fn test_pci_audit_trail() {
        // Test card data access is logged
    }

    #[tokio::test]
    async fn test_pci_luhn_validation() {
        // Test only valid cards are detected
    }

    #[tokio::test]
    async fn test_pci_storage_prohibition() {
        // Test no sensitive auth data is stored
    }
}
```

### 8.5 Performance Tests (Benchmarks)

**Benchmark Suite:**
```rust
// benches/anonymize_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_entity_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_detection");

    let texts = vec![
        ("short", "John at john@example.com"),
        ("medium", "Contact John Doe at john@example.com or call 555-1234"),
        ("long", "Very long prompt with multiple PII..."),
    ];

    // Regex detection
    for (name, text) in &texts {
        group.bench_with_input(
            BenchmarkId::new("regex", name),
            text,
            |b, text| b.iter(|| regex_detector.detect(black_box(text)))
        );
    }

    // NER detection
    for (name, text) in &texts {
        group.bench_with_input(
            BenchmarkId::new("ner", name),
            text,
            |b, text| b.iter(|| ner_detector.detect(black_box(text)))
        );
    }

    // Hybrid detection
    for (name, text) in &texts {
        group.bench_with_input(
            BenchmarkId::new("hybrid", name),
            text,
            |b, text| b.iter(|| hybrid_detector.detect(black_box(text)))
        );
    }

    group.finish();
}

fn bench_anonymization(c: &mut Criterion) {
    let mut group = c.benchmark_group("anonymization");

    // End-to-end anonymization
    let text = "John Doe at john@example.com, SSN: 123-45-6789";
    group.bench_function("full_anonymization", |b| {
        b.iter(|| anonymizer.anonymize(black_box(text)))
    });

    group.finish();
}

fn bench_deanonymization(c: &mut Criterion) {
    let mut group = c.benchmark_group("deanonymization");

    let text = "Hello [PERSON_1] at [EMAIL_1]";
    let session_id = "sess_123";

    group.bench_function("full_deanonymization", |b| {
        b.iter(|| deanonymizer.deanonymize(black_box(text), black_box(session_id)))
    });

    group.finish();
}

criterion_group!(benches, bench_entity_detection, bench_anonymization, bench_deanonymization);
criterion_main!(benches);
```

**Performance Targets:**
| Operation | Target | Acceptable | Poor |
|-----------|--------|------------|------|
| Regex detection | <1ms | <5ms | >10ms |
| NER detection | <50ms | <100ms | >200ms |
| Hybrid detection | <50ms | <100ms | >200ms |
| Full anonymization | <50ms | <100ms | >200ms |
| Deanonymization | <5ms | <10ms | >20ms |
| Vault get/set | <0.1ms | <1ms | >5ms |

---

## 9. INTEGRATION PLAN

### 9.1 Integration with Phase 8 ML Infrastructure

**Phase 8 Components to Use:**

1. **ModelRegistry:**
```rust
// Register NER model
{
  "models": [
    {
      "task": "token-classification",
      "variant": "pii-ner",
      "model_name": "ai4privacy/pii-detection-deberta-v3-base",
      "url": "https://huggingface.co/ai4privacy/pii-detection-deberta-v3-base",
      "onnx_path": "models/pii-ner-fp16.onnx",
      "checksum": "sha256:abc123...",
      "size_mb": 180,
      "labels": [
        "O", "B-PERSON", "I-PERSON", "B-EMAIL", "I-EMAIL",
        "B-PHONE", "I-PHONE", "B-ADDRESS", "I-ADDRESS", ...
      ]
    }
  ]
}
```

2. **ModelLoader:**
```rust
// Load NER model
let registry = Arc::new(ModelRegistry::from_file("models/registry.json")?);
let loader = ModelLoader::new(registry);
let session = loader.load(ModelType::TokenClassification, ModelVariant::FP16).await?;
```

3. **TokenizerWrapper:**
```rust
// Tokenize text for NER
let tokenizer = TokenizerWrapper::from_pretrained(
    "ai4privacy/pii-detection-deberta-v3-base",
    TokenizerConfig {
        max_length: Some(512),
        truncation: Some(TruncationStrategy::LongestFirst),
        padding: Some(PaddingStrategy::MaxLength),
    }
)?;

let encoding = tokenizer.encode(text, EncodingOptions {
    add_special_tokens: true,
    return_offsets: true, // Important for entity position
})?;
```

4. **InferenceEngine:**
```rust
// Run NER inference
let engine = InferenceEngine::new(session);
let logits = engine.infer_async(
    &encoding.input_ids,
    &encoding.attention_mask,
    &ner_labels,
    PostProcessing::Softmax,
).await?;
```

5. **ResultCache:**
```rust
// Cache NER results (expensive operation)
let cache = ResultCache::new(CacheConfig {
    max_size: 1000,
    ttl: Duration::from_secs(300), // 5 minutes
});

let cache_key = ResultCache::hash_key(text);
if let Some(cached_entities) = cache.get(&cache_key) {
    return Ok(cached_entities);
}

let entities = ner_detector.detect(text).await?;
cache.insert(cache_key, entities.clone());
```

### 9.2 Integration with Existing Scanners

**Anonymizer as Input Scanner:**

```rust
// crates/llm-shield-scanners/src/input/anonymize_scanner.rs

use llm_shield_anonymize::{Anonymizer, AnonymizerConfig};
use llm_shield_core::{Scanner, ScanResult, Vault};

pub struct AnonymizeScanner {
    anonymizer: Anonymizer,
}

impl AnonymizeScanner {
    pub fn new(config: AnonymizerConfig) -> Result<Self> {
        let detector = Box::new(HybridDetector::default());
        let vault = Arc::new(MemoryVault::new());
        let anonymizer = Anonymizer::new(config, detector, vault);

        Ok(Self { anonymizer })
    }
}

#[async_trait]
impl Scanner for AnonymizeScanner {
    fn name(&self) -> &str {
        "Anonymize"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        // Anonymize the input
        let result = self.anonymizer.anonymize(input).await?;

        // Store session ID in vault for later deanonymization
        vault.set("anonymize_session_id", result.session_id.clone())?;

        // Return anonymized text as sanitized input
        Ok(ScanResult::pass(result.anonymized_text)
            .with_metadata("entities_found", result.entities.len())
            .with_metadata("session_id", result.session_id))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Input
    }
}
```

**Deanonymizer as Output Scanner:**

```rust
// crates/llm-shield-scanners/src/output/deanonymize_scanner.rs

use llm_shield_anonymize::Deanonymizer;
use llm_shield_core::{Scanner, ScanResult, Vault};

pub struct DeanonymizeScanner {
    deanonymizer: Deanonymizer,
}

#[async_trait]
impl Scanner for DeanonymizeScanner {
    fn name(&self) -> &str {
        "Deanonymize"
    }

    async fn scan(&self, input: &str, vault: &Vault) -> Result<ScanResult> {
        // Get session ID from vault
        let session_id: String = vault
            .get("anonymize_session_id")?
            .ok_or_else(|| Error::vault("No anonymize session ID found"))?;

        // Deanonymize the output
        let restored = self.deanonymizer.deanonymize(input, &session_id).await?;

        Ok(ScanResult::pass(restored)
            .with_metadata("deanonymized", "true")
            .with_metadata("session_id", session_id))
    }

    fn scanner_type(&self) -> ScannerType {
        ScannerType::Output
    }
}
```

**Usage in Pipeline:**

```rust
// Example: Full pipeline with anonymization

use llm_shield_core::scan_pipeline;
use llm_shield_scanners::input::{AnonymizeScanner, PromptInjection};
use llm_shield_scanners::output::{DeanonymizeScanner, Sensitive};

async fn process_llm_request(prompt: &str) -> Result<String> {
    let vault = Vault::new();

    // Input pipeline: Anonymize -> PromptInjection check
    let input_scanners: Vec<Box<dyn Scanner>> = vec![
        Box::new(AnonymizeScanner::new(AnonymizerConfig::default())?),
        Box::new(PromptInjection::default_config()?),
    ];

    let input_result = scan_pipeline(&input_scanners, prompt, &vault).await?;
    if !input_result.is_valid {
        return Err(Error::scan("Input validation failed"));
    }

    // LLM call with anonymized prompt
    let llm_response = call_llm_api(&input_result.sanitized_input).await?;

    // Output pipeline: Sensitive check -> Deanonymize
    let output_scanners: Vec<Box<dyn Scanner>> = vec![
        Box::new(Sensitive::default_config()?),
        Box::new(DeanonymizeScanner::new(vault.clone())?),
    ];

    let output_result = scan_pipeline(&output_scanners, &llm_response, &vault).await?;
    if !output_result.is_valid {
        return Err(Error::scan("Output validation failed"));
    }

    Ok(output_result.sanitized_input)
}
```

### 9.3 WASM Integration

**WASM Bindings:**

```rust
// crates/llm-shield-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use llm_shield_anonymize::{Anonymizer, Deanonymizer, AnonymizerConfig};

#[wasm_bindgen]
pub struct AnonymizerWasm {
    anonymizer: Anonymizer,
}

#[wasm_bindgen]
impl AnonymizerWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str) -> Result<AnonymizerWasm, JsValue> {
        let config: AnonymizerConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let detector = Box::new(HybridDetector::default());
        let vault = Arc::new(MemoryVault::new());
        let anonymizer = Anonymizer::new(config, detector, vault);

        Ok(AnonymizerWasm { anonymizer })
    }

    #[wasm_bindgen]
    pub async fn anonymize(&self, text: &str) -> Result<JsValue, JsValue> {
        let result = self.anonymizer.anonymize(text).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Serialize to JSON
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[wasm_bindgen]
pub struct DeanonymizerWasm {
    deanonymizer: Deanonymizer,
}

#[wasm_bindgen]
impl DeanonymizerWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DeanonymizerWasm {
        let vault = Arc::new(MemoryVault::new());
        let deanonymizer = Deanonymizer::new(vault);

        DeanonymizerWasm { deanonymizer }
    }

    #[wasm_bindgen]
    pub async fn deanonymize(&self, text: &str, session_id: &str) -> Result<String, JsValue> {
        self.deanonymizer.deanonymize(text, session_id).await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
```

**JavaScript Usage:**

```javascript
import { AnonymizerWasm, DeanonymizerWasm } from 'llm-shield-wasm';

// Anonymize
const config = JSON.stringify({
  entity_types: ['PERSON', 'EMAIL', 'PHONE_NUMBER'],
  detection_strategy: 'Hybrid',
  placeholder_format: 'Numbered',
});

const anonymizer = new AnonymizerWasm(config);
const result = await anonymizer.anonymize("John Doe at john@example.com");

console.log(result.anonymized_text);  // "[PERSON_1] at [EMAIL_1]"
console.log(result.session_id);        // "sess_abc123"

// Send to LLM...

// Deanonymize
const deanonymizer = new DeanonymizerWasm();
const restored = await deanonymizer.deanonymize(
  llmResponse,
  result.session_id
);

console.log(restored);  // "Hello John Doe at john@example.com"
```

---

## 10. RISK MANAGEMENT

### 10.1 Technical Risks

| Risk | Probability | Impact | Severity | Mitigation |
|------|-------------|--------|----------|------------|
| **NER Model Accuracy Below 90%** | Medium | High | 🟡 MEDIUM | - Use validated model (ai4privacy)<br>- Hybrid approach (regex fallback)<br>- Extensive testing with real data<br>- Fine-tuning option |
| **ONNX Conversion Issues** | Low | High | 🟢 LOW | - Use optimum library<br>- Test conversion early<br>- Fallback to Python inference (temporary) |
| **Performance Degradation** | Medium | Medium | 🟡 MEDIUM | - Aggressive caching<br>- Lazy model loading<br>- Profiling and optimization<br>- Batch processing |
| **Vault Data Loss** | Low | High | 🟡 MEDIUM | - TTL warnings<br>- Persistent vault options<br>- Replication (Redis)<br>- Graceful degradation |
| **Memory Leaks** | Low | High | 🟢 LOW | - Arc/RwLock patterns<br>- Automatic cleanup<br>- Memory profiling<br>- Load testing |

### 10.2 Compliance Risks

| Risk | Probability | Impact | Severity | Mitigation |
|------|-------------|--------|----------|------------|
| **GDPR Non-Compliance** | Medium | Critical | 🔴 HIGH | - Built-in GDPR mode<br>- TTL enforcement<br>- Audit logging<br>- Legal review |
| **HIPAA Non-Compliance** | Medium | Critical | 🔴 HIGH | - All 18 identifiers<br>- Encryption required<br>- Access controls<br>- Compliance testing |
| **PCI Non-Compliance** | Low | Critical | 🟡 MEDIUM | - Luhn validation<br>- PAN masking<br>- Log redaction<br>- Audit trail |
| **Audit Trail Gaps** | Low | Medium | 🟢 LOW | - Comprehensive logging<br>- Immutable logs<br>- Regular audits |

### 10.3 Security Risks

| Risk | Probability | Impact | Severity | Mitigation |
|------|-------------|--------|----------|------------|
| **Vault Compromise** | Low | Critical | 🟡 MEDIUM | - AES-256 encryption<br>- Access controls<br>- Session isolation<br>- Security audits |
| **PII Leakage in Logs** | Low | Critical | 🟡 MEDIUM | - Log redaction<br>- No PII in logs<br>- Automated checks<br>- Code review |
| **Session Hijacking** | Low | High | 🟢 LOW | - Unique session IDs<br>- TTL expiration<br>- No guessable tokens |
| **Placeholder Collisions** | Very Low | Low | 🟢 LOW | - Session-scoped counters<br>- UUID option<br>- Validation tests |

### 10.4 Integration Risks

| Risk | Probability | Impact | Severity | Mitigation |
|------|-------------|--------|----------|------------|
| **Scanner API Incompatibility** | Low | Medium | 🟢 LOW | - Follow Scanner trait<br>- Integration tests<br>- Clear documentation |
| **Phase 8 Dependency Issues** | Low | Medium | 🟢 LOW | - Early integration<br>- Version pinning<br>- Regular testing |
| **WASM Size Bloat** | Medium | Low | 🟢 LOW | - Lazy loading<br>- Tree shaking<br>- Optimize ONNX models |

### 10.5 Risk Matrix

```
IMPACT
   ↑
HIGH│  [ONNX Conv]     [NER Accuracy] [Vault Loss]
    │                  [GDPR] [HIPAA]
    │
MED │  [Perf Deg]      [Scanner API]  [Vault Comp]
    │  [PCI]                           [PII Logs]
    │
LOW │  [Memory]        [Session Hijack] [Collisions]
    │  [WASM Size]     [Audit Gaps]     [Integration]
    │
    └────────────────────────────────────────────→
      LOW           MEDIUM          HIGH      PROBABILITY
```

---

## 11. SUCCESS METRICS

### 11.1 Technical Metrics

**Accuracy:**
- ✅ Entity Detection (Regex): 85-90%
- ✅ Entity Detection (NER): 90-95%
- ✅ Entity Detection (Hybrid): 95-99%
- ✅ Deanonymization: 100% (when mapping exists)
- ✅ False Positive Rate: <5%

**Performance:**
- ✅ Anonymization Latency (p50): <20ms
- ✅ Anonymization Latency (p95): <50ms
- ✅ Anonymization Latency (p99): <100ms
- ✅ Deanonymization Latency (p95): <5ms
- ✅ Vault Get/Set (p95): <0.1ms
- ✅ NER Inference (p95): <50ms (CPU)
- ✅ Throughput: 1,000+ req/sec

**Reliability:**
- ✅ Test Coverage: ≥90%
- ✅ All Tests Passing: 105/105
- ✅ Zero Memory Leaks: Validated
- ✅ Concurrent Sessions: 10,000+
- ✅ Vault Cleanup: <5% overhead

### 11.2 Compliance Metrics

**GDPR:**
- ✅ Data Minimization: Only PII + session stored
- ✅ Right to Erasure: `delete_session()` implemented
- ✅ Storage Limitation: TTL enforced (1 hour default)
- ✅ Encryption: AES-256 at rest
- ✅ Audit Trail: All operations logged

**HIPAA:**
- ✅ 18 Identifiers: All supported
- ✅ De-identification: Safe Harbor method
- ✅ Access Controls: Session-scoped
- ✅ Audit Logging: Comprehensive
- ✅ Encryption: Required and validated

**PCI-DSS:**
- ✅ PAN Masking: Show last 4 only
- ✅ No PAN in Logs: Redacted
- ✅ Audit Trail: Card data access logged
- ✅ Luhn Validation: Only valid cards
- ✅ Storage Prohibition: Enforced

### 11.3 User Experience Metrics

**Developer Experience:**
- ✅ API Simplicity: 2 main methods (`anonymize`, `deanonymize`)
- ✅ Configuration Presets: 4 presets (default, GDPR, HIPAA, PCI)
- ✅ Documentation: Complete (API + guides)
- ✅ Examples: 10+ usage examples
- ✅ Error Messages: Clear and actionable

**Integration:**
- ✅ Scanner Integration: <1 hour
- ✅ Phase 8 Integration: Seamless
- ✅ WASM Support: Full
- ✅ Backward Compatibility: Maintained

### 11.4 Business Metrics

**Time to Value:**
- ✅ Setup Time: <15 minutes
- ✅ First Anonymization: <5 minutes
- ✅ Production Deployment: <1 day

**Cost:**
- ✅ Memory Usage: <100MB for 10K sessions
- ✅ CPU Usage: <5% overhead
- ✅ Storage: Minimal (in-memory default)

**Risk Reduction:**
- ✅ Zero PII in LLM logs
- ✅ Compliance risk eliminated
- ✅ Audit trail for reporting
- ✅ Security posture improved

---

## 12. APPENDICES

### Appendix A: Entity Types Reference

| Entity Type | Regex | NER | Example | GDPR | HIPAA | PCI |
|-------------|-------|-----|---------|------|-------|-----|
| PERSON | ❌ | ✅ | "John Doe" | ✅ | ✅ | ❌ |
| EMAIL | ✅ | ❌ | john@example.com | ✅ | ✅ | ❌ |
| PHONE_NUMBER | ✅ | ❌ | +1-555-123-4567 | ✅ | ✅ | ❌ |
| SSN | ✅ | ❌ | 123-45-6789 | ✅ | ✅ | ❌ |
| CREDIT_CARD | ✅ | ❌ | 4532-1488-0343-6467 | ❌ | ❌ | ✅ |
| IP_ADDRESS | ✅ | ❌ | 192.168.1.1 | ✅ | ✅ | ❌ |
| URL | ✅ | ❌ | https://example.com | ✅ | ✅ | ❌ |
| DATE_OF_BIRTH | ✅ | ❌ | 01/15/1990 | ✅ | ✅ | ❌ |
| BANK_ACCOUNT | ✅ | ❌ | 12345678901234 | ✅ | ✅ | ❌ |
| ADDRESS | ❌ | ✅ | "123 Main St" | ✅ | ✅ | ❌ |
| ORGANIZATION | ❌ | ✅ | "Acme Corp" | ❌ | ❌ | ❌ |
| DRIVER_LICENSE | ✅ | ❌ | DL-123456 | ✅ | ✅ | ❌ |
| PASSPORT | ✅ | ❌ | AB1234567 | ✅ | ✅ | ❌ |
| MEDICAL_RECORD | ✅ | ❌ | MRN-789012 | ❌ | ✅ | ❌ |
| POSTAL_CODE | ✅ | ❌ | 12345 | ✅ | ✅ | ❌ |

### Appendix B: Configuration Examples

**Default Configuration:**
```rust
let config = AnonymizerConfig {
    entity_types: vec![
        EntityType::Person, EntityType::Email, EntityType::PhoneNumber,
        EntityType::SSN, EntityType::CreditCard, EntityType::IPAddress,
    ],
    detection_strategy: DetectionStrategy::Hybrid,
    placeholder_format: PlaceholderFormat::Numbered,
    vault_backend: VaultBackend::Memory,
    vault_ttl: Duration::from_secs(3600), // 1 hour
    compliance_mode: ComplianceMode::None,
};
```

**GDPR Compliant:**
```rust
let config = AnonymizerConfig::gdpr_compliant();
// Includes:
// - All entity types
// - Hybrid detection
// - 1 hour TTL (max)
// - Encryption required
// - Audit logging enabled
// - Right to erasure supported
```

**HIPAA Compliant:**
```rust
let config = AnonymizerConfig::hipaa_compliant();
// Includes:
// - All 18 HIPAA identifiers
// - Hybrid detection
// - 30 minutes TTL
// - Encryption required
// - Access controls
// - Comprehensive audit logging
```

**PCI Compliant:**
```rust
let config = AnonymizerConfig::pci_compliant();
// Includes:
// - Credit card detection with Luhn
// - Show last 4 masking
// - No PAN in logs
// - Audit trail for card data
// - Short TTL (15 minutes)
```

### Appendix C: Glossary

- **Anonymization:** Replacing PII with placeholder tokens
- **Deanonymization:** Restoring original PII from placeholders
- **PII (Personally Identifiable Information):** Data that can identify an individual
- **NER (Named Entity Recognition):** ML technique for extracting entities from text
- **BIO Tagging:** Begin, Inside, Outside - NER labeling format
- **ONNX:** Open Neural Network Exchange - ML model format
- **Luhn Algorithm:** Checksum validation for credit card numbers
- **TTL (Time To Live):** Expiration time for stored data
- **Vault:** Secure storage for PII mappings
- **Session:** Unique anonymization context with mappings
- **Placeholder:** Token representing anonymized entity (e.g., [PERSON_1])
- **Hybrid Detection:** Combination of regex and NER for best accuracy
- **Graceful Degradation:** Continuing operation with reduced functionality on failure

### Appendix D: References

**Standards & Regulations:**
- GDPR: https://gdpr.eu/
- HIPAA: https://www.hhs.gov/hipaa/
- PCI-DSS: https://www.pcisecuritystandards.org/

**ML Models:**
- ai4privacy/pii-detection-deberta-v3-base: https://huggingface.co/ai4privacy/pii-detection-deberta-v3-base
- Microsoft Presidio: https://github.com/microsoft/presidio

**Rust Crates:**
- regex: https://docs.rs/regex/
- ort (ONNX Runtime): https://docs.rs/ort/
- fake-rs: https://docs.rs/fake/
- redis: https://docs.rs/redis/
- rusqlite: https://docs.rs/rusqlite/

**Prior Art:**
- Python LLM Guard: https://github.com/protectai/llm-guard
- SecretScout: https://github.com/globalbusinessadvisors/SecretScout

---

## APPROVAL SIGN-OFF

**Phase 9 Implementation Plan**

| Role | Name | Status | Date |
|------|------|--------|------|
| **Technical Lead** | ___________ | ☐ Approved | ______ |
| **Security Lead** | ___________ | ☐ Approved | ______ |
| **Compliance Officer** | ___________ | ☐ Approved | ______ |
| **Product Manager** | ___________ | ☐ Approved | ______ |

**Approval Criteria:**
- ☐ Technical approach validated
- ☐ Compliance requirements met
- ☐ Security risks assessed
- ☐ Timeline and budget approved
- ☐ Success metrics defined

**Next Steps:**
1. Approval sign-off
2. Kickoff meeting (Week 1, Day 1)
3. Begin Phase 9A implementation
4. Weekly progress reviews

---

**Document Version:** 1.0
**Last Updated:** 2025-10-31
**Status:** Ready for Approval
**Estimated Start Date:** TBD
**Estimated Completion:** TBD + 10 weeks
