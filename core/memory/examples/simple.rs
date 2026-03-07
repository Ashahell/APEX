use apex_memory::task_repo::TaskRepository;
use apex_memory::tasks::{CreateTask, TaskStatus, TaskTier};
use sqlx::SqlitePool;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = Path::new("test.db");
    if db_path.exists() {
        std::fs::remove_file(db_path)?;
    }

    let pool = SqlitePool::connect(&format!("sqlite:{}?mode=rwc", db_path.display())).await?;

    sqlx::query(
        r#"
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
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            project TEXT,
            priority TEXT DEFAULT 'medium',
            category TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let repo = TaskRepository::new(&pool);

    let task_input = CreateTask {
        input_content: "Hello, world!".to_string(),
        channel: Some("api".to_string()),
        thread_id: None,
        author: Some("test".to_string()),
        skill_name: None,
        project: Some("example-project".to_string()),
        priority: Some("medium".to_string()),
        category: None,
    };

    let task = repo
        .create("test-001", task_input, TaskTier::Shallow)
        .await?;
    println!("Created task: {:?}", task.id);

    let found = repo.find_by_id("test-001").await?;
    println!("Found task: {:?}", found.status);

    repo.update_status("test-001", TaskStatus::Running).await?;

    let updated = repo.find_by_id("test-001").await?;
    println!("Updated status: {:?}", updated.status);

    repo.update_completed(
        "test-001",
        TaskStatus::Completed,
        Some("Done!".to_string()),
        Some(5),  // 5 cents
    )
    .await?;

    let completed = repo.find_by_id("test-001").await?;
    println!("Completed: output={:?}", completed.output_content);

    let count = repo.count().await?;
    println!("Total tasks: {}", count);

    pool.close().await;

    Ok(())
}
