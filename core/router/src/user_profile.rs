//! User Profile Modeling (Hermes-style User Preferences)
//!
//! Tracks and models user preferences for personalized agent interactions.
//!
//! Features:
//! - Communication style preferences
//! - Skill usage patterns
//! - Task category preferences
//! - Response format preferences

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::unified_config::user_constants::{MAX_PREFERRED_CATEGORIES, MAX_PREFERRED_TOOLS};

/// User profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Communication style preference
    pub communication_style: CommunicationStyle,
    /// Preferred response verbosity
    pub verbosity: Verbosity,
    /// Preferred task categories (most used)
    pub preferred_categories: Vec<String>,
    /// Preferred tools (most used)
    pub preferred_tools: Vec<String>,
    /// Response format preference
    pub response_format: ResponseFormat,
    /// Whether to include reasoning in responses
    pub include_reasoning: bool,
    /// Preferred language
    pub language: String,
    /// Timezone for scheduling
    pub timezone: String,
}

/// Communication style preferences
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CommunicationStyle {
    Formal,
    Casual,
    Technical,
    Concise,
}

impl Default for CommunicationStyle {
    fn default() -> Self {
        CommunicationStyle::Casual
    }
}

impl CommunicationStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommunicationStyle::Formal => "formal",
            CommunicationStyle::Casual => "casual",
            CommunicationStyle::Technical => "technical",
            CommunicationStyle::Concise => "concise",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "formal" => CommunicationStyle::Formal,
            "technical" => CommunicationStyle::Technical,
            "concise" => CommunicationStyle::Concise,
            _ => CommunicationStyle::Casual,
        }
    }

    /// Get system prompt prefix for this style
    pub fn system_prompt_prefix(&self) -> &'static str {
        match self {
            CommunicationStyle::Formal => "Please respond in a formal and professional manner.",
            CommunicationStyle::Casual => {
                "Feel free to be conversational and friendly in responses."
            }
            CommunicationStyle::Technical => {
                "Provide detailed technical explanations with code examples when appropriate."
            }
            CommunicationStyle::Concise => {
                "Keep responses brief and to the point. Prioritize clarity over detail."
            }
        }
    }
}

/// Response verbosity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Verbosity {
    Brief,
    Normal,
    Detailed,
    Comprehensive,
}

impl Default for Verbosity {
    fn default() -> Self {
        Verbosity::Normal
    }
}

impl Verbosity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Verbosity::Brief => "brief",
            Verbosity::Normal => "normal",
            Verbosity::Detailed => "detailed",
            Verbosity::Comprehensive => "comprehensive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "brief" => Verbosity::Brief,
            "detailed" => Verbosity::Detailed,
            "comprehensive" => Verbosity::Comprehensive,
            _ => Verbosity::Normal,
        }
    }

    /// Get max response length hint for this verbosity
    pub fn max_tokens_hint(&self) -> usize {
        match self {
            Verbosity::Brief => 200,
            Verbosity::Normal => 500,
            Verbosity::Detailed => 1000,
            Verbosity::Comprehensive => 2000,
        }
    }
}

/// Response format preferences
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseFormat {
    PlainText,
    Markdown,
    Structured,
}

impl Default for ResponseFormat {
    fn default() -> Self {
        ResponseFormat::Markdown
    }
}

impl ResponseFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ResponseFormat::PlainText => "plain",
            ResponseFormat::Markdown => "markdown",
            ResponseFormat::Structured => "structured",
        }
    }
}

/// User profile manager
pub struct UserProfileManager {
    profile: Arc<RwLock<UserProfile>>,
    pool: Pool<Sqlite>,
}

impl UserProfileManager {
    /// Create a new user profile manager
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            profile: Arc::new(RwLock::new(UserProfile::default())),
            pool,
        }
    }

    /// Load profile from database
    pub async fn load(&self) -> Result<(), UserProfileError> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM config WHERE key = 'user_profile'")
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| UserProfileError::DatabaseError(e.to_string()))?;

        if let Some((value,)) = row {
            if let Ok(profile) = serde_json::from_str(&value) {
                let mut p = self.profile.write().await;
                *p = profile;
            }
        }

        Ok(())
    }

    /// Save profile to database
    pub async fn save(&self) -> Result<(), UserProfileError> {
        let profile = self.profile.read().await;
        let value = serde_json::to_string(&*profile)
            .map_err(|e| UserProfileError::SerializationError(e.to_string()))?;

        sqlx::query(
            "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES ('user_profile', ?, datetime('now'))"
        )
        .bind(&value)
        .execute(&self.pool)
        .await
        .map_err(|e| UserProfileError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get current profile
    pub async fn get_profile(&self) -> UserProfile {
        self.profile.read().await.clone()
    }

    /// Set the full profile (for API use)
    pub async fn set_profile(&self, new_profile: UserProfile) -> Result<(), UserProfileError> {
        {
            let mut profile = self.profile.write().await;
            *profile = new_profile;
        }
        self.save().await
    }

    /// Update communication style
    pub async fn set_communication_style(
        &self,
        style: CommunicationStyle,
    ) -> Result<(), UserProfileError> {
        {
            let mut profile = self.profile.write().await;
            profile.communication_style = style;
        }
        self.save().await
    }

    /// Update verbosity
    pub async fn set_verbosity(&self, verbosity: Verbosity) -> Result<(), UserProfileError> {
        {
            let mut profile = self.profile.write().await;
            profile.verbosity = verbosity;
        }
        self.save().await
    }

    /// Update response format
    pub async fn set_response_format(
        &self,
        format: ResponseFormat,
    ) -> Result<(), UserProfileError> {
        {
            let mut profile = self.profile.write().await;
            profile.response_format = format;
        }
        self.save().await
    }

    /// Toggle reasoning in responses
    pub async fn set_include_reasoning(&self, include: bool) -> Result<(), UserProfileError> {
        {
            let mut profile = self.profile.write().await;
            profile.include_reasoning = include;
        }
        self.save().await
    }

    /// Update language preference
    pub async fn set_language(&self, language: String) -> Result<(), UserProfileError> {
        {
            let mut profile = self.profile.write().await;
            profile.language = language;
        }
        self.save().await
    }

    /// Record tool usage for preference learning
    pub async fn record_tool_usage(&self, tool_name: &str) -> Result<(), UserProfileError> {
        // Update in-memory profile
        {
            let mut profile = self.profile.write().await;
            if !profile.preferred_tools.contains(&tool_name.to_string()) {
                profile.preferred_tools.insert(0, tool_name.to_string());
                if profile.preferred_tools.len() > MAX_PREFERRED_TOOLS {
                    profile.preferred_tools.pop();
                }
            }
        }

        // Also persist to database
        self.save().await
    }

    /// Record category usage for preference learning
    pub async fn record_category_usage(&self, category: &str) -> Result<(), UserProfileError> {
        // Update in-memory profile
        {
            let mut profile = self.profile.write().await;
            if !profile.preferred_categories.contains(&category.to_string()) {
                profile.preferred_categories.insert(0, category.to_string());
                if profile.preferred_categories.len() > MAX_PREFERRED_CATEGORIES {
                    profile.preferred_categories.pop();
                }
            }
        }

        // Also persist to database
        self.save().await
    }

    /// Get system prompt additions based on user profile
    pub async fn get_system_prompt_additions(&self) -> String {
        let profile = self.profile.read().await;

        let mut additions = Vec::new();

        // Communication style
        additions.push(
            profile
                .communication_style
                .system_prompt_prefix()
                .to_string(),
        );

        // Verbosity hint
        let verbosity_tokens = profile.verbosity.max_tokens_hint();
        additions.push(format!(
            "Keep responses around {} tokens unless more detail is necessary.",
            verbosity_tokens
        ));

        // Response format
        match profile.response_format {
            ResponseFormat::Markdown => {
                additions.push("Use markdown formatting for better readability.".to_string())
            }
            ResponseFormat::Structured => additions
                .push("Structure responses with clear headers and bullet points.".to_string()),
            ResponseFormat::PlainText => {
                additions.push("Use plain text without markdown formatting.".to_string())
            }
        }

        // Language
        if profile.language != "en" {
            additions.push(format!("Respond in {}.", profile.language));
        }

        // Reasoning
        if profile.include_reasoning {
            additions.push("Show your reasoning process when helpful.".to_string());
        }

        additions.join(" ")
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            communication_style: CommunicationStyle::default(),
            verbosity: Verbosity::default(),
            preferred_categories: Vec::new(),
            preferred_tools: Vec::new(),
            response_format: ResponseFormat::default(),
            include_reasoning: true,
            language: "en".to_string(),
            timezone: "UTC".to_string(),
        }
    }
}

/// User profile errors
#[derive(Debug, thiserror::Error)]
pub enum UserProfileError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_communication_style_system_prompt() {
        assert!(CommunicationStyle::Formal
            .system_prompt_prefix()
            .contains("formal"));
        assert!(CommunicationStyle::Casual
            .system_prompt_prefix()
            .contains("conversational"));
        assert!(CommunicationStyle::Technical
            .system_prompt_prefix()
            .contains("technical"));
        assert!(CommunicationStyle::Concise
            .system_prompt_prefix()
            .contains("brief"));
    }

    #[test]
    fn test_verbosity_token_hints() {
        assert!(Verbosity::Brief.max_tokens_hint() < Verbosity::Normal.max_tokens_hint());
        assert!(Verbosity::Detailed.max_tokens_hint() > Verbosity::Normal.max_tokens_hint());
        assert!(Verbosity::Comprehensive.max_tokens_hint() > Verbosity::Detailed.max_tokens_hint());
    }

    #[test]
    fn test_default_profile() {
        let profile = UserProfile::default();
        assert_eq!(profile.communication_style, CommunicationStyle::Casual);
        assert_eq!(profile.verbosity, Verbosity::Normal);
        assert_eq!(profile.response_format, ResponseFormat::Markdown);
        assert!(profile.include_reasoning);
        assert_eq!(profile.language, "en");
    }
}
