//! Rate limiting types and models
//!
//! ## SPARC Phase 3: Construction (TDD - RED Phase)
//!
//! Core data structures for rate limiting following the specification.

use crate::config::rate_limit::{RateLimitTier, TierLimits};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Decision from rate limiter
///
/// ## Invariants
/// - `remaining <= limit` always holds
/// - `reset_at` is always in the future
/// - `retry_after.is_some()` IFF `allowed == false`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl RateLimitDecision {
    /// Create a decision allowing the request
    pub fn allow(limit: u32, remaining: u32, reset_at: u64) -> Self {
        Self {
            allowed: true,
            limit,
            remaining,
            reset_at,
            retry_after: None,
        }
    }

    /// Create a decision denying the request
    pub fn deny(limit: u32, reset_at: u64, retry_after: u32) -> Self {
        Self {
            allowed: false,
            limit,
            remaining: 0,
            reset_at,
            retry_after: Some(retry_after),
        }
    }

    /// Validate invariants (for testing)
    #[cfg(test)]
    pub fn validate_invariants(&self) -> Result<(), String> {
        // Invariant 1: remaining <= limit
        if self.remaining > self.limit {
            return Err(format!(
                "remaining ({}) > limit ({})",
                self.remaining, self.limit
            ));
        }

        // Invariant 2: reset_at is in the future
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if self.reset_at <= now {
            return Err(format!(
                "reset_at ({}) is not in the future (now: {})",
                self.reset_at, now
            ));
        }

        // Invariant 3: retry_after.is_some() IFF allowed == false
        if !self.allowed && self.retry_after.is_none() {
            return Err("retry_after must be Some when allowed is false".to_string());
        }

        if self.allowed && self.retry_after.is_some() {
            return Err("retry_after must be None when allowed is true".to_string());
        }

        Ok(())
    }
}

/// Quota usage across multiple time windows
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaUsage {
    /// Requests in current minute
    pub minute: u32,

    /// Requests in current hour
    pub hour: u32,

    /// Requests in current day
    pub day: u32,

    /// Requests in current month
    pub month: u32,
}

impl QuotaUsage {
    /// Create new empty quota usage
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if usage exceeds limits
    pub fn exceeds(&self, limits: &TierLimits) -> bool {
        self.minute >= limits.requests_per_minute
            || self.hour >= limits.requests_per_hour
            || self.day >= limits.requests_per_day
    }

    /// Increment all counters
    pub fn increment(&mut self) {
        self.minute += 1;
        self.hour += 1;
        self.day += 1;
        self.month += 1;
    }
}

/// Window type for quota tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Window {
    Minute,
    Hour,
    Day,
    Month,
}

impl Window {
    /// Get the window duration in seconds
    pub fn duration_secs(&self) -> u64 {
        match self {
            Window::Minute => 60,
            Window::Hour => 3600,
            Window::Day => 86400,
            Window::Month => 2_592_000, // 30 days
        }
    }

    /// Calculate the next reset time for this window
    pub fn next_reset(&self, now: SystemTime) -> SystemTime {
        let duration = std::time::Duration::from_secs(self.duration_secs());
        now + duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_decision_allow() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let decision = RateLimitDecision::allow(100, 95, now + 60);

        assert!(decision.allowed);
        assert_eq!(decision.limit, 100);
        assert_eq!(decision.remaining, 95);
        assert_eq!(decision.reset_at, now + 60);
        assert!(decision.retry_after.is_none());

        // Validate invariants
        assert!(decision.validate_invariants().is_ok());
    }

    #[test]
    fn test_rate_limit_decision_deny() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let decision = RateLimitDecision::deny(100, now + 60, 60);

        assert!(!decision.allowed);
        assert_eq!(decision.limit, 100);
        assert_eq!(decision.remaining, 0);
        assert_eq!(decision.reset_at, now + 60);
        assert_eq!(decision.retry_after, Some(60));

        // Validate invariants
        assert!(decision.validate_invariants().is_ok());
    }

    #[test]
    fn test_rate_limit_decision_invariant_remaining_le_limit() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let invalid_decision = RateLimitDecision {
            allowed: true,
            limit: 100,
            remaining: 150, // Invalid: exceeds limit
            reset_at: now + 60,
            retry_after: None,
        };

        assert!(invalid_decision.validate_invariants().is_err());
    }

    #[test]
    fn test_quota_usage_new() {
        let usage = QuotaUsage::new();
        assert_eq!(usage.minute, 0);
        assert_eq!(usage.hour, 0);
        assert_eq!(usage.day, 0);
        assert_eq!(usage.month, 0);
    }

    #[test]
    fn test_quota_usage_increment() {
        let mut usage = QuotaUsage::new();
        usage.increment();

        assert_eq!(usage.minute, 1);
        assert_eq!(usage.hour, 1);
        assert_eq!(usage.day, 1);
        assert_eq!(usage.month, 1);
    }

    #[test]
    fn test_quota_usage_exceeds() {
        use crate::config::rate_limit::TierLimits;

        let limits = TierLimits {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        let mut usage = QuotaUsage::new();
        assert!(!usage.exceeds(&limits));

        // Exceed minute limit
        usage.minute = 10;
        assert!(usage.exceeds(&limits));

        // Reset minute but exceed hour
        usage.minute = 5;
        usage.hour = 100;
        assert!(usage.exceeds(&limits));

        // Reset hour but exceed day
        usage.hour = 50;
        usage.day = 1000;
        assert!(usage.exceeds(&limits));
    }

    #[test]
    fn test_window_duration_secs() {
        assert_eq!(Window::Minute.duration_secs(), 60);
        assert_eq!(Window::Hour.duration_secs(), 3600);
        assert_eq!(Window::Day.duration_secs(), 86400);
        assert_eq!(Window::Month.duration_secs(), 2_592_000);
    }

    #[test]
    fn test_window_next_reset() {
        let now = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000);

        let next_minute = Window::Minute.next_reset(now);
        let expected_minute = now + std::time::Duration::from_secs(60);
        assert_eq!(next_minute, expected_minute);

        let next_hour = Window::Hour.next_reset(now);
        let expected_hour = now + std::time::Duration::from_secs(3600);
        assert_eq!(next_hour, expected_hour);
    }
}
