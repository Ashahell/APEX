//! Privacy Guard - Blocks cloud connections when privacy mode is enabled
//!
//! Feature 6: Privacy Toggle

use serde::{Deserialize, Serialize};

use crate::unified_config::privacy_constants::*;

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Whether privacy mode is enabled
    pub enabled: bool,
    /// Blocked providers
    pub blocked_providers: Vec<String>,
    /// Allow local-only connections
    pub allow_local_only: bool,
    /// Whether to audit blocked attempts
    pub audit_log_enabled: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            blocked_providers: CLOUD_PROVIDERS.iter().map(|s| s.to_string()).collect(),
            allow_local_only: true,
            audit_log_enabled: true,
        }
    }
}

/// Privacy check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyCheckResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Provider that was blocked (if any)
    pub blocked_provider: Option<String>,
    /// Reason for blocking
    pub reason: Option<String>,
}

/// Privacy Guard - checks if LLM requests should be blocked
pub struct PrivacyGuard {
    config: PrivacyConfig,
}

impl PrivacyGuard {
    /// Create a new privacy guard with config
    pub fn new(config: PrivacyConfig) -> Self {
        Self { config }
    }

    /// Create with default config (privacy disabled)
    pub fn default_guard() -> Self {
        Self::new(PrivacyConfig::default())
    }

    /// Check if a provider request should be allowed
    pub fn check(&self, provider: &str) -> PrivacyCheckResult {
        // If privacy mode is disabled, allow everything
        if !self.config.enabled {
            return PrivacyCheckResult {
                allowed: true,
                blocked_provider: None,
                reason: None,
            };
        }

        // Check if provider is in blocked list
        let provider_lower = provider.to_lowercase();
        let is_cloud = CLOUD_PROVIDERS
            .iter()
            .any(|p| provider_lower.contains(&p.to_lowercase()));

        if is_cloud {
            PrivacyCheckResult {
                allowed: false,
                blocked_provider: Some(provider.to_string()),
                reason: Some(format!(
                    "Privacy mode is enabled. Cloud provider '{}' is blocked.",
                    provider
                )),
            }
        } else {
            // Local provider - check allow_local_only setting
            if self.config.allow_local_only {
                PrivacyCheckResult {
                    allowed: true,
                    blocked_provider: None,
                    reason: None,
                }
            } else {
                PrivacyCheckResult {
                    allowed: false,
                    blocked_provider: Some(provider.to_string()),
                    reason: Some("Only local providers allowed in privacy mode".to_string()),
                }
            }
        }
    }

    /// Get current config
    pub fn config(&self) -> &PrivacyConfig {
        &self.config
    }

    /// Update config
    pub fn update_config(&mut self, config: PrivacyConfig) {
        self.config = config;
    }

    /// Check if privacy is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get list of cloud providers
    pub fn cloud_providers() -> Vec<&'static str> {
        CLOUD_PROVIDERS.to_vec()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_disabled_allows_all() {
        let guard = PrivacyGuard::default_guard();
        let result = guard.check("openai");
        assert!(result.allowed);
    }

    #[test]
    fn test_privacy_enabled_blocks_cloud() {
        let mut config = PrivacyConfig::default();
        config.enabled = true;
        let guard = PrivacyGuard::new(config);

        let result = guard.check("openai");
        assert!(!result.allowed);
        assert_eq!(result.blocked_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_privacy_enabled_allows_local() {
        let mut config = PrivacyConfig::default();
        config.enabled = true;
        config.allow_local_only = true;
        let guard = PrivacyGuard::new(config);

        let result = guard.check("local");
        assert!(result.allowed);
    }

    #[test]
    fn test_blocks_anthropic() {
        let mut config = PrivacyConfig::default();
        config.enabled = true;
        let guard = PrivacyGuard::new(config);

        let result = guard.check("anthropic");
        assert!(!result.allowed);
    }

    #[test]
    fn test_blocks_google() {
        let mut config = PrivacyConfig::default();
        config.enabled = true;
        let guard = PrivacyGuard::new(config);

        let result = guard.check("google");
        assert!(!result.allowed);
    }

    #[test]
    fn test_cloud_providers_list() {
        let providers = PrivacyGuard::cloud_providers();
        assert!(providers.contains(&"openai"));
        assert!(providers.contains(&"anthropic"));
        assert!(providers.contains(&"google"));
    }

    #[test]
    fn test_default_config_disabled() {
        let config = PrivacyConfig::default();
        assert!(!config.enabled);
        assert!(config.audit_log_enabled);
    }
}
