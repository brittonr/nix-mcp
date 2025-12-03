use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// TTL cache with capacity limits for expensive operations.
///
/// This cache combines time-based expiration (TTL) with capacity limits
/// to prevent unbounded memory growth. When the cache reaches its maximum
/// capacity, the oldest entry (by insertion time) is evicted.
///
/// # Features
///
/// - **Time-based expiration**: Entries automatically expire after TTL
/// - **Capacity limits**: Maximum number of entries enforced
/// - **LRU-like eviction**: Oldest entries removed when at capacity
/// - **Thread-safe**: Uses Mutex for concurrent access
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
/// use onix_mcp::common::cache::TtlCache;
///
/// let cache = TtlCache::new(Duration::from_secs(60), 1000);
/// cache.insert("key".to_string(), "value".to_string());
///
/// if let Some(value) = cache.get(&"key".to_string()) {
///     println!("Cached value: {}", value);
/// }
/// ```
pub struct TtlCache<K, V> {
    data: Mutex<HashMap<K, CacheEntry<V>>>,
    ttl: Duration,
    max_capacity: usize,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
    inserted_at: Instant,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> TtlCache<K, V> {
    /// Create a new TTL cache with the specified time-to-live and maximum capacity.
    ///
    /// # Arguments
    ///
    /// * `ttl` - Time-to-live for cache entries
    /// * `max_capacity` - Maximum number of entries (0 = unlimited, not recommended)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use onix_mcp::common::cache::TtlCache;
    ///
    /// // Cache with 10-minute TTL and max 1000 entries
    /// let cache = TtlCache::new(Duration::from_secs(600), 1000);
    /// ```
    pub fn new(ttl: Duration, max_capacity: usize) -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
            ttl,
            max_capacity,
        }
    }

    /// Get a value from the cache if it exists and hasn't expired
    pub fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.lock().ok()?;

        if let Some(entry) = data.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                data.remove(key);
            }
        }

        None
    }

    /// Insert a value into the cache.
    ///
    /// If the cache is at maximum capacity, the oldest entry (by insertion time)
    /// will be evicted before inserting the new entry.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert
    /// * `value` - The value to cache
    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut data) = self.data.lock() {
            let now = Instant::now();

            // Evict oldest entry if at capacity (and max_capacity > 0)
            if self.max_capacity > 0 && data.len() >= self.max_capacity && !data.contains_key(&key)
            {
                // Find and remove the oldest entry by insertion time
                if let Some(oldest_key) = data
                    .iter()
                    .min_by_key(|(_, entry)| entry.inserted_at)
                    .map(|(k, _)| k.clone())
                {
                    data.remove(&oldest_key);
                }
            }

            data.insert(
                key,
                CacheEntry {
                    value,
                    expires_at: now + self.ttl,
                    inserted_at: now,
                },
            );
        }
    }

    /// Clear all entries from the cache
    #[allow(dead_code)]
    pub fn clear(&self) {
        if let Ok(mut data) = self.data.lock() {
            data.clear();
        }
    }

    /// Remove expired entries
    #[allow(dead_code)]
    pub fn cleanup(&self) {
        if let Ok(mut data) = self.data.lock() {
            let now = Instant::now();
            data.retain(|_, entry| now < entry.expires_at);
        }
    }

    /// Get the number of entries in the cache (including expired)
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.data.lock().map(|d| d.len()).unwrap_or(0)
    }

    /// Check if the cache is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic() {
        let cache = TtlCache::new(Duration::from_secs(60), 1000);

        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = TtlCache::new(Duration::from_millis(100), 1000);

        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = TtlCache::new(Duration::from_millis(100), 1000);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        assert_eq!(cache.len(), 2);

        thread::sleep(Duration::from_millis(150));
        cache.cleanup();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_capacity_limit() {
        // Create cache with max capacity of 3
        let cache = TtlCache::new(Duration::from_secs(60), 3);

        // Insert 3 entries - should all fit
        cache.insert("key1".to_string(), "value1".to_string());
        thread::sleep(Duration::from_millis(10)); // Ensure different insertion times
        cache.insert("key2".to_string(), "value2".to_string());
        thread::sleep(Duration::from_millis(10));
        cache.insert("key3".to_string(), "value3".to_string());
        assert_eq!(cache.len(), 3);

        // Insert 4th entry - should evict oldest (key1)
        thread::sleep(Duration::from_millis(10));
        cache.insert("key4".to_string(), "value4".to_string());
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&"key1".to_string()), None); // Oldest evicted
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));
        assert_eq!(cache.get(&"key3".to_string()), Some("value3".to_string()));
        assert_eq!(cache.get(&"key4".to_string()), Some("value4".to_string()));
    }

    #[test]
    fn test_cache_unlimited_capacity() {
        // Cache with 0 capacity = unlimited
        let cache = TtlCache::new(Duration::from_secs(60), 0);

        // Should be able to insert many entries
        for i in 0..100 {
            cache.insert(format!("key{}", i), format!("value{}", i));
        }
        assert_eq!(cache.len(), 100);
    }

    #[test]
    fn test_cache_update_existing_key() {
        let cache = TtlCache::new(Duration::from_secs(60), 3);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());

        // Update existing key - should not evict anything
        cache.insert("key2".to_string(), "value2_updated".to_string());
        assert_eq!(cache.len(), 3);
        assert_eq!(
            cache.get(&"key2".to_string()),
            Some("value2_updated".to_string())
        );
    }
}
