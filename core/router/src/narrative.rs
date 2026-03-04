use apex_memory::{NarrativeConfig, NarrativeEntry, NarrativeMemory, MemoryStats};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NarrativeService {
    memory: Arc<RwLock<NarrativeMemory>>,
}

impl NarrativeService {
    pub async fn new(config: NarrativeConfig) -> Result<Self, apex_memory::MemoryError> {
        let memory = NarrativeMemory::new(config);
        memory.initialize().await.map_err(|e| apex_memory::MemoryError::Narrative(e.to_string()))?;
        
        Ok(Self {
            memory: Arc::new(RwLock::new(memory)),
        })
    }

    pub async fn record_task(
        &self,
        task_id: &str,
        input_content: &str,
        output_content: Option<&str>,
        status: &str,
        tools_used: &[String],
    ) -> Result<NarrativeEntry, apex_memory::MemoryError> {
        let lessons = self.extract_lessons(status, tools_used);
        
        let memory = self.memory.read().await;
        memory.narrativize_task(
            task_id,
            input_content,
            output_content,
            status,
            tools_used,
            &lessons,
        ).await.map_err(|e| apex_memory::MemoryError::Narrative(e.to_string()))
    }

    fn extract_lessons(&self, status: &str, tools_used: &[String]) -> Vec<String> {
        let mut lessons = Vec::new();
        
        if tools_used.is_empty() && status == "completed" {
            lessons.push("Task completed without tool usage - possibly a direct response".to_string());
        }
        
        if tools_used.len() > 5 {
            lessons.push(format!("Complex task requiring {} tool invocations", tools_used.len()));
        }
        
        match status {
            "completed" => lessons.push("Execution successful".to_string()),
            "failed" => lessons.push("Execution failed - requires review".to_string()),
            "cancelled" => lessons.push("Execution was cancelled".to_string()),
            _ => {}
        }
        
        lessons
    }

    pub async fn add_reflection(&self, title: &str, content: &str) -> Result<std::path::PathBuf, apex_memory::MemoryError> {
        let memory = self.memory.read().await;
        memory.add_reflection(title, content).await.map_err(|e| apex_memory::MemoryError::Narrative(e.to_string()))
    }

    pub async fn add_entity(&self, entity_type: &str, name: &str, content: &str) -> Result<std::path::PathBuf, apex_memory::MemoryError> {
        let memory = self.memory.read().await;
        memory.add_entity(entity_type, name, content).await.map_err(|e| apex_memory::MemoryError::Narrative(e.to_string()))
    }

    pub async fn add_knowledge(&self, category: &str, title: &str, content: &str) -> Result<std::path::PathBuf, apex_memory::MemoryError> {
        let memory = self.memory.read().await;
        memory.add_knowledge(category, title, content).await.map_err(|e| apex_memory::MemoryError::Narrative(e.to_string()))
    }

    pub async fn get_stats(&self) -> Result<MemoryStats, apex_memory::MemoryError> {
        let memory = self.memory.read().await;
        memory.get_memory_stats().await.map_err(|e| apex_memory::MemoryError::Narrative(e.to_string()))
    }
}
