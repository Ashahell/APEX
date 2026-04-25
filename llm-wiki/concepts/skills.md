# Skills System

**Status:** Implemented (v1.6.0)
**Date:** 2026-04-25
**Source:** [AGENTS.md](raw/AGENTS.md)

## Overview

APEX implements a curated skill system (~33 built-in + unlimited auto-created) using the SKILL.md standard from Agent Zero.

## Permission Tiers

| Tier | Actions | Confirmation | Count |
|------|---------|--------------|-------|
| **T0** | Read-only queries, search | None | 3 skills |
| **T1** | File writes, drafts | Tap confirm | 11 skills |
| **T2** | External API calls, git push | Type confirm | 8 skills |
| **T3** | Destructive ops, cost >$10 | TOTP + 5min delay | 1 skill |

> Note: `shell.execute` was moved from T2 to T3 per security audit

## Built-in Skills (33 Total)

### Development
- `code.generate` - Generate code from description (T1)
- `code.review` - Review code for bugs/style (T0)
- `code.refactor` - Refactor code structure (T1)
- `code.document` - Generate documentation (T1)
- `code.test` - Generate and run tests (T2)
- `git.commit` - Stage, commit, push changes (T2)
- `repo.search` - Semantic code search (T0)
- `deps.check` - Check for vulnerabilities (T0)
- `shell.execute` - Run shell commands (T3)
- `docker.build` - Build container images (T2)
- `api.design` - Design API schemas (T1)
- `db.schema` - Design database schemas (T1)
- `db.migrate` - Generate migration scripts (T2)
- `ci.configure` - Generate CI/CD configs (T1)
- `docs.read` - Read and summarize docs (T0)

### Auto-created Skills
- Unbounded count
- Created after 5+ tool calls during complex tasks
- SKILL.md format with YAML frontmatter
- Security scanned before activation
- Stored in `~/.apex/skill_suggestions/`

## SKILL.md Standard

Each skill directory contains:
```
skill-name/
├── SKILL.md           # Manifest and behavior
├── package.json       # Metadata
├── src/index.ts       # Implementation
└── tests/            # Test suite
```

## Hermes Agent Features

- **Bounded Curated Memory**: 2,200 chars (agent) / 1,375 chars (user)
- **Auto-created Skills**: After 5+ tool calls
- **Session Search**: FTS5 + BM25 ranking
- **User Profile**: Communication style, verbosity, response format

## Skills Hub

Marketplace with trust levels:
- **Verified** > **Trusted** > **Community**
- Configurable request timeout
- Search and browse capabilities

## Files
- Full skill registry: [AGENTS.md](raw/AGENTS.md)
- Skill SDK: [docs/SKILL-SDK.md](raw/SKILL-SDK.md)
- Skill plan: [docs/SKILL.md](raw/SKILL.md)