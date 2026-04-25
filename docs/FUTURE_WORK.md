# APEX Future Work Plan

> **Status:** Draft  
> **Created:** 2026-04-25  
> **Sources:** OpenClaw v2026.4.23, Agent Zero v1.9, Hermes v0.11.0

---

## Executive Summary

This plan synthesizes findings from monitoring three parent repositories:
- **OpenClaw** (364K stars) - Extensibility, plugin ecosystem
- **Agent Zero** (17K stars) - Dark UI, execution patterns  
- **Hermes** (116K stars) - Bounded memory, self-improving agent

APEX v1.6.0 "Sapphire" already aligns with Hermes features. This plan prioritizes high-impact, low-risk improvements.

---

## Phase 1: UX & Resilience Improvements

**Goal:** Improve user experience and connection resilience  
**Risk:** Low (UI-only or additive features)  
**Effort:** ~2-3 weeks

### 1.1 Stop-Button Abort Persistence (OpenClaw #70673)

**Problem:** Agent continues running after WebSocket disconnect  
**Reference:** OpenClaw queues Stop-button aborts across reconnections

**Steps:**
```
[ ] 1. Add `cancelled` flag to task state in database
[ ] 2. Modify skill_worker.rs to check cancellation before each step
[ ] 3. Modify deep_task_worker.rs to check cancellation before each step
[ ] 4. Store cancellation requests in memory/repo with timestamp
[ ] 5. On reconnect, check pending cancellation requests
[ ] 6. Send cancellation signal to active execution context
[ ] 7. Update UI: show "Cancelling..." state during abort
[ ] 8. Add test for cancellation across reconnects
```

**Files to modify:**
- `core/memory/src/task_repo.rs` - Add cancellation state
- `core/router/src/skill_worker.rs` - Check cancellation flag
- `core/router/src/deep_task_worker.rs` - Check cancellation flag
- `core/router/src/api/tasks.rs` - Add cancel endpoint wiring
- `ui/src/stores/appStore.ts` - Persist cancellation state

---

### 1.2 Lexical Skill Matching Fallback (Agent Zero)

**Problem:** Vector search may fail or be slow for skill recall  
**Reference:** Agent Zero restored lexical trigger-word scoring

**Steps:**
```
[ ] 1. Define keyword triggers for each built-in skill
[ ] 2. Create skill_trigger table in memory database
[ ] 3. Implement keyword matching in skill_manager.rs
[ ] 4. Add fallback: try vector search, if empty try lexical
[ ] 5. Add weight scoring: exact match > partial match > keyword
[ ] 6. Add UI toggle in Settings for lexical fallback
[ ] 7. Add tests for lexical matching
```

**Files to modify:**
- `core/memory/migrations/` - Add skill_triggers migration
- `core/router/src/skill_manager.rs` - Add lexical matching
- `core/router/src/api/skills.rs` - Add trigger endpoints
- `ui/src/components/skills/` - Add settings toggle

---

### 1.3 Chat Compaction (Hermes)

**Problem:** Long conversations consume context window  
**Reference:** Hermes has chat compaction plugin

**Steps:**
```
[ ] 1. Create compaction service in core/router/src/
[ ] 2. Design compaction algorithm:
     - Summarize older messages using LLM
     - Replace message range with summary
     - Preserve tool calls and outcomes
[ ] 3. Add compaction trigger: auto at X tokens or manual /compact
[ ] 4. Add UI button in Chat header: "Compact Chat"
[ ] 5. Add settings: compaction threshold (default 50% context)
[ ] 6. Add test for compaction correctness
[ ] 7. Add compaction history to session
```

**Files to modify:**
- `core/router/src/compaction.rs` - New service
- `core/router/src/api/sessions.rs` - Add /compact endpoint
- `ui/src/components/chat/Chat.tsx` - Add compact button
- `ui/src/stores/appStore.ts` - Add compaction state

---

## Phase 2: Memory System Enhancements

**Goal:** Improve memory integrity and search quality  
**Risk:** Medium (database changes)  
**Effort:** ~3-4 weeks

### 2.1 Memory Integrity Verification

**Problem:** sqlite_vec index may become corrupted  
**Reference:** Agent Zero v1.8 added FAISS integrity via SHA-256 sidecar

**Steps:**
```
[ ] 1. Create hash_store table in memory database:
     - chunk_hash TEXT PRIMARY KEY
     - text_hash TEXT
     - vector_hash TEXT
     - created_at TIMESTAMP
[ ] 2. Implement hash computation in background_indexer.rs
[ ] 3. Add integrity check on startup:
     - Load all chunk hashes
     - Verify sqlite_vec matches
     - Flag discrepancies
[ ] 4. Add repair option: re-index flagged chunks
[ ] 5. Add API endpoint: GET /api/v1/memory/integrity
[ ] 6. Add UI panel in Memory tab: "Integrity Status"
[ ] 7. Add tests for integrity check
```

**Files to modify:**
- `core/memory/migrations/` - Add hash_store migration
- `core/memory/src/embedder.rs` - Add hash computation
- `core/memory/src/background_indexer.rs` - Verify on save
- `core/router/src/api/memory.rs` - Add integrity endpoints
- `ui/src/components/memory/` - Add Integrity panel

---

### 2.2 Enhanced Session Search (Hermes-style)

**Problem:** Current FTS5 search could be more intelligent  
**Reference:** Hermes uses LLM summarization for cross-session recall

**Steps:**
```
[ ] 1. Add summarization model config to LLM settings
[ ] 2. Create session_summary table:
     - session_id TEXT
     - summary TEXT
     - key_topics TEXT (JSON array)
     - created_at TIMESTAMP
     - updated_at TIMESTAMP
[ ] 3. Implement auto-summary on session end (>50 messages)
[ ] 4. Add LLM-powered search refinement:
     - Initial FTS5 search returns candidates
     - LLM re-ranks based on relevance
[ ] 5. Add context window extraction for results
[ ] 6. Update UI: show "Summarized" badge on old sessions
[ ] 7. Add tests for summarization quality
```

**Files to modify:**
- `core/router/src/api/session_search_api.rs` - Add LLM re-ranking
- `core/memory/migrations/` - Add session_summary migration
- `core/router/src/llama.rs` - Add summarization prompt
- `ui/src/components/chat/SessionSearch.tsx` - Show summaries

---

### 2.3 Temporal Decay Improvements

**Problem:** Current decay may not be aggressive enough  
**Reference:** OpenClaw's hybrid search exposes component scores

**Steps:**
```
[ ] 1. Implement MMR (Maximal Marginal Relevance) for deduplication
[ ] 2. Add recency weight tuning to unified config:
     - APEX_MEMORY_HALF_LIFE_DAYS already exists (default 30)
     - Add: APEX_MEMORY_DECAY_CURVE (linear/exponential)
[ ] 3. Add quality score to search results:
     - Relevance score from vector search
     - Recency score from temporal decay
     - Combine with RRF
[ ] 4. Add search result scoring to UI tooltips
[ ] 5. Add tests for scoring algorithm
```

**Files to modify:**
- `core/router/src/unified_config.rs` - Add decay config
- `core/memory/src/memory.rs` - Implement MMR
- `ui/src/components/memory/MemorySearch.tsx` - Show scores

---

## Phase 3: LLM Provider Abstraction

**Goal:** Support more providers and improve transport layer  
**Risk:** Medium (core API changes)  
**Effort:** ~4-6 weeks

### 3.1 Transport Layer Abstraction (Hermes-style)

**Problem:** LLM client handles both transport and format conversion  
**Reference:** Hermes has `AnthropicTransport`, `BedrockTransport`, etc.

**Steps:**
```
[ ] 1. Define Transport trait in core/router/src/llm/transports.rs:
     ```
     trait LLMTransport {
         async fn send(&self, request: Request) -> Result<Response>;
         fn format_request(&self, messages: Vec<Message>) -> Vec<u8>;
         fn parse_response(&self, bytes: Bytes) -> Result<StreamEvent>;
     }
     ```
[ ] 2. Create AnthropicTransport struct
[ ] 3. Create OpenAICompatTransport struct (for OpenRouter, LM Studio)
[ ] 4. Create BedrockTransport struct (future AWS support)
[ ] 5. Add transport selection to unified config
[ ] 6. Update llama.rs to use selected transport
[ ] 7. Add provider-specific retry logic per transport
[ ] 8. Add tests for each transport
```

**Files to modify:**
- `core/router/src/llm/transports.rs` - New trait + implementations
- `core/router/src/llm/mod.rs` - Update module
- `core/router/src/llama.rs` - Use transports
- `core/router/src/unified_config.rs` - Provider config

---

### 3.2 Fast Mode (Hermes/Agent Zero)

**Problem:** No priority queue for low-latency tasks  
**Reference:** Hermes `/fast` mode, Agent Zero priorities

**Steps:**
```
[ ] 1. Add task priority enum: Low, Normal, High, Fast
[ ] 2. Create priority queue in message_bus.rs
[ ] 3. Add Fast Mode toggle in Settings UI
[ ] 4. When Fast Mode enabled:
     - Use smaller model (qwen3-4B vs larger)
     - Skip skill worker (immediate execution)
     - Reduce max_steps (10 instead of 50)
[ ] 5. Add keyboard shortcut: Ctrl+Shift+F for Fast Mode
[ ] 6. Update UI: show "Fast" badge when active
[ ] 7. Add tests for Fast Mode behavior
```

**Files to modify:**
- `core/router/src/message_bus.rs` - Priority queue
- `core/router/src/api/tasks.rs` - Add priority field
- `core/router/src/unified_config.rs` - Fast mode config
- `ui/src/stores/appStore.ts` - Fast mode state
- `ui/src/components/settings/` - Fast mode toggle

---

### 3.3 Multi-Model Routing

**Problem:** All tasks use same model  
**Reference:** Hermes auxiliary models per task

**Steps:**
```
[ ] 1. Add model assignment to task create:
     - primary: Main LLM
     - auxiliary: Compression, Vision, Search, Title
[ ] 2. Create model routing config:
     ```
     model_routing:
       compression: qwen3-4b
       vision: qwen3-vl
       session_search: qwen3-4b
       title_generation: qwen3-4b
     ```
[ ] 3. Implement routing in agent_loop.rs
[ ] 4. Add UI: Model Assignment panel in Settings
[ ] 5. Add API: GET /api/v1/llms/models (with capabilities)
[ ] 6. Add tests for routing correctness
```

**Files to modify:**
- `core/router/src/api/tasks.rs` - Model assignment
- `core/router/src/agent_loop.rs` - Route to model
- `core/router/src/api/llms.rs` - Model listing
- `ui/src/components/settings/LLMSettings.tsx` - Routing UI

---

## Phase 4: Advanced Features

**Goal:** Enable advanced agent capabilities  
**Risk:** High (new patterns)  
**Effort:** ~6-8 weeks

### 4.1 Mid-Run Agent Nudges (/steer)

**Problem:** Can't guide agent mid-execution  
**Reference:** Hermes `/steer` injects guidance without breaking cache

**Steps:**
```
[ ] 1. Create steer injection mechanism:
     - Store steer message in task context
     - Inject after next tool call
     - Don't interrupt current turn
[ ] 2. Add API: POST /api/v1/tasks/:id/steer
[ ] 3. Add WebSocket event for steer injection
[ ] 4. Add UI: Chat input with /steer command
[ ] 5. Show steer history in ProcessGroup
[ ] 6. Add tests for steer injection
```

**Files to modify:**
- `core/router/src/api/tasks.rs` - Steer endpoint
- `core/router/src/agent_loop.rs` - Inject steer
- `core/router/src/message_bus.rs` - Steer events
- `ui/src/components/chat/Chat.tsx` - Steer UI
- `ui/src/stores/appStore.ts` - Steer state

---

### 4.2 Subagent Pool Execution

**Problem:** Subagent API exists but execution not wired  
**Reference:** Hermes smarter delegation, Agent Zero subagents

**Steps:**
```
[ ] 1. Design subagent execution flow:
     - Decompose task via LLM
     - Spawn parallel workers (limited to 4)
     - File coordination for shared state
[ ] 2. Implement file coordination layer:
     - Lock files for concurrent sibling edits
     - Merge strategy: last-write-wins or manual
[ ] 3. Wire subagent pool to execution engine
[ ] 4. Add orchestrator role (can spawn workers)
[ ] 5. Add max_spawn_depth config (default: 2)
[ ] 6. Add UI: Subagent Debug panel in Settings
[ ] 7. Add tests for parallel execution
```

**Files to modify:**
- `core/router/src/subagent.rs` - Execution wiring
- `core/router/src/execution_stream.rs` - Worker pool
- `core/router/src/api/subagent.rs` - Endpoints
- `ui/src/components/` - Subagent debug UI

---

### 4.3 Webhook Direct Delivery (Hermes)

**Problem:** All notifications go through agent  
**Reference:** Hermes webhook can deliver without LLM

**Steps:**
```
[ ] 1. Add webhook subscription model:
     - url: TEXT
     - events: TEXT (JSON array)
     - direct_delivery: BOOLEAN
[ ] 2. Create webhook handler (no agent involvement)
[ ] 3. Add API: POST /api/v1/webhooks/subscribe
[ ] 4. Add event routing:
     - task.completed -> direct delivery
     - task.failed -> agent notification
[ ] 5. Add UI: Webhook panel in Settings
[ ] 6. Add tests for direct delivery
```

**Files to modify:**
- `core/router/src/webhook.rs` - Direct delivery handler
- `core/router/src/api/webhooks.rs` - Subscription endpoints
- `ui/src/components/settings/WebhookSettings.tsx` - UI

---

### 4.4 Shell Hooks (Hermes)

**Problem:** Custom skill lifecycle requires Python plugin  
**Reference:** Hermes can wire shell scripts to lifecycle hooks

**Steps:**
```
[ ] 1. Define hook types:
     - pre_tool_call
     - post_tool_call
     - on_session_start
     - on_session_end
[ ] 2. Add hook config to unified config:
     ```
     shell_hooks:
       pre_tool_call: /path/to/hook.sh
       post_tool_call: null
     ```
[ ] 3. Implement hook runner:
     - Execute shell script with context JSON
     - Capture output for logging
     - Timeout after 5 seconds
[ ] 4. Add API: GET /api/v1/hooks, PUT /api/v1/hooks
[ ] 5. Add UI: Shell Hooks panel in Settings
[ ] 6. Add tests for hook execution
```

**Files to modify:**
- `core/router/src/unified_config.rs` - Hook config
- `core/router/src/hook_runner.rs` - New service
- `core/router/src/skill_worker.rs` - Call hooks
- `ui/src/components/settings/` - Hooks UI

---

## Implementation Priority Matrix

| Feature | Impact | Effort | Risk | Priority |
|---------|--------|--------|------|----------|
| Stop-Button Persistence | High | Low | Low | **1** |
| Memory Integrity Check | High | Medium | Medium | **2** |
| Lexical Skill Fallback | Medium | Low | Low | **3** |
| Fast Mode | High | Medium | Low | **4** |
| Transport Abstraction | High | High | Medium | **5** |
| Chat Compaction | Medium | Medium | Low | **6** |
| Mid-Run Nudges (/steer) | Medium | Medium | Medium | **7** |
| Multi-Model Routing | High | High | Medium | **8** |
| Enhanced Session Search | Medium | Medium | Medium | **9** |
| Webhook Direct Delivery | Low | Medium | Low | **10** |
| Subagent Pool Execution | High | High | High | **11** |
| Shell Hooks | Low | Medium | Medium | **12** |

---

## Testing Strategy

### Unit Tests (per feature)
- Feature-specific test in `tests/` directory
- Mock external dependencies
- Test happy path + error cases

### Integration Tests
- Test API endpoints with in-memory database
- Test worker execution flow
- Test WebSocket events

### E2E Tests (marked #[ignore])
- Full task lifecycle
- WebSocket reconnection
- Multi-step task execution

---

## Rollout Plan

1. **Phase 1 (Weeks 1-3):** UX improvements
   - Low risk, high visibility
   - Good for initial user feedback

2. **Phase 2 (Weeks 4-7):** Memory enhancements
   - Medium risk, database changes
   - Requires backup testing

3. **Phase 3 (Weeks 8-13):** LLM improvements
   - Higher risk, core API changes
   - Requires regression testing

4. **Phase 4 (Weeks 14-21):** Advanced features
   - Highest risk, significant changes
   - Consider feature flags

---

## Notes

- All features should maintain backward compatibility
- Use feature flags for experimental features
- Document breaking changes in CHANGELOG.md
- All UI changes must be responsive (mobile-friendly)
- Follow existing code conventions (see AGENTS.md)