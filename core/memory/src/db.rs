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

        let db = Self { pool };
        
        db.run_migrations().await?;
        db.configure_pragma().await?;

        Ok(db)
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

        // Migration 013: Enhanced Memory System
        // memory_chunks: Chunked text from memory files
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memory_chunks (
                id TEXT PRIMARY KEY NOT NULL,
                file_path TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                memory_type TEXT NOT NULL,
                task_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                accessed_at TEXT NOT NULL DEFAULT (datetime('now')),
                access_count INTEGER NOT NULL DEFAULT 0,
                UNIQUE(file_path, chunk_index)
            )
        "#).execute(&self.pool).await?;

        // memory_fts: FTS5 for BM25 keyword search
        sqlx::query(r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
                content,
                memory_type,
                tokenize='porter unicode61'
            )
        "#).execute(&self.pool).await?;

        // Triggers to keep FTS in sync with memory_chunks
        sqlx::query(r#"
            CREATE TRIGGER IF NOT EXISTS memory_chunks_ai AFTER INSERT ON memory_chunks BEGIN
                INSERT INTO memory_fts(rowid, content, memory_type)
                VALUES (new.rowid, new.content, new.memory_type);
            END
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TRIGGER IF NOT EXISTS memory_chunks_ad AFTER DELETE ON memory_chunks BEGIN
                INSERT INTO memory_fts(memory_fts, rowid, content, memory_type)
                VALUES ('delete', old.rowid, old.content, old.memory_type);
            END
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TRIGGER IF NOT EXISTS memory_chunks_au AFTER UPDATE ON memory_chunks BEGIN
                INSERT INTO memory_fts(memory_fts, rowid, content, memory_type)
                VALUES ('delete', old.rowid, old.content, old.memory_type);
                INSERT INTO memory_fts(rowid, content, memory_type)
                VALUES (new.rowid, new.content, new.memory_type);
            END
        "#).execute(&self.pool).await?;

        // memory_entities: Lightweight entity store
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memory_entities (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                attributes TEXT NOT NULL DEFAULT '{}',
                first_seen TEXT NOT NULL DEFAULT (datetime('now')),
                last_updated TEXT NOT NULL DEFAULT (datetime('now')),
                mention_count INTEGER NOT NULL DEFAULT 1
            )
        "#).execute(&self.pool).await?;

        sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS idx_memory_entities_name_type ON memory_entities(name, entity_type)")
            .execute(&self.pool).await.ok();

        // memory_index_state: Tracks which files have been indexed
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memory_index_state (
                file_path TEXT PRIMARY KEY NOT NULL,
                mtime_unix INTEGER NOT NULL,
                chunk_count INTEGER NOT NULL,
                indexed_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // working_memory: Per-task scratchpad
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS working_memory (
                task_id TEXT PRIMARY KEY NOT NULL,
                scratchpad TEXT NOT NULL DEFAULT '',
                entities_json TEXT NOT NULL DEFAULT '{}',
                causal_links_json TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
        "#).execute(&self.pool).await?;

        // memory_vec: Vector storage (fallback - stores embeddings as JSON)
        // Note: sqlite-vec (vec0) provides better performance but requires extension loading
        // This table provides basic functionality without the extension
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS memory_vec (
                chunk_id TEXT PRIMARY KEY NOT NULL,
                embedding TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        // Create indexes for enhanced memory
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_chunks_type ON memory_chunks(memory_type)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_chunks_task ON memory_chunks(task_id)")
            .execute(&self.pool).await.ok();
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_memory_chunks_accessed ON memory_chunks(accessed_at DESC)")
            .execute(&self.pool).await.ok();

        tracing::info!("Migrations completed successfully");
        Ok(())
    }

    pub async fn configure_pragma(&self) -> Result<(), sqlx::Error> {
        // Enable WAL mode for better concurrency (D1 recommendation)
        sqlx::query("PRAGMA journal_mode=WAL")
            .execute(&self.pool).await?;

        // Safe with WAL, faster than FULL
        sqlx::query("PRAGMA synchronous=NORMAL")
            .execute(&self.pool).await?;

        // 64MB cache
        sqlx::query("PRAGMA cache_size=-64000")
            .execute(&self.pool).await?;

        // Temp tables in memory
        sqlx::query("PRAGMA temp_store=MEMORY")
            .execute(&self.pool).await?;

        tracing::info!("SQLite pragma configured: WAL mode enabled");
        Ok(())
    }
}

pub type SharedDatabase = Arc<RwLock<Option<Database>>>;
