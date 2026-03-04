use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub struct ConstitutionManager {
    protected_keys: Vec<String>,
}

impl ConstitutionManager {
    pub fn new() -> Self {
        Self {
            protected_keys: vec![
                "CONSTITUTION_VERSION".to_string(),
                "IMMUTABLE_VALUES".to_string(),
                "EMERGENCY_CONTACT".to_string(),
                "SELF_DESTRUCT_IF".to_string(),
            ],
        }
    }

    pub fn compute_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("sha256:{}", hex::encode(result))
    }

    pub fn verify_constitution(&self, content: &str, expected_hash: &str) -> bool {
        let computed = self.compute_hash(content);
        computed == expected_hash
    }

    pub fn is_protected_key(&self, key: &str) -> bool {
        self.protected_keys.iter().any(|k| key.contains(k))
    }

    pub fn check_modification_allowed(
        &self,
        key: &str,
        tier: &str,
    ) -> Result<(), ConstitutionError> {
        if self.is_protected_key(key) {
            match tier {
                "T3" => Err(ConstitutionError::ProtectedModification(key.to_string())),
                _ => Err(ConstitutionError::InsufficientPermission(tier.to_string())),
            }
        } else {
            Ok(())
        }
    }

    pub fn extract_constitution_section(content: &str) -> Option<HashMap<String, String>> {
        let mut in_constitution = false;
        let mut vars = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("# CONSTITUTION") || line.contains("CONSTITUTION") {
                in_constitution = true;
                continue;
            }

            if in_constitution && line.is_empty() {
                break;
            }

            if in_constitution && line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    vars.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                }
            }
        }

        if vars.is_empty() {
            None
        } else {
            Some(vars)
        }
    }
}

impl Default for ConstitutionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConstitutionError {
    #[error("Protected key modification: {0}")]
    ProtectedModification(String),

    #[error("Insufficient permission: {0} required for protected keys")]
    InsufficientPermission(String),

    #[error("Constitution hash mismatch")]
    HashMismatch,

    #[error("Invalid constitution format: {0}")]
    InvalidFormat(String),
}

pub struct ModificationRequest {
    pub key: String,
    pub old_value: String,
    pub new_value: String,
    pub requester_tier: String,
    pub timestamp: String,
}

impl ModificationRequest {
    pub fn requires_t3(&self) -> bool {
        ConstitutionManager::new().is_protected_key(&self.key)
    }

    pub fn authorize(&self) -> Result<(), ConstitutionError> {
        if self.requires_t3() && self.requester_tier != "T3" {
            return Err(ConstitutionError::InsufficientPermission(
                "T3 required for protected keys".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let manager = ConstitutionManager::new();
        let hash = manager.compute_hash("test content");
        assert!(hash.starts_with("sha256:"));
    }

    #[test]
    fn test_is_protected_key() {
        let manager = ConstitutionManager::new();
        assert!(manager.is_protected_key("CONSTITUTION_VERSION"));
        assert!(manager.is_protected_key("IMMUTABLE_VALUES"));
        assert!(!manager.is_protected_key("name"));
    }

    #[test]
    fn test_modification_requires_t3() {
        let request = ModificationRequest {
            key: "CONSTITUTION_VERSION".to_string(),
            old_value: "1.0".to_string(),
            new_value: "2.0".to_string(),
            requester_tier: "T2".to_string(),
            timestamp: "2026-03-03".to_string(),
        };

        assert!(request.requires_t3());
        assert!(request.authorize().is_err());
    }
}
