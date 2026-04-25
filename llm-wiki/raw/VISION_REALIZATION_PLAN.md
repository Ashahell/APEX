# APEX Vision Realization Plan

> **Date**: 2026-03-09
> **Goal**: Realize the vision of combining OpenClaw + AgentZero with security-first design
> **Platform**: Windows 11 (primary)

---

## Current State Assessment

### What We Have ✅

| Component | Status | Notes |
|-----------|--------|-------|
| **Vector Database** | ✅ Implemented | sqlite_vec + embedder (local/OpenAI) + hybrid search |
| **Memory System** | ✅ Implemented | Narrative, journal, reflections, working memory |
| **MCP Client/Server** | ✅ Implemented | Connection pooling, validation, metrics |
| **Skills Framework** | ✅ 33 skills | T0-T3 permission tiers |
| **VM Pool** | ⚠️ Configured | Docker works, Firecracker/gVisor need testing |
| **UI Theme** | ✅ AgentZero style | Dark navy/cyan aesthetic |
| **Execution Engine** | ✅ Docker | Python agent in containers |

### What We Need 🔧

| Component | Status | Priority |
|-----------|--------|----------|
| **Firecracker on Windows** | ❌ Not working | P0 - Critical |
| **Thought Streaming UI** | ❌ Partial | P0 - Critical |
| **Subagent Pool** | ❌ Not implemented | P1 - High |
| **Dynamic Tool Gen** | ❌ Not implemented | P1 - High |
| **MCP Marketplace** | ❌ Not implemented | P2 - Medium |
| **Real Security Audit** | ❌ Never done | P1 - High |

---

## Phase 1: Foundation (Weeks 1-2)

### 1.1 Execution Isolation - Windows Compatible

**Problem**: Firecracker requires KVM, doesn't work natively on Windows. gVisor has limited Windows support.

**Solution Options**:

| Option | Security | Windows Support | Effort |
|--------|----------|-----------------|--------|
| A: WSL2 + Firecracker | ⭐⭐⭐⭐⭐ | Good | Medium |
| B: Docker (current) | ⭐⭐⭐⭐ | Excellent | Low |
| C: gVisor via WSL2 | ⭐⭐⭐⭐⭐ | Good | Medium |
| D: Hyper-V based | ⭐⭐⭐⭐⭐ | Native | High |

**Recommended**: Hybrid approach for Windows:
1. **Default**: Docker with security hardening (read-only, no network, resource limits)
2. **WSL2 path**: For Firecracker when user has WSL2 installed
3. **Future**: Hyper-V micro-VM when stable

**Deliverables**:
- [ ] Document Windows execution setup (WSL2 + Firecracker guide)
- [ ] Add WSL2 detection and auto-configuration
- [ ] Hardened Docker config (seccomp, no network, minimal perms)
- [ ] Test scripts for each backend on Windows

### 1.2 Real-Time Thought Streaming

**Problem**: Agent thoughts not streaming to UI in real-time.

**Current State**: Basic execution events stream, but not full thought trace.

**Solution**:
```
LLM → Thought/Action/Observation → WebSocket → UI ProcessGroup
```

**Deliverables**:
- [ ] Extend message bus with `Thought` event type
- [ ] Update LLM client to stream tokens
- [ ] Frontend ProcessGroup component for thought trace
- [ ] Toggle to enable/disable thought streaming (cost saving)

---

## Phase 2: Core Capabilities (Weeks 3-5)

### 2.1 Subagent Pool

**Problem**: Single-task worker model, no parallelism for complex tasks.

**Solution**:
```
Deep Task → Split into subtasks → Distribute to pool → Aggregate results
```

**Architecture**:
```
                    ┌─────────────┐
                    │ Task Router │
                    └──────┬──────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
    ┌────▼────┐      ┌────▼────┐      ┌────▼────┐
    │Worker 1 │      │Worker 2 │      │Worker N │
    │(subtask)│      │(subtask)│      │(subtask)│
    └────┬────┘      └────┬────┘      └────┬────┘
         │                 │                 │
         └─────────────────┼─────────────────┘
                           │
                    ┌──────▼──────┐
                    │  Aggregator │
                    └─────────────┘
```

**Deliverables**:
- [x] Subagent pool worker manager
- [x] Task decomposition logic (what can parallelize)
- [x] Result aggregation
- [x] Timeout and failure handling per subtask

### 2.2 Dynamic Tool Generation

**Problem**: Fixed skill set, no ability to create tools at runtime.

**Solution**:
```
User Request → LLM generates Python code → Validate → Create skill → Execute
```

**Security Concerns**:
- Code execution sandbox required
- Whitelist allowed imports
- Time/memory limits
- Audit trail mandatory

**Deliverables**:
- [x] Tool generation prompt template
- [x] Python code validator (allowed imports, no network, etc.)
- [x] Dynamic skill registration
- [ ] Tool versioning/tracking

### 2.3 Enhanced Memory System

**What's Already There**:
- sqlite_vec for vector search
- Embedder (local via llama-server or OpenAI)
- Hybrid search (keyword + vector)
- Background indexer
- Narrative memory, journal, reflections
- Temporal decay for search ranking
- SOUL.md loader with auto-reload on wake

**What's Added**:
- [x] Memory export/import functionality

**What's Missing**:
- [ ] Memory consolidation (auto-delete old entries based on retention)
- [ ] Memory retrieval with context window optimization

---

## Phase 3: Ecosystem (Weeks 6-8)

### 3.1 MCP Marketplace

**Problem**: Limited to built-in skills, no discovery.

**Solution**:
```
Registry Server → Tool Index → APEX discovers → Adds to skill pool
```

**Features**:
- [x] REST endpoint for registry management
- [x] Tool search/filter
- [x] Version tracking
- [x] Trust/reputation system (simple)

### 3.2 Skill SDK & Plugin System

**Problem**: Skills are hardcoded, no easy extension.

**Solution**:
```
skill-name/
├── skill.yaml        # Metadata
├── src/
│   └── index.ts      # Entry point
├── prompts/          # LLM prompts
└── tests/            # Test cases
```

**Features**:
- [x] skill.yaml schema
- [x] CLI for scaffolding new skills (`apex-skills create <name>`)
- [x] Local registry file system watcher
- [x] Hot-reload for development

---

## Phase 4: Security Hardening (Weeks 9-12)

### 4.1 Security Audit Prep

**Before Audit**:
- [x] Complete threat model document
- [x] Input sanitization review (already done for MCP)
- [x] Auth flow documentation
- [ ] Penetration test scope definition

### 4.2 Security Improvements Based on Audit

**Implemented**:
- [x] Rate limiting (token bucket algorithm)
- [x] Audit log integrity (hash chain verification)
- [ ] Session management
- [ ] Secret storage (currently env vars, should be encrypted)

### 4.3 Hardened Docker Configuration

**Current**: Basic resource limits

**Hardened**:
- [x] Custom seccomp profile (Docker default)
- [x] No capabilities (drop all)
- [x] Read-only root filesystem
- [x] No new privileges
- [x] Network namespace isolation
- [ ] AppArmor/SELinux profiles (if available)

---

## Phase 5: Polish & Features (Weeks 13-16)

### 5.1 UI Enhancements

- [x] Streaming thought visualization - ProcessGroup component with Thought/ToolCall/ToolResult events
- [x] Skill marketplace UI - SkillMarketplace.tsx component
- [x] Memory timeline visualization - MemoryViewer, NarrativeMemoryViewer components
- [x] Execution cost breakdown UI - MetricsPanel, session cost in header
- [x] Keyboard shortcuts documentation - KEYBOARD_SHORTCUTS.md

### 5.2 Documentation

- [x] API documentation (auto-generated) - Existing API endpoints in AGENTS.md
- [x] Deployment guide for Windows - VM_BACKEND_WINDOWS.md, FIRECRACKER_WSL2.md
- [x] Security architecture document - SECURITY.md, SECURITY_THREAT_MODEL.md
- [x] Skill development guide - SKILL-SDK.md

---

## Windows 11 Specific Considerations

### Firecracker on Windows

**The Challenge**: Firecracker requires KVM which is Linux-only. Windows needs WSL2 or Hyper-V.

**Recommended Path**:
```powershell
# Option 1: WSL2 (recommended)
# Pros: Full Linux kernel, Firecracker works
# Cons: Requires Windows Pro/Enterprise, 4GB RAM for VM

# Install WSL2
wsl --install

# Verify
wsl -l -v

# Set up Firecracker in WSL2
# See docs/FIRECRACKER_WSL2.md
```

**Alternative**: Use Docker as default on Windows, document WSL2 path for advanced users.

### Development Setup on Windows

```powershell
# Prerequisites
- Rust 1.93+ (via rustup)
- Node.js 20+ (via nvm)
- Python 3.11+ (via pyenv-win or direct)
- Docker Desktop (for execution)
- pnpm
- Poetry

# Clone and build
git clone https://github.com/your-repo/apex.git
cd apex

# Build Rust
cargo build --release

# Install frontend deps
cd ui && pnpm install

# Start (uses apex.bat)
.\apex.bat start
```

---

## Priority Matrix

| Priority | Item | Effort | Impact |
|----------|------|--------|--------|
| P0 | Firecracker on Windows (WSL2) | 2 weeks | Security |
| P0 | Thought streaming | 2 weeks | UX |
| P1 | Subagent pool | 3 weeks | Capability |
| P1 | Dynamic tools | 3 weeks | Capability |
| P1 | Security audit prep | 2 weeks | Security |
| P2 | MCP marketplace | 3 weeks | Ecosystem |
| P2 | Skill SDK | 2 weeks | Ecosystem |
| P3 | UI polish | Ongoing | UX |

---

## Success Metrics

| Phase | Metric | Target |
|-------|--------|--------|
| 1 | Execution backend works on Windows | Docker + WSL2 Firecracker |
| 1 | Thought streaming latency | < 500ms |
| 2 | Subagent parallel tasks | 4+ workers |
| 2 | Dynamic tool success rate | > 90% |
| 3 | Marketplace tools available | 50+ |
| 4 | Security issues found | < 10 (critical) |
| 5 | UI satisfaction score | > 4/5 |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Firecracker won't work on WSL2 | Medium | High | Fall back to Docker, document limitations |
| Dynamic tool generation unsafe | High | Critical | Strict sandbox, audit all generated code |
| LLM cost explodes with streaming | High | Medium | Toggle, budget limits, caching |
| Scope creep | High | Medium | Strict phase gates, cut non-essentials |

---

## Immediate Next Steps (This Week)

1. **Test current Docker execution** - Verify it works reliably
2. **Set up WSL2 + Firecracker** - Document the process
3. **Add thought streaming** - Start with LLM client modifications
4. **Create VM_BACKEND_WINDOWS.md** - Document Windows-specific setup

---

*This plan is iterative. Review and update bi-weekly.*
