use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct SkillHotReload {
    watcher: Arc<RwLock<Option<RecommendedWatcher>>>,
    skills_dir: PathBuf,
    reload_callback: Arc<RwLock<Option<Box<dyn Fn(String) + Send + Sync>>>>,
}

impl SkillHotReload {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            watcher: Arc::new(RwLock::new(None)),
            skills_dir,
            reload_callback: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start<F>(&self, callback: F) -> Result<(), String>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let skills_dir = self.skills_dir.clone();
        
        *self.reload_callback.write().await = Some(Box::new(callback));

        let callback = self.reload_callback.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    if let Some(path) = event.paths.first() {
                        if path.extension().map_or(false, |ext| ext == "md") {
                            if let Some(file_name) = path.file_name() {
                                let skill_name = file_name.to_string_lossy().replace(".md", "");
                                if let Ok(callback) = callback.try_read() {
                                    if let Some(cb) = callback.as_ref() {
                                        cb(skill_name.clone());
                                        tracing::info!(skill = %skill_name, "Skill hot-reloaded");
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Config::default(),
        ).map_err(|e| format!("Failed to create watcher: {}", e))?;

        watcher.watch(&skills_dir, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;

        *self.watcher.write().await = Some(watcher);
        
        tracing::info!(path = %skills_dir.display(), "Skill hot-reload started");
        Ok(())
    }

    pub async fn stop(&self) {
        *self.watcher.write().await = None;
        *self.reload_callback.write().await = None;
        tracing::info!("Skill hot-reload stopped");
    }

    pub fn is_watching(&self) -> bool {
        // Check if watcher exists
        self.watching()
    }

    fn watching(&self) -> bool {
        // Simple check - if we can get a read lock, we're watching
        self.watcher.try_read().map_or(false, |w| w.is_some())
    }
}

impl Default for SkillHotReload {
    fn default() -> Self {
        Self::new(PathBuf::from("./skills"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_skill_hot_reload_new() {
        let temp_dir = TempDir::new().unwrap();
        let reload = SkillHotReload::new(PathBuf::from(temp_dir.path()));
        assert!(!reload.is_watching());
    }
}
