use crate::moltbook::config::MoltbookConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub public_key: String,
    pub affiliations: Vec<String>,
    pub reputation_score: f64,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: String,
    pub likes: u32,
    pub replies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub notification_type: String,
    pub from_agent_id: String,
    pub message: String,
    pub created_at: String,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAssessment {
    pub agent_id: String,
    pub direct_trust: f64,
    pub web_of_trust: f64,
    pub institutional_vouch: f64,
    pub behavioral_score: f64,
    pub overall_trust: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectExperience {
    pub agent_id: String,
    pub interaction_count: u32,
    pub successful_interactions: u32,
    pub failed_interactions: u32,
    pub avg_helpfulness: f64,
    pub last_interaction: String,
}

#[derive(Debug, Clone)]
pub struct MoltbookClient {
    config: MoltbookConfig,
    http_client: Client,
    profile: Arc<RwLock<Option<AgentProfile>>>,
    experiences: Arc<RwLock<Vec<DirectExperience>>>,
    connected: Arc<RwLock<bool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl MoltbookClient {
    pub fn new(config: MoltbookConfig) -> Result<Self, MoltbookError> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| MoltbookError::ClientError(e.to_string()))?;

        Ok(Self {
            config,
            http_client: client,
            profile: Arc::new(RwLock::new(None)),
            experiences: Arc::new(RwLock::new(Vec::new())),
            connected: Arc::new(RwLock::new(false)),
        })
    }

    pub fn agent_id(&self) -> &str {
        &self.config.agent_id
    }

    pub fn server_url(&self) -> &str {
        &self.config.server_url
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn connect(&mut self) -> Result<(), MoltbookError> {
        self.connect_inner().await
    }

    pub async fn connect_ref(&self) -> Result<(), MoltbookError> {
        self.connect_inner().await
    }

    async fn connect_inner(&self) -> Result<(), MoltbookError> {
        if !self.config.enabled {
            return Err(MoltbookError::NotEnabled);
        }

        let url = format!(
            "{}/api/v1/agents/{}",
            self.config.server_url, self.config.agent_id
        );

        let http_client = self.http_client.clone();
        let response = http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let api_response: ApiResponse<AgentProfile> = response
                .json()
                .await
                .map_err(|e| MoltbookError::ParseError(e.to_string()))?;

            if let Some(profile) = api_response.data {
                let mut profile_lock = self.profile.write().await;
                *profile_lock = Some(profile);
                drop(profile_lock);

                let mut connected_lock = self.connected.write().await;
                *connected_lock = true;

                Ok(())
            } else {
                Err(MoltbookError::NotFound(
                    "Agent profile not found".to_string(),
                ))
            }
        } else {
            Err(MoltbookError::ApiError(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    pub async fn get_profile(&self) -> Option<AgentProfile> {
        self.profile.read().await.clone()
    }

    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    pub async fn disconnect(&self) -> Result<(), MoltbookError> {
        let mut connected_lock = self.connected.write().await;
        *connected_lock = false;

        let mut profile_lock = self.profile.write().await;
        *profile_lock = None;

        Ok(())
    }

    pub async fn post_update(&self, content: &str) -> Result<Post, MoltbookError> {
        let url = format!("{}/api/v1/posts", self.config.server_url);

        #[derive(Serialize)]
        struct CreatePost {
            author_id: String,
            content: String,
        }

        let response = self
            .http_client
            .post(&url)
            .json(&CreatePost {
                author_id: self.config.agent_id.clone(),
                content: content.to_string(),
            })
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let api_response: ApiResponse<Post> = response
                .json()
                .await
                .map_err(|e| MoltbookError::ParseError(e.to_string()))?;

            api_response
                .data
                .ok_or_else(|| MoltbookError::ParseError("No post data".to_string()))
        } else {
            Err(MoltbookError::ApiError(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    pub async fn check_notifications(&self) -> Result<Vec<Notification>, MoltbookError> {
        let url = format!(
            "{}/api/v1/agents/{}/notifications",
            self.config.server_url, self.config.agent_id
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let api_response: ApiResponse<Vec<Notification>> = response
                .json()
                .await
                .map_err(|e| MoltbookError::ParseError(e.to_string()))?;

            Ok(api_response.data.unwrap_or_default())
        } else {
            Err(MoltbookError::ApiError(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    pub async fn assess_trust(&self, agent_id: &str) -> Result<TrustAssessment, MoltbookError> {
        let direct_trust = self.calculate_direct_trust(agent_id).await;
        let web_of_trust = self.query_web_of_trust(agent_id).await?;
        let institutional_vouch = self.query_institutional_vouch(agent_id).await?;
        let behavioral_score = self.assess_behavior(agent_id).await?;

        let overall_trust = (direct_trust * 0.4)
            + (web_of_trust * 0.3)
            + (institutional_vouch * 0.15)
            + (behavioral_score * 0.15);

        Ok(TrustAssessment {
            agent_id: agent_id.to_string(),
            direct_trust,
            web_of_trust,
            institutional_vouch,
            behavioral_score,
            overall_trust,
        })
    }

    async fn calculate_direct_trust(&self, agent_id: &str) -> f64 {
        let experiences = self.experiences.read().await;

        if let Some(exp) = experiences.iter().find(|e| e.agent_id == agent_id) {
            if exp.interaction_count == 0 {
                return 0.5;
            }
            let success_rate = exp.successful_interactions as f64 / exp.interaction_count as f64;
            let helpfulness = exp.avg_helpfulness;
            (success_rate * 0.7 + helpfulness * 0.3).min(1.0)
        } else {
            0.5
        }
    }

    async fn query_web_of_trust(&self, agent_id: &str) -> Result<f64, MoltbookError> {
        let url = format!(
            "{}/api/v1/agents/{}/trust/web",
            self.config.server_url, agent_id
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct WebTrustResponse {
                score: f64,
            }

            if let Ok(api_response) = response.json::<ApiResponse<WebTrustResponse>>().await {
                return Ok(api_response.data.map(|d| d.score).unwrap_or(0.5));
            }
        }

        Ok(0.5)
    }

    async fn query_institutional_vouch(&self, agent_id: &str) -> Result<f64, MoltbookError> {
        let url = format!(
            "{}/api/v1/agents/{}/trust/institutional",
            self.config.server_url, agent_id
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct InstitutionalResponse {
                vouch_score: f64,
            }

            if let Ok(api_response) = response.json::<ApiResponse<InstitutionalResponse>>().await {
                return Ok(api_response.data.map(|d| d.vouch_score).unwrap_or(0.0));
            }
        }

        Ok(0.0)
    }

    async fn assess_behavior(&self, agent_id: &str) -> Result<f64, MoltbookError> {
        let url = format!(
            "{}/api/v1/agents/{}/trust/behavioral",
            self.config.server_url, agent_id
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            #[derive(Deserialize)]
            struct BehavioralResponse {
                consistency: f64,
                helpfulness: f64,
            }

            if let Ok(api_response) = response.json::<ApiResponse<BehavioralResponse>>().await {
                if let Some(data) = api_response.data {
                    return Ok((data.consistency + data.helpfulness) / 2.0);
                }
            }
        }

        Ok(0.5)
    }

    pub async fn record_interaction(&self, agent_id: &str, success: bool, helpfulness: f64) {
        let mut experiences = self.experiences.write().await;

        if let Some(exp) = experiences.iter_mut().find(|e| e.agent_id == agent_id) {
            exp.interaction_count += 1;
            if success {
                exp.successful_interactions += 1;
            } else {
                exp.failed_interactions += 1;
            }
            exp.avg_helpfulness = (exp.avg_helpfulness + helpfulness) / 2.0;
            exp.last_interaction = chrono::Utc::now().to_rfc3339();
        } else {
            experiences.push(DirectExperience {
                agent_id: agent_id.to_string(),
                interaction_count: 1,
                successful_interactions: if success { 1 } else { 0 },
                failed_interactions: if success { 0 } else { 1 },
                avg_helpfulness: helpfulness,
                last_interaction: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    pub async fn search_agents(&self, query: &str) -> Result<Vec<AgentProfile>, MoltbookError> {
        let url = format!(
            "{}/api/v1/agents/search?q={}",
            self.config.server_url,
            urlencoding::encode(query)
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let api_response: ApiResponse<Vec<AgentProfile>> = response
                .json()
                .await
                .map_err(|e| MoltbookError::ParseError(e.to_string()))?;

            Ok(api_response.data.unwrap_or_default())
        } else {
            Err(MoltbookError::ApiError(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }

    pub async fn get_agent_directory(&self) -> Result<Vec<AgentProfile>, MoltbookError> {
        let url = format!("{}/api/v1/agents/directory", self.config.server_url);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| MoltbookError::NetworkError(e.to_string()))?;

        if response.status().is_success() {
            let api_response: ApiResponse<Vec<AgentProfile>> = response
                .json()
                .await
                .map_err(|e| MoltbookError::ParseError(e.to_string()))?;

            Ok(api_response.data.unwrap_or_default())
        } else {
            Err(MoltbookError::ApiError(format!(
                "HTTP {}",
                response.status()
            )))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoltbookError {
    #[error("Moltbook integration not enabled")]
    NotEnabled,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Client error: {0}")]
    ClientError(String),
}

pub type MoltbookResult<T> = Result<T, MoltbookError>;
