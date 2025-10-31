//! ResultCache tests following London School TDD
//!
//! These tests verify:
//! - Cache insert and retrieval
//! - LRU eviction policy
//! - TTL expiration
//! - Thread safety
//! - Cache statistics and hit rates

use llm_shield_core::ScanResult;
use llm_shield_models::cache::{CacheConfig, ResultCache};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Helper to create a test ScanResult
fn create_test_result(text: &str, risk_score: f32) -> ScanResult {
    ScanResult::new(text.to_string(), risk_score < 0.5, risk_score)
}

#[test]
fn test_cache_insert_and_retrieve() {
    // Given: A cache with capacity 3
    let cache = ResultCache::new(CacheConfig {
        max_size: 3,
        ttl: Duration::from_secs(60),
    });

    // When: We insert a result
    let result = create_test_result("safe text", 0.0);
    cache.insert("key1".to_string(), result.clone());

    // Then: We can retrieve it
    let retrieved = cache.get("key1");
    assert!(retrieved.is_some(), "Should retrieve inserted value");
    assert_eq!(retrieved.unwrap(), result);
}

#[test]
fn test_cache_miss() {
    // Given: An empty cache
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_secs(60),
    });

    // When: We try to get a non-existent key
    let result = cache.get("nonexistent");

    // Then: It returns None
    assert!(result.is_none(), "Should return None for cache miss");
}

#[test]
fn test_cache_lru_eviction() {
    // Given: A cache with capacity 2
    let cache = ResultCache::new(CacheConfig {
        max_size: 2,
        ttl: Duration::from_secs(60),
    });

    // When: We insert 3 items
    let result1 = create_test_result("text1", 0.1);
    let result2 = create_test_result("text2", 0.2);
    let result3 = create_test_result("text3", 0.3);

    cache.insert("key1".to_string(), result1);
    cache.insert("key2".to_string(), result2.clone());
    cache.insert("key3".to_string(), result3.clone()); // This should evict key1

    // Then: Oldest item is evicted
    assert!(cache.get("key1").is_none(), "key1 should be evicted");
    assert_eq!(
        cache.get("key2").unwrap(),
        result2,
        "key2 should still exist"
    );
    assert_eq!(
        cache.get("key3").unwrap(),
        result3,
        "key3 should still exist"
    );
}

#[test]
fn test_cache_lru_eviction_updates_on_access() {
    // Given: A cache with capacity 2
    let cache = ResultCache::new(CacheConfig {
        max_size: 2,
        ttl: Duration::from_secs(60),
    });

    let result1 = create_test_result("text1", 0.1);
    let result2 = create_test_result("text2", 0.2);
    let result3 = create_test_result("text3", 0.3);

    // When: We insert 2 items, access key1, then insert a 3rd
    cache.insert("key1".to_string(), result1.clone());
    cache.insert("key2".to_string(), result2);

    // Access key1 to make it more recently used
    let _ = cache.get("key1");

    // Insert key3, should evict key2 (now least recently used)
    cache.insert("key3".to_string(), result3.clone());

    // Then: key2 is evicted, but key1 remains
    assert_eq!(
        cache.get("key1").unwrap(),
        result1,
        "key1 should still exist (was accessed)"
    );
    assert!(cache.get("key2").is_none(), "key2 should be evicted");
    assert_eq!(
        cache.get("key3").unwrap(),
        result3,
        "key3 should still exist"
    );
}

#[test]
fn test_cache_ttl_expiration() {
    // Given: A cache with short TTL
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_millis(100),
    });

    // When: We insert an item and wait for expiration
    let result = create_test_result("text", 0.0);
    cache.insert("key1".to_string(), result.clone());

    // Verify it exists before expiration
    assert!(cache.get("key1").is_some(), "Should exist before expiration");

    // Wait for TTL to expire
    thread::sleep(Duration::from_millis(150));

    // Then: Item is expired
    assert!(cache.get("key1").is_none(), "Should be expired after TTL");
}

#[test]
fn test_cache_ttl_refresh_on_update() {
    // Given: A cache with short TTL
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_millis(200),
    });

    let result1 = create_test_result("text1", 0.0);
    let result2 = create_test_result("text2", 0.0);

    // When: We insert, wait, then update same key
    cache.insert("key1".to_string(), result1);
    thread::sleep(Duration::from_millis(100));

    // Update the same key (should refresh TTL)
    cache.insert("key1".to_string(), result2.clone());

    // Wait another 100ms (total 200ms from first insert, but only 100ms from update)
    thread::sleep(Duration::from_millis(100));

    // Then: Item still exists because TTL was refreshed
    assert_eq!(
        cache.get("key1").unwrap(),
        result2,
        "Should exist after update"
    );
}

#[test]
fn test_cache_clear() {
    // Given: A cache with items
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_secs(60),
    });

    cache.insert("key1".to_string(), create_test_result("text1", 0.0));
    cache.insert("key2".to_string(), create_test_result("text2", 0.0));

    assert_eq!(cache.len(), 2, "Cache should have 2 items");

    // When: We clear the cache
    cache.clear();

    // Then: Cache is empty
    assert_eq!(cache.len(), 0, "Cache should be empty");
    assert!(cache.get("key1").is_none(), "key1 should be gone");
    assert!(cache.get("key2").is_none(), "key2 should be gone");
}

#[test]
fn test_cache_len() {
    // Given: A cache
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_secs(60),
    });

    // When: We insert items
    assert_eq!(cache.len(), 0, "Empty cache should have len 0");

    cache.insert("key1".to_string(), create_test_result("text1", 0.0));
    assert_eq!(cache.len(), 1, "Cache should have len 1");

    cache.insert("key2".to_string(), create_test_result("text2", 0.0));
    assert_eq!(cache.len(), 2, "Cache should have len 2");

    // Update existing key shouldn't change length
    cache.insert("key1".to_string(), create_test_result("text3", 0.0));
    assert_eq!(cache.len(), 2, "Cache should still have len 2 after update");
}

#[test]
fn test_cache_thread_safety_concurrent_reads() {
    // Given: A shared cache with data
    let cache = Arc::new(ResultCache::new(CacheConfig {
        max_size: 100,
        ttl: Duration::from_secs(60),
    }));

    let result = create_test_result("shared text", 0.5);
    cache.insert("shared_key".to_string(), result.clone());

    // When: Multiple threads read concurrently
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let cache_clone = Arc::clone(&cache);
            thread::spawn(move || {
                for _ in 0..100 {
                    let retrieved = cache_clone.get("shared_key");
                    assert!(retrieved.is_some(), "Should always find the key");
                }
            })
        })
        .collect();

    // Then: All threads complete successfully
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}

#[test]
fn test_cache_thread_safety_concurrent_writes() {
    // Given: A shared cache
    let cache = Arc::new(ResultCache::new(CacheConfig {
        max_size: 1000,
        ttl: Duration::from_secs(60),
    }));

    // When: Multiple threads write concurrently
    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let cache_clone = Arc::clone(&cache);
            thread::spawn(move || {
                for i in 0..50 {
                    let key = format!("thread{}_key{}", thread_id, i);
                    let result = create_test_result(&key, 0.1);
                    cache_clone.insert(key, result);
                }
            })
        })
        .collect();

    // Then: All threads complete successfully
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // Verify some data was written
    assert!(cache.len() > 0, "Cache should contain items");
}

#[test]
fn test_cache_thread_safety_mixed_operations() {
    // Given: A shared cache with initial data
    let cache = Arc::new(ResultCache::new(CacheConfig {
        max_size: 500,
        ttl: Duration::from_secs(60),
    }));

    // Pre-populate with some data
    for i in 0..50 {
        cache.insert(format!("key{}", i), create_test_result(&format!("text{}", i), 0.0));
    }

    // When: Threads perform mixed read/write operations
    let handles: Vec<_> = (0..8)
        .map(|thread_id| {
            let cache_clone = Arc::clone(&cache);
            thread::spawn(move || {
                for i in 0..50 {
                    if thread_id % 2 == 0 {
                        // Even threads: write
                        let key = format!("key{}", i);
                        cache_clone.insert(key, create_test_result("updated", 0.2));
                    } else {
                        // Odd threads: read
                        let key = format!("key{}", i);
                        let _ = cache_clone.get(&key);
                    }
                }
            })
        })
        .collect();

    // Then: All operations complete without panics
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}

#[test]
fn test_cache_statistics() {
    // Given: A cache with statistics tracking
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_secs(60),
    });

    cache.insert("key1".to_string(), create_test_result("text1", 0.0));
    cache.insert("key2".to_string(), create_test_result("text2", 0.0));

    // When: We perform hits and misses
    let _ = cache.get("key1"); // hit
    let _ = cache.get("key1"); // hit
    let _ = cache.get("key3"); // miss
    let _ = cache.get("key2"); // hit
    let _ = cache.get("key4"); // miss

    // Then: Statistics are accurate
    let stats = cache.stats();
    assert_eq!(stats.hits, 3, "Should have 3 hits");
    assert_eq!(stats.misses, 2, "Should have 2 misses");
    assert_eq!(stats.total_requests(), 5, "Should have 5 total requests");
    assert!((stats.hit_rate() - 0.6).abs() < 0.01, "Hit rate should be 60%");
}

#[test]
fn test_cache_statistics_reset() {
    // Given: A cache with statistics
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_secs(60),
    });

    cache.insert("key1".to_string(), create_test_result("text1", 0.0));
    let _ = cache.get("key1");
    let _ = cache.get("key2");

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);

    // When: We reset statistics
    cache.reset_stats();

    // Then: Statistics are cleared
    let stats = cache.stats();
    assert_eq!(stats.hits, 0, "Hits should be reset");
    assert_eq!(stats.misses, 0, "Misses should be reset");
}

#[test]
fn test_cache_clone() {
    // Given: A cache with data
    let cache1 = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_secs(60),
    });

    let result = create_test_result("text1", 0.0);
    cache1.insert("key1".to_string(), result.clone());

    // When: We clone the cache
    let cache2 = cache1.clone();

    // Then: Both caches share the same data
    assert_eq!(cache2.get("key1").unwrap(), result);

    // And: Updates to one affect the other
    cache2.insert("key2".to_string(), create_test_result("text2", 0.0));
    assert!(cache1.get("key2").is_some(), "Original cache should see new key");
}

#[test]
fn test_cache_hash_key_generation() {
    // Given: A cache
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_secs(60),
    });

    let input = "test input text";

    // When: We generate a hash key
    let key1 = ResultCache::hash_key(input);
    let key2 = ResultCache::hash_key(input);

    // Then: Same input produces same hash
    assert_eq!(key1, key2, "Hash should be deterministic");

    // And: Different inputs produce different hashes
    let key3 = ResultCache::hash_key("different text");
    assert_ne!(key1, key3, "Different inputs should produce different hashes");

    // And: We can use the hash as a cache key
    cache.insert(key1.clone(), create_test_result(input, 0.0));
    assert!(cache.get(&key1).is_some(), "Should retrieve by hash key");
}

#[test]
fn test_cache_expired_items_cleaned_up_lazily() {
    // Given: A cache with short TTL
    let cache = ResultCache::new(CacheConfig {
        max_size: 10,
        ttl: Duration::from_millis(50),
    });

    // When: We insert items that will expire
    cache.insert("key1".to_string(), create_test_result("text1", 0.0));
    cache.insert("key2".to_string(), create_test_result("text2", 0.0));

    assert_eq!(cache.len(), 2, "Should have 2 items");

    // Wait for expiration
    thread::sleep(Duration::from_millis(100));

    // Then: Length still shows 2 (lazy cleanup)
    assert_eq!(cache.len(), 2, "Expired items not yet cleaned");

    // But: Accessing expired items returns None and cleans them up
    assert!(cache.get("key1").is_none(), "key1 should be expired");
    assert!(cache.get("key2").is_none(), "key2 should be expired");

    // After accessing, they should be removed
    // Note: len() counts physical entries, not valid ones
}

#[test]
fn test_cache_zero_capacity() {
    // Given: A cache with zero capacity
    let cache = ResultCache::new(CacheConfig {
        max_size: 0,
        ttl: Duration::from_secs(60),
    });

    // When: We try to insert
    cache.insert("key1".to_string(), create_test_result("text1", 0.0));

    // Then: Nothing is stored
    assert_eq!(cache.len(), 0, "Zero capacity cache should store nothing");
    assert!(cache.get("key1").is_none(), "Should not retrieve from zero capacity cache");
}
