use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeEntry {
    pub id: String,
    pub task_id: String,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeConfig {
    pub base_path: PathBuf,
    pub retention_days: u32,
    pub forgetting_threshold_days: u32,
}

impl Default for NarrativeConfig {
    fn default() -> Self {
        Self {
            base_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".apex")
                .join("memory"),
            retention_days: 90,
            forgetting_threshold_days: 30,
        }
    }
}

pub struct NarrativeMemory {
    config: NarrativeConfig,
}

impl NarrativeMemory {
    pub fn new(config: NarrativeConfig) -> Self {
        Self { config }
    }

    pub async fn initialize(&self) -> std::io::Result<()> {
        let dirs = [
            self.config.base_path.join("journal"),
            self.config.base_path.join("entities"),
            self.config.base_path.join("entities").join("agents"),
            self.config.base_path.join("knowledge"),
            self.config.base_path.join("knowledge").join("technical"),
            self.config.base_path.join("knowledge").join("institutional"),
            self.config.base_path.join("reflections"),
        ];

        for dir in dirs {
            fs::create_dir_all(&dir).await?;
        }

        Ok(())
    }

    pub async fn narrativize_task(
        &self,
        task_id: &str,
        input_content: &str,
        output_content: Option<&str>,
        status: &str,
        tools_used: &[String],
        lessons: &[String],
    ) -> std::io::Result<NarrativeEntry> {
        let now = Utc::now();
        let date_path = format!("{}/{}/", now.format("%Y"), now.format("%m"));
        let journal_dir = self.config.base_path.join("journal").join(&date_path);
        fs::create_dir_all(&journal_dir).await?;

        let file_name = format!("{}-{}.md", now.format("%d-%H%M%S"), &task_id[..8.min(task_id.len())]);
        let file_path = journal_dir.join(&file_name);

        let summary = self.generate_summary(input_content, status);
        let narrative = self.build_narrative(
            task_id,
            input_content,
            output_content,
            status,
            tools_used,
            lessons,
            now,
        );

        let mut file = fs::File::create(&file_path).await?;
        file.write_all(narrative.as_bytes()).await?;
        file.flush().await?;

        Ok(NarrativeEntry {
            id: uuid::Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            path: file_path,
            created_at: now,
            summary,
        })
    }

    fn generate_summary(&self, input_content: &str, status: &str) -> String {
        let preview = if input_content.len() > 100 {
            format!("{}...", &input_content[..100])
        } else {
            input_content.to_string()
        };
        format!("[{}] {}", status, preview)
    }

    fn build_narrative(
        &self,
        task_id: &str,
        input_content: &str,
        output_content: Option<&str>,
        status: &str,
        tools_used: &[String],
        lessons: &[String],
        timestamp: DateTime<Utc>,
    ) -> String {
        let mut narrative = String::new();
        
        narrative.push_str(&format!("# Task Narrative: {}\n\n", task_id));
        narrative.push_str(&format!("**Date**: {}\n", timestamp.format("%Y-%m-%d %H:%M UTC")));
        narrative.push_str(&format!("**Status**: {}\n\n", status));
        
        narrative.push_str("## Context\n\n");
        narrative.push_str(input_content);
        narrative.push_str("\n\n");

        if let Some(output) = output_content {
            narrative.push_str("## What Happened\n\n");
            if output.len() > 2000 {
                narrative.push_str(&output[..2000]);
                narrative.push_str("\n\n_[Output truncated]_\n\n");
            } else {
                narrative.push_str(output);
                narrative.push_str("\n\n");
            }
        }

        if !tools_used.is_empty() {
            narrative.push_str("## Tools Used\n\n");
            for tool in tools_used {
                narrative.push_str(&format!("- {}\n", tool));
            }
            narrative.push_str("\n");
        }

        if !lessons.is_empty() {
            narrative.push_str("## What I Learned\n\n");
            for lesson in lessons {
                narrative.push_str(&format!("- {}\n", lesson));
            }
            narrative.push_str("\n");
        }

        narrative.push_str("## Reflection\n\n");
        narrative.push_str(&self.generate_reflection(status, tools_used, lessons));
        narrative.push_str("\n");

        narrative
    }

    fn generate_reflection(&self, status: &str, tools_used: &[String], lessons: &[String]) -> String {
        let mut reflection = String::new();

        match status {
            "completed" => {
                reflection.push_str("This task was completed successfully. ");
                if !tools_used.is_empty() {
                    reflection.push_str(&format!(
                        "The execution used {} tool(s). ",
                        tools_used.len()
                    ));
                }
            }
            "failed" => {
                reflection.push_str("This task encountered difficulties. ");
                reflection.push_str("Consider reviewing the error messages and adjusting the approach. ");
            }
            "cancelled" => {
                reflection.push_str("This task was cancelled before completion. ");
            }
            _ => {
                reflection.push_str("The task is still in progress or in an unknown state. ");
            }
        }

        if !lessons.is_empty() {
            reflection.push_str("Key insights have been captured for future reference.");
        }

        reflection
    }

    pub async fn add_reflection(&self, title: &str, content: &str) -> std::io::Result<PathBuf> {
        let now = Utc::now();
        let reflections_dir = self.config.base_path.join("reflections");
        fs::create_dir_all(&reflections_dir).await?;

        let file_name = format!(
            "{}-{}.md",
            now.format("%Y-%m-%d"),
            title.to_lowercase().replace(' ', "-")
        );
        let file_path = reflections_dir.join(&file_name);

        let mut file = fs::File::create(&file_path).await?;
        file.write_all(format!("# {}\n\n**Date**: {}\n\n{}\n", title, now.format("%Y-%m-%d %H:%M UTC"), content).as_bytes()).await?;
        file.flush().await?;

        Ok(file_path)
    }

    pub async fn add_entity(&self, entity_type: &str, name: &str, content: &str) -> std::io::Result<PathBuf> {
        let entity_dir = self.config.base_path.join("entities").join(entity_type);
        fs::create_dir_all(&entity_dir).await?;

        let file_name = format!("{}.md", name.to_lowercase().replace(' ', "-"));
        let file_path = entity_dir.join(&file_name);

        let mut file = fs::File::create(&file_path).await?;
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        Ok(file_path)
    }

    pub async fn add_knowledge(&self, category: &str, title: &str, content: &str) -> std::io::Result<PathBuf> {
        let knowledge_dir = self.config.base_path.join("knowledge").join(category);
        fs::create_dir_all(&knowledge_dir).await?;

        let file_name = format!("{}.md", title.to_lowercase().replace(' ', "-"));
        let file_path = knowledge_dir.join(&file_name);

        let mut file = fs::File::create(&file_path).await?;
        file.write_all(content.as_bytes()).await?;
        file.flush().await?;

        Ok(file_path)
    }

    pub async fn read_journal(&self, date: Option<DateTime<Utc>>) -> std::io::Result<Vec<PathBuf>> {
        let journal_dir = self.config.base_path.join("journal");
        
        let target_date = date.unwrap_or_else(Utc::now);
        let date_path = journal_dir.join(target_date.format("%Y").to_string())
            .join(target_date.format("%m").to_string());

        if !date_path.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let mut dir = fs::read_dir(&date_path).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                entries.push(path);
            }
        }

        entries.sort();
        Ok(entries)
    }

    pub async fn get_memory_stats(&self) -> std::io::Result<MemoryStats> {
        let journal_count = self.count_files_in(&self.config.base_path.join("journal")).await?;
        let entities_count = self.count_files_in(&self.config.base_path.join("entities")).await?;
        let knowledge_count = self.count_files_in(&self.config.base_path.join("knowledge")).await?;
        let reflections_count = self.count_files_in(&self.config.base_path.join("reflections")).await?;

        Ok(MemoryStats {
            journal_entries: journal_count,
            entities: entities_count,
            knowledge_items: knowledge_count,
            reflections: reflections_count,
            total_files: journal_count + entities_count + knowledge_count + reflections_count,
        })
    }

    async fn count_files_in(&self, dir: &PathBuf) -> std::io::Result<u32> {
        let mut count = 0_u32;
        
        if !dir.exists() {
            return Ok(0);
        }

        let mut stack = vec![dir.clone()];
        
        while let Some(current_dir) = stack.pop() {
            let mut entries = fs::read_dir(&current_dir).await?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().map_or(false, |ext| ext == "md") {
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub journal_entries: u32,
    pub entities: u32,
    pub knowledge_items: u32,
    pub reflections: u32,
    pub total_files: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[tokio::test]
    async fn test_narrative_memory_initialization() {
        let temp_dir = temp_dir().join(format!("test_narrative_{}", uuid::Uuid::new_v4()));
        let config = NarrativeConfig {
            base_path: temp_dir.clone(),
            retention_days: 90,
            forgetting_threshold_days: 30,
        };
        
        let memory = NarrativeMemory::new(config);
        memory.initialize().await.unwrap();
        
        assert!(temp_dir.join("journal").exists());
        assert!(temp_dir.join("entities").exists());
        assert!(temp_dir.join("knowledge").exists());
        assert!(temp_dir.join("reflections").exists());
    }

    #[tokio::test]
    async fn test_narrativize_task() {
        let temp_dir = temp_dir().join(format!("test_narrative_{}", uuid::Uuid::new_v4()));
        let config = NarrativeConfig {
            base_path: temp_dir.clone(),
            ..Default::default()
        };
        
        let memory = NarrativeMemory::new(config);
        memory.initialize().await.unwrap();
        
        let entry = memory.narrativize_task(
            "task-123",
            "Test task input",
            Some("Test task output"),
            "completed",
            &["tool1".to_string(), "tool2".to_string()],
            &["lesson1".to_string()],
        ).await.unwrap();
        
        assert!(entry.path.exists());
        assert_eq!(entry.task_id, "task-123");
    }
}
