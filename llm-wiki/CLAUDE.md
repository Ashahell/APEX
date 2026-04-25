# CLAUDE.md — APEX Project Schema

This file serves as the schema layer for the LLM Wiki, telling the LLM how project knowledge is structured.

## Project Structure

```
APEX/
├── AGENTS.md              # Primary knowledge base
├── llm-wiki/              # LLM-maintained persistent wiki
│   ├── CLAUDE.md          # This file (schema layer)
│   ├── index.md           # Content catalog
│   ├── log.md             # Chronological activity log
│   ├── raw/               # Immutable source documents
│   ├── concepts/          # Patterns and techniques
│   ├── phases/            # Version phase documentation
│   └── api/               # API documentation
├── graphify-out/          # Knowledge graph outputs
│   ├── graph.json         # Raw graph data
│   ├── graph.html         # Interactive visualization
│   └── GRAPH_REPORT.md    # Graph audit report
├── .opencode/             # OpenCode configuration
│   └── skills/            # Available skills
├── core/                  # Rust core (L2/L3)
│   ├── router/            # Task router (HTTP API)
│   ├── memory/            # Memory service (SQLite)
│   └── security/          # Capability tokens
├── gateway/               # TypeScript gateway (L1)
├── skills/                # TypeScript skills (L4)
├── execution/              # Python execution engine (L5)
└── ui/                    # React UI (L6)
```

## Architecture Overview

APEX is a 6-layer autonomous agent platform:
- **L1 Gateway**: TypeScript messaging adapters (REST, Slack, Discord, Telegram)
- **L2 Router**: Task routing and API (Rust)
- **L3 Memory**: Persistent memory with SQLite
- **L4 Skills**: TypeScript skill framework
- **L5 Execution**: Python agent engine
- **L6 UI**: React frontend

## Wiki Conventions

### Page Types
- **Phase pages**: `phase-X-name.md` - Implementation phase documentation
- **Concept pages**: `concepts/*.md` - Development patterns and techniques
- **API pages**: `api/*.md` - API documentation

### Entry Format
```markdown
# Page Title

**Status:** Active/In Progress/Complete
**Date:** YYYY-MM-DD

## Overview
## Key Components
## Implementation Details
## Files
## Next Steps
```

### Log Format
```markdown
## [YYYY-MM-DD] action | Description
- [Page] created/updated/ingested
```

## Key Patterns

### Security Model
- **T0**: Read-only (no confirmation)
- **T1**: Tap to confirm
- **T2**: Type to confirm
- **T3**: TOTP verification required

### Skill System
- Skills follow standard interface: `package.json` + `src/index.ts`
- Typed by permission tier (T0-T3)
- Auto-created skills after 5+ tool calls

### Hermes Integration
- Bounded memory: 2,200 chars (agent) / 1,375 chars (user)
- Auto-created skills via SKILL.md format
- Session search with FTS5

## Source Documents

Raw sources go in `llm-wiki/raw/`:
- API specifications
- Design documents
- Research papers