//! System Component Trait
//!
//! Provides a unified interface for all system components (managers, pools, services)
//! to ensure consistent lifecycle management, health monitoring, and graceful shutdown.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Common errors for system components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentError {
    pub code: String,
    pub message: String,
}

impl ComponentError {
    pub fn not_initialized(name: &str) -> Self {
        Self {
            code: "NOT_INITIALIZED".to_string(),
            message: format!("Component '{}' is not initialized", name),
        }
    }

    pub fn already_initialized(name: &str) -> Self {
        Self {
            code: "ALREADY_INITIALIZED".to_string(),
            message: format!("Component '{}' is already initialized", name),
        }
    }

    pub fn timeout(operation: &str) -> Self {
        Self {
            code: "TIMEOUT".to_string(),
            message: format!("Operation '{}' timed out", operation),
        }
    }

    pub fn failed(component: &str, reason: &str) -> Self {
        Self {
            code: "FAILED".to_string(),
            message: format!("Component '{}' failed: {}", component, reason),
        }
    }

    pub fn not_running(name: &str) -> Self {
        Self {
            code: "NOT_RUNNING".to_string(),
            message: format!("Component '{}' is not running", name),
        }
    }
}

impl std::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ComponentError {}

/// Health status for a component
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status")]
pub enum HealthStatus {
    /// Component is healthy and operating normally
    Healthy,
    /// Component is degraded but functional
    Degraded { reason: String },
    /// Component is unhealthy and needs attention
    Unhealthy { reason: String },
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

impl ComponentInfo {
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
        }
    }
}

/// Component state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ComponentState {
    Created,
    Initialized,
    Running,
    Stopping,
    Stopped,
    Failed,
}

impl Default for ComponentState {
    fn default() -> Self {
        ComponentState::Created
    }
}

/// Core trait for all system components
///
/// This trait provides a unified interface for managing the lifecycle of
/// all system components in APEX. Components should implement this trait
/// to ensure consistent initialization, startup, shutdown, and health monitoring.
///
/// # Example
///
/// ```rust
/// use async_trait::async_trait;
/// use apex_router::system_component::{SystemComponent, ComponentInfo, ComponentError, HealthStatus, ComponentState};
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
///
/// struct MyManager {
///     state: ComponentState,
///     // ... other fields
/// }
///
/// #[async_trait]
/// impl SystemComponent for MyManager {
///     fn info(&self) -> ComponentInfo {
///         ComponentInfo::new("my-manager", "1.0.0", "My custom manager")
///     }
///
///     async fn initialize(&self) -> Result<(), ComponentError> {
///         // Initialization logic
///         Ok(())
///     }
///
///     async fn start(&self) -> Result<(), ComponentError> {
///         // Start logic
///         Ok(())
///     }
///
///     async fn stop(&self) -> Result<(), ComponentError> {
///         // Stop logic
///         Ok(())
///     }
///
///     async fn health(&self) -> HealthStatus {
///         HealthStatus::Healthy
///     }
///
///     fn is_initialized(&self) -> bool {
///         self.state >= ComponentState::Initialized
///     }
///
///     fn is_running(&self) -> bool {
///         self.state == ComponentState::Running
///     }
///     
///     fn state(&self) -> ComponentState {
///         self.state
///     }
/// }
/// ```
#[async_trait]
pub trait SystemComponent: Send + Sync + 'static {
    /// Get component metadata
    fn info(&self) -> ComponentInfo;

    /// Initialize the component
    ///
    /// This is called once during startup before `start()`.
    /// Use this for:
    /// - Establishing database connections
    /// - Loading configuration
    /// - Setting up internal state
    /// - Creating worker tasks
    async fn initialize(&self) -> Result<(), ComponentError>;

    /// Start the component
    ///
    /// This is called after `initialize()` to begin operations.
    /// Use this for:
    /// - Starting background tasks
    /// - Opening network connections
    /// - Registering with other services
    async fn start(&self) -> Result<(), ComponentError>;

    /// Stop the component gracefully
    ///
    /// This is called during shutdown to stop operations.
    /// Use this for:
    /// - Cancelling background tasks
    /// - Closing connections
    /// - Flushing buffers
    /// - Saving state
    async fn stop(&self) -> Result<(), ComponentError>;

    /// Get the health status of the component
    ///
    /// Returns the current health of the component.
    /// Implementations should perform quick checks only.
    async fn health(&self) -> HealthStatus;

    /// Check if the component has been initialized
    fn is_initialized(&self) -> bool;

    /// Check if the component is running
    fn is_running(&self) -> bool;

    /// Get the current state of the component
    fn state(&self) -> ComponentState;
}

/// Extension trait for working with component references
pub trait ComponentExt: SystemComponent {
    /// Wait until the component is healthy
    async fn wait_healthy(&self, timeout_secs: u64) -> Result<(), ComponentError> {
        let start = std::time::Instant::now();
        while self.health().await != HealthStatus::Healthy {
            if start.elapsed().as_secs() > timeout_secs {
                return Err(ComponentError::timeout("wait_healthy"));
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        Ok(())
    }

    /// Get the component name
    fn name(&self) -> String {
        self.info().name
    }
}

impl<T: SystemComponent> ComponentExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_error_display() {
        let err = ComponentError::not_initialized("test");
        assert_eq!(
            err.to_string(),
            "[NOT_INITIALIZED] Component 'test' is not initialized"
        );
    }

    #[test]
    fn test_health_status_default() {
        assert_eq!(HealthStatus::default(), HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_component_info() {
        let info = ComponentInfo::new("test", "1.0.0", "Test component");
        assert_eq!(info.name, "test");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, "Test component");
    }
}
