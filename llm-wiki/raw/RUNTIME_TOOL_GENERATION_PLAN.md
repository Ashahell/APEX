# Runtime Tool Generation Implementation Plan

> **Version**: 1.1
> **Date**: 2026-03-12
> **Status**: ✅ IMPLEMENTED
> **Dependencies**: Sandbox complete

---

## Executive Summary

Runtime Tool Generation allows APEX agents to dynamically create and execute custom Python tools at runtime based on task requirements. Currently, APEX has:
- ✅ **Tool Generation**: LLM generates tool code (IMPLEMENTED in `dynamic_tools.rs`)
- ❌ **Tool Execution**: Placeholder only - needs sandbox (NOT IMPLEMENTED)

This plan outlines the implementation of a secure sandbox environment for executing dynamically generated Python code.

---

## Current State Analysis

### What's Already Implemented

| Component | Location | Status |
|-----------|----------|--------|
| DynamicTool struct | `core/router/src/dynamic_tools.rs` | ✅ Complete |
| ToolRegistry | `core/router/src/dynamic_tools.rs` | ✅ Complete |
| Tool Generation (LLM) | `core/router/src/dynamic_tools.rs:95-172` | ✅ Complete |
| Tool Execution (Sandbox) | `core/router/src/dynamic_tools.rs:174-275` | ✅ Complete |
| Python Sandbox | `execution/src/apex_agent/sandbox.py` | ✅ Complete |
| Agent Integration | `core/router/src/agent_loop.rs:644-680` | ✅ Complete |
| Tool Caching | `core/router/src/agent_loop.rs:355-380` | ✅ Complete |

### The Implementation

The `execute_dynamic_tool()` function now:
1. Locates the sandbox.py file
2. Passes tool code and parameters to Python subprocess
3. Executes code in secure sandbox with:
   - Import allowlist (only safe stdlib)
   - Timeout enforcement (30s)
   - Blocked dangerous builtins (exec, eval, open, etc.)
4. Returns execution result (success/failure, output, timing)

---

## Architecture Design

### Option 1: Python Sandbox (Recommended)

**Approach**: Execute generated Python code in a controlled Python environment.

```
┌─────────────────────────────────────────────────────────────┐
│                      APEX Router                            │
│  ┌─────────────────────────────────────────────────────┐  │
│  │              DynamicToolExecutor                      │  │
│  │  1. Validate generated code                        │  │
│  │  2. Extract parameters                             │  │
│  │  3. Send to Execution Sandbox                      │  │
│  └─────────────────────────────────────────────────────┘  │
└──────────────────────────┬──────────────────────────────────┘
                         │ subprocess / HTTP
                         ▼
┌─────────────────────────────────────────────────────────────┐
│               Python Execution Sandbox                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │
│  │ PyPy/CPython│  │  Restricted │  │   Result        │  │
│  │   Interpreter│  │    imports │  │   Capture      │  │
│  └─────────────┘  └─────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Implementation**: Reuse existing L5 Execution Engine (Python/Docker)

### Option 2: WebAssembly (Future)

**Approach**: Compile Python to WASM for browser-like isolation.

**Pros**: Near-native speed, strong sandboxing
**Cons**: Requires Pyodide integration, limited Python packages

**Timeline**: Not in scope for v1.x

---

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|------------|
| Malicious code execution | Sandboxed execution environment |
| Infinite loops | Timeout enforcement |
| Resource exhaustion | CPU/memory limits |
| File system access | Restricted to working directory |
| Network access | Controlled allowlist |
| Import attacks | Restricted import list |

### Security Controls

1. **Timeout**: Max 30 seconds execution
2. **Memory Limit**: Max 512MB
3. **Import Allowlist**: Only safe stdlib modules
4. **No File I/O**: Except working directory
5. **No Network**: Optional allowlist
6. **Audit Log**: All executions logged

---

## Implementation Phases

### Phase 1: Sandbox Execution Engine

**Duration**: 4-6 hours

**Tasks**:
1.1 Create `PythonSandbox` struct in `execution/src/`
1.2 Implement subprocess-based Python execution
1.3 Add import allowlist filtering
1.4 Implement timeout enforcement
1.5 Add result capture (stdout/stderr)

**Deliverable**: Working Python code execution in sandbox

### Phase 2: Integration with DynamicTools

**Duration**: 2-3 hours

**Tasks**:
2.1 Update `execute_dynamic_tool()` to use sandbox
2.2 Add parameter injection into generated code
2.3 Implement result serialization
4.4 Add error handling and logging

**Deliverable**: Full tool execution pipeline

### Phase 3: Agent Integration

**Duration**: 2 hours

**Tasks**:
3.1 Enable tool generation in agent loop
3.2 Add tool caching (avoid regenerating same tool)
3.3 Implement tool cleanup (expire after N hours)

**Deliverable**: Agents can create and use custom tools

### Phase 4: Testing & Hardening

**Duration**: 3-4 hours

**Tasks**:
4.1 Security tests (infinite loop, memory exhaustion)
4.2 Integration tests with agent loop
4.3 Performance benchmarking
4.4 Documentation

**Deliverable**: Production-ready implementation

---

## Detailed Task Breakdown

### Phase 1: Sandbox Execution Engine

| Task | Description | Est. Hours |
|------|-------------|------------|
| 1.1.1 | Create `python_sandbox.py` with execution logic | 1 |
| 1.1.2 | Implement import allowlist | 0.5 |
| 1.1.3 | Add timeout via signal/thread | 0.5 |
| 1.1.4 | Capture stdout/stderr | 0.5 |
| 1.1.5 | Memory limit via resource module | 0.5 |
| 1.1.6 | Unit tests | 1 |

### Phase 2: Integration

| Task | Description | Est. Hours |
|------|-------------|------------|
| 2.1.1 | Update `execute_dynamic_tool()` | 1 |
| 2.1.2 | Parameter injection | 0.5 |
| 2.1.3 | Error handling | 0.5 |
| 2.1.4 | Logging | 0.5 |

### Phase 3: Agent Integration

| Task | Description | Est. Hours |
|------|-------------|------------|
| 3.1.1 | Enable in agent loop | 0.5 |
| 3.1.2 | Tool caching | 1 |
| 3.1.3 | Tool expiration | 0.5 |

### Phase 4: Testing

| Task | Description | Est. Hours |
|------|-------------|------------|
| 4.1.1 | Security tests | 1 |
| 4.1.2 | Integration tests | 1 |
| 4.1.3 | Documentation | 1 |

---

## File Changes

### New Files

| File | Purpose |
|------|---------|
| `execution/src/apex_agent/sandbox.py` | Python sandbox executor (33 tests passing) |
| `execution/tests/test_sandbox.py` | Sandbox security tests (33 tests) |

### Modified Files

| File | Changes |
|------|---------|
| `core/router/src/dynamic_tools.rs` | Replaced placeholder with real sandbox execution |
| `core/router/src/agent_loop.rs` | Added tool caching to avoid regenerating same tools |

---

## Acceptance Criteria

- [x] Generated Python code executes in sandbox
- [x] Execution timeout enforced (30s max)
- [x] Memory limit enforced (512MB max) - with platform-specific support
- [x] Only allowed imports available
- [x] Parameters correctly injected
- [x] Results captured and returned
- [x] Errors handled gracefully
- [x] All executions logged
- [x] Integration tests pass
- [x] Tool expiration/cleanup - tools older than 24h are removed

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Security vulnerabilities | Extensive security testing, start restrictive |
| Performance issues | Benchmark before/after, optimize hot paths |
| LLM generates bad code | Add validation, error handling, fallback |
| Sandbox escape | Use multiple layers (Docker + seccomp) |

---

## Future Enhancements (Post-v1.x)

1. **Tool Marketplace**: Share generated tools between agents
2. **Tool Versioning**: Track tool evolution over time
3. **WASM Sandbox**: Faster execution via WebAssembly
4. **Multi-language**: Support JavaScript/TypeScript tools

---

## References

- [E2B Sandbox](https://e2b.dev) - Firecracker-based Python sandboxes
- [OpenAI Code Interpreter](https://openai.com/blog/chatgpt-can-now-see-hear-and-speak) - Production implementation
- [Python Restricted Interpreter](https://docs.python.org/3/library/restricted.html) - stdlib restrictions
- [gVisor](https://gvisor.dev) - Container sandboxing

---

*This plan was created based on research into production AI agent runtimes and analysis of the current APEX codebase.*
