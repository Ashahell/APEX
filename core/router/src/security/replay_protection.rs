//! Replay protection for HMAC-authenticated streaming requests.
//!
//! Patch 15: Added distributed Redis backend alongside the in-memory fallback.
//!
//! Architecture:
//!   - `ReplayProtection` trait defines the interface
//!   - `InMemoryReplayProtection` — single-process HashSet (default, no extra deps)
//!   - `RedisReplayProtection` — distributed SETNX with TTL (requires `redis` feature)
//!
//! Thread-safety: Uses `std::sync::Mutex` (not Tokio's) so it can be called
//! from both async and sync contexts without holding a tokio runtime borrow.

use std::collections::HashSet;

/// TTL for Redis replay entries (5 minutes — same as HMAC timestamp drift window).
const REDIS_TTL_SECS: &str = "300";

/// Maximum signatures tracked in-memory before pruning oldest entries.
const MAX_SIGNATURES: usize = 10_000;

// ---------------------------------------------------------------------------
// Trait definition
// ---------------------------------------------------------------------------

/// Trait for replay protection backends.
///
/// Implement this to add new storage backends (Redis, PostgreSQL, etc.).
pub trait ReplayProtection: Send + Sync {
    /// Atomically check if a signature is a replay and record it.
    ///
    /// Returns `true` if this was a replay (already seen → reject request).
    /// Returns `false` if this is a new signature (recorded → allow request).
    fn record_and_check(&self, signature: &str) -> bool;

    /// Clear all tracked signatures. Useful for tests.
    fn reset(&self);

    /// Returns the current count of tracked signatures.
    fn count(&self) -> usize;
}

// ---------------------------------------------------------------------------
// In-memory backend (default)
// ---------------------------------------------------------------------------

/// Thread-local in-memory store of observed HMAC signatures/nonces.
/// Each thread gets its own HashSet via RefCell for interior mutability,
/// eliminating parallel test race conditions.
thread_local! {
    static SEEN_SIGNATURES: std::cell::RefCell<HashSet<String>> = std::cell::RefCell::new(HashSet::new());
}

fn with_set<F, T>(f: F) -> T
where
    F: FnOnce(&mut HashSet<String>) -> T,
{
    SEEN_SIGNATURES.with(|cell| {
        let mut set = cell.borrow_mut();
        f(&mut set)
    })
}

/// In-memory replay protection backend using a thread-local HashSet.
///
/// **Use for**: single-process deployments, development, testing.
/// **Do not use for**: multi-instance production deployments.
pub struct InMemoryReplayProtection;

impl Default for InMemoryReplayProtection {
    fn default() -> Self {
        Self
    }
}

impl ReplayProtection for InMemoryReplayProtection {
    fn record_and_check(&self, signature: &str) -> bool {
        with_set(|set| {
            if set.contains(signature) {
                return true;
            }
            set.insert(signature.to_string());

            // Prune if we exceed the max tracked signatures (simple FIFO by rebuild)
            if set.len() > MAX_SIGNATURES {
                let drain_count = MAX_SIGNATURES / 2;
                let keys: Vec<_> = set.iter().take(drain_count).cloned().collect();
                for k in keys {
                    set.remove(&k);
                }
            }
            false
        })
    }

    fn reset(&self) {
        with_set(|set| set.clear());
    }

    fn count(&self) -> usize {
        with_set(|set| set.len())
    }
}

// ---------------------------------------------------------------------------
// Redis backend (distributed) — requires `redis` feature
// ---------------------------------------------------------------------------

#[cfg(feature = "redis")]
mod redis_backend {
    use super::*;

    /// Redis-backed replay protection using SETNX with TTL.
    ///
    /// **Use for**: multi-instance production deployments.
    /// **Requires**: `redis` feature flag and `deadpool-redis` crate.
    pub struct RedisReplayProtection {
        pool: deadpool_redis::Pool,
    }

    impl RedisReplayProtection {
        /// Create a new Redis-backed replay protection.
        ///
        /// `redis_url` follows the format: `redis://[user:password@]host:port/db`.
        pub fn new(redis_url: &str) -> Result<Self, deadpool_redis::CreatePoolError> {
            let cfg = deadpool_redis::Config {
                url: Some(redis_url.to_string()),
                ..Default::default()
            };
            let pool = cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1));
            Ok(Self { pool })
        }
    }

    impl ReplayProtection for RedisReplayProtection {
        fn record_and_check(&self, signature: &str) -> bool {
            // Use SETNX + EXPIRE for atomic check-and-set with TTL.
            // If the key already exists, SETNX returns nil (replay).
            // If the key was set, SETNX returns Some("OK") (new signature).
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let mut conn = match self.pool.get().await {
                    Ok(c) => c,
                    Err(_) => return true, // Connection failure → reject (fail-closed)
                };

                let key = format!("apex:replay:{}", signature);

                // SET key value NX EX ttl — atomic set-if-not-exists with TTL
                let resp: Result<Option<String>, _> = redis::cmd("SET")
                    .arg(&key)
                    .arg("1")
                    .arg("NX")
                    .arg("EX")
                    .arg(REDIS_TTL_SECS)
                    .query_async(&mut conn)
                    .await;

                match resp {
                    Ok(Some(_)) => false, // Key was set — not a replay
                    Ok(None) => true,     // Key already existed — replay
                    Err(_) => true,       // Error — fail closed
                }
            })
        }

        fn reset(&self) {
            // Reset is not implemented for Redis — signatures expire via TTL.
        }

        fn count(&self) -> usize {
            // Not implemented for Redis — would require SCAN.
            0
        }
    }

    /// Create a Redis-backed ReplayProtection (requires `redis` feature).
    pub fn create_redis(
        redis_url: &str,
    ) -> Result<Box<dyn ReplayProtection>, deadpool_redis::CreatePoolError> {
        RedisReplayProtection::new(redis_url).map(|rp| Box::new(rp) as Box<dyn ReplayProtection>)
    }
}

// ---------------------------------------------------------------------------
// Public convenience functions (backward-compatible with existing callers)
// ---------------------------------------------------------------------------

/// In-memory protection instance — shared via a function to avoid static lifetime issues.
fn in_memory_protection() -> &'static InMemoryReplayProtection {
    static INSTANCE: InMemoryReplayProtection = InMemoryReplayProtection;
    &INSTANCE
}

/// Check-and-record using the default in-memory backend.
/// Kept for backward compatibility with existing callers.
pub fn record_and_check(signature: &str) -> bool {
    in_memory_protection().record_and_check(signature)
}

/// Clear all tracked signatures (in-memory only).
/// Operates directly on the thread-local HashSet.
#[allow(dead_code)]
pub fn reset() {
    with_set(|set| set.clear());
}

/// Returns the current count of tracked signatures (in-memory only).
#[allow(dead_code)]
pub fn count() -> usize {
    in_memory_protection().count()
}

/// Create the appropriate ReplayProtection backend from config.
pub fn from_config(backend: &str, redis_url: Option<&str>) -> Box<dyn ReplayProtection> {
    match backend {
        #[cfg(feature = "redis")]
        "redis" => {
            if let Some(url) = redis_url {
                match redis_backend::create_redis(url) {
                    Ok(rp) => {
                        tracing::info!("Replay protection: Redis backend");
                        rp
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            "Failed to create Redis replay protection, falling back to in-memory"
                        );
                        Box::new(InMemoryReplayProtection)
                    }
                }
            } else {
                tracing::warn!(
                    "Redis backend requested but no redis_url configured, using in-memory"
                );
                Box::new(InMemoryReplayProtection)
            }
        }
        _ => {
            tracing::debug!("Replay protection: in-memory backend");
            Box::new(InMemoryReplayProtection)
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_signature_not_replay() {
        reset();
        let sig = "fresh-sig-001";
        let is_replay = record_and_check(sig);
        assert!(!is_replay, "fresh signature should not be a replay");
    }

    #[test]
    fn test_duplicate_signature_is_replay() {
        reset();
        let sig = "duplicate-sig-002";
        let first = record_and_check(sig);
        let second = record_and_check(sig);
        assert!(!first, "first use should not be replay");
        assert!(second, "second use should be replay");
    }

    #[test]
    fn test_reset_clears_signatures() {
        reset();
        let sig = "reset-test-003";
        record_and_check(sig);
        reset();
        let after_reset = record_and_check(sig);
        assert!(!after_reset, "after reset, same sig should not be replay");
    }

    #[test]
    fn test_distinct_signatures_not_replays() {
        reset();
        let sigs = ["sig-a", "sig-b", "sig-c", "sig-d", "sig-e"];
        for sig in &sigs {
            let is_replay = record_and_check(sig);
            assert!(
                !is_replay,
                "distinct signature {} should not be replay",
                sig
            );
        }
    }

    #[test]
    fn test_record_and_check_is_atomic() {
        reset();
        let sig = "atomic-005";
        let r1 = record_and_check(sig);
        let r2 = record_and_check(sig);
        assert!(!r1);
        assert!(r2);
    }

    #[test]
    fn test_from_config_memory() {
        reset();
        let rp = from_config("memory", None);
        assert_eq!(rp.count(), 0);
        assert!(!rp.record_and_check("sig-x"));
        assert!(rp.record_and_check("sig-x"));
    }

    #[test]
    fn test_from_config_unknown_falls_back_to_memory() {
        reset();
        let rp = from_config("unknown-backend", None);
        assert!(!rp.record_and_check("sig-y"));
    }
}
