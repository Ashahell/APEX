# APEX - Autonomous Agent Platform

> ⚠️ **Status: PRE-ALPHA** - This is an experimental research project. Not production ready.

APEX combines the **best of OpenClaw and AgentZero** with **significantly stronger security**. A single-user autonomous agent platform with messaging interfaces and secure code execution.

## Vision

| Reference | What We Take |
|-----------|-------------|
| **OpenClaw** | Open architecture, extensibility, community-driven plugin ecosystem, messaging adapters |
| **AgentZero** | Dark navy/cyan aesthetic, polished UI, smooth UX patterns, agent loop logic |
| **Hermes** | Bounded memory, auto-created skills, session search, user profile |
| **Security-first** | Hardened beyond both — T0-T3 permission tiers, HMAC auth, TOTP verification, input sanitization, connection pooling |

APEX is **more secure than both** by design:
- Single-user architecture (no multi-tenancy attack surface)
- Hardened MCP with connection pooling and input validation
- Firecracker/gVisor isolation for code execution
- Audit trail with decision journal and reflection tracking

## 🚨 Pre-Alpha Warnings

- **Security-first but unaudited** — Built with strong security principles, but not formally audited yet
- **Limited testing** — Many features are proof-of-concept
- **API instability** — Breaking changes expected
- **No production support** — Use at your own risk
- **Execution isolation** — Firecracker/gVisor/Docker/Mock backends available

## Status

**Version**: v1.6.0 (Pre-Alpha) - Sapphire Features Complete

## Architecture

6-layer system (Proof-of-Concept):
- **L1**: TypeScript Gateway (REST API adapters)
- **L2**: Rust Router (Task routing & classification)
- **L3**: Rust Memory Service (SQLite persistence)
- **L4**: TypeScript Skills Framework
- **L5**: Python Execution Engine (Docker)
- **L6**: React UI

## Features Implemented (Experimental)

### Core (POC)
- Task routing with automatic tier classification (Instant/Shallow/Deep)
- HMAC authentication between components (basic)
- SQLite database with migrations (22+ migrations)
- WebSocket real-time updates (basic)
- Execution streaming with consequence preview (POC)

### Security Features
- **Encrypted Secret Storage** - AES-256-GCM encrypted key-value store
- **Enhanced Rate Limiting** - Per-endpoint with progressive throttling
- **TOTP Persistence** - Time-based OTP with encrypted storage
- **Input Validation** - MCP sanitization tests (31 tests)
- **Audit Chain** - Hash chain verification tests (12 tests)
- **Permission Tiers** - T0-T3 enforcement tests (14 tests)
- **Sandbox Security** - Python sandbox with import allowlist (33 tests)
- **Anomaly Detection** - Runtime behavior monitoring

### Skills System (34 skills) - Experimental
- **T0 (Read-only)**: code.review, repo.search, deps.check, file.search, docs.read
- **T1 (Tap confirm)**: file.delete, git.commit, code.generate, code.format, code.test, code.refactor, code.document, api.test, api.design, db.migrate, db.schema, copy.generate, script.draft, script.outline, seo.optimize, music.extend, music.generate, music.remix, video.edit, video.generate
- **T2 (Type confirm)**: db.drop, deploy.kubectl, docker.build, docker.run, git.branch, git.force_push, aws.lambda, ci.configure
- **T3 (TOTP)**: shell.execute

### OpenClaw Integration Features (v1.4.0)

| Feature | Description |
|---------|-------------|
| **Control UI Dashboard** | DashboardLayout, PinnedMessages, SessionManager, CommandPalette |
| **Fast Mode & Provider Plugins** | provider_repo, FastModeToggle, ModelPicker with fallback chains |
| **sessions_yield & sessions_resume** | Session checkpointing, yield/resume control flow |
| **PDF Tool** | pdf_repo, PDF upload, text extraction, analysis with LLM |
| **Multimodal Memory** | Image/audio embeddings with Gemini, hybrid search |
| **Additional Channels** | 10+ new messaging adapters (Signal, IRC, Matrix, Teams, etc.) |
| **Secrets Expansion** | 64 predefined secret targets, rotation logs, access audit |
| **Slack Block Kit** | Rich Slack messages, 6 pre-built templates, variable interpolation |

### Hermes Agent Integration Features (v1.5.0)

Inspired by NousResearch's Hermes Agent architecture.

| Feature | Description |
|---------|-------------|
| **Bounded Curated Memory** | Character-limited stores (2,200 agent / 1,375 user chars) with automatic consolidation |
| **Agent-Managed Skills** | Auto-create skills after 5+ tool calls, SKILL.md format with YAML frontmatter |
| **Skills Hub** | Trust levels (Verified > Trusted > Community), marketplace integration |
| **Session Search** | FTS5 with LIKE fallback, BM25 ranking, context extraction |
| **User Profile** | Communication styles, verbosity levels, response format preferences |

### Advanced Features (Research/POC)
- **SOUL.md Identity** - Agent reads identity file on wake (POC)
- **Heartbeat Daemon** - Autonomous wake cycles (experimental)
- **Narrative Memory** - Tracks agent decisions (basic)
- **Moltbook Social** - Federated agent network integration (POC)
- **Governance Engine** - Constitution enforcement (basic)
- **Execution Streaming** - Real-time thought/action/observation to UI (POC)
- **Dynamic Tool Generation** - ✅ LLM generates custom Python tools with secure sandbox execution
- **Subagent Pool** - Parallel task splitting (POC)

### UI Features (AgentZero Theme)
- Real-time chat with task sidebar
- **AgentZero Styling**: Indigo (#4248f1) primary, CSS variables, SVG icons
- **Message Reactions**: Copy, edit, regenerate on hover
- **Toast Notifications**: success/error/warning/info with auto-dismiss
- **Attachment Support**: File upload with preview
- **Speech Input**: Voice recording via Web Speech API
- **Enhanced Welcome Screen**: Quick action cards
- Kanban board for task management
- Workflow visualizer (basic)
- Memory/narrative viewer (basic)
- Skills marketplace
- System health monitoring
- Quick command bar (Ctrl+P)
- Skill quick launcher (Ctrl+K)

### Database Migrations (22+)

| # | Migration | Tables |
|---|-----------|--------|
| 015 | control_ui | dashboard_layout, pinned_messages, chat_bookmarks, session_metadata |
| 016 | fast_mode_providers | provider_plugins, session_fast_mode, model_fallbacks |
| 017 | subagent_control | session_yield_log, session_resume_history, session_attachments |
| 018 | pdf_tool | pdf_documents, pdf_extraction_jobs |
| 019 | multimodal_memory | memory_embeddings, memory_indexing_jobs, memory_multimodal_config |
| 020 | messaging_channels | channel_settings, channel_templates, channel_webhooks |
| 021 | secrets_expansion | secret_refs, secret_rotation_log, secret_access_log |

## Tech Stack (Development)

- **Backend**: Rust (Axum, Tokio, Sqlx)
- **Frontend**: React 18, TypeScript, Tailwind CSS, Zustand
- **Database**: SQLite
- **LLM**: llama.cpp (Qwen3-4B) - requires local setup (disabled in development mode by default)

## Getting Started (Development)

> **Development Mode**: By default, APEX runs with local LLM disabled to avoid unnecessary LLM usage during development. Enable LLM via Settings → LLM in the UI, or set `APEX_USE_LLM=1` when testing LLM features.

```powershell
# Clone and setup
cd apex

# Install dependencies
cargo install  # Rust
cd ui && pnpm install  # Frontend

# Start all services (with LLM + Docker)
.\apex.bat start

# Start all services INCLUDING embedding server (for memory search)
.\apex.bat start-full

# Or with different isolation backends:
.\apex.bat router-docker       # Docker container (default)
.\apex.bat router-gvisor       # gVisor sandbox (Linux only)
.\apex.bat router-firecracker # Firecracker VM (Linux only)  
.\apex.bat router-mock        # No real execution

# Individual services:
.\apex.bat llama    # LLM server (port 8080)
.\apex.bat embed   # Embedding server (port 8081)
.\apex.bat router  # Router

# Or manually:
# Terminal 1: llama-server (local LLM)
# Terminal 2: cargo run --release --bin apex-router
# Terminal 3: cd ui && pnpm dev
```

## API

See `docs/ARCHITECTURE.md` for API documentation (subject to change).

### Secrets API (v1.4.0)

```bash
# List all secrets
GET /api/v1/secrets

# Get secret by ID
GET /api/v1/secrets/:id

# List categories
GET /api/v1/secrets/categories

# Get secrets by category
GET /api/v1/secrets/category/:category

# Get rotation history
GET /api/v1/secrets/rotation/:secret_name

# Get recent rotations
GET /api/v1/secrets/rotation/recent

# Get access history
GET /api/v1/secrets/access/:secret_ref_id

# Get recent accesses
GET /api/v1/secrets/access/recent

# Get failed accesses
GET /api/v1/secrets/access/failed
```

## Roadmap

- [x] Formal security audit (Phases 1-2 complete)
- [x] Dynamic tool generation (POC)
- [x] Subagent pool (POC)
- [x] Comprehensive testing (388+ tests)
- [x] Task classification rules (Instant/Shallow/Deep tiers)
- [x] Capability enforcement (fail-closed for unknown skills)
- [x] Gateway optional (auth can be disabled)
- [x] MCP marketplace UI (templates, install modal, registries)
- [x] Production hardening (seccomp, AppArmor, SIEM docs)
- [x] SystemComponent trait (unified lifecycle management)
- [x] OpenClaw Features (v1.4.0) - Dashboard, Fast Mode, Sessions, PDF, Multimodal, Channels, Secrets
- [x] Hermes Agent Integration (v1.5.0) - Bounded Memory, Auto-Skills, Hub, Session Search, User Profile

## License

MIT
