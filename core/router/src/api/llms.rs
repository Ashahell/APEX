use axum::{
    extract::State,
    extract::Query,
    routing::{get, post, put, delete},
    Json, Router,
    extract::Path,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use crate::api::AppState;
use crate::unified_config::{AppConfig, LlmConfig, LlmProvider, GLOBAL_CONFIG};
use apex_memory::provider_repo::{
    ModelFallback, ProviderHealth, ProviderModel, ProviderPlugin, ProviderRepository,
    SessionFastMode,
};

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/llms", get(list_llms))
        .route("/api/v1/llms", post(add_llm))
        .route("/api/v1/llms/:id", get(get_llm))
        .route("/api/v1/llms/:id", put(update_llm))
        .route("/api/v1/llms/:id", delete(delete_llm))
        .route("/api/v1/llms/:id/test", post(test_llm))
        .route("/api/v1/llms/default", get(get_default_llm))
        .route("/api/v1/llms/default", put(set_default_llm))
        .route("/api/v1/llms/providers", get(list_providers))
        // Provider plugins (NEW)
        .route("/api/v1/llms/plugins", get(list_provider_plugins))
        .route("/api/v1/llms/plugins", post(create_provider_plugin))
        .route("/api/v1/llms/plugins/:id", get(get_provider_plugin))
        .route("/api/v1/llms/plugins/:id", put(update_provider_plugin))
        .route("/api/v1/llms/plugins/:id", delete(delete_provider_plugin))
        .route("/api/v1/llms/plugins/:id/models", get(list_provider_models))
        .route("/api/v1/llms/plugins/:id/health", get(get_provider_health_status))
        // Fast mode (NEW) - session-based only
        .route("/api/v1/llms/sessions/:session_id/fast-mode", get(get_session_fast_mode))
        .route("/api/v1/llms/sessions/:session_id/fast-mode", put(set_session_fast_mode))
        // Model fallbacks (NEW)
        .route("/api/v1/llms/fallbacks", get(list_model_fallbacks))
        .route("/api/v1/llms/fallbacks", post(add_model_fallback))
        .route("/api/v1/llms/fallbacks/:id", delete(delete_model_fallback))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub default_url: String,
    pub default_model: String,
    pub requires_api_key: bool,
    pub api_type: String,
}

pub fn get_provider_info() -> Vec<ProviderInfo> {
    vec![
        ProviderInfo {
            id: "local".to_string(),
            name: "Local (llama.cpp)".to_string(),
            default_url: "http://localhost:8080/v1".to_string(),
            default_model: "qwen3-4b".to_string(),
            requires_api_key: false,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            default_url: "http://localhost:11434/v1".to_string(),
            default_model: "llama3.1".to_string(),
            requires_api_key: false,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "vllm".to_string(),
            name: "vLLM".to_string(),
            default_url: "http://localhost:8000/v1".to_string(),
            default_model: "llama-3.1-8b".to_string(),
            requires_api_key: false,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "lmstudio".to_string(),
            name: "LM Studio".to_string(),
            default_url: "http://localhost:1234/v1".to_string(),
            default_model: "llama-3.1-8b".to_string(),
            requires_api_key: false,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            default_url: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-4o".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "anthropic".to_string(),
            name: "Anthropic (Claude)".to_string(),
            default_url: "https://api.anthropic.com".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            requires_api_key: true,
            api_type: "anthropic".to_string(),
        },
        ProviderInfo {
            id: "google".to_string(),
            name: "Google (Gemini)".to_string(),
            default_url: "https://generativelanguage.googleapis.com/v1".to_string(),
            default_model: "gemini-2.0-flash".to_string(),
            requires_api_key: true,
            api_type: "google".to_string(),
        },
        ProviderInfo {
            id: "azure".to_string(),
            name: "Azure OpenAI".to_string(),
            default_url: "https://YOUR_RESOURCE.openai.azure.com/openai/deployments/YOUR_DEPLOYMENT".to_string(),
            default_model: "gpt-4o".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "openrouter".to_string(),
            name: "OpenRouter".to_string(),
            default_url: "https://openrouter.ai/api/v1".to_string(),
            default_model: "anthropic/claude-sonnet-4-20250514".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "cloudflare".to_string(),
            name: "Cloudflare AI Gateway".to_string(),
            default_url: "https://gateway.ai.cloudflare.com/v1/account/YOUR_ACCOUNT_ID/gateway".to_string(),
            default_model: "@cf/meta/llama-3.1-8b-instruct".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "vercel".to_string(),
            name: "Vercel AI Gateway".to_string(),
            default_url: "https://gateway.vercel.ai/api".to_string(),
            default_model: "openai/gpt-4o".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "together".to_string(),
            name: "Together AI".to_string(),
            default_url: "https://api.together.ai/v1".to_string(),
            default_model: "meta-llama/Llama-3.3-70B-Instruct".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "litellm".to_string(),
            name: "LiteLLM (Unified)".to_string(),
            default_url: "http://localhost:4000".to_string(),
            default_model: "gpt-4o".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "mistral".to_string(),
            name: "Mistral AI".to_string(),
            default_url: "https://api.mistral.ai/v1".to_string(),
            default_model: "mistral-large-latest".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "cohere".to_string(),
            name: "Cohere".to_string(),
            default_url: "https://api.cohere.ai/v1".to_string(),
            default_model: "command-r-plus".to_string(),
            requires_api_key: true,
            api_type: "cohere".to_string(),
        },
        ProviderInfo {
            id: "groq".to_string(),
            name: "Groq".to_string(),
            default_url: "https://api.groq.com/openai/v1".to_string(),
            default_model: "llama-3.3-70b-versatile".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "fireworks".to_string(),
            name: "Fireworks AI".to_string(),
            default_url: "https://api.fireworks.ai/inference/v1".to_string(),
            default_model: "accounts/fireworks/models/llama-v3-3-70b-instruct".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "huggingface".to_string(),
            name: "Hugging Face".to_string(),
            default_url: "https://router.huggingface.co".to_string(),
            default_model: "meta-llama/Llama-3.3-70B-Instruct".to_string(),
            requires_api_key: true,
            api_type: "huggingface".to_string(),
        },
        ProviderInfo {
            id: "zhipu-glm".to_string(),
            name: "Zhipu GLM".to_string(),
            default_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            default_model: "glm-4-plus".to_string(),
            requires_api_key: true,
            api_type: "zhipu".to_string(),
        },
        ProviderInfo {
            id: "qianfan".to_string(),
            name: "Baidu Qianfan".to_string(),
            default_url: "https://qianfan.baidubce.com/v2".to_string(),
            default_model: "ernie-4.0-8k".to_string(),
            requires_api_key: true,
            api_type: "baidu".to_string(),
        },
        ProviderInfo {
            id: "moonshot".to_string(),
            name: "Moonshot (Kimi)".to_string(),
            default_url: "https://api.moonshot.ai/v1".to_string(),
            default_model: "kimi-k2.5".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "minimax".to_string(),
            name: "MiniMax".to_string(),
            default_url: "https://api.minimax.chat/v1".to_string(),
            default_model: "abab6.5s-chat".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "bedrock".to_string(),
            name: "Amazon Bedrock".to_string(),
            default_url: "https://bedrock-runtime.us-east-1.amazonaws.com".to_string(),
            default_model: "anthropic.claude-3-sonnet-20240229-v1:0".to_string(),
            requires_api_key: true,
            api_type: "aws".to_string(),
        },
        ProviderInfo {
            id: "vertex".to_string(),
            name: "Google Vertex AI".to_string(),
            default_url: "https://us-central1-aiplatform.googleapis.com".to_string(),
            default_model: "gemini-2.0-flash".to_string(),
            requires_api_key: true,
            api_type: "google".to_string(),
        },
        ProviderInfo {
            id: "xai".to_string(),
            name: "xAI (Grok)".to_string(),
            default_url: "https://api.x.ai/v1".to_string(),
            default_model: "grok-2-1212".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "venice".to_string(),
            name: "Venice AI".to_string(),
            default_url: "https://api.venice.ai/api/v1".to_string(),
            default_model: "llama-3.3-70b".to_string(),
            requires_api_key: true,
            api_type: "openai".to_string(),
        },
        ProviderInfo {
            id: "custom".to_string(),
            name: "Custom (OpenAI-compatible)".to_string(),
            default_url: "https://your-api.example.com/v1".to_string(),
            default_model: "model-name".to_string(),
            requires_api_key: false,
            api_type: "openai".to_string(),
        },
    ]
}

async fn list_providers() -> Json<Vec<ProviderInfo>> {
    Json(get_provider_info())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub url: String,
    pub model: String,
    pub has_api_key: bool,
    // Extended settings
    pub ctx_length: Option<u32>,
    pub ctx_history: Option<f32>,
    pub vision: Option<bool>,
    pub rl_requests: Option<u32>,
    pub rl_input: Option<u32>,
    pub rl_output: Option<u32>,
    pub kwargs: Option<String>,
}

impl From<LlmConfig> for LlmResponse {
    fn from(config: LlmConfig) -> Self {
        Self {
            id: config.id,
            name: config.name,
            provider: match config.provider {
                LlmProvider::Local => "local".to_string(),
                LlmProvider::Ollama => "ollama".to_string(),
                LlmProvider::Vllm => "vllm".to_string(),
                LlmProvider::LmStudio => "lmstudio".to_string(),
                LlmProvider::OpenAI => "openai".to_string(),
                LlmProvider::Anthropic => "anthropic".to_string(),
                LlmProvider::Google => "google".to_string(),
                LlmProvider::Azure => "azure".to_string(),
                LlmProvider::OpenRouter => "openrouter".to_string(),
                LlmProvider::Cloudflare => "cloudflare".to_string(),
                LlmProvider::Vercel => "vercel".to_string(),
                LlmProvider::Together => "together".to_string(),
                LlmProvider::LiteLlama => "litellm".to_string(),
                LlmProvider::Mistral => "mistral".to_string(),
                LlmProvider::Cohere => "cohere".to_string(),
                LlmProvider::Groq => "groq".to_string(),
                LlmProvider::Fireworks => "fireworks".to_string(),
                LlmProvider::HuggingFace => "huggingface".to_string(),
                LlmProvider::ZhipuGlm => "zhipu-glm".to_string(),
                LlmProvider::Qianfan => "qianfan".to_string(),
                LlmProvider::Moonshot => "moonshot".to_string(),
                LlmProvider::MiniMax => "minimax".to_string(),
                LlmProvider::Bedrock => "bedrock".to_string(),
                LlmProvider::Vertex => "vertex".to_string(),
                LlmProvider::Xai => "xai".to_string(),
                LlmProvider::Venice => "venice".to_string(),
                LlmProvider::Custom => "custom".to_string(),
            },
            url: config.url,
            model: config.model,
            has_api_key: config.api_key.is_some(),
            ctx_length: config.ctx_length,
            ctx_history: config.ctx_history,
            vision: config.vision,
            rl_requests: config.rl_requests,
            rl_input: config.rl_input,
            rl_output: config.rl_output,
            kwargs: config.kwargs,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLlmRequest {
    pub name: String,
    pub provider: String,
    pub url: String,
    pub model: String,
    pub api_key: Option<String>,
    // Extended settings
    pub ctx_length: Option<u32>,
    pub ctx_history: Option<f32>,
    pub vision: Option<bool>,
    pub rl_requests: Option<u32>,
    pub rl_input: Option<u32>,
    pub rl_output: Option<u32>,
    pub kwargs: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLlmRequest {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub url: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    // Extended settings
    pub ctx_length: Option<u32>,
    pub ctx_history: Option<f32>,
    pub vision: Option<bool>,
    pub rl_requests: Option<u32>,
    pub rl_input: Option<u32>,
    pub rl_output: Option<u32>,
    pub kwargs: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TestLlmResponse {
    pub success: bool,
    pub message: String,
    pub latency_ms: Option<u64>,
}

async fn list_llms(State(state): State<AppState>) -> Json<Vec<LlmResponse>> {
    let llms = state.config.agent.llms.clone();
    Json(llms.into_iter().map(LlmResponse::from).collect())
}

async fn get_llm(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<LlmResponse>, String> {
    state
        .config
        .agent
        .llms
        .iter()
        .find(|llm| llm.id == id)
        .map(|llm| Json(LlmResponse::from(llm.clone())))
        .ok_or_else(|| "LLM not found".to_string())
}

async fn add_llm(
    State(state): State<AppState>,
    Json(payload): Json<CreateLlmRequest>,
) -> Result<Json<LlmResponse>, String> {
    let provider = match payload.provider.to_lowercase().as_str() {
        "local" => LlmProvider::Local,
        "ollama" => LlmProvider::Ollama,
        "vllm" => LlmProvider::Vllm,
        "lmstudio" => LlmProvider::LmStudio,
        "openai" => LlmProvider::OpenAI,
        "anthropic" => LlmProvider::Anthropic,
        "google" => LlmProvider::Google,
        "azure" => LlmProvider::Azure,
        "openrouter" => LlmProvider::OpenRouter,
        "cloudflare" => LlmProvider::Cloudflare,
        "vercel" => LlmProvider::Vercel,
        "together" => LlmProvider::Together,
        "litellm" => LlmProvider::LiteLlama,
        "mistral" => LlmProvider::Mistral,
        "cohere" => LlmProvider::Cohere,
        "groq" => LlmProvider::Groq,
        "fireworks" => LlmProvider::Fireworks,
        "huggingface" => LlmProvider::HuggingFace,
        "zhipu-glm" => LlmProvider::ZhipuGlm,
        "qianfan" => LlmProvider::Qianfan,
        "moonshot" => LlmProvider::Moonshot,
        "minimax" => LlmProvider::MiniMax,
        "bedrock" => LlmProvider::Bedrock,
        "vertex" => LlmProvider::Vertex,
        "xai" => LlmProvider::Xai,
        "venice" => LlmProvider::Venice,
        "custom" => LlmProvider::Custom,
        _ => return Err("Invalid provider type".to_string()),
    };

    let id = format!("llm-{}", ulid::Ulid::new());
    let new_llm = LlmConfig {
        id: id.clone(),
        name: payload.name,
        provider,
        url: payload.url,
        model: payload.model,
        api_key: payload.api_key,
        ctx_length: payload.ctx_length,
        ctx_history: payload.ctx_history,
        vision: payload.vision,
        rl_requests: payload.rl_requests,
        rl_input: payload.rl_input,
        rl_output: payload.rl_output,
        kwargs: payload.kwargs,
    };

    let mut config = state.config.clone();
    config.agent.llms.push(new_llm.clone());
    
    // Update in-memory global config
    if let Ok(mut global_config) = GLOBAL_CONFIG.write() {
        *global_config = Some(config.clone());
    }

    // Persist to database
    let agent_config = config.agent.clone();
    if let Err(e) = AppConfig::save_section_to_db(&state.config_repo, "agent", &agent_config).await {
        tracing::warn!("Failed to persist LLM config to database: {}", e);
    }

    Ok(Json(LlmResponse::from(new_llm)))
}

async fn update_llm(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateLlmRequest>,
) -> Result<Json<LlmResponse>, String> {
    let mut config = state.config.clone();
    
    let llm = config
        .agent
        .llms
        .iter_mut()
        .find(|llm| llm.id == id)
        .ok_or_else(|| "LLM not found".to_string())?;

    if let Some(name) = payload.name {
        llm.name = name;
    }
    if let Some(provider) = payload.provider {
        llm.provider = match provider.to_lowercase().as_str() {
            "local" => LlmProvider::Local,
            "ollama" => LlmProvider::Ollama,
            "vllm" => LlmProvider::Vllm,
            "lmstudio" => LlmProvider::LmStudio,
            "openai" => LlmProvider::OpenAI,
            "anthropic" => LlmProvider::Anthropic,
            "google" => LlmProvider::Google,
            "azure" => LlmProvider::Azure,
            "openrouter" => LlmProvider::OpenRouter,
            "cloudflare" => LlmProvider::Cloudflare,
            "vercel" => LlmProvider::Vercel,
            "together" => LlmProvider::Together,
            "litellm" => LlmProvider::LiteLlama,
            "mistral" => LlmProvider::Mistral,
            "cohere" => LlmProvider::Cohere,
            "groq" => LlmProvider::Groq,
            "fireworks" => LlmProvider::Fireworks,
            "huggingface" => LlmProvider::HuggingFace,
            "zhipu-glm" => LlmProvider::ZhipuGlm,
            "qianfan" => LlmProvider::Qianfan,
            "moonshot" => LlmProvider::Moonshot,
            "minimax" => LlmProvider::MiniMax,
            "bedrock" => LlmProvider::Bedrock,
            "vertex" => LlmProvider::Vertex,
            "xai" => LlmProvider::Xai,
            "venice" => LlmProvider::Venice,
            "custom" => LlmProvider::Custom,
            _ => return Err("Invalid provider type".to_string()),
        };
    }
    if let Some(url) = payload.url {
        llm.url = url;
    }
    if let Some(model) = payload.model {
        llm.model = model;
    }
    if let Some(api_key) = payload.api_key {
        llm.api_key = Some(api_key);
    }

    let updated = llm.clone();

    // Update AppConfig global
    if let Ok(mut global_config) = GLOBAL_CONFIG.write() {
        *global_config = Some(config.clone());
    }

    // Persist to database
    let agent_config = config.agent.clone();
    if let Err(e) = AppConfig::save_section_to_db(&state.config_repo, "agent", &agent_config).await {
        tracing::warn!("Failed to persist LLM config to database: {}", e);
    }

    Ok(Json(LlmResponse::from(updated)))
}

async fn delete_llm(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let mut config = state.config.clone();
    
    let initial_len = config.agent.llms.len();
    config.agent.llms.retain(|llm| llm.id != id);
    
    if config.agent.llms.len() == initial_len {
        return Err("LLM not found".to_string());
    }

    // Clear default if deleted
    if config.agent.default_llm_id.as_ref() == Some(&id) {
        config.agent.default_llm_id = config.agent.llms.first().map(|l| l.id.clone());
    }

    // Update AppConfig global
    if let Ok(mut global_config) = GLOBAL_CONFIG.write() {
        *global_config = Some(config.clone());
    }

    // Persist to database
    let agent_config = config.agent.clone();
    if let Err(e) = AppConfig::save_section_to_db(&state.config_repo, "agent", &agent_config).await {
        tracing::warn!("Failed to persist LLM config to database: {}", e);
    }

    Ok(Json(serde_json::json!({ "success": true, "message": "LLM deleted" })))
}

async fn test_llm(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<TestLlmResponse>, String> {
    let llm = state
        .config
        .agent
        .llms
        .iter()
        .find(|l| l.id == id)
        .ok_or_else(|| "LLM not found".to_string())?;

    let start = std::time::Instant::now();
    
    // Test connection based on provider
    let client = reqwest::Client::new();
    let test_result = match llm.provider {
        LlmProvider::Local 
        | LlmProvider::Ollama 
        | LlmProvider::Vllm 
        | LlmProvider::LmStudio
        | LlmProvider::OpenAI 
        | LlmProvider::Azure
        | LlmProvider::OpenRouter 
        | LlmProvider::Cloudflare
        | LlmProvider::Vercel 
        | LlmProvider::Together
        | LlmProvider::LiteLlama
        | LlmProvider::Mistral
        | LlmProvider::Groq
        | LlmProvider::Fireworks
        | LlmProvider::Moonshot
        | LlmProvider::MiniMax
        | LlmProvider::Xai
        | LlmProvider::Venice
        | LlmProvider::Custom => {
            // Most providers use OpenAI-compatible API
            let url = format!("{}/v1/models", llm.url.trim_end_matches('/'));
            let mut req = client.get(&url);
            if let Some(ref key) = llm.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            req.send().await
        }
        LlmProvider::Anthropic => {
            // Anthropic uses different API
            let url = format!("{}/v1/messages", llm.url.trim_end_matches('/'));
            let mut req = client.post(&url);
            if let Some(ref key) = llm.api_key {
                req = req.header("x-api-key", key);
                req = req.header("anthropic-version", "2023-06-01");
            }
            req = req.header("Content-Type", "application/json");
            req = req.json(&serde_json::json!({
                "model": llm.model,
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "hi"}]
            }));
            req.send().await
        }
        LlmProvider::Google | LlmProvider::Vertex => {
            // Google uses REST API
            let url = format!("{}/models?key={}", llm.url.trim_end_matches('/'), 
                llm.api_key.as_deref().unwrap_or(""));
            let req = client.get(&url);
            req.send().await
        }
        LlmProvider::Cohere => {
            let url = format!("{}/v1/models", llm.url.trim_end_matches('/'));
            let mut req = client.get(&url);
            if let Some(ref key) = llm.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            req.send().await
        }
        LlmProvider::HuggingFace => {
            let url = format!("{}/models", llm.url.trim_end_matches('/'));
            let mut req = client.get(&url);
            if let Some(ref key) = llm.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            req.send().await
        }
        LlmProvider::ZhipuGlm 
        | LlmProvider::Qianfan 
        | LlmProvider::Bedrock => {
            // These have special APIs, just try the URL
            let mut req = client.get(&llm.url);
            if let Some(ref key) = llm.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            req.send().await
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;

    match test_result {
        Ok(resp) if resp.status().is_success() => {
            Ok(Json(TestLlmResponse {
                success: true,
                message: "Connection successful".to_string(),
                latency_ms: Some(latency_ms),
            }))
        }
        Ok(resp) => {
            Ok(Json(TestLlmResponse {
                success: false,
                message: format!("HTTP {}", resp.status()),
                latency_ms: Some(latency_ms),
            }))
        }
        Err(e) => {
            Ok(Json(TestLlmResponse {
                success: false,
                message: e.to_string(),
                latency_ms: Some(latency_ms),
            }))
        }
    }
}

async fn get_default_llm(State(state): State<AppState>) -> Json<Option<LlmResponse>> {
    if let Some(id) = &state.config.agent.default_llm_id {
        let llm = state.config.agent.llms.iter().find(|l| &l.id == id);
        Json(llm.map(|l| LlmResponse::from(l.clone())))
    } else {
        Json(None)
    }
}

async fn set_default_llm(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<LlmResponse>, String> {
    let id = payload["id"]
        .as_str()
        .ok_or_else(|| "Missing LLM id".to_string())?
        .to_string();

    // Verify LLM exists
    let llm = state
        .config
        .agent
        .llms
        .iter()
        .find(|l| l.id == id)
        .ok_or_else(|| "LLM not found".to_string())?
        .clone();

    let mut config = state.config.clone();
    config.agent.default_llm_id = Some(id.clone());
    config.agent.llama_url = llm.url.clone();
    config.agent.llama_model = llm.model.clone();

    // Update AppConfig global
    if let Ok(mut global_config) = GLOBAL_CONFIG.write() {
        *global_config = Some(config.clone());
    }

    // Persist to database
    let agent_config = config.agent.clone();
    if let Err(e) = AppConfig::save_section_to_db(&state.config_repo, "agent", &agent_config).await {
        tracing::warn!("Failed to persist LLM config to database: {}", e);
    }

    Ok(Json(LlmResponse::from(llm)))
}

// ============ Provider Plugins (NEW) ============

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub provider_type: String,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
    pub config: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
    pub config: Option<String>,
    pub enabled: Option<bool>,
}

async fn list_provider_plugins(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProviderPlugin>>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let plugins = repo
        .list_providers(None, false)
        .await
        .map_err(|e| format!("Failed to list providers: {}", e))?;
    Ok(Json(plugins))
}

async fn create_provider_plugin(
    State(state): State<AppState>,
    Json(req): Json<CreateProviderRequest>,
) -> Result<Json<ProviderPlugin>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let id = Ulid::new().to_string();
    let plugin = repo
        .create_provider(
            &id,
            &req.provider_type,
            &req.name,
            &req.base_url,
            req.api_key.as_deref(),
            req.default_model.as_deref(),
            req.config.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to create provider: {}", e))?;
    Ok(Json(plugin))
}

async fn get_provider_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ProviderPlugin>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let plugin = repo
        .get_provider(&id)
        .await
        .map_err(|e| format!("Failed to get provider: {}", e))?;
    Ok(Json(plugin))
}

async fn update_provider_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProviderRequest>,
) -> Result<Json<ProviderPlugin>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let plugin = repo
        .update_provider(
            &id,
            req.name.as_deref(),
            req.base_url.as_deref(),
            req.api_key.as_deref(),
            req.default_model.as_deref(),
            req.config.as_deref(),
            req.enabled,
        )
        .await
        .map_err(|e| format!("Failed to update provider: {}", e))?;
    Ok(Json(plugin))
}

async fn delete_provider_plugin(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ProviderRepository::new(&state.pool);
    repo.delete_provider(&id)
        .await
        .map_err(|e| format!("Failed to delete provider: {}", e))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}

async fn list_provider_models(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<Vec<ProviderModel>>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let models = repo
        .list_provider_models(&provider_id)
        .await
        .map_err(|e| format!("Failed to list models: {}", e))?;
    Ok(Json(models))
}

async fn get_provider_health_status(
    State(state): State<AppState>,
    Path(provider_id): Path<String>,
) -> Result<Json<Option<ProviderHealth>>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let health = repo
        .get_provider_health(&provider_id)
        .await
        .map_err(|e| format!("Failed to get health: {}", e))?;
    Ok(Json(health))
}

// ============ Fast Mode (NEW) ============

#[derive(Debug, Deserialize)]
pub struct SetFastModeRequest {
    pub fast_enabled: bool,
    pub fast_model: Option<String>,
    pub fast_config: Option<String>,
    pub toggles: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FastModeResponse {
    pub session_id: String,
    pub fast_enabled: bool,
    pub fast_model: Option<String>,
    pub fast_config: Option<serde_json::Value>,
    pub toggles: Option<serde_json::Value>,
}

// Global fast mode (stored in memory for now)
static GLOBAL_FAST_MODE: std::sync::LazyLock<std::sync::Mutex<GlobalFastMode>> = 
    std::sync::LazyLock::new(|| std::sync::Mutex::new(GlobalFastMode {
        enabled: false,
        fast_model: None,
    }));

struct GlobalFastMode {
    enabled: bool,
    fast_model: Option<String>,
}

async fn get_global_fast_mode(
    State(_state): State<AppState>,
) -> Json<GlobalFastMode> {
    let guard = GLOBAL_FAST_MODE.lock().unwrap();
    Json(GlobalFastMode {
        enabled: guard.enabled,
        fast_model: guard.fast_model.clone(),
    })
}

async fn set_global_fast_mode(
    State(_state): State<AppState>,
    Json(req): Json<SetFastModeRequest>,
) -> Json<GlobalFastMode> {
    let mut guard = GLOBAL_FAST_MODE.lock().unwrap();
    guard.enabled = req.fast_enabled;
    guard.fast_model = req.fast_model;
    Json(GlobalFastMode {
        enabled: guard.enabled,
        fast_model: guard.fast_model.clone(),
    })
}

async fn get_session_fast_mode(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<FastModeResponse>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let fast_mode = repo
        .get_session_fast_mode(&session_id)
        .await
        .map_err(|e| format!("Failed to get fast mode: {}", e))?;

    match fast_mode {
        Some(fm) => {
            let config: Option<serde_json::Value> = fm.fast_config
                .as_ref()
                .and_then(|c| serde_json::from_str(c).ok());
            let toggles: Option<serde_json::Value> = fm.toggles
                .as_ref()
                .and_then(|t| serde_json::from_str(t).ok());
            
            Ok(Json(FastModeResponse {
                session_id: fm.session_id,
                fast_enabled: fm.fast_enabled != 0,
                fast_model: fm.fast_model,
                fast_config: config,
                toggles,
            }))
        }
        None => Ok(Json(FastModeResponse {
            session_id,
            fast_enabled: false,
            fast_model: None,
            fast_config: None,
            toggles: None,
        })),
    }
}

async fn set_session_fast_mode(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(req): Json<SetFastModeRequest>,
) -> Result<Json<FastModeResponse>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let id = Ulid::new().to_string();
    
    let fast_mode = repo
        .upsert_session_fast_mode(
            &id,
            &session_id,
            req.fast_enabled,
            req.fast_model.as_deref(),
            req.fast_config.as_deref(),
            req.toggles.as_deref(),
        )
        .await
        .map_err(|e| format!("Failed to set fast mode: {}", e))?;

    let config: Option<serde_json::Value> = fast_mode.fast_config
        .as_ref()
        .and_then(|c| serde_json::from_str(c).ok());
    let toggles: Option<serde_json::Value> = fast_mode.toggles
        .as_ref()
        .and_then(|t| serde_json::from_str(t).ok());

    Ok(Json(FastModeResponse {
        session_id: fast_mode.session_id,
        fast_enabled: fast_mode.fast_enabled != 0,
        fast_model: fast_mode.fast_model,
        fast_config: config,
        toggles,
    }))
}

// ============ Model Fallbacks (NEW) ============

#[derive(Debug, Deserialize)]
pub struct AddFallbackRequest {
    pub primary_model: String,
    pub fallback_model: String,
    pub provider: Option<String>,
    pub priority: Option<i32>,
}

async fn list_model_fallbacks(
    State(state): State<AppState>,
    Query(params): Query<FallbackQuery>,
) -> Result<Json<Vec<ModelFallback>>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let fallbacks = repo
        .list_fallbacks(params.primary_model.as_deref())
        .await
        .map_err(|e| format!("Failed to list fallbacks: {}", e))?;
    Ok(Json(fallbacks))
}

#[derive(Debug, Deserialize)]
pub struct FallbackQuery {
    pub primary_model: Option<String>,
}

async fn add_model_fallback(
    State(state): State<AppState>,
    Json(req): Json<AddFallbackRequest>,
) -> Result<Json<ModelFallback>, String> {
    let repo = ProviderRepository::new(&state.pool);
    let id = Ulid::new().to_string();
    let priority = req.priority.unwrap_or(1);
    
    let fallback = repo
        .add_fallback(
            &id,
            &req.primary_model,
            &req.fallback_model,
            req.provider.as_deref(),
            priority,
        )
        .await
        .map_err(|e| format!("Failed to add fallback: {}", e))?;
    Ok(Json(fallback))
}

async fn delete_model_fallback(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, String> {
    let repo = ProviderRepository::new(&state.pool);
    repo.delete_fallback(&id)
        .await
        .map_err(|e| format!("Failed to delete fallback: {}", e))?;
    Ok(Json(serde_json::json!({ "deleted": true })))
}
