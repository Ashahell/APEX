# Phase 5 Runbook: Memory Architecture Parity (Hermes-style)

## Overview
- Phase 5 implements Hermes-like memory surface in UI and backend; memory search integration; TTL semantics; and consolidation.
- This runbook provides operational steps for incident response, debugging, and escalation.

## Phase 5 Features

| Feature | Description |
|---------|-------------|
| Memory Viewer UI | 6-tab interface: Memory, User, Search, TTL, Consolidation, Snapshot |
| Semantic Search | Hybrid search (BM25 + embeddings) with MMR reranking |
| TTL Semantics | Configurable TTL per store type with auto-cleanup |
| Memory Consolidation | AI-suggested entry consolidation with approval workflow |
| Frozen Snapshots | System prompt-ready snapshots of bounded memory |

---

## Incident Response Procedures

### 1. Search Failures

**Symptoms:**
- Search returns empty results
- Search timeout errors
- Index statistics show 0 indexed entries

**Immediate Steps:**
```bash
# Check index statistics
curl -s http://localhost:3000/api/v1/search/sessions/stats | jq .

# Rebuild search index
curl -X POST http://localhost:3000/api/v1/search/reindex

# Test search query
curl -s "http://localhost:3000/api/v1/search/sessions?q=test&limit=10" | jq .
```

**Debug Commands:**
```bash
# Check FTS5 status
# Review: core/router/src/session_search.rs

# Check embedding server connectivity
curl -s http://localhost:8081/health
```

**Common Causes:**
- FTS5 not available (falls back to LIKE)
- Embedding server not running
- Index corruption

**Rollback:** Rebuild index, restart embedding server.

---

### 2. TTL Cleanup Issues

**Symptoms:**
- Memory entries not expiring
- Auto-cleanup not running
- TTL configuration not persisting

**Immediate Steps:**
```bash
# Check TTL configuration
curl -s http://localhost:3000/api/v1/memory/bounded/ttl | jq .

# Update TTL configuration
curl -X PUT http://localhost:3000/api/v1/memory/bounded/ttl \
  -H "Content-Type: application/json" \
  -d '{"memory_ttl_hours": 168, "user_ttl_hours": 720}'
```

**Debug Commands:**
```bash
# Check cleanup job status
# Review: core/router/src/api/memory_ttl_api.rs

# Verify TTL values are valid
# memory_ttl_hours > 0, user_ttl_hours > 0, cleanup_interval_hours > 0
```

**Common Causes:**
- Invalid TTL configuration
- Cleanup job not scheduled
- Database connection issues

**Rollback:** Reset TTL to defaults, restart router.

---

### 3. Consolidation Problems

**Symptoms:**
- No consolidation candidates found
- Consolidation approval fails
- Memory entries not being consolidated

**Immediate Steps:**
```bash
# Check consolidation candidates
curl -s http://localhost:3000/api/v1/memory/bounded/consolidation/candidates | jq .

# Approve consolidation (example)
curl -X POST http://localhost:3000/api/v1/memory/bounded/consolidation/approve \
  -H "Content-Type: application/json" \
  -d '{"entries": ["entry1", "entry2"], "suggested_summary": "Combined"}'
```

**Debug Commands:**
```bash
# Check consolidation logic
# Review: core/router/src/api/memory_ttl_api.rs

# Verify memory entries exist
curl -s http://localhost:3000/api/v1/memory/bounded/memory | jq .
```

**Common Causes:**
- No similar entries to consolidate
- Consolidation API not wired
- Database write failures

**Rollback:** Manual consolidation via UI, investigate root cause.

---

## Debug Commands Quick Reference

| Command | Purpose |
|---------|---------|
| `curl -s http://localhost:3000/api/v1/memory/bounded/stats` | Memory statistics |
| `curl -s http://localhost:3000/api/v1/search/sessions/stats` | Search index stats |
| `curl -s http://localhost:3000/api/v1/memory/bounded/ttl` | TTL configuration |
| `curl -s http://localhost:3000/api/v1/memory/bounded/consolidation/candidates` | Consolidation candidates |
| `curl -s http://localhost:3000/api/v1/memory/bounded/snapshot` | Frozen snapshot |

---

## Test Commands

```bash
# Run Phase 5 memory tests
cd core && cargo test --test memory_integration_phase5

# Run all memory-related tests
cd core && cargo test memory
cd core && cargo test ttl
cd core && cargo test consolidation
```

---

## Escalation Paths

| Issue | First Contact | Escalation |
|-------|--------------|------------|
| Search failures | @backend-team | @infra-team |
| TTL issues | @backend-team | @engineering-ops |
| Consolidation issues | @backend-team | @engineering-ops |
| UI issues | @frontend-team | @backend-team |

---

## Rollback Procedure

If Phase 5 changes cause critical issues:

1. **Disable memory features:**
   ```bash
   # Remove memory_ttl_api router merge
   # Revert BoundedMemory.tsx to previous version
   ```

2. **Restart services:**
   ```bash
   cargo run --release --bin apex-router
   ```

3. **Verify recovery:**
   ```bash
   curl -s http://localhost:3000/api/v1/memory/bounded/stats
   # Should return basic memory statistics
   ```

---

## Verification Checklist

After any incident, verify:

- [ ] Memory statistics endpoint returns data
- [ ] Search functionality works
- [ ] TTL configuration accessible
- [ ] Consolidation UI renders
- [ ] Frozen snapshot available
- [ ] All 9 Phase 5 tests pass

---

## Contacts

- On-call: @engineering-ops
- Backend Memory: @backend-team
- UI Memory: @frontend-team
- Infrastructure: @infra-team

---

## Last Updated

- Phase 5: Memory Architecture Parity (Hermes-style)
- Version: 1.0
- Date: 2026-03-31
