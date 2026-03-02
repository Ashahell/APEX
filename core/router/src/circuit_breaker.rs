use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

const DEFAULT_FAILURE_THRESHOLD: u32 = 5;
const DEFAULT_RECOVERY_TIMEOUT_SECS: u64 = 60;

#[derive(Clone, Debug, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Clone, Debug)]
pub struct CircuitBreaker {
    state: CircuitBreakerState,
    failure_count: u32,
    last_failure: Option<Instant>,
    failure_threshold: u32,
    recovery_timeout: Duration,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            last_failure: None,
            failure_threshold: DEFAULT_FAILURE_THRESHOLD,
            recovery_timeout: Duration::from_secs(DEFAULT_RECOVERY_TIMEOUT_SECS),
        }
    }

    pub fn with_threshold(mut self, threshold: u32) -> Self {
        self.failure_threshold = threshold;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.recovery_timeout = Duration::from_secs(timeout_secs);
        self
    }

    pub fn is_available(&mut self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() >= self.recovery_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        return true;
                    }
                }
                false
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitBreakerState::Open;
        }
    }

    pub fn get_state(&self) -> &CircuitBreakerState {
        &self.state
    }

    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.state = CircuitBreakerState::Closed;
        self.last_failure = None;
    }
}

#[derive(Clone)]
pub struct CircuitBreakerRegistry {
    breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreakerRegistry {
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn is_available(&self, skill_name: &str) -> bool {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(skill_name) {
            match breaker.state {
                CircuitBreakerState::Closed => true,
                CircuitBreakerState::HalfOpen => true,
                CircuitBreakerState::Open => {
                    if let Some(last_failure) = breaker.last_failure {
                        if last_failure.elapsed() >= breaker.recovery_timeout {
                            return true;
                        }
                    }
                    false
                }
            }
        } else {
            true
        }
    }

    pub async fn record_success(&self, skill_name: &str) {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(skill_name) {
            breaker.record_success();
        }
    }

    pub async fn record_failure(&self, skill_name: &str) {
        let mut breakers = self.breakers.write().await;
        let breaker = breakers
            .entry(skill_name.to_string())
            .or_insert_with(CircuitBreaker::new);
        breaker.record_failure();
    }

    pub async fn get_state(&self, skill_name: &str) -> CircuitBreakerState {
        let breakers = self.breakers.read().await;
        breakers
            .get(skill_name)
            .map(|b| b.get_state().clone())
            .unwrap_or(CircuitBreakerState::Closed)
    }

    pub async fn reset(&self, skill_name: &str) {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(skill_name) {
            breaker.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_closed_by_default() {
        let mut breaker = CircuitBreaker::new();
        assert!(matches!(breaker.get_state(), CircuitBreakerState::Closed));
        assert!(breaker.is_available());
    }

    #[test]
    fn test_circuit_breaker_opens_after_threshold() {
        let mut breaker = CircuitBreaker::new().with_threshold(3);

        breaker.record_failure();
        assert!(breaker.is_available());
        breaker.record_failure();
        assert!(breaker.is_available());
        breaker.record_failure();

        assert!(matches!(breaker.get_state(), CircuitBreakerState::Open));
        assert!(!breaker.is_available());
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let mut breaker = CircuitBreaker::new().with_threshold(2);

        breaker.record_failure();
        breaker.record_failure();
        assert!(matches!(breaker.get_state(), CircuitBreakerState::Open));

        breaker.record_success();
        assert!(matches!(breaker.get_state(), CircuitBreakerState::Closed));
        assert!(breaker.is_available());
    }
}
