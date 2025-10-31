//! Rate limiting module
//!
//! ## Overview
//!
//! Multi-tier rate limiting with:
//! - Token bucket algorithm (per-minute limits)
//! - Multi-window quota tracking (minute, hour, day, month)
//! - Concurrent request limiting (semaphores)
//! - Tier-based limits (Free, Pro, Enterprise)
//!
//! ## Architecture
//!
//! ```text
//! Request → RateLimiter → [Token Bucket] → [Quota Check] → Decision
//!                              ↓                ↓
//!                         governor           QuotaTracker
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llm_shield_api::rate_limiting::{MultiTierRateLimiter, RateLimiter};
//! use llm_shield_api::config::{RateLimitConfig, RateLimitTier};
//!
//! let config = RateLimitConfig::default();
//! let limiter = MultiTierRateLimiter::new(config);
//!
//! let decision = limiter.check_rate_limit("user123", RateLimitTier::Free).await;
//!
//! if decision.allowed {
//!     // Process request
//! } else {
//!     // Return 429 with retry_after
//! }
//! ```

pub mod concurrent;
pub mod limiter;
pub mod quota;
pub mod types;

// Re-exports
pub use concurrent::{ConcurrentLimiter, ConcurrentPermit};
pub use limiter::{MultiTierRateLimiter, RateLimiter};
pub use quota::QuotaTracker;
pub use types::{QuotaUsage, RateLimitDecision, Window};
