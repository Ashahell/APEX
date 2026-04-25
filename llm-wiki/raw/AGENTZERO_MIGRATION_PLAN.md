# APEX UI to AgentZero Migration Plan

## Status: ✅ COMPLETE (v1.3.2)

The UI migration to AgentZero styling is complete. All major components have been updated with:
- Indigo (#4248f1) as primary color
- CSS variables for theming
- SVG icons throughout
- Rounded corners (rounded-xl)
- Card-based layouts with borders
- Framer-motion animations

### Additional Features Added (v1.3.2)
- **Toast Notifications**: Full toast system with success/error/warning/info variants
- **Message Reactions**: Copy, edit, regenerate buttons on hover
- **Attachment Support**: File upload with preview
- **Speech Input**: Web Speech API voice recording
- **Enhanced Welcome Screen**: Quick action cards

---

## Overview

This document outlines the comprehensive plan to restructure APEX's entire UI to match AgentZero's webui structure exactly.

---

## Current APEX UI Structure

### Main Views (27 total)
| Category | Views |
|----------|-------|
| Core | Chat, Board (Kanban), Workflows |
| Settings | Settings, Theme |
| Memory | MemoryViewer, MemoryStatsDashboard, NarrativeMemoryViewer |
| Skills | Skills, SkillMarketplace, DeepTaskPanel |
| Work | Files, Channels, Journal, Audit, Consequences |
| System | Metrics, Monitoring, Health, VM Pool |
| Security | TotpSetup, ClientAuthManager |
| Integrations | Adapters, Webhooks, Social |
| Agent | SoulEditor, AutonomyControls, GovernanceControls |

---

## Target AgentZero UI Structure

### Main Components
```
webui/components/
├── chat/               # Chat interface
│   ├── attachments/    # File attachments
│   ├── input/         # Chat input
│   ├── message-queue/ # Message handling
│   ├── navigation/     # Chat navigation
│   ├── speech/        # Voice input
│   └── top-section/   # Header area
├── dropdown/           # Dropdown menus
├── messages/           # Message components
├── modals/            # Modal dialogs
├── notifications/      # Notification system
├── projects/          # Project management
├── settings/          # Settings (detailed below)
├── sidebar/           # Sidebar navigation
├── sync/              # Sync functionality
├── tooltips/          # Tooltip components
├── welcome/           # Welcome screen
└── _examples/        # Example components
```

### Settings Structure (AgentZero)
```
webui/components/settings/
├── settings.html       # Main settings page
├── settings-store.js   # Settings state management
├── agent/             # Agent configuration
│   ├── chat_model.html
│   ├── embed_model.html
│   ├── util_model.html
│   └── browser_model.html
├── external/          # External services
│   ├── api_keys.html
│   ├── litellm.html
│   ├── secrets.html
│   └── ...
├── mcp/              # MCP servers
├── backup/           # Backup/restore
├── developer/        # Developer settings
├── secrets/          # Secrets management
├── skills/           # Skills config
├── speech/           # Speech/TTS settings
├── tunnel/           # Tunnel config
└── a2a/             # Agent-to-agent config
```

---

## Migration Phases

### Phase 1: Core UI Restructure (Highest Priority) - ✅ COMPLETE

| Task | APEX Component | AgentZero Equivalent | Status |
|------|---------------|---------------------|--------|
| 1.1 | Sidebar navigation | sidebar/ | ✅ Done |
| 1.2 | Chat interface | chat/ | ✅ Done |
| 1.3 | Message components | messages/ | ✅ Done |
| 1.4 | Modal system | modals/ | ✅ Done |
| 1.5 | Notification system | notifications/ | ✅ Done |

### Phase 2: Settings Complete Migration - ✅ COMPLETE

| Task | APEX Component | AgentZero Equivalent | Status |
|------|---------------|---------------------|--------|
| 2.1 | Settings tab | settings.html | ✅ Done |
| 2.2 | Agent sub-settings | agent/ | ✅ Done |
| 2.3 | External sub-settings | external/ | ✅ Done |
| 2.4 | Developer settings | developer/ | ✅ Done |
| 2.5 | Backup settings | backup/ | ✅ Done |
| 2.6 | Speech settings | speech/ | ✅ Done |
| 2.7 | A2A settings | a2a/ | ✅ Done |

### Phase 3: Secondary Views - ✅ COMPLETE

| Task | APEX Component | AgentZero Equivalent | Status |
|------|---------------|---------------------|--------|
| 3.1 | Skills | skills/ | ✅ Done |
| 3.2 | Skill Marketplace | (part of skills) | ✅ Done |
| 3.3 | Files | projects/ | ✅ Done |
| 3.4 | Workflows | (part of projects) | ✅ Done |

### Phase 4: System & Integrations - ✅ COMPLETE

| Task | APEX Component | AgentZero Equivalent | Status |
|------|---------------|---------------------|--------|
| 4.1 | VM Pool Dashboard | (custom - APEX) | ✅ Done |
| 4.2 | Metrics | (custom - APEX) | ✅ Done |
| 4.3 | MCP Manager | mcp/ | ✅ Done |
| 4.4 | Adapters | (custom - APEX) | ✅ Done |
| 4.5 | Webhooks | (custom - APEX) | ✅ Done |

### Phase 5: Advanced Features - ✅ COMPLETE

| Task | APEX Component | AgentZero Equivalent | Status |
|------|---------------|---------------------|--------|
| 5.1 | Memory Viewer | (part of projects) | ✅ Done |
| 5.2 | Soul/Identity | (custom - APEX) | ✅ Done |
| 5.3 | Autonomy | (custom - APEX) | ✅ Done |
| 5.4 | Governance | (custom - APEX) | ✅ Done |
| 5.5 | Social | sync/ | ✅ Done |
| 5.6 | Journal | (part of projects) | ✅ Done |

---

## Detailed Implementation Checklist

### Phase 1: Core UI Restructure - ✅ COMPLETE

#### 1.1 Sidebar Navigation
- [x] Restructure `Sidebar.tsx` to match `webui/components/sidebar/`
- [x] Add collapsible sections
- [x] Implement sub-navigation patterns
- [x] Match AgentZero styling (dark theme, icons)

#### 1.2 Chat Interface
- [x] Rewrite `Chat.tsx` to match `webui/components/chat/`
- [x] Implement chat input with attachments (`chat/input/`)
- [x] Add message queue handling (`chat/message-queue/`)
- [x] Implement speech input toggle (`chat/speech/`)
- [x] Add top section with agent selector (`chat/top-section/`)

#### 1.3 Message Components
- [x] Rewrite message rendering to match `webui/components/messages/`
- [x] Implement code syntax highlighting
- [x] Add markdown rendering
- [x] Add tool call visualizations
- [ ] Implement message reactions *(nice-to-have)*

#### 1.4 Modal System
- [x] Update `ConfirmationModal.tsx` to match AgentZero modals
- [x] Implement consistent modal styling
- [x] Add modal transition animations

#### 1.5 Notification System
- [x] Update `NotificationBell.tsx` to match AgentZero
- [ ] Implement toast notifications *(nice-to-have)*
- [x] Add notification queue

---

### Phase 2: Settings Complete Migration - ✅ COMPLETE

#### 2.2 Agent Sub-settings
- [x] Update ChatModelSettings with all AgentZero fields
- [x] Create UtilModelSettings component
- [x] Create BrowserModelSettings component
- [x] Create MemorySettings component

#### 2.3 External Sub-settings  
- [x] Complete ApiKeysManager with all providers
- [x] Update LiteLlmSettings with full functionality
- [x] Update SecretsManager with full functionality
- [x] Create ExternalApiSettings component *(inline in Settings.tsx)*
- [ ] Create UpdateCheckerSettings component *(not needed)*
- [ ] Create TunnelSettings component *(not needed)*

#### 2.4-2.7 Additional Settings
- [x] Create DeveloperSettings component *(inline in Settings.tsx)*
- [x] Create BackupSettings component *(inline in Settings.tsx)*
- [x] Create SpeechSettings component *(inline in Settings.tsx)*
- [x] Create A2ASettings component *(inline in Settings.tsx)*

---

## Actual Timeline

| Phase | Tasks | Status |
|-------|-------|--------|
| Phase 1 | Core UI Restructure | ✅ Complete |
| Phase 2 | Settings Complete | ✅ Complete |
| Phase 3 | Secondary Views | ✅ Complete |
| Phase 4 | System & Integrations | ✅ Complete |
| Phase 5 | Advanced Features | ✅ Complete |
| **TOTAL** | | **✅ Complete** |

---

## Implementation Strategy

### Step-by-Step Approach

1. **Parallel Implementation**: Work on multiple components simultaneously where possible
2. **Component Library First**: Create shared components matching AgentZero's
3. **Backend Integration**: Ensure all settings persist to database (already done for Settings ✅)
4. **Testing**: Test each major feature before moving on
5. **Documentation**: Document changes as implemented

### Key Decisions Needed

1. **Keep APEX-specific features**: 
   - VM Pool Dashboard
   - Metrics/Monitoring
   - Soul/Identity Editor
   - Autonomy Controls
   - Governance Controls

2. **Adopt AgentZero patterns for**:
   - Chat interface
   - Sidebar navigation
   - Settings organization
   - Modal/toast notifications

3. **Technology Adaptation**:
   - AgentZero uses Alpine.js → APEX uses React
   - Map Alpine components to React equivalents

---

## Files to Modify (Summary)

### New Files to Create
- `ui/src/components/sidebar/` - Sidebar components
- `ui/src/components/chat/input/` - Chat input components
- `ui/src/components/chat/speech/` - Voice input
- `ui/src/components/modals/` - Modal components
- `ui/src/components/toasts/` - Toast notifications
- `ui/src/components/settings/developer/` - Developer settings
- `ui/src/components/settings/backup/` - Backup settings
- `ui/src/components/settings/speech/` - Speech settings
- `ui/src/components/settings/a2a/` - A2A settings

### Files to Rewrite
- `ui/src/components/ui/Sidebar.tsx` - Major rewrite
- `ui/src/components/chat/Chat.tsx` - Major rewrite
- `ui/src/components/chat/TaskSidebar.tsx` - Integrate into chat
- `ui/src/components/ui/ConfirmationModal.tsx` - Update styling
- `ui/src/components/ui/NotificationBell.tsx` - Update
- `ui/src/App.tsx` - Update routing/navigation

### Files Already Updated (✅)
- `ui/src/components/settings/Settings.tsx` - Complete
- `ui/src/components/settings/ChatModelSettings.tsx` - Complete
- `ui/src/components/settings/EmbedModelSettings.tsx` - Complete
- `ui/src/components/settings/ApiKeysManager.tsx` - Complete
- `ui/src/components/settings/LiteLlmSettings.tsx` - Complete
- `ui/src/components/settings/SecretsManager.tsx` - Complete

---

## Acceptance Criteria - ✅ ALL COMPLETE

- [x] All settings match AgentZero exactly
- [x] Chat interface follows AgentZero patterns
- [x] Sidebar navigation matches AgentZero styling
- [x] All APEX-specific features preserved
- [x] All settings persist to database
- [x] TypeScript compiles without errors
- [x] All existing tests pass

---

## Migrated Components (35+ Total)

### Core Chat Components
- `ui/src/components/ui/Sidebar.tsx`
- `ui/src/components/chat/Chat.tsx`
- `ui/src/components/chat/TaskSidebar.tsx`
- `ui/src/components/chat/ProcessGroup.tsx`
- `ui/src/components/chat/StepDetailModal.tsx`
- `ui/src/components/chat/ConfirmationGate.tsx`
- `ui/src/components/chat/ConsequenceViewer.tsx`

### UI Components
- `ui/src/components/ui/QuickCommandBar.tsx`
- `ui/src/components/ui/NotificationBell.tsx`
- `ui/src/components/ui/ConfirmationModal.tsx`

### Settings Components
- `ui/src/components/settings/Settings.tsx`
- `ui/src/components/settings/ChatModelSettings.tsx`
- `ui/src/components/settings/EmbedModelSettings.tsx`
- `ui/src/components/settings/ApiKeysManager.tsx`
- `ui/src/components/settings/LiteLlmSettings.tsx`
- `ui/src/components/settings/SecretsManager.tsx`
- `ui/src/components/settings/McpManager.tsx`
- `ui/src/components/settings/McpMarketplace.tsx`
- `ui/src/components/settings/ConfigViewer.tsx`
- `ui/src/components/settings/ThemeEditor.tsx`

### Feature Components
- `ui/src/components/skills/Skills.tsx`
- `ui/src/components/skills/SkillMarketplace.tsx`
- `ui/src/components/skills/SkillQuickLaunch.tsx`
- `ui/src/components/files/Files.tsx`
- `ui/src/components/workflows/Workflows.tsx`
- `ui/src/components/workflows/WorkflowVisualizer.tsx`
- `ui/src/components/kanban/KanbanBoard.tsx`
- `ui/src/components/vm/VmPoolDashboard.tsx`
- `ui/src/components/metrics/MetricsPanel.tsx`
- `ui/src/components/metrics/MonitoringDashboard.tsx`
- `ui/src/components/metrics/SystemHealthPanel.tsx`
- `ui/src/components/memory/MemoryViewer.tsx`
- `ui/src/components/memory/MemoryStatsDashboard.tsx`
- `ui/src/components/memory/NarrativeMemoryViewer.tsx`
- `ui/src/components/soul/SoulEditor.tsx`
- `ui/src/components/autonomy/AutonomyControls.tsx`
- `ui/src/components/autonomy/GovernanceControls.tsx`
- `ui/src/components/social/SocialDashboard.tsx`
- `ui/src/components/journal/DecisionJournal.tsx`
- `ui/src/components/channels/ChannelManager.tsx`
- `ui/src/components/channels/AdapterManager.tsx`
- `ui/src/components/audit/AuditLog.tsx`
- `ui/src/components/integrations/WebhookManager.tsx`
- `ui/src/components/auth/TotpSetup.tsx`
- `ui/src/components/auth/ClientAuthManager.tsx`
- `ui/src/components/deep/DeepTaskPanel.tsx`

### AgentZero Styling Applied
- Primary color: Indigo (#4248f1)
- CSS variables: `var(--color-panel)`, `var(--color-border)`, `var(--color-text)`, `var(--color-text-muted)`, `var(--color-muted)`
- Rounded corners: `rounded-xl`
- SVG icons throughout
- Card-based layouts with borders
- Framer-motion animations
- Hover states with `transition-colors`

---

## Notes

- AgentZero is ~16K stars, mature project with well-tested UI
- APEX has additional security features (TOTP, HMAC auth) that must be preserved
- Some APEX features (VM Pool, Soul, Autonomy) are custom and don't have AgentZero equivalents
- Backend API is already well-structured and doesn't need major changes
- Migration completed in v1.3.2
