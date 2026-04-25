# APEX Enhancement Plan: Hermes-Inspired Features

**Version**: 1.0  
**Date**: 2026-03-21  
**Based on**: Hermes Agent v0.3.0 analysis  
**Status**: ✅ COMPLETE - All features implemented in v1.5.0

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Feature Matrix](#feature-matrix)
3. [Phase 1: Bounded Curated Memory](#phase-1-bounded-curated-memory)
4. [Phase 2: Agent-Managed Skills](#phase-2-agent-managed-skills)
5. [Phase 3: Skills Hub Integration](#phase-3-skills-hub-integration)
6. [Phase 4: Session Search Enhancement](#phase-4-session-search-enhancement)
7. [Phase 5: Honcho-Style User Modeling](#phase-5-honcho-style-user-modeling)
8. [UI Components](#ui-components)
9. [Wiring Diagrams](#wiring-diagrams)
10. [Constants Reference](#constants-reference)

---

## Executive Summary

### Current State vs Target

| Aspect | Current APEX | Target (Hermes) |
|--------|--------------|-----------------|
| Memory | Vector + narrative (unbounded) | Bounded curated (2,200/1,375 chars) |
| Skills | 33 static skills | Agent creates skills from experience |
| Skill Discovery | Manual installation | 7 marketplace sources |
| Session Search | Vector similarity | FTS5 + LLM summarization |
| User Modeling | None | Cross-session Honcho |

### Implementation Approach

1. **No God Code**: Each feature is self-contained with clear interfaces
2. **No Magic Numbers**: All limits defined in `config_constants` module
3. **Incremental**: Phases can be implemented independently
4. **UI-First**: Every feature has corresponding UI components

---

## Feature Matrix

| Feature | Priority | Complexity | Files to Change |
|---------|----------|------------|----------------|
| Bounded Memory | P0 | Medium | memory.rs, appStore.ts, Memory.tsx |
| Agent-Managed Skills | P0 | High | agent_loop.rs, skill_manage.rs, Skills.tsx |
| Progressive Disclosure | P1 | Medium | skills/loader.rs, Skills.tsx |
| Skills Hub | P1 | High | hub_client.rs, Skills.tsx, Marketplace.tsx |
| Session Search FTS5 | P2 | Medium | memory/db.rs, Search.tsx |
| User Profiles | P2 | Medium | user_profile.rs, Settings.tsx |
| Slash Commands | P2 | Low | Chat.tsx, CommandPalette.tsx |

---

## Phase 1: Bounded Curated Memory

### Overview

Implement Hermes-style bounded memory with:
- **MEMORY.md**: Agent's notes (configurable limit)
- **USER.md**: User profile (configurable limit)
- **Frozen snapshot pattern** for prefix caching
- **Character limits** with automatic consolidation

### Constants to Add

**File**: `core/router/src/unified_config.rs`

```rust
pub mod memory_constants {
    // Memory limits (in characters)
    pub const DEFAULT_MEMORY_CHAR_LIMIT: usize = 2_200;      // ~800 tokens
    pub const DEFAULT_USER_CHAR_LIMIT: usize = 1_375;       // ~500 tokens
    
    // Capacity thresholds
    pub const MEMORY_WARNING_THRESHOLD: f32 = 0.80;          // 80% capacity warning
    pub const MEMORY_CRITICAL_THRESHOLD: f32 = 0.95;         // 95% - force consolidation
    
    // Entry constraints
    pub const MAX_ENTRY_LENGTH: usize = 500;                 // Max single entry
    pub const MIN_ENTRY_LENGTH: usize = 10;                  // Min entry to be useful
    pub const ENTRY_DELIMITER: &str = "§";
}
```

### Task 1.1: Create Memory Stores Module

**File**: `core/router/src/memory_stores.rs` (new)

```rust
// core/router/src/memory_stores.rs

mod memory_constants {
    pub const DEFAULT_MEMORY_CHAR_LIMIT: usize = 2_200;
    pub const DEFAULT_USER_CHAR_LIMIT: usize = 1_375;
    pub const MEMORY_WARNING_THRESHOLD: f32 = 0.80;
    pub const MIN_ENTRY_LENGTH: usize = 10;
    pub const ENTRY_DELIMITER: &str = "§";
}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    pub entries: Vec<MemoryEntry>,
    pub char_limit: usize,
    pub used_chars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub entry_type: EntryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryType {
    Memory,   // Agent's notes
    User,     // User profile
}

impl MemoryStore {
    pub fn new(char_limit: usize) -> Self {
        Self {
            entries: Vec::new(),
            char_limit,
            used_chars: 0,
        }
    }
    
    pub fn usage_percent(&self) -> f32 {
        self.used_chars as f32 / self.char_limit as f32
    }
    
    pub fn is_warning(&self) -> bool {
        self.usage_percent() >= memory_constants::MEMORY_WARNING_THRESHOLD
    }
    
    pub fn can_add(&self, content: &str) -> bool {
        self.used_chars + content.len() <= self.char_limit
    }
    
    pub fn add_entry(&mut self, content: String, entry_type: EntryType) -> Result<String, MemoryError> {
        if content.len() < memory_constants::MIN_ENTRY_LENGTH {
            return Err(MemoryError::EntryTooShort);
        }
        
        // Check for duplicates
        if self.entries.iter().any(|e| e.content == content) {
            return Err(MemoryError::DuplicateEntry);
        }
        
        if !self.can_add(&content) {
            return Err(MemoryError::CapacityExceeded {
                current: self.used_chars,
                limit: self.char_limit,
                needed: content.len(),
            });
        }
        
        let id = ulid::Ulid::new().to_string();
        let now = chrono::Utc::now().timestamp();
        
        let entry = MemoryEntry {
            id: id.clone(),
            content: content.clone(),
            created_at: now,
            updated_at: now,
            entry_type,
        };
        
        self.used_chars += content.len();
        self.entries.push(entry);
        
        Ok(id)
    }
    
    pub fn remove_entry(&mut self, old_text: &str) -> Result<(), MemoryError> {
        let index = self.entries
            .iter()
            .position(|e| e.content.contains(old_text))
            .ok_or(MemoryError::EntryNotFound)?;
        
        let entry = self.entries.remove(index);
        self.used_chars -= entry.content.len();
        Ok(())
    }
    
    pub fn replace_entry(&mut self, old_text: &str, new_content: String) -> Result<(), MemoryError> {
        let index = self.entries
            .iter()
            .position(|e| e.content.contains(old_text))
            .ok_or(MemoryError::EntryNotFound)?;
        
        let old_entry = &self.entries[index];
        let char_diff = new_content.len() as i64 - old_entry.content.len() as i64;
        
        // Check if new content would exceed limit
        if self.used_chars as i64 + char_diff > self.char_limit as i64 {
            return Err(MemoryError::CapacityExceeded {
                current: self.used_chars,
                limit: self.char_limit,
                needed: new_content.len(),
            });
        }
        
        self.used_chars = (self.used_chars as i64 + char_diff) as usize;
        self.entries[index].content = new_content;
        self.entries[index].updated_at = chrono::Utc::now().timestamp();
        
        Ok(())
    }
    
    pub fn to_snapshot(&self) -> String {
        let entries: Vec<String> = self.entries
            .iter()
            .map(|e| e.content.clone())
            .collect();
        
        format!(
            "═══ MEMORY ({}% — {}/{} chars) ═══\n{}\n",
            (self.usage_percent() * 100.0) as usize,
            self.used_chars,
            self.char_limit,
            entries.join(memory_constants::ENTRY_DELIMITER)
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Entry too short (minimum {} chars)", memory_constants::MIN_ENTRY_LENGTH)]
    EntryTooShort,
    
    #[error("Duplicate entry")]
    DuplicateEntry,
    
    #[error("Capacity exceeded: {current}/{limit} chars, need {needed} more")]
    CapacityExceeded { current: usize, limit: usize, needed: usize },
    
    #[error("Entry not found")]
    EntryNotFound,
}
```

### Task 1.2: Add Memory Tools to Agent Loop

**File**: `core/router/src/agent_loop.rs` (modify)

Add these constants near the top:

```rust
// Memory thresholds for agent-managed memory
const MIN_TOOL_CALLS_FOR_MEMORY: u32 = 5;  // Agent creates memory after 5+ tool calls
const AUTO_MEMORY_TRIGGERS: &[&str] = &[
    "learned",
    "discovered",
    "figured out",
    "found that",
    "remember to",
];
```

Add memory tool definitions to the agent's tool list:

```rust
pub struct AgentConfig {
    // ... existing fields ...
    pub memory_store: Option<Arc<tokio::sync::Mutex<MemoryStore>>>,
    pub user_store: Option<Arc<tokio::sync::Mutex<MemoryStore>>>,
}
```

Add a method to check if agent should save memory:

```rust
impl AgentLoop {
    // In plan() or act() - call after successful task completion
    async fn should_save_memory(&self, tool_calls: u32, success: bool) -> bool {
        tool_calls >= MIN_TOOL_CALLS_FOR_MEMORY && success
    }
    
    // Check if goal contains memory-worthy content
    fn contains_memory_trigger(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        AUTO_MEMORY_TRIGGERS.iter().any(|t| text_lower.contains(t))
    }
}
```

### Task 1.3: Create Memory API Endpoints

**File**: `core/router/src/api/memory_stores.rs` (new)

```rust
use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use crate::api::AppState;

// Routes to add:
// POST /api/v1/memory/memory          - Add memory entry
// POST /api/v1/memory/user            - Add user profile entry  
// PUT /api/v1/memory/memory/:old_text - Replace memory entry
// PUT /api/v1/memory/user/:old_text   - Replace user entry
// DELETE /api/v1/memory/memory        - Remove memory entry
// DELETE /api/v1/memory/user          - Remove user entry
// GET /api/v1/memory/snapshot         - Get frozen snapshot for system prompt
// GET /api/v1/memory/stats            - Get memory stats
```

### Task 1.4: Create Memory UI Components

**File**: `ui/src/components/memory/Memory.tsx` (new)

```tsx
interface MemoryEntry {
  id: string;
  content: string;
  created_at: string;
  entry_type: 'memory' | 'user';
}

interface MemoryStats {
  memory_used: number;
  memory_limit: number;
  memory_percent: number;
  user_used: number;
  user_limit: number;
  user_percent: number;
  entry_count: number;
}

export function Memory() {
  // Shows:
  // - Memory usage bar (MEMORY.md capacity)
  // - User profile usage bar (USER.md capacity)
  // - List of entries grouped by type
  // - Add/Edit/Delete buttons
  // - Frozen snapshot preview
}
```

**File**: `ui/src/components/memory/MemoryEntry.tsx` (new)

```tsx
interface Props {
  entry: MemoryEntry;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
  canEdit: boolean;
}

export function MemoryEntry({ entry, onEdit, onDelete, canEdit }: Props) {
  // Compact display with:
  // - Content (truncated if long)
  // - Timestamp
  // - Edit/Delete actions
}
```

---

## Phase 2: Agent-Managed Skills

### Overview

Hermes agents automatically create skills after:
- Complex tasks (5+ tool calls)
- Discovering non-trivial workflows
- User corrections
- Successful error recovery

### Constants to Add

**File**: `core/router/src/unified_config.rs`

```rust
pub mod skill_constants {
    // Complexity thresholds
    pub const MIN_TOOL_CALLS_FOR_SKILL: u32 = 5;
    pub const MIN_SUCCESSFUL_STEPS: u32 = 3;
    
    // Skill metadata
    pub const SKILL_VERSION: &str = "1.0.0";
    pub const AUTO_SKILL_CATEGORY: &str = "auto-created";
    
    // Directory structure
    pub const AUTO_SKILLS_DIR: &str = "skills/auto-created";
    pub const SKILL_REFERENCES_DIR: &str = "references";
}
```

### Task 2.1: Create Skill Manager Module

**File**: `core/router/src/skill_manager.rs` (new)

```rust
// core/router/src/skill_manager.rs

mod skill_constants {
    pub const MIN_TOOL_CALLS_FOR_SKILL: u32 = 5;
    pub const AUTO_SKILL_CATEGORY: &str = "auto-created";
    pub const SKILL_VERSION: &str = "1.0.0";
}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub category: String,
    pub platforms: Vec<String>,  // ["macos", "linux", "windows"]
    pub created_at: i64,
    pub trigger_conditions: Vec<String>,
    pub auto_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCreateRequest {
    pub name: String,
    pub content: String,
    pub category: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPatchRequest {
    pub old_string: String,
    pub new_string: String,
}

pub struct SkillManager {
    skills_dir: PathBuf,
}

impl SkillManager {
    pub fn new(skills_dir: PathBuf) -> Self {
        Self { skills_dir }
    }
    
    pub async fn create_skill(&self, req: SkillCreateRequest) -> Result<SkillMetadata, SkillError> {
        // Create directory structure
        let skill_dir = self.skills_dir.join(&req.name);
        std::fs::create_dir_all(skill_dir.join("references"))?;
        
        // Generate SKILL.md
        let skill_md = self.generate_skill_md(&req)?;
        let skill_path = skill_dir.join("SKILL.md");
        std::fs::write(&skill_path, skill_md)?;
        
        // Return metadata
        Ok(SkillMetadata {
            name: req.name,
            description: req.description.unwrap_or_default(),
            version: skill_constants::SKILL_VERSION.to_string(),
            category: req.category.unwrap_or_else(|| skill_constants::AUTO_SKILL_CATEGORY.to_string()),
            platforms: vec!["linux".to_string(), "macos".to_string(), "windows".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            trigger_conditions: Vec::new(),
            auto_created: true,
        })
    }
    
    fn generate_skill_md(&self, req: &SkillCreateRequest) -> Result<String, SkillError> {
        Ok(format!(r#"---
name: {}
description: {}
version: {}
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [auto-created]
    category: auto-created
---

# {}

## When to Use
Automatically created from successful task execution.

## Procedure
{}

## Pitfalls
- None documented yet

## Verification
Task completed successfully.
"#, 
            req.name,
            req.description.as_deref().unwrap_or("Auto-created skill"),
            skill_constants::SKILL_VERSION,
            req.name,
            req.content
        ))
    }
    
    pub async fn patch_skill(&self, name: &str, patch: SkillPatchRequest) -> Result<(), SkillError> {
        let skill_path = self.skills_dir.join(name).join("SKILL.md");
        let content = std::fs::read_to_string(&skill_path)?;
        
        if !content.contains(&patch.old_string) {
            return Err(SkillError::ContentNotFound);
        }
        
        let new_content = content.replace(&patch.old_string, &patch.new_string);
        std::fs::write(&skill_path, new_content)?;
        
        Ok(())
    }
    
    pub async fn delete_skill(&self, name: &str) -> Result<(), SkillError> {
        let skill_dir = self.skills_dir.join(name);
        if skill_dir.exists() {
            std::fs::remove_dir_all(skill_dir)?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Skill already exists")]
    AlreadyExists,
    
    #[error("Skill not found")]
    NotFound,
    
    #[error("Content not found for patch")]
    ContentNotFound,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Task 2.2: Integrate with Agent Loop

**File**: `core/router/src/agent_loop.rs` (modify)

Add skill manager to AgentConfig:

```rust
pub struct AgentConfig {
    // ... existing fields ...
    pub skill_manager: Option<Arc<SkillManager>>,
    pub task_complexity: u32,  // Count of tool calls
}
```

Add skill creation trigger in the execution loop:

```rust
impl AgentLoop {
    async fn maybe_create_skill(&self, state: &AgentState) -> Option<String> {
        let config = &self.config;
        
        // Only create skills if we have a skill manager and met threshold
        let skill_manager = config.skill_manager.as_ref()?;
        if state.history.len() < skill_constants::MIN_TOOL_CALLS_FOR_SKILL as usize {
            return None;
        }
        
        // Generate skill name from goal
        let skill_name = self.generate_skill_name(&state.goal)?;
        
        // Check if similar skill already exists
        if self.skill_exists(&skill_name).await {
            return None;
        }
        
        // Create skill from successful execution
        let skill_content = self.generate_skill_content(state)?;
        
        let req = SkillCreateRequest {
            name: skill_name.clone(),
            content: skill_content,
            category: Some(skill_constants::AUTO_SKILL_CATEGORY.to_string()),
            description: Some(format!("Created from task: {}", state.goal.chars().take(50))),
        };
        
        match skill_manager.create_skill(req).await {
            Ok(_) => {
                tracing::info!(skill = %skill_name, "Agent created new skill");
                Some(skill_name)
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to create skill");
                None
            }
        }
    }
    
    fn generate_skill_name(&self, goal: &str) -> Option<String> {
        // Extract keywords and create skill name
        // e.g., "Deploy to Kubernetes" -> "deploy-k8s"
        let words: Vec<&str> = goal.split_whitespace().take(3).collect();
        if words.is_empty() {
            return None;
        }
        let slug = words.join("-").to_lowercase();
        Some(format!("auto-{}", slug))
    }
    
    fn generate_skill_content(&self, state: &AgentState) -> Option<String> {
        // Summarize the successful execution path
        let steps: Vec<String> = state.history.iter()
            .filter_map(|step| {
                match step {
                    AgentStep::ToolCall { tool, input, output, .. } => {
                        Some(format!("1. Use {} with input: {:?}\n   Result: {}", 
                            tool, input, output.as_deref().unwrap_or("success")))
                    }
                    _ => None
                }
            })
            .collect();
        
        if steps.is_empty() {
            return None;
        }
        
        Some(steps.join("\n\n"))
    }
}
```

### Task 2.3: Add Skill Management API

**File**: `core/router/src/api/skills_v2.rs` (new)

```rust
// Extended skill management API
// POST /api/v1/skills/agent          - Agent creates skill
// PUT /api/v1/skills/:name/patch     - Agent patches skill
// DELETE /api/v1/skills/:name         - Agent deletes skill
// GET /api/v1/skills/auto-created     - List auto-created skills
```

### Task 2.4: Create Skill Management UI

**File**: `ui/src/components/skills/SkillManager.tsx` (new)

```tsx
interface AutoCreatedSkill {
  name: string;
  description: string;
  category: string;
  created_at: string;
  trigger_count: number;
}

export function SkillManager() {
  // Shows:
  // - Auto-created skills section
  // - Skill edit interface
  // - Delete confirmation
  // - Skill usage analytics
}
```

---

## Phase 3: Skills Hub Integration

### Overview

Connect to external skill marketplaces:
1. skills.sh (Vercel)
2. well-known endpoints (Mintlify, etc.)
3. GitHub repos
4. Official optional skills

### Constants to Add

**File**: `core/router/src/unified_config.rs`

```rust
pub mod hub_constants {
    // Trust levels
    pub const TRUST_LEVEL_BUILTIN: &str = "builtin";
    pub const TRUST_LEVEL_OFFICIAL: &str = "official";
    pub const TRUST_LEVEL_TRUSTED: &str = "trusted";
    pub const TRUST_LEVEL_COMMUNITY: &str = "community";
    
    // Well-known endpoints
    pub const SKILLS_SH_URL: &str = "https://skills.sh";
    pub const AGENTSKILLS_URL: &str = "https://agentskills.io";
    
    // Cache settings
    pub const HUB_CACHE_TTL_SECS: u64 = 3600;  // 1 hour
    pub const HUB_LIST_TIMEOUT_MS: u64 = 5000;
}
```

### Task 3.1: Create Hub Client Module

**File**: `core/router/src/hub_client.rs` (new)

```rust
// core/router/src/hub_client.rs

mod hub_constants {
    pub const SKILLS_SH_URL: &str = "https://skills.sh";
    pub const HUB_CACHE_TTL_SECS: u64 = 3600;
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: HubSource,
    pub trust_level: String,
    pub install_count: u32,
    pub security_audit: Option<SecurityAudit>,
    pub metadata: SkillMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HubSource {
    SkillsSh,
    WellKnown { url: String },
    GitHub { owner: String, repo: String },
    Official,
    Community,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAudit {
    pub verdict: AuditVerdict,
    pub findings: Vec<SecurityFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditVerdict {
    Safe,
    Caution,
    Warn,
    Dangerous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub severity: String,
    pub description: String,
    pub location: Option<String>,
}

pub struct HubClient {
    http_client: reqwest::Client,
    cache: HashMap<String, CachedResponse>,
}

struct CachedResponse {
    data: Vec<HubSkill>,
    timestamp: i64,
}

impl HubClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
            cache: HashMap::new(),
        }
    }
    
    pub async fn search_skills_sh(&self, query: &str) -> Result<Vec<HubSkill>, HubError> {
        let url = format!("{}/api/search?q={}", hub_constants::SKILLS_SH_URL, query);
        let response = self.http_client.get(&url).send().await?;
        // Parse and return skills
        Ok(Vec::new())  // TODO: Implement actual parsing
    }
    
    pub async fn search_well_known(&self, url: &str) -> Result<Vec<HubSkill>, HubError> {
        let index_url = format!("{}/.well-known/skills/index.json", url);
        let response = self.http_client.get(&index_url).send().await?;
        // Parse well-known index
        Ok(Vec::new())
    }
    
    pub async fn install_from_github(&self, owner: &str, repo: &str, path: &str) -> Result<HubSkill, HubError> {
        let url = format!("https://raw.githubusercontent.com/{}/{}/main/{}", owner, repo, path);
        let response = self.http_client.get(&url).send().await?;
        let content = response.text().await?;
        
        // Validate skill content
        self.security_scan(&content)?;
        
        // Save to local skills directory
        // ...
        
        Ok(HubSkill {
            id: format!("{}/{}/{}", owner, repo, path),
            name: path.replace(".md", ""),
            description: String::new(),
            source: HubSource::GitHub { owner: owner.to_string(), repo: repo.to_string() },
            trust_level: hub_constants::TRUST_LEVEL_COMMUNITY.to_string(),
            install_count: 0,
            security_audit: None,
            metadata: SkillMetadata::default(),
        })
    }
    
    fn security_scan(&self, content: &str) -> Result<(), HubError> {
        // Check for dangerous patterns
        let dangerous_patterns = [
            "curl | sh",           // Pipe to shell
            "rm -rf /",            // Destructive
            "eval ",               // Code injection
            "base64 -d",           // Obfuscation
            "wget .* | sh",        // Download and execute
        ];
        
        for pattern in dangerous_patterns {
            if content.contains(pattern) {
                return Err(HubError::SecurityBlocked {
                    pattern: pattern.to_string(),
                });
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HubError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Security blocked: {pattern}")]
    SecurityBlocked { pattern: String },
    
    #[error("Parse error: {0}")]
    ParseError(String),
}
```

### Task 3.2: Add Hub API Endpoints

**File**: `core/router/src/api/hub.rs` (new)

```rust
// Routes:
// GET /api/v1/hub/skills              - Browse hub skills
// GET /api/v1/hub/skills/search        - Search skills
// POST /api/v1/hub/skills/install      - Install from hub
// GET /api/v1/hub/skills/check         - Check for updates
// POST /api/v1/hub/skills/update       - Update skills
// GET /api/v1/hub/skills/audit         - Security audit
```

### Task 3.3: Create Marketplace UI

**File**: `ui/src/components/skills/Marketplace.tsx` (new)

```tsx
interface HubSkill {
  id: string;
  name: string;
  description: string;
  source: string;
  trustLevel: 'builtin' | 'official' | 'trusted' | 'community';
  installCount: number;
  securityVerdict: 'safe' | 'caution' | 'warn' | 'dangerous';
}

interface MarketplaceFilters {
  source: string[];
  trustLevel: string[];
  category: string[];
  searchQuery: string;
}

export function Marketplace() {
  // Features:
  // - Browse skills by source (skills.sh, GitHub, etc.)
  // - Search with filters
  // - Trust level badges
  // - Security verdict indicators
  // - Install/Uninstall buttons
  // - Update check button
}
```

---

## Phase 4: Session Search Enhancement

### Overview

Add FTS5 full-text search with LLM summarization for past conversations.

### Constants to Add

**File**: `core/router/src/unified_config.rs`

```rust
pub mod search_constants {
    // FTS5 settings
    pub const FTS5_TOKENIZER: &str = "porter unicode61";
    pub const FTS5_RANK: &str = "bm25";
    
    // Search limits
    pub const MAX_SEARCH_RESULTS: usize = 20;
    pub const MAX_SUMMARY_LENGTH: usize = 500;
    
    // Relevance
    pub const FTS5_BM25_K1: f64 = 1.2;
    pub const FTS5_BM25_B: f64 = 0.75;
}
```

### Task 4.1: Create FTS5 Session Search

**File**: `core/router/src/session_search.rs` (new)

```rust
// core/router/src/session_search.rs

mod search_constants {
    pub const FTS5_TOKENIZER: &str = "porter unicode61";
    pub const MAX_SEARCH_RESULTS: usize = 20;
    pub const MAX_SUMMARY_LENGTH: usize = 500;
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSearchResult {
    pub session_id: String,
    pub relevance_score: f64,
    pub matched_content: String,
    pub summary: String,
    pub timestamp: i64,
}

pub struct SessionSearch {
    pool: sqlx::SqlitePool,
    llama_client: Option<LlamaClient>,
}

impl SessionSearch {
    pub fn new(pool: sqlx::SqlitePool, llama_client: Option<LlamaClient>) -> Self {
        Self { pool, llama_client }
    }
    
    pub async fn initialize_fts(&self) -> Result<(), SearchError> {
        sqlx::query(
            r#"
            CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(
                session_id,
                content,
                tokenize='porter unicode61'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn index_session(&self, session_id: &str, content: &str) -> Result<(), SearchError> {
        // Insert into FTS table
        sqlx::query(
            "INSERT INTO sessions_fts (session_id, content) VALUES (?, ?)"
        )
        .bind(session_id)
        .bind(content)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn search(&self, query: &str) -> Result<Vec<SessionSearchResult>, SearchError> {
        // Execute FTS5 search
        let rows = sqlx::query_as::<_, (String, String, f64)>(
            r#"
            SELECT session_id, content, bm25(sessions_fts, 20, 1.2, 0.75) as score
            FROM sessions_fts
            WHERE content MATCH ?
            ORDER BY score
            LIMIT ?
            "#
        )
        .bind(query)
        .bind(search_constants::MAX_SEARCH_RESULTS as i64)
        .fetch_all(&self.pool)
        .await?;
        
        let mut results = Vec::new();
        for (session_id, content, score) in rows {
            let summary = self.summarize(&content).await?;
            results.push(SessionSearchResult {
                session_id,
                relevance_score: score,
                matched_content: content,
                summary,
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
        
        Ok(results)
    }
    
    async fn summarize(&self, content: &str) -> Result<String, SearchError> {
        let client = match &self.llama_client {
            Some(c) => c,
            None => return Ok(content.chars().take(200).collect()),
        };
        
        let prompt = format!(
            "Summarize this conversation in {} characters or less:\n\n{}",
            search_constants::MAX_SUMMARY_LENGTH,
            content
        );
        
        match client.chat("You are a helpful assistant.", &prompt).await {
            Ok(summary) => Ok(summary),
            Err(_) => Ok(content.chars().take(200).collect()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),
    
    #[error("LLM error: {0}")]
    LlmError(String),
}
```

### Task 4.2: Add Search API

**File**: `core/router/src/api/sessions.rs` (modify)

```rust
// Add to existing sessions API:
// GET /api/v1/sessions/search?q=query    - FTS5 search
// GET /api/v1/sessions/:id/messages     - Get session messages
```

### Task 4.3: Create Search UI

**File**: `ui/src/components/SessionSearch.tsx` (new)

```tsx
interface SearchResult {
  sessionId: string;
  relevanceScore: number;
  summary: string;
  timestamp: string;
}

export function SessionSearch() {
  // Features:
  // - Search input with debounce
  // - Results with relevance scores
  // - Summary preview
  // - Click to expand full session
  // - Filter by date range
}
```

---

## Phase 5: Honcho-Style User Modeling

### Overview

Track user preferences and communication style across sessions.

### Constants to Add

**File**: `core/router/src/unified_config.rs`

```rust
pub mod user_constants {
    // User profile
    pub const USER_NAME_MAX_LENGTH: usize = 100;
    pub const USER_PREFERENCES_MAX_LENGTH: usize = 1000;
    
    // Communication styles
    pub const COMMUNICATION_STYLES: [&str; 4] = [
        "concise",      // Brief responses
        "detailed",     // Comprehensive responses
        "technical",    // Technical depth
        "casual",       // Conversational
    ];
    
    // Learning
    pub const PREFERENCE_CONFIDENCE_THRESHOLD: f32 = 0.8;
    pub const MIN_INTERACTIONS_FOR_PREFERENCE: u32 = 3;
}
```

### Task 5.1: Create User Profile Module

**File**: `core/router/src/user_profile.rs` (new)

```rust
// core/router/src/user_profile.rs

mod user_constants {
    pub const MIN_INTERACTIONS_FOR_PREFERENCE: u32 = 3;
    pub const PREFERENCE_CONFIDENCE_THRESHOLD: f32 = 0.8;
}

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: Option<String>,
    pub timezone: Option<String>,
    pub communication_style: CommunicationStyle,
    pub preferences: HashMap<String, Preference>,
    pub pet_peeves: Vec<String>,
    pub interaction_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStyle {
    pub style: String,  // concise, detailed, technical, casual
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preference {
    pub value: String,
    pub confidence: f32,
    pub interaction_count: u32,
    pub first_mentioned: i64,
    pub last_mentioned: i64,
}

impl UserProfile {
    pub fn new() -> Self {
        Self {
            name: None,
            timezone: None,
            communication_style: CommunicationStyle {
                style: "detailed".to_string(),  // Default
                confidence: 0.0,
            },
            preferences: HashMap::new(),
            pet_peeves: Vec::new(),
            interaction_count: 0,
        }
    }
    
    pub fn record_interaction(&mut self) {
        self.interaction_count += 1;
    }
    
    pub fn learn_preference(&mut self, key: &str, value: &str) {
        let entry = self.preferences.entry(key.to_string()).or_insert(Preference {
            value: value.to_string(),
            confidence: 0.0,
            interaction_count: 0,
            first_mentioned: chrono::Utc::now().timestamp(),
            last_mentioned: chrono::Utc::now().timestamp(),
        });
        
        if entry.value == value {
            // Same value, increase confidence
            entry.interaction_count += 1;
            entry.confidence = (entry.interaction_count as f32 / self.interaction_count as f32)
                .min(1.0);
        } else {
            // New value, reset
            entry.value = value.to_string();
            entry.interaction_count = 1;
            entry.confidence = 1.0 / self.interaction_count as f32;
        }
        
        entry.last_mentioned = chrono::Utc::now().timestamp();
    }
    
    pub fn infer_communication_style(&mut self, response_length: usize) {
        // Simple heuristic based on response length
        let style = if response_length < 100 {
            "concise"
        } else if response_length < 300 {
            "detailed"
        } else {
            "technical"
        };
        
        // Update confidence based on consistency
        if self.communication_style.style == style {
            self.communication_style.confidence = 
                (self.communication_style.confidence + 0.1).min(1.0);
        } else {
            self.communication_style.confidence = 
                (self.communication_style.confidence - 0.1).max(0.0);
        }
    }
    
    pub fn to_system_prompt(&self) -> String {
        let mut parts = Vec::new();
        
        if let Some(ref name) = self.name {
            parts.push(format!("User's name: {}", name));
        }
        
        parts.push(format!(
            "User prefers {} responses",
            self.communication_style.style
        ));
        
        // Add high-confidence preferences
        for (key, pref) in &self.preferences {
            if pref.confidence >= user_constants::PREFERENCE_CONFIDENCE_THRESHOLD {
                parts.push(format!("{}: {}", key, pref.value));
            }
        }
        
        if !self.pet_peeves.is_empty() {
            parts.push(format!("Avoid: {}", self.pet_peeves.join(", ")));
        }
        
        format!("═══ USER PROFILE ═══\n{}\n", parts.join("\n"))
    }
}
```

### Task 5.2: Integrate with Agent Loop

**File**: `core/router/src/agent_loop.rs` (modify)

```rust
pub struct AgentConfig {
    // ... existing fields ...
    pub user_profile: Option<Arc<tokio::sync::Mutex<UserProfile>>>,
}
```

In `act()` method, after generating response:

```rust
// Learn from interaction
if let Some(ref profile) = self.config.user_profile {
    let mut profile = profile.lock().await;
    profile.record_interaction();
    profile.infer_communication_style(response.len());
}
```

### Task 5.3: Create User Profile UI

**File**: `ui/src/components/settings/UserProfile.tsx` (new)

```tsx
interface UserProfile {
  name: string;
  timezone: string;
  communicationStyle: 'concise' | 'detailed' | 'technical' | 'casual';
  preferences: Record<string, { value: string; confidence: number }>;
  petPeeves: string[];
}

export function UserProfile() {
  // Features:
  // - Name and timezone settings
  // - Communication style selector
  // - Learned preferences display
  // - Pet peeves list
  // - Manual override options
  // - "Start fresh" option
}
```

---

## UI Components

### Summary of UI Changes

| Component | File | Features |
|-----------|------|----------|
| Memory Panel | `ui/src/components/memory/Memory.tsx` | Usage bars, entry list, CRUD |
| Memory Entry | `ui/src/components/memory/MemoryEntry.tsx` | Compact entry display |
| Skill Manager | `ui/src/components/skills/SkillManager.tsx` | Auto-created skills |
| Marketplace | `ui/src/components/skills/Marketplace.tsx` | Hub browsing, install |
| Session Search | `ui/src/components/SessionSearch.tsx` | FTS5 search UI |
| User Profile | `ui/src/components/settings/UserProfile.tsx` | Profile management |

### Navigation Integration

**File**: `ui/src/App.tsx` (modify)

Add new tabs to navigation:

```tsx
// Add to existing navigation structure:
// Memory → Memory (MEMORY.md + USER.md)
// Skills → Skills Registry (existing) + Marketplace (new)
// Search → Session Search (new)
```

### Shared Components Needed

```tsx
// ui/src/components/shared/UsageBar.tsx
interface UsageBarProps {
  used: number;
  limit: number;
  label: string;
  warningThreshold?: number;  // Default: 0.8
  criticalThreshold?: number;  // Default: 0.95
}

// ui/src/components/shared/TrustBadge.tsx
interface TrustBadgeProps {
  level: 'builtin' | 'official' | 'trusted' | 'community';
}

// ui/src/components/shared/SecurityVerdict.tsx
interface SecurityVerdictProps {
  verdict: 'safe' | 'caution' | 'warn' | 'dangerous';
}
```

---

## Wiring Diagrams

### Memory System Wiring

```
┌─────────────────────────────────────────────────────────────────┐
│                        Memory System                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ MemoryStore  │    │  UserStore   │    │   Snapshot   │      │
│  │  (2,200ch)   │    │  (1,375ch)   │    │   Generator  │      │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘      │
│         │                   │                   │               │
│         └───────────┬───────┘                   │               │
│                     │                           │               │
│                     ▼                           ▼               │
│         ┌───────────────────────┐    ┌─────────────────┐       │
│         │    Agent Loop        │───▶│  System Prompt  │       │
│         │  (reads/writes)      │    │  (frozen snap)  │       │
│         └───────────────────────┘    └─────────────────┘       │
│                     │                                           │
│                     ▼                                           │
│         ┌───────────────────────┐                              │
│         │    Memory API         │                              │
│         │ POST/GET/PUT/DELETE  │                              │
│         └───────────┬───────────┘                              │
│                     │                                           │
│                     ▼                                           │
│         ┌───────────────────────┐                              │
│         │    React UI           │                              │
│         │  Memory.tsx           │                              │
│         └───────────────────────┘                              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Skills Hub Wiring

```
┌─────────────────────────────────────────────────────────────────┐
│                       Skills Hub System                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  SkillsSh    │    │ WellKnown    │    │   GitHub    │       │
│  │  Client      │    │  Client      │    │   Client    │       │
│  └──────┬───────┘    └──────┬───────┘    └──────┬─────┘       │
│         │                   │                   │                │
│         └───────────┬───────┴──────────────────┘                │
│                     │                                            │
│                     ▼                                            │
│         ┌───────────────────────┐                               │
│         │      Hub Client       │                               │
│         │  (unified interface)  │                               │
│         └───────────┬───────────┘                               │
│                     │                                            │
│         ┌───────────┴───────────┐                               │
│         │                       │                               │
│         ▼                       ▼                               │
│  ┌──────────────┐    ┌───────────────────────┐                 │
│  │ Security     │    │    Install Manager    │                 │
│  │ Scanner      │    │  (save to skills/)    │                 │
│  └──────────────┘    └───────────┬───────────┘                 │
│                                  │                              │
│                                  ▼                              │
│                       ┌───────────────────────┐                │
│                       │    Skills Loader      │                │
│                       │  (existing loader)    │                │
│                       └───────────────────────┘                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### User Modeling Wiring

```
┌─────────────────────────────────────────────────────────────────┐
│                      User Modeling System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ UserProfile  │◀───│  Inference   │◀───│ Interaction │       │
│  │   Store     │    │   Engine     │    │   Tracker   │       │
│  └──────┬───────┘    └──────────────┘    └──────────────┘       │
│         │                                                         │
│         │                                                         │
│         ▼                                                         │
│  ┌───────────────────────┐                                      │
│  │   System Prompt       │                                      │
│  │   (with profile)      │                                      │
│  └───────────────────────┘                                      │
│                                                                  │
│  ┌───────────────────────┐    ┌──────────────┐                  │
│  │   Profile API          │───▶│  Settings    │                  │
│  │   (CRUD operations)    │    │    UI        │                  │
│  └───────────────────────┘    └──────────────┘                  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Constants Reference

All magic numbers are defined here. No magic numbers elsewhere.

### File: `core/router/src/unified_config.rs`

```rust
pub mod config_constants {
    // Server
    pub const DEFAULT_PORT: u16 = 3000;
    pub const DEFAULT_HOST: &str = "0.0.0.0";

    // Agent
    pub const DEFAULT_MAX_ITERATIONS: u32 = 50;
    pub const DEFAULT_MAX_BUDGET_CENTS: i64 = 500;
    pub const DEFAULT_CONTEXT_WINDOW_TOKENS: usize = 32768;

    // Memory (NEW)
    pub mod memory {
        pub const DEFAULT_MEMORY_CHAR_LIMIT: usize = 2_200;
        pub const DEFAULT_USER_CHAR_LIMIT: usize = 1_375;
        pub const WARNING_THRESHOLD: f32 = 0.80;
        pub const CRITICAL_THRESHOLD: f32 = 0.95;
        pub const MIN_ENTRY_LENGTH: usize = 10;
        pub const MAX_ENTRY_LENGTH: usize = 500;
        pub const ENTRY_DELIMITER: &str = "§";
    }

    // Skills (NEW)
    pub mod skills {
        pub const MIN_TOOL_CALLS_FOR_SKILL: u32 = 5;
        pub const AUTO_SKILL_CATEGORY: &str = "auto-created";
        pub const SKILL_VERSION: &str = "1.0.0";
    }

    // Hub (NEW)
    pub mod hub {
        pub const TRUST_LEVEL_BUILTIN: &str = "builtin";
        pub const TRUST_LEVEL_OFFICIAL: &str = "official";
        pub const TRUST_LEVEL_TRUSTED: &str = "trusted";
        pub const TRUST_LEVEL_COMMUNITY: &str = "community";
        pub const SKILLS_SH_URL: &str = "https://skills.sh";
        pub const CACHE_TTL_SECS: u64 = 3600;
    }

    // Search (NEW)
    pub mod search {
        pub const FTS5_TOKENIZER: &str = "porter unicode61";
        pub const MAX_SEARCH_RESULTS: usize = 20;
        pub const MAX_SUMMARY_LENGTH: usize = 500;
        pub const BM25_K1: f64 = 1.2;
        pub const BM25_B: f64 = 0.75;
    }

    // User Modeling (NEW)
    pub mod user {
        pub const NAME_MAX_LENGTH: usize = 100;
        pub const PREFERENCES_MAX_LENGTH: usize = 1000;
        pub const COMMUNICATION_STYLES: [&str; 4] = ["concise", "detailed", "technical", "casual"];
        pub const PREFERENCE_CONFIDENCE_THRESHOLD: f32 = 0.8;
        pub const MIN_INTERACTIONS_FOR_PREFERENCE: u32 = 3;
    }

    // Execution (existing)
    pub const DEFAULT_VCPUS: u32 = 2;
    pub const DEFAULT_MEMORY_MIB: u64 = 2048;
    pub const DEFAULT_TIMEOUT_SECS: u64 = 60;

    // Sandbox (existing)
    pub const DEFAULT_SANDBOX_MEMORY_MB: u64 = 512;
    pub const DEFAULT_SANDBOX_TIMEOUT_SECS: u64 = 30;

    // Skill Pool (existing)
    pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30_000;
    pub const DEFAULT_ACQUIRE_TIMEOUT_MS: u64 = 5_000;
    pub const DEFAULT_POOL_SIZE: u32 = 4;

    // URLs (existing)
    pub const DEFAULT_LLAMA_URL: &str = "http://localhost:8080";
    pub const DEFAULT_EMBED_URL: &str = "http://localhost:8081";
    pub const DEFAULT_NATS_URL: &str = "127.0.0.1:4222";

    // Auth (existing)
    pub const DEFAULT_DEV_SECRET: &str = "dev-secret-change-in-production";
}
```

---

## Implementation Order

### Phase 0: Foundation (No dependencies)
1. Add all constants to `config_constants` module
2. Create `memory_constants` submodule
3. Create `skill_constants` submodule
4. Create `hub_constants` submodule
5. Create `search_constants` submodule
6. Create `user_constants` submodule

### Phase 1: Bounded Memory (Foundation complete)
1. Create `memory_stores.rs` module
2. Add memory API endpoints
3. Create React Memory component
4. Integrate with agent loop
5. Add usage bars to UI

### Phase 2: Agent-Managed Skills (Phase 1 complete)
1. Create `skill_manager.rs` module
2. Add skill management API
3. Create React SkillManager component
4. Integrate skill creation triggers in agent loop
5. Add skill management to navigation

### Phase 3: Skills Hub (Phase 2 complete)
1. Create `hub_client.rs` module
2. Implement security scanner
3. Add hub API endpoints
4. Create React Marketplace component
5. Add marketplace to navigation

### Phase 4: Session Search (Phase 0 complete)
1. Create `session_search.rs` module
2. Add FTS5 tables to migrations
3. Add search API endpoints
4. Create React SessionSearch component
5. Add search to navigation

### Phase 5: User Modeling (Phase 4 complete)
1. Create `user_profile.rs` module
2. Add user profile API
3. Create React UserProfile component
4. Integrate with agent loop
5. Add to Settings tab

---

## Testing Strategy

### Unit Tests
- Each module has `mod tests` with test cases
- Test memory store operations (add, remove, replace, limits)
- Test skill manager (create, patch, delete)
- Test security scanner patterns

### Integration Tests
- Test memory API end-to-end
- Test skill creation flow in agent loop
- Test hub installation flow

### UI Tests
- Test memory usage bar thresholds
- Test skill installation/uninstall
- Test search results display

---

## Migration Notes

### For existing memory data
- Convert existing narrative memory entries to new bounded format
- Respect new character limits (may require consolidation)
- Preserve creation timestamps

### For existing skills
- Existing skills remain unchanged
- New auto-created skills use separate directory
- SKILL.md format unchanged (compatible)

---

## Estimated Effort

| Phase | Complexity | Estimated Time |
|-------|------------|----------------|
| Phase 0: Constants | Low | 2 hours |
| Phase 1: Memory | Medium | 1-2 days |
| Phase 2: Agent Skills | High | 2-3 days |
| Phase 3: Hub | High | 2-3 days |
| Phase 4: Search | Medium | 1-2 days |
| Phase 5: User Profile | Medium | 1-2 days |
| **Total** | - | **9-14 days** |
