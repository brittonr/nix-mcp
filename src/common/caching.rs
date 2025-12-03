use crate::common::cache::TtlCache;
use rmcp::model::{CallToolResult, Content};
use rmcp::ErrorData as McpError;
use std::future::Future;
use std::sync::Arc;

/// Helper for executing operations with caching
pub struct CachedExecutor {
    cache: Arc<TtlCache<String, String>>,
}

impl CachedExecutor {
    pub fn new(cache: Arc<TtlCache<String, String>>) -> Self {
        Self { cache }
    }

    /// Execute with cache-check-execute-cache pattern for string results
    ///
    /// This is the most common pattern:
    /// 1. Check if result is in cache
    /// 2. If found, return cached result
    /// 3. If not found, execute the future to get a string
    /// 4. Cache the string result
    /// 5. Return as CallToolResult
    pub async fn execute_with_string_cache<F, Fut>(
        &self,
        cache_key: String,
        executor: F,
    ) -> Result<CallToolResult, McpError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String, McpError>>,
    {
        // Check cache first
        if let Some(cached_result) = self.cache.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute the operation to get string result
        let result_string = executor().await?;

        // Cache the result
        self.cache.insert(cache_key, result_string.clone());

        Ok(CallToolResult::success(vec![Content::text(result_string)]))
    }

    /// Execute with cache-check-execute-cache pattern for CallToolResult
    ///
    /// Note: This version doesn't cache the result automatically since
    /// CallToolResult might contain non-text content. Use execute_with_string_cache
    /// for automatic caching, or cache manually after execution.
    pub async fn execute_with_cache<F, Fut>(
        &self,
        cache_key: String,
        executor: F,
    ) -> Result<CallToolResult, McpError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<CallToolResult, McpError>>,
    {
        // Check cache first
        if let Some(cached_result) = self.cache.get(&cache_key) {
            return Ok(CallToolResult::success(vec![Content::text(cached_result)]));
        }

        // Execute the operation
        executor().await
    }

    /// Execute with custom cache key formatter (for string results)
    ///
    /// Useful when cache key needs to be computed from multiple parameters
    pub async fn execute_with_formatted_cache<F, Fut, K>(
        &self,
        key_parts: K,
        executor: F,
    ) -> Result<CallToolResult, McpError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String, McpError>>,
        K: IntoIterator,
        K::Item: std::fmt::Display,
    {
        let cache_key = key_parts
            .into_iter()
            .map(|part| part.to_string())
            .collect::<Vec<_>>()
            .join(":");

        self.execute_with_string_cache(cache_key, executor).await
    }

    /// Get a value from cache without execution
    pub fn get(&self, key: &str) -> Option<String> {
        self.cache.get(&key.to_string())
    }

    /// Insert a value into cache
    pub fn insert(&self, key: String, value: String) {
        self.cache.insert(key, value);
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = Arc::new(TtlCache::new(Duration::from_secs(60)));
        let executor = CachedExecutor::new(cache.clone());

        // Pre-populate cache
        cache.insert("key1".to_string(), "cached_value".to_string());

        // Execute - should return cached value
        let result = executor
            .execute_with_string_cache("key1".to_string(), || async {
                // This should not be called
                panic!("Should not execute when cached");
            })
            .await
            .unwrap();

        // Verify result content exists
        assert!(!result.content.is_empty());
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = Arc::new(TtlCache::new(Duration::from_secs(60)));
        let executor = CachedExecutor::new(cache.clone());

        // Execute - cache miss, should execute and cache
        let result = executor
            .execute_with_string_cache("key1".to_string(), || async {
                Ok("fresh_value".to_string())
            })
            .await
            .unwrap();

        // Verify result content exists
        assert!(!result.content.is_empty());

        // Verify it was cached
        assert_eq!(
            cache.get(&"key1".to_string()),
            Some("fresh_value".to_string())
        );
    }

    #[tokio::test]
    async fn test_formatted_cache_key() {
        let cache = Arc::new(TtlCache::new(Duration::from_secs(60)));
        let executor = CachedExecutor::new(cache.clone());

        // Execute with formatted key
        let result = executor
            .execute_with_formatted_cache(vec!["part1", "part2", "part3"], || async {
                Ok("value".to_string())
            })
            .await
            .unwrap();

        // Verify result content exists
        assert!(!result.content.is_empty());

        // Verify cached with correct key
        assert_eq!(
            cache.get(&"part1:part2:part3".to_string()),
            Some("value".to_string())
        );
    }
}
