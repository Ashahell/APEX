# UI (User Interface)

## Overview

APEX provides a React 18 + TypeScript UI with real-time streaming, 4 themes, and comprehensive settings.

## Tech Stack

- **Framework**: React 18, TypeScript
- **Styling**: Tailwind CSS, Radix UI components
- **State**: Zustand
- **Data**: TanStack React Query
- **Real-time**: Socket.io-client, SSE

## Themes

| Theme | Description |
|-------|-------------|
| Modern 2026 | Default modern look |
| Amiga Workbench | Retro Amiga aesthetic |
| AgentZero | Dark navy (#0f0f1a) with cyan (#00d4ff) |
| High Contrast | WCAG AAA accessible |

## Features

### Real-time
- WebSocket with auto-reconnection
- SSE streaming for task events
- Polling fallback

### Navigation
- Top-level: Chat, Board, Workflows, Settings
- Sub-menus: Memory, Skills, Work, System, Security, Integrations, Agent

### Components
- Chat with message reactions, attachments
- Kanban board for task management
- Process groups for execution traces
- Confirmation gates (T1-T3)

### Settings (15+ tabs)
- Chat, Embed, Util, Browser, Memory
- User Profile, Continuity, Fast Mode
- MCP Manager, Skill Security
- Theme Editor, Runtime Settings

## Architecture

```
ui/src/
├── components/    # React components
├── hooks/         # Custom hooks (useApi, useTheme, useWebSocket)
├── lib/           # Utilities (api, sse, websocket)
├── stores/        # Zustand stores
├── pages/        # Page components
└── themes/       # Theme definitions
```