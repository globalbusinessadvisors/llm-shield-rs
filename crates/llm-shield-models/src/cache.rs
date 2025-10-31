//! Result caching with LRU eviction and TTL
//!
//! ## Design Philosophy
//!
//! This cache implementation follows enterprise-grade patterns:
//! - **Thread-Safe**: Uses Arc + RwLock for concurrent access
//! - **LRU Eviction**: Least Recently Used items are evicted first
//! - **TTL Support**: Entries expire after configured time-to-live
//! - **Statistics**: Tracks hits, misses, and hit rates
//! - **Lazy Cleanup**: Expired items cleaned on access (no background threads)
//!
//! ## Usage Example
//!
//! ```rust
//! use llm_shield_models::cache::{ResultCache, CacheConfig};
//! use llm_shield_core::ScanResult;
//! use std::time::Duration;
//!
//! let cache = ResultCache::new(CacheConfig {
//!     max_size: 1000,
//!     ttl: Duration::from_secs(300),
//! });
//!
//! // Insert a result
//! let result = ScanResult::pass("safe text".to_string());
//! cache.insert("key1".to_string(), result);
//!
//! // Retrieve it
//! if let Some(cached_result) = cache.get("key1") {
//!     println!("Cache hit!");
//! }
//!
//! // Check statistics
//! let stats = cache.stats();
//! println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
//! ```

use llm_shield_core::ScanResult;
use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Configuration for the result cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_size: usize,
    /// Time-to-live for cache entries
    pub ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Thread-safe result cache with LRU eviction and TTL
///
/// ## Performance Characteristics
///
/// - **Get**: O(1) average, O(n) worst case for access order update
/// - **Insert**: O(1) average, O(n) worst case for eviction
/// - **Memory**: O(max_size * entry_size)
///
/// ## Thread Safety
///
/// Uses `Arc<RwLock<_>>` for interior mutability:
/// - Multiple concurrent readers
/// - Exclusive writer access
/// - Clone creates a new reference to same cache
pub struct ResultCache {
    inner: Arc<RwLock<CacheInner>>,
}

/// Internal cache state
struct CacheInner {
    config: CacheConfig,
    entries: HashMap<String, CacheEntry>,
    access_order: Vec<String>, // LRU tracking (oldest first, newest last)
    stats: CacheStats,
}

/// A single cache entry with metadata
struct CacheEntry {
    result: ScanResult,
    inserted_at: Instant,
}

/// Cache performance statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
}

impl CacheStats {
    /// Total number of cache requests
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }

    /// Hit rate as a value between 0.0 and 1.0
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl ResultCache {
    /// Create a new result cache with the given configuration
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::cache::{ResultCache, CacheConfig};
    /// use std::time::Duration;
    ///
    /// let cache = ResultCache::new(CacheConfig {
    ///     max_size: 1000,
    ///     ttl: Duration::from_secs(300),
    /// });
    /// ```
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner {
                config,
                entries: HashMap::new(),
                access_order: Vec::new(),
                stats: CacheStats::default(),
            })),
        }
    }

    /// Get a cached result by key
    ///
    /// Returns `None` if:
    /// - Key doesn't exist
    /// - Entry has expired (and removes it)
    ///
    /// Updates LRU access order on cache hit.
    pub fn get(&self, key: &str) -> Option<ScanResult> {
        let mut inner = self.inner.write().unwrap();

        // Check if key exists
        if let Some(entry) = inner.entries.get(key) {
            // Check if expired
            if entry.inserted_at.elapsed() < inner.config.ttl {
                // Clone the result before updating access order
                let result = entry.result.clone();

                // Cache hit - update stats and access order
                inner.stats.hits += 1;

                // Update LRU: move to end (most recently used)
                inner.access_order.retain(|k| k != key);
                inner.access_order.push(key.to_string());

                return Some(result);
            } else {
                // Expired - remove it (lazy cleanup)
                inner.entries.remove(key);
                inner.access_order.retain(|k| k != key);
            }
        }

        // Cache miss
        inner.stats.misses += 1;
        None
    }

    /// Insert or update a cache entry
    ///
    /// If the cache is at capacity, evicts the least recently used entry.
    /// If the key already exists, updates it and refreshes the TTL.
    pub fn insert(&self, key: String, result: ScanResult) {
        let mut inner = self.inner.write().unwrap();

        // Handle zero capacity edge case
        if inner.config.max_size == 0 {
            return;
        }

        // If key already exists, remove it from access order
        if inner.entries.contains_key(&key) {
            inner.access_order.retain(|k| k != &key);
        } else if inner.entries.len() >= inner.config.max_size {
            // At capacity and new key - evict oldest
            if let Some(oldest_key) = inner.access_order.first().cloned() {
                inner.entries.remove(&oldest_key);
                inner.access_order.remove(0);
            }
        }

        // Insert new entry
        inner.entries.insert(
            key.clone(),
            CacheEntry {
                result,
                inserted_at: Instant::now(),
            },
        );

        // Add to end of access order (most recently used)
        inner.access_order.push(key);
    }

    /// Clear all entries from the cache
    ///
    /// This does not reset statistics.
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.entries.clear();
        inner.access_order.clear();
    }

    /// Get the number of entries in the cache
    ///
    /// Note: This includes expired entries that haven't been lazily cleaned yet.
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.inner.read().unwrap().stats.clone()
    }

    /// Reset cache statistics
    ///
    /// This does not affect cached entries.
    pub fn reset_stats(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.stats = CacheStats::default();
    }

    /// Generate a deterministic hash key from input text
    ///
    /// Useful for caching scan results based on input content.
    ///
    /// # Example
    ///
    /// ```
    /// use llm_shield_models::cache::ResultCache;
    ///
    /// let input = "some text to scan";
    /// let key = ResultCache::hash_key(input);
    /// ```
    pub fn hash_key(input: &str) -> String {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Clone for ResultCache {
    /// Clone creates a new reference to the same underlying cache
    ///
    /// All clones share the same cache data and statistics.
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_result(text: &str) -> ScanResult {
        ScanResult::pass(text.to_string())
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_size, 10_000);
        assert_eq!(config.ttl, Duration::from_secs(300));
    }

    #[test]
    fn test_cache_stats_empty() {
        let stats = CacheStats::default();
        assert_eq!(stats.total_requests(), 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_calculation() {
        let stats = CacheStats {
            hits: 7,
            misses: 3,
        };
        assert_eq!(stats.total_requests(), 10);
        assert!((stats.hit_rate() - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_basic_insert_get() {
        let cache = ResultCache::new(CacheConfig {
            max_size: 10,
            ttl: Duration::from_secs(60),
        });

        let result = create_test_result("test");
        cache.insert("key1".to_string(), result.clone());

        assert_eq!(cache.get("key1"), Some(result));
    }

    #[test]
    fn test_cache_miss() {
        let cache = ResultCache::new(CacheConfig {
            max_size: 10,
            ttl: Duration::from_secs(60),
        });

        assert_eq!(cache.get("nonexistent"), None);
    }

    #[test]
    fn test_is_empty() {
        let cache = ResultCache::new(CacheConfig::default());
        assert!(cache.is_empty());

        cache.insert("key".to_string(), create_test_result("test"));
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_hash_key_deterministic() {
        let key1 = ResultCache::hash_key("test input");
        let key2 = ResultCache::hash_key("test input");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_hash_key_different_inputs() {
        let key1 = ResultCache::hash_key("input1");
        let key2 = ResultCache::hash_key("input2");
        assert_ne!(key1, key2);
    }
}
