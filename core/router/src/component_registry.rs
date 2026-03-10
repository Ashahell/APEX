//! Component Registry
//! 
//! Provides centralized management for all system components.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::system_component::{ComponentError, ComponentInfo, HealthStatus, SystemComponent};

/// Registry for managing system components
/// 
/// This struct provides centralized lifecycle management for all system components.
/// It maintains registration order and ensures proper startup/shutdown sequencing.
pub struct ComponentRegistry {
    components: RwLock<HashMap<String, Arc<dyn SystemComponent>>>,
    startup_order: RwLock<Vec<String>>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    /// Create a new component registry
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            startup_order: RwLock::new(Vec::new()),
        }
    }

    /// Register a component
    /// 
    /// Components are stored in registration order for proper startup/shutdown.
    pub async fn register(&self, component: Arc<dyn SystemComponent>) -> Result<(), ComponentError> {
        let info = component.info();
        let name = info.name.clone();
        
        let mut components = self.components.write().await;
        let mut order = self.startup_order.write().await;
        
        if components.contains_key(&name) {
            return Err(ComponentError::already_initialized(&name));
        }
        
        components.insert(name.clone(), component);
        order.push(name);
        
        info!("Component registered: {}", info.name);
        Ok(())
    }

    /// Get a component by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn SystemComponent>> {
        let components = self.components.read().await;
        components.get(name).cloned()
    }

    /// Get all component names
    pub async fn names(&self) -> Vec<String> {
        let order = self.startup_order.read().await;
        order.clone()
    }

    /// Initialize all components in registration order
    pub async fn initialize_all(&self) -> Result<(), ComponentError> {
        let order = self.startup_order.read().await;
        
        for name in order.iter() {
            let components = self.components.read().await;
            if let Some(component) = components.get(name) {
                info!("Initializing component: {}", name);
                match component.initialize().await {
                    Ok(_) => info!("Component initialized: {}", name),
                    Err(e) => {
                        error!("Failed to initialize component {}: {}", name, e);
                        return Err(e);
                    }
                }
            }
        }
        
        info!("All components initialized");
        Ok(())
    }

    /// Start all components in registration order
    pub async fn start_all(&self) -> Result<(), ComponentError> {
        let order = self.startup_order.read().await;
        
        for name in order.iter() {
            let components = self.components.read().await;
            if let Some(component) = components.get(name) {
                info!("Starting component: {}", name);
                match component.start().await {
                    Ok(_) => info!("Component started: {}", name),
                    Err(e) => {
                        error!("Failed to start component {}: {}", name, e);
                        return Err(e);
                    }
                }
            }
        }
        
        info!("All components started");
        Ok(())
    }

    /// Stop all components in reverse registration order
    pub async fn stop_all(&self) -> Result<(), ComponentError> {
        let order = self.startup_order.read().await;
        
        // Stop in reverse order
        for name in order.iter().rev() {
            let components = self.components.read().await;
            if let Some(component) = components.get(name) {
                info!("Stopping component: {}", name);
                match component.stop().await {
                    Ok(_) => info!("Component stopped: {}", name),
                    Err(e) => {
                        error!("Failed to stop component {}: {}", name, e);
                        // Continue stopping other components even if one fails
                    }
                }
            }
        }
        
        info!("All components stopped");
        Ok(())
    }

    /// Get health status of all components
    pub async fn health_all(&self) -> HashMap<String, HealthStatus> {
        let mut result = HashMap::new();
        let components = self.components.read().await;
        
        for (name, component) in components.iter() {
            let health = component.health().await;
            result.insert(name.clone(), health);
        }
        
        result
    }

    /// Get info for all components
    pub async fn info_all(&self) -> Vec<ComponentInfo> {
        let components = self.components.read().await;
        components.values().map(|c| c.info()).collect()
    }

    /// Wait for all components to be healthy
    pub async fn wait_all_healthy(&self, timeout_secs: u64) -> Result<(), ComponentError> {
        let order = self.startup_order.read().await;
        let start = std::time::Instant::now();
        
        for name in order.iter() {
            let components = self.components.read().await;
            if let Some(component) = components.get(name) {
                while component.health().await != HealthStatus::Healthy {
                    if start.elapsed().as_secs() > timeout_secs {
                        return Err(ComponentError::timeout(&format!("wait_healthy for {}", name)));
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
        
        Ok(())
    }

    /// Get a summary of the registry
    pub async fn summary(&self) -> ComponentRegistrySummary {
        let components = self.components.read().await;
        let order = self.startup_order.read().await;
        
        let mut states = HashMap::new();
        let mut healths = HashMap::new();
        
        for (name, component) in components.iter() {
            states.insert(name.clone(), component.state());
            healths.insert(name.clone(), component.health().await);
        }
        
        ComponentRegistrySummary {
            component_count: components.len(),
            startup_order: order.clone(),
            states,
            healths,
        }
    }
}

/// Summary of the component registry
#[derive(Debug, serde::Serialize)]
pub struct ComponentRegistrySummary {
    pub component_count: usize,
    pub startup_order: Vec<String>,
    pub states: HashMap<String, crate::system_component::ComponentState>,
    pub healths: HashMap<String, HealthStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use crate::system_component::ComponentState;

    #[derive(Debug)]
    struct MockComponent {
        name: String,
        state: std::sync::atomic::AtomicU8,
    }

    #[async_trait]
    impl SystemComponent for MockComponent {
        fn info(&self) -> ComponentInfo {
            ComponentInfo::new(&self.name, "1.0.0", "Mock component")
        }

        async fn initialize(&self) -> Result<(), ComponentError> {
            self.state.store(ComponentState::Initialized as u8, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn start(&self) -> Result<(), ComponentError> {
            self.state.store(ComponentState::Running as u8, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn stop(&self) -> Result<(), ComponentError> {
            self.state.store(ComponentState::Stopped as u8, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn health(&self) -> HealthStatus {
            HealthStatus::Healthy
        }

        fn is_initialized(&self) -> bool {
            self.state.load(std::sync::atomic::Ordering::SeqCst) >= ComponentState::Initialized as u8
        }

        fn is_running(&self) -> bool {
            self.state.load(std::sync::atomic::Ordering::SeqCst) == ComponentState::Running as u8
        }

        fn state(&self) -> ComponentState {
            ComponentState::from_u8(self.state.load(std::sync::atomic::Ordering::SeqCst))
        }
    }

    #[tokio::test]
    async fn test_registry_registration() {
        let registry = ComponentRegistry::new();
        
        let comp1 = Arc::new(MockComponent {
            name: "comp1".to_string(),
            state: std::sync::atomic::AtomicU8::new(0),
        });
        
        let comp2 = Arc::new(MockComponent {
            name: "comp2".to_string(),
            state: std::sync::atomic::AtomicU8::new(0),
        });
        
        registry.register(comp1).await.unwrap();
        registry.register(comp2).await.unwrap();
        
        let names = registry.names().await;
        assert_eq!(names, vec!["comp1", "comp2"]);
    }

    #[tokio::test]
    async fn test_lifecycle() {
        let registry = ComponentRegistry::new();
        
        let comp1 = Arc::new(MockComponent {
            name: "comp1".to_string(),
            state: std::sync::atomic::AtomicU8::new(0),
        });
        
        registry.register(comp1.clone()).await.unwrap();
        registry.initialize_all().await.unwrap();
        registry.start_all().await.unwrap();
        
        assert!(comp1.is_initialized());
        assert!(comp1.is_running());
        
        registry.stop_all().await.unwrap();
        
        let summary = registry.summary().await;
        assert_eq!(summary.states.get("comp1"), Some(&ComponentState::Stopped));
    }
}
