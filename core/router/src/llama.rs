use crate::unified_config::AppConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct LlamaClient {
    client: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: i32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl LlamaClient {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
        }
    }

    pub fn from_env() -> Self {
        let config = AppConfig::global();
        Self::new(config.agent.llama_url, config.agent.llama_model)
    }

    pub async fn chat(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 512,
        };

        let url = format!("{}/v1/chat/completions", self.base_url);

        tracing::debug!(url = %url, model = %self.model, "Calling llama-server");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to llama-server: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::error!(status = %status, body = %body, "LLM request failed");
            return Err(format!("LLM request failed: {} - {}", status, body));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No response from LLM".to_string())
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, String> {
        let request = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.7,
                "num_predict": 512
            }
        });

        let url = format!("{}/v1/completions", self.base_url);

        tracing::debug!(url = %url, "Calling llama-server for completion");

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to llama-server: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(format!("LLM request failed: {}", status));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        json["choices"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|c| c["text"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No response from LLM".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llama_client_from_env() {
        unsafe {
            std::env::set_var("LLAMA_SERVER_URL", "http://localhost:1234");
            std::env::set_var("LLAMA_MODEL", "test-model");
        }

        let client = LlamaClient::from_env();
        assert_eq!(client.base_url, "http://localhost:1234");
        assert_eq!(client.model, "test-model");

        unsafe {
            std::env::remove_var("LLAMA_SERVER_URL");
            std::env::remove_var("LLAMA_MODEL");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_llama_server_connectivity() {
        let config = AppConfig::global();
        let client = LlamaClient::new(config.agent.llama_url, config.agent.llama_model);

        let result = client.chat("You are a helpful assistant.", "Say 'hello' in one word.").await;
        
        match result {
            Ok(response) => {
                println!("LLM response: {}", response);
                assert!(!response.is_empty(), "Response should not be empty");
            }
            Err(e) => {
                panic!("Failed to connect to llama-server: {}. Make sure llama-server is running on port 8080", e);
            }
        }
    }
}
