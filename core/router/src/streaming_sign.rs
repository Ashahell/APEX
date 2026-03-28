use axum::{
    extract::Query,
    routing::get,
    Json,
    Router,
};
use serde::Deserialize;

/// Query parameters for streaming sign endpoint
#[derive(Debug, Deserialize)]
pub struct StreamSignQuery {
    /// The streaming path to sign (e.g., "/stream/stats", "/stream/hands/123")
    path: String,
}

/// Response containing signed URL components
#[derive(Debug, serde::Serialize)]
pub struct StreamSignResponse {
    /// The signed URL with query parameters
    url: String,
    /// The signature component
    signature: String,
    /// The timestamp used
    timestamp: i64,
    /// Number of seconds until expiry
    expires_in: i64,
}

/// Generate a signed URL for streaming endpoints.
/// This allows browser clients to authenticate without exposing secrets.
///
/// The signature is generated using HMAC-SHA256 with the shared secret,
/// signed against the path and current timestamp.
pub async fn sign_stream_endpoint(
    Query(query): Query<StreamSignQuery>,
) -> Json<StreamSignResponse> {
    use crate::auth::sign_request;
    use crate::unified_config::AppConfig;

    // Get the shared secret from config
    let config = AppConfig::global();
    let secret = &config.auth.shared_secret;

    // Generate timestamp (current time)
    let timestamp = chrono::Utc::now().timestamp();

    // Sign the request
    let signature = sign_request(secret, "GET", &query.path, &[], timestamp);

    // Build the signed URL (full path with query params)
    let url = format!("{}?sig={}&ts={}", query.path, signature, timestamp);

    // Expiry window (5 minutes = 300 seconds)
    let expires_in: i64 = 300;

    Json(StreamSignResponse {
        url,
        signature,
        timestamp,
        expires_in,
    })
}

/// Create the streaming sign router
pub fn create_stream_sign_router() -> Router<crate::api::AppState> {
    Router::new()
        .route("/api/v1/streams/sign", get(sign_stream_endpoint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_response_serialization() {
        let response = StreamSignResponse {
            url: "/stream/stats?sig=abc&ts=123".to_string(),
            signature: "abc".to_string(),
            timestamp: 123,
            expires_in: 300,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("signature"));
        assert!(json.contains("expires_in"));
    }
}
