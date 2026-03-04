use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl: Arc<RwLock<Duration>>,
    endpoint_ttls: Arc<RwLock<HashMap<String, u64>>>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    cached_at: Instant,
    ttl: Duration,
}

impl ResponseCache {
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Arc::new(RwLock::new(Duration::from_secs(default_ttl_secs))),
            endpoint_ttls: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let cache = self.cache.read().await;
        let entry = cache.get(key)?;
        
        if entry.is_expired() {
            return None;
        }
        
        serde_json::from_slice(&entry.data).ok()
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_secs: Option<u64>) {
        let ttl = self.get_ttl_for_endpoint(key, ttl_secs).await;
        let data = serde_json::to_vec(value).ok();
        
        if let Some(data) = data {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), CacheEntry {
                data,
                cached_at: Instant::now(),
                ttl,
            });
        }
    }

    async fn get_ttl_for_endpoint(&self, key: &str, explicit_ttl: Option<u64>) -> Duration {
        if let Some(ttl) = explicit_ttl {
            return Duration::from_secs(ttl);
        }
        
        let ttls = self.endpoint_ttls.read().await;
        for (prefix, ttl_secs) in ttls.iter() {
            if key.starts_with(prefix) {
                return Duration::from_secs(*ttl_secs);
            }
        }
        
        *self.default_ttl.read().await
    }

    pub async fn set_endpoint_ttl(&self, endpoint_prefix: &str, ttl_secs: u64) {
        let mut ttls = self.endpoint_ttls.write().await;
        ttls.insert(endpoint_prefix.to_string(), ttl_secs);
    }

    pub async fn get_default_ttl(&self) -> u64 {
        self.default_ttl.read().await.as_secs()
    }

    pub async fn set_default_ttl(&self, ttl_secs: u64) {
        let mut default = self.default_ttl.write().await;
        *default = Duration::from_secs(ttl_secs);
    }

    pub async fn invalidate(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
    }

    pub async fn invalidate_prefix(&self, prefix: &str) {
        let mut cache = self.cache.write().await;
        cache.retain(|key, _| !key.starts_with(prefix));
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let mut total_entries = 0;
        let mut expired_entries = 0;
        
        for entry in cache.values() {
            total_entries += 1;
            if entry.is_expired() {
                expired_entries += 1;
            }
        }
        
        CacheStats {
            total_entries,
            expired_entries,
            active_entries: total_entries - expired_entries,
        }
    }

    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().await;
        let before = cache.len();
        cache.retain(|_, entry| !entry.is_expired());
        before - cache.len()
    }
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_and_get() {
        let cache = ResponseCache::new(60);
        
        cache.set("/api/test", &"hello".to_string(), None).await;
        let result: Option<String> = cache.get("/api/test").await;
        
        assert_eq!(result, Some("hello".to_string()));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = ResponseCache::new(60);
        
        let result: Option<String> = cache.get("/nonexistent").await;
        
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_invalidate() {
        let cache = ResponseCache::new(60);
        
        cache.set("/api/test", &"hello".to_string(), None).await;
        cache.invalidate("/api/test").await;
        
        let result: Option<String> = cache.get("/api/test").await;
        
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = ResponseCache::new(60);
        
        cache.set("/api/1", &"test1".to_string(), None).await;
        cache.set("/api/2", &"test2".to_string(), None).await;
        
        let stats = cache.stats().await;
        
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.active_entries, 2);
    }
}
