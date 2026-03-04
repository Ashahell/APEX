use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: &Path) -> Result<Self, sqlx::Error> {
        let connection_string = format!("sqlite:{}?mode=rwc", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        tracing::info!("Running migrations manually...");
        
        // Migration 001: Initial schema
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                tier TEXT NOT NULL DEFAULT 'instant',
                input_content TEXT NOT NULL,
                output_content TEXT,
                channel TEXT,
                thread_id TEXT,
                author TEXT,
                skill_name TEXT,
                error_message TEXT,
                cost_estimate_cents INTEGER NOT NULL DEFAULT 0,
                actual_cost_cents INTEGER NOT NULL DEFAULT 0,
                started_at TEXT,
                completed_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                prev_hash TEXT NOT NULL,
                hash TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                action TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                details TEXT
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS preferences (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL,
                encrypted INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS skill_registry (
                name TEXT PRIMARY KEY NOT NULL,
                version TEXT NOT NULL,
                tier TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                health_status TEXT NOT NULL DEFAULT 'unknown',
                last_health_check TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY NOT NULL,
                task_id TEXT,
                channel TEXT NOT NULL,
                thread_id TEXT,
                author TEXT NOT NULL,
                content TEXT NOT NULL,
                role TEXT NOT NULL,
                attachments TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS vector_store (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                embedding TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // Migration 002: Kanban fields
        sqlx::query("ALTER TABLE tasks ADD COLUMN project TEXT")
            .execute(&self.pool).await.map_err(|e| {
                if e.to_string().contains("duplicate column") {
                    return sqlx::Error::RowNotFound;
                }
                sqlx::Error::Configuration(Box::new(e))
            }).ok();

        sqlx::query("ALTER TABLE tasks ADD COLUMN priority TEXT DEFAULT 'medium'")
            .execute(&self.pool).await.map_err(|e| {
                if e.to_string().contains("duplicate column") {
                    return sqlx::Error::RowNotFound;
                }
                sqlx::Error::Configuration(Box::new(e))
            }).ok();

        sqlx::query("ALTER TABLE tasks ADD COLUMN category TEXT")
            .execute(&self.pool).await.map_err(|e| {
                if e.to_string().contains("duplicate column") {
                    return sqlx::Error::RowNotFound;
                }
                sqlx::Error::Configuration(Box::new(e))
            }).ok();

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_tier ON tasks(tier)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_category ON tasks(category)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_task ON messages(task_id)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_vector_key ON vector_store(key)")
            .execute(&self.pool).await.ok();

        // Migration 006: Channels and Decision Journal
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS decision_journal (
                id TEXT PRIMARY KEY NOT NULL,
                task_id TEXT,
                title TEXT NOT NULL,
                context TEXT,
                decision TEXT NOT NULL,
                rationale TEXT,
                outcome TEXT,
                tags TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (task_id) REFERENCES tasks(id)
            )
        "#).execute(&self.pool).await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_channels_name ON channels(name)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_decision_journal_task ON decision_journal(task_id)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_decision_journal_created ON decision_journal(created_at)")
            .execute(&self.pool).await.ok();

        // Insert default channels
        sqlx::query("INSERT OR IGNORE INTO channels (id, name, description) VALUES ('default', 'default', 'Default conversation channel')")
            .execute(&self.pool).await.ok();
        sqlx::query("INSERT OR IGNORE INTO channels (id, name, description) VALUES ('general', 'general', 'General discussions')")
            .execute(&self.pool).await.ok();

        // Migration 007: Add currency cents columns
        sqlx::query("ALTER TABLE tasks ADD COLUMN cost_estimate_cents INTEGER")
            .execute(&self.pool).await.ok();
        sqlx::query("ALTER TABLE tasks ADD COLUMN actual_cost_cents INTEGER")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_actual_cost_cents ON tasks(actual_cost_cents)")
            .execute(&self.pool).await.ok();

        tracing::info!("Migrations completed successfully");
        Ok(())
    }
}

pub type SharedDatabase = Arc<RwLock<Option<Database>>>;
