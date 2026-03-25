# APEX Computer Use Implementation Plan

> **Document Type**: Development Specification  
> **Version**: 1.0  
> **Date**: 2026-03-24  
> **Status**: Planning  

---

## Executive Summary

This document outlines a comprehensive plan to implement **Computer Use** capabilities in APEX—the ability for the autonomous agent to interact with computers through visual understanding (screenshots) and direct manipulation (mouse, keyboard). This positions APEX alongside Anthropic's Claude and OpenAI's Operator as capable of general-purpose desktop automation.

**Target State**: APEX agents can perform multi-step tasks by:
1. Capturing screenshots of the current screen state
2. Analyzing visual information using Vision-Language Models (VLMs)
3. Executing actions (click, type, scroll, keyboard shortcuts)
4. Iterating until tasks complete

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Architecture Design](#2-architecture-design)
3. [Implementation Roadmap](#3-implementation-roadmap)
4. [Component Specifications](#4-component-specifications)
5. [Integration Points](#5-integration-points)
6. [Security Considerations](#6-security-considerations)
7. [Testing Strategy](#7-testing-strategy)
8. [Cost Analysis](#8-cost-analysis)
9. [Milestones](#9-milestones)
10. [Future Enhancements](#10-future-enhancements)

---

## 1. Current State Analysis

### 1.1 Existing Execution Architecture

| Component | Technology | Status |
|-----------|------------|--------|
| **VM Pool** | Docker, Firecracker, gVisor, Mock | ✅ Implemented |
| **Python Sandbox** | Import allowlist, timeout, memory limits | ✅ Implemented |
| **Skill System** | 34 skills (T0-T3 tiers) | ✅ Implemented |
| **Dynamic Tools** | LLM-generated Python with sandbox | ✅ Implemented |
| **Agent Loop** | Plan-Act-Observe-Reflect | ✅ Implemented |

### 1.2 Capabilities Gap

| Capability | Current State | Required |
|------------|---------------|----------|
| Screenshot capture | ❌ None | Required |
| Visual analysis | ❌ None | Required |
| Mouse/keyboard control | ❌ None (shell only) | Required |
| Browser automation | ❌ Basic web.fetch | Enhanced |
| Computer-use model | ❌ None | Required |

### 1.3 Relevant Precedents

| System | Approach | OSWorld Score | Relevance |
|--------|----------|---------------|-----------|
| **Anthropic Claude** | Screenshot + API | N/A | Industry standard |
| **Agent S** | Screenshot + grounding | 72.6% | Open-source leader |
| **Browser-Use** | DOM-based | 89.1% (WebVoyager) | Web automation |
| **OpenCUA** | Training framework | 42%+ | Open-source training |

---

## 2. Architecture Design

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         L6: React UI                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐   │
│  │ TaskSidebar │  │ ProcessGroup│  │ ComputerUseViewer         │   │
│  │             │  │             │  │ (live screenshot stream)   │   │
│  └─────────────┘  └─────────────┘  └─────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    L2: Router (Rust/Axum)                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐   │
│  │ agent_loop.rs   │  │ computer_use.rs│  │ execution_stream   │   │
│  │                 │  │ (NEW)          │  │                    │   │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘   │
│            │                  │                      │              │
│            ▼                  ▼                      ▼              │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │              ComputerUseOrchestrator (NEW)                   │    │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │    │
│  │  │ScreenshotMgr│  │ ActionExec  │  │ VLMController   │   │    │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   L5: Execution Engine                              │
│  ┌─────────────────┐  ┌─────────────────────────────────────────┐   │
│  │ ComputerUseVM  │  │        VM Pool (Docker/Firecracker)    │   │
│  │ (NEW container)│  │                                          │   │
│  │  ┌───────────┐ │  │  ┌────────┐ ┌──────────┐ ┌──────────┐ │   │
│  │  │ Xvfb/Xvnc │ │  │  │  Docker │ │Firecracker│ │ gVisor  │ │   │
│  │  │ (display) │ │  │  └────────┘ └──────────┘ └──────────┘ │   │
│  │  └───────────┘ │  │                                          │   │
│  │  ┌───────────┐ │  │                                          │   │
│  │  │screenshot │ │  │                                          │   │
│  │  │ capture   │ │  │                                          │   │
│  │  └───────────┘ │  │                                          │   │
│  │  ┌───────────┐ │  │                                          │   │
│  │  │  input   │ │  │                                          │   │
│  │  │ simulation│ │  │                                          │   │
│  │  └───────────┘ │  │                                          │   │
│  └─────────────────┘  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    External: VLM Providers                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ Claude 3.5   │  │  GPT-4V     │  │  Local VLM  │              │
│  │ Sonnet       │  │  (Operator)  │  │  (UI-TARS)  │              │
│  └──────────────┘  └──────────────┘  └──────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Overview

| Component | Responsibility | Location |
|-----------|---------------|----------|
| `ComputerUseOrchestrator` | Main state machine, action loop | `core/router/src/computer_use.rs` |
| `ScreenshotManager` | Capture, compression, caching | `core/router/src/computer_use/screenshot.rs` |
| `ActionExecutor` | Mouse, keyboard, command execution | `core/router/src/computer_use/actions.rs` |
| `VLMController` | LLM communication, prompt crafting | `core/router/src/computer_use/vlm.rs` |
| `ComputerUseVM` | Isolated execution environment | `execution/computer_use_vm/` |
| `ComputerUseViewer` | Live streaming to UI | `ui/src/components/computer_use/` |

### 2.3 Execution Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Computer Use Loop                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  1. TASK RECEIVED                                                  │
│     "Book a flight from SF to NYC on delta.com"                    │
│           │                                                         │
│           ▼                                                         │
│  2. CAPTURE SCREENSHOT                                            │
│     ┌──────────────────┐                                            │
│     │ ScreenshotMgr   │                                            │
│     │ - Capture PNG   │                                            │
│     │ - Compress     │                                            │
│     │ - Cache        │                                            │
│     └──────────────────┘                                            │
│           │                                                         │
│           ▼                                                         │
│  3. ANALYZE WITH VLM                                             │
│     ┌──────────────────┐                                            │
│     │ VLMController   │                                            │
│     │ - Build prompt  │                                            │
│     │ - Send to LLM  │                                            │
│     │ - Parse response│                                            │
│     └──────────────────┘                                            │
│           │                                                         │
│           │  "I see the Delta homepage. Need to click             │
│           │   'Round Trip' then enter SFO and JFK..."              │
│           ▼                                                         │
│  4. EXECUTE ACTION                                                 │
│     ┌──────────────────┐                                            │
│     │ ActionExecutor  │                                            │
│     │ - click(x,y)   │                                            │
│     │ - type(text)   │                                            │
│     │ - scroll(down) │                                            │
│     └──────────────────┘                                            │
│           │                                                         │
│           ▼                                                         │
│  5. CHECK COMPLETION                                               │
│     ┌──────────────────┐                                            │
│     │ Orchestrator    │                                            │
│     │ - Max steps?  │                                            │
│     │ - Success?     │                                            │
│     │ - Error?       │                                            │
│     └──────────────────┘                                            │
│           │                                                         │
│     ┌────┴────┐                                                   │
│     │         │                                                   │
│   DONE     REPEAT (step 2)                                        │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-3)

| Week | Task | Deliverable |
|------|------|-------------|
| 1 | Set up isolated VM/container for computer use | `computer_use_vm/` Docker image with Xvfb |
| 1 | Implement screenshot capture | `ScreenshotManager` component |
| 2 | Implement action execution (mouse/keyboard) | `ActionExecutor` with pyautogui |
| 2 | Create VLM integration layer | `VLMController` with Claude API |
| 3 | Build basic orchestration loop | `ComputerUseOrchestrator` state machine |

### Phase 2: Core Features (Weeks 4-6)

| Week | Task | Deliverable |
|------|------|-------------|
| 4 | Implement screenshot streaming to UI | WebSocket feed to `ComputerUseViewer` |
| 4 | Add action retry and error handling | Robustness improvements |
| 5 | Implement browser automation | Playwright integration for web tasks |
| 5 | Add file editing capabilities | str_replace_based_edit_tool equivalent |
| 6 | Security hardening | Isolation verification, audit logging |

### Phase 3: Intelligence (Weeks 7-9)

| Week | Task | Deliverable |
|------|------|-------------|
| 7 | Grounding model integration | UI-TARS for element detection |
| 7 | Prompt optimization | Task-specific prompt templates |
| 8 | Multi-step planning | Chain-of-thought for complex tasks |
| 8 | Self-correction | Error recovery strategies |
| 9 | Performance optimization | Caching, compression, parallelization |

### Phase 4: Production (Weeks 10-12)

| Week | Task | Deliverable |
|------|------|-------------|
| 10 | Comprehensive testing | Integration tests, OSWorld benchmark |
| 10 | Security audit | Penetration testing |
| 11 | Performance tuning | Latency optimization |
| 11 | Documentation | API docs, usage guides |
| 12 | Beta release | Feature flag enabled production |

---

## 4. Component Specifications

### 4.1 ScreenshotManager

```rust
// core/router/src/computer_use/screenshot.rs

pub struct ScreenshotConfig {
    /// Display to capture (0 = primary)
    pub display: u32,
    /// Compression quality (0-100)
    pub quality: u8,
    /// Maximum dimensions (resize if larger)
    pub max_width: u32,
    pub max_height: u32,
    /// Cache duration in milliseconds
    pub cache_ttl_ms: u64,
}

pub struct ScreenshotManager {
    config: ScreenshotConfig,
    cache: Mutex<HashMap<String, (Vec<u8>, Instant)>>,
}

impl ScreenshotManager {
    /// Capture a screenshot from the remote display
    pub async fn capture(&self, display: u32) -> Result<CapturedScreenshot, ScreenshotError> {
        // 1. Connect to display via VNC or Xvfb
        // 2. Capture framebuffer
        // 3. Compress to JPEG/PNG
        // 4. Cache result
    }

    /// Get diff between two screenshots
    pub async fn get_diff(&self, before: &[u8], after: &[u8]) -> Result<ScreenDiff, ScreenshotError> {
        // Use image comparison to detect changes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedScreenshot {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
    pub timestamp: i64,
    pub display: u32,
}
```

### 4.2 ActionExecutor

```rust
// core/router/src/computer_use/actions.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ComputerAction {
    /// Mouse actions
    Click { x: i32, y: i32, button: MouseButton },
    DoubleClick { x: i32, y: i32, button: MouseButton },
    RightClick { x: i32, y: i32 },
    Hover { x: i32, y: i32 },
    Drag { from_x: i32, from_y: i32, to_x: i32, to_y: i32 },
    
    /// Keyboard actions
    Type { text: String },
    KeyPress { key: String, modifiers: Vec<KeyModifier> },
    HotKey { keys: Vec<String> },
    
    /// Scroll
    Scroll { x: i32, y: i32, delta_x: i32, delta_y: i32 },
    
    /// Wait (for page load, animation)
    Wait { duration_ms: u64 },
    
    /// Screenshot
    Screenshot,
    
    /// Shell command (within VM)
    Bash { command: String },
    
    /// File operations
    ReadFile { path: String },
    WriteFile { path: String, content: String },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

pub struct ActionExecutor {
    vm_connection: VmConnector,
    retry_policy: RetryPolicy,
}

impl ActionExecutor {
    pub async fn execute(&self, action: ComputerAction) -> Result<ActionResult, ActionError> {
        match action {
            ComputerAction::Click { x, y, button } => {
                self.mouse_click(x, y, button).await
            }
            ComputerAction::Type { text } => {
                self.keyboard_type(text).await
            }
            // ... other actions
        }
    }
}
```

### 4.3 VLMController

```rust
// core/router/src/computer_use/vlm.rs

pub struct VLMConfig {
    /// Provider: "anthropic", "openai", "google", "local"
    pub provider: VLMProvider,
    /// Model name
    pub model: String,
    /// API key or endpoint
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    /// Maximum tokens in response
    pub max_tokens: u32,
    /// Temperature for sampling
    pub temperature: f32,
}

pub struct VLMController {
    config: VLMConfig,
    client: reqwest::Client,
}

impl VLMController {
    /// Analyze screenshot and propose next action
    pub async fn analyze_and_plan(
        &self,
        screenshot: &CapturedScreenshot,
        task: &str,
        context: &ExecutionContext,
    ) -> Result<VLMResponse, VLMError> {
        // 1. Build prompt with screenshot
        let prompt = self.build_prompt(screenshot, task, context);
        
        // 2. Send to VLM
        let response = self.call_vlm(&prompt).await?;
        
        // 3. Parse response into actions
        let actions = self.parse_actions(&response)?;
        
        Ok(VLMResponse {
            reasoning: response.reasoning,
            actions,
            confidence: response.confidence,
        })
    }

    fn build_prompt(
        &self,
        screenshot: &CapturedScreenshot,
        task: &str,
        context: &ExecutionContext,
    ) -> Prompt {
        // System prompt with capabilities
        let system = include_str!("../prompts/computer_use_system.md");
        
        // Task description
        let user = format!(
            r##"
## Task
{}

## Screen Resolution
{}x{}

## Recent History (last 3 actions)
{}

## Available Actions
- click(x, y, button?)
- double_click(x, y)
- right_click(x, y)
- hover(x, y)
- drag(x1, y1, x2, y2)
- type(text)
- key(key, modifiers?)
- scroll(x, y, delta_x, delta_y)
- wait(ms)
- screenshot
- bash(command)
- read_file(path)
- write_file(path, content)

## Output Format
Provide your reasoning and then the action to take in JSON format.
"##,
            task,
            screenshot.width,
            screenshot.height,
            context.action_history.join("\n"),
        );

        Prompt { system, user }
    }
}
```

### 4.4 ComputerUseOrchestrator

```rust
// core/router/src/computer_use/orchestrator.rs

pub struct ComputerUseOrchestrator {
    screenshot_mgr: ScreenshotManager,
    action_exec: ActionExecutor,
    vlm_ctrl: VLMController,
    config: ComputerUseConfig,
    state: OrchestratorState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputerUseConfig {
    /// Maximum steps per task
    pub max_steps: u32,
    /// Maximum cost in USD
    pub max_cost_usd: f64,
    /// Maximum time in seconds
    pub timeout_secs: u64,
    /// Confidence threshold for actions
    pub confidence_threshold: f32,
    /// Enable screenshot streaming to UI
    pub stream_screenshots: bool,
    /// Retry failed actions
    pub max_retries: u32,
}

pub struct ExecutionContext {
    pub task_id: String,
    pub original_task: String,
    pub action_history: Vec<ActionRecord>,
    pub cost_accumulated: f64,
    pub start_time: Instant,
}

impl ComputerUseOrchestrator {
    pub async fn execute(&mut self, task: &str) -> Result<ExecutionResult, OrchestratorError> {
        let mut context = ExecutionContext::new(task);
        
        for step in 0..self.config.max_steps {
            // Check timeout
            if context.start_time.elapsed().as_secs() > self.config.timeout_secs {
                return Err(OrchestratorError::Timeout);
            }
            
            // Check cost
            if context.cost_accumulated > self.config.max_cost_usd {
                return Err(OrchestratorError::BudgetExceeded);
            }
            
            // Capture screenshot
            let screenshot = self.screenshot_mgr.capture(0).await?;
            
            // Stream to UI if enabled
            if self.config.stream_screenshots {
                self.stream_screenshot(&screenshot).await;
            }
            
            // Get VLM decision
            let response = self.vlm_ctrl.analyze_and_plan(
                &screenshot,
                task,
                &context,
            ).await?;
            
            // Execute action(s)
            for action in response.actions {
                let result = self.action_exec.execute(action).await?;
                
                context.action_history.push(ActionRecord {
                    action: action.clone(),
                    result: result.clone(),
                    timestamp: Utc::now(),
                });
                
                // Update cost
                context.cost_accumulated += result.cost;
                
                // Check for task completion
                if result.is_completion {
                    return Ok(ExecutionResult {
                        success: true,
                        steps: step + 1,
                        cost: context.cost_accumulated,
                        final_state: result.state,
                    });
                }
            }
        }
        
        Err(OrchestratorError::MaxStepsExceeded)
    }
}
```

---

## 5. Integration Points

### 5.1 Skill System Integration

```typescript
// skills/computer.use/src/index.ts

import { Skill, SkillContext, SkillResult } from '../../types';

const computerUseSkill: Skill = {
  name: 'computer.use',
  version: '1.0.0',
  tier: 'T2', // Type to confirm - computer use is potentially destructive
  
  inputSchema: z.object({
    task: z.string().describe('Task to accomplish via computer use'),
    maxSteps: z.number().optional().default(50),
    maxCostUsd: z.number().optional().default(5.0),
    stream: z.boolean().optional().default(true),
  }),
  
  outputSchema: z.object({
    success: z.boolean(),
    steps: z.number(),
    cost: z.number(),
    finalState: z.record(z.any()).optional(),
    error: z.string().optional(),
  }),
  
  async execute(ctx: SkillContext, input: {
    task: string;
    maxSteps?: number;
    maxCostUsd?: number;
    stream?: boolean;
  }): Promise<SkillResult> {
    // Call computer use API
    const response = await fetch('/api/v1/computer-use/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        task: input.task,
        max_steps: input.maxSteps,
        max_cost_usd: input.maxCostUsd,
        stream: input.stream,
      }),
    });
    
    return response.json();
  },
  
  async healthCheck(): Promise<boolean> {
    // Check if computer use VM is running
    const response = await fetch('/api/v1/computer-use/health');
    return response.ok;
  },
};

export default computerUseSkill;
```

### 5.2 API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/computer-use/execute` | POST | Start computer use task |
| `/api/v1/computer-use/status/:id` | GET | Get task status |
| `/api/v1/computer-use/cancel/:id` | POST | Cancel running task |
| `/api/v1/computer-use/stream/:id` | WS | WebSocket for live screenshots |
| `/api/v1/computer-use/screenshots/:id` | GET | Get screenshot history |

### 5.3 Event Stream Integration

Computer use events integrate with existing `ExecutionStream`:

```rust
// Emit computer use events
event_tx.send(ExecutionEvent::ComputerUseStep(ComputerUseStepEvent {
    task_id: task_id.clone(),
    step: context.action_history.len() as u32,
    screenshot_id: screenshot.id.clone(),
    action: action.clone(),
    result: result.clone(),
    vlm_reasoning: response.reasoning.clone(),
})).await;
```

---

## 6. Security Considerations

### 6.1 Threat Model

| Threat | Risk Level | Mitigation |
|--------|------------|------------|
| Prompt injection via screenshots | **Critical** | Input sanitization, content filtering |
| Sensitive data exfiltration | **Critical** | Network isolation, VM-level firewall |
| Unauthorized actions | **High** | T2/T3 confirmation, rate limiting |
| Resource exhaustion | **Medium** | Max steps, timeout, cost limits |
| VM escape | **Critical** | Firecracker + seccomp + gVisor |

### 6.2 Security Controls

```rust
// Isolation configuration for computer use VM

pub fn get_computer_use_vm_config() -> VmConfig {
    VmConfig {
        // Network: Isolated but can reach internet for VLM APIs
        network: NetworkConfig {
            mode: NetworkMode::AllowSpecific(vec![
                // Anthropic API
                "api.anthropic.com:443",
                // Or local VLM
                "localhost:8080",
            ]),
            // Or: NetworkMode::Isolated (no internet)
        },
        
        // Filesystem: Restricted
        filesystem: FilesystemConfig {
            read_only_dirs: vec!["/usr", "/bin", "/lib"],
            writable_dirs: vec!["$HOME/apex_workspace", "/tmp"],
            blocked_paths: vec!["/etc", "/root", "/var/log"],
        },
        
        // Resources
        resources: ResourceConfig {
            cpu_limit: 2,
            memory_limit_mb: 4096,
            timeout_secs: 3600, // 1 hour max
        },
        
        // Security
        security: SecurityConfig {
            seccomp: true,
            cap_drop: vec!["ALL"],
            no_new_privileges: true,
            read_only_rootfs: true,
        },
    }
}
```

### 6.3 Confirmation Gates

| Action Type | Confirmation Required |
|------------|----------------------|
| Click on "Submit"/"Send"/"Buy" | T2 (type confirm) |
| File write to ~/Documents | T1 (tap confirm) |
| File write to system directories | T2 (type confirm) |
| Shell command with sudo | T3 (TOTP) |
| Network request to external domain | T1 (tap confirm) |

---

## 7. Testing Strategy

### 7.1 Unit Tests

| Component | Test Coverage Target |
|-----------|---------------------|
| ScreenshotManager | 80% - capture, cache, diff |
| ActionExecutor | 90% - all action types, error cases |
| VLMController | 80% - prompt building, response parsing |
| Orchestrator | 80% - state machine, timeout, retries |

### 7.2 Integration Tests

```rust
// tests/computer_use_integration.rs

#[tokio::test]
async fn test_complete_web_navigation_task() {
    // Task: "Go to google.com and search for 'APEX AI'"
    
    let orchestrator = create_test_orchestrator().await;
    
    let result = orchestrator.execute("Go to google.com and search for 'APEX AI'").await;
    
    assert!(result.success);
    assert!(result.steps < 20);
    assert!(result.final_state.contains_key("google_search_results"));
}

#[tokio::test]
async fn test_file_creation_task() {
    // Task: "Create a file called test.txt with 'Hello World'"
    
    let orchestrator = create_test_orchestrator().await;
    
    let result = orchestrator.execute("Create a file called test.txt with 'Hello World'").await;
    
    assert!(result.success);
    // Verify file exists in workspace
}
```

### 7.3 Benchmark Testing

| Benchmark | Target | Priority |
|-----------|--------|----------|
| OSWorld (subset) | 30% success | High |
| WebVoyager | 70% success | High |
| Custom APEX tasks | 80% success | Medium |

---

## 8. Cost Analysis

### 8.1 Infrastructure Costs

| Component | Cost Model | Estimated |
|-----------|------------|-----------|
| **VLM API (Claude)** | Per-token | $0.015/1K input (with images) |
| **VLM API (GPT-4V)** | Per-token | $0.01/1K input (with images) |
| **VM Instance** | Per-hour | $0.05-0.20/hour |
| **Storage** | Per-GB | $0.02/GB/month |

### 8.2 Cost Per Task Type

| Task Type | Avg Steps | Avg Cost |
|-----------|-----------|----------|
| Web form fill | 10-20 | $0.50-1.50 |
| File operation | 5-10 | $0.25-0.75 |
| Complex workflow | 30-50 | $2.00-5.00 |

### 8.3 Optimization Strategies

1. **Screenshot caching**: Don't re-capture if no significant change
2. **Local VLM**: Use UI-TARS for element detection (faster, cheaper)
3. **Batch actions**: Execute multiple actions per VLM call
4. **Smart sampling**: Lower resolution for simple tasks

---

## 9. Milestones

### Milestone 1: MVP (Week 3)
- [ ] Screenshot capture working
- [ ] Basic mouse/keyboard control
- [ ] VLM integration (Claude)
- [ ] Simple task execution (open app, type text)
- [ ] Internal demo

### Milestone 2: Beta (Week 6)
- [ ] Web automation (Playwright)
- [ ] File editing
- [ ] Screenshot streaming to UI
- [ ] Error handling and retry
- [ ] Security controls

### Milestone 3: Production (Week 12)
- [ ] Performance optimization
- [ ] Comprehensive testing
- [ ] Security audit
- [ ] Documentation
- [ ] Beta release

---

## 10. Future Enhancements

### 10.1 Advanced Features

| Feature | Description | Priority |
|---------|-------------|----------|
| **DOM-based mode** | Faster web automation using accessibility tree | High |
| **Local VLM** | Run UI-TARS locally for cost savings | High |
| **Multi-display** | Support for multiple monitors | Medium |
| **Recording** | Replay capability for debugging | Medium |
| **Voice** | Audio input for tasks | Low |

### 10.2 Model Improvements

| Enhancement | Expected Improvement |
|-------------|---------------------|
| Grounding model | +20% accuracy on UI elements |
| Chain-of-thought | Better complex task handling |
| Self-correction | Faster error recovery |

### 10.3 Ecosystem Integration

| Integration | Use Case |
|-------------|----------|
| **Browser extensions** | Browser-use MCP server |
| **Desktop apps** | Native desktop automation |
| **Mobile** | Android/iOS emulator control |

---

## Appendix A: Prompt Templates

### System Prompt

```
You are a computer use agent. Your role is to accomplish tasks by interacting with a computer screen.

## Capabilities
- Take screenshots of the current screen
- Move mouse cursor, click, double-click, right-click, drag
- Type text, press keyboard shortcuts
- Scroll, wait for loading
- Execute shell commands
- Read and write files

## Guidelines
1. Start by taking a screenshot to understand the current state
2. Plan your actions to achieve the task efficiently
3. After each action, take a new screenshot to verify the result
4. If something doesn't work, try a different approach
5. Break down complex tasks into simple steps

## Output Format
Always respond with:
1. Reasoning: What you observe and what you plan to do
2. Action: The specific action to take
```

---

## Appendix B: API Reference

### ComputerUseConfig

```typescript
interface ComputerUseConfig {
  // Task constraints
  maxSteps: number;        // Default: 50
  maxCostUsd: number;      // Default: $5.00
  timeoutSecs: number;      // Default: 600 (10 min)
  
  // VLM settings
  vlmProvider: 'anthropic' | 'openai' | 'google' | 'local';
  vlmModel: string;
  temperature: number;     // Default: 0.7
  
  // Display settings
  displayWidth: number;    // Default: 1024
  displayHeight: number;   // Default: 768
  screenshotQuality: number; // 0-100, default: 70
  
  // Behavior
  streamScreenshots: boolean; // Default: true
  confidenceThreshold: number; // Default: 0.8
}
```

---

## Appendix C: Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Screenshot is blank | Display not initialized | Ensure Xvfb/Xvnc is running |
| Click coordinates wrong | Resolution mismatch | Verify display dimensions |
| VLM API errors | Rate limit | Add retry with backoff |
| VM out of memory | Too many screenshots | Reduce cache size |
| Actions not working | Permission denied | Check container capabilities |

---

## References

- [Anthropic Computer Use Documentation](https://docs.anthropic.com/en/docs/build-with-claude/computer-use)
- [OSWorld Benchmark](https://os-world.github.io/)
- [Agent S Repository](https://github.com/simular-ai/agent-s)
- [Browser-Use](https://github.com/browser-use/browser-use)
- [pyautogui](https://pyautogui.readthedocs.io/)

---

*Document Version: 1.0*  
*Last Updated: 2026-03-24*
