//! Skill Signer - Cryptographic signing for skills/plugins
//!
//! Feature 5: Plugin Signing (ed25519 verification)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::unified_config::signing_constants::*;

/// Signature status for a skill
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SignatureStatus {
    /// Signature is valid
    Valid,
    /// Signature is invalid/tampered
    Invalid,
    /// Skill is not signed
    Unsigned,
    /// Signature has expired
    Expired,
}

/// Skill signature metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSignature {
    /// The skill name
    pub skill_name: String,
    /// Signature data (hex encoded)
    pub signature: String,
    /// When the signature was created
    pub signed_at: i64,
    /// When the signature expires
    pub expires_at: i64,
    /// Public key that signed this
    pub signer_public_key: String,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether the signature is valid
    pub valid: bool,
    /// Status of the signature
    pub status: SignatureStatus,
    /// Error message if invalid
    pub error: Option<String>,
    /// Skill name
    pub skill_name: String,
}

/// Key pair for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKeyPair {
    /// Public key (hex encoded)
    pub public_key: String,
    /// Whether keys exist on disk
    pub keys_exist: bool,
}

/// Signer state
pub struct SkillSigner {
    keys_dir: PathBuf,
}

/// Calculate expiry timestamp
fn calculate_expiry() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    now + (SIGNATURE_EXPIRY_DAYS * 24 * 60 * 60)
}

/// Calculate hash of skill content (simplified - in production use SHA256)
fn calculate_skill_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

impl SkillSigner {
    /// Create a new skill signer
    pub fn new(keys_dir: PathBuf) -> Self {
        Self { keys_dir }
    }

    /// Get the keys directory path
    pub fn keys_dir(&self) -> &PathBuf {
        &self.keys_dir
    }

    /// Initialize keys if they don't exist
    /// In a real implementation, this would generate actual ed25519 keys
    /// For now, we simulate key generation
    pub fn initialize_keys(&self) -> Result<SigningKeyPair, String> {
        // In production: generate actual ed25519 keypair
        // For now, simulate with deterministic keys based on path
        let keys_exist = self.keys_dir.join(VERIFY_KEY_FILE).exists();

        let public_key = if keys_exist {
            std::fs::read_to_string(self.keys_dir.join(VERIFY_KEY_FILE))
                .map(|k| k.trim().to_string())
                .unwrap_or_else(|_| "simulated_key_abc123".to_string())
        } else {
            // Create keys directory if needed
            std::fs::create_dir_all(&self.keys_dir).map_err(|e| e.to_string())?;
            "simulated_key_abc123".to_string()
        };

        Ok(SigningKeyPair {
            public_key,
            keys_exist,
        })
    }

    /// Get the public verification key
    pub fn get_public_key(&self) -> Result<String, String> {
        let key_path = self.keys_dir.join(VERIFY_KEY_FILE);

        if key_path.exists() {
            std::fs::read_to_string(key_path)
                .map(|k| k.trim().to_string())
                .map_err(|e| e.to_string())
        } else {
            // Return simulated key for now
            Ok("simulated_verification_key_xyz789".to_string())
        }
    }

    /// Sign a skill's content
    pub fn sign_skill(&self, skill_name: &str, content: &str) -> Result<SkillSignature, String> {
        let content_hash = calculate_skill_hash(content);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // In production: actual ed25519 signature
        // For now: create deterministic "signature" based on content hash
        let signature = format!("ed25519_sig_{}_{}", content_hash, skill_name);

        let public_key = self.get_public_key()?;

        Ok(SkillSignature {
            skill_name: skill_name.to_string(),
            signature,
            signed_at: now,
            expires_at: calculate_expiry(),
            signer_public_key: public_key,
        })
    }

    /// Verify a skill's signature
    pub fn verify_signature(
        &self,
        skill_name: &str,
        content: &str,
        sig: &SkillSignature,
    ) -> VerificationResult {
        // Check if signature belongs to this skill
        if sig.skill_name != skill_name {
            return VerificationResult {
                valid: false,
                status: SignatureStatus::Invalid,
                error: Some("Signature does not match skill name".to_string()),
                skill_name: skill_name.to_string(),
            };
        }

        // Check expiry
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if sig.expires_at < now {
            return VerificationResult {
                valid: false,
                status: SignatureStatus::Expired,
                error: Some("Signature has expired".to_string()),
                skill_name: skill_name.to_string(),
            };
        }

        // Verify content hash
        let content_hash = calculate_skill_hash(content);
        let expected_sig = format!("ed25519_sig_{}_{}", content_hash, skill_name);

        if sig.signature == expected_sig {
            VerificationResult {
                valid: true,
                status: SignatureStatus::Valid,
                error: None,
                skill_name: skill_name.to_string(),
            }
        } else {
            VerificationResult {
                valid: false,
                status: SignatureStatus::Invalid,
                error: Some("Content hash mismatch - may be tampered".to_string()),
                skill_name: skill_name.to_string(),
            }
        }
    }

    /// Check signature status without full verification
    pub fn check_status(&self, sig: &Option<SkillSignature>) -> SignatureStatus {
        match sig {
            None => SignatureStatus::Unsigned,
            Some(s) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                if s.expires_at < now {
                    SignatureStatus::Expired
                } else {
                    // Assume valid if not expired (full verification would check content)
                    SignatureStatus::Valid
                }
            }
        }
    }
}

/// Default implementation
impl Default for SkillSigner {
    fn default() -> Self {
        Self::new(PathBuf::from(KEYS_DIR))
    }
}

/// Store for skill signatures
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SignatureStore {
    /// Signatures indexed by skill name
    pub signatures: HashMap<String, SkillSignature>,
}

impl SignatureStore {
    /// Create a new signature store
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
        }
    }

    /// Add or update a signature
    pub fn set_signature(&mut self, skill_name: String, signature: SkillSignature) {
        self.signatures.insert(skill_name, signature);
    }

    /// Get a signature
    pub fn get_signature(&self, skill_name: &str) -> Option<&SkillSignature> {
        self.signatures.get(skill_name)
    }

    /// Remove a signature
    pub fn remove_signature(&mut self, skill_name: &str) -> Option<SkillSignature> {
        self.signatures.remove(skill_name)
    }

    /// Get all skills that have signatures
    pub fn signed_skills(&self) -> Vec<&str> {
        self.signatures.keys().map(|k| k.as_str()).collect()
    }

    /// Get count of signed skills
    pub fn signed_count(&self) -> usize {
        self.signatures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_signer_creation() {
        let signer = SkillSigner::new(PathBuf::from("/tmp/test_keys"));
        assert_eq!(signer.keys_dir().to_string_lossy(), "/tmp/test_keys");
    }

    #[test]
    fn test_sign_and_verify() {
        let signer = SkillSigner::new(PathBuf::from("/tmp/test_keys"));

        let content = "skill content here";
        let sig = signer.sign_skill("test_skill", content).unwrap();

        assert_eq!(sig.skill_name, "test_skill");
        assert!(sig.signature.starts_with("ed25519_sig_"));

        let result = signer.verify_signature("test_skill", content, &sig);
        assert!(result.valid);
        assert_eq!(result.status, SignatureStatus::Valid);
    }

    #[test]
    fn test_verify_wrong_content() {
        let signer = SkillSigner::new(PathBuf::from("/tmp/test_keys"));

        let content = "original content";
        let sig = signer.sign_skill("test_skill", content).unwrap();

        let result = signer.verify_signature("test_skill", "tampered content", &sig);
        assert!(!result.valid);
        assert_eq!(result.status, SignatureStatus::Invalid);
    }

    #[test]
    fn test_signature_store() {
        let mut store = SignatureStore::new();

        let sig = SkillSignature {
            skill_name: "test".to_string(),
            signature: "sig_data".to_string(),
            signed_at: 1000,
            expires_at: 2000000000, // Far future
            signer_public_key: "key".to_string(),
        };

        store.set_signature("test".to_string(), sig.clone());

        assert_eq!(store.signed_count(), 1);
        assert!(store.get_signature("test").is_some());

        store.remove_signature("test");
        assert_eq!(store.signed_count(), 0);
    }

    #[test]
    fn test_check_status_unsigned() {
        let signer = SkillSigner::new(PathBuf::from("/tmp/test"));
        let status = signer.check_status(&None);
        assert_eq!(status, SignatureStatus::Unsigned);
    }

    #[test]
    fn test_get_public_key() {
        let signer = SkillSigner::new(PathBuf::from("/tmp/nonexistent"));
        let key = signer.get_public_key().unwrap();
        assert!(!key.is_empty());
    }
}
