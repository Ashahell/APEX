use crate::unified_config::AppConfig;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct LlamaClient {
    client: Client,
    base_url: String,
    model: String,
    api_key: Option<String>,
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
    pub fn new(base_url: String, model: String, api_key: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model,
            api_key,
        }
    }

    pub fn from_env() -> Self {
        let config = AppConfig::global();
        // Get the API key from the default LLM if available
        let api_key = config.agent.llms.iter()
            .find(|l| l.id == config.agent.default_llm_id.as_deref().unwrap_or("default"))
            .and_then(|l| l.api_key.clone());
        Self::new(config.agent.llama_url, config.agent.llama_model, api_key)
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

        // Retry logic for rate limits (429)
        let max_retries = 3;
        let mut last_error = String::new();
        
        for attempt in 0..max_retries {
            if attempt > 0 {
                let delay = tokio::time::Duration::from_secs(2_u64.pow(attempt));
                tracing::info!(attempt = attempt + 1, delay_secs = delay.as_secs(), "Retrying LLM request after rate limit");
                tokio::time::sleep(delay).await;
            }

            let mut req = self.client.post(&url).json(&request);
            
            // Add API key if available
            if let Some(ref key) = self.api_key {
                tracing::debug!(has_api_key = true, key_length = key.len(), "Sending request with API key");
                req = req.header("Authorization", format!("Bearer {}", key));
            } else {
                tracing::warn!("No API key available for LLM request");
            }
            
            let response = req
                .send()
                .await
                .map_err(|e| format!("Failed to connect to llama-server: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                
                // Check for rate limit (429)
                if status.as_u16() == 429 && attempt < max_retries - 1 {
                    tracing::warn!(status = %status, attempt = attempt + 1, "Rate limited, will retry");
                    last_error = format!("{} - {}", status, body);
                    continue;
                }
                
                tracing::error!(status = %status, body = %body, "LLM request failed");
                return Err(format!("LLM request failed: {} - {}", status, body));
            }

            let chat_response: ChatResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

            return chat_response
                .choices
                .first()
                .map(|c| c.message.content.clone())
                .ok_or_else(|| "No response from LLM".to_string());
        }
        
        Err(format!("LLM rate limit exceeded after {} retries: {}", max_retries, last_error))
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
        std::env::set_var("LLAMA_SERVER_URL", "http://localhost:1234");
        std::env::set_var("LLAMA_MODEL", "test-model");

        let client = LlamaClient::from_env();
        assert_eq!(client.base_url, "http://localhost:1234");
        assert_eq!(client.model, "test-model");

        std::env::remove_var("LLAMA_SERVER_URL");
        std::env::remove_var("LLAMA_MODEL");
    }

    #[tokio::test]
    async fn test_llama_server_connectivity() {
        use std::net::TcpStream;
        
        // Check if llama-server is running on the expected port
        let config = AppConfig::global();
        let host = config.agent.llama_url.trim_start_matches("http://").trim_start_matches("https://");
        let port = host.split(':').nth(1).unwrap_or("80");
        
        if TcpStream::connect(format!("{}:{}", host.split(':').next().unwrap_or("localhost"), port)).is_err() {
            eprintln!("llama-server not running on {} - skipping test. Start llama-server and try again.", config.agent.llama_url);
            return;
        }

        let client = LlamaClient::new(config.agent.llama_url, config.agent.llama_model, None);

        let result = client.chat("You are a helpful assistant.", "Say 'hello' in one word.").await;
        
        match result {
            Ok(response) => {
                println!("LLM response: {}", response);
                assert!(!response.is_empty(), "Response should not be empty");
            }
            Err(e) => {
                eprintln!("LLM test failed: {}. Make sure llama-server is running on port 8080", e);
            }
        }
    }
}
