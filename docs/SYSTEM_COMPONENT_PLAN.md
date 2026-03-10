# SystemComponent Trait - Implementation Plan

> **Status**: Planning  
> **Version**: 1.3.0  
> **Created**: 2026-03-10

---

## Overview

This document outlines the phased implementation of a `SystemComponent` trait to unify component lifecycle management across APEX.

### Problem Statement

Currently, APEX has 15+ components (managers, pools, services) that each implement their own lifecycle methods inconsistently:

| Component | Constructor | Init | Start | Stop | Health |
|-----------|------------|------|-------|------|--------|
| `SkillPool` | `new()` | - | - | - | `stats()` |
| `VmPool` | `new()` | `initialize()` | - | `shutdown()` | - |
| `McpServerManager` | `new()` | - | - | - | `health_check()` |
| `NarrativeService` | `new()` | - | - | - | - |
| `TotpManager` | `new()` | - | - | - | - |
| `SkillPluginManager` | `new()` | - | - | - | - |

### Goals

1. **Unified Lifecycle** - All components follow same init/start/stop pattern
2. **Health Monitoring** - Standard health check interface
3. **Dependency Injection** - Components can declare dependencies
4. **Graceful Shutdown** - Ordered shutdown with timeout

---

## Phase 1: Define Trait (Day 1)

### Objectives
- Create the `SystemComponent` trait
- Define core methods
- Add documentation

### Deliverables
```
src/
  system_component.rs  (NEW)
```

### API Design

```rust
use async_trait::async_trait;
use std::sync::Arc;

/// Common errors for system components
#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("Component not initialized: {0}")]
    NotInitialized(String),
    #[error("Component already initialized")]
    AlreadyInitialized,
    #[error("Operation timed out: {0}")]
    Timeout(String),
    #[error("Component failed: {0}")]
    Failed(String),
}

/// Health status for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Component metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Core trait for all system components
#[async_trait]
pub trait SystemComponent: Send + Sync {
    /// Component metadata
    fn info(&self) -> ComponentInfo;
    
    /// Initialize component (constructor phase)
    async fn initialize(&self) -> Result<(), ComponentError>;
    
    /// Start component (begin operations)
    async fn start(&self) -> Result<(), ComponentError>;
    
    /// Stop component (graceful shutdown)
    async fn stop(&self) -> Result<(), ComponentError>;
    
    /// Health check
    async fn health(&self) -> HealthStatus;
    
    /// Check if initialized
    fn is_initialized(&self) -> bool;
    
    /// Check if running
    fn is_running(&self) -> bool;
}
```

---

## Phase 2: Implement Trait for Core Components (Day 1-2)

### Objectives
- Implement trait for 3-5 core components
- Test trait implementation

### Components to Refactor

| Priority | Component | File | Complexity |
|----------|-----------|------|------------|
| 1 | `SkillPool` | `skill_pool.rs` | Medium |
| 2 | `VmPool` | `vm_pool.rs` | Medium |
| 3 | `McpServerManager` | `mcp/server.rs` | High |
| 4 | `NarrativeService` | `narrative.rs` | Low |
| 5 | `TotpManager` | `totp.rs` | Low |

### Deliverables
- Updated component implementations
- Unit tests for trait methods

---

## Phase 3: Component Registry (Day 2)

### Objectives
- Create a registry to track all components
- Enable ordered startup/shutdown

### API Design

```rust
/// Registry for managing system components
pub struct ComponentRegistry {
    components: HashMap<String, Arc<dyn SystemComponent>>,
    startup_order: Vec<String>,
}

impl ComponentRegistry {
    pub fn new() -> Self;
    
    /// Register a component
    pub fn register(&mut self, component: Arc<dyn SystemComponent>) -> Result<(), ComponentError>;
    
    /// Initialize all components in order
    pub async fn initialize_all(&self) -> Result<(), ComponentError>;
    
    /// Start all components in order
    pub async fn start_all(&self) -> Result<(), ComponentError>;
    
    /// Stop all components in reverse order
    pub async fn stop_all(&self) -> Result<(), ComponentError>;
    
    /// Get health of all components
    pub async fn health_all(&self) -> HashMap<String, HealthStatus>;
}
```

### Deliverables
```
src/
  component_registry.rs  (NEW)
```

---

## Phase 4: Integration with Main (Day 3)

### Objectives
- Update main.rs to use ComponentRegistry
- Add health endpoint

### Changes to main.rs

```rust
// Create component registry
let registry = ComponentRegistry::new();

// Register components
registry.register(skill_pool).await?;
registry.register(vm_pool).await?;
registry.register(mcp_manager).await?;

// Initialize all
registry.initialize_all().await?;

// Start all
registry.start_all().await?;

// Graceful shutdown handler
tokio::signal::ctrl_c().await {
    registry.stop_all().await?;
}
```

### Deliverables
- Updated `main.rs`
- Health endpoint `/api/v1/system/components`

---

## Phase 5: Advanced Features (Optional, Day 4+)

### Objectives
- Add dependency resolution
- Add metrics collection

### Features

1. **Dependency Injection**
   ```rust
   #[async_trait]
   trait SystemComponent: Send + Sync {
       fn dependencies(&self) -> Vec<&'static str>;
   }
   ```

2. **Metrics**
   ```rust
   async fn metrics(&self) -> ComponentMetrics;
   ```

3. **Readiness Probe**
   ```rust
   async fn ready(&self) -> bool;
   ```

---

## Implementation Checklist

### Phase 1: Trait Definition
- [ ] Create `system_component.rs`
- [ ] Define `ComponentError`, `HealthStatus`, `ComponentInfo`
- [ ] Define `SystemComponent` trait
- [ ] Add tests for trait definition

### Phase 2: Core Components
- [ ] Implement for `SkillPool`
- [ ] Implement for `VmPool`
- [ ] Implement for `McpServerManager`
- [ ] Implement for `NarrativeService`
- [ ] Implement for `TotpManager`

### Phase 3: Registry
- [ ] Create `ComponentRegistry`
- [ ] Implement registration
- [ ] Implement ordered startup/shutdown
- [ ] Add tests

### Phase 4: Integration
- [ ] Update `main.rs`
- [ ] Add health endpoint
- [ ] Test graceful shutdown
- [ ] Update documentation

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes to existing APIs | High | Implement trait, don't change existing methods |
| Complex async interactions | Medium | Add integration tests |
| Performance regression | Low | Benchmark before/after |

---

## Timeline

- **Phase 1**: 1 day
- **Phase 2**: 1-2 days
- **Phase 3**: 1 day
- **Phase 4**: 1 day
- **Total**: 4-5 days

---

*Plan Version: 1.0*
