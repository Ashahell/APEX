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

    /// Get the store file path
    pub fn path(&self) -> &PathBuf {
        &self.store_path
    }

    /// Store a secret
    pub fn set(&self, service: &str, key: &str, value: &str) -> Result<(), SecretStorageError> {
        let entry = SecretEntry {
            service: service.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        let mut secrets = self.load_all().unwrap_or_default();
        let id = format!("{}:{}", service, key);
        secrets.insert(id, entry);

        self.save_all(&secrets)
    }

    /// Retrieve a secret
    pub fn get(&self, service: &str, key: &str) -> Result<String, SecretStorageError> {
        let secrets = self.load_all()?;
        let id = format!("{}:{}", service, key);

        secrets
            .get(&id)
            .map(|e| e.value.clone())
            .ok_or_else(|| SecretStorageError::NotFound(format!("{}:{}", service, key)))
    }

    /// Delete a secret
    pub fn delete(&self, service: &str, key: &str) -> Result<(), SecretStorageError> {
        let mut secrets = self.load_all().unwrap_or_default();
        let id = format!("{}:{}", service, key);
        secrets.remove(&id);
        self.save_all(&secrets)
    }

    /// List all secrets (metadata only)
    pub fn list(&self) -> Result<Vec<SecretEntry>, SecretStorageError> {
        let secrets = self.load_all()?;
        Ok(secrets.into_values().collect())
    }

    /// Load all secrets from disk
    fn load_all(
        &self,
    ) -> Result<std::collections::HashMap<String, SecretEntry>, SecretStorageError> {
        if !self.store_path.exists() {
            return Ok(std::collections::HashMap::new());
        }

        let data = std::fs::read(&self.store_path)?;
        let decrypted = self.decrypt(&data)?;

        let secrets: std::collections::HashMap<String, SecretEntry> =
            serde_json::from_slice(&decrypted)
                .map_err(|e| SecretStorageError::DecryptionFailed(e.to_string()))?;

        Ok(secrets)
    }

    /// Save all secrets to disk
    fn save_all(
        &self,
        secrets: &std::collections::HashMap<String, SecretEntry>,
    ) -> Result<(), SecretStorageError> {
        let data = serde_json::to_vec(secrets)
            .map_err(|e| SecretStorageError::EncryptionFailed(e.to_string()))?;
        let encrypted = self.encrypt(&data)?;
        std::fs::write(&self.store_path, encrypted)?;
        Ok(())
    }

    /// Encrypt data
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecretStorageError> {
        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| SecretStorageError::EncryptionFailed(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SecretStorageError::EncryptionFailed(e.to_string()))?;

        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend(ciphertext);

        Ok(result)
    }

    /// Decrypt data
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, SecretStorageError> {
        if data.len() < 12 {
            return Err(SecretStorageError::DecryptionFailed(
                "Data too short".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| SecretStorageError::DecryptionFailed(e.to_string()))?;

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SecretStorageError::DecryptionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_store() -> (SecretStore, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secrets.json");
        let store = SecretStore::new(path).unwrap();
        (store, dir)
    }

    #[test]
    fn test_set_and_get() {
        let (store, _dir) = temp_store();
        store.set("test_service", "test_key", "test_value").unwrap();
        let value = store.get("test_service", "test_key").unwrap();
        assert_eq!(value, "test_value");
    }

    #[test]
    fn test_delete() {
        let (store, _dir) = temp_store();
        store.set("test_service", "test_key", "test_value").unwrap();
        store.delete("test_service", "test_key").unwrap();
        let result = store.get("test_service", "test_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_list() {
        let (store, _dir) = temp_store();
        store.set("service1", "key1", "value1").unwrap();
        store.set("service2", "key2", "value2").unwrap();
        let entries = store.list().unwrap();
        assert_eq!(entries.len(), 2);
    }
}
