# Phase 4 Runbook: MCP, Tool Ecosystem, and Governance Parity

## Overview
- Phase 4 closes MCP tooling, marketplace, and governance gaps; provides stable governance cadence.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 4 Features

| Feature | Description |
|---------|-------------|
| MCP Tool Discovery | `/api/v1/mcp/tools/discover` returns all tools with metadata |
| Server Health Monitoring | `/api/v1/mcp/servers/health` returns health scores |
| Tool Health Tracking | `/api/v1/mcp/tools/:tool_key/health` returns per-tool health |
| Marketplace Scaffolding | `/api/v1/mcp/marketplace` returns 5 sample tools |
| Governance Cadence | Documented policy change process, constitution workflow |

---

## Incident Response Procedures

### 1. MCP Server Disconnect

**Symptoms:**
- Server status shows "disconnected" or "error"
- Tools from server unavailable
- Health score drops to 0.0

**Immediate Steps:**
```bash
# Check server health
curl -s http://localhost:3000/api/v1/mcp/servers/health | jq .

# Check specific server
curl -s http://localhost:3000/api/v1/mcp/servers/<server-id> | jq .

# Attempt reconnection
curl -X POST http://localhost:3000/api/v1/mcp/servers/<server-id>/connect
```

**Debug Commands:**
```bash
# Check server logs
# Review: core/router/src/mcp/server_manager.rs

# Check tool availability
curl -s http://localhost:3000/api/v1/mcp/tools/discover | jq '.tools | length'
```

**Common Causes:**
- Server process crashed
- Network connectivity issues
- Configuration error

**Rollback:** Restart server, verify configuration.

---

### 2. Tool Execution Failure

**Symptoms:**
- Tool returns error response
- Health status shows "unhealthy"
- Error count increasing

**Immediate Steps:**
```bash
# Check tool health
curl -s http://localhost:3000/api/v1/mcp/tools/<tool-key>/health | jq .

# Check all tools
curl -s http://localhost:3000/api/v1/mcp/tools/discover | jq '.tools[] | {name, health}'
```

**Debug Commands:**
```bash
# Check tool execution logs
# Review: core/router/src/mcp/client.rs

# Test tool directly
curl -X POST http://localhost:3000/api/v1/mcp/servers/<server-id>/tools/<tool-name> \
  -H "Content-Type: application/json" \
  -d '{"arguments": {...}}'
```

**Common Causes:**
- Tool input validation failure
- Server-side error
- Timeout

**Rollback:** Disable tool, investigate root cause.

---

### 3. Marketplace Issues

**Symptoms:**
- Marketplace returns empty list
- Tool installation fails
- Rating/install count incorrect

**Immediate Steps:**
```bash
# Check marketplace
curl -s http://localhost:3000/api/v1/mcp/marketplace | jq '. | length'

# Check registry
curl -s http://localhost:3000/api/v1/mcp/registries | jq .
```

**Debug Commands:**
```bash
# Check registry tools
curl -s http://localhost:3000/api/v1/mcp/registries/<rid>/tools | jq .

# Discover tools
curl -X POST http://localhost:3000/api/v1/mcp/registries/<rid>/tools/discover
```

**Common Causes:**
- Registry schema not initialized
- No tools registered
- Database error

**Rollback:** Re-initialize registry, re-register tools.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `curl -s http://localhost:3000/api/v1/mcp/servers/health` | Server health overview |
| `curl -s http://localhost:3000/api/v1/mcp/tools/discover` | Tool discovery |
| `curl -s http://localhost:3000/api/v1/mcp/marketplace` | Marketplace listing |
| `curl -s http://localhost:3000/api/v1/mcp/tools/:key/health` | Tool health |
| `curl -s http://localhost:3000/api/v1/governance/immutable` | Audit trail |

---

## Test Commands

```bash
# Run all tests
cd core && cargo test

# Run MCP-specific tests
cd core && cargo test mcp
```

---

## Escalation Paths

| Issue | First Contact | Escalation |
|-------|--------------|------------|
| MCP server issues | @backend-team | @infra-team |
| Tool failures | @backend-team | @engineering-ops |
| Governance issues | @user | @governance-board |
| Marketplace issues | @frontend-team | @backend-team |

---

## Rollback Procedure

If Phase 4 changes cause critical issues:

1. **Disable MCP features:**
   ```bash
   # Stop MCP server connections
   # Remove marketplace tools
   ```

2. **Restart services:**
   ```bash
   cargo run --release --bin apex-router
   ```

3. **Verify recovery:**
   ```bash
   curl -s http://localhost:3000/api/v1/mcp/servers/health
   curl -s http://localhost:3000/api/v1/mcp/tools/discover
   ```

---

## Verification Checklist

After any incident, verify:

- [ ] MCP servers health endpoint returns data
- [ ] Tool discovery returns tools
- [ ] Marketplace returns sample tools
- [ ] Governance docs accessible
- [ ] All tests pass

---

## Contacts

- On-call: @engineering-ops
- Backend MCP: @backend-team
- UI Marketplace: @frontend-team
- Governance: @user

---

## Last Updated

- Phase 4: MCP, Tool Ecosystem, and Governance Parity
- Version: 1.0
- Date: 2026-03-31
