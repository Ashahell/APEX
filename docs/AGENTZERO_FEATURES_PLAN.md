# AgentZero Features Implementation Plan

## Features to Implement

### 1. Step Detail Modal (HIGH PRIORITY)
**Purpose**: Deep debugging on any execution step - view full details of a step including input, output, errors, timing

**AgentZero Behavior**:
- Click on any step in process group to open modal
- Shows step type, status, timestamps, full input/output
- Allows copying data, viewing raw JSON
- Collapsible sections for long content

**APEX Implementation**:
- Create `StepDetailModal.tsx` component
- Integrate with existing ProcessGroup component
- Show modal on step click

### 2. Message Queue (HIGH PRIORITY)
**Purpose**: Queue instructions while agent is running - don't wait for agent to finish

**AgentZero Behavior**:
- Input field accepts multiple messages
- Messages queued and processed sequentially
- Queue indicator shows pending messages
- Can cancel pending messages

**APEX Implementation**:
- Enhance Chat input to support queuing
- Add message queue state management
- Show queue count in UI
- Process queue sequentially

### 3. File Browser Enhancements (MEDIUM PRIORITY)
**Purpose**: Better file management in browser

**AgentZero Behavior**:
- Rename files inline
- Actions dropdown (copy path, delete, move)
- Breadcrumb navigation
- Drag and drop support (optional)

**APEX Implementation**:
- Check existing Files component
- Add rename functionality
- Add actions dropdown menu

## Implementation Order

1. **Step Detail Modal** - Most valuable for debugging
2. **Message Queue** - Improves UX significantly
3. **File Browser** - Nice to have

## Files to Create/Modify

### Step Detail Modal
- `ui/src/components/chat/StepDetailModal.tsx` (new)
- `ui/src/components/chat/ProcessGroup.tsx` (update - add click handler)

### Message Queue
- `ui/src/components/chat/Chat.tsx` (update - add queue support)
- `ui/src/stores/appStore.ts` (update - add queue state)

### File Browser
- `ui/src/components/work/Files.tsx` or similar (update - add actions)

## Success Criteria
- Step Detail Modal opens on step click, shows all step details
- Messages can be queued, show pending count, process sequentially
- File browser has rename and actions dropdown
