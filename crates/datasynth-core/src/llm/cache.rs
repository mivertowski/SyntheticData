use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use sha2::{Digest, Sha256};

/// In-memory LRU cache for LLM responses.
pub struct LlmCache {
    entries: Arc<RwLock<HashMap<u64, CacheEntry>>>,
    max_entries: usize,
}

#[derive(Clone)]
struct CacheEntry {
    content: String,
    access_count: u64,
}

impl LlmCache {
    /// Create a new cache with maximum entry count.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_entries,
        }
    }

    /// Compute cache key from prompt + system + seed.
    pub fn cache_key(prompt: &str, system: Option<&str>, seed: Option<u64>) -> u64 {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        if let Some(sys) = system {
            hasher.update(sys.as_bytes());
        }
        if let Some(s) = seed {
            hasher.update(s.to_le_bytes());
        }
        let hash = hasher.finalize();
        u64::from_le_bytes(hash[..8].try_into().unwrap_or([0u8; 8]))
    }

    /// Get a cached response.
    pub fn get(&self, key: u64) -> Option<String> {
        let mut entries = self.entries.write().ok()?;
        if let Some(entry) = entries.get_mut(&key) {
            entry.access_count += 1;
            Some(entry.content.clone())
        } else {
            None
        }
    }

    /// Insert a response into the cache.
    pub fn insert(&self, key: u64, content: String) {
        if let Ok(mut entries) = self.entries.write() {
            // Evict least-accessed entry if at capacity
            if entries.len() >= self.max_entries {
                if let Some((&evict_key, _)) = entries.iter().min_by_key(|(_, v)| v.access_count) {
                    entries.remove(&evict_key);
                }
            }
            entries.insert(
                key,
                CacheEntry {
                    content,
                    access_count: 1,
                },
            );
        }
    }

    /// Number of entries in cache.
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all entries.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_and_get() {
        let cache = LlmCache::new(100);
        let key = LlmCache::cache_key("test", None, Some(42));
        cache.insert(key, "response".to_string());
        assert_eq!(cache.get(key), Some("response".to_string()));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let cache = LlmCache::new(100);
        assert_eq!(cache.get(12345), None);
    }

    #[test]
    fn test_cache_eviction() {
        let cache = LlmCache::new(2);
        cache.insert(1, "a".to_string());
        cache.insert(2, "b".to_string());
        cache.insert(3, "c".to_string()); // Should evict one entry
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_cache_key_deterministic() {
        let k1 = LlmCache::cache_key("prompt", Some("system"), Some(42));
        let k2 = LlmCache::cache_key("prompt", Some("system"), Some(42));
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_differs() {
        let k1 = LlmCache::cache_key("prompt1", None, None);
        let k2 = LlmCache::cache_key("prompt2", None, None);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_clear() {
        let cache = LlmCache::new(100);
        cache.insert(1, "a".to_string());
        cache.insert(2, "b".to_string());
        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }
}
