use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Sqlite};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Option<String>,
    pub env: Option<String>,
    pub enabled: bool,
    pub status: String,
    pub last_error: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct McpTool {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<String>,
}

#[derive(Clone)]
pub struct ConfigRepository {
    pool: Pool<Sqlite>,
}

impl ConfigRepository {
    pub fn new(pool: &Pool<Sqlite>) -> Self {
        Self { pool: pool.clone() }
    }

    pub async fn get(&self, key: &str) -> Result<Option<ConfigEntry>, sqlx::Error> {
        sqlx::query_as::<_, ConfigEntry>("SELECT key, value, created_at, updated_at FROM config_store WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO config_store (key, value, updated_at)
            VALUES (?, ?, datetime('now'))
            ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = datetime('now')
            "#
        )
        .bind(key)
        .bind(value)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM config_store WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_all(&self) -> Result<Vec<ConfigEntry>, sqlx::Error> {
        sqlx::query_as::<_, ConfigEntry>("SELECT key, value, created_at, updated_at FROM config_store")
            .fetch_all(&self.pool)
            .await
    }

    // MCP Server methods
    pub async fn get_mcp_servers(&self) -> Result<Vec<McpServer>, sqlx::Error> {
        sqlx::query_as::<_, McpServer>(
            "SELECT id, name, command, args, env, enabled, status, last_error, created_at, updated_at FROM mcp_servers ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_mcp_server(&self, id: &str) -> Result<Option<McpServer>, sqlx::Error> {
        sqlx::query_as::<_, McpServer>(
            "SELECT id, name, command, args, env, enabled, status, last_error, created_at, updated_at FROM mcp_servers WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn save_mcp_server(&self, server: &McpServer) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO mcp_servers (id, name, command, args, env, enabled, status, last_error, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
            ON CONFLICT(id) DO UPDATE SET 
                name = ?, command = ?, args = ?, env = ?, enabled = ?, status = ?, last_error = ?, updated_at = datetime('now')
            "#
        )
        .bind(&server.id)
        .bind(&server.name)
        .bind(&server.command)
        .bind(&server.args)
        .bind(&server.env)
        .bind(server.enabled as i32)
        .bind(&server.status)
        .bind(&server.last_error)
        .bind(&server.name)
        .bind(&server.command)
        .bind(&server.args)
        .bind(&server.env)
        .bind(server.enabled as i32)
        .bind(&server.status)
        .bind(&server.last_error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_mcp_server(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_mcp_server_status(&self, id: &str, status: &str, error: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE mcp_servers SET status = ?, last_error = ?, updated_at = datetime('now') WHERE id = ?"
        )
        .bind(status)
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // MCP Tools methods
    pub async fn get_mcp_tools(&self, server_id: &str) -> Result<Vec<McpTool>, sqlx::Error> {
        sqlx::query_as::<_, McpTool>(
            "SELECT id, server_id, name, description, input_schema FROM mcp_tools WHERE server_id = ?"
        )
        .bind(server_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_all_mcp_tools(&self) -> Result<Vec<McpTool>, sqlx::Error> {
        sqlx::query_as::<_, McpTool>(
            "SELECT id, server_id, name, description, input_schema FROM mcp_tools"
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn save_mcp_tool(&self, tool: &McpTool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO mcp_tools (id, server_id, name, description, input_schema)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET name = ?, description = ?, input_schema = ?
            "#
        )
        .bind(&tool.id)
        .bind(&tool.server_id)
        .bind(&tool.name)
        .bind(&tool.description)
        .bind(&tool.input_schema)
        .bind(&tool.name)
        .bind(&tool.description)
        .bind(&tool.input_schema)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_mcp_tools_for_server(&self, server_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM mcp_tools WHERE server_id = ?")
            .bind(server_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
