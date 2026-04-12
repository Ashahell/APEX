# Skills Framework

## Overview

APEX implements a skill-based plugin system with 34 built-in skills and auto-creation capabilities.

## Skill Structure

```
skill-name/
├── package.json
└── src/index.ts    # Exports: name, version, tier, inputSchema, outputSchema, execute(), healthCheck()
```

## Permission Tiers

| Tier | Count | Confirmation |
|------|-------|--------------|
| T0 | 3 | None (read-only) |
| T1 | 11 | Tap |
| T2 | 8 | Type action |
| T3 | 1 | TOTP (shell.execute) |

## Built-in Skills

### Code
- code.generate, code.review, code.format, code.refactor, code.document, code.test

### DevOps
- docker.build, docker.run, ci.configure, deploy.kubectl

### Data
- db.schema, db.migrate, db.drop

### Git
- git.commit, git.branch, git.force_push

### File
- file.search, file.delete

### Other
- shell.execute (T3), docs.read, deps.check, seo.optimize, script.draft, script.outline, copy.generate

## Auto-Created Skills

Generated after complex tasks (5+ tool calls) in SKILL.md format with YAML frontmatter.

## Skills Hub

- Trust levels: Verified > Trusted > Community
- Plugin signing with ed25519
- Marketplace with search capabilities