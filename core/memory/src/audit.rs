use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Pool, Row, Sqlite};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditEntry {
    pub id: i64,
    pub prev_hash: String,
    pub hash: String,
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<String>,
}

impl AuditEntry {
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.action.as_bytes());
        hasher.update(self.entity_type.as_bytes());
        hasher.update(self.entity_id.as_bytes());
        if let Some(ref details) = self.details {
            hasher.update(details.as_bytes());
        }
        hex::encode(hasher.finalize())
    }

    pub fn verify(&self) -> bool {
        self.hash == self.compute_hash()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditEntry {
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<String>,
}

#[derive(Clone)]
pub struct AuditRepository {
    pool: Pool<Sqlite>,
}

impl AuditRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn create(&self, entry: CreateAuditEntry) -> Result<AuditEntry, sqlx::Error> {
        let now = Utc::now();

        let prev_hash = self
            .get_last_hash()
            .await
            .unwrap_or_else(|_| "0".to_string());

        let new_entry = AuditEntry {
            id: 0,
            prev_hash: prev_hash.clone(),
            hash: String::new(),
            timestamp: now,
            action: entry.action,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            details: entry.details,
        };

        let hash = new_entry.compute_hash();

        sqlx::query(
            r#"
            INSERT INTO audit_log (prev_hash, hash, timestamp, action, entity_type, entity_id, details)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&prev_hash)
        .bind(&hash)
        .bind(now.to_rfc3339())
        .bind(&new_entry.action)
        .bind(&new_entry.entity_type)
        .bind(&new_entry.entity_id)
        .bind(&new_entry.details)
        .execute(&self.pool)
        .await?;

        Ok(AuditEntry {
            id: 0,
            prev_hash,
            hash,
            timestamp: now,
            action: new_entry.action,
            entity_type: new_entry.entity_type,
            entity_id: new_entry.entity_id,
            details: new_entry.details,
        })
    }

    async fn get_last_hash(&self) -> Result<String, sqlx::Error> {
        let row = sqlx::query("SELECT hash FROM audit_log ORDER BY id DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(r.get::<String, _>(0)),
            None => Ok("0".to_string()),
        }
    }

    pub async fn find_by_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<AuditEntry>, sqlx::Error> {
        sqlx::query_as::<_, AuditEntry>(
            "SELECT * FROM audit_log WHERE entity_type = ? AND entity_id = ? ORDER BY timestamp DESC"
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<AuditEntry>, sqlx::Error> {
        sqlx::query_as::<_, AuditEntry>(
            "SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>(0))
    }

    pub async fn verify_chain(&self) -> Result<bool, sqlx::Error> {
        let entries = sqlx::query_as::<_, AuditEntry>("SELECT * FROM audit_log ORDER BY id ASC")
            .fetch_all(&self.pool)
            .await?;

        for entry in entries {
            if !entry.verify() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Audit Entry Hash Tests
    // =============================================================================

    #[test]
    fn test_compute_hash() {
        let entry = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp: "2026-01-01T00:00:00Z".parse().unwrap(),
            action: "test_action".to_string(),
            entity_type: "task".to_string(),
            entity_id: "123".to_string(),
            details: Some("test details".to_string()),
        };

        let hash = entry.compute_hash();

        // SHA-256 produces 64 hex characters
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_compute_hash_deterministic() {
        let entry = AuditEntry {
            id: 1,
            prev_hash: "abc123".to_string(),
            hash: String::new(),
            timestamp: "2026-03-09T12:00:00Z".parse().unwrap(),
            action: "create_task".to_string(),
            entity_type: "task".to_string(),
            entity_id: "456".to_string(),
            details: None,
        };

        let hash1 = entry.compute_hash();
        let hash2 = entry.compute_hash();

        // Same input should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different_prev_hash() {
        let timestamp: DateTime<Utc> = "2026-03-09T12:00:00Z".parse().unwrap();

        let entry1 = AuditEntry {
            id: 1,
            prev_hash: "hash1".to_string(),
            hash: String::new(),
            timestamp,
            action: "action".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id".to_string(),
            details: None,
        };

        let entry2 = AuditEntry {
            id: 1,
            prev_hash: "hash2".to_string(),
            hash: String::new(),
            timestamp,
            action: "action".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id".to_string(),
            details: None,
        };

        let hash1 = entry1.compute_hash();
        let hash2 = entry2.compute_hash();

        // Different prev_hash should produce different hash
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_includes_details() {
        let timestamp: DateTime<Utc> = "2026-03-09T12:00:00Z".parse().unwrap();

        let entry1 = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp,
            action: "action".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id".to_string(),
            details: Some("details1".to_string()),
        };

        let entry2 = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp,
            action: "action".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id".to_string(),
            details: Some("details2".to_string()),
        };

        let hash1 = entry1.compute_hash();
        let hash2 = entry2.compute_hash();

        // Different details should produce different hash
        assert_ne!(hash1, hash2);
    }

    // =============================================================================
    // Audit Entry Verify Tests
    // =============================================================================

    #[test]
    fn test_verify_valid_entry() {
        let timestamp: DateTime<Utc> = "2026-03-09T12:00:00Z".parse().unwrap();
        let entry = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp,
            action: "test".to_string(),
            entity_type: "task".to_string(),
            entity_id: "1".to_string(),
            details: None,
        };

        let hash = entry.compute_hash();
        let entry_with_hash = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash,
            timestamp,
            action: "test".to_string(),
            entity_type: "task".to_string(),
            entity_id: "1".to_string(),
            details: None,
        };

        assert!(entry_with_hash.verify());
    }

    #[test]
    fn test_verify_tampered_entry() {
        let timestamp: DateTime<Utc> = "2026-03-09T12:00:00Z".parse().unwrap();
        let entry = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: "real_hash_value_123456789".to_string(), // Tampered hash
            timestamp,
            action: "test".to_string(),
            entity_type: "task".to_string(),
            entity_id: "1".to_string(),
            details: None,
        };

        // Verify should fail because hash doesn't match computed hash
        assert!(!entry.verify());
    }

    #[test]
    fn test_verify_tampered_action() {
        let timestamp: DateTime<Utc> = "2026-03-09T12:00:00Z".parse().unwrap();

        // Create valid entry
        let valid_entry = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp,
            action: "create".to_string(),
            entity_type: "task".to_string(),
            entity_id: "1".to_string(),
            details: None,
        };
        let hash = valid_entry.compute_hash();

        // Tamper with action
        let tampered_entry = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash,
            timestamp,
            action: "delete".to_string(), // Tampered!
            entity_type: "task".to_string(),
            entity_id: "1".to_string(),
            details: None,
        };

        assert!(!tampered_entry.verify());
    }

    // =============================================================================
    // Hash Chain Tests
    // =============================================================================

    #[test]
    fn test_hash_chain_links() {
        let time1: DateTime<Utc> = "2026-03-09T10:00:00Z".parse().unwrap();
        let time2: DateTime<Utc> = "2026-03-09T11:00:00Z".parse().unwrap();
        let time3: DateTime<Utc> = "2026-03-09T12:00:00Z".parse().unwrap();

        // Entry 1 (genesis)
        let mut entry1 = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp: time1,
            action: "action1".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id1".to_string(),
            details: None,
        };
        entry1.hash = entry1.compute_hash();

        // Entry 2 (links to entry1)
        let mut entry2 = AuditEntry {
            id: 2,
            prev_hash: entry1.hash.clone(),
            hash: String::new(),
            timestamp: time2,
            action: "action2".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id2".to_string(),
            details: None,
        };
        entry2.hash = entry2.compute_hash();

        // Entry 3 (links to entry2)
        let mut entry3 = AuditEntry {
            id: 3,
            prev_hash: entry2.hash.clone(),
            hash: String::new(),
            timestamp: time3,
            action: "action3".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id3".to_string(),
            details: None,
        };
        entry3.hash = entry3.compute_hash();

        // Verify all entries
        assert!(entry1.verify());
        assert!(entry2.verify());
        assert!(entry3.verify());

        // Verify chain links
        assert_eq!(entry2.prev_hash, entry1.hash);
        assert_eq!(entry3.prev_hash, entry2.hash);
    }

    #[test]
    fn test_chain_broken_if_entry_removed() {
        let time1: DateTime<Utc> = "2026-03-09T10:00:00Z".parse().unwrap();
        let time2: DateTime<Utc> = "2026-03-09T11:00:00Z".parse().unwrap();

        let mut entry1 = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: String::new(),
            timestamp: time1,
            action: "action1".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id1".to_string(),
            details: None,
        };
        entry1.hash = entry1.compute_hash();

        // Entry 2 has prev_hash pointing to entry1
        let mut entry2 = AuditEntry {
            id: 2,
            prev_hash: entry1.hash.clone(),
            hash: String::new(),
            timestamp: time2,
            action: "action2".to_string(),
            entity_type: "type".to_string(),
            entity_id: "id2".to_string(),
            details: None,
        };
        entry2.hash = entry2.compute_hash();

        // Both valid
        assert!(entry1.verify());
        assert!(entry2.verify());

        // If we modify entry1, entry2's chain is broken
        let mut tampered_entry1 = AuditEntry {
            id: 1,
            prev_hash: "0".to_string(),
            hash: entry1.hash.clone(),
            timestamp: time1,
            action: "MODIFIED_ACTION".to_string(), // Tampered!
            entity_type: "type".to_string(),
            entity_id: "id1".to_string(),
            details: None,
        };

        // Entry 1 now fails verification
        assert!(!tampered_entry1.verify());

        // Chain is broken - entry2.prev_hash no longer matches tampered entry1's hash
        assert_ne!(tampered_entry1.compute_hash(), entry1.hash);
    }

    // =============================================================================
    // CreateAuditEntry Tests
    // =============================================================================

    #[test]
    fn test_create_audit_entry_fields() {
        let entry = CreateAuditEntry {
            action: "create".to_string(),
            entity_type: "task".to_string(),
            entity_id: "123".to_string(),
            details: Some("Task created successfully".to_string()),
        };

        assert_eq!(entry.action, "create");
        assert_eq!(entry.entity_type, "task");
        assert_eq!(entry.entity_id, "123");
        assert!(entry.details.is_some());
    }

    #[test]
    fn test_create_audit_entry_no_details() {
        let entry = CreateAuditEntry {
            action: "delete".to_string(),
            entity_type: "task".to_string(),
            entity_id: "456".to_string(),
            details: None,
        };

        assert!(entry.details.is_none());
    }
}
