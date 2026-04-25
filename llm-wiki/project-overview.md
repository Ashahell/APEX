# APEX Project Overview

**Status:** Active
**Date:** 2026-04-25
**Version:** v1.6.0 (Sapphire Features)

## Overview

APEX combines OpenClaw and AgentZero with security-first design. A single-user autonomous agent platform with messaging interfaces and secure code execution.

## Architecture

| Layer | Component | Status |
|------|-----------|--------|
| L1 Gateway | REST, Slack, Discord, Telegram | Built |
| L2 Router | Task routing, HMAC auth | Built |
| L3 Memory | SQLite + Vector search | Built |
| L4 Skills | 33 built-in + auto-created | Built |
| L5 Execution | Docker/Firecracker isolation | Built |
| L6 UI | React + WebSocket | Built |

## Key Features

### Security
- HMAC request signing
- TOTP verification for T3 tasks
- T0-T3 permission tiers
- Execution isolation (Docker/Firecracker/gVisor)

### Hermes Agent Integration
- Bounded curated memory
- Auto-created skills
- Session search with FTS5
- User profile modeling

### Sapphire Features (v1.6.0)
- Tool Maker runtime validation
- Persona assembly
- Context scope isolation
- Continuity scheduler
- Plugin signing
- Privacy toggle
- Story engine

## Related
- [Knowledge Graph](graphify-out/GRAPH_REPORT.md)
- [Available Skills](.opencode/skills/)