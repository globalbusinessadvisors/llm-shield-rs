//! Concurrent request limiter using semaphores
//!
//! ## SPARC Phase 3: Construction (TDD)
//!
//! Limits the number of simultaneous requests per client using tokio semaphores.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};

/// Permit guard for concurrent requests
///
/// Automatically releases the semaphore permit when dropped.
pub struct ConcurrentPermit {
    #[allow(dead_code)]
    permit: OwnedSemaphorePermit,
}

/// Per-client semaphore for concurrent limiting
#[derive(Clone)]
struct ClientSemaphore {
    semaphore: Arc<Semaphore>,
    max_concurrent: usize,
}

impl ClientSemaphore {
    fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    /// Try to acquire a permit (non-blocking)
    fn try_acquire_owned(self) -> Option<OwnedSemaphorePermit> {
        Arc::clone(&self.semaphore).try_acquire_owned().ok()
    }

    /// Get available permits
    fn available(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// Get max concurrent
    fn max(&self) -> usize {
        self.max_concurrent
    }
}

/// Concurrent request limiter
///
/// ## Features
///
/// - Limits simultaneous requests per client
/// - Non-blocking permit acquisition
/// - Automatic permit release via RAII
/// - Tier-based concurrent limits
///
/// ## Example
///
/// ```rust,ignore
/// let limiter = ConcurrentLimiter::new();
/// let limits = TierLimits { max_concurrent: 10, /* ... */ };
///
/// if let Some(permit) = limiter.try_acquire("user1", &limits).await {
///     // Process request
///     // Permit automatically released when dropped
/// } else {
///     // Return 429 Too Many Requests
/// }
/// ```
pub struct ConcurrentLimiter {
    /// Client semaphores indexed by key
    semaphores: Arc<RwLock<HashMap<String, ClientSemaphore>>>,
}

impl ConcurrentLimiter {
    /// Create a new concurrent limiter
    pub fn new() -> Self {
        Self {
            semaphores: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Try to acquire a concurrent request permit
    ///
    /// # Arguments
    ///
    /// * `key` - Identifier for the client (API key, IP, etc.)
    /// * `max_concurrent` - Maximum concurrent requests for this client
    ///
    /// # Returns
    ///
    /// * `Some(ConcurrentPermit)` - Permit acquired, request allowed
    /// * `None` - No permits available, too many concurrent requests
    pub async fn try_acquire(
        &self,
        key: &str,
        max_concurrent: usize,
    ) -> Option<ConcurrentPermit> {
        // Get or create semaphore
        let semaphore = {
            let mut semaphores = self.semaphores.write().await;

            semaphores
                .entry(key.to_string())
                .or_insert_with(|| ClientSemaphore::new(max_concurrent))
                .clone()
        };

        // Try to acquire permit
        semaphore.try_acquire_owned().map(|permit| ConcurrentPermit { permit })
    }

    /// Get number of available permits for a client
    pub async fn available_permits(&self, key: &str) -> Option<usize> {
        let semaphores = self.semaphores.read().await;
        semaphores.get(key).map(|s| s.available())
    }

    /// Get max concurrent for a client
    pub async fn max_concurrent(&self, key: &str) -> Option<usize> {
        let semaphores = self.semaphores.read().await;
        semaphores.get(key).map(|s| s.max())
    }

    /// Get number of tracked clients
    pub async fn client_count(&self) -> usize {
        self.semaphores.read().await.len()
    }

    /// Clean up unused semaphores (for maintenance)
    pub async fn cleanup(&self) {
        let mut semaphores = self.semaphores.write().await;

        // Remove semaphores that are at full capacity (no active requests)
        semaphores.retain(|_, sem| sem.available() != sem.max());
    }
}

impl Default for ConcurrentLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ConcurrentLimiter {
    fn clone(&self) -> Self {
        Self {
            semaphores: Arc::clone(&self.semaphores),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_limiter_new() {
        let limiter = ConcurrentLimiter::new();
        assert_eq!(limiter.client_count().await, 0);
    }

    #[tokio::test]
    async fn test_try_acquire_first_permit() {
        let limiter = ConcurrentLimiter::new();

        let permit = limiter.try_acquire("user1", 10).await;
        assert!(permit.is_some());

        // Should have 9 available (10 - 1)
        assert_eq!(limiter.available_permits("user1").await, Some(9));
    }

    #[tokio::test]
    async fn test_permit_released_on_drop() {
        let limiter = ConcurrentLimiter::new();

        {
            let _permit = limiter.try_acquire("user1", 10).await;
            assert_eq!(limiter.available_permits("user1").await, Some(9));
        }

        // Permit dropped, should be back to 10
        assert_eq!(limiter.available_permits("user1").await, Some(10));
    }

    #[tokio::test]
    async fn test_concurrent_limit_enforced() {
        let limiter = ConcurrentLimiter::new();

        // Acquire 5 permits (max_concurrent = 5)
        let mut permits = Vec::new();
        for _ in 0..5 {
            let permit = limiter.try_acquire("user1", 5).await;
            assert!(permit.is_some(), "Should acquire permit");
            permits.push(permit);
        }

        // 6th should fail
        let permit = limiter.try_acquire("user1", 5).await;
        assert!(permit.is_none(), "Should not acquire 6th permit");

        // Drop one permit
        permits.pop();

        // Now should succeed
        let permit = limiter.try_acquire("user1", 5).await;
        assert!(permit.is_some(), "Should acquire after release");
    }

    #[tokio::test]
    async fn test_different_clients_separate_limits() {
        let limiter = ConcurrentLimiter::new();

        // User1 exhausts quota
        let mut user1_permits = Vec::new();
        for _ in 0..3 {
            let permit = limiter.try_acquire("user1", 3).await;
            user1_permits.push(permit);
        }

        assert!(limiter.try_acquire("user1", 3).await.is_none());

        // User2 should still have quota
        let user2_permit = limiter.try_acquire("user2", 3).await;
        assert!(user2_permit.is_some());
    }

    #[tokio::test]
    async fn test_available_permits() {
        let limiter = ConcurrentLimiter::new();

        // Initially no client
        assert_eq!(limiter.available_permits("user1").await, None);

        // After first acquire
        let _permit1 = limiter.try_acquire("user1", 5).await;
        assert_eq!(limiter.available_permits("user1").await, Some(4));

        // After second acquire
        let _permit2 = limiter.try_acquire("user1", 5).await;
        assert_eq!(limiter.available_permits("user1").await, Some(3));
    }

    #[tokio::test]
    async fn test_max_concurrent() {
        let limiter = ConcurrentLimiter::new();

        let _permit = limiter.try_acquire("user1", 10).await;
        assert_eq!(limiter.max_concurrent("user1").await, Some(10));

        let _permit = limiter.try_acquire("user2", 50).await;
        assert_eq!(limiter.max_concurrent("user2").await, Some(50));
    }

    #[tokio::test]
    async fn test_concurrent_limiter_clone() {
        let limiter1 = ConcurrentLimiter::new();
        let limiter2 = limiter1.clone();

        // Acquire via limiter1
        let _permit = limiter1.try_acquire("user1", 10).await;

        // Should be visible via limiter2
        assert_eq!(limiter2.available_permits("user1").await, Some(9));
    }

    #[tokio::test]
    async fn test_concurrent_requests_simulation() {
        let limiter = Arc::new(ConcurrentLimiter::new());
        let max_concurrent = 3;

        // Spawn 10 concurrent tasks
        let mut handles = Vec::new();

        for i in 0..10 {
            let limiter_clone = Arc::clone(&limiter);

            let handle = tokio::spawn(async move {
                if let Some(_permit) = limiter_clone.try_acquire("user1", max_concurrent).await {
                    // Simulate work
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    Ok::<_, ()>(i)
                } else {
                    Err(())
                }
            });

            handles.push(handle);
        }

        // Wait for all tasks
        let results = futures::future::join_all(handles).await;

        // Count successes and failures
        let successes = results.iter().filter(|r| {
            if let Ok(Ok(_)) = r {
                true
            } else {
                false
            }
        }).count();

        let failures = results.iter().filter(|r| {
            if let Ok(Err(_)) = r {
                true
            } else {
                false
            }
        }).count();

        // Some should succeed, some should fail
        assert!(successes > 0, "Some requests should succeed");
        assert!(failures > 0, "Some requests should fail due to concurrency limit");
        assert_eq!(successes + failures, 10);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = ConcurrentLimiter::new();

        {
            let _permit = limiter.try_acquire("user1", 5).await;
            let _permit = limiter.try_acquire("user2", 5).await;
        }

        // Permits released
        assert_eq!(limiter.client_count().await, 2);

        // Cleanup should remove entries at full capacity
        limiter.cleanup().await;

        // All permits released, so entries removed
        assert_eq!(limiter.client_count().await, 0);
    }
}
