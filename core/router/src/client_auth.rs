use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCredentials {
    pub client_id: String,
    pub client_name: String,
    pub secret_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub rate_limit: u32,
    pub enabled: bool,
}

impl ClientCredentials {
    pub fn new(client_id: String, client_name: String, secret_hash: String) -> Self {
        Self {
            client_id,
            client_name,
            secret_hash,
            created_at: Utc::now(),
            last_used: None,
            rate_limit: 60,
            enabled: true,
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub client_name: String,
    pub secret: String,
    pub rate_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientResponse {
    pub client_id: String,
    pub client_name: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub rate_limit: u32,
    pub enabled: bool,
}

impl From<&ClientCredentials> for ClientResponse {
    fn from(creds: &ClientCredentials) -> Self {
        Self {
            client_id: creds.client_id.clone(),
            client_name: creds.client_name.clone(),
            created_at: creds.created_at,
            last_used: creds.last_used,
            rate_limit: creds.rate_limit,
            enabled: creds.enabled,
        }
    }
}

pub struct ClientAuth {
    clients: Arc<RwLock<HashMap<String, ClientCredentials>>>,
    rate_limiter: Arc<RateLimiter>,
}

impl ClientAuth {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RateLimiter::new(100)),
        }
    }

    pub async fn register_client(&self, request: CreateClientRequest) -> Result<ClientResponse, ClientAuthError> {
        let client_id = ulid::Ulid::new().to_string();
        let secret_hash = Self::hash_secret(&request.secret)?;

        let mut client = ClientCredentials::new(
            client_id.clone(),
            request.client_name,
            secret_hash,
        );

        if let Some(limit) = request.rate_limit {
            client.rate_limit = limit;
        }

        let response = ClientResponse::from(&client);

        let mut clients = self.clients.write().await;
        clients.insert(client_id, client);

        Ok(response)
    }

    pub async fn authenticate(&self, client_id: &str, secret: &str) -> Result<bool, ClientAuthError> {
        let clients = self.clients.read().await;
        
        let client = clients.get(client_id)
            .ok_or(ClientAuthError::ClientNotFound)?;

        if !client.enabled {
            return Err(ClientAuthError::ClientDisabled);
        }

        let secret_hash = Self::hash_secret(secret)?;
        let is_valid = client.secret_hash == secret_hash;

        if is_valid {
            drop(clients);
            let mut clients = self.clients.write().await;
            if let Some(client) = clients.get_mut(client_id) {
                client.update_last_used();
            }
        }

        Ok(is_valid)
    }

    pub async fn check_rate_limit(&self, client_id: &str) -> Result<bool, ClientAuthError> {
        let clients = self.clients.read().await;
        let client = clients.get(client_id)
            .ok_or(ClientAuthError::ClientNotFound)?;

        let allowed = self.rate_limiter.check(client_id, client.rate_limit);
        Ok(allowed)
    }

    pub async fn list_clients(&self) -> Vec<ClientResponse> {
        let clients = self.clients.read().await;
        clients.values()
            .map(ClientResponse::from)
            .collect()
    }

    pub async fn get_client(&self, client_id: &str) -> Option<ClientResponse> {
        let clients = self.clients.read().await;
        clients.get(client_id).map(ClientResponse::from)
    }

    pub async fn revoke_client(&self, client_id: &str) -> Result<(), ClientAuthError> {
        let mut clients = self.clients.write().await;
        let client = clients.get_mut(client_id)
            .ok_or(ClientAuthError::ClientNotFound)?;
        
        client.enabled = false;
        Ok(())
    }

    pub async fn rotate_secret(&self, client_id: &str, new_secret: &str) -> Result<(), ClientAuthError> {
        let mut clients = self.clients.write().await;
        let client = clients.get_mut(client_id)
            .ok_or(ClientAuthError::ClientNotFound)?;

        client.secret_hash = Self::hash_secret(new_secret)?;
        Ok(())
    }

    fn hash_secret(secret: &str) -> Result<String, ClientAuthError> {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        let result = hasher.finalize();
        
        Ok(hex::encode(result))
    }
}

impl Default for ClientAuth {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(window_secs: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            window_secs,
        }
    }

    pub fn check(&self, key: &str, limit: u32) -> bool {
        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(self.window_secs as i64);
        
        let count = self.requests
            .blocking_read()
            .get(key)
            .map(|times| times.iter().filter(|t| *t > window_start).count())
            .unwrap_or(0);

        count < limit as usize
    }

    pub async fn record(&self, key: &str) {
        let now = Utc::now();
        let mut requests = self.requests.write().await;
        
        requests.entry(key.to_string())
            .or_insert_with(Vec::new)
            .push(now);

        let window_start = now - chrono::Duration::seconds(self.window_secs as i64);
        if let Some(times) = requests.get_mut(key) {
            times.retain(|t| *t > window_start);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientAuthError {
    #[error("Client not found")]
    ClientNotFound,
    
    #[error("Client is disabled")]
    ClientDisabled,
    
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Hash error: {0}")]
    HashError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_client() {
        let auth = ClientAuth::new();
        
        let request = CreateClientRequest {
            client_name: "test-client".to_string(),
            secret: "secret123".to_string(),
            rate_limit: Some(100),
        };
        
        let result = auth.register_client(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_authenticate_valid() {
        let auth = ClientAuth::new();
        
        let request = CreateClientRequest {
            client_name: "test-client".to_string(),
            secret: "secret123".to_string(),
            rate_limit: None,
        };
        
        let response = auth.register_client(request).await.unwrap();
        let is_valid = auth.authenticate(&response.client_id, "secret123").await.unwrap();
        
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_authenticate_invalid() {
        let auth = ClientAuth::new();
        
        let request = CreateClientRequest {
            client_name: "test-client".to_string(),
            secret: "secret123".to_string(),
            rate_limit: None,
        };
        
        let response = auth.register_client(request).await.unwrap();
        let is_valid = auth.authenticate(&response.client_id, "wrong-secret").await.unwrap();
        
        assert!(!is_valid);
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(60);
        
        for _ in 0..50 {
            assert!(limiter.check("test", 100));
        }
        
        assert!(!limiter.check("test", 50));
    }
}
