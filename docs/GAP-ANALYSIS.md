# APEX Design Gap Analysis

**Date**: 2026-03-02  
**Document**: Comparison of `docs/APEX-Design.md` (v4.0) vs Actual Implementation

---

## Executive Summary

This document identifies gaps between the architectural specification (v4.0) and the current implementation. The implementation covers core functionality but lacks several advanced features specified in the design.

**Overall Assessment**: ~60% implementation of design spec  
**Critical Gaps**: 5  
**Medium Gaps**: 8  
**Minor Gaps**: 6

---

## 1. Architecture Layer Comparison

### Design (Section 4)
```
L1: Messaging Gateway → NATS → L2: Task Router → L4/L5 → L3: Memory
L6: Web UI (via WebSocket)
```

### Implementation
```
L1: Messaging Gateway → tokio broadcast → L2: Task Router → L4/L5 → L3: Memory
L6: Web UI (via HTTP polling, not WebSocket)
```

| Aspect | Design | Implementation | Gap |
|--------|--------|----------------|-----|
| Messaging bus | NATS JetStream | tokio broadcast | **Medium** - Simplified |
| L6→L2 transport | WebSocket | HTTP polling | **Medium** |
| Distributed ready | Yes | No (single-node) | Minor |

**Noted in spec**: NATS was removed due to OpenSSL dependency issues (acceptable simplification)

---

## 2. Database Schema Gaps

### Design (Section 8.1)
Required tables:
- `tasks` ✅
- `audit_log` ✅
- `messages` ✅
- `skill_registry` ✅
- `vector_store` ✅
- `preferences` ✅
- **`workflows`** ❌

### Implementation Tables
```sql
tasks                  -- ✅
audit_log             -- ✅ (with hash chain)
messages              -- ✅
skill_registry       -- ✅
preferences          -- ✅ (with base64 encoding)
vector_store         -- ✅ (JSON-based, not sqlite-vec)
workflows            -- ❌ MISSING
```

| Gap | Severity | Notes |
|-----|----------|-------|
| Missing `workflows` table | Medium | No YAML workflow storage |
| vector_store not sqlite-vec | Minor | Using JSON file storage |
| No memory tier enforcement | Medium | No TTL/retention policies |

---

## 3. Skill Registry Gaps

### Design (Section 7): 50 curated skills
- Development: 15 skills
- AI Music: 10 skills  
- AI Video: 8 skills
- Script Writing: 8 skills
- Marketing: 9 skills

### Implementation: 5 skills
```
skills/skills/code.generate/
skills/skills/code.review/
skills/skills/shell.execute/
skills/skills/docs.read/
skills/skills/git.commit/
```

| Gap | Severity | Impact |
|-----|----------|--------|
| 45 skills missing (90%) | **Critical** | Core use case not fulfilled |
| No AI Music skills | **Critical** | Major design feature missing |
| No AI Video skills | **Critical** | Major design feature missing |
| No Script Writing skills | **Critical** | Major design feature missing |
| No Marketing skills | **Critical** | Major design feature missing |
| No skill tier confirmation UI | Medium | T1-T3 not enforced |

---

## 4. Messaging Gateway Gaps

### Design (Section 9): 6 channels
| Channel | Status |
|---------|--------|
| Slack | ✅ Implemented |
| Discord | ✅ Implemented |
| Telegram | ✅ Implemented |
| WhatsApp | ❌ Missing |
| Email (IMAP/SMTP) | ❌ Missing |
| REST API | ✅ Implemented |

| Gap | Severity |
|-----|----------|
| WhatsApp adapter | **Critical** |
| Email adapter | **Critical** |

---

## 5. Web UI Features Gaps

### Design (Section 6.2): Required Features
- Chat Interface ✅
- Skill Marketplace ✅ (basic)
- File Browser ✅ (placeholder)
- **Memory Viewer** ❌
- **Workflow Visualizer** ❌
- Cost Dashboard ✅
- Settings ✅
- **Real-time streaming** ❌ (uses polling)

### Implementation Features
```
ui/src/components/
├── chat/Chat.tsx        -- ✅ Chat
├── skills/Skills.tsx    -- ✅ Skill marketplace (basic)
├── files/Files.tsx      -- ✅ File browser (placeholder)
├── kanban/KanbanBoard.tsx -- ✅ Task board
└── settings/Settings.tsx -- ✅ Settings + metrics
```

| Gap | Severity |
|-----|----------|
| Memory Viewer | Medium |
| Workflow Visualizer | Medium |
| WebSocket real-time | Medium |
| Monaco Editor | Minor (using placeholder) |

---

## 6. Execution Engine Gaps

### Design (Section 10): Firecracker micro-VMs
- 125ms cold start
- 1 vCPU, 512MB RAM, 5-min timeout
- Network allowlist
- 2-3 pre-warmed VMs

### Implementation: Mock backend
```
core/router/src/vm_pool.rs
├── MockVmBackend     -- ✅ Implemented
├── DockerBackend    -- ✅ Implemented (fallback)
├── GVisorBackend   -- ✅ Implemented (fallback)
└── FirecrackerBackend -- ❌ NOT IMPLEMENTED
```

| Gap | Severity | Notes |
|-----|----------|-------|
| No Firecracker integration | **Critical** | Security isolation missing |
| No real VM pre-warming | **Critical** | Performance issue |
| Network allowlist | Medium | Not enforced in mock |
| Resource limits | Medium | Not enforced in mock |

---

## 7. Security Gaps

### Design (Section 5)
| Feature | Design | Implementation |
|---------|--------|----------------|
| Capability tokens | PASETO v4 | Simple base64 |
| Permission tiers | T0-T3 with confirmations | TaskTier (no gates) |
| Prompt injection defense | Regex + separation | Basic sanitize function |
| CSP headers | Strict | Basic TraceLayer |

| Gap | Severity | Notes |
|-----|----------|-------|
| PASETO tokens | Minor | Using base64 (acceptable) |
| T1-T3 confirmation UI | **Critical** | No enforcement |
| VM isolation | **Critical** | Using mock/Docker |

---

## 8. Additional Gaps

### Design Feature | Implementation Status
---|---
**Dynamic Tool Promotion** (Section 10.3) | ❌ Not implemented
**Skill Development Kit** (Section 5.4) | ❌ Not implemented
**Cost Estimation** (Section 8) | ⚠️ Basic tracking only
**Upstream fork tracking** | ❌ Not present
**Conversation history TTL** | ❌ Not enforced (90-day design)

---

## Summary Table

| Category | Design Items | Implemented | Gap | Severity |
|----------|-------------|-------------|-----|----------|
| Architecture | 6 layers | 6 layers | NATS→broadcast, WS→polling | Medium |
| Database tables | 7 | 6 | workflows missing | Medium |
| Skills | 50 | 5 | 45 missing (90%) | **Critical** |
| Messaging channels | 6 | 4 | WhatsApp, Email | **Critical** |
| UI features | 8 | 5 | Memory viewer, Workflow | Medium |
| Execution | Firecracker | Mock | No VM isolation | **Critical** |
| Security | Full | Basic | No T1-T3 gates | **Critical** |

---

## Recommendations

### Priority 1 (Critical)
1. **Implement T1-T3 confirmation UI** - Security requirement
2. **Add more skills** - At minimum: 5 more core dev skills
3. **Add WhatsApp/Email adapters** - Design requirement

### Priority 2 (High)
4. **Implement Firecracker** - Or document gVisor as production
5. **Add Memory Viewer UI** - Section 6.2 requirement

### Priority 3 (Medium)
6. **Add Workflow Visualizer**
7. **Add WebSocket for real-time updates**
8. **Implement workflows table**

---

## Already Addressed (From Implementation)

| Item | Status |
|------|--------|
| NATS → tokio broadcast | ✅ Documented in IMPLEMENTATION-PLAN.md |
| PASETO → base64 | ✅ Documented |
| SQLite migrations | ✅ Implemented |
| 48 tests passing | ✅ Implemented |
| Kanban board | ✅ Added post-spec |

---

*This analysis based on APEX-Design.md v4.0 and implementation as of 2026-03-02*
