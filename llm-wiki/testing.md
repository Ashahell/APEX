# Testing

## Overview

APEX maintains comprehensive test coverage across all layers with 583+ tests.

## Test Suite

| Component | Tests | Location |
|-----------|-------|----------|
| Rust Unit | 348 | core/*/src/*_test.rs |
| Rust Integration | 59 + 40 (security) + 16 (streaming) + 9 (telemetry) + 10 (auth) | core/router/tests/ |
| Python | 53 | execution/tests/ |
| Gateway | 8 | gateway/src/*.test.ts |
| Skills | 8 | skills/src/*.test.ts |
| UI | 20 | ui/src/**/*.test.tsx |
| **Total** | **583+** | |

## Test Categories

### Unit Tests
- Individual function testing
- Module isolation
- Mock external dependencies

### Integration Tests
- API endpoint testing
- Database operations
- Service communication

### E2E Tests
- Spawn router binary
- Real HTTP requests
- Marked `#[ignore]` - run with `-- --ignored`

## Running Tests

### Rust
```bash
# All tests
cd core && cargo test

# Unit only
cargo test --lib

# Integration only
cargo test --test integration

# E2E (slow)
cargo test --test e2e
```

### TypeScript
```bash
cd gateway && pnpm test
cd skills && pnpm test
cd ui && pnpm test
```

### Python
```bash
cd execution && poetry run pytest
```

## CI Integration

### GitHub Actions
- Rust: `.github/workflows/rust.yml`
- TypeScript: `.github/workflows/typescript.yml`
- Python: `.github/workflows/python.yml`
- UI: `.github/workflows/ui.yml`
- Security: `.github/workflows/ci.yml` (cargo audit, npm audit)

## Security Tests

### Injection Detection
- 40+ tests covering prompt injection, SQL injection, command injection, path traversal, XSS

### Replay Protection
- 9 tests covering duplicate rejection, distinct signatures, capacity limits

### Config Validation
- 12 tests covering all config sections, defaults, edge cases