# Index - APEX LLM Wiki

> **Last Updated:** 2026-04-13

---

## Entities (Core Concepts)

### Architecture (L1-L6)
- [architecture-overview.md](summaries/architecture-overview.md) - System architecture (L1-L6)
- [task-router.md](entities/task-router.md) - L2 Rust task router with classification
- [memory-service.md](entities/memory-service.md) - L3 SQLite-based memory
- [execution-engine.md](entities/execution-engine.md) - L5 Python/Docker execution

### Security
- [permission-tiers.md](entities/permission-tiers.md) - T0-T3 permission tier system
- [hmac-auth.md](entities/hmac-auth.md) - HMAC request authentication
- [secrets-management.md](entities/secrets-management.md) - 53 secrets targets

### Skills & Tools
- [skills-framework.md](entities/skills-framework.md) - TypeScript skill system (34 built-in)
- [skill-loader.md](entities/skill-loader.md) - Dynamic skill loading
- [skill-security.md](entities/skill-security.md) - Skill permission tiers
- [dynamic-tools.md](entities/dynamic-tools.md) - Dynamic tool generation

### Streaming & Events
- [streaming-system.md](entities/streaming-system.md) - SSE streaming
- [execution-stream.md](entities/execution-stream.md) - Execution event streaming
- [progress-reporting.md](entities/progress-reporting.md) - H2: Subagent progress events

### Storage & Caching
- [response-cache.md](entities/response-cache.md) - M1: Response caching layer
- [task-repository.md](entities/task-repository.md) - Task persistence with activity
- [inactivity-tracking.md](entities/inactivity-tracking.md) - M2: Inactivity-based timeouts

### UI & Components
- [ui-system.md](entities/ui-system.md) - React UI with streaming
- [model-picker.md](entities/model-picker.md) - L1: Interactive model picker

---

## Concepts (Patterns & Techniques)

- [api-patterns.md](concepts/api-patterns.md) - REST API conventions
- [error-handling.md](concepts/error-handling.md) - Error handling patterns
- [caching-strategy.md](concepts/caching-strategy.md) - Response caching
- [activity-tracking.md](concepts/activity-tracking.md) - Inactivity timeout implementation

---

## Summaries (Source Syntheses)

- [v1.8.0-release-notes.md](summaries/v1.8.0-release-notes.md) - Parent Improvements Release
- [parent-improvements.md](summaries/parent-improvements.md) - Implementation summary

---

## Comparisons & Analysis

- [llm-providers.md](comparisons/llm-providers.md) - LLM provider comparison
- [方案的对比.md](comparisons/方案的对比.md) - Implementation comparison

---

## v1.8.0 New Features (2026-04-13)

| Feature | File | Status |
|---------|------|--------|
| H2: Progress Reporting | progress-reporting.md | ✅ |
| M1: Response Caching | response-cache.md | ✅ |
| H3: Expanded Secrets | secrets-management.md | ✅ (53 targets) |
| M2: Inactivity Tracking | inactivity-tracking.md | ✅ |
| M3: Skills Search | skill-loader.md | ✅ |
| H4: Security Audit | security-audit.yml | ✅ (CI) |
| L2: Nix Flake | flake.nix | ✅ |
| UI: TS Fix | StreamingDashboard.tsx | ✅ |

---

## Sources (Original Documentation)

### Primary
- [raw/AGENTS.md](../AGENTS.md) - Main development guide (v1.8.0)
- [raw/PARENT_IMPROVEMENTS_PLAN.md](../docs/PARENT_IMPROVEMENTS_PLAN.md) - Implementation plan
- [raw/APEX-Design.md](../docs/APEX-Design.md) - System design

### API
- [raw/API.md](../docs/API.md) - API documentation
- [raw/API_SURFACE.md](../docs/API_SURFACE.md) - API surface

### Security
- [raw/APEX_Security_Implementation_Plan.md](../docs/APEX_Security_Implementation_Plan.md)

---

## Quick Links

- **Tests**: 336+ Rust, 53 Python, 16 TypeScript, 20 UI
- **Build**: `cargo build && pnpm build`
- **Run**: `./apex.bat start`

---

**To query the wiki:**
1. Read this index.md
2. Navigate to relevant entity/concept
3. Synthesize answer with citations

**To add new content:**
1. Save source to `raw/`
2. Write summary to appropriate directory
3. Update this index.md
4. Append to log.md