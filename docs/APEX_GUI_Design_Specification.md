# APEX GUI Design Specification
### v1.0 — Based on analysis of OpenClaw Control UI and Agent Zero WebUI

---

## Table of Contents

1. [Parent UI Analysis](#1-parent-ui-analysis)
2. [Synthesis — What to Take, What to Drop](#2-synthesis--what-to-take-what-to-drop)
3. [Design Principles](#3-design-principles)
4. [Information Architecture](#4-information-architecture)
5. [Layout & Shell](#5-layout--shell)
6. [Component Specifications](#6-component-specifications)
7. [Interaction Patterns](#7-interaction-patterns)
8. [Visual Design System](#8-visual-design-system)
9. [Real-Time Communication Model](#9-real-time-communication-model)
10. [Responsive & Accessibility Constraints](#10-responsive--accessibility-constraints)
11. [Technology Recommendation](#11-technology-recommendation)
12. [Screen-by-Screen Specifications](#12-screen-by-screen-specifications)
13. [What APEX Introduces That Neither Parent Has](#13-what-apex-introduces-that-neither-parent-has)
14. [Implementation Priority Order](#14-implementation-priority-order)

---

## 1. Parent UI Analysis

### 1.1 OpenClaw Control UI

**Technology:** Vite + Lit, single-page app, WebSocket-first against the Gateway WS on port 18789. Token-based device pairing on first connect. CSS custom properties for theming.

**What it does well:**

- **Channel management as a first-class surface.** The UI dedicates a full panel to channel status, QR login flows, per-channel config, and presence. Connecting WhatsApp or Telegram is a guided flow with live feedback, not buried in settings JSON. This dramatically lowers onboarding friction.
- **Skill marketplace with install gating.** Skills have a dedicated UI surface: status, enable/disable, API key requirements surfaced inline, install via ClawHub. Users know what capabilities the system has and can extend it without touching the CLI.
- **Cron/scheduler panel.** A fully-featured cron job UI with add/edit/run/enable/disable, run history, per-job delivery mode (announce/webhook/none), advanced options (stagger, model override, thinking override), and inline validation with field-level errors. This is a mature, complete feature.
- **Config as a form, not raw JSON.** The settings panel renders `~/.openclaw/openclaw.json` as a validated schema-driven form with plugin and channel schemas included. Raw JSON editor is available as an escape hatch but is not the primary path. This is the right decision.
- **Live log tailing with filter and export.** Debug logs are surfaced in the UI with live tail, not requiring terminal access. Essential for non-developer users.
- **Exec approvals UI.** Gateway and node exec allowlists are editable in-browser, with ask/deny policy controls. Meaningful security surface made accessible.
- **Update flow in-browser.** `update.run` with restart and a restart report. Users don't need CLI access to keep the system current.

**What it does poorly:**

- **Chat is secondary, not primary.** WebChat in OpenClaw is a capability listed among many panels, not the central surface. The UI is primarily a control panel for a messaging agent — the chat interface itself is thin. It has no image upload (known open issue #16152 as of Feb 2026), no rich attachment preview, no streaming token rendering.
- **No visibility into task execution.** There is no way to see what the agent is currently doing, what tools it is calling, or what decisions it made. The agent is a black box beyond the message log.
- **No process/agent introspection.** Agent Zero's process groups — collapsible execution traces with GEN/USE/EXE/WWW badges — have no equivalent. If something goes wrong, there is no execution trace to inspect.
- **Diagnostic tools are buried.** Status/health/models snapshots and manual RPC calls exist but require knowing to look for them. The debug panel is power-user territory.
- **Memory is not a visible surface.** Memory exists in OpenClaw (LanceDB support, FTS search, compaction) but is not surfaced as a manageable UI object. Users cannot see what the agent remembers.

**Overall character:** A control plane for an always-on daemon. Optimised for configuration, channel management, and monitoring. Chat is a feature, not the product.

---

### 1.2 Agent Zero WebUI

**Technology:** Alpine.js 3 for reactive state, Bootstrap 5 for component logic, Marked.js for markdown, KaTeX for math, Ace Editor in settings, Flask serving static assets and API. Socket.IO WebSocket primary, HTTP polling fallback (250ms). Session-based auth with CSRF protection. Custom `<x-component>` fragment loader for modularity.

**What it does well:**

- **Process groups with execution transparency.** This is Agent Zero's most distinctive and valuable UI contribution. Every agent action is rendered in a collapsible process group with type-specific badges: GEN (reasoning), USE (tool call), EXE (code execution), WWW (browser), MCP (MCP tool), SUB (subagent). Users see exactly what the agent did, in what order, with expandable detail for each step. Completed groups collapse automatically, preserving a clean history while keeping detail accessible.
- **Multi-context chat (parallel agents).** The left sidebar lists active chat contexts. Users can run multiple agents simultaneously and switch between them. Context switching triggers a full state resync from the backend. This is a powerful capability with a simple, clear UI.
- **Process group incremental rendering.** The message system updates DOM in-place rather than re-rendering. KVP tables within process steps are updated incrementally. Scroll position is preserved during updates. This makes watching a running agent feel live and smooth rather than jumpy.
- **Memory Management Dashboard.** A dedicated surface for viewing, searching, and managing agent memories. Introduced in v0.9.6. Makes the vector store human-inspectable rather than invisible.
- **Projects panel.** Per-project custom instructions, integrated memory and knowledge per project, project-specific secrets. Projects are a first-class UI concept, not just file folders.
- **Scheduler UI.** Three task types (scheduled/cron, planned/one-shot, triggered), full CRUD, Flatpickr datetime picker, run history. Similar capability to OpenClaw's cron panel but scoped to the execution context.
- **Settings as schema-driven form.** Multiple field types (text, number, password, textarea, switch, range, select, button, html), two-way Alpine.js binding, save/reset flow. Clean, extensible settings architecture.
- **Attachment system.** Drag-and-drop file uploads, 120×120px preview tiles with hover states and remove buttons, full-screen drop overlay. Image and file attachment rendered in message history.
- **Voice input and TTS.** Whisper-based speech-to-text and browser TTS output. Local voice mode.

**What it does poorly:**

- **No channel management.** Agent Zero has no concept of messaging channels. It is a local web UI only. There is no path from "I want to interact via WhatsApp" without entirely custom integration.
- **No skill marketplace.** Skills exist (the Skills system, v0.9.8) but there is no marketplace, no install-from-registry flow, no community skill discovery. Skills are local files.
- **Auth is MD5 credentials hash.** `login.get_credentials_hash()` uses MD5. This is not acceptable for anything beyond localhost. The auth model assumes local-only deployment.
- **Chat is a single thread per context.** You cannot have multiple conversations within the same project/context. The context IS the conversation.
- **No permission tier UI.** Agent Zero has no concept of T0–T3 confirmation gates. Everything executes immediately or with basic user acknowledgement.
- **No channel-specific response formatting.** All output is markdown for the web UI. There is no concept of formatting a result differently for Slack vs. WhatsApp vs. email.
- **Settings are a modal.** The full configuration surface is a modal dialog over the chat. For a complex system, this creates a cramped UX.

**Overall character:** A transparent execution cockpit. Optimised for watching an agent work, understanding what it did, and managing its memory and projects. Everything is visible and inspectable. The chat IS the product.

---

## 2. Synthesis — What to Take, What to Drop

### Take from OpenClaw

| Feature | Why |
|---------|-----|
| Channel management as a primary surface | APEX is multi-channel; onboarding and channel health must be first-class |
| Skill marketplace with install gating | 5,700+ ClawHub skills; discovery and install UX matters |
| Config-as-validated-form (not raw JSON) | Reduces misconfiguration; plugin/channel schemas render inline |
| Cron/scheduler panel | Workflow templates and horizon scanner need a scheduling UI |
| Exec approvals UI | Maps to APEX's permission tier model |
| In-browser update flow | Operational accessibility |
| Live log tail with filter/export | Replaces terminal access for non-developers |
| Device pairing model | Cleaner than session cookies for remote access |

### Take from Agent Zero

| Feature | Why |
|---------|-----|
| Process groups with type-specific badges | APEX's Deep tasks need execution transparency; T2/T3 users must understand what ran |
| Multi-context parallel agents | APEX supports concurrent Deep tasks; sidebar list of active tasks is essential |
| Incremental DOM updates for live rendering | Smooth streaming experience for long-running tasks |
| Memory Management Dashboard | APEX has a tiered memory model; users must be able to inspect and delete memories |
| Projects as first-class UI concept | Maps directly to APEX's project memory tier |
| Attachment system (drag-and-drop, preview tiles) | Chat-primary UX needs rich attachment handling |
| Schema-driven settings form (not modal) | APEX's configuration surface is larger; needs full-page treatment |
| Scheduler with run history | Core to workflow templates |

### Drop from OpenClaw

| Feature | Why |
|---------|-----|
| Chat as a secondary panel | APEX chat is the primary surface |
| Manual RPC call panel | Too low-level; replaced by structured debug views |
| Terminal-style log viewer as the primary debug tool | Replaced by structured execution traces |

### Drop from Agent Zero

| Feature | Why |
|---------|-----|
| MD5 auth | APEX has PASETO tokens and TOTP; this is replaced entirely |
| Settings as a modal | APEX settings are too large for a modal |
| Single-thread-per-context model | APEX supports multi-task contexts |
| HTTP polling as primary comms | APEX uses WebSocket + NATS; polling is only a fallback |

---

## 3. Design Principles

These seven principles govern every design decision. When two options conflict, the higher-numbered principle wins.

**P1 — Chat is the product, not a feature.**
The conversation interface is the primary surface. Every other panel is secondary. Navigation should always allow returning to chat in one action.

**P2 — Transparency without noise.**
Users should be able to see everything the agent did. But they should not be forced to. Execution details collapse by default; expand on demand. Completed work retreats; active work surfaces.

**P3 — Confirmation gates are UI, not pop-ups.**
T1–T3 confirmation is a first-class interaction, not an interruption modal pasted over the interface. It must be contextual, visible, and non-dismissable by accident.

**P4 — System state is always visible.**
Connection status, active task count, VM pool utilisation, and budget consumption are always visible in the shell — not buried in a diagnostic panel.

**P5 — Channel-native, not channel-oblivious.**
The UI must surface that APEX is a multi-channel system. Channel health, connected platforms, and per-channel status are surfaced in navigation, not hidden in settings.

**P6 — Configuration is a form, not a file.**
No user should need to edit JSON manually for standard configuration. JSON is the escape hatch, not the path.

**P7 — Security decisions are explicit, visible, and attributable.**
Every T2/T3 action shows the full action plan before execution. Every executed action is traceable to the user and timestamp. The audit log is user-accessible, not ops-only.

---

## 4. Information Architecture

```
APEX
├── Chat                          (primary surface)
│   ├── Active conversation
│   ├── Task context switcher     (sidebar list of active tasks)
│   ├── Process group viewer      (execution trace per Deep task)
│   └── Confirmation gates        (T1/T2/T3 inline)
│
├── Tasks                         (kanban / task list)
│   ├── Active tasks
│   ├── Completed tasks
│   └── Failed / timed-out tasks
│
├── Workflows                     (scheduler + templates)
│   ├── Scheduled workflows
│   ├── Templates library
│   └── Run history
│
├── Skills                        (marketplace + registry)
│   ├── Installed skills
│   ├── ClawHub browse/install
│   └── Promoted (generated) skills
│
├── Memory                        (tiered memory viewer)
│   ├── Session memory
│   ├── Project memory
│   └── Long-term memory
│
├── Channels                      (connection management)
│   ├── Connected channels + health
│   ├── Add / configure channel
│   └── Per-channel settings
│
├── Projects                      (project isolation)
│   ├── Project list
│   ├── Project context (memory, files, settings)
│   └── Secrets per project
│
├── Audit Log                     (security & transparency)
│   ├── Task audit trail
│   ├── T2/T3 action log
│   └── Export
│
└── Settings                      (full-page, schema-driven)
    ├── Models & providers
    ├── Cost & budget
    ├── Security (permission tiers, exec approvals)
    ├── Notifications
    └── System (updates, diagnostics)
```

---

## 5. Layout & Shell

### 5.1 Primary Shell Layout

```
┌────────────────────────────────────────────────────────────────────────┐
│ TOPBAR                                                                  │
│ [≡ APEX]  [● Connected]  [2 tasks running]  [Budget: $0.12]  [⚙ ●]   │
├──────────┬─────────────────────────────────────────────────────────────┤
│          │                                                              │
│ LEFT     │  MAIN CONTENT AREA                                          │
│ NAV      │                                                              │
│          │  (Chat / Tasks / Workflows / Skills / Memory /               │
│ Chat     │   Channels / Projects / Audit / Settings)                   │
│ Tasks    │                                                              │
│ Workflows│                                                              │
│ Skills   │                                                              │
│ Memory   │                                                              │
│ Channels │                                                              │
│ Projects │                                                              │
│ Audit    │                                                              │
│ ─────    │                                                              │
│ Settings │                                                              │
│          │                                                              │
└──────────┴─────────────────────────────────────────────────────────────┘
```

**Topbar** — always visible, 48px height:
- Left: APEX wordmark + hamburger (collapses left nav on small viewports)
- Centre-left: Connection status pill (green/amber/red dot + "Connected / Degraded / Disconnected")
- Centre: Active task count badge — "2 tasks running" — clicking jumps to Tasks view
- Centre-right: Budget ticker — live cumulative cost for the current billing period
- Right: Notification bell with unread count + Settings cog with a dot when there are pending updates

**Left nav** — 200px wide, collapsible to 48px icon-only rail:
- Icon + label for each primary section
- Chat section shows a count badge for unread responses
- Channels section shows a dot per disconnected channel
- Settings is visually separated at the bottom with a rule

**Main content area** — fills remaining space, scrollable per section.

### 5.2 Chat Layout (Primary Surface)

```
┌──────────┬──────────────────────────────────┬──────────────────────┐
│ LEFT NAV │  CHAT MAIN                       │  TASK SIDEBAR        │
│          │                                  │                      │
│          │  ┌──────────────────────────┐   │  Active Tasks (3)    │
│          │  │  Process Group           │   │  ──────────────────  │
│          │  │  [▶ Collapsed — 4 steps] │   │  ● deep-task-001     │
│          │  │                          │   │    code refactor      │
│          │  │  ● User message          │   │    Running 00:42      │
│          │  │                          │   │                      │
│          │  │  ┌────────────────────┐  │   │  ● deep-task-002     │
│          │  │  │ Process Group      │  │   │    web research       │
│          │  │  │ [▼ Expanded]       │  │   │    Waiting confirm   │
│          │  │  │  GEN: reasoning    │  │   │                      │
│          │  │  │  USE: web_search   │  │   │  ✓ skill-task-003    │
│          │  │  │  EXE: python run   │  │   │    email draft        │
│          │  │  │  ─────             │  │   │    Completed 14:23   │
│          │  │  │  Response ●        │  │   │                      │
│          │  │  └────────────────────┘  │   │  ──────────────────  │
│          │  │                          │   │  Decision Journal    │
│          │  │  [T2 CONFIRM GATE]       │   │  (last task)         │
│          │  │                          │   │                      │
│          │  └──────────────────────────┘   │                      │
│          │                                  │                      │
│          │  ┌──────────────────────────┐   │                      │
│          │  │ ○ Type a message...  [↑] │   │                      │
│          │  │ [📎] [🎤]                │   │                      │
│          │  └──────────────────────────┘   │                      │
└──────────┴──────────────────────────────────┴──────────────────────┘
```

The task sidebar (right, 280px) is collapsible. It lists all active, pending, and recently completed tasks with status, elapsed time, and one-click jump to the relevant process group. This directly addresses a gap in both parents: OpenClaw has no task list, Agent Zero's task list is in the left sidebar competing with context switching.

---

## 6. Component Specifications

### 6.1 Process Group (from Agent Zero, enhanced)

The process group is the most important rendering component in APEX. It wraps every Deep task execution, making the agent's work inspectable without being intrusive.

**States:**
- `running` — group has a pulsing left border in accent colour; steps appear in real time
- `awaiting_confirmation` — group has an amber left border; confirmation gate rendered inside
- `completed` — group collapses to a one-line summary; expandable
- `failed` — group has a red left border; error message inline; expand shows full trace
- `timed_out` — group has a grey left border; partial result displayed

**Structure (expanded):**
```
┌─ Process Group ──────────────────────────────────────────────────┐
│ ▼  Refactor auth module   •  4 steps  •  00:23  •  $0.008       │
├──────────────────────────────────────────────────────────────────┤
│  GEN  Planning approach                              [▶ expand]  │
│  USE  tool: repo.search  query="auth module"         [▶ expand]  │
│  EXE  python: ast_parser.py                          [▶ expand]  │
│  GEN  Writing refactored code                        [▶ expand]  │
├──────────────────────────────────────────────────────────────────┤
│  ● Response                                                       │
│  Here is the refactored auth module. I replaced the MD5...       │
└──────────────────────────────────────────────────────────────────┘
```

**Step badges (from Agent Zero, mapped to APEX skill types):**

| Badge | Colour | Meaning |
|-------|--------|---------|
| GEN | Blue | LLM generation / reasoning step |
| USE | Teal | Skill invocation (L4) |
| EXE | Amber | Code execution in VM (L5) |
| WWW | Purple | Browser / web access |
| SUB | Indigo | Subagent spawned |
| MEM | Green | Memory read or write |
| AUD | Red | Audit event (T2/T3 action logged) |

**Expandable step detail:**
Each step expands to show: input parameters (KVP table), output/result, timing, cost contribution, and (for GEN steps) the full reasoning text.

**Decision Journal toggle:**
A `[📖 Journal]` button on the process group header opens the Decision Journal panel for that task — showing the structured reasoning log from L3 (what options were considered, what was rejected, key assumptions).

### 6.2 Confirmation Gate (APEX-specific, built into chat)

The confirmation gate is not a modal. It renders inline in the chat stream, blocking further execution of that task until resolved.

**T1 — Tap:**
```
┌──────────────────────────────────────────────────────────────────┐
│ ℹ  This action will create a file in your workspace.             │
│    /workspace/src/auth/auth.py  (new file, 847 bytes)            │
│                                                                  │
│    [  Confirm  ]    [ Cancel ]                                   │
└──────────────────────────────────────────────────────────────────┘
```

**T2 — Type:**
```
┌──────────────────────────────────────────────────────────────────┐
│ ⚠  This action will push to a remote repository.                │
│    Remote: origin/main  •  3 commits  •  +847 -23 lines          │
│    Skill: git.commit                                             │
│                                                                  │
│    Type "push to main" to confirm:                               │
│    [                                    ]                        │
│                                                                  │
│    [ Confirm ]    [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────┘
```

**T3 — Authenticate + Review:**
```
┌──────────────────────────────────────────────────────────────────┐
│ 🔴  ELEVATED ACTION — Full review required                       │
│                                                                  │
│    Skill: deploy.kubectl                                         │
│    Target: production cluster / namespace: api                   │
│    Action: rolling update  apex-router:v1.2.3 → v1.2.4          │
│    Estimated impact: ~30s downtime, 0 pod restarts               │
│                                                                  │
│    [ View full action plan ]                                     │
│                                                                  │
│    Enter TOTP code:  [ _ _ _ _ _ _ ]                            │
│                                                                  │
│    [ Confirm ]    [ Cancel ]                                     │
└──────────────────────────────────────────────────────────────────┘
```

The T3 gate shows the full action plan (expandable) and requires TOTP entry in-browser. Clicking Confirm without a valid TOTP code does nothing — the button is disabled until a 6-digit code is entered. The task remains in `awaiting_confirmation` state until resolved or cancelled.

### 6.3 Channel Card (from OpenClaw, adapted)

Each connected channel renders as a status card in the Channels panel:

```
┌─ Slack ──────────────────────────────────────────────────────────┐
│  ● Connected   workspace: acme-corp   bot: APEX                  │
│  Last message: 2 minutes ago                                     │
│  [Configure]  [Disconnect]                                       │
└──────────────────────────────────────────────────────────────────┘

┌─ WhatsApp ────────────────────────────────────────────────────────┐
│  ○ Not connected                                                  │
│  [Connect — scan QR code]                                        │
└──────────────────────────────────────────────────────────────────┘
```

QR login for WhatsApp and Telegram renders inline in the card (not a new page or modal) — directly from OpenClaw's approach.

### 6.4 Skill Card (from OpenClaw, enhanced with tier display)

```
┌─ shell.execute ───────────────────────────────────────── T3 ─────┐
│  Execute shell commands in the workspace VM                      │
│  ● Enabled    Health: OK    v1.2.0                               │
│  [Disable]  [Details]                                            │
└──────────────────────────────────────────────────────────────────┘

┌─ music.generate ──────────────────────────────────────── T1 ─────┐
│  Generate music from text prompt                                 │
│  ○ Not installed    Source: ClawHub                              │
│  Requires: SUNO_API_KEY                                          │
│  [Install]                                                       │
└──────────────────────────────────────────────────────────────────┘
```

The tier badge (T0/T1/T2/T3) is always visible on the skill card. Users understand the risk class before installing.

### 6.5 Memory Viewer (from Agent Zero Memory Dashboard, extended for APEX tiers)

Three tabs: Session / Project / Long-term.

Each memory entry shows: content preview, source task ID (linked to process group), creation timestamp, TTL remaining, and a Delete button. Search is full-text + semantic (vector) — the search bar runs both in parallel and shows ranked results.

The Long-term tab shows a `Used in N tasks` count per memory entry — helping users understand which memories are actively influencing agent behaviour.

### 6.6 Budget Ticker & Cost Panel

The topbar budget ticker is a live-updating number. Clicking it opens a slide-out panel:

```
┌─ Cost & Budget ─────────────────────────────────────────────────┐
│  This session:    $0.12  /  $5.00 soft cap                      │
│  This month:      $3.47  /  $20.00 hard cap                     │
│                                                                  │
│  Recent tasks:                                                   │
│  deep-task-001  code refactor    $0.031  (GPT-4o)               │
│  deep-task-002  web research     $0.009  (GPT-4o-mini)          │
│  skill-task-003 email draft      $0.002  (skill)                │
│                                                                  │
│  Cost profile:  [● Balance]  [ Minimise ]  [ Quality ]          │
│                                                                  │
│  [Edit caps]                                                     │
└─────────────────────────────────────────────────────────────────┘
```

This directly surfaces the Cost Intelligence layer described in the architecture spec. Both parents have no equivalent.

---

## 7. Interaction Patterns

### 7.1 Message Input Bar

Consistent with Agent Zero's chat bar, extended:

- Auto-expanding textarea (1–8 lines, then scrolls)
- `Enter` sends; `Shift+Enter` newlines
- Paperclip icon opens file picker; drag-and-drop anywhere on the chat surface activates a full-screen drop zone (from Agent Zero)
- Microphone icon for voice input (Whisper-backed, from Agent Zero)
- `@skill` autocomplete — type `@` to invoke a skill directly from the input bar
- `/stop` or click Stop button aborts the current agent run (from OpenClaw's abort phrases)
- Attachment preview bar appears above input when files are queued: 80×80px tiles with hover-remove

### 7.2 Task Switching

The right sidebar task list supports:
- Click a task → scroll the chat to its process group and expand it
- Right-click a task → context menu: Cancel / Retry / Copy task ID / View in Audit Log
- Drag to reorder priority queue (future)

### 7.3 Channel Onboarding Flow

First-time users with no channels connected see a focused onboarding surface instead of the normal Chat view:

```
Welcome to APEX

Connect your first channel to start sending messages:

  [Slack]  [Discord]  [Telegram]  [WhatsApp]  [Email]  [REST API]

Or start with the web interface →
```

Clicking a channel card opens an inline guided connection flow (auth, token, QR scan). No modals, no page navigation.

### 7.4 Keyboard Navigation

| Shortcut | Action |
|----------|--------|
| `⌘K` / `Ctrl+K` | Command palette (search tasks, skills, memory) |
| `⌘/` | Focus message input |
| `⌘1`–`⌘8` | Jump to nav section by position |
| `Esc` | Collapse expanded process group / close slide-out panel |
| `⌘↑/↓` | Navigate between tasks in the sidebar |

### 7.5 Streaming Updates

All message content streams token-by-token. The rendering strategy mirrors Agent Zero's incremental DOM update approach: containers are updated in-place, not recreated. KVP tables within process steps update incrementally. The chat surface autoscrolls to the bottom unless the user has scrolled up, in which case scroll position is preserved (from Agent Zero's Scroller class logic).

---

## 8. Visual Design System

### 8.1 Colour Palette

```
Background (primary):     #0F1117   (near-black, dark default)
Background (secondary):   #1A1D27   (panels, cards)
Background (tertiary):    #22273A   (input fields, hover states)
Border:                   #2E3348   (subtle separators)

Text (primary):           #E8EAF0   (main content)
Text (secondary):         #8B92A8   (labels, metadata)
Text (disabled):          #4A5068

Accent (primary):         #5B8DEF   (primary actions, links, active states)
Accent (success):         #34C77B   (connected, completed, T0)
Accent (warning):         #F5A623   (running, pending, T1)
Accent (danger):          #E85555   (failed, T3 actions, destructive)
Accent (info):            #7C6CF5   (T2 actions, subagent)

Badge colours (process steps):
  GEN:   #5B8DEF  (blue)
  USE:   #2EB8A0  (teal)
  EXE:   #F5A623  (amber)
  WWW:   #9B6CF5  (purple)
  SUB:   #5B7CF5  (indigo)
  MEM:   #34C77B  (green)
  AUD:   #E85555  (red)
```

Light mode inverts the background stack (white → light grey → mid grey) and adjusts text to near-black. All colour assignments remain semantically consistent across modes.

### 8.2 Typography

```
UI font:       Inter (system fallback: -apple-system, sans-serif)
Mono font:     JetBrains Mono (code blocks, terminal output, task IDs)

Size scale:
  xs:   11px   (badges, metadata, timestamps)
  sm:   13px   (secondary labels, sidebar items)
  md:   15px   (body text, chat messages) — base
  lg:   17px   (panel headings)
  xl:   21px   (section titles)
  2xl:  27px   (page titles, empty state headings)

Weight:
  Regular:  400   (body)
  Medium:   500   (labels, nav items)
  Semibold: 600   (headings, badges)
  Bold:     700   (confirmation gate titles, alerts)
```

### 8.3 Spacing & Radius

Base unit: 4px. All spacing is multiples of 4px (4, 8, 12, 16, 20, 24, 32, 48).

Border radius:
- Cards, panels: 8px
- Buttons: 6px
- Input fields: 6px
- Badges: 4px
- Pills (status indicators): 100px (fully rounded)

### 8.4 Motion

All transitions use `ease-out` at 150ms for micro-interactions (hover, focus, badge appearance) and 250ms for layout changes (panel expand/collapse, process group expand).

Process group running state: left border pulse animation, 1.5s period, opacity 0.6→1.0.

No decorative animations. Motion communicates state change, not personality.

### 8.5 Dark / Light Mode

Toggle in topbar (⊙ icon) and in Settings. Mode preference stored in Long-term Memory (user preference). Respects OS prefers-color-scheme on first load.

---

## 9. Real-Time Communication Model

The APEX UI uses a WebSocket-first communication model backed by the NATS JetStream infrastructure. This differs from both parents and must be explicitly designed.

### 9.1 Connection States (visible in topbar)

| State | Topbar indicator | Behaviour |
|-------|-----------------|-----------|
| Connected | ● green "Connected" | Full real-time push via WebSocket |
| Degraded | ● amber "Degraded" | HTTP polling fallback at 500ms (slower than Agent Zero's 250ms to reduce server load) |
| Reconnecting | ● amber "Reconnecting…" | Automatic exponential backoff |
| Disconnected | ● red "Disconnected" | No comms; retry button shown |

### 9.2 Task State Streaming

Task updates stream from L2 via WebSocket. Each update carries:
- Task ID
- Status transition (RECEIVED → CLASSIFIED → EXECUTING → COMPLETED etc.)
- New process step (if executing)
- Partial result content (for streaming LLM output)
- Budget delta

The UI applies updates incrementally. A task that transitions to `COMPLETED` causes its process group to collapse after a 1-second delay (enough time for the user to see the final state before it collapses).

### 9.3 Reconnection Strategy

On WebSocket disconnect:
1. Immediately switch to HTTP polling (500ms interval)
2. Attempt WebSocket reconnect after 2s, then 4s, 8s, 16s (capped at 30s)
3. On successful reconnect, request a full state snapshot to reconcile any missed updates
4. Switch back to WebSocket push, stop polling

---

## 10. Responsive & Accessibility Constraints

### 10.1 Breakpoints

| Name | Width | Layout change |
|------|-------|---------------|
| Desktop | ≥ 1200px | Full 3-column (nav + main + task sidebar) |
| Tablet | 768–1199px | Nav collapses to icon rail; task sidebar collapses to a tab |
| Mobile | < 768px | Single-column; nav becomes bottom tab bar; task sidebar becomes a drawer |

Mobile is not a primary target but must not be broken. The confirmation gate must be fully usable on mobile — no confirmation should require hover or keyboard-only interactions.

### 10.2 Accessibility

- All interactive elements have visible focus rings (2px solid accent colour, 2px offset)
- Colour is never the only differentiator — every status has an icon or label in addition to colour
- Process group badges include `aria-label` attributes ("LLM generation step", not just "GEN")
- Confirmation gates are focussed automatically when they appear; Escape does not dismiss T2/T3 gates
- WCAG AA contrast minimum enforced for all text and interactive elements
- Keyboard navigation covers all surfaces (see Section 7.4)
- Screen reader live regions for task status changes and new messages

---

## 11. Technology Recommendation

The existing APEX UI is React 18 + TypeScript + Tailwind + Zustand. This is a reasonable stack. The following recommendations apply:

**Keep:**
- React 18 with concurrent features (enables streaming/suspense patterns for live task updates)
- TypeScript throughout
- Zustand for client state (lightweight, no boilerplate)
- Tailwind for utility-first styling (but enforce a design token layer over raw Tailwind classes to maintain visual consistency)

**Add:**
- `@tanstack/react-query` for server state (task list, skill registry, memory entries) — replaces manual polling logic with declarative, cache-aware data fetching
- `socket.io-client` for WebSocket (consistent with Agent Zero's approach; well-tested fallback behaviour)
- `react-markdown` + `react-syntax-highlighter` for message rendering (replaces raw dangerouslySetInnerHTML with Marked.js)
- `@radix-ui/react-*` primitives for accessible modal/popover/dropdown foundations (unstyled, composable, WCAG-compliant)
- `framer-motion` for process group expand/collapse animations (scoped — not for decorative use)
- `react-hot-toast` for the notification stack (replaces custom implementation)

**Do not add:**
- A charting library (Recharts, Chart.js) for this phase — the budget ticker and cost panel are text-based; charts are phase 2
- A drag-and-drop library for kanban — the current KanbanBoard component is sufficient for now; defer until multi-user mode requires it

**Deprecate:**
- Direct `fetch()` calls in components — all API communication goes through React Query hooks
- UI polling loop (the `setInterval` approach shown in the current architecture) — replaced by WebSocket + React Query

---

## 12. Screen-by-Screen Specifications

### 12.1 Chat Screen

**Left nav:** Chat item is active/highlighted. Unread badge shows if messages arrived while away.

**Chat main area:**
- Conversation history renders in reverse-chronological order (newest at bottom, scrolled to bottom on load)
- User messages: right-aligned, rounded bubble, light background
- Agent responses: left-aligned, no bubble, full width up to 720px, markdown rendered
- Process groups: left-aligned, full width, bordered card with left-colour-coded stripe
- System messages (info/warning/error): centered, subdued, smaller text
- Timestamps: shown on hover for all messages; shown inline for messages > 5 minutes from the previous message

**Input bar:**
- Fixed to bottom of chat main
- Textarea auto-expands
- Attachment preview bar above when files queued
- Character count shown at 2000+ characters
- Cost estimate shown for Deep tasks after a 500ms debounce: "~$0.02 estimated"

**Right task sidebar:**
- Collapsible with a `[»]` toggle
- Active tasks show pulsing status dot
- Tasks awaiting confirmation show amber badge "Needs confirm"
- Click task → smooth scroll to process group + auto-expand

### 12.2 Tasks Screen (Kanban)

Three columns: **Running** / **Awaiting Confirmation** / **Completed & Failed**.

Each task card shows: tier badge, skill name or "Deep task", elapsed time, cost, channel source (icon), and a status indicator. Clicking a card expands it to show the full process group (same component as in chat, but in a modal-style panel over the kanban). A "View in Chat" link jumps to the conversation that originated the task.

The Awaiting Confirmation column is sorted to top-of-list and highlighted — it represents work that is blocked on the user.

### 12.3 Workflows Screen

Two tabs: **Scheduled** (recurring cron workflows) and **Templates** (saved reusable workflow definitions).

**Scheduled tab:** Card-per-workflow. Shows: name, cron expression, last run status, next run time, enable/disable toggle, Edit/Delete actions. Run history is expandable inline (last 10 runs, status + cost + duration each). "New Workflow" button opens a side panel with the YAML editor for the workflow DSL, a cron expression builder (GUI + raw), and delivery configuration.

**Templates tab:** Card-per-template. Shows: name, step count, last used, Run Now button. Templates are parameterised — clicking "Run Now" opens a parameter form before dispatch.

### 12.4 Skills Screen

Left: filter sidebar (tier: T0/T1/T2/T3 / source: built-in/ClawHub/generated / status: enabled/disabled/not-installed).

Right: skill cards in a 3-column grid (2 on tablet, 1 on mobile). Each card: skill name, description, tier badge, source, health status, version, and primary action button (Enable/Disable/Install/Details).

"Browse ClawHub" button at top opens a full-page modal with search, category filter, and install flow. Install pulls from the ClawHub registry, shows API key requirements, runs health check, and adds to the registry — all in-UI.

### 12.5 Memory Screen

Three tabs: **Session** / **Project** / **Long-term**.

Each tab: search bar (runs full-text + semantic in parallel), results list with entry cards (content preview, source task, timestamp, TTL badge, Delete button). Long-term tab adds "Used in N tasks" counter. "Export all" and "Delete all" buttons at section level with confirmation gates.

### 12.6 Channels Screen

Grid of channel cards (see Section 6.3). Top section: connected channels. Bottom section: available channels not yet connected. "Add custom channel" link for REST API/webhook configuration.

### 12.7 Audit Log Screen

Table view: timestamp, user, task ID, action, tier, status. Filterable by tier, date range, action type. Each row is expandable: shows full capability token claims, action parameters, and execution outcome. "Export CSV" button at top. Read-only — no actions from this surface.

### 12.8 Settings Screen

Full-page, not a modal. Left: settings section nav (Models, Budget, Security, Notifications, System). Right: form panel for selected section. Same schema-driven form approach as Agent Zero but at full-page width.

**Security section** is the most important: permission tier configuration, exec approval allowlists (from OpenClaw), TOTP setup/reset for T3, connected device management (from OpenClaw's device pairing model).

**System section:** APEX version, update button ("Check for updates" → "Update now" → restart report), diagnostic health check with expandable per-component status, log viewer (live tail, filter, export).

---

## 13. What APEX Introduces That Neither Parent Has

### 13.1 Inline Confirmation Gates

Both parents have no tiered, in-chat confirmation system. OpenClaw's exec approvals are a configuration surface, not an execution gate. Agent Zero has no permission model. APEX's T1–T3 confirmation gates, rendered inline in the conversation stream with full action plan display and TOTP for T3, are entirely novel.

### 13.2 Budget Ticker + Cost Panel

Neither parent surfaces LLM/compute cost in the UI. APEX puts this in the topbar as a persistent ticker and expands to a per-task cost breakdown with cost profile controls. This directly corresponds to the Cost Intelligence architecture layer.

### 13.3 Decision Journal Viewer

The `[📖 Journal]` button on each process group opens the Decision Journal — structured reasoning log per task (what was considered, what was rejected, what assumptions were made). Neither parent captures or surfaces this. Agent Zero shows *what* the agent did; APEX shows *why*.

### 13.4 Multi-Tier Memory Viewer

Agent Zero has a Memory Dashboard for its single memory layer. OpenClaw doesn't surface memory at all. APEX's Memory viewer exposes all four tiers (working memory is not visible — it's transient — but session/project/long-term are) with tier-specific TTL display, semantic search, and source task attribution.

### 13.5 Horizon Scanner Status

If the Horizon Scanner feature is enabled, a `[📡 Scanner]` indicator appears in the topbar next to the budget ticker. Clicking it opens a panel showing: configured data sources, last scan timestamps, recent signals found, and relevance threshold controls. This is entirely novel.

### 13.6 Output Contract Validation Display

When a workflow step fails output contract validation and enters the retry loop, the process group shows a `CONTRACT FAIL` step (in red) followed by the retry attempt. Users see the validation error and can understand why the agent is retrying. This makes schema validation failures visible rather than silent.

---

## 14. Implementation Priority Order

Based on the current state of the APEX codebase (v0.1.0), the following order maximises value while building on what already exists.

**Phase 1 — Stabilise what exists (immediate):**
- Replace polling with WebSocket connection to Router (addresses the architecture critique)
- Implement proper connection state indicator in topbar (connected/degraded/disconnected)
- Build the Task Sidebar (right panel in chat view) — this requires only consuming existing task API endpoints
- Add tier badge to ConfirmationModal (already exists) — T3 must show TOTP field, not just 5-second delay

**Phase 2 — Process transparency (weeks 2–4):**
- Process Group component — the most important new component; map APEX's task execution steps to GEN/USE/EXE/AUD badges
- Decision Journal panel — read from L3 decision event records once they exist
- Budget Ticker in topbar — consume cost data from existing CostEstimator

**Phase 3 — Configuration surfaces (weeks 4–8):**
- Full-page Settings (replace any modal-based settings)
- Channel management panel (card-per-channel, inline QR flow)
- Skill cards with tier badges and install flow (ClawHub integration)
- Memory Viewer (session + project + long-term tabs)

**Phase 4 — Workflow and advanced surfaces (weeks 8–12):**
- Workflows screen (scheduler + templates)
- Audit Log screen (read from L3 audit table)
- Horizon Scanner status panel
- Output contract validation display in process groups

---

*APEX GUI Design Specification · v1.0*
*Based on analysis of: [OpenClaw Control UI](https://docs.openclaw.ai/web/control-ui) · [Agent Zero WebUI](https://deepwiki.com/agent0ai/agent-zero/12-web-user-interface)*
*Architecture references: [OpenClaw](https://github.com/openclaw/openclaw) · [Agent Zero](https://github.com/agent0ai/agent-zero)*
