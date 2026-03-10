# APEX Keyboard Shortcuts

> **Last Updated**: 2026-03-09

APEX provides keyboard shortcuts for quick navigation and common actions.

## Available Shortcuts

### Navigation

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+1` | Chat | Go to Chat tab |
| `Ctrl+2` | Skills | Go to Skills tab |
| `Ctrl+3` | Memory | Go to Memory tab |
| `Ctrl+4` | Board | Go to Kanban Board |
| `Ctrl+5` | Settings | Go to Settings tab |
| `Ctrl+B` | Toggle sidebar | Show/hide the sidebar |

### Actions

| Shortcut | Action | Description |
|----------|--------|-------------|
| `Ctrl+K` | Command palette | Open quick command bar |
| `Ctrl+/` | Focus input | Focus the message input field |
| `Escape` | Close modal | Close any open modal or panel |

## Usage Tips

### Modal Context
When a text input, textarea, or content-editable element is focused:
- Most shortcuts are disabled
- `Escape` still works to blur/close

### Mac Users
- `Ctrl` can be replaced with `Cmd` (⌘) on Mac
- `Ctrl+K` = `⌘+K`

## Custom Shortcuts

You can add custom keyboard shortcuts in your component:

```typescript
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';

function MyComponent() {
  useKeyboardShortcuts([
    { 
      key: 's', 
      ctrl: true, 
      action: () => console.log('Save!'),
      description: 'Save current work'
    },
  ]);
}
```

## Default Shortcuts Reference

```
┌─────────────────────────────────────────────────────────────┐
│                    APEX Keyboard Shortcuts                   │
├─────────────────────────────────────────────────────────────┤
│  Navigation           │  Actions                            │
│  ──────────────────  │  ────────────────────────────────   │
│  Ctrl+1  Chat        │  Ctrl+K  Command Palette           │
│  Ctrl+2  Skills      │  Ctrl+/  Focus Input               │
│  Ctrl+3  Memory      │  Esc    Close Modal               │
│  Ctrl+4  Board       │                                    │
│  Ctrl+5  Settings    │                                    │
│  Ctrl+B  Sidebar    │                                    │
└─────────────────────────────────────────────────────────────┘
```
