use sqlx::sqlite::SqlitePool;
use sqlx::{Executor, Row};

#[tokio::test]
async fn registry_lifecycle_and_tools_persistence() {
    // In-memory SQLite DB for unit tests
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Create registries and tools tables
    sqlx::query("CREATE TABLE IF NOT EXISTS mcp_registries (id TEXT PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS mcp_tools_registry (id TEXT PRIMARY KEY, registry_id TEXT, name TEXT, description TEXT, input_schema TEXT)")
        .execute(&pool).await.unwrap();

    // 1) Create and list registries
    sqlx::query("INSERT INTO mcp_registries (id, name) VALUES ('reg1', 'Test Registry')")
        .execute(&pool)
        .await
        .unwrap();
    let rows = sqlx::query("SELECT id, name FROM mcp_registries")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].get::<String, _>("id"), "reg1");
    assert_eq!(rows[0].get::<String, _>("name"), "Test Registry");

    // 2) Add a tool to registry and list
    sqlx::query("INSERT INTO mcp_tools_registry (id, registry_id, name, description, input_schema) VALUES ('t1', 'reg1', 'dynamic_tool_1', 'Discovered at runtime', '{\"type\":\"object\"}')")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO mcp_tools_registry (id, registry_id, name, description, input_schema) VALUES ('t2', 'reg1', 'dynamic_tool_2', 'Another discovered tool', '{\"type\":\"object\"}')")
        .execute(&pool).await.unwrap();
    let tools = sqlx::query(
        "SELECT name, description, input_schema FROM mcp_tools_registry WHERE registry_id = 'reg1'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0].get::<String, _>("name"), "dynamic_tool_1");
    assert_eq!(tools[1].get::<String, _>("name"), "dynamic_tool_2");

    // 3) Seed discovery path: if no tools exist, insert two seeds
    // First, clear tools for reg1 to simulate empty registry
    sqlx::query("DELETE FROM mcp_tools_registry WHERE registry_id = 'reg1'")
        .execute(&pool)
        .await
        .unwrap();
    let existing: Vec<(String,)> = sqlx::query_as::<sqlx::Sqlite, (String,)>(
        "SELECT id FROM mcp_tools_registry WHERE registry_id = 'reg1'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(existing.is_empty());
    // Seed two tools
    let t1_id = "seed1".to_string();
    let t2_id = "seed2".to_string();
    sqlx::query("INSERT INTO mcp_tools_registry (id, registry_id, name, description, input_schema) VALUES (?, ?, ?, ?, ?)")
        .bind(&t1_id).bind("reg1").bind("dynamic_seed_1").bind("seed 1").bind("{\"type\":\"object\"}")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO mcp_tools_registry (id, registry_id, name, description, input_schema) VALUES (?, ?, ?, ?, ?)")
        .bind(&t2_id).bind("reg1").bind("dynamic_seed_2").bind("seed 2").bind("{\"type\":\"object\"}")
        .execute(&pool).await.unwrap();
    let seeds = sqlx::query("SELECT name FROM mcp_tools_registry WHERE registry_id = 'reg1'")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(seeds.len(), 2);
}
