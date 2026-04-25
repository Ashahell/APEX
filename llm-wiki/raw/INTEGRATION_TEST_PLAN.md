# Integration Test Implementation Plan

> **Date**: 2026-03-24  
> **Version**: 1.0  
> **Purpose**: Comprehensive integration test coverage for critical APEX paths

---

## Executive Summary

This plan outlines integration tests for critical paths in the APEX codebase that are currently not covered by existing unit and integration tests. The focus is on end-to-end flows that involve multiple components working together.

### Current Test Coverage

| Test Type | Count | Coverage |
|-----------|-------|----------|
| Unit Tests (Rust) | 313 | Core modules |
| Integration Tests (Rust) | 59 | Basic API endpoints |
| Python Tests | 53 | Sandbox, enforcement |
| TypeScript Tests | 16 | Gateway, Skills, UI |

### Identified Gaps

Critical paths lacking integration tests:
1. Authentication & Authorization Flow
2. Task Classification Pipeline
3. Skill Execution End-to-End
4. Memory Operations (Create → Search → Retrieve)
5. WebSocket Event Flow
6. MCP Server Integration
7. TOTP Verification Flow
8. Bounded Memory Consolidation
9. Session Search Indexing

---

## Test Categories

### Category 1: Authentication & Authorization (Priority: Critical)

Tests the complete auth flow from request to permission check.

#### Test 1.1: HMAC Request Signing Flow
**File**: `tests/integration.rs`  
**Function**: `test_hmac_auth_flow`

**Steps**:
1. Create request with timestamp, method, path
2. Generate HMAC-SHA256 signature
3. Send request with signature headers
4. Verify request passes auth middleware
5. Verify invalid signature is rejected
6. Verify expired timestamp is rejected

**Assertions**:
- Valid signature returns 200
- Invalid signature returns 401
- Expired timestamp returns 401
- Missing headers returns 401

#### Test 1.2: Permission Tier Enforcement
**File**: `tests/integration.rs`  
**Function**: `test_tier_permission_flow`

**Steps**:
1. Create task requiring T0 (read-only)
2. Verify T0 task executes without confirmation
3. Create task requiring T1 (tap confirm)
4. Verify T1 task requires confirmation
5. Create task requiring T2 (type confirm)
6. Verify T2 task requires type confirmation
7. Verify T3 task requires TOTP

**Assertions**:
- T0: Immediate execution
- T1: Returns confirmation_required status
- T2: Returns confirmation_required with action text
- T3: Returns confirmation_required with TOTP requirement

#### Test 1.3: TOTP Verification Flow
**File**: `tests/integration.rs`  
**Function**: `test_totp_verification_flow`

**Steps**:
1. Setup TOTP for user (generate secret)
2. Verify TOTP status shows not configured
3. Verify TOTP with valid token succeeds
4. Verify TOTP with invalid token fails
5. Verify TOTP with reused token fails

**Assertions**:
- Initial status shows configured: false
- Valid token returns success
- Invalid token returns error
- Reused token returns error

---

### Category 2: Task Processing Pipeline (Priority: Critical)

Tests the complete task flow from creation to execution.

#### Test 2.1: Task Classification Flow
**File**: `tests/integration.rs`  
**Function**: `test_task_classification_flow`

**Steps**:
1. Create instant-classified task ("what time is it")
2. Verify task classified as Instant tier
3. Create shallow-classified task ("generate code")
4. Verify task classified as Shallow tier
5. Create deep-classified task (complex prompt)
6. Verify task classified as Deep tier

**Assertions**:
- Simple queries → Instant
- Single skill → Shallow  
- Complex reasoning → Deep

#### Test 2.2: Skill Execution Flow
**File**: `tests/integration.rs`  
**Function**: `test_skill_execution_flow`

**Steps**:
1. Register a test skill with execution handler
2. Create task that triggers the skill
3. Verify skill executes and returns result
4. Verify tool_calls recorded in task
5. Verify execution time tracked

**Assertions**:
- Skill executes successfully
- Result matches expected output
- Metadata recorded correctly

#### Test 2.3: Deep Task Execution Flow
**File**: `tests/integration.rs`  
**Function**: `test_deep_task_execution_flow`

**Steps**:
1. Create deep task with LLM requirement
2. Verify task enters processing state
3. Verify subtasks created
4. Verify task completes with reasoning
5. Verify cost calculated correctly

**Assertions**:
- Task state transitions: pending → running → completed
- Subtasks have proper parent_id
- Final response contains reasoning

---

### Category 3: Memory Operations (Priority: High)

Tests the complete memory lifecycle.

#### Test 3.1: Memory CRUD Flow
**File**: `tests/integration.rs`  
**Function**: `test_memory_crud_flow`

**Steps**:
1. Create memory entry with content
2. Verify memory saved with ID
3. Retrieve memory by ID
4. Verify content matches
5. Update memory content
6. Verify updated content
7. Delete memory
8. Verify memory no longer exists

**Assertions**:
- Create returns valid ID
- Retrieve returns correct content
- Update reflects new content
- Delete removes entry

#### Test 3.2: Memory Search Flow
**File**: `tests/integration.rs`  
**Function**: `test_memory_search_flow`

**Steps**:
1. Create multiple memory entries with different content
2. Index entries for search
3. Perform semantic search with query
4. Verify relevant results returned
5. Verify results ranked by relevance

**Assertions**:
- Results contain relevant entries
- Results ordered by relevance
- Empty query returns recent entries

#### Test 3.3: Bounded Memory Consolidation
**File**: `tests/integration.rs`  
**Function**: `test_bounded_memory_consolidation`

**Steps**:
1. Add entries until approaching limit (2200 chars)
2. Verify warning threshold triggered
3. Add more entries over limit
4. Verify consolidation runs
5. Verify total within limit
6. Verify snapshot updated

**Assertions**:
- Warning at 80% capacity
- Consolidation reduces size
- Snapshot contains key information

---

### Category 4: WebSocket Real-Time (Priority: High)

Tests WebSocket connectivity and event flow.

#### Test 4.1: WebSocket Connection Flow
**File**: `tests/integration.rs`  
**Function**: `test_websocket_connection_flow`

**Steps**:
1. Connect to WebSocket endpoint
2. Verify connection established
3. Receive welcome/connected event
4. Create task via REST API
5. Verify task event received via WebSocket
6. Disconnect gracefully
7. Verify disconnect event

**Assertions**:
- Connection returns success
- Events arrive in order
- Task updates visible in real-time
- Clean disconnect without errors

#### Test 4.2: WebSocket Event Types
**File**: `tests/integration.rs`  
**Function**: `test_websocket_event_types`

**Steps**:
1. Connect to WebSocket
2. Create task → verify task.created event
3. Start task → verify task.started event
4. Complete task → verify task.completed event
5. Receive error → verify error event

**Assertions**:
- All event types received
- Event payload matches expected schema
- Events include timestamps

---

### Category 5: MCP Integration (Priority: Medium)

Tests MCP server connectivity and tool execution.

#### Test 5.1: MCP Server Connection Flow
**File**: `tests/integration.rs`  
**Function**: `test_mcp_server_connection`

**Steps**:
1. Register MCP server configuration
2. Connect to MCP server
3. Verify connection status is "connected"
4. List available tools
5. Verify tools returned

**Assertions**:
- Connection succeeds
- Status is "connected"
- Tools list is non-empty

#### Test 5.2: MCP Tool Execution Flow
**File**: `tests/integration.rs`  
**Function**: `test_mcp_tool_execution`

**Steps**:
1. Connect to MCP server with tools
2. Select a tool from list
3. Execute tool with test parameters
4. Verify tool execution succeeds
5. Verify result returned correctly

**Assertions**:
- Execution returns success
- Result matches expected output format

---

### Category 6: Skills System (Priority: High)

Tests skill registration, discovery, and execution.

#### Test 6.1: Skill Registration Flow
**File**: `tests/integration.rs`  
**Function**: `test_skill_registration_flow`

**Steps**:
1. Prepare skill manifest (SKILL.md)
2. Register skill via API
3. Verify skill appears in list
4. Verify skill has correct tier
5. Verify health check passes

**Assertions**:
- Registration returns success
- Skill retrievable by name
- Tier matches manifest

#### Test 6.2: Auto-Created Skill Flow
**File**: `tests/integration.rs`  
**Function**: `test_auto_created_skill_flow`

**Steps**:
1. Execute task with 5+ tool calls
2. Verify skill suggestion generated
3. Review suggestion
4. Accept suggestion (create skill)
5. Verify skill registered
6. Execute new skill
7. Verify skill executes correctly

**Assertions**:
- Suggestion appears after threshold
- New skill registered successfully
- Skill executable

---

### Category 7: Workflow Execution (Priority: Medium)

Tests workflow creation and execution.

#### Test 7.1: Workflow Execution Flow
**File**: `tests/integration.rs`  
**Function**: `test_workflow_execution_flow`

**Steps**:
1. Create workflow with steps
2. Execute workflow
3. Verify first step starts
4. Complete first step
5. Verify second step starts
6. Complete all steps
7. Verify workflow completed

**Assertions**:
- Steps execute in order
- Each step state transitions correctly
- Final state is "completed"

---

### Category 8: External Integrations (Priority: Low)

Tests external service integrations.

#### Test 8.1: LLM Connection Flow
**File**: `tests/integration.rs`  
**Function**: `test_llm_connection_flow`

**Steps**:
1. Configure LLM endpoint
2. Test connection
3. Send simple prompt
4. Verify response received

**Assertions**:
- Connection test returns success
- Response is valid

#### Test 8.2: Webhook Delivery Flow
**File**: `tests/integration.rs`  
**Function**: `test_webhook_delivery_flow`

**Steps**:
1. Register webhook URL
2. Trigger event (task complete)
3. Verify webhook received
4. Verify payload matches event

**Assertions**:
- Webhook receives POST
- Payload contains expected data

---

## Implementation Order

### Phase 1: Critical Paths (Week 1)

| Step | Test | Estimated Effort |
|------|------|-----------------|
| 1.1 | HMAC Auth Flow | 2 hours |
| 1.2 | Permission Tier Flow | 2 hours |
| 1.3 | TOTP Verification Flow | 2 hours |
| 2.1 | Task Classification Flow | 2 hours |
| 2.2 | Skill Execution Flow | 3 hours |
| 2.3 | Deep Task Flow | 3 hours |

**Phase 1 Total**: ~14 hours

### Phase 2: Core Features (Week 2)

| Step | Test | Estimated Effort |
|------|------|-----------------|
| 3.1 | Memory CRUD Flow | 2 hours |
| 3.2 | Memory Search Flow | 2 hours |
| 3.3 | Bounded Memory Consolidation | 2 hours |
| 4.1 | WebSocket Connection Flow | 2 hours |
| 4.2 | WebSocket Event Types | 2 hours |

**Phase 2 Total**: ~10 hours

### Phase 3: Integrations (Week 3)

| Step | Test | Estimated Effort |
|------|------|-----------------|
| 5.1 | MCP Server Connection | 2 hours |
| 5.2 | MCP Tool Execution | 2 hours |
| 6.1 | Skill Registration Flow | 2 hours |
| 6.2 | Auto-Created Skill Flow | 3 hours |
| 7.1 | Workflow Execution Flow | 2 hours |

**Phase 3 Total**: ~11 hours

### Phase 4: External (Week 4)

| Step | Test | Estimated Effort |
|------|------|-----------------|
| 8.1 | LLM Connection Flow | 1 hour |
| 8.2 | Webhook Delivery Flow | 2 hours |

**Phase 4 Total**: ~3 hours

---

## Test File Structure

### Location
```
core/router/tests/
├── integration.rs          # Existing tests
├── auth_integration.rs    # NEW: Auth & permission tests
├── task_pipeline.rs       # NEW: Classification & execution
├── memory_integration.rs  # NEW: Memory operations
├── websocket_integration.rs # NEW: WebSocket tests
├── mcp_integration.rs    # NEW: MCP tests
├── skills_integration.rs # NEW: Skills tests
└── external_integration.rs # NEW: External services
```

### Test Utilities

Create shared test utilities in `tests/test_utils.rs`:

```rust
// Shared helpers for integration tests
pub async fn create_test_app_state() -> AppState;
pub async fn create_test_task(tier: TaskTier) -> Task;
pub async fn setup_test_skill() -> Skill;
pub fn generate_valid_hmac_headers(method: &str, path: &str, body: &str) -> HeaderMap;
```

---

## Success Criteria

### Definition of Done

Each integration test must:
- [ ] Follow Arrange-Act-Assert pattern
- [ ] Have clear setup and cleanup
- [ ] Use shared test utilities where possible
- [ ] Include descriptive test name and docstring
- [ ] Assert on specific values, not just "no error"
- [ ] Clean up resources after execution

### Coverage Targets

After implementation:
- Authentication flows: 100% coverage
- Task pipeline: 100% coverage  
- Memory operations: 80% coverage
- WebSocket: 100% coverage
- Skills: 80% coverage
- External integrations: 60% coverage

### CI Integration

All integration tests should:
- Run in CI pipeline on PR
- Have timeout of 30 seconds max
- Be independent (no shared state)
- Be idempotent (can run multiple times)

---

## Notes

### Test Data Management

- Use factories for creating test data
- Clean up database between tests
- Use unique identifiers to avoid conflicts

### Async Test Handling

- Use `#[tokio::test]` for async tests
- Set `flavor = "multi_thread"` for tests needing runtime
- Use timeouts to prevent hanging tests

### Debugging Failed Tests

- Include request/response in assertions for debugging
- Log intermediate states for complex flows
- Use `dbg!()` for quick debugging output

---

## References

- Existing integration tests: `core/router/tests/integration.rs`
- Test utilities: Create `tests/test_utils.rs`
- Constants: `core/router/src/unified_config.rs`

---

*Generated for APEX v1.6.0*
