//! Enhanced Rate Limiter
//!
//! Provides advanced rate limiting with:
//! - Per-endpoint specialized limits
//! - Progressive throttling (longer wait on repeated violations)
//! - IP-based and user-based limiting
//! - Detailed statistics

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Enhanced rate limiter with per-endpoint configuration
#[derive(Clone)]
pub struct EnhancedRateLimiter {
    /// Per-endpoint limiters
    endpoint_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
    /// Global fallback limiter
    global_limiter: RateLimiter,
    /// Progressive throttling state
    throttle_state: Arc<RwLock<HashMap<String, ThrottleState>>>,
    /// Configuration
    config: Arc<RwLock<EnhancedConfig>>,
}

/// Per-endpoint configuration
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub progressive_enabled: bool,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
            progressive_enabled: true,
        }
    }
}

/// Enhanced configuration
#[derive(Debug, Clone)]
pub struct EnhancedConfig {
    pub global_limit: u32,
    pub endpoints: HashMap<String, EndpointConfig>,
    pub default_endpoint_config: EndpointConfig,
}

impl Default for EnhancedConfig {
    fn default() -> Self {
        let mut endpoints = HashMap::new();

        // Task creation - stricter limit
        endpoints.insert(
            "/api/v1/tasks".to_string(),
            EndpointConfig {
                requests_per_minute: 10,
                burst_size: 2,
                progressive_enabled: true,
            },
        );

        // Skill execution - medium limit
        endpoints.insert(
            "/api/v1/skills/execute".to_string(),
            EndpointConfig {
                requests_per_minute: 30,
                burst_size: 5,
                progressive_enabled: true,
            },
        );

        // Deep tasks - very strict
        endpoints.insert(
            "/api/v1/deep".to_string(),
            EndpointConfig {
                requests_per_minute: 5,
                burst_size: 1,
                progressive_enabled: true,
            },
        );

        Self {
            global_limit: 60,
            endpoints,
            default_endpoint_config: EndpointConfig::default(),
        }
    }
}

/// Progressive throttling state
#[derive(Debug, Clone)]
struct ThrottleState {
    violation_count: u32,
    last_violation: Option<Instant>,
    blocked_until: Option<Instant>,
}

impl ThrottleState {
    fn new() -> Self {
        Self {
            violation_count: 0,
            last_violation: None,
            blocked_until: None,
        }
    }

    fn record_violation(&mut self) {
        self.violation_count += 1;
        self.last_violation = Some(Instant::now());

        // Progressive penalty: 10s, 30s, 60s, 120s...
        let penalty_secs = 10 * (2u64.pow(self.violation_count.min(6) as u32));
        self.blocked_until = Some(Instant::now() + Duration::from_secs(penalty_secs));
    }

    fn is_blocked(&self) -> bool {
        if let Some(until) = self.blocked_until {
            if until > Instant::now() {
                return true;
            }
        }
        false
    }

    fn reset(&mut self) {
        self.violation_count = 0;
        self.last_violation = None;
        self.blocked_until = None;
    }
}

/// Standard rate limiter (used internally)
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
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(RateLimitConfig {
                requests_per_minute,
                burst_size: requests_per_minute / 6,
            })),
        }
    }

    pub async fn check_limit(&self, key: &str) -> RateLimitResult {
        let config = self.config.read().await;
        let window = Duration::from_secs(60);

        let mut requests = self.requests.write().await;
        let entry = requests
            .entry(key.to_string())
            .or_insert_with(|| RateLimitEntry {
                count: 0,
                window_start: Instant::now(),
            });

        if entry.window_start.elapsed() > window {
            entry.count = 1;
            entry.window_start = Instant::now();
            return RateLimitResult::Allowed {
                remaining: config.requests_per_minute - 1,
                reset_in_secs: 60,
            };
        }

        if entry.count >= config.requests_per_minute {
            let reset_in = window
                .saturating_sub(entry.window_start.elapsed())
                .as_secs();
            return RateLimitResult::Denied {
                remaining: 0,
                reset_in_secs: reset_in as u32,
            };
        }

        entry.count += 1;
        let remaining = config.requests_per_minute - entry.count;
        let reset_in = window
            .saturating_sub(entry.window_start.elapsed())
            .as_secs();

        RateLimitResult::Allowed {
            remaining,
            reset_in_secs: reset_in as u32,
        }
    }

    pub async fn set_config(&self, requests_per_minute: u32) {
        let mut config = self.config.write().await;
        config.requests_per_minute = requests_per_minute;
        config.burst_size = requests_per_minute / 6;
    }

    pub async fn get_config(&self) -> RateLimitConfig {
        self.config.read().await.clone()
    }

    pub async fn reset(&self, key: &str) {
        let mut requests = self.requests.write().await;
        requests.remove(key);
    }

    pub async fn stats(&self) -> EnhancedStats {
        let requests = self.requests.read().await;
        let config = self.config.read().await;

        let mut active_keys = 0;
        let mut total_requests = 0u64;

        for entry in requests.values() {
            active_keys += 1;
            total_requests += entry.count as u64;
        }

        EnhancedStats {
            active_keys,
            total_requests,
            global_limit: config.requests_per_minute,
            per_endpoint_stats: HashMap::new(),
            throttle_stats: ThrottleStats {
                total_blocked: 0,
                currently_blocked: 0,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed {
        remaining: u32,
        reset_in_secs: u32,
    },
    Denied {
        remaining: u32,
        reset_in_secs: u32,
    },
    Throttled {
        retry_after_secs: u32,
        violation_count: u32,
    },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }

    pub fn remaining(&self) -> u32 {
        match self {
            RateLimitResult::Allowed { remaining, .. } => *remaining,
            RateLimitResult::Denied { remaining, .. } => *remaining,
            RateLimitResult::Throttled { .. } => 0,
        }
    }

    pub fn reset_in_secs(&self) -> u32 {
        match self {
            RateLimitResult::Allowed { reset_in_secs, .. } => *reset_in_secs,
            RateLimitResult::Denied { reset_in_secs, .. } => *reset_in_secs,
            RateLimitResult::Throttled {
                retry_after_secs, ..
            } => *retry_after_secs,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EnhancedStats {
    pub active_keys: usize,
    pub total_requests: u64,
    pub global_limit: u32,
    pub per_endpoint_stats: HashMap<String, EndpointStats>,
    pub throttle_stats: ThrottleStats,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EndpointStats {
    pub limit: u32,
    pub active_keys: usize,
    pub total_requests: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ThrottleStats {
    pub total_blocked: u64,
    pub currently_blocked: u32,
}

// =============================================================================
// Enhanced Rate Limiter Implementation
// =============================================================================

impl EnhancedRateLimiter {
    /// Create a new enhanced rate limiter
    pub fn new() -> Self {
        let config = EnhancedConfig::default();

        let mut endpoint_limiters = HashMap::new();
        for (endpoint, endpoint_config) in &config.endpoints {
            endpoint_limiters.insert(
                endpoint.clone(),
                RateLimiter::new(endpoint_config.requests_per_minute),
            );
        }

        Self {
            endpoint_limiters: Arc::new(RwLock::new(endpoint_limiters)),
            global_limiter: RateLimiter::new(config.global_limit),
            throttle_state: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Check rate limit for an endpoint
    pub async fn check_limit(&self, endpoint: &str, client_id: &str) -> RateLimitResult {
        // First check progressive throttle
        let _should_throttle = {
            let mut throttle = self.throttle_state.write().await;
            let state = throttle
                .entry(client_id.to_string())
                .or_insert_with(ThrottleState::new);

            if state.is_blocked() {
                if let Some(until) = state.blocked_until {
                    let retry_after =
                        until.saturating_duration_since(Instant::now()).as_secs() as u32;
                    return RateLimitResult::Throttled {
                        retry_after_secs: retry_after,
                        violation_count: state.violation_count,
                    };
                }
            }
            false
        };

        // Try endpoint-specific limiter first
        let endpoint_config = {
            let config = self.config.read().await;
            config
                .endpoints
                .get(endpoint)
                .cloned()
                .unwrap_or_else(|| config.default_endpoint_config.clone())
        };

        // Check endpoint limiter if configured
        if let Some(limiter) = self.endpoint_limiters.read().await.get(endpoint) {
            let full_key = format!("{}:{}", endpoint, client_id);
            let result = limiter.check_limit(&full_key).await;

            match &result {
                RateLimitResult::Allowed { .. } => {
                    // Success - reset throttle state
                    let mut throttle = self.throttle_state.write().await;
                    if let Some(state) = throttle.get_mut(client_id) {
                        state.reset();
                    }
                }
                RateLimitResult::Denied { .. } => {
                    // Failed - record violation for progressive throttle
                    if endpoint_config.progressive_enabled {
                        let mut throttle = self.throttle_state.write().await;
                        if let Some(state) = throttle.get_mut(client_id) {
                            state.record_violation();
                        }
                    }
                }
                _ => {}
            }

            return result;
        }

        // Fall back to global limiter
        let full_key = format!("global:{}", client_id);
        let result = self.global_limiter.check_limit(&full_key).await;

        match &result {
            RateLimitResult::Allowed { .. } => {
                let mut throttle = self.throttle_state.write().await;
                if let Some(state) = throttle.get_mut(client_id) {
                    state.reset();
                }
            }
            RateLimitResult::Denied { .. } => {
                if endpoint_config.progressive_enabled {
                    let mut throttle = self.throttle_state.write().await;
                    if let Some(state) = throttle.get_mut(client_id) {
                        state.record_violation();
                    }
                }
            }
            _ => {}
        }

        result
    }

    /// Get enhanced statistics
    pub async fn stats(&self) -> EnhancedStats {
        let config = self.config.read().await;

        let mut per_endpoint_stats = HashMap::new();

        let endpoint_limiters = self.endpoint_limiters.read().await;
        for (endpoint, limiter) in endpoint_limiters.iter() {
            let endpoint_stats = limiter.stats().await;
            per_endpoint_stats.insert(
                endpoint.clone(),
                EndpointStats {
                    limit: config
                        .endpoints
                        .get(endpoint)
                        .map(|c| c.requests_per_minute)
                        .unwrap_or(60),
                    active_keys: endpoint_stats.active_keys,
                    total_requests: endpoint_stats.total_requests,
                },
            );
        }

        let throttle = self.throttle_state.read().await;
        let mut total_blocked = 0u64;
        let mut currently_blocked = 0u32;

        for state in throttle.values() {
            if state.violation_count > 0 {
                total_blocked += 1;
            }
            if state.is_blocked() {
                currently_blocked += 1;
            }
        }

        EnhancedStats {
            active_keys: throttle.len(),
            total_requests: 0, // Would need aggregation
            global_limit: config.global_limit,
            per_endpoint_stats,
            throttle_stats: ThrottleStats {
                total_blocked,
                currently_blocked,
            },
        }
    }

    /// Update endpoint configuration
    pub async fn set_endpoint_config(&self, endpoint: &str, endpoint_config: EndpointConfig) {
        {
            let mut config = self.config.write().await;
            config
                .endpoints
                .insert(endpoint.to_string(), endpoint_config.clone());
        }

        // Update or create the limiter
        let mut limiters = self.endpoint_limiters.write().await;
        limiters.insert(
            endpoint.to_string(),
            RateLimiter::new(endpoint_config.requests_per_minute),
        );
    }

    /// Reset throttle state and endpoint limiter for a client
    /// This resets both the progressive throttle state AND the underlying
    /// rate limiter counters, allowing the client to start fresh.
    pub async fn reset_throttle(&self, client_id: &str) {
        // Clear progressive throttle state
        let mut throttle = self.throttle_state.write().await;
        throttle.remove(client_id);

        // Reset endpoint limiters for common endpoints
        let endpoints = ["/api/v1/tasks", "/api/v1/skills/execute", "/api/v1/deep"];
        let mut limiters = self.endpoint_limiters.write().await;
        for endpoint in endpoints {
            if let Some(limiter) = limiters.get_mut(endpoint) {
                let full_key = format!("{}:{}", endpoint, client_id);
                limiter.reset(&full_key).await;
            }
        }

        // Also reset global limiter
        let full_key = format!("global:{}", client_id);
        self.global_limiter.reset(&full_key).await;
    }
}

impl Default for EnhancedRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_rate_limiter_allows() {
        let limiter = EnhancedRateLimiter::new();

        // Should allow requests up to limit
        for _ in 0..10 {
            let result = limiter.check_limit("/api/v1/tasks", "client1").await;
            assert!(result.is_allowed(), "Should be allowed");
        }

        // 11th should be denied
        let result = limiter.check_limit("/api/v1/tasks", "client1").await;
        assert!(!result.is_allowed());
    }

    #[tokio::test]
    async fn test_enhanced_rate_limiter_different_endpoints() {
        let limiter = EnhancedRateLimiter::new();

        // Different endpoints have different limits
        // /api/v1/tasks has limit of 10
        let result1 = limiter.check_limit("/api/v1/tasks", "client1").await;

        // /api/v1/skills/execute has limit of 30
        let result2 = limiter
            .check_limit("/api/v1/skills/execute", "client1")
            .await;

        assert!(result1.is_allowed());
        assert!(result2.is_allowed());
    }

    #[tokio::test]
    async fn test_enhanced_rate_limiter_different_clients() {
        let limiter = EnhancedRateLimiter::new();

        // Different clients have separate limits
        let result1 = limiter.check_limit("/api/v1/tasks", "client1").await;
        let result2 = limiter.check_limit("/api/v1/tasks", "client2").await;

        assert!(result1.is_allowed());
        assert!(result2.is_allowed());
    }

    #[tokio::test]
    async fn test_enhanced_stats() {
        let limiter = EnhancedRateLimiter::new();

        limiter.check_limit("/api/v1/tasks", "client1").await;

        let stats = limiter.stats().await;

        assert!(stats.per_endpoint_stats.contains_key("/api/v1/tasks"));
    }

    #[tokio::test]
    async fn test_throttle_reset() {
        let limiter = EnhancedRateLimiter::new();

        // Make requests until throttled
        for _ in 0..15 {
            let _ = limiter.check_limit("/api/v1/tasks", "client1").await;
        }

        // Should be throttled now
        let result = limiter.check_limit("/api/v1/tasks", "client1").await;

        // Reset throttle
        limiter.reset_throttle("client1").await;

        // Should be allowed now
        let result_after = limiter.check_limit("/api/v1/tasks", "client1").await;
        assert!(result_after.is_allowed());
    }
}
