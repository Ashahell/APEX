# APEX Project Overview

**Status:** Active
**Date:** 2026-04-25
**Version:** v1.6.0 (Sapphire Features) — Pre-Alpha ⚠️

## Overview

APEX combines OpenClaw and AgentZero with security-first design. A single-user autonomous agent platform with messaging interfaces and secure code execution.

> ⚠️ **WARNING: PRE-ALPHA** - This is an experimental research project. Not production ready.

## Architecture

| Layer | Component | Status | Location |
|-------|-----------|--------|----------|
| L1 Gateway | REST, Slack, Discord, Telegram | Built | `gateway/` |
| L2 Router | Task routing, HMAC auth | Built | `core/router/` |
| L3 Memory | SQLite + Vector search | Built | `core/memory/` |
| L4 Skills | 33 built-in + auto-created | Built | `skills/` |
| L5 Execution | Docker/Firecracker isolation | Built | `execution/` |
| L6 UI | React + WebSocket | Built | `ui/` |

## Security Features

- **HMAC request signing** — All API requests signed
- **TOTP verification** — T3 tasks require authenticator app
- **T0-T3 permission tiers** — Read → Tap → Type → TOTP
- **Execution isolation** — Docker/Firecracker/gVisor
- **SSRF protection** — Blocks localhost, private IPs, cloud metadata
- **Schema security** — Argon2id key derivation, FK enforcement, auto-encrypt

## Hermes Agent Integration (v1.5.0)

- **Bounded curated memory** — 2,200 chars (agent) / 1,375 chars (user)
- **Auto-created skills** — After 5+ tool calls via SKILL.md
- **Session search** — FTS5 + BM25 ranking
- **User profile** — Communication style, verbosity, response format

## Sapphire Features (v1.6.0)

- **Tool Maker runtime validation** — Import allowlist levels (Strict/Moderate/Permissive)
- **Persona assembly** — Prompt + voice + tools + model bundles
- **Context scope isolation** — Per-conversation data separation
- **Continuity scheduler** — Enhanced heartbeat/cron tasks
- **Plugin signing** — Ed25519 verification
- **Privacy toggle** — One-click cloud block
- **Story engine** — Interactive fiction as tasks

## OpenClaw Integration (v1.4.0)

- **Control UI** — Dashboard, PinnedMessages, SessionManager, CommandPalette
- **Fast Mode** — Priority queue with smaller model
- **Sessions Yield/Resume** — Pause and resume multi-turn conversations
- **PDF Tool** — Upload, view, analyze PDFs
- **Multimodal Memory** — Image/video memory
- **Death Spiral Detection** — Execution pattern anomalies
- **External Notifications** — Discord webhook + Telegram bot
- **64 Secret Targets** — Expanded secrets system
- **Slack Block Kit** — Structured Slack messages

## Test Suite

| Component | Tests | Location |
|-----------|-------|----------|
| Rust unit tests | 313+ | `core/*/src/*_test.rs` |
| Rust integration | 59 | `core/router/tests/` |
| Rust e2e | 2 (#[ignore]) | `core/router/tests/e2e.rs` |
| Python tests | 53 | `execution/tests/` |
| Gateway tests | 8 | `gateway/src/*.test.ts` |
| Skills tests | 8 | `skills/src/*.test.ts` |
| UI tests | 20 | `ui/src/**/*.test.tsx` |
| **Total** | **461+** | |

## Build Commands

```powershell
.\apex.bat start          # Start all services
.\apex.bat start-full     # Include embedding server
cargo test                # Run all Rust tests
cd ui && pnpm dev         # Run UI dev server
```

## Related

- [architecture.md](concepts/architecture.md)
- [skills.md](concepts/skills.md)
- [phase-1-ux-improvements.md](concepts/phase-1-ux-improvements.md)
- [concepts/future-work.md](concepts/future-work.md)
- [concepts/streaming.md](concepts/streaming.md)
- [concepts/security.md](concepts/security.md)
- [concepts/test-suite.md](concepts/test-suite.md)
- [Knowledge Graph](graphify-out/GRAPH_REPORT.md)