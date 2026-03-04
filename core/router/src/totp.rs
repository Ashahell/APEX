use serde::{Deserialize, Serialize};
use totp_rs::{TOTP, Algorithm};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use base32::Alphabet;

#[derive(Clone)]
pub struct TotpManager {
    secrets: Arc<RwLock<HashMap<String, String>>>,
}

impl TotpManager {
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn generate_secret(&self, user_id: &str) -> Result<String, String> {
        let secret_bytes: Vec<u8> = (0..20).map(|_| rand::random::<u8>()).collect();
        let secret_encoded = base32::encode(Alphabet::Rfc4648 { padding: false }, &secret_bytes);
        
        let mut secrets = self.secrets.write().await;
        secrets.insert(user_id.to_string(), secret_encoded.clone());
        
        Ok(secret_encoded)
    }

    pub async fn verify(&self, user_id: &str, token: &str) -> Result<bool, String> {
        let secrets = self.secrets.read().await;
        let secret_str = secrets.get(user_id)
            .ok_or_else(|| "No TOTP secret found for user".to_string())?;
        
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret_str.as_str())
            .ok_or_else(|| "Invalid base32 secret".to_string())?;
        
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
            Some("APEX".to_string()),
            user_id.to_string(),
        ).map_err(|e| format!("Failed to create TOTP: {}", e))?;

        let is_valid = totp.check(token, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_secs());

        Ok(is_valid)
    }

    pub async fn remove_secret(&self, user_id: &str) {
        let mut secrets = self.secrets.write().await;
        secrets.remove(user_id);
    }

    pub async fn has_secret(&self, user_id: &str) -> bool {
        let secrets = self.secrets.read().await;
        secrets.contains_key(user_id)
    }

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
