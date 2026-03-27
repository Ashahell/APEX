use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

/// Slack Block Kit template
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SlackBlockTemplate {
    pub id: String,
    pub name: String,
    pub template: String, // JSON: Block Kit template
    pub description: Option<String>,
    pub created_at: String,
}

/// Slack Block Template repository
pub struct SlackBlockRepository {
    pool: Pool<Sqlite>,
}

impl SlackBlockRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    /// Get all templates
    pub async fn list_templates(&self) -> Result<Vec<SlackBlockTemplate>, sqlx::Error> {
        sqlx::query_as::<_, SlackBlockTemplate>("SELECT * FROM slack_block_templates ORDER BY name")
            .fetch_all(&self.pool)
            .await
    }

    /// Get template by ID
    pub async fn get_template(&self, id: &str) -> Result<SlackBlockTemplate, sqlx::Error> {
        sqlx::query_as::<_, SlackBlockTemplate>("SELECT * FROM slack_block_templates WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    /// Get template by name
    pub async fn get_by_name(&self, name: &str) -> Result<SlackBlockTemplate, sqlx::Error> {
        sqlx::query_as::<_, SlackBlockTemplate>(
            "SELECT * FROM slack_block_templates WHERE name = ?",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
    }

    /// Create a new template
    pub async fn create_template(
        &self,
        id: &str,
        name: &str,
        template: &str,
        description: Option<&str>,
    ) -> Result<SlackBlockTemplate, sqlx::Error> {
        sqlx::query_as::<_, SlackBlockTemplate>(
            r#"
            INSERT INTO slack_block_templates (id, name, template, description)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(template)
        .bind(description)
        .fetch_one(&self.pool)
        .await
    }

    /// Update template
    pub async fn update_template(
        &self,
        id: &str,
        name: Option<&str>,
        template: Option<&str>,
        description: Option<&str>,
    ) -> Result<SlackBlockTemplate, sqlx::Error> {
        // Get existing
        let existing = self.get_template(id).await?;

        let new_name = name.unwrap_or(&existing.name);
        let new_template = template.unwrap_or(&existing.template);
        let new_description = description.or(existing.description.as_deref());

        sqlx::query_as::<_, SlackBlockTemplate>(
            r#"
            UPDATE slack_block_templates
            SET name = ?, template = ?, description = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(new_name)
        .bind(new_template)
        .bind(new_description)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    /// Delete template
    pub async fn delete_template(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM slack_block_templates WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Render template with variables
    pub fn render_template(
        &self,
        template: &str,
        variables: &serde_json::Value,
    ) -> Result<String, sqlx::Error> {
        let mut result = template.to_string();

        if let serde_json::Value::Object(map) = variables {
            for (key, value) in map {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = match value {
                    serde_json::Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn create_tables(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS slack_block_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            template TEXT NOT NULL,
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        )
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_and_get_template() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SlackBlockRepository::new(&pool);
        let template = repo.create_template(
            "test-1",
            "test_template",
            r#"{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "Hello {{name}}"}}]}"#,
            Some("A test template"),
        ).await.unwrap();

        assert_eq!(template.name, "test_template");

        let fetched = repo.get_template("test-1").await.unwrap();
        assert_eq!(fetched.name, "test_template");
    }

    #[tokio::test]
    async fn test_list_templates() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SlackBlockRepository::new(&pool);

        repo.create_template("t1", "template1", "{}", None)
            .await
            .unwrap();
        repo.create_template("t2", "template2", "{}", None)
            .await
            .unwrap();

        let templates = repo.list_templates().await.unwrap();
        assert_eq!(templates.len(), 2);
    }

    #[tokio::test]
    async fn test_render_template() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        create_tables(&pool).await;

        let repo = SlackBlockRepository::new(&pool);

        let template = r#"{"text": "Hello {{name}}, you have {{count}} tasks"}"#;
        let variables = serde_json::json!({
            "name": "Alice",
            "count": 5
        });

        let rendered = repo.render_template(template, &variables).unwrap();
        assert_eq!(rendered, "{\"text\": \"Hello Alice, you have 5 tasks\"}");
    }
}
