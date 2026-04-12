# APEX Architecture Overview

## System Overview

APEX is a 6-layer autonomous agent platform combining OpenClaw, AgentZero, Hermes, and OpenFang with security-first design.

## Layer Architecture

| Layer | Technology | Purpose |
|-------|-----------|---------|
| L1 | TypeScript (Fastify) | Gateway with HMAC-signed requests |
| L2 | Rust (Axum, Tokio) | Task routing, classification, agent loop |
| L3 | Rust (SQLite, sqlx) | Memory service, bounded memory, search |
| L4 | TypeScript | Skills framework, MCP client/server |
| L5 | Python (Docker) | Secure execution engine with sandboxing |
| L6 | React 18 + TypeScript | UI with 4 themes, real-time streaming |

## Key Components

- **Task Router**: Automatic tier classification (Instant/Shallow/Deep)
- **Agent Loop**: Plan/act/observe cycle with deep task worker
- **Streaming**: TinySSE-based real-time events
- **Memory**: Bounded memory, semantic search, TTL semantics
- **Security**: HMAC auth, TOTP verification, T0-T3 permission tiers

## Technology Stack

- Backend: Rust 1.93+ (Axum, Tokio, sqlx)
- Frontend: React 18, TypeScript, Tailwind CSS, Zustand
- Database: SQLite (with FTS5 for search)
- Execution: Docker / Firecracker / gVisor / Mock

## Version

- Current: v2.0.0 (Parity Complete)
- Parity Score: 9.45/10