# Phase 10B: Enhanced REST API - Specification

**Project:** LLM Shield Rust/WASM
**Phase:** 10B - Enhanced REST API Features
**Date:** 2025-10-31
**Methodology:** SPARC (Specification Phase) + London School TDD
**Status:** 📋 SPECIFICATION

---

## Table of Contents

1. [Overview](#overview)
2. [Acceptance Criteria](#acceptance-criteria)
3. [Functional Requirements](#functional-requirements)
4. [Non-Functional Requirements](#non-functional-requirements)
5. [Interface Specifications](#interface-specifications)
6. [Data Models](#data-models)
7. [Invariants and Constraints](#invariants-and-constraints)
8. [Test Strategy](#test-strategy)
9. [Success Metrics](#success-metrics)

---

## Overview

Phase 10B enhances the REST API (Phase 10A) with enterprise-grade security and documentation features:

1. **Rate Limiting** - Multi-tier, multi-window rate limiting to prevent abuse
2. **Authentication** - API key-based authentication with secure storage
3. **OpenAPI Documentation** - Auto-generated API documentation with Swagger UI

**Dependencies:**
- Phase 10A: REST API foundation (✅ Complete)
- `governor` crate for token bucket rate limiting
- `argon2` crate for secure key hashing
- `utoipa` crate for OpenAPI schema generation

---

## Acceptance Criteria

### AC1: Rate Limiting

**Given** a client making requests to the API
**When** the client exceeds their tier's rate limit
**Then** the API returns HTTP 429 (Too Many Requests) with appropriate headers

**Acceptance Tests:**
1. Free tier limited to 10 req/min
2. Pro tier limited to 100 req/min
3. Enterprise tier limited to 1000 req/min
4. Rate limit headers include: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
5. Concurrent request limiting prevents >N simultaneous requests

### AC2: Authentication

**Given** a client with a valid API key
**When** the client includes the key in the `Authorization: Bearer <key>` header
**Then** the API authenticates the request and proceeds

**Acceptance Tests:**
1. Valid API key grants access
2. Invalid API key returns HTTP 401 (Unauthorized)
3. Missing API key returns HTTP 401
4. Expired API key returns HTTP 401
5. API keys are stored securely with argon2 hashing
6. Key rotation does not break existing valid keys

### AC3: OpenAPI Documentation

**Given** the API is running
**When** a client navigates to `/docs`
**Then** Swagger UI displays interactive API documentation

**Acceptance Tests:**
1. All endpoints documented with request/response schemas
2. Authentication scheme documented
3. Example requests/responses included
4. OpenAPI 3.0+ compliant specification
5. `/api-docs/openapi.json` endpoint serves raw specification

---

## Functional Requirements

### FR1: Rate Limiting

#### FR1.1: Token Bucket Rate Limiting
- Implement token bucket algorithm using `governor` crate
- Support per-minute, per-hour, and per-day windows
- Support tier-based limits (Free, Pro, Enterprise)

#### FR1.2: Quota Tracking
- Track request counts across multiple time windows
- Reset counters at window boundaries
- Persist quota data in-memory (file/Redis optional)

#### FR1.3: Concurrent Request Limiting
- Limit simultaneous requests per client using semaphores
- Default: 10 concurrent requests per client
- Configurable per tier

#### FR1.4: Rate Limit Response Headers
```
X-RateLimit-Limit: <max requests per window>
X-RateLimit-Remaining: <requests remaining>
X-RateLimit-Reset: <unix timestamp when limit resets>
Retry-After: <seconds until retry allowed>
```

### FR2: Authentication

#### FR2.1: API Key Generation
- Generate cryptographically secure random keys (32 bytes)
- Format: `llm_shield_<40 character base62 string>`
- Example: `llm_shield_k3jH8mN2pQ9rT5vW1xY4zA7bC6dE0fG8hJ`

#### FR2.2: API Key Storage
- Store keys with argon2id hashing
- Support multiple storage backends via trait:
  - `MemoryKeyStorage` - In-memory HashMap
  - `FileKeyStorage` - JSON file persistence
  - `RedisKeyStorage` - Redis backend (optional)

#### FR2.3: API Key Validation
- Extract key from `Authorization: Bearer <key>` header
- Validate key format
- Verify key exists and is not expired
- Check key permissions/tier

#### FR2.4: Role-Based Access Control
- Associate each key with a tier (Free, Pro, Enterprise)
- Each tier has different rate limits and quotas
- Keys can have expiration dates

### FR3: OpenAPI Documentation

#### FR3.1: Schema Generation
- Use `utoipa` derives on all DTOs
- Generate OpenAPI 3.0+ specification
- Include all endpoints, request/response schemas

#### FR3.2: Swagger UI Integration
- Serve Swagger UI at `/docs`
- Serve OpenAPI JSON at `/api-docs/openapi.json`
- Include "Try it out" functionality with auth

#### FR3.3: Documentation Content
- Endpoint descriptions
- Request/response examples
- Error response schemas
- Authentication requirements
- Rate limiting information

---

## Non-Functional Requirements

### NFR1: Performance

| Metric | Requirement |
|--------|-------------|
| Rate limit check latency | <1ms p95 |
| Auth validation latency | <0.5ms p95 |
| Throughput with rate limiting | >10,000 req/s |
| Memory usage (10k active keys) | <50MB |

### NFR2: Security

- **SEC-1**: API keys MUST be hashed with argon2id before storage
- **SEC-2**: Keys MUST use cryptographically secure random generation
- **SEC-3**: Constant-time comparison for key validation
- **SEC-4**: Rate limiting MUST prevent DoS attacks
- **SEC-5**: No sensitive data in logs (mask API keys)

### NFR3: Reliability

- **REL-1**: Rate limiter state MUST be thread-safe
- **REL-2**: Key storage MUST be thread-safe
- **REL-3**: Service MUST handle storage backend failures gracefully
- **REL-4**: No data loss on clean shutdown (file persistence)

### NFR4: Observability

- **OBS-1**: Emit metrics for rate limit hits/misses
- **OBS-2**: Emit metrics for auth success/failure
- **OBS-3**: Log rate limit violations at WARN level
- **OBS-4**: Log auth failures at WARN level
- **OBS-5**: Trace all rate limit and auth decisions

### NFR5: Maintainability

- **MAINT-1**: 90%+ test coverage for all new code
- **MAINT-2**: All public APIs documented with rustdoc
- **MAINT-3**: Configuration validation on startup
- **MAINT-4**: Clear error messages for misconfigurations

---

## Interface Specifications

### Rate Limiting Interfaces

#### `RateLimiter` Trait

```rust
/// Rate limiter for request throttling
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request should be allowed
    ///
    /// # Arguments
    /// * `key` - Identifier for the client (API key, IP, etc.)
    /// * `tier` - Client's rate limit tier
    ///
    /// # Returns
    /// * `Ok(RateLimitDecision)` - Decision with metadata
    /// * `Err(Error)` - Rate limiter error
    async fn check_rate_limit(
        &self,
        key: &str,
        tier: RateLimitTier,
    ) -> Result<RateLimitDecision>;
}

/// Decision from rate limiter
pub struct RateLimitDecision {
    /// Whether the request is allowed
    pub allowed: bool,

    /// Current limit for this window
    pub limit: u32,

    /// Remaining requests in this window
    pub remaining: u32,

    /// Unix timestamp when limit resets
    pub reset_at: u64,

    /// Seconds until retry allowed (if not allowed)
    pub retry_after: Option<u32>,
}
```

**Invariants:**
- `remaining <= limit` always holds
- `reset_at` is always in the future
- `retry_after.is_some()` IFF `allowed == false`

#### `QuotaTracker` Interface

```rust
/// Tracks request quotas across multiple time windows
pub struct QuotaTracker {
    // Internal: Arc<RwLock<HashMap<String, QuotaState>>>
}

impl QuotaTracker {
    /// Record a request and check quota
    ///
    /// # Returns
    /// * `true` - Request allowed (within quota)
    /// * `false` - Request denied (quota exceeded)
    pub async fn check_and_increment(
        &self,
        key: &str,
        limits: &TierLimits,
    ) -> bool;

    /// Get current quota usage
    pub async fn get_usage(&self, key: &str) -> QuotaUsage;
}

pub struct QuotaUsage {
    pub minute: u32,
    pub hour: u32,
    pub day: u32,
    pub month: u32,
}
```

**Invariants:**
- `minute <= limits.per_minute` when check succeeds
- Counters reset at window boundaries
- Thread-safe access via RwLock

### Authentication Interfaces

#### `KeyStorage` Trait

```rust
/// Storage backend for API keys
#[async_trait]
pub trait KeyStorage: Send + Sync {
    /// Store a new API key
    async fn store(&self, key: &ApiKey) -> Result<()>;

    /// Retrieve an API key by its value
    async fn get(&self, key_value: &str) -> Result<Option<ApiKey>>;

    /// Delete an API key
    async fn delete(&self, key_value: &str) -> Result<()>;

    /// List all keys (for admin)
    async fn list(&self) -> Result<Vec<ApiKey>>;
}
```

#### `ApiKey` Model

```rust
/// API key with metadata
pub struct ApiKey {
    /// The raw key value (hashed in storage)
    pub value: String,

    /// Unique identifier
    pub id: String,

    /// Human-readable name/description
    pub name: String,

    /// Rate limit tier
    pub tier: RateLimitTier,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Optional expiration
    pub expires_at: Option<DateTime<Utc>>,

    /// Whether key is active
    pub active: bool,

    /// Hashed key (for storage)
    pub(crate) hashed_value: String,
}

impl ApiKey {
    /// Generate a new API key
    pub fn generate(name: String, tier: RateLimitTier) -> Result<Self>;

    /// Verify a raw key against this stored key
    pub fn verify(&self, raw_key: &str) -> Result<bool>;

    /// Check if key is expired
    pub fn is_expired(&self) -> bool;

    /// Check if key is valid (active and not expired)
    pub fn is_valid(&self) -> bool;
}
```

**Invariants:**
- `value` format: `llm_shield_[a-zA-Z0-9]{40}`
- `hashed_value` is argon2id hash of `value`
- `is_valid() == active && !is_expired()`
- `created_at <= expires_at` (if set)

#### `AuthService` Interface

```rust
/// Authentication service
pub struct AuthService {
    storage: Arc<dyn KeyStorage>,
}

impl AuthService {
    /// Create a new API key
    pub async fn create_key(
        &self,
        name: String,
        tier: RateLimitTier,
        expires_in_days: Option<u32>,
    ) -> Result<ApiKey>;

    /// Validate an API key from request
    pub async fn validate_key(&self, raw_key: &str) -> Result<ApiKey>;

    /// Revoke an API key
    pub async fn revoke_key(&self, key_value: &str) -> Result<()>;

    /// List all keys
    pub async fn list_keys(&self) -> Result<Vec<ApiKey>>;
}
```

### OpenAPI Interfaces

```rust
/// OpenAPI schema builder
pub struct OpenApiBuilder {
    // Internal: utoipa::openapi::OpenApi
}

impl OpenApiBuilder {
    /// Build OpenAPI specification
    pub fn build() -> utoipa::openapi::OpenApi;
}

/// Swagger UI handler
pub async fn swagger_ui() -> impl IntoResponse;

/// OpenAPI JSON handler
pub async fn openapi_json() -> impl IntoResponse;
```

---

## Data Models

### Rate Limiting Models

```rust
/// Rate limit tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitTier {
    Free,
    Pro,
    Enterprise,
}

/// Tier-specific limits
#[derive(Debug, Clone)]
pub struct TierLimits {
    pub per_minute: u32,
    pub per_hour: u32,
    pub per_day: u32,
    pub per_month: u32,
    pub concurrent: u32,
}

impl TierLimits {
    pub fn for_tier(tier: RateLimitTier) -> Self {
        match tier {
            RateLimitTier::Free => Self {
                per_minute: 10,
                per_hour: 100,
                per_day: 500,
                per_month: 10_000,
                concurrent: 2,
            },
            RateLimitTier::Pro => Self {
                per_minute: 100,
                per_hour: 1_000,
                per_day: 10_000,
                per_month: 200_000,
                concurrent: 10,
            },
            RateLimitTier::Enterprise => Self {
                per_minute: 1_000,
                per_hour: 10_000,
                per_day: 100_000,
                per_month: 2_000_000,
                concurrent: 50,
            },
        }
    }
}
```

### Authentication Models

```rust
/// Stored API key (with hash)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredApiKey {
    pub id: String,
    pub name: String,
    pub hashed_value: String,  // argon2id hash
    pub tier: RateLimitTier,
    pub created_at: i64,       // Unix timestamp
    pub expires_at: Option<i64>,
    pub active: bool,
}

/// Key generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    pub tier: RateLimitTier,
    pub expires_in_days: Option<u32>,
}

/// Key generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    pub key: String,            // Raw key (only shown once!)
    pub id: String,
    pub name: String,
    pub tier: RateLimitTier,
    pub created_at: String,
    pub expires_at: Option<String>,
}
```

---

## Invariants and Constraints

### Rate Limiting Invariants

1. **Monotonic Counters**: Request counters only increase within a window
2. **Window Boundaries**: Counters reset at exact window boundaries
3. **Tier Consistency**: A client's tier never changes mid-request
4. **Thread Safety**: All rate limit state is thread-safe (Arc + RwLock)
5. **No Negative Remaining**: `remaining >= 0` always holds

### Authentication Invariants

1. **Hash Security**: All stored keys MUST be hashed with argon2id
2. **Key Uniqueness**: No two keys share the same `id` or `value`
3. **Format Compliance**: All keys match `llm_shield_[a-zA-Z0-9]{40}`
4. **Expiration Check**: Expired keys MUST NOT validate
5. **Constant-Time Comparison**: Key verification uses constant-time comparison

### OpenAPI Invariants

1. **Schema Completeness**: All DTOs have `ToSchema` derives
2. **Endpoint Coverage**: All routes are documented
3. **Valid Spec**: Generated OpenAPI is valid per OpenAPI 3.0+ spec
4. **Security Schemes**: Authentication is properly documented

---

## Test Strategy

### London School TDD Approach

Following **outside-in** development:

1. **Write acceptance tests first** (API-level tests)
2. **Write unit tests with mocks** (individual components)
3. **Implement minimal code** to pass tests (GREEN)
4. **Refactor** for quality and performance

### Test Pyramid

```
        /\
       /  \      E2E (5-10 tests)
      /    \     - Full API integration
     /------\    - Auth + Rate limit + OpenAPI
    /        \
   /  INTEG   \  Integration (30-40 tests)
  /            \ - Middleware integration
 /--------------\- Multi-component flows
/                \
/      UNIT       \ Unit (50-60 tests)
/                  \- Individual components
/--------------------\- Mocked dependencies
```

**Target: 90+ tests, 90%+ coverage**

### Test Categories

#### 1. Rate Limiting Tests

**Unit Tests (15-20):**
- `test_token_bucket_allows_within_limit()`
- `test_token_bucket_denies_over_limit()`
- `test_quota_tracker_resets_at_window()`
- `test_concurrent_limiter_enforces_max()`
- `test_rate_limit_decision_invariants()`

**Integration Tests (10-15):**
- `test_rate_limit_middleware_returns_429()`
- `test_rate_limit_headers_included()`
- `test_different_tiers_have_different_limits()`
- `test_rate_limit_persists_across_requests()`

#### 2. Authentication Tests

**Unit Tests (20-25):**
- `test_api_key_generation_format()`
- `test_api_key_hashing_with_argon2()`
- `test_api_key_verification_success()`
- `test_api_key_verification_failure()`
- `test_expired_key_is_invalid()`
- `test_memory_storage_crud_operations()`
- `test_file_storage_persistence()`

**Integration Tests (10-15):**
- `test_auth_middleware_valid_key()`
- `test_auth_middleware_invalid_key_401()`
- `test_auth_middleware_missing_key_401()`
- `test_auth_middleware_expired_key_401()`
- `test_key_rotation_works()`

#### 3. OpenAPI Tests

**Unit Tests (5-10):**
- `test_openapi_spec_generation()`
- `test_all_dtos_have_schemas()`
- `test_security_scheme_documented()`

**Integration Tests (5):**
- `test_swagger_ui_accessible()`
- `test_openapi_json_endpoint()`
- `test_openapi_spec_validity()`

#### 4. End-to-End Tests

**E2E Tests (5-10):**
- `test_full_flow_with_auth_and_rate_limit()`
- `test_rate_limit_applies_per_key()`
- `test_different_tiers_enforced()`
- `test_openapi_docs_accessible()`

---

## Success Metrics

### Functional Completeness ✅

- [ ] Rate limiting enforces all tier limits
- [ ] Authentication validates API keys securely
- [ ] OpenAPI docs accessible and complete
- [ ] All acceptance criteria met

### Performance ✅

- [ ] Rate limit check: <1ms p95
- [ ] Auth validation: <0.5ms p95
- [ ] Throughput: >10,000 req/s
- [ ] Memory: <50MB for 10k keys

### Quality ✅

- [ ] 90+ tests passing
- [ ] 90%+ test coverage
- [ ] All public APIs documented
- [ ] No clippy warnings

### Security ✅

- [ ] Keys hashed with argon2id
- [ ] Constant-time key comparison
- [ ] No keys in logs
- [ ] Rate limiting prevents DoS

### Deliverables ✅

- [ ] Rate limiting module complete
- [ ] Authentication module complete
- [ ] OpenAPI integration complete
- [ ] Integration tests passing
- [ ] Documentation updated
- [ ] Completion report written

---

## Next Steps

After specification approval:

1. **Pseudocode Phase** - Design algorithms for each component
2. **Architecture Phase** - Create component diagrams and interactions
3. **Refinement Phase** - TDD implementation (RED → GREEN → REFACTOR)
4. **Completion Phase** - Documentation and validation

---

**Specification Status:** ✅ READY FOR IMPLEMENTATION

**Estimated Implementation Time:** 60-80 hours (3-4 weeks)

**Risk Level:** 🟡 MEDIUM (New crates, security-critical features)

**Commercial Viability:** ✅ ENTERPRISE-GRADE (Security, performance, documentation)
