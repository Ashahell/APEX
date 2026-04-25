# APEX Theme System Design Specification

**Version**: 4.0  
**Architecture ref**: APEX v1.0.0 (2026-03-04)  
**Status**: Design specification — implementation pending  
**Scope**: Core theme system · APEX-OS Experience Mode · Visual identity assets

> **Document structure.** Part I (Sections 1–10) covers the core theme system — data model, rendering engine, editor UI, security, and API. It is a hard prerequisite for everything else. Part II (Sections 11–16) covers **APEX-OS**, an opt-in alternative UI mode inspired by classic Amiga Workbench aesthetics but built with original naming and original implementation. Part III (Section 17) covers image generation prompts for the splash screen, logo, avatar, and icon guidance. Part II depends entirely on Part I being stable first. The roadmap enforces this.
Every location in v4 that needs updating:

Section 1 design principles (line 60) — remove the stub exclusion statement; replace with an active-system mapping statement
Section 3.2 colour system — add a system colour group: heartbeat, soul, governance, moltbook tokens
Section 5.1 CSS compiler — emit the new --color-system-* variables
Section 5.5 WCAG pairs — add the four new system colour pairs
Section 14.1 ApexLoader visual — add Heartbeat, Soul, Governance, Moltbook to the boot status display
Section 14.2 boot sequence code — add boot phases for the four systems
Section 15.1 desktop layout — add 4 icons to the desktop ASCII diagram; update the icon mapping table; remove the NOT ACTIVE note
Section 15.3 feature mapping table — add 4 new rows for the active systems
Section 15.5 menu system — add the new surfaces to the Folders menu
Section 15.6 Workspaces — add a new Identity or Agent workspace for Soul/Governance/Heartbeat; add Social workspace for Moltbook
Section 16 Phase 7 roadmap — remove the conditional note on Heartbeat; make it unconditional
Section 16 Phase 8 — add the four systems to integration checklist
Section 17.5 AROS icon mapping table — add 4 rows
Section 17.5 custom icon prompts — add generation prompts for each new system
Section 17.7 asset file structure — add 4 new icon files
---

## Table of Contents

**Part I — Core Theme System**

1. [Design Principles](#1-design-principles)
2. [Architecture & Layer Integration](#2-architecture--layer-integration)
3. [Theme Data Model](#3-theme-data-model)
4. [Database Schema](#4-database-schema)
5. [Theme Engine](#5-theme-engine)
6. [Theme Manager UI](#6-theme-manager-ui)
7. [User Workflows](#7-user-workflows)
8. [Security & Sandboxing](#8-security--sandboxing)
9. [REST & WebSocket API](#9-rest--websocket-api)
10. [Implementation Roadmap — Core](#10-implementation-roadmap--core)

**Part II — APEX-OS Experience Mode**

11. [Why APEX-OS](#11-why-apex-os)
12. [Copyright & Naming](#12-copyright--naming)
13. [Mode Architecture & Renderer Decision](#13-mode-architecture--renderer-decision)
14. [ApexLoader Boot Sequence](#14-apexloader-boot-sequence)
15. [ApexShell Desktop & Window Manager](#15-apexshell-desktop--window-manager)
16. [Implementation Roadmap — APEX-OS](#16-implementation-roadmap--apex-os)

**Part III — Visual Identity**

17. [Image Generation & Icon Assets](#17-image-generation--icon-assets)

---

---

# Part I — Core Theme System

---

## 1. Design Principles

**Themes are data, not code.** Every theme compiles to CSS custom properties on `:root`. A theme cannot execute JavaScript, load external URLs, or override structural layout. This is enforced by L2 sanitisation on every write, not by convention in component code.

**Components consume tokens, never raw values.** Any component that hardcodes `color: #00d4aa` defeats every theme switch. Migration of all hardcoded colour, spacing, and typography values in the existing React component tree is a required deliverable in Phase 2, not an afterthought.

**Validation is server-enforced, client-advisory.** The editor provides instant WCAG contrast feedback and schema hints, but none of that is trusted. L2 runs the full validation pipeline — schema check, CSS sanitisation, WCAG AA contrast pairs — before storing anything. A theme that passes client checks but fails server validation is rejected.

**Inheritance is resolved at read time, stored sparse at write time.** A child theme stores only its overrides. The resolved theme (parent values merged with child overrides) is computed on request and cached. Circular `extends` chains are detected and rejected as hard errors before any resolution is attempted.

**APEX-OS mode is a feature flag, not the default.** The standard React SPA ships and stabilises first. APEX-OS mode is built on top of a proven theme system. The PhaseI/II/III ordering in the roadmap enforces this.

**Stub components are not surface-mapped.** The APEX v1.0.0 architecture explicitly marks `Heartbeat`, `Soul`, `Governance`, and `Moltbook` as `NOT ACTIVE - stub`. They do not appear in any theme surface mapping, APEX-OS window mapping, or icon assignment. They will be added when they ship.

---

## 2. Architecture & Layer Integration

### 2.1 Layer Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│  L6: React UI  (React 18, TypeScript, Tailwind, Zustand)         │
│                                                                  │
│  ThemeProvider (React Context)                                   │
│  ├── ThemeRegistry     all installed themes (summaries only)     │
│  ├── ActiveTheme       current fully-resolved theme              │
│  ├── StagedTheme       preview; auto-reverts after 30 seconds    │
│  └── UserOverrides     per-user micro-adjustments (persisted)    │
│             │                                                    │
│             ▼  CSS custom properties on :root (prepended)        │
│  Component Library — Chat, Skills, Files, Kanban, Settings,      │
│  TaskSidebar, ProcessGroup, ConfirmationGate, MemoryViewer,      │
│  KanbanBoard, AuditLog, DecisionJournal, ChannelManager          │
│  ALL values consumed via CSS variables — zero hardcoded colours  │
└────────────────────────┬─────────────────────────────────────────┘
                         │  HTTP + HMAC-SHA256 / WebSocket
┌────────────────────────▼─────────────────────────────────────────┐
│  L2: Router (Rust / Axum)  —  ThemeService                       │
│  ├── CSS sanitisation        lightningcss AST — not string match  │
│  ├── WCAG AA contrast pairs  enforced server-side on every write  │
│  ├── Inheritance resolver    typed, cycle-detected                │
│  ├── Resource limit checks   size, asset count, nesting depth     │
│  └── Import/export pipeline  checksum verify → sanitise → store  │
└────────────────────────┬─────────────────────────────────────────┘
                         │  sqlx
┌────────────────────────▼─────────────────────────────────────────┐
│  L3: Memory (SQLite via sqlx)                                    │
│  ├── themes            registry + validated definitions          │
│  ├── theme_assets      fonts, images, icons (blobs/paths)        │
│  ├── user_theme_prefs  active theme, overrides, auto-switch      │
│  └── theme_audit_log   append-only change history                │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 Theme Data Flow

```
[User edits theme in UI]
        │
        ▼
[Client validates locally]  ───►  [Live preview < 100ms via CSS var swap]
        │
        ▼  POST /api/v1/themes  (HMAC-signed)
[L2: Schema validation]     fail ──► structured error response
        │
[L2: CSS sanitisation]      fail ──► sanitisation error with rule location
        │
[L2: WCAG AA check]         fail ──► list of failing pairs + actual ratios
        │
[L2: Resource limits]       fail ──► limit violation detail
        │  all pass
[Store in SQLite (L3)]
        │
        ├──► WebSocket broadcast:  theme:created + compiled CSS
        │         └──► All connected L6 clients hot-reload in < 100ms
        │
        └──► Audit log entry appended (actor, action, timestamp INTEGER ms)
```

---

## 3. Theme Data Model

### 3.1 Core Interface

```typescript
interface Theme {
  // Identity
  id:          string;   // ULID — server-generated on create
  name:        string;   // Max 64 chars
  description: string;   // Max 256 chars
  version:     string;   // Semver
  author:      string;   // User ID or "apex-team"
  created_at:  number;   // Unix epoch ms (INTEGER — never string)
  updated_at:  number;   // Unix epoch ms (INTEGER — never string)

  // Inheritance
  extends?:          string;                     // Parent theme ULID
  override_strategy: 'merge' | 'replace';        // Default: 'merge'

  // Classification
  // Note: 'system' is NOT a valid category — built-in covers system themes
  category: 'built-in' | 'user' | 'community';
  tags:     string[];

  // Visual systems (all required on root themes; partial on child themes)
  color_system:      ColorSystem;
  typography_system: TypographySystem;
  spacing_system:    SpacingSystem;
  component_styles:  ComponentStyles;

  // Assets
  assets: ThemeAsset[];

  // Constraints
  is_editable:       boolean;  // false for built-in themes
  is_sharable:       boolean;
  min_apex_version:  string;
  max_apex_version?: string;
}
```

### 3.2 Colour System

All colour values are semantic — components reference token names, never hex values.

```typescript
interface ColorSystem {
  // Full 9-step scales
  primary:   ColorScale;   // Brand, accents, primary CTAs
  secondary: ColorScale;   // Secondary actions
  neutral:   ColorScale;   // Backgrounds, borders, disabled states

  semantic: {
    success: ColorScale;
    warning: ColorScale;
    error:   ColorScale;
    info:    ColorScale;
  };

  background: {
    base:     string;   // Page background, deepest layer
    elevated: string;   // Cards, panels, sidebar
    overlay:  string;   // Modals, dropdowns, popovers
  };

  text: {
    primary:   string;   // Main content — validated against background.base
    secondary: string;   // Labels, metadata, timestamps
    disabled:  string;   // Inactive elements
    inverse:   string;   // Text on coloured backgrounds
  };

  interactive: {
    default:  string;
    hover:    string;
    active:   string;
    focus:    string;   // Must satisfy WCAG 3:1 non-text contrast
    disabled: string;
  };

  // APEX-specific agent state colours
  // Maps directly to agent state in L5 ExecutionStream
  agent: {
    idle:     string;   // Waiting for input
    active:   string;   // Task running — animated in UI
    thinking: string;   // LLM call in progress (DeepTaskWorker)
    alert:    string;   // Requires user attention (ConfirmationGate)
  };

  // Process step badge colours (maps to ProcessGroup step types in Chat.tsx)
  // All validated against background.elevated
  badges: {
    gen: string;   // LLM generation / PLAN step
    use: string;   // Skill invocation (SkillWorker)
    exe: string;   // Shell / VM execution (Firecracker/Docker)
    www: string;   // web.search / web.fetch
    sub: string;   // Subagent spawn
    mem: string;   // Memory read/write (NarrativeService)
    aud: string;   // Audit event (T2/T3 confirmation)
  };

  // Confirmation tier indicator colours (ConfirmationGate.tsx)
  tiers: {
    t1: string;   // Tap — neutral
    t2: string;   // Type — amber
    t3: string;   // TOTP — red
  };

  // Optional: retro chrome overrides (APEX-OS theme only)
  chrome?: {
    title_bar_active:   GradientDef;
    title_bar_inactive: GradientDef;
    button_raised:      BevelDef;
    button_depressed:   BevelDef;
  };
}

interface ColorScale {
  50:  string;  100: string;  200: string;  300: string;  400: string;
  500: string;  600: string;  700: string;  800: string;  900: string;
}

interface GradientDef {
  type:   'linear' | 'radial';
  stops:  string[];
  angle?: number;
}

interface BevelDef {
  highlight: string;
  shadow:    string;
  depth:     number;
}
```

### 3.3 Typography System

Sizes are aligned with the APEX GUI Design Specification. No divergence.

```typescript
interface TypographySystem {
  font_families: {
    sans:      string;   // UI text, body
    mono:      string;   // Code blocks, ProcessGroup step detail, task IDs
    display?:  string;   // Headings; falls back to sans
  };

  font_sizes: {
    xs:    string;   // 11px — badges, timestamps, tier labels
    sm:    string;   // 13px — secondary labels, sidebar items
    base:  string;   // 15px — body, chat messages
    lg:    string;   // 17px — panel headings
    xl:    string;   // 21px — section titles
    '2xl': string;   // 27px — page titles, empty state headings
  };

  font_weights: { normal: number; medium: number; semibold: number; bold: number; };
  line_heights: { tight: number; normal: number; relaxed: number; };
  letter_spacing: { tight: string; normal: string; wide: string; };
}
```

### 3.4 Spacing System

Base unit 4px, matching the GUI spec.

```typescript
interface SpacingSystem {
  base_unit:   number;   // 4px
  scale_ratio: number;   // 2

  space: {
    0: string; 1: string; 2: string; 3: string; 4: string;
    5: string; 6: string; 7: string; 8: string; 9: string;
    // 0 · 4px · 8px · 12px · 16px · 24px · 32px · 48px · 64px · 96px
  };

  border_radius: {
    none: string; sm: string; base: string;
    md: string;   lg: string; full: string;
  };

  shadows: {
    none: string; sm: string; base: string;
    md: string;   lg: string; inner: string;
  };
}
```

### 3.5 Component Styles

All string values are token references (`--color-primary-500`, `--space-4`, etc.). Raw hex/rgb values are rejected at server validation.

```typescript
interface ComponentStyles {
  button:  ButtonStyles;
  input:   InputStyles;
  panel:   PanelStyles;
  card:    CardStyles;
  badge:   BadgeStyles;
  modal:   ModalStyles;
}

interface ButtonStyles {
  primary_bg: string;    primary_text: string;  primary_border: string;
  secondary_bg: string;  secondary_text: string; secondary_border: string;
  danger_bg: string;     danger_text: string;   // T3 destructive actions
  padding_x: string;     padding_y: string;
  border_radius: string; font_weight: string;
  bevel?: boolean;       // APEX-OS only
}

interface InputStyles {
  bg: string;  text: string;  border: string;
  border_focus: string;  placeholder: string;
  border_radius: string;  padding_x: string;  padding_y: string;
  inset?: boolean;   // APEX-OS: render as sunken gadget
}

interface PanelStyles {
  bg: string;  border: string;  border_radius: string;  shadow: string;
  bevel?: boolean;            // APEX-OS
  gradient_title?: boolean;   // APEX-OS
}

interface CardStyles {
  bg: string;  border: string;  border_radius: string;
  shadow: string;  hover_shadow: string;
}

interface BadgeStyles {
  border_radius: string;  padding_x: string;  padding_y: string;
  font_size: string;  font_weight: string;
}

interface ModalStyles {
  bg: string;  border: string;  border_radius: string;
  shadow: string;  overlay_bg: string;
}
```

### 3.6 Theme Asset

```typescript
interface ThemeAsset {
  id:       string;   // ULID
  type:     'font' | 'image' | 'icon' | 'css';
  name:     string;
  format:   string;   // 'woff2', 'woff', 'png', 'svg', 'css'
  data?:    string;   // Base64 for assets < 50 KB
  path?:    string;   // Row reference in theme_assets for larger assets
  license?: string;   // Required for embedded fonts
  checksum: string;   // SHA-256 — NOT NULL
}
```

### 3.7 Export Bundle

```typescript
interface ThemeBundle {
  manifest: {
    name: string;  version: string;  author: string;
    description: string;  apex_version: string;
    exported_at: number;   // Unix epoch ms (INTEGER — not string)
  };
  definition: Theme;
  assets:     ThemeAsset[];
  checksum:   string;   // SHA-256 of JSON.stringify({ definition, assets })
}
```

---

## 4. Database Schema

All timestamps: `INTEGER NOT NULL` (Unix epoch milliseconds). SQLite has no BOOLEAN — boolean columns use `INTEGER NOT NULL CHECK(col IN (0,1))`. A partial unique index enforces exactly one active theme.

```sql
-- Theme registry
CREATE TABLE themes (
  id                TEXT    NOT NULL PRIMARY KEY,
  name              TEXT    NOT NULL,
  description       TEXT,
  version           TEXT    NOT NULL,
  author            TEXT    NOT NULL,
  created_at        INTEGER NOT NULL,           -- Unix ms
  updated_at        INTEGER NOT NULL,           -- Unix ms
  extends           TEXT    REFERENCES themes(id) ON DELETE SET NULL,
  override_strategy TEXT    NOT NULL DEFAULT 'merge'
                            CHECK(override_strategy IN ('merge','replace')),
  category          TEXT    NOT NULL
                            CHECK(category IN ('built-in','user','community')),
  tags              TEXT,                       -- JSON array
  definition        TEXT    NOT NULL,           -- Validated Theme JSON
  is_active         INTEGER NOT NULL DEFAULT 0
                            CHECK(is_active IN (0,1)),
  is_editable       INTEGER NOT NULL DEFAULT 1
                            CHECK(is_editable IN (0,1)),
  is_sharable       INTEGER NOT NULL DEFAULT 1
                            CHECK(is_sharable IN (0,1)),
  min_apex_version  TEXT,
  max_apex_version  TEXT,
  checksum          TEXT    NOT NULL            -- SHA-256 of definition; NOT NULL
);

-- Exactly one theme active at any time
CREATE UNIQUE INDEX idx_one_active_theme
  ON themes(is_active) WHERE is_active = 1;

-- Theme assets: fonts, images, icons, supplementary CSS
CREATE TABLE theme_assets (
  id           TEXT    NOT NULL PRIMARY KEY,
  theme_id     TEXT    NOT NULL REFERENCES themes(id) ON DELETE CASCADE,
  name         TEXT    NOT NULL,
  type         TEXT    NOT NULL CHECK(type IN ('font','image','icon','css')),
  format       TEXT,
  data         BLOB,
  path         TEXT,
  license_text TEXT,
  checksum     TEXT    NOT NULL,               -- SHA-256 of data; NOT NULL
  created_at   INTEGER NOT NULL
);

-- Exactly one of data or path per row
CREATE TRIGGER trg_asset_source
  BEFORE INSERT ON theme_assets
  WHEN (NEW.data IS NULL AND NEW.path IS NULL)
    OR (NEW.data IS NOT NULL AND NEW.path IS NOT NULL)
BEGIN
  SELECT RAISE(ABORT,'theme_assets: exactly one of data or path must be set');
END;

-- User theme preferences (single-user: user_id = 'default')
CREATE TABLE user_theme_prefs (
  user_id            TEXT    NOT NULL PRIMARY KEY,
  active_theme_id    TEXT    REFERENCES themes(id) ON DELETE SET NULL,
  custom_overrides   TEXT,
  auto_switch_mode   TEXT    NOT NULL DEFAULT 'system'
                             CHECK(auto_switch_mode IN ('manual','time','system')),
  auto_switch_config TEXT,
  -- JSON: { "sunset": "20:00", "sunrise": "06:30",
  --         "dark_theme_id": "...", "light_theme_id": "..." }
  updated_at         INTEGER NOT NULL
);

-- Append-only audit log — no UPDATE/DELETE from application layer
CREATE TABLE theme_audit_log (
  id             TEXT    NOT NULL PRIMARY KEY,
  theme_id       TEXT    REFERENCES themes(id),
  action         TEXT    NOT NULL
                         CHECK(action IN (
                           'created','modified','activated',
                           'deleted','exported','imported'
                         )),
  previous_state TEXT,                         -- JSON snapshot before change
  actor          TEXT    NOT NULL,             -- 'user','system','skill:name'
  reason         TEXT,
  timestamp      INTEGER NOT NULL
);
```

---

## 5. Theme Engine

### 5.1 CSS Variable Compilation

Every token category is emitted. The original spec only emitted ~6 of 40+ categories, leaving most components without their variables.

```typescript
function compileTheme(theme: ResolvedTheme): string {
  const vars: Record<string, string> = {};

  // Colour scales
  for (const [step, val] of Object.entries(theme.color_system.primary))
    vars[`--color-primary-${step}`] = val;
  for (const [step, val] of Object.entries(theme.color_system.secondary))
    vars[`--color-secondary-${step}`] = val;
  for (const [step, val] of Object.entries(theme.color_system.neutral))
    vars[`--color-neutral-${step}`] = val;
  for (const [name, scale] of Object.entries(theme.color_system.semantic))
    for (const [step, val] of Object.entries(scale))
      vars[`--color-${name}-${step}`] = val;

  // Surfaces
  vars['--color-bg-base']     = theme.color_system.background.base;
  vars['--color-bg-elevated'] = theme.color_system.background.elevated;
  vars['--color-bg-overlay']  = theme.color_system.background.overlay;

  // Text
  for (const [k, v] of Object.entries(theme.color_system.text))
    vars[`--color-text-${k}`] = v;

  // Interactive states
  for (const [k, v] of Object.entries(theme.color_system.interactive))
    vars[`--color-interactive-${k}`] = v;

  // Agent states (L5 → UI via WebSocket ExecutionStream)
  for (const [k, v] of Object.entries(theme.color_system.agent))
    vars[`--color-agent-${k}`] = v;

  // Process step badges (ProcessGroup.tsx)
  for (const [k, v] of Object.entries(theme.color_system.badges))
    vars[`--color-badge-${k}`] = v;

  // Confirmation tier colours (ConfirmationGate.tsx)
  for (const [k, v] of Object.entries(theme.color_system.tiers))
    vars[`--color-tier-${k}`] = v;

  // Typography
  vars['--font-sans']    = theme.typography_system.font_families.sans;
  vars['--font-mono']    = theme.typography_system.font_families.mono;
  vars['--font-display'] = theme.typography_system.font_families.display
    ?? theme.typography_system.font_families.sans;
  for (const [k, v] of Object.entries(theme.typography_system.font_sizes))
    vars[`--text-${k}`] = v;
  for (const [k, v] of Object.entries(theme.typography_system.font_weights))
    vars[`--font-weight-${k}`] = String(v);
  for (const [k, v] of Object.entries(theme.typography_system.line_heights))
    vars[`--leading-${k}`] = String(v);
  for (const [k, v] of Object.entries(theme.typography_system.letter_spacing))
    vars[`--tracking-${k}`] = v;

  // Spacing
  for (const [k, v] of Object.entries(theme.spacing_system.space))
    vars[`--space-${k}`] = v;
  for (const [k, v] of Object.entries(theme.spacing_system.border_radius))
    vars[`--radius-${k}`] = v;
  for (const [k, v] of Object.entries(theme.spacing_system.shadows))
    vars[`--shadow-${k}`] = v;

  // Component token aliases
  const b = theme.component_styles.button;
  vars['--btn-primary-bg']     = tok(b.primary_bg);
  vars['--btn-primary-text']   = tok(b.primary_text);
  vars['--btn-secondary-bg']   = tok(b.secondary_bg);
  vars['--btn-secondary-text'] = tok(b.secondary_text);
  vars['--btn-danger-bg']      = tok(b.danger_bg);
  vars['--btn-danger-text']    = tok(b.danger_text);
  vars['--btn-padding-x']      = tok(b.padding_x);
  vars['--btn-padding-y']      = tok(b.padding_y);
  vars['--btn-radius']         = tok(b.border_radius);

  const i = theme.component_styles.input;
  vars['--input-bg']           = tok(i.bg);
  vars['--input-text']         = tok(i.text);
  vars['--input-border']       = tok(i.border);
  vars['--input-border-focus'] = tok(i.border_focus);
  vars['--input-radius']       = tok(i.border_radius);

  const declarations = Object.entries(vars)
    .map(([k, v]) => `  ${k}: ${v};`)
    .join('\n');

  return `:root {\n${declarations}\n}`;
}

// Token ref ("--color-primary-500") → "var(--color-primary-500)"
// Raw values ("transparent", "inherit") pass through unchanged
function tok(token: string): string {
  return token.startsWith('--') ? `var(${token})` : token;
}
```

### 5.2 CSS Injection (Prepend)

The style element is **prepended** to `<head>`. Appending causes component stylesheets loaded earlier to silently win — the opposite of intended behaviour.

```typescript
function applyThemeCss(css: string): void {
  const ID = 'apex-theme-variables';
  let el = document.getElementById(ID) as HTMLStyleElement | null;
  if (!el) {
    el = document.createElement('style');
    el.id = ID;
    document.head.prepend(el);   // PREPEND — must underlie all component CSS
  }
  el.textContent = css;
}
```

### 5.3 Inheritance Resolution (Typed, Cycle-Detected)

```typescript
function resolveTheme(
  themeId:  string,
  registry: ReadonlyMap<string, Theme>
): ResolvedTheme {
  return resolveInner(themeId, registry, new Set<string>());
}

function resolveInner(
  id:       string,
  registry: ReadonlyMap<string, Theme>,
  visited:  Set<string>
): ResolvedTheme {
  if (visited.has(id))
    throw new Error(`Theme inheritance cycle: ${[...visited, id].join(' → ')}`);
  const theme = registry.get(id);
  if (!theme) throw new Error(`Theme not found: ${id}`);
  if (!theme.extends) return theme as ResolvedTheme;
  visited.add(id);
  const parent = resolveInner(theme.extends, registry, visited);
  return theme.override_strategy === 'replace'
    ? { ...parent, ...theme, extends: undefined } as ResolvedTheme
    : deepMerge(parent as Record<string, unknown>, theme as Record<string, unknown>,
        new Set(['extends','override_strategy','id','created_at'])) as ResolvedTheme;
}

function deepMerge(
  parent:   Record<string, unknown>,
  child:    Record<string, unknown>,
  skipKeys: ReadonlySet<string> = new Set()
): Record<string, unknown> {
  const out = structuredClone(parent);
  for (const [k, v] of Object.entries(child)) {
    if (skipKeys.has(k) || v === undefined) continue;
    const p = out[k];
    if (Array.isArray(v) && Array.isArray(p))
      out[k] = [...p, ...v];                   // tags, assets: child appends
    else if (v !== null && typeof v === 'object' && !Array.isArray(v)
          && p !== null && typeof p === 'object' && !Array.isArray(p))
      out[k] = deepMerge(p as Record<string, unknown>, v as Record<string, unknown>);
    else
      out[k] = v;
  }
  return out;
}
```

### 5.4 React ThemeProvider

Note: the original had a name collision — `previewTheme` was both a field and a method. Fixed here as `stagedTheme` (field) and `startPreview` (method).

```typescript
interface ThemeContextType {
  availableThemes: ThemeSummary[];
  activeTheme:     ResolvedTheme;
  stagedTheme:     ResolvedTheme | null;   // renamed from previewTheme (collision)
  userOverrides:   Partial<Theme>;

  activateTheme:  (id: string) => Promise<void>;
  startPreview:   (id: string) => Promise<void>;   // renamed from previewTheme
  cancelPreview:  () => void;
  createTheme:    (baseId: string | null, edits: Partial<Theme>) => Promise<Theme>;
  updateTheme:    (id: string, edits: Partial<Theme>) => Promise<Theme>;
  deleteTheme:    (id: string) => Promise<void>;
  exportTheme:    (id: string) => Promise<ThemeBundle>;
  importTheme:    (bundle: ThemeBundle) => Promise<Theme>;
  setOverride:    (path: string, value: string) => void;
  resetOverrides: () => void;
}
```

### 5.5 WCAG Contrast Pairs (Server-Enforced)

All pairs below are checked for every theme. Minimum ratios: 4.5:1 normal text, 3:1 large text / UI components / focus indicators.

| Pair | Min ratio |
|---|---|
| `text.primary` vs `background.base` | 4.5:1 |
| `text.primary` vs `background.elevated` | 4.5:1 |
| `text.secondary` vs `background.base` | 4.5:1 |
| `text.secondary` vs `background.elevated` | 4.5:1 |
| `text.inverse` vs `primary.500` | 4.5:1 |
| `interactive.focus` vs `background.base` | 3:1 (non-text) |
| Each `badges.*` vs `background.elevated` | 3:1 (non-text) |
| Each `tiers.*` vs `background.elevated` | 3:1 (non-text) |

Failures are returned as a structured array with pair name, actual ratio, and required ratio. Client renders them inline in the colour editor.

---

## 6. Theme Manager UI

The Theme Manager lives in **Settings → Appearance** as a full-page section, not a modal. It maps to the existing `Settings.tsx` tab pattern.

### 6.1 Layout

```
┌────────────────────────────────────────────────────────────────────────┐
│  Settings › Appearance                                                 │
├──────────────────┬─────────────────────────────────────────────────────┤
│                  │                                                     │
│  INSTALLED       │  PREVIEW & EDITOR                                   │
│                  │                                                     │
│  [Search...]     │  ┌──────────────────────────────────────────────┐  │
│                  │  │                                              │  │
│  ▶ Built-in (3)  │  │    Live Preview Canvas                       │  │
│    ● APEX Tech   │  │    (interactive, updates < 100ms)            │  │
│    ○ APEX-OS     │  │                                              │  │
│    ○ Hi Contrast │  └──────────────────────────────────────────────┘  │
│                  │                                                     │
│  ▶ My Themes (2) │  [Colors] [Typography] [Spacing] [Components]      │
│    ○ Neon        │  [Code Editor ↗]  [Diff vs Parent]                 │
│    ○ Minimal     │                                                     │
│    [+ New]       │  ┌──────────────────────────────────────────────┐  │
│                  │  │  Color Editor                                │  │
│  ▶ Imported (1)  │  │  Primary    [████] #00d4aa                   │  │
│    ○ Nord        │  │  Background [████] #0f1117                   │  │
│                  │  │  [Generate full scale from primary]          │  │
│                  │  │                                              │  │
│                  │  │  Contrast check                              │  │
│                  │  │  ✓ text.primary / bg-base     12.4:1 AA ✓   │  │
│                  │  │  ✓ text.secondary / bg-base    5.1:1 AA ✓   │  │
│                  │  │  ✗ badge-gen / bg-elevated     2.8:1 AA ✗   │  │
│                  │  └──────────────────────────────────────────────┘  │
│                  │                                                     │
│                  │  [Revert]  [Save Draft]  [Activate]  [Export]      │
│                  │                                                     │
└──────────────────┴─────────────────────────────────────────────────────┘
```

### 6.2 Editor Modes

**Visual (default)** — Colour pickers with per-pair WCAG feedback; failing pairs block [Activate]. Typography scale preview with live sample text. Spacing visualiser. Per-component rendering in the Components tab.

**Code Editor** — Direct JSON editing of the Theme structure. Schema validation inline. Auto-complete for token references (`--color-...`, `--space-...`). Preview updates on valid JSON only.

**Diff View** — Side-by-side comparison against parent theme. Values identical to parent shown in grey. Per-field "revert to parent". Full reset button.

### 6.3 Preview Canvas

```
┌─────────────────────────────────────────────────────────────────────┐
│  Preview  [Chat] [Tasks] [Kanban] [Skills] [Confirmation]           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─ ProcessGroup ──────────────────────────────────────────────┐   │
│  │ ▼ Refactor auth module  •  3 steps  •  $0.008  •  T1       │   │
│  │   [GEN] Planning         [USE] repo.search                  │   │
│  │   [EXE] python: ast.py                                      │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│  [Primary]  [Secondary]  [Danger ⚠]                                │
│  ● idle  ◉ active  ◎ thinking  ◆ alert                             │
│                                                                     │
│  ┌──────────────────────────────────────────────────────┐          │
│  │ Input field with focus ring                          │          │
│  └──────────────────────────────────────────────────────┘          │
│                                                                     │
│  ┌── T2 ConfirmationGate ──────────────────────────────────┐       │
│  │ ⚠  push to origin/main — type "push to main":           │       │
│  │ [________________________]  [Confirm]  [Cancel]          │       │
│  └──────────────────────────────────────────────────────────┘       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 7. User Workflows

### 7.1 Creating a New Theme

```
1. Click [+ New] in the theme list
2. Choose base:
     "Extend APEX Technical"  ← recommended; inherits all tokens
     "Extend APEX-OS"
     "Extend High Contrast"
     "Start from scratch"     ← must define all required tokens
3. Enter name and description
4. Visual editor opens pre-populated with parent values
5. Adjust colours — WCAG pairs update inline; failures block [Activate]
6. Adjust Typography, Spacing, Components as needed
7. [Save Draft]  — stores to SQLite, does not activate
   [Activate]    — stores and broadcasts CSS to all clients in < 100ms
8. Audit log entry appended
```

### 7.2 Customising a Built-in Theme

Built-in themes are `is_editable: false`. Clicking them shows [Customise] rather than an edit form.

```
1. Select a built-in (e.g. "APEX Technical")
2. Click [Customise]
   → Creates a new user theme inheriting the built-in
   → Diff view opens immediately showing zero overrides
3. Adjust values; diff view highlights overrides
4. [Activate] applies the child theme
5. Built-in remains unchanged
6. Per-field [Revert to parent] removes individual overrides
```

### 7.3 Importing a Community Theme

```
1. Drop .apex-theme file or click [Import]
2. L2 pipeline (server-side, in order):
   a. Parse JSON
   b. Verify bundle checksum (fail immediately on mismatch)
   c. Validate against Theme schema
   d. CSS sanitise all assets (lightningcss AST)
   e. WCAG AA check
   f. Resource limits check
   g. apex_version compatibility check
3. Failures → structured error with specific reason per step
4. Success → preview in sandboxed iframe
5. User confirms (T2: type "import [theme name]")
6. Stored as category: 'community'; audit log appended
```

### 7.4 Auto-Switch

Configured in Settings → Appearance → Auto-switch:
- **Manual** — explicit user action only
- **System** — follows OS `prefers-color-scheme`
- **Time-based** — dark/light themes at configured sunrise/sunset times

---

## 8. Security & Sandboxing

### 8.1 CSS Sanitisation

String-matching on property names is insufficient and was used in the v1 spec. It is bypassed by case variation (`URL()` vs `url()`), Unicode escapes, or shorthand values embedding `url()`. The correct approach parses the CSS AST and inspects typed node values.

```rust
use lightningcss::stylesheet::{StyleSheet, ParserOptions, PrinterOptions};
use lightningcss::rules::CssRule;

pub fn sanitise_css(input: &str) -> Result<String, SanitisationError> {
    let mut sheet = StyleSheet::parse(input, ParserOptions::default())
        .map_err(|e| SanitisationError::Parse(e.to_string()))?;

    sheet.rules.0.retain_mut(|rule| match rule {
        CssRule::Import(_) => false,   // block @import unconditionally

        CssRule::FontFace(ff) => {
            // Allow @font-face only with data: URIs or local()
            ff.declarations.iter().all(|d| !decl_has_external_url(d))
        }

        CssRule::Keyframes(_) | CssRule::Media(_) | CssRule::Supports(_) => true,

        CssRule::Style(style) => {
            let sel = style.selectors.to_string();
            let blocked = ["script","link[","meta","<style","head ","body "];
            if blocked.iter().any(|b| sel.contains(b)) { return false; }
            style.declarations.declarations.retain(|d| !decl_has_external_url(d));
            true
        }

        _ => false,   // default DENY for unknown/future at-rules
    });

    sheet.to_css(PrinterOptions::default())
        .map(|r| r.code)
        .map_err(|e| SanitisationError::Print(e.to_string()))
}

fn is_external_url(url: &str) -> bool {
    let s = url.trim().trim_matches('"').trim_matches('\'');
    if s.starts_with("data:") { return false; }
    if s.starts_with("local(") { return false; }
    true   // http, https, //, relative paths — all external
}
```

### 8.2 Permission Model

Aligned to APEX v1.0.0 tier model (T0–T3 with TOTP):

| Action | Tier | Rationale |
|---|---|---|
| Activate any installed theme | T0 | Cosmetic; immediately reversible |
| Preview any theme (30s auto-revert) | T0 | Temporary |
| Create new theme | T1 | Tap to confirm; new SQLite row |
| Modify own user theme | T1 | Tap to confirm |
| Export any theme | T0 | Read-only |
| Delete own user theme | T1 | Tap to confirm |
| Import community theme | T2 | External code; type "import [name]" |
| Agent proposes theme change | T2 | Agent suggests; user types confirmation |
| Modify built-in theme | T3 | TOTP required; should not be needed in practice |

### 8.3 Resource Limits

```rust
pub struct ThemeLimits {
    pub max_definition_bytes:    usize,   // 512 KB
    pub max_assets_total_bytes:  usize,   // 10 MB
    pub max_asset_count:         usize,   // 50
    pub max_font_count:          usize,   // 5
    pub max_image_count:         usize,   // 20
    pub max_css_nesting_depth:   usize,   // 8
    pub max_css_variable_count:  usize,   // 500
    pub max_inheritance_depth:   usize,   // 5
    pub require_wcag_aa:         bool,    // true
    pub forbid_external_urls:    bool,    // true
}
```

---

## 9. REST & WebSocket API

### 9.1 REST Endpoints

All requests are HMAC-signed (`X-APEX-Signature` + `X-APEX-Timestamp`), consistent with the existing API layer in `src/api.rs`.

```
GET    /api/v1/themes                    List (filters: category, tag, author)
POST   /api/v1/themes                    Create — T1
GET    /api/v1/themes/:id                Get full resolved definition
PUT    /api/v1/themes/:id                Update — T1 (user) / T3 (built-in)
DELETE /api/v1/themes/:id                Delete — T1 (user) / T3 (built-in)

POST   /api/v1/themes/:id/activate       Apply immediately — T0
POST   /api/v1/themes/:id/preview        Stage 30s preview — T0
DELETE /api/v1/themes/preview            Cancel staged preview

POST   /api/v1/themes/import             Upload .apex-theme bundle — T2
GET    /api/v1/themes/:id/export         Download .apex-theme bundle — T0
GET    /api/v1/themes/:id/export/css     Compiled CSS variables only — T0

GET    /api/v1/user/theme-prefs          Get active theme + overrides
PUT    /api/v1/user/theme-prefs          Update preferences
PATCH  /api/v1/user/theme-prefs/overrides Apply/reset micro-overrides

POST   /api/v1/themes/validate           Validate JSON+CSS without storing — T0
POST   /api/v1/themes/generate-palette   Full ColorScale from one hex input — T0
POST   /api/v1/themes/check-contrast     WCAG ratios for colour pair set — T0
```

### 9.2 WebSocket Events

Delivered via the existing WebSocket server (`src/websocket.rs`):

```typescript
// Server → Client
interface ThemeServerEvents {
  'theme:activated':      { theme_id: string; name: string; css: string };
  'theme:created':        { theme_id: string; author: string; name: string };
  'theme:updated':        { theme_id: string; changed_paths: string[]; css: string };
  'theme:deleted':        { theme_id: string };
  'theme:preview:start':  { theme_id: string; css: string; expires_at: number };
  'theme:preview:end':    { reverted_to: string };
}

// Client → Server
interface ThemeClientCommands {
  'theme:preview':        { theme_id: string };
  'theme:preview:commit': { activate: boolean };
  'theme:live_edit':      { path: string; value: string };
  // path: dot-notation in Theme JSON e.g. "color_system.primary.500"
}
```

---

## 10. Implementation Roadmap — Core

Each phase is a hard prerequisite for the next. APEX-OS mode does not start until Phase 4 is complete and stable.

### Phase 1 — Data Foundation (Weeks 1–3)

- [ ] SQLite schema: INTEGER timestamps, partial unique index, asset trigger
- [ ] Complete TypeScript interfaces (Theme, all sub-interfaces, ThemeBundle)
- [ ] CSS compiler emitting all token categories (Sections 5.1)
- [ ] Typed inheritance resolver with cycle detection (Section 5.3)
- [ ] Rust `ThemeLimits` with `Default`
- [ ] Rust CSS sanitiser using `lightningcss` AST (Section 8.1)
- [ ] WCAG contrast validator covering all required pairs (Section 5.5)
- [ ] Basic REST API: CRUD + activate + validate + generate-palette

### Phase 2 — UI Integration (Weeks 4–6)

- [ ] `ThemeProvider` with corrected context interface (no name collision)
- [ ] `applyThemeCss` using `document.head.prepend`
- [ ] Theme selector in Settings → Appearance
- [ ] WebSocket handlers for `theme:activated`, `theme:updated` hot-reload
- [ ] Built-in themes: APEX Technical, High Contrast
- [ ] Auto-switch: system mode (`prefers-color-scheme`)
- [ ] **Component migration pass**: audit all L6 components; replace every hardcoded colour/spacing/typography value with a CSS variable reference. This is blocking — the theme system has no effect until this is done.

### Phase 3 — Theme Editor (Weeks 7–10)

- [ ] Visual colour editor with per-pair WCAG feedback (failing pairs block activation)
- [ ] Typography and spacing controls with live preview
- [ ] Per-component style tab with live component rendering
- [ ] Preview canvas (ProcessGroup, badges, buttons, input, ConfirmationGate, agent states)
- [ ] Code editor mode (JSON + schema validation + token auto-complete)
- [ ] Diff view vs. parent with per-field revert
- [ ] Save Draft / Activate / Export flows

### Phase 4 — Advanced Features (Weeks 11–14)

- [ ] Import pipeline with full server-side validation and T2 confirmation
- [ ] APEX-OS built-in theme (CSS mode — standard component tree, no window manager)
- [ ] Auto-switch: time-based configuration
- [ ] `check-contrast` utility endpoint
- [ ] Palette import from Coolors, Figma token JSON
- [ ] Community theme import (T2, sandboxed preview)

---

---

# Part II — APEX-OS Experience Mode

---

## 11. Why APEX-OS

The Amiga was the first mass-market computer to do pre-emptive multitasking, hardware-accelerated graphics, and true multimedia — in 1985. Its operating environment, Workbench, was designed around an honest model: the computer is yours, your windows go where you put them, your files are visible on the desktop, and the machine responds immediately. Every action was instant. There was no "loading…" for things that should not take time.

APEX maps onto this model precisely:
- It runs locally, on your hardware, for you alone
- Tasks execute immediately or transparently — no opaque cloud
- Skills are visible, discoverable, invocable from the desktop
- The agent loop is observable step by step
- The interface should feel personal, not generic

APEX-OS is the Workbench for an autonomous agent. It is not nostalgia. It is a claim that this model of computing — immediate, personal, visible — is correct, and that the Amiga got it right before the rest of the industry forgot.

---

## 12. Copyright & Naming

### 12.1 What We Cannot Use

The following are trademarked or owned by Cloanto Corporation (who hold Amiga IP):

- "Amiga", "AmigaOS", "Workbench" (as a product name)
- "Kickstart" (as a product name)
- The Boing ball logo
- The specific Commodore/Amiga colour scheme as a brand identity (the colours themselves are not protected, but the combined trade dress may be)

Using these names for an APEX product without a licence creates legal exposure.

### 12.2 What We CAN Use

- Concepts and metaphors: desktop, window, icon, drawer, screen — all generic computing terms
- AROS (AROS Research Operating System) is open-source under the AROS Public License (APL, MPL-based) and is a clean-room reimplementation with no Cloanto IP. Its artwork may be used per APL terms.
- Functional UI patterns: title bars, gadgets, bevel chrome, menu bars — these are UI concepts, not trademarks

### 12.3 APEX-OS Naming Scheme

All APEX-OS component names are original:

| Amiga term | APEX-OS equivalent | Notes |
|---|---|---|
| AmigaOS / Workbench | **APEX-OS** | The overall UI mode |
| Kickstart | **ApexLoader** | Boot sequence |
| Workbench (desktop) | **ApexShell** | Desktop environment |
| Intuition | **ApexWM** | Window manager |
| Wanderer | **ApexShell** | (same — AROS's Wanderer inspired our Shell) |
| Requester | **Dialog** | Generic; fine to use |
| Gadget | **Control** | Generic |
| Drawer | **Folder** | Generic |
| AppIcon | **DockIcon** | Generic |
| Screen | **Workspace** | Generic |
| Topaz font | **System font / ApexMono** | Use free pixel fonts (see Section 17) |

The aesthetic is unmistakably Amiga-inspired. The names are unambiguously APEX. The wink is intentional.

---

## 13. Mode Architecture & Renderer Decision

### 13.1 Two UI Modes

```
APEX L6 UI
│
├── Standard React SPA  (default)
│   Chat, Skills, Files, Kanban, Settings — standard layout
│
└── APEX-OS Experience Mode  (opt-in, Settings → Appearance → UI Mode)
    ApexLoader boot sequence
    ApexShell desktop with Folder/DockIcon grid
    ApexWM window manager
    All APEX surfaces mapped to windows
```

Emergency fallback: Hold **Shift** during page load to force Standard mode.

### 13.2 Renderer Decision: DOM (Not Canvas)

The earlier spec left this unresolved. It is resolved here.

**Use DOM rendering, not canvas.**

Canvas produces pixel-perfect Amiga visuals but: screen readers cannot traverse canvas; ARIA focus management requires complete reimplementation; all content types (markdown chat, code blocks, task tables, kanban cards) must be hand-drawn rather than reusing React components; accessibility testing is an order of magnitude harder; and React's reconciler cannot help at all.

DOM rendering with constrained CSS achieves the same visual result — bevel borders via `box-shadow`, title bar gradients via `background: linear-gradient`, pixel fonts via `@font-face` — while retaining React component reuse, ARIA, and the existing WebSocket/API infrastructure.

```typescript
interface ApexOsConfig {
  pixelScale:      1 | 2 | 3;           // 1 = authentic; 2/3 = HiDPI
  iconSize:        32 | 48 | 64;         // px; default 48
  fontMode:        'system' | 'pixel';   // pixel = embedded ApexMono
  snapToGrid:      boolean;              // default true
  gridSize:        number;               // px; default 16
  dragMode:        'outline' | 'full';   // outline = authentic; full = comfortable
  animationMode:   'none' | 'transitions'; // none = authentic
  skipLoader:      boolean;              // Skip ApexLoader, boot straight to ApexShell
  simpleMode:      boolean;              // Single window, tabs — accessibility mode
  iconSet:         'aros-gorilla' | 'aros-mason' | 'custom';
}
```

---

## 14. ApexLoader Boot Sequence

Displayed on first load of APEX-OS mode in a session. Skipped if `skipLoader: true`.

### 14.1 Visual

```
┌────────────────────────────────────────────────────────────────────────┐
│                                                                        │
│   ████████████████████████████████████████████████████████████████   │
│   █                                                              █   │
│   █   ◈◈ APEX-OS ◈◈                        Version 1.0.0        █   │
│   █                                                              █   │
│   █   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░         █   │
│   █   Loading: apex.autonomy.core                                █   │
│   █                                                              █   │
│   ████████████████████████████████████████████████████████████████   │
│                                                                        │
│   Memory check:    16384K OK                                          │
│   Router:          Connected  (Axum / localhost:3000)                 │
│   Database:        SQLite     (apex.db)                               │
│   Execution:       Python     (AgentLoop ready)                       │
│   VM pool:         Docker     (3 slots available)                     │
│   TOTP:            Configured                                         │
│   SOUL.md:         Loaded                                             │
│                                                                        │
│   [ OK — Enter ApexShell ]                                            │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

### 14.2 Boot Phases

Boot phases map to actual APEX startup checks — not cosmetic only. Each phase resolves a real API call.

```typescript
interface BootPhase {
  id:         string;
  label:      string;    // Displayed in "Loading: ..." line
  check:      () => Promise<BootPhaseResult>;
  durationMs: number;    // Minimum display time (even if check resolves faster)
}

const BOOT_SEQUENCE: BootPhase[] = [
  {
    id: 'router',
    label: 'apex.router.axum',
    durationMs: 600,
    check: () => fetch('/health').then(r => ({ ok: r.ok, detail: 'Router connected' })),
  },
  {
    id: 'database',
    label: 'apex.memory.sqlite',
    durationMs: 500,
    check: () => api.get('/api/v1/config/summary').then(r => ({
      ok: true, detail: `${r.database.type} — ${r.database.status}`,
    })),
  },
  {
    id: 'skills',
    label: 'apex.skills.registry',
    durationMs: 700,
    check: () => api.get('/api/v1/skills').then(r => ({
      ok: true, detail: `${r.length} skills registered`,
    })),
  },
  {
    id: 'execution',
    label: 'apex.execution.python',
    durationMs: 800,
    check: () => api.get('/api/v1/vm/stats').then(r => ({
      ok: true, detail: `VM pool: ${r.available} slots available`,
    })),
  },
  {
    id: 'totp',
    label: 'apex.auth.totp',
    durationMs: 400,
    check: () => api.get('/api/v1/totp/status').then(r => ({
      ok: r.configured, detail: r.configured ? 'TOTP configured' : 'TOTP not set up',
    })),
  },
  {
    id: 'soul',
    label: 'apex.identity.soul',
    durationMs: 600,
    check: () => api.get('/api/v1/config').then(r => ({
      ok: true, detail: `SOUL.md: ${r.soul_dir}`,
    })),
  },
  {
    id: 'ready',
    label: 'apex.shell.ready',
    durationMs: 800,
    check: async () => ({ ok: true, detail: 'Enter ApexShell' }),
  },
];
// Total typical duration: ~4.4 seconds cold boot
```

Warm restart (page reload with existing session): skips to ApexShell in < 500ms with a 2-second abbreviated flash showing only the APEX-OS logo.

### 14.3 Audio (Optional, Off by Default)

```typescript
const BOOT_AUDIO = {
  phase_tick: 'apex-os/boot-tick.mp3',      // Short beep per phase
  ready:      'apex-os/boot-ready.mp3',     // Completion chime
};
// All audio has visual equivalents. Audio is never the sole indicator of state.
// Controlled via ApexOsConfig.audio (boolean, default false)
```

---

## 15. ApexShell Desktop & Window Manager

### 15.1 Desktop Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  APEX-OS 1.0.0    APEX-OS │ Folders │ Tools │ Settings │ Help          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐          │
│  │[icon]│  │[icon]│  │[icon]│  │[icon]│  │[icon]│  │[icon]│          │
│  │      │  │      │  │      │  │      │  │      │  │      │          │
│  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘          │
│   Chat      Memory    Skills    Tasks     Journal    Tools             │
│                                                                         │
│  ┌──────┐  ┌──────┐  ┌──────┐                                          │
│  │[icon]│  │[icon]│  │[icon]│                                          │
│  │      │  │      │  │      │                                          │
│  └──────┘  └──────┘  └──────┘                                          │
│   Kanban   Channels   Audit                                             │
│                                                                         │
│  ════════════════════════════════════════════════════════════           │
│  ◉ Active  │  Tasks: 2  │  Unread: 3  │  Budget: $0.12  │  11:34      │
└─────────────────────────────────────────────────────────────────────────┘
```

Icons are 48×48px SVGs with the APEX-OS bevel treatment (highlight top-left, shadow bottom-right). Double-click opens the corresponding window. Right-click opens a context menu. Positions persist to SQLite via `user_theme_prefs` or a dedicated `apexos_icon_positions` table.

**Surfaces mapped from APEX v1.0.0 L6 components:**

| ApexShell Icon | Opens | L6 Component |
|---|---|---|
| Chat | Chat window | `Chat.tsx` + `TaskSidebar.tsx` |
| Memory | Memory Folder | `MemoryViewer.tsx` (3-tab) |
| Skills | Skills Folder | `Skills.tsx` |
| Tasks | Task window | `TaskSidebar.tsx` as standalone |
| Journal | Journal window | `DecisionJournal.tsx` |
| Kanban | Kanban window | `KanbanBoard.tsx` |
| Channels | Channels window | `ChannelManager.tsx` |
| Audit | Audit window | `AuditLog.tsx` |
| Tools | Tools Folder | Config, Metrics, TOTP setup |

**NOT mapped (NOT ACTIVE stubs — excluded until they ship):** Heartbeat, Soul, Governance, Moltbook.

### 15.2 ApexWM Window Chrome

```
┌─────────────────────────────────────────────────────────────────────┐  outer shadow
│┌───────────────────────────────────────────────────────────────────┐│  inner highlight
││ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ ││  title bar gradient
││ ▓ [≡] Chat with APEX                        ◉ active  [▣][□][×] ▓ ││  depth/zoom/close
││ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ ││
││                                                                   ││  content area —
││  React component renders here, unstyled by ApexWM                ││  all content from
││  Chat.tsx / TaskSidebar.tsx / Skills.tsx etc.                     ││  existing L6 comps
││                                                                   ││
││  ┌─────────────────────────────────────────────────────────┐     ││
││  │ Type your message...                            [Send]  │     ││  inset input
││  └─────────────────────────────────────────────────────────┘     ││
│└───────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
                                                      └─ resize handle
```

**[≡] Depth** — send to back (or bring to front if already at back).  
**[▣] Zoom** — toggle between full-screen and previous size.  
**[×] Close** — close window; content state preserved for re-open.  
**Title bar drag** — moves window; outline-only mode uses a ghost outline.  
**Resize handle** — bottom-right; live or outline depending on `dragMode`.  
**All controls respond immediately. No easing. No animation in default mode.**

### 15.3 APEX Feature Mapping

| APEX Feature | ApexShell Pattern | Notes |
|---|---|---|
| Chat messages | Chat window — scrollable list, inset input | `Chat.tsx` |
| ProcessGroup steps | Expandable list within Chat window | `ProcessGroup.tsx` |
| Task Sidebar | Pinnable panel or separate window | `TaskSidebar.tsx` |
| Memory (3 tabs) | Memory Folder → window | `MemoryViewer.tsx` |
| Skills | Skills Folder — icon per category | `Skills.tsx` |
| Decision Journal | Journal window — searchable list | `DecisionJournal.tsx` |
| Kanban | Kanban window | `KanbanBoard.tsx` |
| Channels | Channels window | `ChannelManager.tsx` |
| Audit Log | Audit window with CSV export | `AuditLog.tsx` |
| Settings / Config | Settings window — tabs | `Settings.tsx` + `ConfigViewer.tsx` |
| T1 Confirmation | Simple Dialog: [Confirm] [Cancel] | `ConfirmationGate.tsx` |
| T2 Confirmation | String Dialog: type action name | `ConfirmationGate.tsx` |
| T3 Confirmation | TOTP Dialog: type phrase + 6-digit code | `ConfirmationGate.tsx` |
| Agent active | DockIcon in status bar — glows with `--color-agent-active` | |
| Budget | Status bar — live via WebSocket | |

### 15.4 T3 TOTP Dialog

No countdown timer. Both fields must be complete before [Proceed] enables.

```
┌──────────────────────────────────────────────────────────────────────┐
│┌────────────────────────────────────────────────────────────────────┐│
││ ▓▓ ⚠  CONFIRMATION REQUIRED — T3                            ▓▓▓▓▓▓ ││
│└────────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  You are about to execute:                                           │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  shell.execute: rm -rf /important/data                         │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  This action is IRREVERSIBLE. Data cannot be recovered.              │
│                                                                      │
│  Type to confirm:                                                    │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  confirm delete /important/data           [________________]   │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
│  TOTP code (6 digits):  [______]                                     │
│                                                                      │
│  [Proceed — disabled until both fields complete]    [Cancel]         │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### 15.5 Menu System

Top-of-screen menu bar, anchored to the Workspace (not to individual windows). Click to open; drag-through selection (press on menu title, drag to item, release to activate) is an optional `animationMode: 'transitions'` feature.

```
┌─────────────────────────────────────────────────────────────────────┐
│  APEX-OS 1.0.0  │ APEX-OS │ Folders │ Tools │ Settings │ Help       │
│                 │ ─────── │ ─────── │ ───── │ ──────── │ ────       │
│                 │ About   │ Chat    │ Skills│ Themes   │ Guide      │
│                 │ ─────── │ Memory  │ Config│ Sounds   │ About      │
│                 │ Quit    │ Tasks   │       │ ─────────│ ───────    │
│                 │         │ Kanban  │       │ Prefs    │ Shortcuts  │
│                 │         │ ─────── │       └──────────┘ └──────    │
│                 │         │ Journal │                               │
│                 │         │ Audit   │                               │
│                 └─────────┘─────────┘                               │
```

### 15.6 Workspaces (Multiple Screens)

ApexShell supports multiple named Workspaces (equivalent to Amiga Screens), each with an independent window arrangement. Switching is instantaneous (CSS `display` swap).

Default Workspaces:
- **Main** — Chat, Tasks, Memory (default on boot)
- **Dev** — Skills, Kanban, Journal
- **System** — Settings, Config, Audit, Channels

### 15.7 Accessibility in APEX-OS Mode

| Concern | Mitigation |
|---|---|
| Pixel font readability | `fontMode: 'system'` uses system sans-serif; font scale ×1/×2/×3 |
| Window management complexity | `simpleMode: true` — single window, tabbed navigation; same surfaces, no desktop |
| No animations by default | `animationMode: 'transitions'` adds CSS transitions for users who need them |
| Audio alerts | Off by default; visual equivalents for every audio cue |
| Bevel chrome low contrast | APEX-OS theme validated against WCAG AA; High Contrast built-in always available |
| Screen reader support | ARIA roles on all controls; `role="dialog"` + `aria-modal` on all Dialogs; window focus trap on active window |
| Keyboard navigation | Tab through controls within active window; Alt+[letter] for menu items; Escape closes focused window |

### 15.8 Performance Targets

| Metric | Target | Implementation |
|---|---|---|
| ApexLoader to ApexShell | < 5s (real checks) | API calls in parallel where possible |
| Warm restart to ApexShell | < 500ms | Session flag; skip ApexLoader |
| Window open | < 50ms | DOM insert + CSS reflow |
| Icon drag | 60fps | CSS `transform: translate()` — no layout |
| Theme switch | < 100ms | CSS variable replace on `:root` |
| Menu open | < 16ms (one frame) | CSS `visibility` toggle |

---

## 16. Implementation Roadmap — APEX-OS

**Hard prerequisite**: Part I Phases 1–4 must be complete and stable. APEX-OS begins when the core theme system is proven in production.

### Phase 5 — Shell Foundation (Weeks 15–18)

- [ ] `ApexOsConfig` interface and persistence to `user_theme_prefs`
- [ ] Mode switch in Settings → Appearance → UI Mode
- [ ] Shift-load fallback to Standard mode
- [ ] ApexLoader component: phase loop, real API health checks, progress bar
- [ ] ApexShell desktop: icon grid with snap-to-grid
- [ ] Icon position persistence to SQLite
- [ ] Drag-and-drop (outline mode; full mode optional)
- [ ] Double-click detection (configurable timing)
- [ ] APEX-OS built-in theme fully implemented (CSS only)

### Phase 6 — Window Manager (Weeks 19–22)

- [ ] ApexWM: window chrome (bevel, title bar gradient via CSS)
- [ ] Controls: [≡] depth, [▣] zoom, [×] close — correct behaviours
- [ ] Title bar drag (move window)
- [ ] Resize handle (bottom-right)
- [ ] Z-order management
- [ ] Scroll bars (proportional thumb, arrows at ends)
- [ ] Window state persistence (position, size) to SQLite

### Phase 7 — System Components (Weeks 23–25)

- [ ] Top-of-screen menu bar with keyboard navigation
- [ ] Dialog system: Simple, String (T2), TOTP (T3)
- [ ] Status bar: agent state, task count, budget, clock
- [ ] DockIcon for agent state (heartbeat visual — when Heartbeat ships)
- [ ] Workspace (multi-screen) switching
- [ ] Optional audio system (off by default)

### Phase 8 — APEX Integration & Polish (Weeks 26–28)

- [ ] All APEX surfaces mapped to ApexShell windows (see Section 15.3)
- [ ] `simpleMode` — single window with tabs
- [ ] ARIA audit: roles, focus management, dialog traps
- [ ] Screen reader testing (NVDA, VoiceOver)
- [ ] Keyboard navigation complete
- [ ] Performance audit against targets (Section 15.8)
- [ ] User testing: both Amiga-familiar and Amiga-unfamiliar users

---

---

# Part III — Visual Identity

---

## 17. Image Generation & Icon Assets

This section provides ready-to-use prompts for generating APEX visual assets, plus guidance on sourcing and adapting AROS icons. All generated assets are APEX-original — no Amiga/Commodore trademarks are used in prompts or outputs.

---

### 17.1 APEX Logo

**Purpose**: Primary brand logo used in the ApexLoader screen, app favicon, README, and documentation header.

**Design intent**: Bold geometric wordmark. The "A" in APEX should feel like a data structure — triangular, precise, with a horizontal cut suggesting hierarchy or a process graph edge. Cyan on deep navy. Clean enough to read at 16px favicon size.

#### Prompt — Midjourney / DALL·E 3 / Ideogram / Recraft

```
APEX wordmark logo, geometric sans-serif typeface, the letter A styled as a 
triangular data-flow node with a single horizontal crossbar cut at 40% height, 
uppercase letters E-P-X following in the same weight, electric cyan (#00d4aa) 
on deep navy (#0f1117) background, flat vector style, no gradients, 
no shadows, no decorative elements, suitable for use as a software product 
wordmark at small sizes, high legibility, balanced letter spacing, 
--ar 3:1 --style raw --v 6
```

#### Prompt — for icon/favicon variant (square)

```
Single letter A as a software application icon, geometric triangular form 
with a horizontal cut through the upper third suggesting a data graph node, 
electric cyan on deep navy square background with 12% corner radius, 
flat vector, no gradients, clean edges, legible at 16x16 pixels, 
--ar 1:1 --style raw --v 6
```

#### Prompt — Stable Diffusion / ComfyUI

```
flat vector logo, single capital letter A, geometric triangle with horizontal 
crossbar cut, electric cyan color #00d4aa, deep navy background #0f1117, 
minimalist software product icon, no shadows, no textures, sharp edges, 
SVG-ready, centered composition, negative space used deliberately
```

**Post-processing**: Export as SVG. Ensure the logo works in monochrome (one-colour version for dark/light mode contexts). Minimum size: test at 16×16px favicon.

---

### 17.2 APEX Avatar

**Purpose**: The agent's visual identity in chat messages, the SOUL.md, the status bar DockIcon, and any agent persona context. This is what APEX "looks like" when it speaks.

**Design intent**: Geometric, non-human, unmistakably synthetic but not threatening. Think: a glowing terminal cursor that has developed opinions. Abstract, geometric face — or no face at all, just a presence. Cyan glow on dark ground. Should animate well (pulse for "active" state).

#### Prompt — Midjourney

```
Abstract AI agent avatar, geometric form, octagonal or hexagonal outline, 
interior filled with a subtle data-circuit pattern in electric cyan (#00d4aa) 
on deep space navy (#0f1117), single glowing cyan dot or triangle suggesting 
presence without a face, soft outer glow suggesting active processing, 
flat vector aesthetic with subtle depth, no human features, no robot clichés, 
no chrome ball, suitable as a 48x48 chat avatar icon, 
--ar 1:1 --style raw --v 6
```

#### Prompt — DALL·E 3

```
Design a software agent avatar icon: an abstract geometric shape (hexagon 
or octagon) with electric cyan circuitry lines on a deep navy background. 
A single glowing cyan point at center suggesting awareness. No face, no 
humanoid features. Flat vector style. The overall feeling should be 
"intelligent presence" not "robot". Clean, minimal, suitable for 48x48 icon.
```

#### Prompt — Stable Diffusion (SDXL)

```
abstract geometric AI avatar, hexagonal frame, interior circuit pattern, 
electric cyan glow, deep navy background, single illuminated node at center, 
flat 2D vector style, no face, no humanoid features, clean minimalist design, 
icon format, sharp edges with subtle glow effect
```

**States**: Generate two variants — **idle** (dim, lower opacity glow) and **active** (full brightness, animated pulse). The active state should loop smoothly as a CSS animation on the avatar in the chat UI.

---

### 17.3 ApexLoader Splash Screen

**Purpose**: The full-screen boot image shown during the ApexLoader sequence in APEX-OS mode. Displayed behind the boot text for ~4 seconds.

**Design intent**: Evokes the Amiga Kickstart boot screen energy — bold, dark, slightly monumental — but is unmistakably APEX, not Amiga. Dark background with the APEX logo prominent. A subtle geometric pattern or data-flow graph in the background. Feels like booting something real and capable.

#### Prompt — Midjourney

```
Full-screen boot splash screen for a software application called APEX-OS, 
dark deep navy background (#0f1117), large centered geometric APEX wordmark 
in electric cyan, subtle background pattern of hexagonal data nodes connected 
by thin cyan lines fading toward the edges, version number below the wordmark 
in small monospace text, lower third shows a horizontal progress bar 
partially filled in cyan, text "Loading: apex.autonomy.core" in monospace 
below the bar, overall aesthetic: serious, technical, slightly retro terminal, 
no gradients except the fading background pattern, high resolution 1920x1080, 
--ar 16:9 --style raw --v 6
```

#### Prompt — DALL·E 3

```
Create a dark-themed application boot screen image. Background: deep navy 
(#0f1117). Centered: large bold text "APEX-OS" in electric cyan (#00d4aa) 
using a geometric sans-serif font. Below the logo: a thin horizontal progress 
bar, 60% filled in cyan. Below the bar: small monospace text reading 
"Loading: apex.autonomy.core". Background has a subtle faint pattern of 
connected hexagonal nodes in very dark cyan, barely visible. 
Aspect ratio 16:9, resolution 1920x1080. No gradients in the logo. 
The mood is: serious, capable, technical.
```

#### Prompt — Stable Diffusion (SDXL + ControlNet text)

```
dark software boot screen, navy background #0f1117, centered large monospace 
text "APEX-OS" in bright cyan, horizontal progress bar below, loading text 
below that, faint hexagonal network pattern in background, 16:9 aspect ratio, 
technical aesthetic, no lens flare, no glow blooms, clean flat design, 
high resolution
```

**Notes**: The splash screen is a static image displayed behind a React component that renders the actual boot text and progress bar. The image provides ambiance; all functional text is rendered by the component and must be readable against the image.

---

### 17.4 APEX-OS Theme Screenshot / Marketing Image

**Purpose**: A composed screenshot showing the ApexShell desktop with a few windows open, for README, documentation, and release announcements.

#### Prompt — Midjourney

```
Desktop screenshot of a retro-futurist operating system called APEX-OS, 
dark navy desktop background with a subtle hexagonal grid pattern, 
several rectangular windows with beveled borders and gradient title bars 
in deep blue-to-navy, each window title in white monospace text, 
icons on the desktop representing applications in a grid, 
electric cyan accent color used for active window title bars and icons, 
a menu bar at the top of the screen, a status bar at the bottom showing 
"Agent: Active | Tasks: 2 | Budget: $0.12", retro computer aesthetic 
meets modern terminal color scheme, ultra detailed, 16:9, 
--ar 16:9 --style raw --v 6
```

---

### 17.5 AROS Icon Sets — Source & License

AROS (AROS Research Operating System) provides two icon sets under the **AROS Public License (APL v1.1)**, which permits use in other projects with attribution.

#### Icon Sets Available

**Gorilla** (default AROS icon set)
- Modern, colourful, 48×48px icons with smooth gradients
- Path in AROS repo: `images/IconSets/Gorilla/`
- GitHub: [github.com/aros-development-team/AROS](https://github.com/aros-development-team/AROS) → `images/IconSets/Gorilla/`

**Mason** (classic-style AROS icons)
- Flatter, more retro aesthetic, closer to original Workbench look
- Path in AROS repo: `images/IconSets/Mason/`

#### How to Obtain

The icon files live in the AROS source tree as `.info` files (Amiga icon format, not PNG/SVG directly). To extract usable PNGs:

1. **From AROS nightly build**: Download a pre-built AROS image from [aros.org/nightly1.html](http://www.aros.org/nightly1.html) → mount the disk image → browse `SYS:Prefs/Presets/` or `SYS:Wanderer/`
2. **From AROS community**: The **Retrofunk icon pack** at [amiga-look.org/artwork/retrofunk-aros-iconpack](https://www.amiga-look.org/artwork/retrofunk-aros-iconpack) provides a ready-to-use zip of AROS-style icons
3. **Direct GitHub**: Clone `github.com/aros-development-team/AROS` and use `workbench/devs/datatypes/` or the build toolchain to extract PNG from `.info` files

#### Attribution Requirement (APL)

If using AROS artwork, include in your project's credits:

```
APEX-OS icon artwork includes elements from the AROS Research Operating System
icon sets (Gorilla / Mason), used under the AROS Public License (APL) v1.1.
Original artwork copyright © AROS Development Team.
https://github.com/aros-development-team/AROS
https://aros.sourceforge.io/license.html
```

#### AROS Icon Mapping for APEX-OS

| APEX-OS Surface | Suggested AROS Icon Category | AROS path (Gorilla set) |
|---|---|---|
| Chat | `tools/` or `utilities/exchange` | `Gorilla/Icons/Medium/Tools/` |
| Memory | `drawers/` | `Gorilla/Icons/Medium/Drawers/` |
| Skills | `tools/` toolbox or wrench | `Gorilla/Icons/Medium/Tools/` |
| Tasks | `prefs/` or `tools/clock` | `Gorilla/Icons/Medium/Prefs/` |
| Journal | `docs/` text document | `Gorilla/Icons/Medium/Docs/` |
| Kanban | `tools/` grid or board | `Gorilla/Icons/Medium/Tools/` |
| Channels | `utilities/multiview` network | `Gorilla/Icons/Medium/Utilities/` |
| Audit | `tools/scout` or system | `Gorilla/Icons/Medium/Tools/` |
| Settings | `prefs/` preferences cog | `Gorilla/Icons/Medium/Prefs/` |
| Disk/RAM | `disks/` RAM disk | `Gorilla/Icons/Medium/Disks/` |
| Folder | `drawers/` standard drawer | `Gorilla/Icons/Medium/Drawers/` |

#### Custom APEX-OS Icons

For icons with no good AROS equivalent, generate custom SVGs matching the APEX-OS aesthetic:

#### Prompt — Chat icon

```
48x48 pixel icon, retro computer operating system style, speech bubble 
with a small circuit node inside suggesting AI chat, electric cyan on 
deep navy, 3D bevel effect with light top-left edge and dark shadow 
bottom-right, no text, no emoji style, flat pixel art aesthetic, 
suitable for 48x48 desktop icon
```

#### Prompt — Agent/Task icon (running process)

```
48x48 pixel desktop icon, retro OS style, abstract agent symbol — 
a small octagon with radiating thin lines suggesting activity, 
electric cyan on deep navy background, 3D bevel border, 
suggesting an active running process, no text, clean pixel edges
```

#### Prompt — Skills/Tools icon

```
48x48 pixel desktop icon, retro OS style, wrench and cog overlapping 
in the center, electric cyan on deep navy, 3D raised bevel border, 
pixel-art inspired but clean vector edges, suggesting a toolbox or 
skill executor, no text
```

---

### 17.6 Font Guidance

**For UI text (ApexShell)**: System font stack first. When `fontMode: 'pixel'` is enabled, use a free embedded pixel/bitmap font:

- **[Press Start 2P](https://fonts.google.com/specimen/Press+Start+2P)** — Google Fonts, OFL license. 8px grid, authentic retro terminal feel.
- **[Silkscreen](https://fonts.google.com/specimen/Silkscreen)** — Google Fonts, OFL license. Cleaner, more legible at small sizes.
- **[VT323](https://fonts.google.com/specimen/VT323)** — Google Fonts, OFL license. Monospace terminal aesthetic.

Do not embed the Topaz font (original Amiga bitmap font) — its license status for commercial/product use is unclear. Use Press Start 2P or Silkscreen as the APEX-OS system font instead.

**For code/terminal/task IDs**: [JetBrains Mono](https://www.jetbrains.com/legalforms/font/) (OFL) or [Fira Code](https://github.com/tonsky/FiraCode) (OFL) — both free, excellent monospace, correct ligatures for code display in ProcessGroup steps.

---

### 17.7 Asset File Structure

```
ui/
└── public/
    └── apex-os/
        ├── boot/
        │   ├── splash.webp          APEX-OS splash (generated, Section 17.3)
        │   ├── boot-tick.mp3        Boot phase audio (optional)
        │   └── boot-ready.mp3       Ready chime (optional)
        ├── icons/
        │   ├── chat.svg             APEX-OS Chat icon
        │   ├── memory.svg           Memory Folder icon
        │   ├── skills.svg           Skills Folder icon
        │   ├── tasks.svg            Tasks icon
        │   ├── journal.svg          Decision Journal icon
        │   ├── kanban.svg           Kanban icon
        │   ├── channels.svg         Channels icon
        │   ├── audit.svg            Audit Log icon
        │   ├── tools.svg            Tools Folder icon
        │   ├── settings.svg         Settings icon
        │   ├── folder.svg           Generic closed folder
        │   └── folder-open.svg      Generic open folder
        ├── logo/
        │   ├── apex-logo.svg        Wordmark (generated, Section 17.1)
        │   ├── apex-logo-mono.svg   Monochrome variant
        │   └── apex-favicon.svg     Square icon variant
        └── avatar/
            ├── apex-avatar-idle.svg    Agent idle state
            └── apex-avatar-active.svg  Agent active state (animated)

ui/
└── src/
    └── fonts/
        ├── PressStart2P.woff2       Boot screen, ApexShell titles
        └── JetBrainsMono.woff2      Code, task IDs, terminal output
```

---

*APEX Theme System Design Specification · v4.0 · Architecture ref: APEX v1.0.0*
