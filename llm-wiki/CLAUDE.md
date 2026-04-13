# CLAUDE.md - APEX LLM Wiki Schema

> **Schema Version:** 1.0  
> **Last Updated:** 2026-04-13  
> **Owner:** APEX Development Team  

## Overview

This file serves as the schema layer for the APEX LLM Wiki. It tells LLMs how the wiki is structured, what conventions to follow, and what workflows to execute.

## Directory Structure

```
apex/
├── llm-wiki/              # The persistent wiki
│   ├── raw/              # Immutable source documents (copied from docs/)
│   ├── index.md          # Content catalog (auto-updated)
│   ├── log.md           # Chronological activity log
│   ├── entities/        # Entity pages (concepts/things)
│   ├── concepts/        # Concept pages (patterns/techniques)
│   ├── summaries/       # Summary pages (source syntheses)
│   └── comparisons/     # Comparison pages
├── CLAUDE.md             # Schema layer (this file)
└── docs/                # Original documentation
```

## Key Concepts

### Architecture (L1-L6)

| Layer | Name | Description |
|-------|------|-------------|
| L1 | Gateway | TypeScript HTTP adapter with HMAC signing |
| L2 | Router | Rust task router with classification |
| L3 | Memory | SQLite-based memory service |
| L4 | Skills | TypeScript skill framework |
| L5 | Execution | Python execution engine (Docker/sandbox) |
| L6 | UI | React UI with streaming |

### Permission Tiers (Security)

- **T0**: Read-only - no confirmation needed
- **T1**: File writes - tap to confirm
- **T2**: External APIs - type to confirm
- **T3**: Destructive - TOTP verification required

### Core Modules

- `apex-router` - Task routing and execution
- `apex-memory` - Memory and persistence
- `apex-security` - Security utilities
- `skills` - TypeScript skill framework
- `ui` - React UI

## Wiki Page Types

| Type | Purpose | Examples |
|------|---------|----------|
| Entity | Single concepts/things | `auth-system.md`, `task-router.md` |
| Concept | Patterns/techniques | `error-handling.md`, `caching-strategy.md` |
| Summary | Source syntheses | `streaming-overview.md` |
| Comparison | Side-by-side | `llm-providers.md` |

## Conventions

### Naming
- Use `snake_case` for file names
- Use `Title Case` for headers
- Frontmatter in YAML format

### Cross-References
```markdown
See [auth-system.md] for authentication details.
```

### Version Format
```markdown
> **Version:** v1.8.0 (Parent Improvements Release)
```

## Workflows

### Query Wiki
1. Read `index.md` for relevant pages
2. Drill into relevant entity/concept pages
3. Synthesize answer with citations

### Update Wiki
1. Drop source into `raw/`
2. Write summary page to appropriate directory
3. Update `index.md`
4. Append entry to `log.md`

### Lint Wiki
Periodically check for:
- Contradictions between pages
- Stale claims superseded by new sources
- Orphan pages with no inbound links
- Missing cross-references

## Important Files

### Documentation
- `AGENTS.md` - Main development guide
- `docs/PARENT_IMPROVEMENTS_PLAN.md` - Implementation plan
- `docs/APEX-Design.md` - System design

### API Endpoints
- Tasks: `/api/v1/tasks`
- Skills: `/api/v1/skills`
- Memory: `/api/v1/memory`
- Streaming: `/api/v1/stream/*`

### Configuration
- Environment variables in `core/router/src/unified_config.rs`
- AppConfig::global() for all settings

## Test Suite

| Component | Tests | Location |
|-----------|-------|----------|
| Rust unit | 336+ | `core/*/src/` |
| Integration | 59+ | `core/router/tests/` |
| Python | 53+ | `execution/tests/` |
| TypeScript | 16+ | `gateway/`, `skills/` |
| UI | 20+ | `ui/src/` |

## Tips

- Keep raw sources immutable (never modify raw docs)
- Wiki pages are owned by LLM
- Start with index.md + log.md, add page types as needed
- Commit wiki to git for version history

---

**Derived from**: [Karpathy's LLM Wiki gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
**APEX Integration**: 2026-04-13