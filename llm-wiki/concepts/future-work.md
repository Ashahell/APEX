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

### 4.1 Mid-Run Agent Nudges (/steer) ✅ Complete
- `POST /api/v1/tasks/:id/steer` endpoint
- SteerMessage in message_bus
- Use: `curl -X POST .../tasks/{id}/steer -d '{"direction": "focus on X"}'`

### 4.2 Subagent Pool Execution ✅ Complete
- `/api/v1/subagent/decompose` endpoint
- Parallel worker pool (max 4 workers)
- SubTask status tracking
- Orchestrator in deep_task_worker

### 4.3 Webhook Direct Delivery ✅ Complete
- Event-based webhook system
- `/api/v1/webhooks` CRUD endpoints
- trigger_event() for task.completed/failed

### 4.4 Shell Hooks ✅ Complete
- ShellHookConfig with env vars (APEX_SHELL_*)
- pre_tool_call, post_tool_call, session_start, session_end
- 5 second timeout
- ShellHooks module ready to use in skill_worker

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