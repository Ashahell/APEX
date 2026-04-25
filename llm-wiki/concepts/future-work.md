# FUTURE_WORK — Implementation Roadmap

**Status:** Phase 1 of 4 complete
**Date:** 2026-04-25
**Source:** [docs/FUTURE_WORK.md](raw/FUTURE_WORK.md)

## Overview

4-phase roadmap based on OpenClaw v2026.4.23, Agent Zero v1.9, and Hermes v0.11.0.

## Phase 1: UX & Resilience ✅ Complete

| Item | Status | Key Files |
|------|--------|-----------|
| Stop-button abort persistence | ✅ | `task_repo.rs`, `skill_worker.rs`, `deep_task_worker.rs` |
| Lexical skill matching fallback | ✅ | `skill_triggers` migration, `skill_manager.rs` |
| Chat compaction | ✅ | `compaction.rs`, `Chat.tsx`, `Settings.tsx` |

**What Phase 1 delivers:**
- Cancellation survives reconnects (migration 025)
- 60+ keyword triggers for skill matching (migration 026)
- Manual compact button in Chat header + settings

## Phase 2: Memory System Enhancements

**Goal:** Improve memory integrity and search quality  
**Risk:** Medium | **Effort:** ~3-4 weeks

### 2.1 Memory Integrity Verification ✅ Complete
- Hash store table with chunk/vector hashes (migration 027)
- Integrity check on startup (SHA-256 sidecar)
- Repair option for flagged chunks
- API: `GET /api/v1/memory/integrity` (+ 5 more endpoints)
- 11 tests: 5 unit + 6 integration

### 2.2 Enhanced Session Search ✅ Complete
- Auto-summary on session end (>50 messages)
- Summary field in session search API
- "Summarized" badge in SessionSearch UI

### 2.3 Temporal Decay Improvements ✅ Complete
- MMR (Maximal Marginal Relevance) for deduplication
- Configurable decay curve (APEX_MEMORY_HALF_LIFE_DAYS)
- Quality score combining relevance + recency + frequency boost
- Configurable mmr_lambda via APEX_MEMORY_MMR_LAMBDA

## Phase 3: LLM Provider Abstraction

**Goal:** Support more providers and improve transport layer  
**Risk:** Medium | **Effort:** ~4-6 weeks

### 3.1 Transport Layer Abstraction ✅ Complete
- `LlmTransport` trait with `chat()` and `provider_name()`
- LlamaClient implements trait (llama-server, Ollama, LM Studio)
- OpenAiTransport for OpenAI-compatible APIs
- Provider-specific retry logic

### 3.2 Fast Mode ✅ Complete
- Task priority: Fast (skips normal routing)
- Direct LLM execution when priority: "fast"
- Set via API: `priority: "fast"` or keyboard shortcut hint (Ctrl+Shift+F)

### 3.3 Multi-Model Routing ✅ Complete
- Per-task model selection via `model` field
- DeepTaskMessage now carries model preference
- API: POST /api/v1/deep with `model: "qwen3-4b"`

### 3.3 Multi-Model Routing
- Per-task model assignment (primary, auxiliary)
- Models for: compression, vision, search, title generation
- UI: Model Assignment panel

## Phase 4: Advanced Features

**Goal:** Enable advanced agent capabilities  
**Risk:** High | **Effort:** ~6-8 weeks

### 4.1 Mid-Run Agent Nudges (/steer)
- Inject guidance without breaking cache
- `POST /api/v1/tasks/:id/steer`
- Show steer history in ProcessGroup

### 4.2 Subagent Pool Execution
- Decompose task via LLM
- Spawn parallel workers (max 4)
- File coordination for shared state
- Orchestrator role

### 4.3 Webhook Direct Delivery
- Deliver without LLM involvement
- `POST /api/v1/webhooks/subscribe`
- Event routing: task.completed → direct, task.failed → agent

### 4.4 Shell Hooks
- Lifecycle hooks: pre/post tool call, session start/end
- Execute shell scripts with context JSON
- Timeout after 5 seconds

## Priority Matrix

| Feature | Impact | Effort | Risk | Priority |
|---------|--------|--------|------|----------|
| Stop-Button Persistence | High | Low | Low | **1** ✅ |
| Memory Integrity Check | High | Medium | Medium | **2** |
| Lexical Skill Fallback | Medium | Low | Low | **3** ✅ |
| Fast Mode | High | Medium | Low | **4** |
| Transport Abstraction | High | High | Medium | **5** |
| Chat Compaction | Medium | Medium | Low | **6** ✅ |
| Mid-Run Nudges (/steer) | Medium | Medium | Medium | **7** |
| Multi-Model Routing | High | High | Medium | **8** |
| Enhanced Session Search | Medium | Medium | Medium | **9** |
| Webhook Direct Delivery | Low | Medium | Low | **10** |
| Subagent Pool Execution | High | High | High | **11** |
| Shell Hooks | Low | Medium | Medium | **12** |

## Rollout Plan

1. **Phase 1 (Weeks 1-3):** ✅ Complete — UX improvements
2. **Phase 2 (Weeks 4-7):** Memory enhancements
3. **Phase 3 (Weeks 8-13):** LLM improvements
4. **Phase 4 (Weeks 14-21):** Advanced features

## Related

- [phase-1-ux-improvements.md](concepts/phase-1-ux-improvements.md)
- [project-overview.md](project-overview.md)