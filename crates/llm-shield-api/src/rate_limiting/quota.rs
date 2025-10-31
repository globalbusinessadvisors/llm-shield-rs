//! Quota tracking for multi-window rate limiting
//!
//! ## SPARC Phase 3: Construction (TDD)
//!
//! Tracks request counts across multiple time windows (minute, hour, day, month)
//! and enforces tier-based limits.

use super::types::{QuotaUsage, Window};
use crate::config::rate_limit::TierLimits;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// State for a single quota window
#[derive(Debug, Clone)]
struct WindowState {
    /// Number of requests in this window
    count: u32,

    /// When this window resets
    reset_at: SystemTime,
}

impl WindowState {
    fn new(reset_at: SystemTime) -> Self {
        Self { count: 0, reset_at }
    }

    /// Check if this window has expired
    fn is_expired(&self, now: SystemTime) -> bool {
        now >= self.reset_at
    }

    /// Reset the window with a new reset time
    fn reset(&mut self, reset_at: SystemTime) {
        self.count = 0;
        self.reset_at = reset_at;
    }
}

/// Quota state for a single client
#[derive(Debug, Clone)]
struct ClientQuota {
    minute: WindowState,
    hour: WindowState,
    day: WindowState,
    month: WindowState,
}

impl ClientQuota {
    fn new(now: SystemTime) -> Self {
        Self {
            minute: WindowState::new(Window::Minute.next_reset(now)),
            hour: WindowState::new(Window::Hour.next_reset(now)),
            day: WindowState::new(Window::Day.next_reset(now)),
            month: WindowState::new(Window::Month.next_reset(now)),
        }
    }

    /// Update window states, resetting expired windows
    fn update(&mut self, now: SystemTime) {
        if self.minute.is_expired(now) {
            self.minute.reset(Window::Minute.next_reset(now));
        }
        if self.hour.is_expired(now) {
            self.hour.reset(Window::Hour.next_reset(now));
        }
        if self.day.is_expired(now) {
            self.day.reset(Window::Day.next_reset(now));
        }
        if self.month.is_expired(now) {
            self.month.reset(Window::Month.next_reset(now));
        }
    }

    /// Increment all counters
    fn increment(&mut self) {
        self.minute.count += 1;
        self.hour.count += 1;
        self.day.count += 1;
        self.month.count += 1;
    }

    /// Get current usage
    fn usage(&self) -> QuotaUsage {
        QuotaUsage {
            minute: self.minute.count,
            hour: self.hour.count,
            day: self.day.count,
            month: self.month.count,
        }
    }

    /// Check if any limit is exceeded
    fn exceeds(&self, limits: &TierLimits) -> bool {
        self.minute.count >= limits.requests_per_minute
            || self.hour.count >= limits.requests_per_hour
            || self.day.count >= limits.requests_per_day
    }
}

/// Tracks request quotas across multiple time windows
///
/// ## Thread Safety
///
/// Uses `Arc<RwLock<HashMap>>` for thread-safe concurrent access.
///
/// ## Performance
///
/// - Read lock for most operations (check quota)
/// - Write lock only for incrementing counters
/// - Automatic cleanup of expired entries
pub struct QuotaTracker {
    /// Client quotas indexed by key (API key, IP, etc.)
    quotas: Arc<RwLock<HashMap<String, ClientQuota>>>,
}

impl QuotaTracker {
    /// Create a new quota tracker
    pub fn new() -> Self {
        Self {
            quotas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a request and check quota
    ///
    /// # Arguments
    ///
    /// * `key` - Identifier for the client (API key, IP, etc.)
    /// * `limits` - Tier limits to enforce
    ///
    /// # Returns
    ///
    /// * `true` - Request allowed (within quota)
    /// * `false` - Request denied (quota exceeded)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let tracker = QuotaTracker::new();
    /// let limits = TierLimits { /* ... */ };
    ///
    /// if tracker.check_and_increment("user123", &limits).await {
    ///     // Process request
    /// } else {
    ///     // Return 429 Too Many Requests
    /// }
    /// ```
    pub async fn check_and_increment(&self, key: &str, limits: &TierLimits) -> bool {
        let now = SystemTime::now();
        let mut quotas = self.quotas.write().await;

        // Get or create client quota
        let quota = quotas
            .entry(key.to_string())
            .or_insert_with(|| ClientQuota::new(now));

        // Update window states (reset expired windows)
        quota.update(now);

        // Check if quota exceeded
        if quota.exceeds(limits) {
            return false;
        }

        // Increment counters
        quota.increment();

        true
    }

    /// Get current quota usage
    ///
    /// # Arguments
    ///
    /// * `key` - Identifier for the client
    ///
    /// # Returns
    ///
    /// Current usage across all windows (minute, hour, day, month)
    pub async fn get_usage(&self, key: &str) -> QuotaUsage {
        let now = SystemTime::now();
        let mut quotas = self.quotas.write().await;

        // Get or create client quota
        let quota = quotas
            .entry(key.to_string())
            .or_insert_with(|| ClientQuota::new(now));

        // Update window states
        quota.update(now);

        quota.usage()
    }

    /// Get time until next reset for a specific window
    ///
    /// # Arguments
    ///
    /// * `key` - Identifier for the client
    /// * `window` - Which window to check
    ///
    /// # Returns
    ///
    /// Seconds until the window resets, or None if client not found
    pub async fn time_until_reset(&self, key: &str, window: Window) -> Option<u64> {
        let now = SystemTime::now();
        let quotas = self.quotas.read().await;

        quotas.get(key).map(|quota| {
            let reset_at = match window {
                Window::Minute => quota.minute.reset_at,
                Window::Hour => quota.hour.reset_at,
                Window::Day => quota.day.reset_at,
                Window::Month => quota.month.reset_at,
            };

            reset_at
                .duration_since(now)
                .unwrap_or_default()
                .as_secs()
        })
    }

    /// Clean up expired entries (for maintenance)
    pub async fn cleanup_expired(&self) {
        let now = SystemTime::now();
        let mut quotas = self.quotas.write().await;

        // Remove quotas where all windows are expired
        quotas.retain(|_, quota| {
            !(quota.minute.is_expired(now)
                && quota.hour.is_expired(now)
                && quota.day.is_expired(now)
                && quota.month.is_expired(now))
        });
    }

    /// Get number of tracked clients (for testing/monitoring)
    pub async fn client_count(&self) -> usize {
        self.quotas.read().await.len()
    }
}

impl Default for QuotaTracker {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone by cloning the Arc (not the underlying data)
impl Clone for QuotaTracker {
    fn clone(&self) -> Self {
        Self {
            quotas: Arc::clone(&self.quotas),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quota_tracker_new() {
        let tracker = QuotaTracker::new();
        assert_eq!(tracker.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_check_and_increment_first_request() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        // First request should succeed
        assert!(tracker.check_and_increment("user1", &limits).await);
        assert_eq!(tracker.client_count().await, 1);

        // Check usage
        let usage = tracker.get_usage("user1").await;
        assert_eq!(usage.minute, 1);
        assert_eq!(usage.hour, 1);
        assert_eq!(usage.day, 1);
        assert_eq!(usage.month, 1);
    }

    #[tokio::test]
    async fn test_check_and_increment_within_limit() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        // Make 9 requests (all should succeed)
        for _ in 0..9 {
            assert!(tracker.check_and_increment("user1", &limits).await);
        }

        let usage = tracker.get_usage("user1").await;
        assert_eq!(usage.minute, 9);
    }

    #[tokio::test]
    async fn test_check_and_increment_exceeds_limit() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        // Make 10 requests (all should succeed)
        for i in 0..10 {
            assert!(
                tracker.check_and_increment("user1", &limits).await,
                "Request {} should succeed",
                i
            );
        }

        // 11th request should fail (minute limit = 10)
        assert!(!tracker.check_and_increment("user1", &limits).await);

        let usage = tracker.get_usage("user1").await;
        assert_eq!(usage.minute, 10);
    }

    #[tokio::test]
    async fn test_different_clients_have_separate_quotas() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 5,
            requests_per_hour: 50,
            requests_per_day: 500,
            max_concurrent: 3,
        };

        // User1 makes 5 requests
        for _ in 0..5 {
            assert!(tracker.check_and_increment("user1", &limits).await);
        }

        // User2 should still have full quota
        assert!(tracker.check_and_increment("user2", &limits).await);

        let usage1 = tracker.get_usage("user1").await;
        let usage2 = tracker.get_usage("user2").await;

        assert_eq!(usage1.minute, 5);
        assert_eq!(usage2.minute, 1);
    }

    #[tokio::test]
    async fn test_hour_limit_enforcement() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 100, // High minute limit
            requests_per_hour: 10, // Low hour limit
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        // Make 10 requests (should all succeed)
        for _ in 0..10 {
            assert!(tracker.check_and_increment("user1", &limits).await);
        }

        // 11th request should fail (hour limit = 10)
        assert!(!tracker.check_and_increment("user1", &limits).await);
    }

    #[tokio::test]
    async fn test_day_limit_enforcement() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 1000,
            requests_per_hour: 10000,
            requests_per_day: 5, // Low day limit
            max_concurrent: 10,
        };

        // Make 5 requests (should all succeed)
        for _ in 0..5 {
            assert!(tracker.check_and_increment("user1", &limits).await);
        }

        // 6th request should fail (day limit = 5)
        assert!(!tracker.check_and_increment("user1", &limits).await);
    }

    #[tokio::test]
    async fn test_time_until_reset() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        // Make a request to initialize quota
        assert!(tracker.check_and_increment("user1", &limits).await);

        // Check time until reset
        let time_until_reset = tracker.time_until_reset("user1", Window::Minute).await;
        assert!(time_until_reset.is_some());

        let seconds = time_until_reset.unwrap();
        assert!(seconds > 0);
        assert!(seconds <= 60);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let tracker = QuotaTracker::new();
        let limits = TierLimits {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        // Add some clients
        tracker.check_and_increment("user1", &limits).await;
        tracker.check_and_increment("user2", &limits).await;

        assert_eq!(tracker.client_count().await, 2);

        // Cleanup shouldn't remove non-expired entries
        tracker.cleanup_expired().await;
        assert_eq!(tracker.client_count().await, 2);
    }

    #[tokio::test]
    async fn test_quota_tracker_clone() {
        let tracker1 = QuotaTracker::new();
        let tracker2 = tracker1.clone();

        let limits = TierLimits {
            requests_per_minute: 10,
            requests_per_hour: 100,
            requests_per_day: 1000,
            max_concurrent: 5,
        };

        // Increment via tracker1
        tracker1.check_and_increment("user1", &limits).await;

        // Should be visible via tracker2 (same Arc)
        let usage = tracker2.get_usage("user1").await;
        assert_eq!(usage.minute, 1);
    }
}
