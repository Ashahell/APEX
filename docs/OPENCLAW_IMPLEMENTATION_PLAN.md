# APEX OpenClaw Feature Implementation Plan

> **Document Status**: Implementation Roadmap  
> **Version**: 1.0  
> **Date**: 2026-03-13  
> **Based on**: OpenClaw v2026.3.11-v2026.3.12 Feature Analysis

---

## Executive Summary

This document details the implementation plan to add missing OpenClaw features to APEX. The plan prioritizes high-impact features that align with APEX's security-first architecture and existing patterns.

### Prioritization Framework

| Priority | Criteria | Features |
|----------|----------|----------|
| **P0 (Critical)** | Core functionality gaps, high user demand | Control UI Dashboard, Fast Mode, Provider Plugins |
| **P1 (High)** | Significant enhancement, moderate effort | PDF Tool, Multimodal Memory, sessions_yield |
| **P2 (Medium)** | Nice-to-have, well-scoped | Additional Channels, Secrets Expansion |
| **P3 (Low)** | Future consideration | Kubernetes, Complex Canvas |

---

## Phase 1: Control UI Dashboard (P0)

### Overview

Implement a modern modular dashboard with:
- Chat view with slash commands, search, export, pinned messages
- Session management with model picker
- Command palette (Ctrl+K)
- Mobile-optimized bottom tabs

### Database Changes

```sql
-- Migration: 015_control_ui
-- Dashboard preferences and pinned messages

-- 1. dashboard_layout: User dashboard layout preferences
CREATE TABLE IF NOT EXISTS dashboard_layout (
    id              TEXT PRIMARY KEY,
    user_id         TEXT NOT NULL DEFAULT 'default',
    layout_config   TEXT NOT NULL,  -- JSON: { sidebar: {}, panels: [], order: [] }
    theme           TEXT DEFAULT 'agentzero',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. pinned_messages: Pinned message storage
CREATE TABLE IF NOT EXISTS pinned_messages (
    id              TEXT PRIMARY KEY,
    message_id      TEXT NOT NULL,
    channel         TEXT NOT NULL,
    task_id         TEXT,
    pinned_by       TEXT NOT NULL,
    pin_note        TEXT,
    pinned_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_pinned_channel ON pinned_messages(channel);
CREATE INDEX IF NOT idx_pinned_task ON pinned_messages(task_id);

-- 3. chat_bookmarks: User bookmarks in conversations
CREATE TABLE IF NOT EXISTS chat_bookmarks (
    id              TEXT PRIMARY KEY,
    message_id      TEXT NOT NULL,
    channel         TEXT NOT NULL,
    thread_id       TEXT,
    bookmark_note   TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 4. session_metadata: Extended session information
CREATE TABLE IF NOT EXISTS session_metadata (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL UNIQUE,
    model           TEXT,
    thinking_level  TEXT,
    verbose_level   TEXT,
    fast_mode       INTEGER DEFAULT 0,
    send_policy     TEXT,
    activation_mode TEXT,
    group_policy    TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_session_model ON session_metadata(model);
```

### Backend Changes

#### New API Module: `api/dashboard.rs`

```rust
// Endpoints to add:
pub fn router() -> Router<AppState> {
    Router::new()
        // Dashboard layout
        .route("/api/v1/dashboard/layout", get(get_layout).put(update_layout))
        
        // Pinned messages
        .route("/api/v1/dashboard/pins", get(list_pins).post(pin_message))
        .route("/api/v1/dashboard/pins/:id", delete(unpin_message))
        
        // Chat search
        .route("/api/v1/dashboard/search", get(search_messages))
        
        // Session management
        .route("/api/v1/dashboard/sessions", get(list_sessions))
        .route("/api/v1/dashboard/sessions/:id", get(get_session).patch(update_session))
        
        // Command palette
        .route("/api/v1/dashboard/commands", get(list_commands))
        
        // Export
        .route("/api/v1/dashboard/export", post(export_conversation))
}
```

#### Request/Response Types

```rust
#[derive(Deserialize)]
pub struct DashboardLayoutRequest {
    pub layout_config: String,  // JSON
    pub theme: Option<String>,
}

#[derive(Deserialize)]
pub struct PinMessageRequest {
    pub message_id: String,
    pub channel: String,
    pub task_id: Option<String>,
    pub pin_note: Option<String>,
}

#[derive(Deserialize)]
pub struct SearchMessagesRequest {
    pub q: String,
    pub channel: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateSessionRequest {
    pub model: Option<String>,
    pub thinking_level: Option<String>,
    pub verbose_level: Option<String>,
    pub fast_mode: Option<bool>,
    pub send_policy: Option<String>,
}

#[derive(Serialize)]
pub struct DashboardLayoutResponse {
    pub layout_config: serde_json::Value,
    pub theme: String,
}

#[derive(Serialize)]
pub struct PinnedMessageResponse {
    pub id: String,
    pub message_id: String,
    pub channel: String,
    pub task_id: Option<String>,
    pub pin_note: Option<String>,
    pub pinned_by: String,
    pub pinned_at: String,
}

#[derive(Serialize)]
pub struct ChatSearchResult {
    pub messages: Vec<MessageResponse>,
    pub total: i32,
}
```

### UI Changes

#### New Components

| Component | File | Description |
|-----------|------|-------------|
| `DashboardLayout` | `ui/src/components/dashboard/DashboardLayout.tsx` | Main dashboard container |
| `DashboardChat` | `ui/src/components/dashboard/ChatView.tsx` | Chat panel with toolbar |
| `CommandPalette` | `ui/src/components/ui/CommandPalette.tsx` | Ctrl+K command palette |
| `PinnedMessages` | `ui/src/components/dashboard/PinnedMessages.tsx` | Pinned messages sidebar |
| `MessageSearch` | `ui/src/components/dashboard/MessageSearch.tsx` | Full-text search UI |
| `SessionManager` | `ui/src/components/dashboard/SessionManager.tsx` | Session list and picker |
| `ExportDialog` | `ui/src/components/dashboard/ExportDialog.tsx` | Conversation export modal |

#### Integration Points

1. **App.tsx**: Add new 'dashboard' tab, integrate CommandPalette
2. **Sidebar.tsx**: Add dashboard icon and layout
3. **WebSocket**: Add new event types for pins, sessions
4. **API**: Use existing `signedFetch` for all dashboard calls

#### Component Structure

```tsx
// ui/src/components/dashboard/DashboardLayout.tsx
import { useState } from 'react';
import { ChatView } from './ChatView';
import { CommandPalette } from '../ui/CommandPalette';
import { PinnedMessages } from './PinnedMessages';
import { SessionManager } from './SessionManager';

export function DashboardLayout() {
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  
  // Keyboard shortcut for command palette
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        setShowCommandPalette(true);
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);
  
  return (
    <div className="flex h-full">
      <div className="flex-1 flex flex-col">
        <DashboardToolbar />
        <ChatView />
      </div>
      <Sidebar>
        <PinnedMessages />
        <SessionManager />
      </Sidebar>
      {showCommandPalette && (
        <CommandPalette onClose={() => setShowCommandPalette(false)} />
      )}
    </div>
  );
}
```

---

## Phase 2: Fast Mode & Provider Plugins (P0)

### Overview

Add session-level fast mode toggles and implement a modular LLM provider architecture.

### Database Changes

```sql
-- Migration: 016_fast_mode_providers

-- 1. provider_plugins: Modular LLM provider configuration
CREATE TABLE IF NOT EXISTS provider_plugins (
    id              TEXT PRIMARY KEY,
    provider_type   TEXT NOT NULL,  -- 'ollama', 'vllm', 'sglang', 'minimax', 'openrouter'
    name            TEXT NOT NULL,
    base_url        TEXT NOT NULL,
    api_key         TEXT,  -- encrypted
    default_model   TEXT,
    config          TEXT,  -- JSON: provider-specific settings
    enabled         INTEGER DEFAULT 1,
    priority        INTEGER DEFAULT 100,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_provider_type ON provider_plugins(provider_type);
CREATE INDEX IF NOT idx_provider_enabled ON provider_plugins(enabled);

-- 2. session_fast_mode: Per-session fast mode state
CREATE TABLE IF NOT EXISTS session_fast_mode (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL UNIQUE,
    fast_enabled    INTEGER DEFAULT 0,
    fast_model      TEXT,  -- model to use in fast mode
    fast_config     TEXT,  -- JSON: { temperature: 0.0, max_tokens: 1024 }
    toggles         TEXT,  -- JSON: { thinking: false, verbose: false }
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 3. model_fallbacks: Fallback chain for models
CREATE TABLE IF NOT EXISTS model_fallbacks (
    id              TEXT PRIMARY KEY,
    primary_model   TEXT NOT NULL,
    fallback_model  TEXT NOT NULL,
    provider        TEXT,
    priority        INTEGER DEFAULT 1,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(primary_model, fallback_model)
);
```

### Backend Changes

#### Enhanced API: `api/llms.rs`

```rust
// New endpoints to add:
pub fn router() -> Router<AppState> {
    Router::new()
        // Existing endpoints...
        
        // Provider plugins
        .route("/api/v1/llms/providers/:type/plugins", get(list_provider_plugins))
        .route("/api/v1/llms/providers/:type/plugins", post(create_provider_plugin))
        .route("/api/v1/llms/providers/:type/plugins/:id", put(update_provider_plugin).delete(delete_provider_plugin))
        
        // Fast mode
        .route("/api/v1/llms/fast-mode", get(get_fast_mode).put(update_fast_mode))
        .route("/api/v1/llms/sessions/:session_id/fast-mode", get(get_session_fast_mode).put(set_session_fast_mode))
        
        // Model fallbacks
        .route("/api/v1/llms/fallbacks", get(list_fallbacks).post(create_fallback))
        .route("/api/v1/llms/fallbacks/:id", delete(delete_fallback))
}
```

#### Provider Plugin Architecture

```rust
// New trait for modular providers
pub trait LlmProvider: Send + Sync {
    fn provider_type(&self) -> &str;
    fn name(&self) -> &str;
    fn list_models(&self) -> impl Future<Output = Result<Vec<ModelInfo>, Error>>;
    fn generate(&self, request: GenerateRequest) -> impl Future<Output = Result<GenerateResponse, Error>>;
    fn validate(&self) -> impl Future<Output = Result<bool, Error>>;
}

// Built-in provider implementations
pub struct OllamaProvider { /* ... */ }
pub struct VllmProvider { /* ... */ }
pub struct SglangProvider { /* ... */ }
pub struct MiniMaxProvider { /* ... */ }

// Provider registry
pub struct ProviderRegistry {
    providers: HashMap<String, Box<dyn LlmProvider>>,
}

impl ProviderRegistry {
    pub fn register(&mut self, provider: Box<dyn LlmProvider>) { /* ... */ }
    pub fn get(&self, provider_type: &str) -> Option<&dyn LlmProvider> { /* ... */ }
}
```

### UI Changes

#### Enhanced Components

| Component | File | Changes |
|-----------|------|---------|
| `LiteLlmSettings` | `ui/src/components/settings/LiteLlmSettings.tsx` | Add provider plugin management |
| `ChatModelSettings` | `ui/src/components/settings/ChatModelSettings.tsx` | Add fast mode toggle, fallback config |
| `ModelPicker` | `ui/src/components/dashboard/ModelPicker.tsx` | NEW: Dropdown with fast mode option |

#### Fast Mode Toggle UI

```tsx
// ui/src/components/dashboard/FastModeToggle.tsx
import { useState } from 'react';

interface FastModeToggleProps {
  sessionId: string;
  enabled: boolean;
  fastModel?: string;
  onToggle: (enabled: boolean) => void;
}

export function FastModeToggle({ sessionId, enabled, fastModel, onToggle }: FastModeToggleProps) {
  const [loading, setLoading] = useState(false);
  
  const handleToggle = async () => {
    setLoading(true);
    try {
      await signedFetch(`/api/v1/llms/sessions/${sessionId}/fast-mode`, {
        method: 'PUT',
        body: JSON.stringify({ fast_enabled: !enabled }),
      });
      onToggle(!enabled);
    } finally {
      setLoading(false);
    }
  };
  
  return (
    <button
      onClick={handleToggle}
      disabled={loading}
      className={`px-3 py-1 rounded-full text-sm font-medium transition-colors ${
        enabled 
          ? 'bg-indigo-500 text-white' 
          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
      }`}
    >
      {loading ? '...' : enabled ? '⚡ Fast' : 'Fast'}
    </button>
  );
}
```

---

## Phase 3: sessions_yield & sessions_resume (P1)

### Overview

Enhance subagent control flow with:
- `sessions_yield`: End current turn immediately, skip queued tool work
- `sessions_resume`: Resume existing conversation sessions

### Database Changes

```sql
-- Migration: 017_subagent_control

-- 1. session_yield_log: Track yield events
CREATE TABLE IF NOT EXISTS session_yield_log (
    id              TEXT PRIMARY KEY,
    parent_session  TEXT NOT NULL,
    child_session   TEXT NOT NULL,
    yield_reason    TEXT,  -- 'completed', 'yielded', 'error', 'timeout'
    yield_payload   TEXT,  -- JSON: hidden payload for next turn
    resumed_at       TEXT,
    created_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_yield_parent ON session_yield_log(parent_session);
CREATE INDEX IF NOT idx_yield_child ON session_yield_log(child_session);

-- 2. session_resume_history: Resume conversation history
CREATE TABLE IF NOT EXISTS session_resume_history (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    resumed_from    TEXT NOT NULL,  -- original session_id
    resume_type     TEXT NOT NULL,  -- 'acp', 'codex', 'manual'
    context_summary TEXT,  -- Summary of previous context
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_resume_session ON session_resume_history(session_id);

-- 3. session_attachments: File attachments for sessions
CREATE TABLE IF NOT EXISTS session_attachments (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    task_id         TEXT,
    file_name       TEXT NOT NULL,
    file_type       TEXT NOT NULL,
    file_size       INTEGER NOT NULL,
    file_path       TEXT NOT NULL,  -- Local path or base64 for small files
    encoding        TEXT DEFAULT 'binary',  -- 'binary', 'base64', 'utf8'
    uploaded_by     TEXT NOT NULL,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_attachment_session ON session_attachments(session_id);
CREATE INDEX IF NOT idx_attachment_task ON session_attachments(task_id);
```

### Backend Changes

#### New API Module: `api/sessions.rs`

```rust
pub fn router() -> Router<AppState> {
    Router::new()
        // Subagent yield
        .route("/api/v1/sessions/:session_id/yield", post(yield_session))
        
        // Session resume
        .route("/api/v1/sessions/resume", post(resume_session))
        .route("/api/v1/sessions/:session_id/resume-history", get(get_resume_history))
        
        // Session attachments
        .route("/api/v1/sessions/:session_id/attachments", get(list_attachments).post(upload_attachment))
        .route("/api/v1/sessions/:session_id/attachments/:attachment_id", get(get_attachment).delete(delete_attachment))
        .route("/api/v1/sessions/:session_id/attachments/:attachment_id/download", get(download_attachment))
}

// Handler implementations
async fn yield_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(payload): Json<YieldRequest>,
) -> Result<Json<YieldResponse>, String> { /* ... */ }

async fn resume_session(
    State(state): State<AppState>,
    Json(payload): Json<ResumeRequest>,
) -> Result<Json<ResumeResponse>, String> { /* ... */ }

async fn upload_attachment(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    // Multipart form upload
) -> Result<Json<AttachmentResponse>, String> { /* ... */ }

#[derive(Deserialize)]
pub struct YieldRequest {
    pub yield_payload: Option<String>,  // Hidden payload for next turn
    pub skip_tool_work: Option<bool>,   // Skip queued tools
    pub reason: Option<String>,
}

#[derive(Serialize)]
pub struct YieldResponse {
    pub yield_id: String,
    pub status: String,
    pub next_session_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ResumeRequest {
    pub resume_session_id: Option<String>,  -- Resume existing
    pub original_session_id: Option<String>, -- From archive
    pub resume_type: String,  // 'acp', 'codex', 'manual'
    pub context_options: Option<ContextOptions>,
}

#[derive(Serialize)]
pub struct ResumeResponse {
    pub session_id: String,
    pub resumed_from: String,
    pub context_loaded: bool,
}
```

### Integration with Subagent System

```rust
// In subagent.rs - enhance SubagentPool
impl SubagentPool {
    pub async fn yield_session(&self, session_id: &str, payload: Option<String>) -> Result<(), Error> {
        // 1. Log yield event
        // 2. Store yield payload for next turn
        // 3. Signal to skip remaining tool work
        // 4. Update session state
    }
    
    pub async fn resume_session(&self, original_session_id: &str) -> Result<String, Error> {
        // 1. Load context summary
        // 2. Create new session with context
        // 3. Link to original session
        // 4. Return new session ID
    }
}
```

---

## Phase 4: PDF Tool (P1)

### Overview

Add native PDF analysis capability using Anthropic/Google PDF providers.

### Database Changes

```sql
-- Migration: 018_pdf_tool

-- 1. pdf_documents: PDF processing cache
CREATE TABLE IF NOT EXISTS pdf_documents (
    id              TEXT PRIMARY KEY,
    file_name       TEXT NOT NULL,
    file_hash       TEXT NOT NULL,  -- SHA256 for deduplication
    file_size       INTEGER NOT NULL,
    page_count      INTEGER,
    extracted_text  TEXT,
    metadata        TEXT,  -- JSON: { author, created, title, etc. }
    provider        TEXT NOT NULL,  -- 'anthropic', 'google', 'fallback'
    model_used      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at      TEXT  -- TTL for cache
);

CREATE INDEX IF NOT idx_pdf_hash ON pdf_documents(file_hash);
CREATE INDEX IF NOT idx_pdf_provider ON pdf_documents(provider);

-- 2. pdf_extraction_jobs: Background job tracking
CREATE TABLE IF NOT EXISTS pdf_extraction_jobs (
    id              TEXT PRIMARY KEY,
    document_id     TEXT NOT NULL,
    status          TEXT NOT NULL,  -- 'pending', 'processing', 'completed', 'failed'
    provider        TEXT NOT NULL,
    error_message   TEXT,
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT
);
```

### Backend Changes

#### New API Module: `api/pdf.rs`

```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/pdf/upload", post(upload_pdf))
        .route("/api/v1/pdf/:id", get(get_pdf))
        .route("/api/v1/pdf/:id/text", get(extract_text))
        .route("/api/v1/pdf/:id/analyze", post(analyze_pdf))
        .route("/api/v1/pdf/:id", delete(delete_pdf))
        .route("/api/v1/pdf/jobs/:job_id", get(get_job_status))
}

// PDF Analysis Request
#[derive(Deserialize)]
pub struct PdfAnalyzeRequest {
    pub prompt: String,
    pub provider: Option<String>,  // 'anthropic', 'google', 'auto'
    pub options: Option<PdfOptions>,
}

#[derive(Serialize)]
pub struct PdfAnalyzeResponse {
    pub analysis: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: i32,
}
```

#### PDF Provider Implementation

```rust
// New module: src/pdf.rs
pub trait PdfProvider: Send + Sync {
    fn name(&self) -> &str;
    fn extract_text(&self, pdf_data: &[u8]) -> impl Future<Output = Result<PdfExtraction, Error>>;
    fn analyze(&self, pdf_data: &[u8], prompt: &str) -> impl Future<Output = Result<PdfAnalysis, Error>>;
}

pub struct AnthropicPdfProvider { /* ... */ }
pub struct GooglePdfProvider { /* ... */ }
pub struct FallbackPdfProvider { /* ... */ }  // Uses pdf-extract library

pub struct PdfManager {
    providers: Vec<Box<dyn PdfProvider>>,
    cache: DocumentCache,
}
```

### UI Changes

#### New Components

| Component | File | Description |
|-----------|------|-------------|
| `PdfUploader` | `ui/src/components/tools/PdfUploader.tsx` | Drag-drop PDF upload |
| `PdfViewer` | `ui/src/components/tools/PdfViewer.tsx` | View extracted text |
| `PdfAnalyzer` | `ui/src/components/tools/PdfAnalyzer.tsx` | Analysis prompt UI |

---

## Phase 5: Multimodal Memory (P1)

### Overview

Add image and audio indexing with Gemini embeddings for semantic memory search.

### Database Changes

```sql
-- Migration: 019_multimodal_memory

-- 1. memory_embeddings: Multimodal embeddings
CREATE TABLE IF NOT EXISTS memory_embeddings (
    id              TEXT PRIMARY KEY,
    memory_id       TEXT NOT NULL,
    memory_type     TEXT NOT NULL,  -- 'entity', 'knowledge', 'reflection', 'journal'
    modality        TEXT NOT NULL,  -- 'text', 'image', 'audio'
    embedding       TEXT NOT NULL,  -- Vector as JSON
    embedding_model TEXT NOT NULL,
    original_data   TEXT,  -- Base64 for image/audio
    mime_type       TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT idx_embedding_modality ON memory_embeddings(modality);
CREATE INDEX IF NOT idx_embedding_model ON memory_embeddings(embedding_model);

-- 2. memory_indexing_jobs: Background indexing
CREATE TABLE IF NOT EXISTS memory_indexing_jobs (
    id              TEXT PRIMARY KEY,
    memory_id       TEXT NOT NULL,
    modality        TEXT NOT NULL,
    status          TEXT NOT NULL,
    error_message   TEXT,
    started_at      TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at    TEXT
);

-- 3. memory_multimodal_config: Multimodal settings
CREATE TABLE IF NOT EXISTS memory_multimodal_config (
    id              TEXT PRIMARY KEY,
    image_indexing  INTEGER DEFAULT 0,
    audio_indexing  INTEGER DEFAULT 0,
    embedding_model  TEXT DEFAULT 'gemini-embedding-2-preview',
    embedding_dim    INTEGER DEFAULT 1536,
    enabled         INTEGER DEFAULT 1,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Backend Changes

#### Enhanced API: `api/memory.rs`

```rust
// New endpoints:
.route("/api/v1/memory/multimodal-config", get(get_multimodal_config).put(update_multimodal_config))
.route("/api/v1/memory/index", post(index_memory))  // Index image/audio
.route("/api/v1/memory/search", get(search_memory_multimodal))  // Enhanced search
```

#### Multimodal Embedding Service

```rust
// New module: src/multimodal.rs
pub struct MultimodalEmbeddingService {
    gemini_client: GeminiClient,
    embedding_dim: usize,
}

impl MultimodalEmbeddingService {
    pub async fn embed_image(&self, image_data: &[u8]) -> Result<Vec<f32>, Error> { /* ... */ }
    pub async fn embed_audio(&self, audio_data: &[u8]) -> Result<Vec<f32>, Error> { /* ... */ }
    pub async fn search(&self, query: &str, modality: Option<&str>, limit: usize) -> Result<Vec<MemorySearchResult>, Error> { /* ... */ }
}
```

### UI Changes

#### Enhanced Components

| Component | File | Changes |
|-----------|------|---------|
| `MemoryViewer` | `ui/src/components/memory/MemoryViewer.tsx` | Add modality filter (text/image/audio) |
| `MemoryStatsDashboard` | `ui/src/components/memory/MemoryStatsDashboard.tsx` | Show multimodal stats |
| `Settings` | `ui/src/components/settings/Settings.tsx` | Add multimodal config tab |

---

## Phase 6: Additional Messaging Channels (P2)

### Overview

Add support for additional messaging channels that APEX doesn't currently have.

### Current APEX Channels (8)
- REST API, Slack, Discord, Telegram, WhatsApp, Email, WebSocket, NATS

### Target Channels (Add 10+)
- Signal, IRC, Microsoft Teams, Matrix, Feishu, LINE, Mattermost, Nostr, Synology Chat, WebChat

### Implementation Strategy

For each channel, create an adapter following the existing pattern:

```
gateway/src/adapters/
├── signal/           # NEW
├── irc/              # NEW
├── matrix/           # NEW
├── teams/            # NEW
├── feishu/           # NEW
├── line/             # NEW
├── mattermost/       # NEW
├── nostr/            # NEW
└── ...existing...
```

#### Database Schema

```sql
-- Migration: 020_messaging_channels

-- Channel configuration (extend existing)
ALTER TABLE channels ADD COLUMN IF NOT EXISTS adapter_config TEXT;
ALTER TABLE channels ADD COLUMN IF NOT EXISTS credentials_encrypted TEXT;

-- Per-channel settings
CREATE TABLE IF NOT EXISTS channel_settings (
    id              TEXT PRIMARY KEY,
    channel_type    TEXT NOT NULL,
    channel_id      TEXT NOT NULL,
    settings        TEXT NOT NULL,  -- JSON: channel-specific config
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
```

#### Adapter Interface

```rust
// gateway/src/adapters/base.rs - extend existing BaseAdapter
pub trait MessagingAdapter: Send + Sync {
    fn adapter_type(&self) -> &str;
    fn name(&self) -> &str;
    
    async fn send_message(&self, to: &str, message: &Message) -> Result<MessageId, Error>;
    async fn receive_message(&self) -> impl Stream<Item = Message> + '_;
    async fn send_media(&self, to: &str, media: &Media) -> Result<MessageId, Error>;
    
    // Optional: typing indicators, read receipts, etc.
    async fn send_typing(&self, to: &str) -> Result<(), Error> { Ok(()) }
}
```

---

## Phase 7: Secrets Expansion (P2)

### Overview

Expand SecretRef support to cover more credential targets (64 targets like OpenClaw).

### Database Changes

```sql
-- Migration: 021_secrets_expansion

-- Extend existing encrypted_secrets table with more categories
ALTER TABLE encrypted_secrets ADD COLUMN IF NOT EXISTS category TEXT DEFAULT 'generic';
ALTER TABLE encrypted_secrets ADD COLUMN IF NOT EXISTS runtime_collector TEXT;
ALTER TABLE encrypted_secrets ADD COLUMN IF NOT EXISTS secret_tags TEXT;  -- JSON array

-- Secret references (for runtime resolution)
CREATE TABLE IF NOT EXISTS secret_refs (
    id              TEXT PRIMARY KEY,
    ref_key         TEXT NOT NULL UNIQUE,
    secret_name     TEXT NOT NULL,
    env_var         TEXT,  -- Maps to this env var
    description     TEXT,
    targets         TEXT NOT NULL,  -- JSON: ['tool.read', 'skill.foo', etc.]
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Secret rotation log
CREATE TABLE IF NOT EXISTS secret_rotation_log (
    id              TEXT PRIMARY KEY,
    secret_name     TEXT NOT NULL,
    rotated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    rotated_by      TEXT,
    status          TEXT NOT NULL
);
```

---

## Phase 8: Slack Block Kit (P2)

### Overview

Support rich Slack messages using Block Kit.

### Database Changes

```sql
-- Migration: 022_slack_blocks

-- Slack message templates
CREATE TABLE IF NOT EXISTS slack_block_templates (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    template        TEXT NOT NULL,  -- JSON: Block Kit template
    description     TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Pre-built templates
INSERT OR IGNORE INTO slack_block_templates (id, name, template, description) VALUES
    (ulid_generate(), 'task_complete', '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "✅ Task Complete"}}]}', 'Task completion notification'),
    (ulid_generate(), 'error_alert', '{"blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "⚠️ Error Alert"}}]}', 'Error notification');
```

### Backend Changes

```rust
// Extend existing slack adapter
pub struct SlackAdapter {
    // Existing fields...
    block_kit_enabled: bool,
}

impl SlackAdapter {
    pub async fn send_blocks(&self, channel: &str, blocks: &[Block]) -> Result<MessageId, Error> { /* ... */ }
    
    pub async fn send_rich_message(&self, to: &str, message: &RichMessage) -> Result<MessageId, Error> {
        if self.block_kit_enabled && !message.blocks.is_empty() {
            self.send_blocks(to, &message.blocks).await
        } else {
            self.send_message(to, &message.text).await
        }
    }
}

#[derive(Deserialize)]
pub struct RichMessage {
    pub text: String,
    pub blocks: Vec<Block>,  // Slack Block Kit
    pub attachments: Vec<Attachment>,
}
```

---

## Implementation Order & Dependencies

```
Phase 1: Control UI Dashboard
├── Requires: New dashboard layout table, pinned_messages table
├── Dependencies: None
└── Risks: Medium - New UI patterns

Phase 2: Fast Mode & Provider Plugins  
├── Requires: provider_plugins, session_fast_mode tables
├── Dependencies: Phase 1 (for session UI)
└── Risks: Low - Extension of existing LLM system

Phase 3: sessions_yield & sessions_resume
├── Requires: session_yield_log, session_resume_history, session_attachments tables
├── Dependencies: Phase 1 (session management UI)
└── Risks: Medium - Changes to subagent flow

Phase 4: PDF Tool
├── Requires: pdf_documents, pdf_extraction_jobs tables
├── Dependencies: Phase 2 (provider architecture for PDF providers)
└── Risks: Medium - New tool integration

Phase 5: Multimodal Memory
├── Requires: memory_embeddings, memory_indexing_jobs tables
├── Dependencies: Phase 1 (memory viewer)
└── Risks: Medium - New embedding pipeline

Phase 6: Additional Channels
├── Requires: channel_settings table
├── Dependencies: None
└── Risks: High - Each channel is different

Phase 7: Secrets Expansion
├── Requires: secret_refs, secret_rotation_log tables
├── Dependencies: Phase 2 (provider plugins for secret resolution)
└── Risks: Low - Extension of existing system

Phase 8: Slack Block Kit
├── Requires: slack_block_templates table
├── Dependencies: Phase 6 (Slack adapter exists)
└── Risks: Low - Extension of existing adapter
```

---

## Testing Strategy

### Unit Tests
- Database migrations (verify schema)
- API endpoint handlers (request/response)
- Provider implementations (mock responses)
- UI components (render tests)

### Integration Tests
- End-to-end flow: Dashboard → Session → Subagent → Result
- PDF upload → Extract → Analyze
- Memory: Index image → Search → Retrieve
- Channel adapters: Send → Receive → Verify

### Test Files Location
- Rust: `core/router/tests/integration/`
- Python: `execution/tests/`
- TypeScript: `gateway/src/*.test.ts`, `skills/src/*.test.ts`
- UI: `ui/src/**/*.test.tsx`

---

## Migration Naming Convention

| Migration | Name | Description |
|-----------|------|-------------|
| 015 | `control_ui` | Dashboard layout, pins, bookmarks |
| 016 | `fast_mode_providers` | Provider plugins, fast mode, fallbacks |
| 017 | `subagent_control` | Yield, resume, attachments |
| 018 | `pdf_tool` | PDF documents and extraction jobs |
| 019 | `multimodal_memory` | Image/audio embeddings and indexing |
| 020 | `messaging_channels` | Additional channel adapters |
| 021 | `secrets_expansion` | Secret refs and rotation |
| 022 | `slack_blocks` | Slack Block Kit templates |

---

## Environment Variables (New)

```bash
# PDF Tool
APEX_PDF_PROVIDER=anthropic  # anthropic, google, fallback
APEX_PDF_MAX_PAGES=100
APEX_PDF_MAX_SIZE_MB=50

# Multimodal Memory
APEX_MULTIMODAL_ENABLED=0    # Set to 1 to enable
APEX_MULTIMODAL_EMBED_MODEL=gemini-embedding-2-preview
APEX_MULTIMODAL_EMBED_DIM=1536

# Fast Mode
APEX_FAST_MODE_DEFAULT=0
APEX_FAST_MODEL=

# Provider Plugins
APEX_PROVIDER_PLUGIN_DIR=./providers

# Channels
APEX_SIGNAL_CLI_PATH=
APEX_IRC_SERVER=
APEX_MATRIX_HOMESERVER=
```

---

## Configuration Files (New)

```yaml
# config/providers.yaml
providers:
  ollama:
    enabled: true
    base_url: "http://localhost:11434"
    default_model: "qwen2.5:7b"
    
  vllm:
    enabled: false
    base_url: "http://localhost:8000"
    
  sglang:
    enabled: false
    base_url: "http://localhost:30000"
    
  minimax:
    enabled: false
    api_key: "${MINIMAX_API_KEY}"

# config/channels.yaml
channels:
  signal:
    enabled: false
    phone_number: "+1234567890"
    
  irc:
    enabled: false
    server: "irc.libera.chat"
    nick: "apex_bot"
    
  matrix:
    enabled: false
    homeserver: "https://matrix.org"
    user_id: "@apex:matrix.org"

# config/pdf.yaml
pdf:
  provider: "anthropic"
  max_pages: 100
  max_size_mb: 50
  cache_ttl_hours: 24

# config/multimodal.yaml
multimodal:
  enabled: false
  embedding_model: "gemini-embedding-2-preview"
  embedding_dim: 1536
  image_indexing: true
  audio_indexing: true
```

---

## Success Criteria

| Phase | Feature | Criteria |
|-------|---------|----------|
| 1 | Control UI | Dashboard loads in <2s, Command palette works, Pins persist |
| 2 | Fast Mode | Toggle works per-session, Providers list/connect |
| 3 | sessions_yield | Yield skips tool queue, Resume restores context |
| 4 | PDF Tool | Upload → Extract → Analyze completes |
| 5 | Multimodal | Image uploads, embeddings generated, search finds |
| 6 | Channels | Each new channel connects and messages |
| 7 | Secrets | Runtime collectors resolve, rotation works |
| 8 | Slack Blocks | Rich messages render in Slack |

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Too many features at once | Quality degradation | Phased implementation with testing at each phase |
| Provider plugin complexity | Runtime errors | Comprehensive provider validation before registration |
| PDF provider API changes | Breakage | Fallback provider always available |
| Multimodal embedding costs | Budget overrun | Configurable limits, caching, TTL |
| Channel complexity | Long tail of issues | Prioritize popular channels, community for niche |

---

## Appendix: File Paths Reference

### Backend (Rust)
```
core/router/src/
├── api/
│   ├── dashboard.rs       # NEW
│   ├── sessions.rs        # NEW (subagent control)
│   ├── pdf.rs             # NEW
│   └── llms.rs            # ENHANCED
├── pdf.rs                 # NEW
├── multimodal.rs          # NEW
├── llm/
│   ├── providers/         # NEW directory
│   │   ├── mod.rs
│   │   ├── ollama.rs
│   │   ├── vllm.rs
│   │   ├── sglang.rs
│   │   └── minimax.rs
│   └── registry.rs        # NEW
└── main.rs                # ENHANCED (routes)

core/memory/migrations/
├── 015_control_ui.sql
├── 016_fast_mode_providers.sql
├── 017_subagent_control.sql
├── 018_pdf_tool.sql
├── 019_multimodal_memory.sql
├── 020_messaging_channels.sql
├── 021_secrets_expansion.sql
└── 022_slack_blocks.sql
```

### Frontend (React)
```
ui/src/
├── components/
│   ├── dashboard/
│   │   ├── DashboardLayout.tsx     # NEW
│   │   ├── ChatView.tsx            # NEW
│   │   ├── PinnedMessages.tsx      # NEW
│   │   ├── SessionManager.tsx      # NEW
│   │   ├── ModelPicker.tsx         # NEW
│   │   ├── CommandPalette.tsx      # NEW
│   │   ├── ExportDialog.tsx       # NEW
│   │   └── FastModeToggle.tsx     # NEW
│   ├── tools/
│   │   ├── PdfUploader.tsx         # NEW
│   │   ├── PdfViewer.tsx           # NEW
│   │   └── PdfAnalyzer.tsx         # NEW
│   └── settings/
│       └── MultimodalSettings.tsx  # NEW
├── lib/
│   ├── api/
│   │   ├── dashboard.ts            # NEW
│   │   ├── pdf.ts                  # NEW
│   │   └── sessions.ts              # NEW
│   └── websocket.ts                # ENHANCED (events)
└── stores/
    └── appStore.ts                 # ENHANCED (state)
```

### Gateway (TypeScript)
```
gateway/src/
├── adapters/
│   ├── signal/                      # NEW
│   ├── irc/                        # NEW
│   ├── matrix/                     # NEW
│   ├── teams/                      # NEW
│   ├── feishu/                     # NEW
│   ├── line/                       # NEW
│   ├── mattermost/                 # NEW
│   └── nostr/                      # NEW
└── index.ts                        # ENHANCED (register adapters)
```

---

## Implementation Status

### Completed Phases ✅

| Phase | Feature | Status | Notes |
|-------|---------|--------|-------|
| Phase 1 | Control UI Dashboard | ✅ Complete | DashboardLayout, PinnedMessages, SessionManager, CommandPalette |
| Phase 2 | Fast Mode & Provider Plugins | ✅ Complete | provider_repo, FastModeToggle, ModelPicker |
| Phase 3 | sessions_yield & sessions_resume | ✅ Complete | session_control_repo, sessions API, SessionControl UI |
| Phase 4 | PDF Tool | ✅ Complete | pdf_repo, PDF API, PdfUploader, PdfViewer, PdfAnalyzer |
| Phase 5 | Multimodal Memory | ✅ Complete | multimodal_repo, MultimodalMemory API, MultimodalMemory UI |
| Phase 6 | Additional Channels | ✅ Complete | channel_settings_repo, channels_extended API, ChannelManager |
| Phase 7 | Secrets Expansion | ✅ Complete | secrets_repo, secrets API, SecretsManager UI (64 targets) |
| Phase 8 | Slack Block Kit | ✅ Complete | slack_block_repo, slack_blocks API, SlackBlockManager UI |
| Phase 9 | Death Spiral Detection | ✅ Complete | execution_pattern_repo, patterns API, anomaly detection (4 patterns) |

### Pending Phases

All Features Complete ✅

### Database Migrations

| Migration | Tables | Status |
|-----------|--------|--------|
| 015_control_ui | 4 tables | ✅ Applied |
| 016_fast_mode_providers | 5 tables | ✅ Applied |
| 017_subagent_control | 5 tables | ✅ Applied |
| 018_pdf_tool | 2 tables | ✅ Applied |
| 019_multimodal_memory | 3 tables | ✅ Applied |
| 020_messaging_channels | 3 tables | ✅ Applied |
| 021_secrets_expansion | 3 tables | ✅ Applied |
| 022_slack_blocks | 1 table | ✅ Applied |
| 023_execution_patterns | 2 tables | ✅ Applied |

---

*Document generated: 2026-03-13*
*Last updated: 2026-03-13*
