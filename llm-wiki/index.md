# Index

## Schema
- [CLAUDE.md](CLAUDE.md) - Project schema layer

## Entities
- [project-overview.md](project-overview.md) - Project architecture, features, version history
- [architecture.md](concepts/architecture.md) - 6-layer architecture, security model, API surface
- [skills.md](concepts/skills.md) - Skill registry, SKILL.md standard, Hermes features

## Concepts
- [architecture.md](concepts/architecture.md) - 6-layer system, security model
- [skills.md](concepts/skills.md) - Skill registry, SKILL.md standard
- [security.md](concepts/security.md) - T0-T3 tiers, HMAC, TOTP, VM isolation, SSRF, schema fixes
- [future-work.md](concepts/future-work.md) - 4-phase roadmap (Phase 1 of 4 complete)
- [phase-1-ux-improvements.md](concepts/phase-1-ux-improvements.md) - Stop-button, lexical matching, compaction
- [chat-compaction.md](concepts/chat-compaction.md) - Context window reduction service
- [session-control.md](concepts/session-control.md) - Yield, resume, compact for multi-turn sessions
- [streaming.md](concepts/streaming.md) - SSE, WebSocket, TinySSE real-time updates
- [test-suite.md](concepts/test-suite.md) - 461+ tests across all layers
- [ssrf-protection.md](concepts/ssrf-protection.md) - SSRF/CVE-2026-4308 mitigation
- [schema-audit.md](concepts/schema-audit.md) - Database schema audit (critical findings)
- [schema-fix-plan.md](concepts/schema-fix-plan.md) - Security fixes: audit chain, key derivation, FK enforcement

## Phases
- [phase-1-ux-improvements.md](concepts/phase-1-ux-improvements.md) - Phase 1 (UX, complete)
- [future-work.md](concepts/future-work.md) - Phase 2-4 roadmap

## Raw Sources
_Immutable source documents (82 files in `llm-wiki/raw/`)_
- Design docs, migration docs, integration plans, security plans

## Recent Activity
See [log.md](log.md)