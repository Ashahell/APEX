# APEX Local Development Plan for Parity

## Overview

This document provides a step-by-step executable plan for developers to achieve parity with OpenClaw, Agent Zero, Hermes, and OpenFang. It covers local environment setup, verification steps, and acceptance criteria.

## Prerequisites

### System Requirements

- **Rust**: 1.93+ (stable)
- **Node.js**: 20+
- **Python**: 3.11+
- **Git**: Latest
- **Docker**: Optional (for VM-based isolation)

### Environment Setup

```bash
# Install Rust
rustup install stable

# Install Node.js (via nvm or nodejs.org)
# Install Python 3.11+

# Install Poetry for Python environments
python -m pip install --user poetry
```

### Environment Variables

| Variable | Purpose | Default |
|----------|---------|---------|
| `APEX_SHARED_SECRET` | HMAC signing secret | dev-secret-change-in-production |
| `APEX_AUTH_DISABLED` | Disable auth for dev | (not set) |
| `APEX_USE_LLM` | Enable LLM | false |
| `APEX_BASE_URL` | Base URL for streaming | http://localhost:3000 |

## Phase 0: Stabilize Foundation

### Tasks

1. **Verify streaming_sign.rs compiles**
   - File: `core/router/src/streaming_sign.rs`
   - Run: `cargo check -p apex-router`

2. **Verify router wiring**
   - File: `core/router/src/api/mod.rs`
   - Check: `.merge(crate::streaming_sign::create_stream_sign_router())`

3. **Verify module exposure**
   - File: `core/router/src/lib.rs`
   - Check: `pub mod streaming_sign;`

4. **Run Rust tests**
   ```bash
   cargo test --lib
   ```

### Acceptance Criteria

- [ ] `cargo check` passes with no errors
- [ ] All unit tests pass
- [ ] Streaming sign endpoint returns valid JSON

### Verification Commands

```bash
# Backend build
cargo check -p apex-router

# Run tests
cargo test --lib

# Verify streaming sign endpoint
curl -s "http://localhost:3000/api/v1/streams/sign?path=/stream/stats"
```

---

## Phase 1: UI Wiring and Accessibility

### Tasks

1. **Verify UI route**
   - File: `ui/src/App.tsx`
   - Check: Lazy loading for StreamingDashboard

2. **Verify sidebar navigation**
   - File: `ui/src/components/ui/Sidebar.tsx`
   - Check: 'streaming' in AppTab type and SIDEBAR_GROUPS

3. **Verify StreamingDashboard**
   - File: `ui/src/components/streaming/StreamingDashboard.tsx`
   - Check: Task selector, tabs (stats, hands, mcp, task)

4. **Verify useStreaming hook**
   - File: `ui/src/hooks/useStreaming.ts`
   - Check: Calls `/api/v1/streams/sign` for signed URLs

5. **Verify accessibility**
   - Check: ARIA labels on panels
   - Check: Keyboard navigation works

### Acceptance Criteria

- [ ] UI builds successfully
- [ ] `/streaming` route loads in dev
- [ ] All four panels render with real data
- [ ] Task selector dropdown functions
- [ ] Accessibility: ARIA labels present on all interactive elements

### Verification Commands

```bash
# Build UI
cd ui && npm run build

# Start UI dev server
cd ui && npm run start

# Navigate to http://localhost:8083/streaming
```

---

## Phase 2: Telemetry and Observability

### Tasks

1. **Verify metrics endpoint**
   - File: `core/router/src/api/system.rs`
   - Check: Streaming metrics in `/api/v1/metrics`

2. **Check metrics structure**
   - Active connections
   - Total connections
   - Event counts (thought, tool_call, tool_progress, tool_result, approval_needed, error, complete)
   - Error counts (auth, replay, internal)

3. **Verify SLOs documented**
   - File: `docs/STREAMING_ROLLOUT.md`
   - Check: Availability, latency, error rate targets

### Acceptance Criteria

- [ ] `/api/v1/metrics` returns streaming data
- [ ] Metrics are properly structured
- [ ] SLOs documented with target values

### Verification Commands

```bash
# Query metrics endpoint
curl -s http://localhost:3000/api/v1/metrics | jq '.streaming'

# Expected structure:
# {
#   "active_connections": 0,
#   "total_connections": 0,
#   "events": { ... },
#   "errors": { ... }
# }
```

---

## Phase 3: Security Review and Testing

### Tasks

1. **Run security tests**
   ```bash
   cargo test --lib security
   ```

2. **Verify auth flow**
   - HMAC signature validation works
   - Timestamp replay protection active
   - Query-based auth (sig/ts) functional

3. **Expand test coverage**
   - Add streaming integration tests
   - Add UI E2E tests (Playwright/Cypress)

4. **Verify CI pipeline**
   - Node 24 compatibility
   - Lint passes
   - Format checks pass

### Acceptance Criteria

- [ ] Security tests pass
- [ ] Auth flow verified
- [ ] CI pipeline green

### Verification Commands

```bash
# Run security tests
cargo test --lib security

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

---

## Phase 4: Runbooks and Documentation

### Tasks

1. **Verify runbooks**
   - File: `docs/STREAMING_RUNBOOKS.md`
   - Check: Incident response procedures

2. **Verify rollout docs**
   - File: `docs/STREAMING_ROLLOUT.md`
   - Check: Rollback procedures

3. **Verify telemetry docs**
   - File: `docs/TELEMETRY_ROLLOUT.md`
   - Check: Metrics documentation

### Acceptance Criteria

- [ ] Runbooks are up to date
- [ ] Rollback procedures documented
- [ ] Metrics documented

---

## Quick Reference

### Key Files Touched

| Component | Files |
|-----------|-------|
| Backend | `core/router/src/streaming_sign.rs`, `core/router/src/api/mod.rs`, `core/router/src/api/system.rs` |
| UI | `ui/src/hooks/useStreaming.ts`, `ui/src/components/streaming/StreamingDashboard.tsx`, `ui/src/App.tsx`, `ui/src/components/ui/Sidebar.tsx` |
| Docs | `docs/STREAMING_ROLLOUT.md`, `docs/TELEMETRY_ROLLOUT.md`, `docs/STREAMING_RUNBOOKS.md` |

### Common Commands

```bash
# Backend
cargo check -p apex-router
cargo test --lib
cargo fmt
cargo clippy -- -D warnings

# Frontend
cd ui && npm install
cd ui && npm run build
cd ui && npm run start

# Verify endpoints
curl -s http://localhost:3000/api/v1/metrics
curl -s "http://localhost:3000/api/v1/streams/sign?path=/stream/stats"
curl -s http://localhost:3000/stream/stats
```

### Start Services

```powershell
# Windows
.\apex.bat start

# Or manually:
# Terminal 1: llama-server (optional)
# Terminal 2: cargo run --release --bin apex-router
# Terminal 3: cd ui && npm run start
```
