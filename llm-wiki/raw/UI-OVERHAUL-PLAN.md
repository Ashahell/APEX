# APEX UI Overhaul Plan
## Based on APEX_GUI_Design_Specification.md Analysis

**Created**: 2026-03-03  
**Current State**: v0.1.0 - Basic React UI with Chat, Skills, Kanban, Memory, Settings  
**Target State**: v1.0 - Full specification implementation per design doc

---

## Executive Summary

The current APEX UI (v0.1.0) provides basic functionality but lacks many features specified in the design document. This plan phases the implementation to deliver maximum value early while building toward the complete specification.

### Gap Analysis Summary

| Category | Current State | Target State | Priority |
|----------|--------------|--------------|----------|
| Real-time comms | Polling (setInterval) | WebSocket + SSE | **CRITICAL** |
| Task sidebar | None | Right panel with active tasks | HIGH |
| Process groups | None | GEN/USE/EXE/AUD badges | HIGH |
| Confirmation gates | Modal (5s delay for T3) | Inline + TOTP for T3 | HIGH |
| Budget ticker | None | Topbar live cost | MEDIUM |
| Connection status | Simple dot | Connected/Degraded/Disconnected | HIGH |
| Channel management | None | Card-per-channel UI | MEDIUM |
| Memory viewer | Basic | 3-tab tiered view | MEDIUM |
| Settings | Modal | Full-page schema-driven | LOW |

---

## Phase 1: Foundation & Real-Time (Weeks 1-2)

### Goal: Replace polling with WebSocket, add task sidebar, fix connection status

### 1.1 WebSocket Integration (CRITICAL)

**Files to modify:**
- `ui/src/lib/api.ts` - Add WebSocket client
- `ui/src/stores/appStore.ts` - Add WebSocket state management
- `ui/src/App.tsx` - Initialize WebSocket connection on mount

**Implementation:**
```typescript
// New: ui/src/lib/websocket.ts
import { apiPost } from './api';

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  
  connect() {
    const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:3000/api/v1/ws';
    this.ws = new WebSocket(wsUrl);
    
    this.ws.onmessage = (event) => {
      const update = JSON.parse(event.data);
      // Dispatch to store based on update.type
    };
    
    this.ws.onclose = () => this.reconnect();
  }
  
  private reconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => this.connect(), Math.pow(2, this.reconnectAttempts) * 1000);
      this.reconnectAttempts++;
    }
  }
}
```

**API changes required:**
- Add WebSocket endpoint in `core/router/src/api.rs`
- Subscribe to task updates via message bus

### 1.2 Connection Status Indicator

**Current:** Simple green/gray dot  
**Target:** Connected (green) / Degraded (amber) / Disconnected (red) with text

**Files to modify:**
- `ui/src/App.tsx` - Enhanced header status
- `ui/src/stores/appStore.ts` - Add connectionState: 'connected' | 'degraded' | 'disconnected'

### 1.3 Task Sidebar (Right Panel)

**Current:** None  
**Target:** Right 280px panel showing active tasks with status

**Files to create:**
- `ui/src/components/chat/TaskSidebar.tsx`

**Implementation:**
```tsx
// Right sidebar showing active tasks
<TaskSidebar>
  <TaskItem status="running">
    <TaskIcon type="deep" />
    <TaskInfo title="Code refactor" elapsed="00:42" />
  </TaskItem>
  <TaskItem status="awaiting_confirmation">
    <TaskIcon type="skill" tier="T2" />
    <TaskInfo title="git push" badge="Needs confirm" />
  </TaskItem>
</TaskSidebar>
```

### 1.4 Phase 1 Deliverables

| Deliverable | Status |
|------------|--------|
| WebSocket client | NEW |
| Connection status indicator (3-state) | ENHANCE |
| Task sidebar in Chat view | NEW |
| Remove polling from Chat.tsx | DEPRECATE |
| SSE endpoint in Router | NEW |

---

## Phase 2: Process Transparency (Weeks 3-5)

### Goal: Process groups with badges, budget ticker, decision journal

### 2.1 Process Group Component (HIGHEST PRIORITY)

**Current:** None  
**Target:** Collapsible card with GEN/USE/EXE/AUD badges

**Files to create:**
- `ui/src/components/chat/ProcessGroup.tsx`
- `ui/src/components/chat/ProcessStep.tsx`

**Badge mapping:**
| Badge | Color | Meaning |
|-------|-------|---------|
| GEN | Blue | LLM reasoning |
| USE | Teal | Skill invocation |
| EXE | Amber | Code execution |
| WWW | Purple | Web access |
| SUB | Indigo | Subagent |
| MEM | Green | Memory read/write |
| AUD | Red | Audit event |

**Implementation:**
```tsx
<ProcessGroup status="running">
  <ProcessGroupHeader 
    title="Refactor auth module"
    steps={4}
    elapsed="00:23"
    cost="$0.008"
  />
  <ProcessStep type="GEN" title="Planning approach" expanded={false} />
  <ProcessStep type="USE" title="tool: repo.search" expanded={false} />
  <ProcessStep type="EXE" title="python: ast_parser.py" expanded={false} />
  <ProcessGroupResponse>
    Here is the refactored auth module...
  </ProcessGroupResponse>
</ProcessGroup>
```

### 2.2 Inline Confirmation Gates

**Current:** Modal with 5-second delay for T3  
**Target:** Inline in chat stream + TOTP verification

**Files to modify:**
- `ui/src/components/ui/ConfirmationModal.tsx` - Convert to inline component
- Add TOTP input field for T3

**T3 Implementation:**
```tsx
{/* T3 Confirmation Gate - inline in chat */}
<ConfirmationGate tier="T3" action={action}>
  <ActionPlan>
    Skill: deploy.kubectl
    Target: production cluster
    Action: rolling update
  </ActionPlan>
  <TotpInput onChange={setTotpCode} />
  <ConfirmButton disabled={!totpCode || !verified} />
</ConfirmationGate>
```

**API integration:**
- Call `/api/v1/totp/verify` with user-entered code
- Show "Verified" state before enabling confirm

### 2.3 Budget Ticker

**Current:** None  
**Target:** Topbar live cost display

**Files to modify:**
- `ui/src/App.tsx` - Add budget to header
- `ui/src/stores/appStore.ts` - Add currentSessionCost, monthlyCost

**Implementation:**
```tsx
<header>
  <BudgetTicker 
    sessionCost={0.12} 
    monthlyCost={3.47}
    onClick={() => setShowCostPanel(true)}
  />
</header>
```

### 2.4 Phase 2 Deliverables

| Deliverable | Status |
|------------|--------|
| Process Group component | NEW |
| Process Step badges | NEW |
| Inline confirmation gates | ENHANCE |
| TOTP input for T3 | NEW |
| Budget ticker in topbar | NEW |
| Cost panel (slide-out) | NEW |

---

## Phase 3: Configuration Surfaces (Weeks 6-10)

### 3.1 Full-Page Settings

**Current:** Modal  
**Target:** Full-page with schema-driven form

**Files to modify:**
- `ui/src/components/settings/Settings.tsx` - Convert to full-page
- Create `ui/src/components/settings/SettingsNav.tsx`

**Sections:**
- Models & providers
- Cost & budget caps
- Security (permission tiers, TOTP setup)
- Notifications
- System (updates, diagnostics)

### 3.2 Channel Management

**Current:** None  
**Target:** Card grid with status + inline QR flow

**Files to create:**
- `ui/src/components/channels/ChannelList.tsx`
- `ui/src/components/channels/ChannelCard.tsx`
- `ui/src/components/channels/ChannelOnboarding.tsx`

**Channel types to support:**
- Slack
- Discord
- Telegram
- WhatsApp
- Email
- REST API

### 3.3 Skills Marketplace

**Current:** Basic list  
**Target:** Cards with tier badges, install flow

**Files to modify:**
- `ui/src/components/skills/Skills.tsx` - Enhanced grid
- `ui/src/components/skills/SkillCard.tsx` - Add tier badge
- Add filter sidebar (tier/source/status)

### 3.4 Memory Viewer

**Current:** Basic viewer  
**Target:** 3-tab tiered view (Session/Project/Long-term)

**Files to modify:**
- `ui/src/components/memory/MemoryViewer.tsx` - Add tabs
- Add search (full-text + semantic)

### 3.5 Phase 3 Deliverables

| Deliverable | Status |
|------------|--------|
| Full-page Settings | ENHANCE |
| Channel management panel | NEW |
| Skills marketplace with tier badges | ENHANCE |
| 3-tab Memory viewer | ENHANCE |

---

## Phase 4: Workflows & Advanced (Weeks 11-14)

### 4.1 Workflows Screen

**Files to create:**
- `ui/src/components/workflows/WorkflowList.tsx`
- `ui/src/components/workflows/WorkflowEditor.tsx`
- `ui/src/components/workflows/TemplateLibrary.tsx`

**Features:**
- Scheduled workflows (cron)
- One-shot planned tasks
- Run history per workflow

### 4.2 Audit Log Screen

**Files to create:**
- `ui/src/components/audit/AuditLog.tsx`

**Features:**
- Filterable table (tier, date, action)
- Expandable rows with full details
- CSV export

### 4.3 Decision Journal

**Files to create:**
- `ui/src/components/chat/DecisionJournal.tsx`

**Features:**
- Per-task reasoning log
- What was considered
- What was rejected

### 4.4 Phase 4 Deliverables

| Deliverable | Status |
|------------|--------|
| Workflows screen | NEW |
| Audit log screen | NEW |
| Decision journal panel | NEW |
| Horizon Scanner status | NEW |

---

## Phase 5: Polish & Accessibility (Weeks 15-16)

### 5.1 Responsive Design

**Breakpoints to implement:**
- Desktop (≥1200px): Full 3-column
- Tablet (768-1199px): Collapsed nav, tabbed task sidebar
- Mobile (<768px): Single column, bottom nav

### 5.2 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd/Ctrl+K | Command palette |
| Cmd/Ctrl+/ | Focus message input |
| Cmd/Ctrl+1-8 | Jump to nav section |
| Esc | Collapse panels |

### 5.3 Accessibility

- Focus rings on all interactive elements
- aria-labels on badges
- Screen reader live regions
- WCAG AA contrast

### 5.4 Dark/Light Mode

- Toggle in topbar
- OS preference detection
- Store in localStorage

---

## Technology Updates

### Add to package.json

```json
{
  "dependencies": {
    "@tanstack/react-query": "^5.0.0",
    "socket.io-client": "^4.7.0",
    "framer-motion": "^11.0.0",
    "@radix-ui/react-dialog": "^1.0.0",
    "@radix-ui/react-dropdown-menu": "^1.0.0",
    "@radix-ui/react-tabs": "^1.0.0",
    "@radix-ui/react-tooltip": "^1.0.0",
    "react-hot-toast": "^2.4.0"
  }
}
```

### Deprecate

- Direct `fetch()` calls → React Query hooks
- setInterval polling → WebSocket + React Query
- Modal settings → Full-page settings

---

## File Change Summary

### New Files (Phase 1-2)
- `ui/src/lib/websocket.ts`
- `ui/src/components/chat/TaskSidebar.tsx`
- `ui/src/components/chat/ProcessGroup.tsx`
- `ui/src/components/chat/ProcessStep.tsx`
- `ui/src/components/chat/ConfirmationGate.tsx`

### New Files (Phase 3)
- `ui/src/components/settings/SettingsNav.tsx`
- `ui/src/components/channels/ChannelList.tsx`
- `ui/src/components/channels/ChannelCard.tsx`
- `ui/src/components/skills/SkillCard.tsx`

### New Files (Phase 4)
- `ui/src/components/workflows/WorkflowList.tsx`
- `ui/src/components/workflows/WorkflowEditor.tsx`
- `ui/src/components/audit/AuditLog.tsx`
- `ui/src/components/chat/DecisionJournal.tsx`

### Major Modifications
- `ui/src/App.tsx` - Header, layout
- `ui/src/stores/appStore.ts` - WebSocket state, costs
- `ui/src/components/chat/Chat.tsx` - Process groups, task sidebar
- `ui/src/components/ui/ConfirmationModal.tsx` - Inline + TOTP
- `ui/src/components/settings/Settings.tsx` - Full-page
- `ui/src/components/memory/MemoryViewer.tsx` - 3 tabs
- `ui/src/components/skills/Skills.tsx` - Enhanced grid

---

## Testing Strategy

### Unit Tests
- ProcessGroup rendering
- ConfirmationGate logic
- WebSocket reconnection

### Integration Tests
- WebSocket → Store flow
- Task creation → Task sidebar update

### E2E Tests (Playwright)
- Full chat → task → confirmation flow
- Settings navigation

---

## Dependencies on Backend

### Required API Endpoints

| Endpoint | Phase | Purpose |
|----------|-------|---------|
| WebSocket /api/v1/ws | 1 | Real-time task updates |
| GET /api/v1/tasks/:id | 1 | Task details |
| POST /api/v1/tasks/:id/confirm | 2 | Confirm task |
| POST /api/v1/totp/setup | 2 | Generate TOTP secret |
| POST /api/v1/totp/verify | 2 | Verify TOTP token |
| GET /api/v1/metrics | 2 | Budget/cost data |
| POST /api/v1/channels | 3 | Connect channel |
| GET /api/v1/memory | 3 | Memory entries |

---

## Success Criteria

### Phase 1 (Week 2)
- [ ] WebSocket connects and receives task updates
- [ ] Connection status shows 3 states
- [ ] Task sidebar shows active tasks
- [ ] No polling in Chat component

### Phase 2 (Week 5)
- [ ] Process groups render with badges
- [ ] T3 confirmation requires TOTP
- [ ] Budget ticker shows live cost

### Phase 3 (Week 10)
- [ ] Settings is full-page
- [ ] Channels display with status
- [ ] Skills show tier badges
- [ ] Memory has 3 tabs

### Phase 4 (Week 14)
- [ ] Workflows screen functional
- [ ] Audit log displays entries
- [ ] Decision journal per task

### Phase 5 (Week 16)
- [ ] Responsive on tablet/mobile
- [ ] Keyboard shortcuts work
- [ ] Dark/light mode toggles

---

*This plan aligns with the APEX_GUI_Design_Specification.md and addresses the architectural critiques from the security audit.*
