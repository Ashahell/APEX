//! TOTP Manager with Persistence Support
//!
//! Provides TOTP verification with optional persistence using SecretStore.

use base32::Alphabet;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use totp_rs::{Algorithm, TOTP};

use crate::secret_store::SecretStore;

/// TOTP Manager - supports both in-memory and persistent storage
#[derive(Clone)]
pub struct TotpManager {
    /// In-memory secrets (fallback when persistence unavailable)
    secrets: Arc<RwLock<HashMap<String, String>>>,
    /// Optional persistent storage
    persistent_store: Option<SecretStore>,
}

impl TotpManager {
    /// Create a new in-memory TOTP manager
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            persistent_store: None,
        }
    }

    /// Create a TOTP manager with persistent storage
    pub fn with_persistence(store: SecretStore) -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            persistent_store: Some(store),
        }
    }

    /// Generate a new TOTP secret for a user
    pub async fn generate_secret(&self, user_id: &str) -> Result<String, String> {
        // Generate random 20-byte secret using cryptographically secure RNG
        let mut secret_bytes = [0u8; 20];
        StdRng::from_entropy().fill_bytes(&mut secret_bytes);
        let secret_encoded = base32::encode(Alphabet::Rfc4648 { padding: false }, &secret_bytes);

        // Store in memory
        let mut secrets = self.secrets.write().await;
        secrets.insert(user_id.to_string(), secret_encoded.clone());

        // Also persist if available
        if let Some(ref store) = self.persistent_store {
            let _ = store.set_totp_secret(user_id, &secret_encoded);
        }

        Ok(secret_encoded)
    }

    /// Verify a TOTP token
    pub async fn verify(&self, user_id: &str, token: &str) -> Result<bool, String> {
        // Try in-memory first
        let secret_str = {
            let secrets = self.secrets.read().await;
            secrets.get(user_id).cloned()
        };

        // Fall back to persistent storage
        let secret = if let Some(s) = secret_str {
            s
        } else if let Some(ref store) = self.persistent_store {
            match store.get_totp_secret(user_id) {
                Ok(s) => s,
                Err(_) => return Err("No TOTP secret found for user".to_string()),
            }
        } else {
            return Err("No TOTP secret found for user".to_string());
        };

        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, &secret)
            .ok_or_else(|| "Invalid base32 secret".to_string())?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("APEX".to_string()),
            user_id.to_string(),
        )
        .map_err(|e| format!("Failed to create TOTP: {}", e))?;

        let is_valid = totp.check(
            token,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("Time error: {}", e))?
                .as_secs(),
        );

        Ok(is_valid)
    }

    /// Remove TOTP secret for a user
    pub async fn remove_secret(&self, user_id: &str) {
        // Remove from memory
        let mut secrets = self.secrets.write().await;
        secrets.remove(user_id);

        // Remove from persistent storage
        if let Some(ref store) = self.persistent_store {
            let _ = store.delete_totp_secret(user_id);
        }
    }

    /// Check if user has TOTP configured
    pub async fn has_secret(&self, user_id: &str) -> bool {
        // Check memory first
        {
            let secrets = self.secrets.read().await;
            if secrets.contains_key(user_id) {
                return true;
            }
        }

        // Check persistent storage
        if let Some(ref store) = self.persistent_store {
            return store.has_totp(user_id);
        }

        false
    }

    /// Load secrets from persistent storage
    pub async fn load_from_persistence(&self) -> Result<(), String> {
        if let Some(ref _store) = self.persistent_store {
            // We can't list all TOTP secrets easily, so this is a no-op
            // The secrets are loaded on-demand via get_totp_secret
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Generate otpauth URI for QR code
    pub fn generate_otpauth_uri(secret: &str, account_name: &str, issuer: &str) -> String {
        format!(
            "otpauth://totp/{}?secret={}&issuer={}",
            account_name, secret, issuer
        )
    }
}

impl Default for TotpManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub otpauth_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TotpVerifyRequest {
    pub token: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_totp_manager_new() {
        let manager = TotpManager::new();
        assert!(!manager.has_secret("test-user").await);
    }

    #[tokio::test]
    async fn test_generate_secret() {
        let manager = TotpManager::new();
        let secret = manager.generate_secret("test-user").await.unwrap();

        assert!(!secret.is_empty());
        assert!(manager.has_secret("test-user").await);
    }

    #[tokio::test]
    async fn test_remove_secret() {
        let manager = TotpManager::new();
        manager.generate_secret("test-user").await.unwrap();

        assert!(manager.has_secret("test-user").await);

        manager.remove_secret("test-user").await;

        assert!(!manager.has_secret("test-user").await);
    }

    #[tokio::test]
    async fn test_verify_no_secret() {
        let manager = TotpManager::new();
        let result = manager.verify("nonexistent-user", "123456").await;

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_otpauth_uri() {
        let uri = TotpManager::generate_otpauth_uri("JBSWY3DPEHPK3PXP", "test-user", "APEX");

        assert!(uri.contains("otpauth://totp/"));
        assert!(uri.contains("secret="));
        assert!(uri.contains("issuer=APEX"));
    }
}
