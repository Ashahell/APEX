//! Encrypted Narrative Storage: Provides encryption for sensitive narrative data
//!
//! This module wraps narrative entries with AES-256-GCM encryption
//! to protect sensitive information like task decisions, reflections, and learnings.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Size of AES-256 key
const KEY_SIZE: usize = 32;
/// Size of GCM nonce
const NONCE_SIZE: usize = 12;

/// Encrypted narrative entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedNarrativeEntry {
    /// Encrypted content (JSON serialized narrative)
    pub ciphertext: Vec<u8>,
    /// Nonce used for encryption
    pub nonce: Vec<u8>,
    /// Original file path (for reference, not encrypted)
    pub original_path: PathBuf,
    /// Timestamp of encryption
    pub encrypted_at: String,
}

/// Encryption key management
pub struct NarrativeKeyManager {
    key: [u8; KEY_SIZE],
}

impl NarrativeKeyManager {
    /// Create a new key manager with a generated key
    pub fn new() -> Self {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        Self { key }
    }

    /// Create from an existing key
    pub fn from_key(key: [u8; KEY_SIZE]) -> Self {
        Self { key }
    }

    /// Derive key from a password/passphrase
    pub fn from_password(password: &str) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let result = hasher.finalize();

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&result[..KEY_SIZE.min(result.len())]);
        Self { key }
    }

    /// Get the key as bytes (for storage)
    pub fn key_bytes(&self) -> [u8; KEY_SIZE] {
        self.key
    }

    /// Encrypt narrative content
    pub fn encrypt(&self, plaintext: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let cipher = Aes256Gcm::new_from_slice(&self.key).expect("Valid key");

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .expect("Encryption success");

        (ciphertext, nonce_bytes.to_vec())
    }

    /// Decrypt narrative content
    pub fn decrypt(&self, ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| e.to_string())?;
        let nonce = Nonce::from_slice(nonce_bytes);

        cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())
    }
}

impl Default for NarrativeKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Narrative encryption configuration
#[derive(Debug, Clone)]
pub struct NarrativeEncryptionConfig {
    /// Whether encryption is enabled
    pub enabled: bool,
    /// Encryption key (if using deterministic key)
    pub key: Option<[u8; KEY_SIZE]>,
    /// Password to derive key from (alternative to key)
    pub password: Option<String>,
    /// Fields to encrypt (if empty, encrypt all sensitive fields)
    pub encrypt_fields: Vec<String>,
}

impl Default for NarrativeEncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default, enable via config
            key: None,
            password: None,
            encrypt_fields: vec![
                "reflection".to_string(),
                "decision".to_string(),
                "lesson".to_string(),
                "context".to_string(),
            ],
        }
    }
}

/// Sensitive fields in narrative entries that should be encrypted
pub const SENSITIVE_FIELDS: &[&str] = &[
    "reflection",
    "decision",
    "lesson",
    "context",
    "error",
    "stack_trace",
];

/// Check if a field name is considered sensitive
pub fn is_sensitive_field(field_name: &str) -> bool {
    SENSITIVE_FIELDS
        .iter()
        .any(|&s| field_name.eq_ignore_ascii_case(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let manager = NarrativeKeyManager::new();

        let plaintext = b"Important reflection: Always validate input before execution";
        let (ciphertext, nonce) = manager.encrypt(plaintext);

        assert_ne!(plaintext.to_vec(), ciphertext);

        let decrypted = manager.decrypt(&ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_key_from_password() {
        let manager = NarrativeKeyManager::from_password("my-secret-password");

        let plaintext = b"Test content";
        let (ciphertext, nonce) = manager.encrypt(plaintext);

        let decrypted = manager.decrypt(&ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_sensitive_field_detection() {
        assert!(is_sensitive_field("reflection"));
        assert!(is_sensitive_field("DECISION"));
        assert!(is_sensitive_field("Lesson"));
        assert!(!is_sensitive_field("timestamp"));
        assert!(!is_sensitive_field("task_id"));
    }
}
