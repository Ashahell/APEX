# Sapphire Features Implementation Plan

## Overview

This plan adapts 7 high-value features from Sapphire (ddxfish/sapphire) for APEX:
1. Tool Maker runtime validation with import allowlist levels
2. Persona assembly (prompt + voice + tools + model bundles)
3. Context scope isolation (per-conversation data separation)
4. Continuity scheduler (enhanced heartbeat/cron tasks)
5. Plugin signing (ed25519 verification)
6. Privacy toggle (one-click cloud block)
7. Story engine (interactive fiction as tasks)

**Version**: 1.6.0 (Proposal)
**Architecture**: Keep Rust core, TypeScript gateway, React UI, Python execution
**License**: AGPL-3.0 compatible (note: Sapphire is AGPL, we remain MIT)

---

## Feature 1: Tool Maker Runtime Validation

### Summary
Enhance APEX's dynamic tool generation with import allowlist levels (Strict/Moderate/System Killer) similar to Sapphire's Tool Maker validation.

### Architecture
```
core/router/src/
├── tool_validator.rs        # NEW - Validation levels
├── tool_sandbox.rs         # NEW - Sandboxed execution
└── dynamic_tools.rs        # MODIFY - Add validation
```

### Constants (in `unified_config.rs`)
```rust
// Tool validation constants
pub mod tool_validation_constants {
    pub const STRICT_IMPORT_ALLOWLIST: &[&str] = &[
        "json", "re", "math", "datetime", "collections", "itertools",
        "random", "uuid", "hashlib", "typing", "os.path", " pathlib",
    ];
    pub const MODERATE_IMPORT_ALLOWLIST: &[&str] = &[
        // Strict +
        "urllib", "http.client", "csv", "xml.etree", "base64",
        "textwrap", "html", "ast", "inspect", "functools",
    ];
    pub const SYSTEM_KILLER_IMPORTS: &[&str] = &[
        "os.system", "subprocess", "pty", "socket", "ctypes",
        "importlib", "__import__", "eval", "exec", "compile",
    ];
    pub const DEFAULT_VALIDATION_LEVEL: ValidationLevel = ValidationLevel::Strict;
}
```

### Implementation Steps

#### Step 1: Create validation level enum
- File: `core/router/src/tool_validator.rs`
- Define `ValidationLevel` enum: `Strict`, `Moderate`, `Permissive`
- Define `ValidationResult` struct with `allowed: bool`, `blocked_imports: Vec<String>`

#### Step 2: Implement import allowlist checker
- Parse Python code AST to extract imports
- Match against allowlist based on validation level
- Return blocked imports if any

#### Step 3: Create sandbox executor
- File: `core/router/src/tool_sandbox.rs`
- Execute validated Python in isolated environment
- Set timeout (from config)
- Capture stdout/stderr
- Kill on timeout

#### Step 4: Wire into dynamic tools
- Modify `core/router/src/api/dynamic_tools.rs` to use validator
- Add `validation_level` field to tool generation request
- Return validation errors before execution

#### Step 5: Add UI component
- File: `ui/src/components/settings/ToolValidationSettings.tsx` (NEW)
- Dropdown for validation level: Strict / Moderate / Permissive
- Show blocked imports preview
- Save to preferences

### API Endpoints
```
PUT /api/v1/config/tool-validation
GET /api/v1/dynamic-tools/:name/validate
```

---

## Feature 2: Persona Assembly

### Summary
Bundle system prompt + voice settings + toolset + model config into swappable persona profiles. Enhance SOUL.md with structured assembly.

### Architecture
```
core/router/src/
├── persona.rs              # NEW - Persona struct and manager
├── persona_api.rs          # NEW - REST endpoints
└── agent_loop.rs          # MODIFY - Use active persona
ui/src/components/
├── settings/
│   └── PersonaEditor.tsx   # NEW - UI for persona management
```

### Constants
```rust
// Persona constants
pub mod persona_constants {
    pub const MAX_PERSONAS: usize = 20;
    pub const DEFAULT_PERSONA_NAME: &str = "default";
    pub const PERSONA_FILE: &str = "personas.json";
    
    // Persona components
    pub const MAX_PROMPT_PIECES: usize = 10;
    pub const MAX_TOOLS_PER_PERSONA: usize = 50;
}
```

### Implementation Steps

#### Step 1: Define Persona struct
- File: `core/router/src/persona.rs`
- `Persona` struct with: id, name, description, prompt_pieces[], tools[], voice_config, model_config
- `PromptPiece` enum: `System`, `Location`, `Emotion`, `Context`, `Custom`
- `VoiceConfig` struct: tts_engine, voice_id, speed, pitch
- `ModelConfig` struct: provider, model, temperature, max_tokens

#### Step 2: Create PersonaManager
- Load/save personas from JSON file
- Add/update/delete personas
- Validate persona (no circular dependencies, valid tools)

#### Step 3: Implement prompt assembly
- `assemble_system_prompt(persona_id)` - Combine pieces with delimiters
- Piece swapping at runtime (like Sapphire's self-modification)

#### Step 4: Add API endpoints
- `GET /api/v1/personas` - List all
- `POST /api/v1/personas` - Create
- `GET /api/v1/personas/:id` - Get details
- `PUT /api/v1/personas/:id` - Update
- `DELETE /api/v1/personas/:id` - Delete
- `POST /api/v1/personas/:id/activate` - Set as active
- `GET /api/v1/personas/active` - Get active persona

#### Step 5: Create UI components
- `PersonaList.tsx` - Sidebar list of personas
- `PersonaEditor.tsx` - Form with tabs: Prompt, Tools, Voice, Model
- `PersonaPieceEditor.tsx` - Reorderable prompt pieces

### Database Schema (Migration 025)
```sql
CREATE TABLE personas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    prompt_pieces TEXT NOT NULL,  -- JSON array
    tools TEXT NOT NULL,          -- JSON array of tool names
    voice_config TEXT,            -- JSON
    model_config TEXT,            -- JSON
    is_active INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

## Feature 3: Context Scope Isolation

### Summary
Implement per-conversation data isolation using Rust's async context. Similar to Sapphire's ContextVars scope isolation.

### Architecture
```
core/router/src/
├── context_scope.rs       # NEW - Scope isolation
├── session_manager.rs      # MODIFY - Add scope to sessions
└── bounded_memory.rs      # MODIFY - Respect scope
```

### Constants
```rust
// Context scope constants
pub mod scope_constants {
    pub const SCOPE_GLOBAL: &str = "global";
    pub const SCOPE_SESSION: &str = "session";
    pub const SCOPE_CHANNEL: &str = "channel";
    
    // Scope types per data type
    pub const MEMORY_SCOPE_DEFAULT: &str = "global";
    pub const BOUNDED_MEMORY_SCOPE_DEFAULT: &str = "global";
    pub const KNOWLEDGE_SCOPE_DEFAULT: &str = "global";
    pub const PREFERENCES_SCOPE_DEFAULT: &str = "session";
    pub const SKILLS_SCOPE_DEFAULT: &str = "global";
}
```

### Implementation Steps

#### Step 1: Define Scope enum
- `Scope` enum: `Global`, `Session(session_id)`, `Channel(channel_id)`
- `ScopeContext` struct - holds current scope for async task

#### Step 2: Create scope middleware
- Extract session_id/channel_id from request
- Set scope in task-local storage (use `tokio::task::LocalKey`)

#### Step 3: Modify memory operations
- `MemoryStore::get_entries(scope)` - Filter by scope
- `MemoryStore::add_entry(content, scope)` - Store with scope

#### Step 4: Add scope to API filters
- All memory endpoints accept optional `scope` parameter
- Default to session scope for new entries

### API Changes
```
GET /api/v1/memory/bounded/memory?scope=global|session|channel
POST /api/v1/memory/bounded/memory {"content": "...", "scope": "session"}
```

---

## Feature 4: Continuity Scheduler (Enhanced Heartbeat)

### Summary
Sophisticated cron-based autonomous task scheduling. Expand current heartbeat daemon with scheduled tasks, morning greetings, check-ins.

### Architecture
```
core/router/src/
├── continuity.rs           # NEW - Scheduler service
├── continuity_api.rs       # NEW - REST endpoints
└── heartbeat.rs           # MODIFY - Integrate continuity
ui/src/components/
├── autonomy/
│   └── ContinuitySettings.tsx  # NEW - Schedule management
```

### Constants
```rust
// Continuity constants
pub mod continuity_constants {
    pub const MAX_SCHEDULED_TASKS: usize = 50;
    pub const TASK_TYPES: &[&str] = &[
        "morning_greeting", "evening_checkin", "weekly_summary",
        "dream_mode", "alarm", "random_checkin", "custom"
    ];
    pub const DEFAULT_MORNING_HOUR: u32 = 8;
    pub const DEFAULT_EVENING_HOUR: u32 = 21;
    pub const MAX_TASK_HISTORY: usize = 100;
}
```

### Implementation Steps

#### Step 1: Define ScheduledTask struct
```rust
pub struct ScheduledTask {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub schedule: CronSchedule,
    pub action: TaskAction,      // What to do
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub run_count: u32,
}
```

#### Step 2: Create Cron parser
- Parse cron expressions (6-field: second minute hour day month weekday)
- Support shortcuts: `@daily`, `@hourly`, `@weekly`, `@morning`, `@evening`

#### Step 3: Implement scheduler service
- Background task checking every minute
- Calculate next run times
- Execute due tasks
- Log results

#### Step 4: Add API endpoints
- `GET /api/v1/continuity/tasks` - List scheduled
- `POST /api/v1/continuity/tasks` - Create
- `PUT /api/v1/continuity/tasks/:id` - Update
- `DELETE /api/v1/continuity/tasks/:id` - Delete
- `POST /api/v1/continuity/tasks/:id/run` - Manual trigger
- `GET /api/v1/continuity/history` - Run history

#### Step 5: Create UI
- `TaskScheduler.tsx` - List with enable/disable toggles
- `TaskEditor.tsx` - Form: name, type, schedule, action
- `TaskHistory.tsx` - Log of past runs

### Database Schema (Migration 026)
```sql
CREATE TABLE continuity_tasks (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    task_type TEXT NOT NULL,
    schedule TEXT NOT NULL,      -- Cron expression
    action TEXT NOT NULL,        -- JSON: what to do
    enabled INTEGER DEFAULT 1,
    last_run TEXT,
    next_run TEXT,
    run_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE continuity_history (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL,        -- running, success, failed
    result TEXT,
    error TEXT
);
```

---

## Feature 5: Plugin Signing (ed25519)

### Summary
Add cryptographic signing for skills/plugins to verify authenticity. Prevents tampered or malicious skills.

### Architecture
```
core/router/src/
├── skill_signer.rs         # NEW - Sign/verify skills
├── skill_manager.rs        # MODIFY - Add signature verification
└── security/
    └── signatures.rs      # NEW - Signature utilities
```

### Constants
```rust
// Signing constants
pub mod signing_constants {
    pub const SIGNATURE_ALGORITHM: &str = "ed25519";
    pub const SIGNATURE_EXPIRY_DAYS: i64 = 365;
    pub const KEY_BITS: usize = 256;
    pub const MAX_SIGNATURE_SIZE: usize = 64;
    
    // Keys location
    pub const SIGNING_KEY_PATH: &str = "keys/signing_key.pem";
    pub const VERIFY_KEY_PATH: &str = "keys/verify_key.pub";
}
```

### Implementation Steps

#### Step 1: Key management
- Generate ed25519 keypair on first run
- Store private key encrypted (use existing secret store)
- Public key available via API

#### Step 2: Sign skill manifest
- When skill is created/updated, sign the SKILL.md hash
- Store signature in skill metadata

#### Step 3: Verify on load
- Before loading skill, verify signature
- Fail load if signature invalid (configurable: block or warn)

#### Step 4: Add API endpoints
- `GET /api/v1/keys/verify-key` - Get public verification key
- `POST /api/v1/skills/:name/sign` - Re-sign skill
- `GET /api/v1/skills/:name/verify` - Check signature status

#### Step 5: Add UI
- `SkillSecurity.tsx` - Show signature status per skill
- "Verify All" button to check all skills
- "Sign New Skill" after creation

### Database Schema (Migration 027)
```sql
ALTER TABLE skill_registry ADD COLUMN signature TEXT;
ALTER TABLE skill_registry ADD COLUMN signed_at TEXT;
ALTER TABLE skill_registry ADD COLUMN signature_valid INTEGER DEFAULT 1;
```

---

## Feature 6: Privacy Toggle

### Summary
One-click toggle to block all cloud connections. Makes privacy explicit and visible. Enhanced version of existing local-first approach.

### Architecture
```
core/router/src/
├── privacy_guard.rs       # NEW - Block cloud connections
├── unified_config.rs      # MODIFY - Add privacy settings
ui/src/components/
└── settings/
    └── PrivacySettings.tsx  # NEW - Privacy toggle UI
```

### Constants
```rust
// Privacy constants
pub mod privacy_constants {
    pub const PRIVACY_MODE_KEY: &str = "privacy_mode_enabled";
    pub const CLOUD_PROVIDERS: &[&str] = &[
        "openai", "anthropic", "google", "cohere", "fireworks",
        "azure", "aws_bedrock", "huggingface"
    ];
    
    pub const PRIVACY_WARNING_THRESHOLD: usize = 80;  // percent
}
```

### Implementation Steps

#### Step 1: Define PrivacyConfig
```rust
pub struct PrivacyConfig {
    pub enabled: bool,
    pub blocked_providers: Vec<String>,
    pub allow_local_only: bool,
    pub audit_log_enabled: bool,
}
```

#### Step 2: Create PrivacyGuard
- Intercept LLM requests
- If privacy enabled + provider is cloud → block
- Log blocked attempt

#### Step 3: Integrate with LLM client
- In `llama.rs` and any cloud LLM client, check PrivacyGuard before request

#### Step 4: Add API
- `GET /api/v1/privacy/status` - Get current status
- `PUT /api/v1/privacy/config` - Update config
- `GET /api/v1/privacy/audit` - Blocked requests log

#### Step 5: Create UI
- `PrivacySettings.tsx` - Large toggle "Enable Privacy Mode"
- Warning message about local-only operation
- "Allowed Providers" list (local only by default)
- "Blocked Requests" log viewer

---

## Feature 7: Story Engine

### Summary
Interactive fiction framework where AI acts as dungeon master. Tasks become narrative adventures with dice, branching, state.

### Architecture
```
core/router/src/
├── story_engine.rs        # NEW - Story state machine
├── story_api.rs           # NEW - REST endpoints
└── tasks.rs              # MODIFY - Add story task type
ui/src/components/
├── stories/
│   ├── StoryPlayer.tsx    # NEW - Main story UI
│   └── StoryEditor.tsx     # NEW - Create stories
```

### Constants
```rust
// Story constants
pub mod story_constants {
    pub const MAX_STORIES: usize = 100;
    pub const MAX_STORY_LENGTH: usize = 10000;  // turns
    pub const DICE_TYPES: &[&str] = &["d4", "d6", "d8", "d10", "d12", "d20", "d100"];
    pub const DEFAULT_DICE: &str = "d20";
    
    pub const STORY_TASK_TYPE: &str = "story";
}
```

### Implementation Steps

#### Step 1: Define Story structs
```rust
pub struct Story {
    pub id: String,
    pub title: String,
    pub setting: StorySetting,     // fantasy, scifi, horror, etc.
    pub characters: Vec<Character>,
    pub state: StoryState,
    pub turn_count: u32,
}

pub struct StoryState {
    pub location: String,
    pub inventory: Vec<String>,
    pub npcs: Vec<NpcState>,
    pub flags: HashMap<String, bool>,
    pub history: Vec<StoryBeat>,
}

pub struct DiceRoll {
    pub dice: String,  // "2d6+3"
    pub result: u32,
    pub description: String,
}
```

#### Step 2: Implement story engine
- `start_story(setting, characters)` - Initialize
- `make_choice(choice_id)` - Process player choice
- `roll_dice(dice)` - Generate random result
- `get_available_choices()` - Return current options

#### Step 3: Add Task type
- New `TaskType::Story` in tasks
- Story tasks have: setting, characters, initial_state

#### Step 4: Add API
- `POST /api/v1/stories` - Create story
- `GET /api/v1/stories/:id` - Get story state
- `POST /api/v1/stories/:id/choice` - Make choice
- `POST /api/v1/stories/:id/roll` - Roll dice
- `GET /api/v1/stories/:id/choices` - Get available choices
- `DELETE /api/v1/stories/:id` - End story

#### Step 5: Create UI
- `StoryPlayer.tsx` - Main game view: narrative text, choices, dice
- `StoryEditor.tsx` - Create: setting, characters, initial prompt
- `StoryHistory.tsx` - Past stories

### Database Schema (Migration 028)
```sql
CREATE TABLE stories (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    setting TEXT NOT NULL,
    characters TEXT NOT NULL,     -- JSON
    state TEXT NOT NULL,          -- JSON (StoryState)
    turn_count INTEGER DEFAULT 0,
    task_id TEXT,                 -- Link to task
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

---

## Implementation Order

### Phase 1: Foundation (Weeks 1-2)
1. **Tool Maker Validation** - Security critical, builds on existing dynamic tools
2. **Privacy Toggle** - Quick win, visible feature

### Phase 2: Core Features (Weeks 3-4)
3. **Persona Assembly** - Major UX enhancement
4. **Context Scope Isolation** - Infrastructure for multi-session

### Phase 3: Advanced (Weeks 5-6)
5. **Continuity Scheduler** - Autonomous capabilities
6. **Plugin Signing** - Security hardening

### Phase 4: Fun (Week 7)
7. **Story Engine** - Creative feature, lighter priority

---

## Testing Strategy

| Feature | Unit Tests | Integration Tests | E2E Tests |
|---------|------------|-------------------|-----------|
| Tool Validation | 10 | 5 | 2 |
| Persona | 8 | 4 | 1 |
| Scope Isolation | 6 | 3 | 1 |
| Continuity | 8 | 5 | 2 |
| Plugin Signing | 5 | 3 | 1 |
| Privacy | 4 | 3 | 1 |
| Story Engine | 7 | 4 | 1 |
| **Total** | **48** | **27** | **9** |

---

## UI Component Summary

| Component | Location | Purpose |
|-----------|----------|---------|
| `ToolValidationSettings.tsx` | settings/ | Validation level dropdown |
| `PersonaList.tsx` | settings/ | Sidebar persona list |
| `PersonaEditor.tsx` | settings/ | Persona form with tabs |
| `ScopeIndicator.tsx` | chat/ | Show current scope |
| `TaskScheduler.tsx` | autonomy/ | Schedule list |
| `TaskEditor.tsx` | autonomy/ | Create/Edit task |
| `SkillSecurity.tsx` | skills/ | Signature status |
| `PrivacySettings.tsx` | settings/ | Privacy toggle |
| `StoryPlayer.tsx` | stories/ | Main game view |
| `StoryEditor.tsx` | stories/ | Create new story |

---

## API Summary

| Feature | New Endpoints | Modified Endpoints |
|---------|---------------|---------------------|
| Tool Validation | 2 | 2 (dynamic-tools) |
| Persona | 7 | 0 |
| Scope Isolation | 2 | 4 (memory endpoints) |
| Continuity | 6 | 1 (heartbeat) |
| Plugin Signing | 3 | 1 (skills) |
| Privacy | 3 | 1 (llm config) |
| Story Engine | 6 | 1 (tasks) |
| **Total** | **29** | **10** |

---

## Migration Files

| Number | Content |
|--------|---------|
| 025 | Personas table |
| 026 | Continuity tasks + history |
| 027 | Skill signature columns |
| 028 | Stories table |

---

## Constants Module Structure

All new constants go into `core/router/src/unified_config.rs`:

```rust
pub mod tool_validation_constants { ... }
pub mod persona_constants { ... }
pub mod scope_constants { ... }
pub mod continuity_constants { ... }
pub mod signing_constants { ... }
pub mod privacy_constants { ... }
pub mod story_constants { ... }
```

---

## Wire-Up Checklist

- [ ] Add new modules to `lib.rs`
- [ ] Initialize new services in `main.rs`
- [ ] Add API routes in `api/mod.rs`
- [ ] Add WebSocket events if needed
- [ ] Add to AppState struct
- [ ] Add navigation items to Sidebar
- [ ] Add feature flags in config
- [ ] Add to AGENTS.md documentation
- [ ] Update version to 1.6.0

---

## Notes

1. **AGPL Consideration**: Sapphire is AGPL-3.0. We're implementing similar concepts but in Rust with MIT license. No code copying - concepts only.

2. **Keep Modules Self-Contained**: Each feature should be a separate module with clear boundaries. No God Code.

3. **Use Constants**: All magic numbers replaced with named constants in `unified_config.rs`.

4. **UI First**: Each feature includes UI components - no backend-only features.

5. **Incremental**: Features can be enabled/disabled independently via feature flags.