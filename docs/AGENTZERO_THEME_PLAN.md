# AgentZero-Style Theme Implementation Plan

## Overview
Create an AgentZero-inspired theme for APEX that mimics the look and feel of the AgentZero AI framework UI.

## AgentZero UI Design Analysis

### Color Scheme (from screenshots and descriptions)
- **Background**: Deep navy/blue-black (#0f0f1a to #1a1a2e range)
- **Surface/Cards**: Slightly lighter blue (#252542, #2a2a4a)
- **Primary Accent**: Cyan (#00d4ff, #00b4d8)
- **Secondary Accent**: Purple/magenta (#8b5cf6, #a855f7)
- **Text Primary**: White (#ffffff, #f0f0f0)
- **Text Secondary**: Gray-blue (#94a3b8, #64748b)
- **Success**: Green (#22c55e)
- **Warning**: Amber (#f59e0b)
- **Error**: Red (#ef4444)
- **Agent states**: Distinct colors for different statuses

### UI Components & Features
1. **Chat Interface**
   - Collapsible message groups (process groups)
   - Real-time streaming with smooth rendering
   - Message queue support
   - Step detail modals

2. **Sidebar**
   - Dark themed navigation
   - Dropdown components
   - Streamlined buttons

3. **File Browser**
   - Enhanced with rename/actions dropdown
   - Clean grid/list view

4. **Skills Panel**
   - Skills framework UI
   - Import/list capabilities

5. **Settings**
   - Modular settings architecture
   - Chat width setting
   - Preferences panel

6. **Terminal Integration**
   - Modal terminal
   - PTY-backed sessions

7. **Visual Elements**
   - Smooth animations
   - Scroll stabilization
   - Image viewer improvements

## Implementation Steps

### Step 1: Create AgentZero Theme Definition
- Create `ui/src/themes/agentzero.ts` with color tokens
- Define color palette matching AgentZero's dark navy/cyan theme
- Include all required color tokens (bg, text, primary, button, accent, agent, badge)

### Step 2: Export Theme
- Update `ui/src/themes/index.ts` to export the new theme

### Step 3: Apply Theme via CSS
- Update `ui/src/hooks/useTheme.tsx` to handle the AgentZero theme
- Ensure proper CSS variable injection
- Add specific CSS rules for AgentZero-style elements

### Step 4: Register Theme in ThemeProvider
- Add AgentZero to available themes
- Ensure theme persistence works

### Step 5: Test Theme Selection
- Verify theme appears in ThemeEditor presets
- Test color customization works with new theme

## APEX Features Missing from AgentZero (to potentially implement)
1. **Message Queue** - Queue instructions while agent is running
2. **Step Detail Modal** - Deep debugging on any step
3. **Skills UI** - Import and list skills from SKILL.md
4. **Git Projects** - Git-based project management
5. **Scheduler Redesign** - Standalone modal with task list
6. **File Browser Enhancements** - Rename, actions dropdown
7. **Image Viewer** - Scroll support, expanded viewer

## Files to Modify/Create
- `ui/src/themes/agentzero.ts` (new)
- `ui/src/themes/index.ts` (update)
- `ui/src/hooks/useTheme.tsx` (update)
- `ui/src/components/settings/ThemeEditor.tsx` (may need updates)

## Success Criteria
- AgentZero theme selectable in Settings → Theme
- Colors match AgentZero's dark navy/cyan aesthetic
- All UI components render correctly under new theme
- Theme customization (color picker) works
