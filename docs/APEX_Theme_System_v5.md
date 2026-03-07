# APEX Theme Management System & Sidebar Reorganization Plan

**Version**: 1.0  
**Date**: 2026-03-07  
**Status**: Design specification

---

## Overview

This document describes a practical theme management system for APEX with two themes:
1. **Modern 2026** — Default dark theme with cyan accents
2. **Amiga Workbench** — Classic Amiga-inspired aesthetic

Additionally, this document addresses the sidebar reorganization to reduce clutter by grouping 28 tabs into logical submenus.

---

## Part 1: Theme Architecture

### 1.1 Design Principles

1. **Themes are CSS variable sets** — No runtime compilation, no complex inheritance
2. **Components use semantic tokens** — `var(--color-bg-elevated)`, not hardcoded hex values
3. **Theme switching is instant** — Swap CSS variable definitions, not class names
4. **Two built-in themes only** — Modern (default) + Amiga (opt-in)
5. **Progressive migration** — Hardcoded values degrade gracefully

### 1.2 Why Not the v4 Specification

The v4 specification (`APEX_Theme_System_v4.md`) proposes:
- Complex inheritance system with cycle detection
- Server-side WCAG validation on every theme write
- Full component migration pass before themes work
- APEX-OS — a complete desktop OS emulator

This plan rejects all of the above because:
- Themes don't need inheritance — users can copy-paste to modify
- Server-side validation adds latency with negligible UX benefit
- The migration can be progressive — themes work partially even with hardcoded values
- APEX-OS is nostalgia-driven bloat — the existing UI works fine

### 1.3 Data Model

```typescript
interface Theme {
  id: string;
  name: string;
  description: string;
  isBuiltIn: boolean;
  tokens: ThemeTokens;
}

interface ThemeTokens {
  colors: ColorTokens;
  spacing?: SpacingTokens;
  typography?: TypographyTokens;
  components?: ComponentTokens;
}

interface ColorTokens {
  bg: {
    base: string;
    elevated: string;
    overlay: string;
    surface?: string;
  };
  text: {
    primary: string;
    secondary: string;
    muted: string;
    inverse?: string;
  };
  primary: {
    DEFAULT: string;
    hover: string;
    active: string;
    muted?: string;
  };
  accent?: {
    success: string;
    warning: string;
    error: string;
    info: string;
  };
  agent?: {
    idle: string;
    active: string;
    thinking: string;
    alert: string;
  };
  badge?: {
    gen: string;
    use: string;
    exe: string;
    www: string;
    sub: string;
    mem: string;
    aud: string;
  };
  chrome?: {
    titleBarActive?: string;
    titleBarInactive?: string;
    buttonRaised?: string;
    buttonDepressed?: string;
    windowBorder?: string;
  };
}
```

### 1.4 Token Categories

| Category | Tokens | Migration Priority |
|----------|--------|-------------------|
| `color-bg-*` | base, elevated, overlay, surface | High |
| `color-text-*` | primary, secondary, muted, inverse | High |
| `color-primary-*` | DEFAULT, hover, active, muted | High |
| `color-accent-*` | success, warning, error, info | Medium |
| `color-agent-*` | idle, active, thinking, alert | Medium |
| `color-badge-*` | gen, use, exe, www, sub, mem, aud | Medium |
| `spacing-*` | 1-9 (4px base unit) | Medium |
| `radius-*` | none, sm, md, lg, full | Low |
| `shadow-*` | sm, md, lg | Low |

---

## Part 2: Theme Definitions

### 2.1 Modern 2026 Theme (Default)

```typescript
// themes/modern-2026.ts
export const modern2026Theme: Theme = {
  id: 'modern-2026',
  name: 'Modern 2026',
  description: 'Clean, minimal dark theme with cyan accents',
  isBuiltIn: true,
  tokens: {
    colors: {
      bg: {
        base: '#0a0a0f',
        elevated: '#12121a',
        overlay: '#1a1a24',
        surface: '#0e0e14',
      },
      text: {
        primary: '#e8e8ec',
        secondary: '#9090a0',
        muted: '#606070',
        inverse: '#0a0a0f',
      },
      primary: {
        DEFAULT: '#00d4aa',
        hover: '#00e6bb',
        active: '#00c29a',
        muted: 'rgba(0, 212, 170, 0.15)',
      },
      accent: {
        success: '#22c55e',
        warning: '#f59e0b',
        error: '#ef4444',
        info: '#3b82f6',
      },
      agent: {
        idle: '#606070',
        active: '#00d4aa',
        thinking: '#f59e0b',
        alert: '#ef4444',
      },
      badge: {
        gen: '#8b5cf6',
        use: '#00d4aa',
        exe: '#3b82f6',
        www: '#f59e0b',
        sub: '#ec4899',
        mem: '#22c55e',
        aud: '#ef4444',
      },
    },
  },
};
```

### 2.2 Amiga Workbench Theme

```typescript
// themes/amiga.ts
export const amigaTheme: Theme = {
  id: 'amiga',
  name: 'Amiga Workbench',
  description: 'Classic Amiga-inspired aesthetic',
  isBuiltIn: true,
  tokens: {
    colors: {
      bg: {
        base: '#000000',
        elevated: '#1a1a2e',
        overlay: '#2a2a4e',
        surface: '#0f0f1f',
      },
      text: {
        primary: '#ffffff',
        secondary: '#a0a0c0',
        muted: '#606080',
        inverse: '#000000',
      },
      primary: {
        DEFAULT: '#5699d4',
        hover: '#6aade4',
        active: '#4a89c4',
        muted: 'rgba(86, 153, 212, 0.2)',
      },
      accent: {
        success: '#4ade80',
        warning: '#fbbf24',
        error: '#f87171',
        info: '#60a5fa',
      },
      agent: {
        idle: '#606080',
        active: '#5699d4',
        thinking: '#fbbf24',
        alert: '#f87171',
      },
      badge: {
        gen: '#a78bfa',
        use: '#5699d4',
        exe: '#60a5fa',
        www: '#fbbf24',
        sub: '#f472b6',
        mem: '#4ade80',
        aud: '#f87171',
      },
      chrome: {
        titleBarActive: 'linear-gradient(180deg, #6a89c4 0%, #3d5a80 100%)',
        titleBarInactive: 'linear-gradient(180deg, #4a4a6a 0%, #2a2a3a 100%)',
        buttonRaised: 'inset 1px 1px 0 rgba(255,255,255,0.3), inset -1px -1px 0 rgba(0,0,0,0.3)',
        buttonDepressed: 'inset 2px 2px 4px rgba(0,0,0,0.4)',
        windowBorder: '#6a89c4',
      },
    },
  },
};
```

### 2.3 Theme Context Implementation

```typescript
// hooks/useTheme.ts
import { useState, useEffect, useCallback } from 'react';
import { modern2026Theme } from '../themes/modern-2026';
import { amigaTheme } from '../themes/amiga';

export type ThemeId = 'modern-2026' | 'amiga';

const themes: Record<ThemeId, Theme> = {
  'modern-2026': modern2026Theme,
  'amiga': amigaTheme,
};

interface ThemeContextValue {
  theme: Theme;
  themeId: ThemeId;
  setTheme: (id: ThemeId) => void;
  availableThemes: Theme[];
}

export function useTheme(): ThemeContextValue {
  const [themeId, setThemeId] = useState<ThemeId>(() => {
    const saved = localStorage.getItem('apex-theme-id') as ThemeId;
    return saved && themes[saved] ? saved : 'modern-2026';
  });

  const theme = themes[themeId];

  useEffect(() => {
    const root = document.documentElement;
    
    // Apply color tokens
    if (theme.tokens.colors) {
      const { bg, text, primary, accent, agent, badge, chrome } = theme.tokens.colors;
      
      if (bg) {
        root.style.setProperty('--color-bg-base', bg.base);
        root.style.setProperty('--color-bg-elevated', bg.elevated);
        root.style.setProperty('--color-bg-overlay', bg.overlay);
        if (bg.surface) root.style.setProperty('--color-bg-surface', bg.surface);
      }
      
      if (text) {
        root.style.setProperty('--color-text-primary', text.primary);
        root.style.setProperty('--color-text-secondary', text.secondary);
        root.style.setProperty('--color-text-muted', text.muted);
        if (text.inverse) root.style.setProperty('--color-text-inverse', text.inverse);
      }
      
      if (primary) {
        root.style.setProperty('--color-primary', primary.DEFAULT);
        if (primary.hover) root.style.setProperty('--color-primary-hover', primary.hover);
        if (primary.active) root.style.setProperty('--color-primary-active', primary.active);
        if (primary.muted) root.style.setProperty('--color-primary-muted', primary.muted);
      }
      
      if (accent) {
        root.style.setProperty('--color-accent-success', accent.success);
        root.style.setProperty('--color-accent-warning', accent.warning);
        root.style.setProperty('--color-accent-error', accent.error);
        root.style.setProperty('--color-accent-info', accent.info);
      }
      
      if (agent) {
        root.style.setProperty('--color-agent-idle', agent.idle);
        root.style.setProperty('--color-agent-active', agent.active);
        root.style.setProperty('--color-agent-thinking', agent.thinking);
        root.style.setProperty('--color-agent-alert', agent.alert);
      }
      
      if (badge) {
        root.style.setProperty('--color-badge-gen', badge.gen);
        root.style.setProperty('--color-badge-use', badge.use);
        root.style.setProperty('--color-badge-exe', badge.exe);
        root.style.setProperty('--color-badge-www', badge.www);
        root.style.setProperty('--color-badge-sub', badge.sub);
        root.style.setProperty('--color-badge-mem', badge.mem);
        root.style.setProperty('--color-badge-aud', badge.aud);
      }
      
      // Amiga-specific chrome tokens
      if (chrome) {
        if (chrome.titleBarActive) root.style.setProperty('--color-chrome-titlebar-active', chrome.titleBarActive);
        if (chrome.titleBarInactive) root.style.setProperty('--color-chrome-titlebar-inactive', chrome.titleBarInactive);
        if (chrome.buttonRaised) root.style.setProperty('--color-chrome-button-raised', chrome.buttonRaised);
        if (chrome.buttonDepressed) root.style.setProperty('--color-chrome-button-depressed', chrome.buttonDepressed);
        if (chrome.windowBorder) root.style.setProperty('--color-chrome-window-border', chrome.windowBorder);
      }
    }
    
    localStorage.setItem('apex-theme-id', themeId);
  }, [theme, themeId]);

  return {
    theme,
    themeId,
    setTheme: setThemeId,
    availableThemes: Object.values(themes),
  };
}
```

---

## Part 3: Sidebar Reorganization

### 3.1 Current State

The sidebar currently has **28 flat items** with icons and labels:
- Chat, Skills, Marketplace, Consequences, Memory, MemoryStats, Narrative, Files, Board, Workflows, Deep, Audit, Channels, Journal, Adapters, Webhooks, VMs, Metrics, Monitoring, Health, Soul, Social, Autonomy, Governance, 2FA, Clients, Settings

This is unmanageable.

### 3.2 Proposed Structure

#### Top-Level Items (5)

| Icon | Label | Shortcut | Rationale |
|------|-------|----------|-----------|
| 💬 | Chat | Ctrl+1 | Primary interaction |
| 📋 | Board | Ctrl+2 | Task management |
| 🔄 | Workflows | Ctrl+3 | Automation |
| ⚙️ | Settings | Ctrl+, | Preferences |
| 🎨 | Theme | — | Theme switcher |

#### Submenu Groups (6)

**🧠 Memory**
- Memory → Memory Viewer
- Stats → Memory Stats
- Narrative → Narrative Memory

**⚡ Skills**
- Registry → Skills
- Marketplace → Skill Marketplace
- Deep → Deep Tasks

**📁 Work**
- Files → File Browser
- Channels → Channel Manager
- Journal → Decision Journal
- Audit → Audit Log
- Preview → Consequence Viewer

**🖥️ System**
- Metrics → Metrics Panel
- Monitor → Monitoring Dashboard
- Health → System Health
- VMs → VM Pool

**🔒 Security**
- 2FA → TOTP Setup
- Clients → Client Auth

**🔌 Integrations**
- Adapters → Adapter Manager
- Webhooks → Webhook Manager
- Social → Social Dashboard

**🤖 Agent**
- Identity → Soul Editor
- Autonomy → Autonomy Controls
- Governance → Governance Controls

### 3.3 Visual Mockup

```
┌─────────────────────────────────────────────────────────────┐
│ APEX                                              [≡] 🌙  │
├──────────┬──────────────────────────────────────────────────┤
│          │                                                   │
│  💬 Chat │                                                   │
│  📋 Board│                                                   │
│  🔄 Work │                                                   │
│  ⚙ Set  │                                                   │
│  🎨 Them │                                                   │
│          │                                                   │
│ ──────── │                                                   │
│          │                                                   │
│ 🧠 Memory│                                                   │
│   ├─ Mem │                                                   │
│   ├─ Stats│                                                   │
│   └─ Narr│                                                   │
│          │                                                   │
│ ⚡ Skills│                                                   │
│   ├─ Reg │                                                   │
│   ├─ Mkpt│                                                   │
│   └─ Deep│                                                   │
│          │                                                   │
│ 📁 Work  │                                                   │
│   ├─File │                                                   │
│   ├─Chan │                                                   │
│   ├─Jrnl │                                                   │
│   ├─Audit│                                                   │
│   └─Previ│                                                   │
│          │                                                   │
│ 🖥️System │                                                   │
│   ├─Metrc│                                                   │
│   ├─Montr│                                                   │
│   ├─Hlth │                                                   │
│   └─VMs  │                                                   │
│          │                                                   │
│ 🔒 Sec   │                                                   │
│   ├─2FA  │                                                   │
│   └─Clnt │                                                   │
│          │                                                   │
│ 🔌 Integ │                                                   │
│   ├─Adap │                                                   │
│   ├─Webh │                                                   │
│   └─Socl │                                                   │
│          │                                                   │
│ 🤖 Agent │                                                   │
│   ├─Iden │                                                   │
│   ├─Auton│                                                   │
│   └─Govn │                                                   │
│          │                                                   │
└──────────┴──────────────────────────────────────────────────┘
```

### 3.4 Implementation

```typescript
// components/ui/Sidebar.tsx
import { useState } from 'react';
import clsx from 'clsx';

type AppTab = 
  // Top-level
  | 'chat' | 'board' | 'workflows' | 'settings' | 'theme'
  // Memory
  | 'memory' | 'memoryStats' | 'narrative'
  // Skills
  | 'skills' | 'marketplace' | 'deep'
  // Work
  | 'files' | 'channels' | 'journal' | 'audit' | 'consequences'
  // System
  | 'metrics' | 'monitoring' | 'health' | 'vm'
  // Security
  | 'totp' | 'clients'
  // Integrations
  | 'adapters' | 'webhooks' | 'social'
  // Agent
  | 'soul' | 'autonomy' | 'governance';

interface SidebarItem {
  id: AppTab;
  label: string;
  icon: string;
  shortcut?: string;
}

interface SidebarGroup {
  id: string;
  label: string;
  icon: string;
  items: SidebarItem[];
  isTopLevel?: boolean;
}

const GROUPS: SidebarGroup[] = [
  // Top-level (always visible)
  {
    id: 'toplevel',
    label: '',
    icon: '',
    isTopLevel: true,
    items: [
      { id: 'chat', label: 'Chat', icon: '💬', shortcut: 'Ctrl+1' },
      { id: 'board', label: 'Board', icon: '📋', shortcut: 'Ctrl+2' },
      { id: 'workflows', label: 'Workflows', icon: '🔄', shortcut: 'Ctrl+3' },
      { id: 'settings', label: 'Settings', icon: '⚙️', shortcut: 'Ctrl+,' },
      { id: 'theme', label: 'Theme', icon: '🎨' },
    ],
  },
  {
    id: 'memory',
    label: 'Memory',
    icon: '🧠',
    items: [
      { id: 'memory', label: 'Memory' },
      { id: 'memoryStats', label: 'Stats' },
      { id: 'narrative', label: 'Narrative' },
    ],
  },
  {
    id: 'skills',
    label: 'Skills',
    icon: '⚡',
    items: [
      { id: 'skills', label: 'Registry' },
      { id: 'marketplace', label: 'Marketplace' },
      { id: 'deep', label: 'Deep Tasks' },
    ],
  },
  {
    id: 'work',
    label: 'Work',
    icon: '📁',
    items: [
      { id: 'files', label: 'Files' },
      { id: 'channels', label: 'Channels' },
      { id: 'journal', label: 'Journal' },
      { id: 'audit', label: 'Audit' },
      { id: 'consequences', label: 'Preview' },
    ],
  },
  {
    id: 'system',
    label: 'System',
    icon: '🖥️',
    items: [
      { id: 'metrics', label: 'Metrics' },
      { id: 'monitoring', label: 'Monitor' },
      { id: 'health', label: 'Health' },
      { id: 'vm', label: 'VMs' },
    ],
  },
  {
    id: 'security',
    label: 'Security',
    icon: '🔒',
    items: [
      { id: 'totp', label: '2FA' },
      { id: 'clients', label: 'Clients' },
    ],
  },
  {
    id: 'integrations',
    label: 'Integrations',
    icon: '🔌',
    items: [
      { id: 'adapters', label: 'Adapters' },
      { id: 'webhooks', label: 'Webhooks' },
      { id: 'social', label: 'Social' },
    ],
  },
  {
    id: 'agent',
    label: 'Agent',
    icon: '🤖',
    items: [
      { id: 'soul', label: 'Identity' },
      { id: 'autonomy', label: 'Autonomy' },
      { id: 'governance', label: 'Governance' },
    ],
  },
];

export function Sidebar({ activeTab, onTabChange, collapsed = false }: SidebarProps) {
  const [expandedGroup, setExpandedGroup] = useState<string | null>('memory');

  const toggleGroup = (groupId: string) => {
    setExpandedGroup(prev => prev === groupId ? null : groupId);
  };

  return (
    <aside className={clsx(
      "hidden md:flex border-r flex-col py-4 gap-1 transition-all duration-200 shrink-0",
      collapsed ? "w-12 items-center" : "w-16 lg:w-20 items-center"
    )}>
      {/* Top-level items */}
      {GROUPS.find(g => g.isTopLevel)?.items.map((item) => (
        <button
          key={item.id}
          onClick={() => onTabChange(item.id as AppTab)}
          className={clsx(
            "rounded-lg flex items-center justify-center transition-colors",
            collapsed ? "w-10 h-10 text-lg" : "w-12 lg:w-14 h-10 lg:h-12 text-lg",
            activeTab === item.id
              ? "bg-primary text-primary-foreground"
              : "hover:bg-muted"
          )}
          title={item.label}
        >
          {item.icon}
        </button>
      ))}
      
      <div className="w-8 border-t my-2" />
      
      {/* Grouped items */}
      {GROUPS.filter(g => !g.isTopLevel).map((group) => (
        <div key={group.id} className="relative">
          <button
            onClick={() => toggleGroup(group.id)}
            className={clsx(
              "rounded-lg flex items-center justify-center transition-colors w-full",
              collapsed ? "w-10 h-10 text-lg" : "w-12 lg:w-14 h-10 lg:h-12 text-lg",
              expandedGroup === group.id
                ? "bg-primary/20 text-primary"
                : "hover:bg-muted"
            )}
            title={group.label}
          >
            {group.icon}
          </button>
          
          {/* Expanded submenu */}
          {expandedGroup === group.id && !collapsed && (
            <div className="absolute left-full top-0 ml-1 bg-popover border rounded-lg shadow-lg py-1 min-w-[140px] z-50">
              <div className="px-3 py-1 text-xs font-semibold text-muted-foreground border-b">
                {group.label}
              </div>
              {group.items.map((item) => (
                <button
                  key={item.id}
                  onClick={() => {
                    onTabChange(item.id as AppTab);
                    setExpandedGroup(null);
                  }}
                  className={clsx(
                    "w-full px-3 py-2 text-left text-sm hover:bg-muted flex items-center justify-between",
                    activeTab === item.id && "bg-primary/10 text-primary"
                  )}
                >
                  <span>{item.label}</span>
                  {item.shortcut && (
                    <span className="text-xs text-muted-foreground ml-2">
                      {item.shortcut}
                    </span>
                  )}
                </button>
              ))}
            </div>
          )}
        </div>
      ))}
    </aside>
  );
}
```

### 3.5 Keyboard Navigation

| Shortcut | Action |
|----------|--------|
| Ctrl+1 | Chat |
| Ctrl+2 | Board (Kanban) |
| Ctrl+3 | Workflows |
| Ctrl+, | Settings |
| Ctrl+M | Toggle Memory submenu |
| Ctrl+K | Toggle Skills submenu |
| Ctrl+W | Toggle Work submenu |
| ↓/↑ | Navigate within submenu |
| Enter | Select item |
| Esc | Close submenu |

---

## Part 4: Implementation Roadmap

### Phase 1: Foundation (Week 1)

- [ ] Create `ui/src/themes/` directory
- [ ] Define `Theme` and `ThemeTokens` types
- [ ] Implement Modern 2026 theme tokens
- [ ] Enhance `useTheme` hook with full token support
- [ ] Test theme switching

### Phase 2: Sidebar (Week 2)

- [ ] Implement collapsible submenu system
- [ ] Group tabs into 8 logical groups
- [ ] Add keyboard navigation for submenus
- [ ] Preserve Ctrl+1-3 shortcuts for top-level items
- [ ] Mobile bottom nav (keep top-level only)

### Phase 3: Amiga Theme (Week 3)

- [ ] Define Amiga color tokens
- [ ] Add chrome-specific tokens (bevels, gradients)
- [ ] Create optional pixel font loading
- [ ] Add theme switcher UI to Settings

### Phase 4: Polish (Week 4)

- [ ] Add smooth transition when switching themes
- [ ] Persist theme preference in localStorage
- [ ] Add "Amiga mode" hint in Settings
- [ ] Document token migration guidelines

---

## Part 5: Component Migration Guidelines

### 5.1 Priority Order

When migrating components to use theme tokens:

1. **High priority**: Components visible on initial load
   - Chat message backgrounds
   - Sidebar background
   - Header background
   - Input fields

2. **Medium priority**: Frequently used components
   - Buttons (primary, secondary, danger)
   - Cards and panels
   - Modals and dialogs

3. **Low priority**: Rarely used components
   - Badge colors
   - Agent state indicators
   - Process step colors

### 5.2 Migration Pattern

```typescript
// Before (hardcoded)
<div className="bg-gray-900 text-white">
  Content
</div>

// After (using tokens)
<div className="bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)]">
  Content
</div>

// Or with Tailwind custom config
<div className="bg-bg-elevated text-text-primary">
  Content
</div>
```

### 5.3 Graceful Degradation

Components that haven't been migrated will continue to work using Tailwind's default colors. The theme system is additive — it enhances themed components without breaking unthemed ones.

---

## Appendix: Theme Token Reference

### Color Tokens

| Token | Description | Example |
|-------|-------------|---------|
| `--color-bg-base` | Page background | `#0a0a0f` |
| `--color-bg-elevated` | Cards, panels | `#12121a` |
| `--color-bg-overlay` | Modals, dropdowns | `#1a1a24` |
| `--color-text-primary` | Main content | `#e8e8ec` |
| `--color-text-secondary` | Labels, metadata | `#9090a0` |
| `--color-text-muted` | Disabled, hints | `#606070` |
| `--color-primary` | Main accent | `#00d4aa` |
| `--color-primary-hover` | Hover state | `#00e6bb` |
| `--color-accent-success` | Success messages | `#22c55e` |
| `--color-accent-warning` | Warnings | `#f59e0b` |
| `--color-accent-error` | Errors | `#ef4444` |
| `--color-agent-active` | Agent running | `#00d4aa` |
| `--color-agent-thinking` | Agent thinking | `#f59e0b` |

### Amiga Chrome Tokens

| Token | Description |
|-------|-------------|
| `--color-chrome-titlebar-active` | Active window title gradient |
| `--color-chrome-titlebar-inactive` | Inactive window title gradient |
| `--color-chrome-button-raised` | Raised button bevel |
| `--color-chrome-button-depressed` | Pressed button bevel |
| `--color-chrome-window-border` | Window border color |

---

*APEX Theme Management System & Sidebar Reorganization — v1.0*
