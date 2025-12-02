use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Simple TTL cache for expensive operations
pub struct TtlCache<K, V> {
    data: Mutex<HashMap<K, CacheEntry<V>>>,
    ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> TtlCache<K, V> {
    /// Create a new TTL cache with the specified time-to-live
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
            ttl,
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

    /// Insert a value into the cache
    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut data) = self.data.lock() {
            data.insert(
                key,
                CacheEntry {
                    value,
                    expires_at: Instant::now() + self.ttl,
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
        let cache = TtlCache::new(Duration::from_secs(60));

        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = TtlCache::new(Duration::from_millis(100));

        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = TtlCache::new(Duration::from_millis(100));

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        assert_eq!(cache.len(), 2);

        thread::sleep(Duration::from_millis(150));
        cache.cleanup();
        assert_eq!(cache.len(), 0);
    }
}
