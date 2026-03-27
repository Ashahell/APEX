# Changelog

All notable changes to APEX are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.7.0] — Streaming MVP — 2026-03-27

### Added

#### TinySSE Streaming Baseline (Patch 3B-12.2.3.2)
- **Clean streaming baseline** using `futures_util::stream::iter()` for deterministic in-memory SSE streams
- **Streaming endpoints**:
  - `GET /api/v1/stream/stats` - Streaming metrics (active connections, event counts)
  - `GET /api/v1/stream/hands/:task_id` - Hands agent SSE stream
  - `GET /api/v1/stream/mcp/:task_id` - MCP SSE stream
  - `GET /api/v1/stream/task/:task_id` - Task SSE stream
- **Type simplification**: Single `SSEItem` type alias: `Result<Event, axum::Error>`
- **Error handling**: `StreamingError` enum with variants (StreamingDisabled, ReplayDetected, Internal, StreamNotFound, AuthRequired)

#### CI Updates
- **Node 24**: Updated TypeScript and UI workflows from Node 20 to Node 24

#### Tests
- `core/router/tests/streaming_integration.rs` — 9 tests passing
- `core/router/tests/streaming_tinysse_tests.rs` — 2 tests passing

### Changed
- **streaming.rs**: Complete rewrite with stable iterator-based approach
- **streaming_types.rs**: Simplified to single SSEItem type alias

---

## [1.6.0] — Sapphire Features — 2026-03-25

### Added

#### Streaming / WebSocket (Patch 14)
- **WebSocket streaming** with ticket-based authentication (`WS /api/v1/stream/ws/:task_id`)
- **StreamTicket** struct with HMAC-SHA256 signing and `constant_time_eq` timing-attack safe verification
- **Ping/pong heartbeat** (30s interval) to detect stale connections
- `GET /api/v1/stream/ticket` endpoint for ticket issuance
- **WSClient** class (`ui/src/lib/ws.ts`) with auto-reconnect, heartbeat ping, and Zustand dispatch
- SSEClient → WSClient migration in `Chat.tsx` and `HandMonitor.tsx`

#### Streaming / Replay Protection (Patch 15)
- **`ReplayProtection` trait** with pluggable backends
- **`InMemoryReplayProtection`** — thread-local `HashSet` via `thread_local!` with `RefCell`. Zero overhead for single-process deployments. Test-isolated via `reset()`.
- **`RedisReplayProtection`** — atomic `SET key EX 300 NX` for cross-instance replay detection. Compiled only when the `redis` feature is enabled.
- `from_config(backend, redis_url)` factory selects backend at startup
- Environment variables: `APEX_REPLAY_BACKEND=memory|redis`, `APEX_REDIS_URL`

#### Streaming / Analytics (Patch 16)
- **`StreamingMetrics`** struct with 12 `AtomicU64` counters:
  - Connections: `active_connections`, `total_connections`
  - Events: `thought`, `tool_call`, `tool_progress`, `tool_result`, `approval`, `error`, `complete`
  - Errors: `auth`, `replay`, `internal`
- **`GET /api/v1/stream/stats`** endpoint returning `StreamingStats` snapshot (JSON)
- All counters wired in SSE handler and WebSocket handler

#### Additional Sapphire Features
- **Computer Use API** — orchestrator, VLM integration, screenshot capture (`core/router/src/computer_use/`)
- **Tool Sandbox** — runtime tool validation (`core/router/src/tool_sandbox.rs`, `tool_validator.rs`)
- **Story Engine** — narrative engine for scripted agent runs (`core/router/src/story_engine.rs`)
- **Persona Assembly** — configurable agent personas (`core/router/src/persona.rs`)
- **Privacy Guard** — telemetry controls (`core/router/src/privacy_guard.rs`)
- **Continuity Scheduler** — stateful task continuation (`core/router/src/continuity.rs`)
- **Context Scope Isolation** — token budget enforcement (`core/router/src/context_scope.rs`)
- **Plugin Signing** — skill manifest signing (`core/router/src/skill_signer.rs`)

#### Integration Tests
- `core/router/tests/streaming_integration.rs` — 7 tests (config gating, replay protection, error variants)
- `core/router/tests/auth_integration.rs` — 10 tests (HMAC, TOTP)
- `core/router/tests/memory_integration.rs` — skill manager tests
- `core/router/tests/skills_integration.rs` — skills CRUD tests
- Performance indexes migration `024_performance_indexes.sql`

### Changed

- **Streaming handler** now increments `streaming_metrics` on connect, disconnect, and per-event
- **`AppState`** extended with `replay_protection: Arc<dyn ReplayProtection>` and `streaming_metrics: Arc<StreamingMetrics>`
- **`StreamingConfig`** extended with `replay_backend: ReplayBackend` and `redis_url: Option<String>`

### Fixed

- Streaming SSE handler: removed duplicate `SinkExt`/`StreamExt` import, fixed socket mutability, fixed `JoinHandle` pinning in `tokio::select!`, fixed `Option` as `Future` pattern
- `InMemoryReplayProtection`: switched from `static Mutex<Option<HashSet>>` to `thread_local!` with `RefCell<HashSet>` to eliminate parallel test race conditions
- `deadpool-redis` crate now gated behind `#[cfg(feature = "redis")]`

### Security

- HMAC tickets use `constant_time_eq` for timing-attack safe signature comparison
- Replay protection prevents duplicate stream connections using the same signature
- All streaming endpoints still require HMAC signing (or ticket for WebSocket)

### Dependencies

- **`deadpool-redis`** added as optional dependency (feature: `redis`)

---

## [1.5.0] — Hermes Agent Integration — 2025-XX-XX

### Added
- **Bounded Curated Memory** — character-limited memory stores (2,200 agent / 1,375 user chars), automatic consolidation, frozen snapshot for system prompts
- **Agent-Managed Skills** — auto-create skills after 5+ tool calls, SKILL.md format with YAML frontmatter, security scanning
- **Skills Hub Client** — trust levels (Verified > Trusted > Community), hub configuration with request timeout
- **Session Search** — FTS5 virtual table, BM25 ranking, case-insensitive partial matching
- **User Profile** — communication styles, verbosity levels, response formats, system prompt additions

---

## [1.4.0] — OpenClaw Features — 2025-XX-XX

### Added
- **Control UI Dashboard** — DashboardLayout, PinnedMessages, SessionManager, CommandPalette
- **Fast Mode & Provider Plugins** — FastModeToggle, ModelPicker, provider_repo
- **sessions_yield & sessions_resume** — session control, SessionManager UI
- **PDF Tool** — pdf_repo, PDF API, PdfUploader, PdfViewer, PdfAnalyzer
- **Multimodal Memory** — multimodal_repo, MultimodalMemory API and UI
- **Additional Channels** — channel_settings_repo, ChannelManager
- **Secrets Expansion** — 64 secret targets, SecretsManager
- **Slack Block Kit** — slack_block_repo, SlackBlockManager
- **Death Spiral Detection** — execution_pattern_repo, anomaly detection

---

## [1.3.2] — AgentZero UI Migration — 2025-XX-XX

### Added
- AgentZero dark UI theme (indigo #4248f1)
- Toast notifications (success/error/warning/info)
- Message reactions (copy, edit, regenerate)
- Attachment support with file upload
- Speech input via Web Speech API
- T3 VM execution with VM pool
