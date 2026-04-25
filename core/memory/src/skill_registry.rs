use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillRegistryEntry {
    pub name: String,
    pub version: String,
    pub tier: String,
    pub enabled: bool,
    pub health_status: String,
    pub last_health_check: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillTrigger {
    pub id: String,
    pub skill_name: String,
    pub keyword: String,
    pub weight: i32,
    pub created_at: String,
}

impl SkillRegistryEntry {
    pub fn new(name: String, version: String, tier: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            name,
            version,
            tier,
            enabled: true,
            health_status: "unknown".to_string(),
            last_health_check: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

pub struct SkillRegistry<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SkillRegistry<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, entry: &SkillRegistryEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO skill_registry (name, version, tier, enabled, health_status, last_health_check, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                version = excluded.version,
                tier = excluded.tier,
                enabled = excluded.enabled,
                health_status = excluded.health_status,
                last_health_check = excluded.last_health_check,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&entry.name)
        .bind(&entry.version)
        .bind(&entry.tier)
        .bind(entry.enabled as i32)
        .bind(&entry.health_status)
        .bind(&entry.last_health_check)
        .bind(&entry.created_at)
        .bind(&entry.updated_at)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<SkillRegistryEntry>, sqlx::Error> {
        let result =
            sqlx::query_as::<_, SkillRegistryEntry>("SELECT * FROM skill_registry WHERE name = ?")
                .bind(name)
                .fetch_optional(self.pool)
                .await?;

        Ok(result)
    }

    pub async fn find_all(&self) -> Result<Vec<SkillRegistryEntry>, sqlx::Error> {
        sqlx::query_as::<_, SkillRegistryEntry>("SELECT * FROM skill_registry ORDER BY name")
            .fetch_all(self.pool)
            .await
    }

    pub async fn find_enabled(&self) -> Result<Vec<SkillRegistryEntry>, sqlx::Error> {
        sqlx::query_as::<_, SkillRegistryEntry>(
            "SELECT * FROM skill_registry WHERE enabled = 1 ORDER BY name",
        )
        .fetch_all(self.pool)
        .await
    }

    pub async fn update_health(&self, name: &str, health_status: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE skill_registry SET health_status = ?, last_health_check = ?, updated_at = ? WHERE name = ?",
        )
        .bind(health_status)
        .bind(&now)
        .bind(&now)
        .bind(name)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_enabled(&self, name: &str, enabled: bool) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query("UPDATE skill_registry SET enabled = ?, updated_at = ? WHERE name = ?")
            .bind(enabled as i32)
            .bind(&now)
            .bind(name)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete(&self, name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM skill_registry WHERE name = ?")
            .bind(name)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_trigger(&self, skill_name: &str, keyword: &str, weight: i32) -> Result<(), sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO skill_triggers (id, skill_name, keyword, weight, created_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(skill_name, keyword) DO UPDATE SET weight = excluded.weight
            "#,
        )
        .bind(&id)
        .bind(skill_name)
        .bind(keyword)
        .bind(weight)
        .bind(&now)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn find_triggers_by_keyword(&self, query: &str) -> Result<Vec<SkillTrigger>, sqlx::Error> {
        let pattern = format!("%{}%", query.to_lowercase());
        sqlx::query_as::<_, SkillTrigger>(
            "SELECT * FROM skill_triggers WHERE LOWER(keyword) LIKE ? ORDER BY weight DESC"
        )
        .bind(&pattern)
        .fetch_all(self.pool)
        .await
    }

    pub async fn find_triggers_for_skill(&self, skill_name: &str) -> Result<Vec<SkillTrigger>, sqlx::Error> {
        sqlx::query_as::<_, SkillTrigger>(
            "SELECT * FROM skill_triggers WHERE skill_name = ? ORDER BY weight DESC"
        )
        .bind(skill_name)
        .fetch_all(self.pool)
        .await
    }

    pub async fn delete_trigger(&self, skill_name: &str, keyword: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM skill_triggers WHERE skill_name = ? AND keyword = ?")
            .bind(skill_name)
            .bind(keyword)
            .execute(self.pool)
            .await?;

        Ok(())
    }

    pub async fn delete_triggers_for_skill(&self, skill_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM skill_triggers WHERE skill_name = ?")
            .bind(skill_name)
            .execute(self.pool)
            .await?;

        Ok(())
    }
}
