# APEX Test Suite

**Status:** Comprehensive
**Date:** 2026-04-25

## Overview

461+ tests across all layers. Rust tests dominate with unit, integration, and e2e categories.

## Test Breakdown

### Rust Core Tests (`core/`)

| Suite | Tests | Run Command |
|-------|-------|-------------|
| **Unit tests (apex-memory)** | 63+ | `cargo test --lib` |
| **Unit tests (apex-router)** | 313+ | `cargo test --lib` |
| **Unit tests (apex-security)** | 6 | `cargo test --lib` |
| **Integration (auth)** | — | `cargo test --test auth_integration` |
| **Integration (memory)** | — | `cargo test --test memory_integration` |
| **Integration (registry_db)** | — | `cargo test --test registry_db_tests` |
| **Integration (skills)** | 8 | `cargo test --test skills_integration` |
| **Integration (streaming)** | — | `cargo test --test streaming_integration` |
| **Integration (validation)** | 5 | `cargo test --test validation_tests` |
| **E2E tests** | 2 (#[ignore]) | `cargo test --test e2e` |

### Python Tests (`execution/`)

| Suite | Tests | Run Command |
|-------|-------|-------------|
| **All tests** | 53 | `cd execution && poetry run pytest` |
| **SSRF protection** | 12 | Part of execution tests |

### TypeScript Tests (`gateway/`, `skills/`)

| Suite | Tests | Run Command |
|-------|-------|-------------|
| **Gateway tests** | 8 | `cd gateway && pnpm test` |
| **Skills tests** | 8 | `cd skills && pnpm test` |

### UI Tests (`ui/`)

| Suite | Tests | Run Command |
|-------|-------|-------------|
| **Component tests** | 20 | `cd ui && pnpm test` |

## Key Test Areas

### Security Tests
- 57 tests: input validation, audit chain, permission tiers
- 33 sandbox security tests: import blocking, timeout, dangerous patterns

### Compaction Tests (Phase 1.9)
- 5 tests: `should_compact`, `disabled`, `summary_generation`, `estimate_tokens`, `preserves_recent`

### Lexical Matching Tests
- 6 unit tests: scoring algorithm

### Memory Tests
- 63+ tests: db operations, vector search, FTS5

### Integration Tests
- API endpoint testing with in-memory SQLite
- Worker execution flow
- WebSocket events

### E2E Tests (marked #[ignore])
- Full task lifecycle
- WebSocket reconnection
- Multi-step task execution

## Running Tests

```bash
# All Rust tests
cd core && cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration

# E2E tests (requires running router)
cargo test --test e2e

# Specific test
cargo test test_name

# TypeScript
cd gateway && pnpm test
cd skills && pnpm test

# UI
cd ui && pnpm test
```

## Related

- [project-overview.md](project-overview.md)
- [concepts/security.md](concepts/security.md)