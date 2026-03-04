use crate::unified_config::AppConfig;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

pub type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct AuthConfig {
    pub secret: Arc<String>,
    pub disabled: bool,
}

impl AuthConfig {
    pub fn new() -> Self {
        let config = AppConfig::global();
        Self {
            secret: Arc::new(config.auth.shared_secret),
            disabled: config.auth.disabled,
        }
    }

    pub fn from_env(secret: String) -> Self {
        Self {
            secret: Arc::new(secret),
            disabled: false,
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sign_request(secret: &str, method: &str, path: &str, body: &[u8], timestamp: i64) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    
    mac.update(timestamp.to_string().as_bytes());
    mac.update(method.as_bytes());
    mac.update(path.as_bytes());
    mac.update(body);
    
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub fn verify_request(
    secret: &str,
    method: &str,
    path: &str,
    body: &[u8],
    signature: &str,
    timestamp: i64,
) -> bool {
    let expected = sign_request(secret, method, path, body, timestamp);
    timing_safe_eq(&expected, signature)
}

fn timing_safe_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    
    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
    }
    
    result == 0
}

pub async fn auth_middleware(
    State(state): State<AuthConfig>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if state.disabled {
        return next.run(request).await;
    }

    let signature = request
        .headers()
        .get("X-APEX-Signature")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let timestamp: Option<i64> = request
        .headers()
        .get("X-APEX-Timestamp")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();

    let (parts, body) = request.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
        Ok(b) => b.to_vec(),
        Err(_) => {
            return Response::builder()
                .status(400)
                .body(Body::from("Invalid request body"))
                .unwrap();
        }
    };

    let (send_signature, send_timestamp) = match (signature, timestamp) {
        (Some(s), Some(t)) => (s, t),
        _ => {
            tracing::warn!(path = %path, "Missing authentication headers");
            return Response::builder()
                .status(401)
                .body(Body::from("Missing authentication: X-APEX-Signature and X-APEX-Timestamp required"))
                .unwrap();
        }
    };

    let now = chrono::Utc::now().timestamp();
    let age = (now - send_timestamp).abs();
    if age > 300 {
        tracing::warn!(path = %path, age = age, "Request timestamp too old");
        return Response::builder()
            .status(401)
            .body(Body::from("Request timestamp too old (max 5 minutes)"))
            .unwrap();
    }

    if !verify_request(&state.secret, &method, &path, &body_bytes, &send_signature, send_timestamp) {
        tracing::warn!(path = %path, "Invalid signature");
        return Response::builder()
            .status(401)
            .body(Body::from("Invalid signature"))
            .unwrap();
    }

    let request = Request::from_parts(parts, Body::from(body_bytes));
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let secret = "test-secret";
        let timestamp = 1234567890i64;
        
        let signature = sign_request(secret, "POST", "/api/v1/tasks", b"{}", timestamp);
        
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_verify_request_valid() {
        let secret = "test-secret";
        let timestamp = chrono::Utc::now().timestamp();
        
        let signature = sign_request(secret, "POST", "/api/v1/tasks", b"{}", timestamp);
        
        assert!(verify_request(secret, "POST", "/api/v1/tasks", b"{}", &signature, timestamp));
    }

    #[test]
    fn test_verify_request_invalid_signature() {
        let secret = "test-secret";
        let timestamp = chrono::Utc::now().timestamp();
        
        assert!(!verify_request(secret, "POST", "/api/v1/tasks", b"{}", "invalid-signature", timestamp));
    }

    #[test]
    fn test_verify_request_wrong_secret() {
        let secret = "test-secret";
        let wrong_secret = "wrong-secret";
        let timestamp = chrono::Utc::now().timestamp();
        
        let signature = sign_request(secret, "POST", "/api/v1/tasks", b"{}", timestamp);
        
        assert!(!verify_request(wrong_secret, "POST", "/api/v1/tasks", b"{}", &signature, timestamp));
    }

    #[test]
    fn test_timing_safe_eq_equal() {
        assert!(timing_safe_eq("test", "test"));
    }

    #[test]
    fn test_timing_safe_eq_different_length() {
        assert!(!timing_safe_eq("test", "testing"));
    }

    #[test]
    fn test_timing_safe_eq_different_content() {
        assert!(!timing_safe_eq("test", "TEST"));
    }

    #[test]
    fn test_auth_config_from_env() {
        let config = AuthConfig::from_env("test-secret-123".to_string());
        
        assert_eq!(*config.secret, "test-secret-123");
        assert!(!config.disabled);
    }

    #[test]
    fn test_sign_request_different_methods_different_signatures() {
        let secret = "test-secret";
        let timestamp = 1234567890i64;
        
        let get_sig = sign_request(secret, "GET", "/api/v1/tasks", b"", timestamp);
        let post_sig = sign_request(secret, "POST", "/api/v1/tasks", b"", timestamp);
        
        assert_ne!(get_sig, post_sig);
    }

    #[test]
    fn test_sign_request_different_paths_different_signatures() {
        let secret = "test-secret";
        let timestamp = 1234567890i64;
        
        let tasks_sig = sign_request(secret, "GET", "/api/v1/tasks", b"", timestamp);
        let skills_sig = sign_request(secret, "GET", "/api/v1/skills", b"", timestamp);
        
        assert_ne!(tasks_sig, skills_sig);
    }
}
