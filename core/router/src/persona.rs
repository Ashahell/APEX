//! Persona System - Persona management for APEX
//!
//! Feature 2: Persona Assembly
//! Bundles system prompt + voice settings + toolset + model config into swappable personas.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::unified_config::persona_constants::*;

/// Type of prompt piece
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PromptPieceType {
    System,
    Location,
    Emotion,
    Context,
    Custom,
}

impl Default for PromptPieceType {
    fn default() -> Self {
        PromptPieceType::System
    }
}

/// A single prompt piece that can be swapped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPiece {
    /// Type of piece
    pub piece_type: PromptPieceType,
    /// Content of the piece
    pub content: String,
    /// Order in the assembled prompt
    pub order: usize,
    /// Whether this piece is active
    pub enabled: bool,
}

impl PromptPiece {
    pub fn new(piece_type: PromptPieceType, content: String, order: usize) -> Self {
        Self {
            piece_type,
            content,
            order,
            enabled: true,
        }
    }
}

/// Voice configuration for a persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// TTS engine to use
    pub tts_engine: Option<String>,
    /// Voice ID
    pub voice_id: Option<String>,
    /// Speech speed (0.5 - 2.0)
    pub speed: Option<f32>,
    /// Pitch adjustment
    pub pitch: Option<f32>,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            tts_engine: None,
            voice_id: None,
            speed: Some(1.0),
            pitch: Some(0.0),
        }
    }
}

/// Model configuration for a persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// LLM provider
    pub provider: Option<String>,
    /// Model name
    pub model: Option<String>,
    /// Temperature (0.0 - 2.0)
    pub temperature: Option<f32>,
    /// Max tokens
    pub max_tokens: Option<u32>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: None,
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(4096),
        }
    }
}

/// A complete persona definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Prompt pieces
    pub prompt_pieces: Vec<PromptPiece>,
    /// Tools available to this persona
    pub tools: Vec<String>,
    /// Voice configuration
    pub voice_config: VoiceConfig,
    /// Model configuration
    pub model_config: ModelConfig,
    /// Is this the active persona
    pub is_active: bool,
    /// Created timestamp
    pub created_at: i64,
    /// Updated timestamp
    pub updated_at: i64,
}

impl Persona {
    /// Create a new persona with defaults
    pub fn new(name: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Ulid::new().to_string(),
            name,
            description: None,
            prompt_pieces: vec![PromptPiece::new(
                PromptPieceType::System,
                "You are a helpful AI assistant.".to_string(),
                0,
            )],
            tools: vec![],
            voice_config: VoiceConfig::default(),
            model_config: ModelConfig::default(),
            is_active: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Assemble the full system prompt from pieces
    pub fn assemble_prompt(&self) -> String {
        let parts: Vec<String> = self
            .prompt_pieces
            .iter()
            .filter(|p| p.enabled)
            .map(|p| p.content.clone())
            .collect();
        parts.join("\n\n")
    }

    /// Validate the persona
    pub fn validate(&self) -> Result<(), PersonaError> {
        // Check name
        if self.name.is_empty() {
            return Err(PersonaError::InvalidName(
                "Name cannot be empty".to_string(),
            ));
        }
        if self.name.len() > MAX_NAME_LENGTH {
            return Err(PersonaError::InvalidName(format!(
                "Name exceeds maximum length of {} characters",
                MAX_NAME_LENGTH
            )));
        }

        // Check description
        if let Some(ref desc) = self.description {
            if desc.len() > MAX_DESCRIPTION_LENGTH {
                return Err(PersonaError::InvalidDescription(format!(
                    "Description exceeds maximum length of {} characters",
                    MAX_DESCRIPTION_LENGTH
                )));
            }
        }

        // Check pieces
        if self.prompt_pieces.len() > MAX_PROMPT_PIECES {
            return Err(PersonaError::TooManyPieces(format!(
                "Maximum {} prompt pieces allowed",
                MAX_PROMPT_PIECES
            )));
        }

        // Check tools
        if self.tools.len() > MAX_TOOLS_PER_PERSONA {
            return Err(PersonaError::TooManyTools(format!(
                "Maximum {} tools allowed per persona",
                MAX_TOOLS_PER_PERSONA
            )));
        }

        Ok(())
    }

    /// Add a prompt piece
    pub fn add_piece(&mut self, piece: PromptPiece) -> Result<(), PersonaError> {
        if self.prompt_pieces.len() >= MAX_PROMPT_PIECES {
            return Err(PersonaError::TooManyPieces(format!(
                "Maximum {} prompt pieces allowed",
                MAX_PROMPT_PIECES
            )));
        }
        self.prompt_pieces.push(piece);
        self.updated_at = Utc::now().timestamp();
        Ok(())
    }

    /// Add a tool
    pub fn add_tool(&mut self, tool: String) -> Result<(), PersonaError> {
        if self.tools.len() >= MAX_TOOLS_PER_PERSONA {
            return Err(PersonaError::TooManyTools(format!(
                "Maximum {} tools allowed per persona",
                MAX_TOOLS_PER_PERSONA
            )));
        }
        if !self.tools.contains(&tool) {
            self.tools.push(tool);
            self.updated_at = Utc::now().timestamp();
        }
        Ok(())
    }

    /// Remove a tool
    pub fn remove_tool(&mut self, tool: &str) {
        self.tools.retain(|t| t != tool);
        self.updated_at = Utc::now().timestamp();
    }
}

/// Persona errors
#[derive(Debug, thiserror::Error)]
pub enum PersonaError {
    #[error("Invalid name: {0}")]
    InvalidName(String),

    #[error("Invalid description: {0}")]
    InvalidDescription(String),

    #[error("Too many prompt pieces: {0}")]
    TooManyPieces(String),

    #[error("Too many tools: {0}")]
    TooManyTools(String),

    #[error("Persona not found: {0}")]
    NotFound(String),

    #[error("Duplicate persona name: {0}")]
    DuplicateName(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Persona manager - handles persistence and operations
pub struct PersonaManager;

impl PersonaManager {
    /// Create default persona
    pub fn create_default() -> Persona {
        let mut persona = Persona::new(DEFAULT_PERSONA_NAME.to_string());
        persona.description = Some("Default APEX persona".to_string());
        persona.is_active = true;
        persona
    }

    /// Validate persona name (alphanumeric, dash, underscore, space)
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= MAX_NAME_LENGTH
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
    }

    /// Get default prompt piece types
    pub fn get_piece_types() -> Vec<&'static str> {
        PROMPT_PIECE_TYPES.iter().copied().collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_creation() {
        let persona = Persona::new("Test Persona".to_string());
        assert_eq!(persona.name, "Test Persona");
        assert!(!persona.is_active);
        assert_eq!(persona.prompt_pieces.len(), 1);
    }

    #[test]
    fn test_persona_validate_valid() {
        let persona = Persona::new("Valid".to_string());
        assert!(persona.validate().is_ok());
    }

    #[test]
    fn test_persona_validate_empty_name() {
        let persona = Persona::new("".to_string());
        assert!(persona.validate().is_err());
    }

    #[test]
    fn test_persona_validate_name_too_long() {
        let persona = Persona::new("a".repeat(MAX_NAME_LENGTH + 1));
        assert!(persona.validate().is_err());
    }

    #[test]
    fn test_assemble_prompt() {
        let mut persona = Persona::new("Test".to_string());
        persona.prompt_pieces.push(PromptPiece::new(
            PromptPieceType::Location,
            "You are in New York.".to_string(),
            1,
        ));
        persona.prompt_pieces.push(PromptPiece::new(
            PromptPieceType::Emotion,
            "You are happy.".to_string(),
            2,
        ));

        let prompt = persona.assemble_prompt();
        assert!(prompt.contains("You are in New York"));
        assert!(prompt.contains("You are happy"));
    }

    #[test]
    fn test_add_tool() {
        let mut persona = Persona::new("Test".to_string());
        persona.add_tool("shell.execute".to_string()).unwrap();
        assert!(persona.tools.contains(&"shell.execute".to_string()));
    }

    #[test]
    fn test_remove_tool() {
        let mut persona = Persona::new("Test".to_string());
        persona.add_tool("shell.execute".to_string()).unwrap();
        persona.remove_tool("shell.execute");
        assert!(!persona.tools.contains(&"shell.execute".to_string()));
    }

    #[test]
    fn test_valid_name() {
        assert!(PersonaManager::is_valid_name("Test Persona"));
        assert!(PersonaManager::is_valid_name("my-persona_123"));
        assert!(!PersonaManager::is_valid_name(""));
        assert!(!PersonaManager::is_valid_name("test@persona"));
    }

    #[test]
    fn test_default_persona() {
        let default = PersonaManager::create_default();
        assert_eq!(default.name, DEFAULT_PERSONA_NAME);
        assert!(default.is_active);
    }
}
