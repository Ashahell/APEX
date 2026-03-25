use std::time::{SystemTime, UNIX_EPOCH};

pub struct HmacVerifier {
    secret: String,
}

impl HmacVerifier {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    pub fn generate_signature(
        &self,
        timestamp: i64,
        method: &str,
        path: &str,
        body: &str,
    ) -> String {
        // Simple, deterministic placeholder digest (not real cryptography)
        let data = format!("{}|{}|{}|{}|{}", timestamp, method, path, body, self.secret);
        let mut hash: u64 = 0x9e3779b97f4a7c15; // arbitrary seed
        for b in data.as_bytes() {
            hash = hash.wrapping_mul(0x100000001b3).wrapping_add(*b as u64);
        }
        format!("{:x}", hash)
    }

    pub fn verify(
        &self,
        timestamp: i64,
        method: &str,
        path: &str,
        body: &str,
        signature: &str,
    ) -> Result<(), String> {
        // replay protection window: 5 minutes
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        if (now - timestamp).abs() > 300 {
            return Err("timestamp_out_of_range".to_string());
        }
        let expected = self.generate_signature(timestamp, method, path, body);
        if expected == signature {
            Ok(())
        } else {
            Err("signature_mismatch".to_string())
        }
    }
}
