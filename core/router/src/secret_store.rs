//! Secure Secret Storage
//!
//! Provides encrypted storage for sensitive data like HMAC secrets and TOTP keys.
//! Uses AES-256-GCM encryption with a machine-derived key.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Errors for secret storage
#[derive(Error, Debug)]
pub enum SecretStorageError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Secret not found: {0}")]
    NotFound(String),
}

/// Secret store entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    pub service: String,
    pub key: String,
    pub value: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Encrypted secret store
#[derive(Clone)]
pub struct SecretStore {
    store_path: PathBuf,
    master_key: [u8; 32],
}

impl SecretStore {
    /// Create a new secret store
    pub fn new(store_path: PathBuf) -> Result<Self, SecretStorageError> {
        // Derive master key from machine ID
        let machine_key = Self::derive_machine_key();

        // Initialize store
        let store = Self {
            store_path,
            master_key: machine_key,
        };

        // Create store directory if needed
        if let Some(parent) = store.store_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(store)
    }

    /// Derive a key from machine-specific values
    fn derive_machine_key() -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Gather machine-specific data
        let mut hasher = DefaultHasher::new();

        // hostname
        if let Ok(hostname) = std::env::var("COMPUTERNAME") {
            hostname.hash(&mut hasher);
        } else if let Ok(hostname) = std::env::var("HOSTNAME") {
            hostname.hash(&mut hasher);
        }

        // username
        if let Ok(username) = std::env::var("USERNAME") {
            username.hash(&mut hasher);
        }

        // Platform
        std::env::consts::OS.hash(&mut hasher);
        std::env::consts::ARCH.hash(&mut hasher);

        let hash = hasher.finish();

        // Expand to 32 bytes
        let mut key = [0u8; 32];
        key[..8].copy_from_slice(&hash.to_le_bytes());
        key[8..16].copy_from_slice(&hash.to_be_bytes());
        key[16..24].copy_from_slice(&hash.to_le_bytes());
        key[24..32].copy_from_slice(&hash.to_be_bytes());

        key
    }

    /// Encrypt a value
    fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, SecretStorageError> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| SecretStorageError::EncryptionFailed(e.to_string()))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| SecretStorageError::EncryptionFailed(e.to_string()))?;

        // Prepend nonce
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// Decrypt a value
    fn decrypt(&self, data: &[u8]) -> Result<String, SecretStorageError> {
        if data.len() < 12 {
            return Err(SecretStorageError::DecryptionFailed(
                "Data too short".into(),
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| SecretStorageError::DecryptionFailed(e.to_string()))?;

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SecretStorageError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| SecretStorageError::DecryptionFailed(e.to_string()))
    }

    /// Store a secret
    pub fn set(&self, service: &str, key: &str, value: &str) -> Result<(), SecretStorageError> {
        let now = chrono::Utc::now().timestamp();

        let entry = SecretEntry {
            service: service.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            created_at: now,
            updated_at: now,
        };

        // Load existing entries
        let mut entries = self.load_entries().unwrap_or_default();

        // Update or add
        if let Some(existing) = entries
            .iter_mut()
            .find(|e| e.service == service && e.key == key)
        {
            existing.value = value.to_string();
            existing.updated_at = now;
        } else {
            entries.push(entry);
        }

        // Save
        self.save_entries(&entries)
    }

    /// Retrieve a secret
    pub fn get(&self, service: &str, key: &str) -> Result<String, SecretStorageError> {
        let entries = self.load_entries()?;

        entries
            .into_iter()
            .find(|e| e.service == service && e.key == key)
            .map(|e| e.value)
            .ok_or_else(|| SecretStorageError::NotFound(format!("{}/{}", service, key)))
    }

    /// Delete a secret
    pub fn delete(&self, service: &str, key: &str) -> Result<(), SecretStorageError> {
        let mut entries = self.load_entries()?;

        let initial_len = entries.len();
        entries.retain(|e| !(e.service == service && e.key == key));

        if entries.len() == initial_len {
            return Err(SecretStorageError::NotFound(format!("{}/{}", service, key)));
        }

        self.save_entries(&entries)
    }

    /// Check if a secret exists
    pub fn exists(&self, service: &str, key: &str) -> bool {
        self.get(service, key).is_ok()
    }

    /// Load and decrypt entries from file
    fn load_entries(&self) -> Result<Vec<SecretEntry>, SecretStorageError> {
        if !self.store_path.exists() {
            return Ok(Vec::new());
        }

        let encrypted = std::fs::read(&self.store_path)?;

        if encrypted.is_empty() {
            return Ok(Vec::new());
        }

        let decrypted = self.decrypt(&encrypted)?;
        let entries: Vec<SecretEntry> = serde_json::from_str(&decrypted)
            .map_err(|e| SecretStorageError::DecryptionFailed(e.to_string()))?;

        Ok(entries)
    }

    /// Encrypt and save entries to file
    fn save_entries(&self, entries: &[SecretEntry]) -> Result<(), SecretStorageError> {
        let json = serde_json::to_string(entries)
            .map_err(|e| SecretStorageError::EncryptionFailed(e.to_string()))?;

        let encrypted = self.encrypt(&json)?;

        std::fs::write(&self.store_path, encrypted)?;

        Ok(())
    }
}

/// HMAC secret management
impl SecretStore {
    /// Get the HMAC secret, generating if needed
    pub fn get_hmac_secret(&self) -> Result<String, SecretStorageError> {
        if let Ok(secret) = self.get("apex", "hmac_secret") {
            return Ok(secret);
        }

        // Generate new secret
        let mut secret = [0u8; 32];
        rand::thread_rng().fill(&mut secret);
        let secret_hex = hex::encode(secret);

        self.set("apex", "hmac_secret", &secret_hex)?;

        Ok(secret_hex)
    }
}

/// TOTP secret management  
impl SecretStore {
    /// Store TOTP secret for a user
    pub fn set_totp_secret(&self, user_id: &str, secret: &str) -> Result<(), SecretStorageError> {
        self.set("apex_totp", user_id, secret)
    }

    /// Get TOTP secret for a user
    pub fn get_totp_secret(&self, user_id: &str) -> Result<String, SecretStorageError> {
        self.get("apex_totp", user_id)
    }

    /// Delete TOTP secret for a user
    pub fn delete_totp_secret(&self, user_id: &str) -> Result<(), SecretStorageError> {
        self.delete("apex_totp", user_id)
    }

    /// Check if user has TOTP configured
    pub fn has_totp(&self, user_id: &str) -> bool {
        self.exists("apex_totp", user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_secret_store_crud() {
        let temp_file = temp_dir().join(format!("test_secrets_{}.enc", uuid::Uuid::new_v4()));
        let store = SecretStore::new(temp_file.clone()).unwrap();

        // Set a secret
        store.set("test_service", "test_key", "test_value").unwrap();

        // Get the secret
        let value = store.get("test_service", "test_key").unwrap();
        assert_eq!(value, "test_value");

        // Update the secret
        store.set("test_service", "test_key", "new_value").unwrap();
        let value = store.get("test_service", "test_key").unwrap();
        assert_eq!(value, "new_value");

        // Delete the secret
        store.delete("test_service", "test_key").unwrap();
        assert!(store.get("test_service", "test_key").is_err());

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_secret_not_found() {
        let temp_file = temp_dir().join(format!("test_secrets_{}.enc", uuid::Uuid::new_v4()));
        let store = SecretStore::new(temp_file.clone()).unwrap();

        let result = store.get("nonexistent", "key");
        assert!(result.is_err());

        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_hmac_secret_generation() {
        let temp_file = temp_dir().join(format!("test_secrets_{}.enc", uuid::Uuid::new_v4()));
        let store = SecretStore::new(temp_file.clone()).unwrap();

        let secret1 = store.get_hmac_secret().unwrap();
        let secret2 = store.get_hmac_secret().unwrap();

        // Same secret should be returned (not regenerated)
        assert_eq!(secret1, secret2);

        // Should be valid hex
        assert!(hex::decode(&secret1).is_ok());

        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_totp_secret_storage() {
        let temp_file = temp_dir().join(format!("test_secrets_{}.enc", uuid::Uuid::new_v4()));
        let store = SecretStore::new(temp_file.clone()).unwrap();

        let user_id = "test_user";
        let totp_secret = "JBSWY3DPEHPK3PXP";

        // Store TOTP secret
        store.set_totp_secret(user_id, totp_secret).unwrap();

        // Check exists
        assert!(store.has_totp(user_id));

        // Retrieve
        let retrieved = store.get_totp_secret(user_id).unwrap();
        assert_eq!(retrieved, totp_secret);

        // Delete
        store.delete_totp_secret(user_id).unwrap();
        assert!(!store.has_totp(user_id));

        let _ = std::fs::remove_file(temp_file);
    }
}
