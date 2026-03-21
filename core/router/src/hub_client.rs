//! Skills Hub Client (Hermes-style Skills Hub)
//!
//! Integrates with external skill repositories and marketplaces.
//!
//! Features:
//! - Query skill hubs for discoverable skills
//! - Trust level verification for community skills
//! - Skill installation from hub

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::unified_config::hub_constants::*;

/// Hub skill entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub trust_level: TrustLevel,
    pub download_count: u64,
    pub rating: f32,
    pub tags: Vec<String>,
    pub source_url: String,
    pub verified: bool,
}

/// Trust level for hub skills
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    Unknown,
    Community,
    Trusted,
    Verified,
}

impl TrustLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustLevel::Verified => "verified",
            TrustLevel::Trusted => "trusted",
            TrustLevel::Community => "community",
            TrustLevel::Unknown => "unknown",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "verified" => TrustLevel::Verified,
            "trusted" => TrustLevel::Trusted,
            "community" => TrustLevel::Community,
            _ => TrustLevel::Unknown,
        }
    }
}

/// Hub client for querying skill marketplaces
pub struct HubClient {
    client: reqwest::Client,
    base_url: String,
}

impl HubClient {
    /// Create a new hub client
    pub fn new(base_url: Option<String>) -> Self {
        let url = base_url.unwrap_or_else(|| DEFAULT_HUB_URL.to_string());
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(HUB_REQUEST_TIMEOUT_SECS))
                .build()
                .unwrap_or_default(),
            base_url: url,
        }
    }
    
    /// Search skills in the hub
    pub async fn search_skills(&self, query: &str) -> Result<Vec<HubSkill>, HubError> {
        let url = format!("{}/skills/search?q={}", self.base_url, urlencoding::encode(query));
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(HubError::HubUnavailable {
                status: response.status().as_u16(),
            });
        }
        
        let skills: Vec<HubSkill> = response.json().await?;
        Ok(skills)
    }
    
    /// Get featured/trending skills
    pub async fn get_featured_skills(&self) -> Result<Vec<HubSkill>, HubError> {
        let url = format!("{}/skills/featured", self.base_url);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(HubError::HubUnavailable {
                status: response.status().as_u16(),
            });
        }
        
        let skills: Vec<HubSkill> = response.json().await?;
        Ok(skills)
    }
    
    /// Get skills by category
    pub async fn get_skills_by_category(&self, category: &str) -> Result<Vec<HubSkill>, HubError> {
        let url = format!("{}/skills/category/{}", self.base_url, urlencoding::encode(category));
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(HubError::HubUnavailable {
                status: response.status().as_u16(),
            });
        }
        
        let skills: Vec<HubSkill> = response.json().await?;
        Ok(skills)
    }
    
    /// Get skill details by ID
    pub async fn get_skill(&self, id: &str) -> Result<HubSkill, HubError> {
        let url = format!("{}/skills/{}", self.base_url, urlencoding::encode(id));
        
        let response = self.client.get(&url).send().await?;
        
        if response.status().as_u16() == 404 {
            return Err(HubError::SkillNotFound { id: id.to_string() });
        }
        
        if !response.status().is_success() {
            return Err(HubError::HubUnavailable {
                status: response.status().as_u16(),
            });
        }
        
        let skill: HubSkill = response.json().await?;
        Ok(skill)
    }
    
    /// Download skill content
    pub async fn download_skill(&self, id: &str) -> Result<String, HubError> {
        let skill = self.get_skill(id).await?;
        
        // Verify trust level before download
        if !Self::is_trustworthy(&skill.trust_level) {
            return Err(HubError::UntrustedSkill {
                id: id.to_string(),
                reason: format!("Trust level '{}' is below minimum '{}'", 
                    skill.trust_level.as_str(), 
                    MIN_TRUST_LEVEL.as_str()
                ),
            });
        }
        
        let url = format!("{}/skills/{}/download", self.base_url, urlencoding::encode(id));
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(HubError::DownloadFailed {
                id: id.to_string(),
                reason: format!("HTTP {}", response.status().as_u16()),
            });
        }
        
        let content = response.text().await?;
        
        // Security check: scan for dangerous patterns
        if Self::contains_dangerous_patterns(&content) {
            return Err(HubError::SecurityBlocked {
                id: id.to_string(),
                reason: "Skill content contains dangerous patterns".to_string(),
            });
        }
        
        Ok(content)
    }
    
    /// Check if trust level meets minimum requirement
    fn is_trustworthy(level: &TrustLevel) -> bool {
        let min_level = match MIN_TRUST_LEVEL {
            TrustLevel::Verified => 3,
            TrustLevel::Trusted => 2,
            TrustLevel::Community => 1,
            TrustLevel::Unknown => 0,
        };
        
        let level_value = match level {
            TrustLevel::Verified => 3,
            TrustLevel::Trusted => 2,
            TrustLevel::Community => 1,
            TrustLevel::Unknown => 0,
        };
        
        level_value >= min_level
    }
    
    /// Scan for dangerous patterns in skill content
    fn contains_dangerous_patterns(content: &str) -> bool {
        let dangerous = [
            "curl | sh",
            "rm -rf /",
            "eval ",
            "base64 -d",
            "wget .* | sh",
            "sudo ",
            "chmod 777",
            "curl -s | bash",
        ];
        
        let content_lower = content.to_lowercase();
        dangerous.iter().any(|p| content_lower.contains(p))
    }
}

/// Hub errors
#[derive(Debug, thiserror::Error)]
pub enum HubError {
    #[error("Hub is unavailable: HTTP {status}")]
    HubUnavailable { status: u16 },
    
    #[error("Skill not found: {id}")]
    SkillNotFound { id: String },
    
    #[error("Skill is not trusted: {id} - {reason}")]
    UntrustedSkill { id: String, reason: String },
    
    #[error("Download failed: {id} - {reason}")]
    DownloadFailed { id: String, reason: String },
    
    #[error("Skill blocked by security: {id} - {reason}")]
    SecurityBlocked { id: String, reason: String },
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Verified as u8 > TrustLevel::Trusted as u8);
        assert!(TrustLevel::Trusted as u8 > TrustLevel::Community as u8);
        assert!(TrustLevel::Community as u8 > TrustLevel::Unknown as u8);
    }
    
    #[test]
    fn test_trust_level_from_str() {
        assert_eq!(TrustLevel::from_str("verified"), TrustLevel::Verified);
        assert_eq!(TrustLevel::from_str("TRUSTED"), TrustLevel::Trusted);
        assert_eq!(TrustLevel::from_str("community"), TrustLevel::Community);
        assert_eq!(TrustLevel::from_str("unknown"), TrustLevel::Unknown);
        assert_eq!(TrustLevel::from_str("invalid"), TrustLevel::Unknown);
    }
    
    #[test]
    fn test_dangerous_patterns() {
        assert!(HubClient::contains_dangerous_patterns("curl | sh"));
        assert!(HubClient::contains_dangerous_patterns("rm -rf /"));
        assert!(HubClient::contains_dangerous_patterns("eval some_code"));
        
        assert!(!HubClient::contains_dangerous_patterns("curl https://example.com"));
        assert!(!HubClient::contains_dangerous_patterns("echo hello"));
    }
}
