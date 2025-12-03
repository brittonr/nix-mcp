use crate::common::cache::TtlCache;
use std::sync::Arc;
use std::time::Duration;

/// Centralized cache registry for all MCP tool caches.
///
/// This struct provides a single point of configuration for all caching
/// throughout the application. Each cache has specific TTL and capacity
/// limits tuned for its use case.
///
/// # Cache Lifetimes
///
/// - `locate`: 5 minutes - File location queries change as packages update
/// - `search`: 10 minutes - Package search results change moderately
/// - `package_info`: 30 minutes - Package metadata is relatively stable
/// - `eval`: 5 minutes - Nix evaluations can change frequently
/// - `prefetch`: 24 hours - URL content hashes are immutable
/// - `closure_size`: 30 minutes - Closure sizes are stable for given derivations
/// - `derivation`: 30 minutes - Derivation info is immutable for a given hash
///
/// # Example
///
/// ```no_run
/// use onix_mcp::common::cache_registry::CacheRegistry;
/// use std::sync::Arc;
///
/// let caches = Arc::new(CacheRegistry::new());
///
/// // Use in tools
/// let package_tools = PackageTools::new(audit, caches.clone());
/// let build_tools = BuildTools::new(audit, caches.clone());
/// ```
#[derive(Clone)]
pub struct CacheRegistry {
    /// Cache for nix-locate file location queries (TTL: 5 minutes)
    pub locate: Arc<TtlCache<String, String>>,

    /// Cache for package search results (TTL: 10 minutes)
    pub search: Arc<TtlCache<String, String>>,

    /// Cache for package metadata (TTL: 30 minutes)
    pub package_info: Arc<TtlCache<String, String>>,

    /// Cache for nix eval expression results (TTL: 5 minutes)
    pub eval: Arc<TtlCache<String, String>>,

    /// Cache for URL prefetch hashes (TTL: 24 hours)
    pub prefetch: Arc<TtlCache<String, String>>,

    /// Cache for closure size calculations (TTL: 30 minutes)
    pub closure_size: Arc<TtlCache<String, String>>,

    /// Cache for derivation info (TTL: 30 minutes)
    pub derivation: Arc<TtlCache<String, String>>,
}

impl CacheRegistry {
    /// Create a new cache registry with default TTL values.
    ///
    /// All caches are created with appropriate TTLs based on the volatility
    /// of their cached data. More frequently changing data has shorter TTLs.
    pub fn new() -> Self {
        Self {
            locate: Arc::new(TtlCache::new(Duration::from_secs(300))), // 5 min
            search: Arc::new(TtlCache::new(Duration::from_secs(600))), // 10 min
            package_info: Arc::new(TtlCache::new(Duration::from_secs(1800))), // 30 min
            eval: Arc::new(TtlCache::new(Duration::from_secs(300))),   // 5 min
            prefetch: Arc::new(TtlCache::new(Duration::from_secs(86400))), // 24 hours
            closure_size: Arc::new(TtlCache::new(Duration::from_secs(1800))), // 30 min
            derivation: Arc::new(TtlCache::new(Duration::from_secs(1800))), // 30 min
        }
    }
}

impl Default for CacheRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_registry_creation() {
        let registry = CacheRegistry::new();

        // Verify all caches are initialized
        assert!(Arc::strong_count(&registry.locate) >= 1);
        assert!(Arc::strong_count(&registry.search) >= 1);
        assert!(Arc::strong_count(&registry.package_info) >= 1);
        assert!(Arc::strong_count(&registry.eval) >= 1);
        assert!(Arc::strong_count(&registry.prefetch) >= 1);
        assert!(Arc::strong_count(&registry.closure_size) >= 1);
        assert!(Arc::strong_count(&registry.derivation) >= 1);
    }

    #[test]
    fn test_cache_registry_clone() {
        let registry = CacheRegistry::new();
        let cloned = registry.clone();

        // Verify clones share the same cache instances
        assert_eq!(Arc::as_ptr(&registry.locate), Arc::as_ptr(&cloned.locate));
        assert_eq!(Arc::as_ptr(&registry.search), Arc::as_ptr(&cloned.search));
    }

    #[test]
    fn test_cache_registry_default() {
        let registry = CacheRegistry::default();

        // Verify default construction works
        assert!(Arc::strong_count(&registry.locate) >= 1);
    }
}
