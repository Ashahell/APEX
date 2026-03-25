use serde::{Deserialize, Serialize};

use crate::computer_use::actions::ComputerAction;
use crate::computer_use::screenshot::CapturedScreenshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VLMProvider {
    Claude,
    GPT4V,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLMConfig {
    pub provider: VLMProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLMResponse {
    pub reasoning: String,
    pub actions: Vec<ComputerAction>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLMError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLMController {
    pub config: VLMConfig,
}

impl VLMController {
    pub fn new() -> Self {
        let cfg = VLMConfig {
            provider: VLMProvider::Local,
            model: String::from("default"),
            api_key: None,
            api_endpoint: None,
            max_tokens: 512,
            temperature: 0.0,
        };
        VLMController { config: cfg }
    }

    pub async fn analyze_and_plan(
        &self,
        _screenshot: &CapturedScreenshot,
        _task: &str,
        _context: &crate::computer_use::orchestrator::ExecutionContext,
    ) -> Result<VLMResponse, VLMError> {
        // Minimal stub: no actions suggested yet
        Ok(VLMResponse {
            reasoning: String::from("stub"),
            actions: Vec::new(),
            confidence: 0.0,
        })
    }
}
