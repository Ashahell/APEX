use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiter configuration constants
pub mod config_constants {
    /// Default number of requests allowed per minute
    pub const DEFAULT_REQUESTS_PER_MINUTE: u32 = 60;
    
    /// Default burst size (max concurrent requests)
    pub const DEFAULT_BURST_SIZE: u32 = 10;
    
    /// Burst size divisor (burst = requests_per_minute / divisor)
    pub const BURST_SIZE_DIVISOR: u32 = 6;
    
    /// Rate limit window duration in seconds
    pub const WINDOW_DURATION_SECS: u64 = 60;
}

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    config: Arc<RwLock<RateLimitConfig>>,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: config_constants::DEFAULT_REQUESTS_PER_MINUTE,
            burst_size: config_constants::DEFAULT_BURST_SIZE,
        }
    }
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(RateLimitConfig {
                requests_per_minute,
                burst_size: requests_per_minute / config_constants::BURST_SIZE_DIVISOR,
            })),
        }
    }

    pub async fn check_limit(&self, key: &str) -> RateLimitResult {
        let config = self.config.read().await;
        let window = Duration::from_secs(config_constants::WINDOW_DURATION_SECS);
        
        let mut requests = self.requests.write().await;
        let entry = requests.entry(key.to_string()).or_insert_with(|| RateLimitEntry {
            count: 0,
            window_start: Instant::now(),
        });

        if entry.window_start.elapsed() > window {
            entry.count = 1;
            entry.window_start = Instant::now();
            return RateLimitResult::Allowed {
                remaining: config.requests_per_minute - 1,
                reset_in_secs: config_constants::WINDOW_DURATION_SECS as u32,
            };
        }

        if entry.count >= config.requests_per_minute {
            let reset_in = window.saturating_sub(entry.window_start.elapsed()).as_secs();
            return RateLimitResult::Denied {
                remaining: 0,
                reset_in_secs: reset_in as u32,
            };
        }

        entry.count += 1;
        let remaining = config.requests_per_minute - entry.count;
        let reset_in = window.saturating_sub(entry.window_start.elapsed()).as_secs();
        
        RateLimitResult::Allowed {
            remaining,
            reset_in_secs: reset_in as u32,
        }
    }

    pub async fn set_config(&self, requests_per_minute: u32) {
        let mut config = self.config.write().await;
        config.requests_per_minute = requests_per_minute;
        config.burst_size = requests_per_minute / config_constants::BURST_SIZE_DIVISOR;
    }

    pub async fn get_config(&self) -> RateLimitConfig {
        self.config.read().await.clone()
    }

    pub async fn reset(&self, key: &str) {
        let mut requests = self.requests.write().await;
        requests.remove(key);
    }

    pub async fn reset_all(&self) {
        let mut requests = self.requests.write().await;
        requests.clear();
    }

    pub async fn stats(&self) -> RateLimitStats {
        let requests = self.requests.read().await;
        let config = self.config.read().await;
        
        let mut active_keys = 0;
        let mut total_requests = 0u64;
        
        for entry in requests.values() {
            active_keys += 1;
            total_requests += entry.count as u64;
        }

        RateLimitStats {
            active_keys,
            total_requests,
            config: config.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed { remaining: u32, reset_in_secs: u32 },
    Denied { remaining: u32, reset_in_secs: u32 },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }

    pub fn remaining(&self) -> u32 {
        match self {
            RateLimitResult::Allowed { remaining, .. } => *remaining,
            RateLimitResult::Denied { remaining, .. } => *remaining,
        }
    }

    pub fn reset_in_secs(&self) -> u32 {
        match self {
            RateLimitResult::Allowed { reset_in_secs, .. } => *reset_in_secs,
            RateLimitResult::Denied { reset_in_secs, .. } => *reset_in_secs,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RateLimitStats {
    pub active_keys: usize,
    pub total_requests: u64,
    pub config: RateLimitConfig,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(config_constants::DEFAULT_REQUESTS_PER_MINUTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows() {
        let limiter = RateLimiter::new(10);
        
        for i in 0..10 {
            let result = limiter.check_limit("test").await;
            assert!(result.is_allowed(), "Request {} should be allowed", i + 1);
        }
        
        let result = limiter.check_limit("test").await;
        assert!(!result.is_allowed(), "11th request should be denied");
    }

    #[tokio::test]
    async fn test_rate_limiter_different_keys() {
        let limiter = RateLimiter::new(5);
        
        let result1 = limiter.check_limit("key1").await;
        let result2 = limiter.check_limit("key2").await;
        
        assert!(result1.is_allowed());
        assert!(result2.is_allowed());
    }

    #[tokio::test]
    async fn test_rate_limiter_stats() {
        let limiter = RateLimiter::new(10);
        
        limiter.check_limit("key1").await;
        limiter.check_limit("key2").await;
        
        let stats = limiter.stats().await;
        
        assert_eq!(stats.active_keys, 2);
    }
}
