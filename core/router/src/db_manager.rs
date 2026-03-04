use serde::{Deserialize, Serialize};

use crate::unified_config::AppConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    Sqlite,
    PostgreSQL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub connection_string: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self::from_config(&AppConfig::global())
    }

    pub fn from_config(config: &AppConfig) -> Self {
        let db_url = &config.database.connection_string;

        let db_type = if db_url.starts_with("postgres") {
            DatabaseType::PostgreSQL
        } else {
            DatabaseType::Sqlite
        };

        DatabaseConfig {
            db_type,
            connection_string: db_url.clone(),
            max_connections: config.database.max_connections,
            min_connections: config.database.min_connections,
        }
    }

    pub fn is_postgresql(&self) -> bool {
        matches!(self.db_type, DatabaseType::PostgreSQL)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub db_type: String,
    pub total_tasks: i64,
    pub total_messages: i64,
    pub total_cost_cents: i64,
    pub avg_task_duration_ms: i64,
}

#[derive(Clone)]
pub struct DatabaseManager {
    config: DatabaseConfig,
}

impl DatabaseManager {
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> Self {
        Self::new(DatabaseConfig::from_env())
    }

    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    pub fn db_type(&self) -> DatabaseType {
        self.config.db_type.clone()
    }

    pub fn sql_placeholder(&self, index: u32) -> String {
        match self.config.db_type {
            DatabaseType::PostgreSQL => format!("${}", index),
            DatabaseType::Sqlite => "?".to_string(),
        }
    }

    pub fn now_function(&self) -> &'static str {
        match self.config.db_type {
            DatabaseType::PostgreSQL => "NOW()",
            DatabaseType::Sqlite => "datetime('now')",
        }
    }

    pub fn uuid_generate_v4(&self) -> &'static str {
        match self.config.db_type {
            DatabaseType::PostgreSQL => "gen_random_uuid()",
            DatabaseType::Sqlite => "lower(hex(randomblob(16)))",
        }
    }

    pub fn text_column(&self, nullable: bool) -> &'static str {
        match self.config.db_type {
            DatabaseType::PostgreSQL => "TEXT",
            DatabaseType::Sqlite => {
                if nullable {
                    "TEXT"
                } else {
                    "TEXT NOT NULL"
                }
            }
        }
    }

    pub fn json_column(&self) -> &'static str {
        match self.config.db_type {
            DatabaseType::PostgreSQL => "JSONB",
            DatabaseType::Sqlite => "TEXT",
        }
    }

    pub fn boolean_column(&self) -> &'static str {
        match self.config.db_type {
            DatabaseType::PostgreSQL => "BOOLEAN",
            DatabaseType::Sqlite => "INTEGER",
        }
    }

    pub fn get_postgresql_migrations() -> Vec<&'static str> {
        vec![
            r#"
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel VARCHAR(50) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    last_activity TIMESTAMP DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(channel, user_id)
);

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    tier VARCHAR(20) NOT NULL DEFAULT 'instant',
    input_content TEXT NOT NULL,
    output_content TEXT,
    channel VARCHAR(50),
    thread_id VARCHAR(100),
    author VARCHAR(255),
    skill_name VARCHAR(100),
    error_message TEXT,
    cost_estimate_usd REAL,
    actual_cost_usd REAL,
    cost_estimate_cents INTEGER,
    actual_cost_cents INTEGER,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    project VARCHAR(100),
    priority VARCHAR(20),
    category VARCHAR(50)
);

CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID REFERENCES tasks(id),
    channel VARCHAR(50) NOT NULL,
    thread_id VARCHAR(100),
    author VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    role VARCHAR(20) NOT NULL,
    attachments TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS channels (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS decision_journal (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID REFERENCES tasks(id),
    title TEXT NOT NULL,
    context TEXT,
    decision TEXT NOT NULL,
    rationale TEXT,
    outcome TEXT,
    tags TEXT,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS skill_registry (
    name VARCHAR(100) PRIMARY KEY,
    version VARCHAR(20) NOT NULL,
    tier VARCHAR(10) NOT NULL,
    enabled BOOLEAN DEFAULT true,
    health_status VARCHAR(20) DEFAULT 'unknown',
    last_health_check TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_log (
    id SERIAL PRIMARY KEY,
    prev_hash VARCHAR(64) NOT NULL,
    hash VARCHAR(64) NOT NULL,
    timestamp TIMESTAMP DEFAULT NOW(),
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id VARCHAR(100) NOT NULL,
    details TEXT
);

CREATE TABLE IF NOT EXISTS preferences (
    key VARCHAR(100) PRIMARY KEY,
    value TEXT NOT NULL,
    encrypted BOOLEAN DEFAULT false,
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS vector_store (
    id SERIAL PRIMARY KEY,
    key VARCHAR(255) NOT NULL UNIQUE,
    embedding TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_tier ON tasks(tier);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_tasks_actual_cost_cents ON tasks(actual_cost_cents);
CREATE INDEX IF NOT EXISTS idx_messages_task ON messages(task_id);
CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(channel, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_vector_key ON vector_store(key);
CREATE INDEX IF NOT EXISTS idx_decision_journal_task ON decision_journal(task_id);
CREATE INDEX IF NOT EXISTS idx_decision_journal_created ON decision_journal(created_at);
"#,
        ]
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_config() {
        std::env::set_var("APEX_DATABASE_URL", "sqlite:apex.db");
        let config = DatabaseConfig::from_env();
        assert!(!config.is_postgresql());
    }

    #[test]
    fn test_postgresql_config() {
        std::env::set_var("APEX_DATABASE_URL", "postgres://user:pass@localhost/apex");
        let config = DatabaseConfig::from_env();
        assert!(config.is_postgresql());
    }

    #[test]
    fn test_sql_placeholder_sqlite() {
        let config = DatabaseConfig {
            db_type: DatabaseType::Sqlite,
            connection_string: "sqlite:test".to_string(),
            max_connections: 5,
            min_connections: 1,
        };
        let mgr = DatabaseManager::new(config);
        assert_eq!(mgr.sql_placeholder(1), "?");
    }

    #[test]
    fn test_sql_placeholder_postgres() {
        let config = DatabaseConfig {
            db_type: DatabaseType::PostgreSQL,
            connection_string: "postgres://test".to_string(),
            max_connections: 5,
            min_connections: 1,
        };
        let mgr = DatabaseManager::new(config);
        assert_eq!(mgr.sql_placeholder(1), "$1");
        assert_eq!(mgr.sql_placeholder(5), "$5");
    }
}
