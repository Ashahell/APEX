---
name: karpathy-llm-wiki
description: Load Karpathy's LLM Wiki pattern for maintaining persistent project knowledge. Applies the pattern: raw sources → LLM-maintained wiki → schema layer in CLAUDE.md.
license: MIT
compatibility: opencode
metadata:
  source: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
  category: knowledge-management
---

# LLM Wiki Pattern

A pattern for building and maintaining persistent project knowledge using LLMs.

## Core Concept

Instead of just retrieving from raw documents at query time, the LLM **incrementally builds and maintains a persistent wiki** — a structured, interlinked collection of markdown files.

**Key difference**: The wiki is a persistent, compounding artifact. Cross-references are already there. Contradictions are flagged. Synthesis reflects everything you've read.

## Directory Structure

```
project/
├── llm-wiki/              # The persistent wiki
│   ├── raw/              # Immutable source documents
│   ├── index.md          # Content catalog (auto-updated)
│   ├── log.md           # Chronological activity log
│   └── *.md             # Wiki pages (entities, concepts, summaries)
├── CLAUDE.md             # Schema layer (this file)
└── workspace/           # Chorus artifacts
```

## Wiki Page Types

| Type | Purpose | Example |
|------|---------|---------|
| Entity pages | Single concepts/things | `api-design.md`, `auth-system.md` |
| Concept pages | Patterns/techniques | `error-handling.md`, `caching-strategy.md` |
| Summary pages | Source syntheses | `architecture-overview.md` |
| Comparison pages | Side-by-side analysis | `方案对比.md` |

## Operations

### Ingest Source

1. Drop source into `llm-wiki/raw/`
2. LLM reads source
3. Writes summary page to wiki
4. Updates `index.md`
5. Updates relevant entity/concept pages
6. Appends entry to `log.md`

### Query Wiki

1. LLM reads `index.md` for relevant pages
2. Drills into relevant pages
3. Synthesizes answer with citations
4. Optionally files answer back into wiki as new page

### Lint Wiki

Periodically ask LLM to:
- Check for contradictions between pages
- Find stale claims superseded by new sources
- Identify orphan pages with no inbound links
- Flag missing cross-references

## Index Format

```markdown
# Index

## Entities
- [auth-system.md](auth-system.md) - User authentication and session management

## Concepts  
- [error-handling.md](error-handling.md) - Patterns for robust error handling

## Summaries
- [architecture-overview.md](architecture-overview.md) - System architecture synthesis

## Sources
- [raw/api-spec.md](raw/api-spec.md) - Original API specification
```

## Log Format

```markdown
# Log

## [2026-04-12] ingest | Architecture Decision Record
## [2026-04-11] query | How does caching work?
## [2026-04-10] lint | Quarterly health check
## [2026-04-09] ingest | Performance benchmarks
```

## Schema Integration

This CLAUDE.md serves as the schema layer. It tells the LLM:
- How the wiki is structured
- What conventions to follow
- What workflows to execute

Update this file as you discover what works for your project.

## Tips

- Keep raw sources immutable (LLM reads but never modifies)
- Wiki pages are owned by LLM (you read, LLM writes)
- Start with index.md + log.md, add page types as needed
- Use Obsidian's graph view to visualize connections
- Commit wiki to git for version history

## Why This Works

The tedious part of maintaining knowledge is bookkeeping — updating cross-references, keeping summaries current, noting contradictions. LLMs don't get bored or forget. The wiki stays maintained because the cost is near zero.

Human job: curate sources, direct analysis, ask good questions.
LLM job: everything else.

---

**Derived from**: [Karpathy's LLM Wiki gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
