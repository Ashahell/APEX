//! Runbook Module - Automated Remediation Workflows
//! 
//! v1.10.0: Automated Runbook Execution
//! 
//! Runbooks are automated remediation workflows that can be triggered
//! by alerts or executed manually.

pub mod models;
pub mod executor;
pub mod parser;

pub use models::*;
pub use executor::RunbookExecutor;
pub use parser::RunbookParser;

use std::sync::Arc;
use tokio::sync::RwLock;

/// RunbookManager - manages runbooks and their executions
pub struct RunbookManager {
    runbooks: RwLock<Vec<Runbook>>,
    executions: RwLock<Vec<RunbookExecution>>,
    executor: Arc<RunbookExecutor>,
}

impl RunbookManager {
    pub fn new() -> Self {
        Self {
            runbooks: RwLock::new(Vec::new()),
            executions: RwLock::new(Vec::new()),
            executor: Arc::new(RunbookExecutor::new()),
        }
    }

    pub async fn list(&self) -> Vec<Runbook> {
        self.runbooks.read().await.clone()
    }

    pub async fn get(&self, id: &str) -> Option<Runbook> {
        self.runbooks.read().await.iter()
            .find(|r| r.id == id)
            .cloned()
    }

    pub async fn create(&self, runbook: Runbook) {
        self.runbooks.write().await.push(runbook);
    }

    pub async fn update(&self, id: &str, runbook: Runbook) -> Option<()> {
        let mut runbooks = self.runbooks.write().await;
        if let Some(existing) = runbooks.iter_mut().find(|r| r.id == id) {
            *existing = runbook;
            Some(())
        } else {
            None
        }
    }

    pub async fn delete(&self, id: &str) -> Option<()> {
        let mut runbooks = self.runbooks.write().await;
        let pos = runbooks.iter().position(|r| r.id == id)?;
        runbooks.remove(pos);
        Some(())
    }

    pub async fn execute(&self, runbook_id: &str, alert_id: Option<String>) -> Option<RunbookExecution> {
        let runbook = self.get(runbook_id).await?;
        if !runbook.enabled {
            return None;
        }

        let execution = self.executor.execute(&runbook, alert_id).await.ok()?;
        
        self.executions.write().await.push(execution.clone());
        
        Some(execution)
    }

    pub async fn get_executions(&self, runbook_id: &str) -> Vec<RunbookExecution> {
        self.executions.read().await
            .iter()
            .filter(|e| e.runbook_id == runbook_id)
            .cloned()
            .collect()
    }

    pub async fn get_execution(&self, execution_id: &str) -> Option<RunbookExecution> {
        self.executions.read().await
            .iter()
            .find(|e| e.id == execution_id)
            .cloned()
    }
}

impl Default for RunbookManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runbook_manager_crud() {
        let manager = RunbookManager::new();
        
        // Create
        let runbook = Runbook::new(
            "test-runbook".to_string(),
            "Test Description".to_string(),
            vec![],
        );
        manager.create(runbook.clone()).await;
        
        // List
        let runbooks = manager.list().await;
        assert_eq!(runbooks.len(), 1);
        
        // Get
        let found = manager.get(&runbook.id).await;
        assert!(found.is_some());
        
        // Delete
        let deleted = manager.delete(&runbook.id).await;
        assert!(deleted.is_some());
        
        let runbooks = manager.list().await;
        assert_eq!(runbooks.len(), 0);
    }
}
