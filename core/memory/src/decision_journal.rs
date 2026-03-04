use sqlx::{Pool, Sqlite};

pub struct DecisionJournalRepository {
    pool: Pool<Sqlite>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DecisionJournalEntry {
    pub id: String,
    pub task_id: Option<String>,
    pub title: String,
    pub context: Option<String>,
    pub decision: String,
    pub rationale: Option<String>,
    pub outcome: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct CreateDecisionEntry {
    pub task_id: Option<String>,
    pub title: String,
    pub context: Option<String>,
    pub decision: String,
    pub rationale: Option<String>,
    pub outcome: Option<String>,
    pub tags: Option<Vec<String>>,
}

impl DecisionJournalRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<DecisionJournalEntry>, sqlx::Error> {
        sqlx::query_as::<_, DecisionJournalEntry>(
            "SELECT id, task_id, title, context, decision, rationale, outcome, tags, created_at, updated_at 
             FROM decision_journal ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<DecisionJournalEntry>, sqlx::Error> {
        sqlx::query_as::<_, DecisionJournalEntry>(
            "SELECT id, task_id, title, context, decision, rationale, outcome, tags, created_at, updated_at 
             FROM decision_journal WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_task(&self, task_id: &str) -> Result<Vec<DecisionJournalEntry>, sqlx::Error> {
        sqlx::query_as::<_, DecisionJournalEntry>(
            "SELECT id, task_id, title, context, decision, rationale, outcome, tags, created_at, updated_at 
             FROM decision_journal WHERE task_id = ? ORDER BY created_at DESC"
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn create(&self, id: &str, entry: CreateDecisionEntry) -> Result<(), sqlx::Error> {
        let tags_json = entry.tags.as_ref().map(|t| t.join(","));
        sqlx::query(
            "INSERT INTO decision_journal (id, task_id, title, context, decision, rationale, outcome, tags) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(&entry.task_id)
        .bind(&entry.title)
        .bind(&entry.context)
        .bind(&entry.decision)
        .bind(&entry.rationale)
        .bind(&entry.outcome)
        .bind(&tags_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, id: &str, entry: CreateDecisionEntry) -> Result<(), sqlx::Error> {
        let tags_json = entry.tags.as_ref().map(|t| t.join(","));
        sqlx::query(
            "UPDATE decision_journal SET task_id = ?, title = ?, context = ?, decision = ?, 
             rationale = ?, outcome = ?, tags = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(&entry.task_id)
        .bind(&entry.title)
        .bind(&entry.context)
        .bind(&entry.decision)
        .bind(&entry.rationale)
        .bind(&entry.outcome)
        .bind(&tags_json)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM decision_journal WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn search(&self, query: &str, limit: i64) -> Result<Vec<DecisionJournalEntry>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);
        sqlx::query_as::<_, DecisionJournalEntry>(
            "SELECT id, task_id, title, context, decision, rationale, outcome, tags, created_at, updated_at 
             FROM decision_journal 
             WHERE title LIKE ? OR context LIKE ? OR decision LIKE ? OR rationale LIKE ?
             ORDER BY created_at DESC LIMIT ?"
        )
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_journal_create_and_find() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE decision_journal (id TEXT PRIMARY KEY, task_id TEXT, title TEXT NOT NULL, context TEXT, decision TEXT NOT NULL, rationale TEXT, outcome TEXT, tags TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))")
            .execute(&pool)
            .await
            .unwrap();

        let repo = DecisionJournalRepository::new(&pool);
        let entry = CreateDecisionEntry {
            task_id: Some("task-123".to_string()),
            title: "Test Decision".to_string(),
            context: Some("Testing".to_string()),
            decision: "We decided to test".to_string(),
            rationale: Some("To ensure quality".to_string()),
            outcome: None,
            tags: Some(vec!["test".to_string(), "quality".to_string()]),
        };
        
        repo.create("entry-1", entry).await.unwrap();

        let entries = repo.find_all(10, 0).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Test Decision");
    }

    #[tokio::test]
    async fn test_journal_search() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE decision_journal (id TEXT PRIMARY KEY, task_id TEXT, title TEXT NOT NULL, context TEXT, decision TEXT NOT NULL, rationale TEXT, outcome TEXT, tags TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))")
            .execute(&pool)
            .await
            .unwrap();

        let repo = DecisionJournalRepository::new(&pool);
        let entry = CreateDecisionEntry {
            task_id: None,
            title: "Use PostgreSQL".to_string(),
            context: Some("We need a database".to_string()),
            decision: "Use PostgreSQL".to_string(),
            rationale: Some("Better performance".to_string()),
            outcome: None,
            tags: None,
        };
        
        repo.create("entry-1", entry).await.unwrap();

        let results = repo.search("PostgreSQL", 10).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_journal_update() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query("CREATE TABLE decision_journal (id TEXT PRIMARY KEY, task_id TEXT, title TEXT NOT NULL, context TEXT, decision TEXT NOT NULL, rationale TEXT, outcome TEXT, tags TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')))")
            .execute(&pool)
            .await
            .unwrap();

        let repo = DecisionJournalRepository::new(&pool);
        let entry = CreateDecisionEntry {
            task_id: None,
            title: "Original".to_string(),
            context: None,
            decision: "Original decision".to_string(),
            rationale: None,
            outcome: None,
            tags: None,
        };
        
        repo.create("entry-1", entry).await.unwrap();

        let update_entry = CreateDecisionEntry {
            task_id: None,
            title: "Updated".to_string(),
            context: None,
            decision: "Updated decision".to_string(),
            rationale: None,
            outcome: Some("Worked well".to_string()),
            tags: None,
        };
        
        repo.update("entry-1", update_entry).await.unwrap();

        let result = repo.find_by_id("entry-1").await.unwrap().unwrap();
        assert_eq!(result.title, "Updated");
        assert_eq!(result.outcome, Some("Worked well".to_string()));
    }
}
