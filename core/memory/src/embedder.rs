use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum EmbeddingProvider {
    Local { url: String, model: String },
    OpenAI { api_key: String, model: String },
}

#[derive(Debug, Clone)]
pub struct Embedder {
    provider: EmbeddingProvider,
    client: Client,
    dim: usize,
}

#[derive(Error, Debug)]
pub enum EmbedError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Embedding API error: {0}")]
    Api(String),
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct LocalEmbeddingResponse {
    data: Vec<LocalEmbeddingData>,
}

#[derive(Deserialize)]
struct LocalEmbeddingData {
    embedding: Vec<f32>,
}

impl Embedder {
    pub fn new(provider: EmbeddingProvider, dim: usize) -> Self {
        Self {
            provider,
            client: Client::new(),
            dim,
        }
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        match &self.provider {
            EmbeddingProvider::Local { url, model } => {
                let prefixed = format!("search_document: {}", text);
                self.embed_local(url, model, &prefixed).await
            }
            EmbeddingProvider::OpenAI { api_key, model } => {
                self.embed_openai(api_key, model, text).await
            }
        }
    }

    pub async fn embed_query(&self, text: &str) -> Result<Vec<f32>, EmbedError> {
        let prefixed = match &self.provider {
            EmbeddingProvider::Local { .. } => format!("search_query: {}", text),
            EmbeddingProvider::OpenAI { .. } => text.to_string(),
        };
        self.embed(&prefixed).await
    }

    async fn embed_local(
        &self,
        url: &str,
        model: &str,
        text: &str,
    ) -> Result<Vec<f32>, EmbedError> {
        let request_body = serde_json::json!({
            "model": model,
            "input": text
        });

        let response = self.client.post(url).json(&request_body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbedError::Api(format!(
                "Local embedding failed: {} - {}",
                status, body
            )));
        }

        let response_data: LocalEmbeddingResponse = response.json().await?;

        response_data
            .data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| EmbedError::Api("No embedding returned".to_string()))
    }

    async fn embed_openai(
        &self,
        api_key: &str,
        model: &str,
        text: &str,
    ) -> Result<Vec<f32>, EmbedError> {
        let request_body = serde_json::json!({
            "model": model,
            "input": text
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbedError::Api(format!(
                "OpenAI embedding failed: {} - {}",
                status, body
            )));
        }

        let response_data: OpenAIEmbeddingResponse = response.json().await?;

        response_data
            .data
            .first()
            .map(|d| d.embedding.clone())
            .ok_or_else(|| EmbedError::Api("No embedding returned".to_string()))
    }

    pub async fn validate_dimension(&self, expected: usize) -> Result<(), EmbedError> {
        let test_vec = self.embed("dimension validation probe").await?;
        if test_vec.len() != expected {
            return Err(EmbedError::DimensionMismatch {
                expected,
                actual: test_vec.len(),
            });
        }
        Ok(())
    }

    pub fn dimension(&self) -> usize {
        self.dim
    }
}

impl Default for Embedder {
    fn default() -> Self {
        Self::new(
            EmbeddingProvider::Local {
                url: "http://localhost:8081/v1/embeddings".to_string(),
                model: "nomic-embed-text-v1".to_string(),
            },
            768,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedder_default_config() {
        let embedder = Embedder::default();
        match &embedder.provider {
            EmbeddingProvider::Local { url, model } => {
                assert_eq!(url, "http://localhost:8081/v1/embeddings");
                assert_eq!(model, "nomic-embed-text-v1");
            }
            _ => panic!("Expected local provider"),
        }
        assert_eq!(embedder.dim, 768);
    }

    #[test]
    fn test_embedder_openai_config() {
        let embedder = Embedder::new(
            EmbeddingProvider::OpenAI {
                api_key: "test-key".to_string(),
                model: "text-embedding-3-small".to_string(),
            },
            1536,
        );
        match &embedder.provider {
            EmbeddingProvider::OpenAI { api_key, model } => {
                assert_eq!(api_key, "test-key");
                assert_eq!(model, "text-embedding-3-small");
            }
            _ => panic!("Expected OpenAI provider"),
        }
        assert_eq!(embedder.dim, 1536);
    }
}
