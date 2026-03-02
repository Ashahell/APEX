use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, Row, SqlitePool};

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

pub struct AuditRepository<'a> {
    pool: &'a SqlitePool,
}

impl<'a> AuditRepository<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, entry: CreateAuditEntry) -> Result<AuditEntry, sqlx::Error> {
        let now = Utc::now();
        
        let prev_hash = self.get_last_hash().await.unwrap_or_else(|_| "0".to_string());
        
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
        
        let result = sqlx::query(
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
        .execute(self.pool)
        .await?;

        let id = result.last_insert_rowid();
        
        Ok(AuditEntry {
            id,
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
            .fetch_optional(self.pool)
            .await?;
        
        match row {
            Some(r) => Ok(r.get::<String, _>(0)),
            None => Ok("0".to_string()),
        }
    }

    pub async fn find_by_entity(&self, entity_type: &str, entity_id: &str) -> Result<Vec<AuditEntry>, sqlx::Error> {
        sqlx::query_as::<_, AuditEntry>(
            "SELECT * FROM audit_log WHERE entity_type = ? AND entity_id = ? ORDER BY timestamp DESC"
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn verify_chain(&self) -> Result<bool, sqlx::Error> {
        let entries = sqlx::query_as::<_, AuditEntry>(
            "SELECT * FROM audit_log ORDER BY id ASC"
        )
        .fetch_all(self.pool)
        .await?;

        for entry in entries {
            if !entry.verify() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}
