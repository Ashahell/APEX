use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub task_id: String,
    pub tier: PermissionTier,
    pub allowed_skills: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub max_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionTier {
    T0,
    T1,
    T2,
    T3,
}

impl PermissionTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionTier::T0 => "T0",
            PermissionTier::T1 => "T1",
            PermissionTier::T2 => "T2",
            PermissionTier::T3 => "T3",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "t0" => Some(PermissionTier::T0),
            "t1" => Some(PermissionTier::T1),
            "t2" => Some(PermissionTier::T2),
            "t3" => Some(PermissionTier::T3),
            _ => None,
        }
    }
}

impl CapabilityToken {
    pub fn new(
        task_id: &str,
        tier: PermissionTier,
        allowed_skills: Vec<String>,
        allowed_domains: Vec<String>,
        max_cost_usd: f64,
        expires_in_seconds: i64,
    ) -> Self {
        Self {
            task_id: task_id.to_string(),
            tier,
            allowed_skills,
            allowed_domains,
            expires_at: Utc::now() + Duration::seconds(expires_in_seconds),
            max_cost_usd,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn can_access_skill(&self, skill_name: &str) -> bool {
        self.allowed_skills
            .iter()
            .any(|s| s == skill_name || s == "*")
    }

    pub fn can_access_domain(&self, domain: &str) -> bool {
        self.allowed_domains.iter().any(|d| d == domain || d == "*")
    }

    pub fn encode(&self) -> String {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let json = serde_json::to_string(self).unwrap_or_default();
        STANDARD.encode(json.as_bytes())
    }

    pub fn decode(token: &str) -> Option<Self> {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let bytes = STANDARD.decode(token).ok()?;
        let json = String::from_utf8(bytes).ok()?;
        serde_json::from_str(&json).ok()
    }
}
