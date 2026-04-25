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

    pub async fn seed_default_triggers(&self) -> Result<(), sqlx::Error> {
        let default_triggers = vec![
            ("shell.execute", "run command", 3),
            ("shell.execute", "execute shell", 3),
            ("shell.execute", "run terminal", 3),
            ("shell.execute", "bash", 2),
            ("shell.execute", "terminal", 2),
            ("code.generate", "write code", 3),
            ("code.generate", "generate code", 3),
            ("code.generate", "implement", 2),
            ("code.generate", "create function", 2),
            ("code.review", "review code", 3),
            ("code.review", "code review", 3),
            ("code.review", "check code", 2),
            ("code.refactor", "refactor", 3),
            ("code.refactor", "improve code", 2),
            ("code.format", "format code", 3),
            ("code.format", "prettier", 2),
            ("code.format", "lint", 2),
            ("code.document", "document code", 3),
            ("code.document", "add docs", 2),
            ("code.test", "write test", 3),
            ("code.test", "create test", 3),
            ("code.test", "unit test", 2),
            ("file.search", "find file", 3),
            ("file.search", "search files", 3),
            ("file.search", "locate file", 2),
            ("file.delete", "delete file", 3),
            ("file.delete", "remove file", 2),
            ("git.commit", "git commit", 3),
            ("git.commit", "commit changes", 3),
            ("git.branch", "git branch", 3),
            ("git.branch", "create branch", 2),
            ("git.force_push", "force push", 3),
            ("git.force_push", "git push --force", 3),
            ("docker.build", "build docker", 3),
            ("docker.build", "docker image", 2),
            ("docker.run", "run docker", 3),
            ("docker.run", "docker run", 3),
            ("db.migrate", "run migration", 3),
            ("db.migrate", "database migration", 3),
            ("db.schema", "database schema", 3),
            ("db.schema", "design database", 2),
            ("db.drop", "drop database", 3),
            ("api.design", "design api", 3),
            ("api.design", "create endpoint", 2),
            ("api.test", "test api", 3),
            ("api.test", "api testing", 2),
            ("docs.read", "read docs", 3),
            ("docs.read", "documentation", 2),
            ("seo.optimize", "seo", 3),
            ("seo.optimize", "search engine", 2),
            ("deploy.kubectl", "kubectl", 3),
            ("deploy.kubectl", "deploy kubernetes", 3),
            ("aws.lambda", "lambda", 3),
            ("aws.lambda", "aws serverless", 2),
            ("ci.configure", "ci cd", 3),
            ("ci.configure", "github actions", 2),
            ("repo.search", "search repository", 3),
            ("repo.search", "find code", 2),
        ];

        for (skill, keyword, weight) in default_triggers {
            self.add_trigger(skill, keyword, weight).await.ok();
        }

        Ok(())
    }
}
