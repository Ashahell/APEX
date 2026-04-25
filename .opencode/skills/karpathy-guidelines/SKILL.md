---
name: karpathy-guidelines
description: Load Karpathy-inspired coding guidelines to reduce common LLM mistakes. Apply these principles when writing or editing code: Think Before Coding, Simplicity First, Surgical Changes, Goal-Driven Execution.
license: MIT
compatibility: opencode
metadata:
  source: https://github.com/forrestchang/andrej-karpathy-skills
  category: coding-guidelines
  derived_from: Andrej Karpathy's observations on LLM coding pitfalls
---

# Karpathy-Inspired Coding Guidelines

These guidelines reduce common LLM coding mistakes. Apply them to every coding task.

**Tradeoff:** Bias toward caution over speed. For trivial tasks (typos, obvious one-liners), use judgment.

## The Four Principles

### 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

### 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

## Signs These Guidelines Are Working

These guidelines are working if you see:
- **Fewer unnecessary changes in diffs** — Only requested changes appear
- **Fewer rewrites due to overcomplication** — Code is simple the first time
- **Clarifying questions come before implementation** — Not after mistakes
- **Clean, minimal PRs** — No drive-by refactoring or "improvements"

## When to Apply

Apply these guidelines when:
- Writing new code or modules
- Editing existing code
- Proposing solutions or approaches
- Creating tests
- Reviewing changes

**Skip for trivial tasks:** typo fixes, obvious one-liners, simple refactoring that doesn't change behavior.
