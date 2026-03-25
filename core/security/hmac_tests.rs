#[cfg(test)]
mod tests {
    use super::super::hmac::HmacVerifier;
    use std::time::{SystemTime, UNIX_EPOCH};

    // ---------------------------------------------------------------------------
    // Core HMAC tests
    // ---------------------------------------------------------------------------

    #[test]
    fn valid_signature_passes() {
        let secret = "secret";
        let hv = HmacVerifier::new(secret);
        let ts = 1i64;
        let sig = hv.generate_signature(ts, "POST", "/execute", "{}");
        assert!(hv.verify(ts, "POST", "/execute", "{}", &sig).is_ok());
    }

    #[test]
    fn expired_timestamp_fails() {
        let secret = "secret";
        let hv = HmacVerifier::new(secret);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let ts_old = now - 1000; // older than 5 minutes window
        let sig = hv.generate_signature(ts_old, "GET", "/health", "");
        assert!(hv.verify(ts_old, "GET", "/health", "", &sig).is_err());
    }

    #[test]
    fn signature_mismatch_fails() {
        let hv = HmacVerifier::new("secret");
        let ts = 1234i64;
        let sig = hv.generate_signature(ts, "POST", "/health", "{}");
        let bad = sig + "X";
        assert!(hv.verify(ts, "POST", "/health", "{}", &bad).is_err());
    }

    // ---------------------------------------------------------------------------
    // Patch 11: Streaming-specific HMAC tests
    // ---------------------------------------------------------------------------

    #[test]
    fn valid_stream_signature_passes() {
        // Simulate a streaming request: GET /api/v1/stream/hands/task-123, empty body
        let secret = "dev-secret-change-in-production";
        let hv = HmacVerifier::new(secret);
        let ts = chrono::Utc::now().timestamp();
        let path = "/api/v1/stream/hands/task-abc";
        let body = "";
        let sig = hv.generate_signature(ts, "GET", path, body);
        assert!(hv.verify(ts, "GET", path, body, &sig).is_ok());
    }

    #[test]
    fn stream_signature_rejects_different_method() {
        // Streaming uses GET; POST should not produce a valid stream signature
        let hv = HmacVerifier::new("secret");
        let ts = chrono::Utc::now().timestamp();
        let sig = hv.generate_signature(ts, "GET", "/stream/hands/123", "");
        // Verify with GET — should pass
        assert!(hv.verify(ts, "GET", "/stream/hands/123", "", &sig).is_ok());
        // Verify with POST — same timestamp, same path, but different method
        assert!(hv
            .verify(ts, "POST", "/stream/hands/123", "", &sig)
            .is_err());
    }

    #[test]
    fn stream_signature_rejects_different_path() {
        // A signature for /stream/hands/task-A should not work for /stream/hands/task-B
        let hv = HmacVerifier::new("secret");
        let ts = chrono::Utc::now().timestamp();
        let sig = hv.generate_signature(ts, "GET", "/stream/hands/task-A", "");
        assert!(hv
            .verify(ts, "GET", "/stream/hands/task-A", "", &sig)
            .is_ok());
        // Different task_id in path
        assert!(hv
            .verify(ts, "GET", "/stream/hands/task-B", "", &sig)
            .is_err());
    }

    #[test]
    fn stream_signature_rejects_future_timestamp() {
        let hv = HmacVerifier::new("secret");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        // Timestamp 10 minutes in the future (outside 5-min window)
        let ts_future = now + 600;
        let sig = hv.generate_signature(ts_future, "GET", "/stream/mcp/task-1", "");
        assert!(hv
            .verify(ts_future, "GET", "/stream/mcp/task-1", "", &sig)
            .is_err());
    }

    #[test]
    fn stream_signature_rejects_past_timestamp() {
        let hv = HmacVerifier::new("secret");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        // Timestamp 10 minutes in the past (outside 5-min window)
        let ts_past = now - 600;
        let sig = hv.generate_signature(ts_past, "GET", "/stream/task/xyz", "");
        assert!(hv
            .verify(ts_past, "GET", "/stream/task/xyz", "", &sig)
            .is_err());
    }

    #[test]
    fn stream_signature_at_boundary_of_window() {
        let hv = HmacVerifier::new("secret");
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        // Exactly 4 minutes 59 seconds in the past — within the 5-min window
        let ts_boundary = now - (4 * 60 + 59);
        let sig = hv.generate_signature(ts_boundary, "GET", "/stream/hands/test", "");
        assert!(hv
            .verify(ts_boundary, "GET", "/stream/hands/test", "", &sig)
            .is_ok());
    }

    #[test]
    fn different_secrets_produce_different_signatures() {
        let hv1 = HmacVerifier::new("secret-1");
        let hv2 = HmacVerifier::new("secret-2");
        let ts = chrono::Utc::now().timestamp();
        let path = "/api/v1/stream/hands/test";
        let sig1 = hv1.generate_signature(ts, "GET", path, "");
        let sig2 = hv2.generate_signature(ts, "GET", path, "");
        assert_ne!(
            sig1, sig2,
            "different secrets should produce different signatures"
        );
        // sig1 should only verify with hv1
        assert!(hv1.verify(ts, "GET", path, "", &sig1).is_ok());
        // sig1 should NOT verify with hv2 (wrong secret)
        assert!(hv2.verify(ts, "GET", path, "", &sig1).is_err());
    }

    #[test]
    fn stream_signature_is_deterministic() {
        // Same inputs should always produce the same signature
        let hv = HmacVerifier::new("deterministic-test");
        let ts = 1_742_985_600i64; // fixed timestamp
        let path = "/stream/hands/test-task";
        let body = "";
        let sig1 = hv.generate_signature(ts, "GET", path, body);
        let sig2 = hv.generate_signature(ts, "GET", path, body);
        assert_eq!(
            sig1, sig2,
            "signature should be deterministic for same inputs"
        );
    }
}
