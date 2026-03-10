import { useState, useEffect, createContext, useContext, ReactNode } from 'react';
import { modern2026Theme, amigaTheme, agentzeroTheme, Theme, ColorTokens } from '../themes';

export type ThemeId = 'modern-2026' | 'amiga' | 'agentzero' | 'custom';

interface ThemeContextValue {
  themeId: ThemeId;
  theme: Theme;
  setTheme: (id: ThemeId) => void;
  toggleTheme: () => void;
  updateTheme: (updates: Partial<Theme>) => void;
  resetTheme: () => void;
  availableThemes: Theme[];
  isCustom: boolean;
  applyPreviewTheme: (colors: ColorTokens) => void;
}

const defaultThemes: Record<ThemeId, Theme> = {
  'modern-2026': modern2026Theme,
  'amiga': amigaTheme,
  'agentzero': agentzeroTheme,
  'custom': { ...modern2026Theme, id: 'custom', name: 'Custom', isBuiltIn: false },
};

const ThemeContext = createContext<ThemeContextValue | null>(null);

function getStoredThemes(): Record<string, Theme> {
  if (typeof window === 'undefined') return {};
  try {
    const stored = localStorage.getItem('apex-custom-themes');
    return stored ? JSON.parse(stored) : {};
  } catch {
    return {};
  }
}

function saveCustomTheme(theme: Theme) {
  const themes = getStoredThemes();
  themes[theme.id] = theme;
  localStorage.setItem('apex-custom-themes', JSON.stringify(themes));
}

function deleteCustomTheme(id: string) {
  const themes = getStoredThemes();
  delete themes[id];
  localStorage.setItem('apex-custom-themes', JSON.stringify(themes));
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [themeId, setThemeId] = useState<ThemeId>(() => {
    if (typeof window === 'undefined') return 'modern-2026';
    const saved = localStorage.getItem('apex-theme-id') as ThemeId;
    return saved && (defaultThemes[saved] || getStoredThemes()[saved]) ? saved : 'modern-2026';
  });

  const [customThemes, setCustomThemes] = useState<Record<string, Theme>>({});
  const [currentTheme, setCurrentTheme] = useState<Theme>(defaultThemes['modern-2026']);

  useEffect(() => {
    setCustomThemes(getStoredThemes());
  }, []);

  useEffect(() => {
    const allThemes = { ...defaultThemes, ...customThemes };
    const theme = allThemes[themeId];
    if (theme) {
      setCurrentTheme(theme);
      localStorage.setItem('apex-theme-id', themeId);
      applyThemeToDOM(theme);
    }
  }, [themeId, customThemes]);

  const setTheme = (id: ThemeId) => {
    setThemeId(id);
  };

  const updateTheme = (updates: Partial<Theme>) => {
    const mergedTokens = {
      ...currentTheme.tokens,
      ...updates.tokens,
      colors: {
        ...currentTheme.tokens.colors,
        ...(updates.tokens?.colors || {}),
        bg: { ...currentTheme.tokens.colors.bg, ...(updates.tokens?.colors?.bg || {}) },
        text: { ...currentTheme.tokens.colors.text, ...(updates.tokens?.colors?.text || {}) },
        primary: { ...currentTheme.tokens.colors.primary, ...(updates.tokens?.colors?.primary || {}) },
        button: { ...currentTheme.tokens.colors.button, ...(updates.tokens?.colors?.button || {}) },
        accent: { ...currentTheme.tokens.colors.accent, ...(updates.tokens?.colors?.accent || {}) },
        agent: { ...currentTheme.tokens.colors.agent, ...(updates.tokens?.colors?.agent || {}) },
        badge: { ...currentTheme.tokens.colors.badge, ...(updates.tokens?.colors?.badge || {}) },
      },
    };
    const newTheme = { ...currentTheme, tokens: mergedTokens, id: 'custom', name: 'Custom', isBuiltIn: false };
    setCurrentTheme(newTheme);
    saveCustomTheme(newTheme);
    setCustomThemes(prev => ({ ...prev, custom: newTheme }));
    setThemeId('custom');
  };

  const resetTheme = () => {
    deleteCustomTheme('custom');
    setCustomThemes(prev => {
      const next = { ...prev };
      delete next.custom;
      return next;
    });
    setThemeId('modern-2026');
  };

  const toggleTheme = () => {
    setThemeId(prev => prev === 'modern-2026' ? 'amiga' : 'modern-2026');
  };

  const applyPreviewTheme = (colors: ColorTokens) => {
    const previewTheme: Theme = {
      ...currentTheme,
      tokens: { colors },
    };
    applyThemeToDOM(previewTheme);
  };

  const allThemes = [...Object.values(defaultThemes), ...Object.values(customThemes).filter((t: Theme) => t.id !== 'custom')];

  return (
    <ThemeContext.Provider value={{
      themeId,
      theme: currentTheme,
      setTheme,
      toggleTheme,
      updateTheme,
      resetTheme,
      availableThemes: allThemes,
      isCustom: themeId === 'custom' || (!defaultThemes[themeId] && !!customThemes[themeId]),
      applyPreviewTheme,
    }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (!context) {
    return {
      themeId: 'modern-2026' as ThemeId,
      theme: defaultThemes['modern-2026'],
      setTheme: () => {},
      toggleTheme: () => {},
      updateTheme: () => {},
      resetTheme: () => {},
      availableThemes: Object.values(defaultThemes),
      isCustom: false,
      applyPreviewTheme: () => {},
    };
  }
  return context;
}

const defaultColors = {
  bg: { base: '#0a0a0f', elevated: '#12121a', overlay: '#1a1a24', surface: '#0e0e14' },
  text: { primary: '#e8e8ec', secondary: '#9090a0', muted: '#606070', inverse: '#0a0a0f' },
  primary: { DEFAULT: '#00d4aa', hover: '#00e6bb', active: '#00c29a', muted: 'rgba(0, 212, 170, 0.15)' },
  button: { bg: '#1a1a24', bgHover: '#2a2a34', bgActive: '#0a0a14', text: '#e8e8ec', border: '#3a3a44' },
  accent: { success: '#22c55e', warning: '#f59e0b', error: '#ef4444', info: '#3b82f6' },
  agent: { idle: '#606070', active: '#00d4aa', thinking: '#f59e0b', alert: '#ef4444' },
  badge: { gen: '#8b5cf6', use: '#00d4aa', exe: '#3b82f6', www: '#f59e0b', sub: '#ec4899', mem: '#22c55e', aud: '#ef4444', mcp: '#06b6d4' },
};

function getColor(obj: Record<string, unknown> | undefined, key: string, fallback: string): string {
  if (obj && typeof obj === 'object' && key in obj) {
    const val = obj[key];
    if (typeof val === 'string' && val.startsWith('#')) return val;
  }
  return fallback;
}

function applyThemeToDOM(theme: Theme) {
  const root = document.documentElement;
  const isAmiga = theme.id === 'amiga';
  const isAgentZero = theme.id === 'agentzero';
  
  root.classList.remove('dark');
  
  const colors = theme.tokens.colors || defaultColors;
  
  const bg = colors.bg || defaultColors.bg;
  const text = colors.text || defaultColors.text;
  const primary = colors.primary || defaultColors.primary;
  const button = colors.button || defaultColors.button;
  const accent = colors.accent || defaultColors.accent;
  
  if (isAmiga) {
    const css = `
      :root {
        --background: #b8b8b8 !important;
        --foreground: #000000 !important;
        --card: #b8b8b8 !important;
        --card-foreground: #000000 !important;
        --popover: #c8c8c8 !important;
        --popover-foreground: #000000 !important;
        --primary: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#0000ee')} !important;
        --primary-foreground: ${getColor(text as unknown as Record<string, unknown>, 'inverse', '#ffffff')} !important;
        --secondary: ${getColor(primary as unknown as Record<string, unknown>, 'muted', '#a0a0a0')} !important;
        --secondary-foreground: #000000 !important;
        --muted: #808080 !important;
        --muted-foreground: #000000 !important;
        --accent: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#0000ee')} !important;
        --accent-foreground: #ffffff !important;
        --destructive: ${getColor(accent as unknown as Record<string, unknown>, 'error', '#ff0000')} !important;
        --destructive-foreground: #ffffff !important;
        --border: #404040 !important;
        --input: #909090 !important;
        --ring: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#0000ee')} !important;
      }
      body, .flex.h-screen, #root, main, aside, header { background-color: #b8b8b8 !important; }
      .text-foreground, .text-white, .text-gray { color: #000000 !important; }
      .text-muted, .text-muted-foreground { color: #505050 !important; }
      button:not(.color-picker), input[type="button"], .btn { 
        background: linear-gradient(180deg, ${getColor(button as unknown as Record<string, unknown>, 'bgHover', '#d0d0d0')}, ${getColor(button as unknown as Record<string, unknown>, 'bg', '#c0c0c0')}) !important; 
        color: ${getColor(button as unknown as Record<string, unknown>, 'text', '#000000')} !important; 
        border: 2px outset ${getColor(button as unknown as Record<string, unknown>, 'bg', '#c0c0c0')} !important; 
      }
      .bg-primary { background: linear-gradient(180deg, ${getColor(primary as unknown as Record<string, unknown>, 'hover', '#0000ee')}, ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#0000ee')}) !important; color: #fff !important; }
    `;
    applyCSS(css);
  } else if (isAgentZero) {
    // AgentZero theme - dark navy with cyan accents
    const css = `
      :root {
        --background: ${getColor(bg as unknown as Record<string, unknown>, 'base', '#0f0f1a')} !important;
        --foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8f0')} !important;
        --card: ${getColor(bg as unknown as Record<string, unknown>, 'elevated', '#1a1a2e')} !important;
        --card-foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8f0')} !important;
        --popover: ${getColor(bg as unknown as Record<string, unknown>, 'overlay', '#252542')} !important;
        --popover-foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8f0')} !important;
        --primary: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#00d4ff')} !important;
        --primary-foreground: ${getColor(text as unknown as Record<string, unknown>, 'inverse', '#0f0f1a')} !important;
        --secondary: ${getColor(button as unknown as Record<string, unknown>, 'bg', '#252542')} !important;
        --secondary-foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8f0')} !important;
        --muted: ${getColor(bg as unknown as Record<string, unknown>, 'surface', '#16162a')} !important;
        --muted-foreground: ${getColor(text as unknown as Record<string, unknown>, 'muted', '#606078')} !important;
        --accent: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#00d4ff')} !important;
        --accent-foreground: ${getColor(text as unknown as Record<string, unknown>, 'inverse', '#0f0f1a')} !important;
        --destructive: ${getColor(accent as unknown as Record<string, unknown>, 'error', '#ef4444')} !important;
        --destructive-foreground: #ffffff !important;
        --border: #3a3a55 !important;
        --input: ${getColor(bg as unknown as Record<string, unknown>, 'surface', '#16162a')} !important;
        --ring: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#00d4ff')} !important;
      }
      body { background-color: ${getColor(bg as unknown as Record<string, unknown>, 'base', '#0f0f1a')} !important; color: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8f0')} !important; }
      button:not(.color-picker):not([type="color"]), input[type="button"], .btn { 
        background-color: ${getColor(button as unknown as Record<string, unknown>, 'bg', '#252542')} !important; 
        color: ${getColor(button as unknown as Record<string, unknown>, 'text', '#e8e8f0')} !important; 
        border: 1px solid ${getColor(button as unknown as Record<string, unknown>, 'border', '#3a3a55')} !important; 
      }
      button:not(.color-picker):not([type="color"]):hover, .btn:hover { background-color: ${getColor(button as unknown as Record<string, unknown>, 'bgHover', '#303055')} !important; }
      button:not(.color-picker):not([type="color"]):active, .btn:active { background-color: ${getColor(button as unknown as Record<string, unknown>, 'bgActive', '#1a1a2e')} !important; }
    `;
    applyCSS(css);
  } else {
    const css = `
      :root {
        --background: ${getColor(bg as unknown as Record<string, unknown>, 'base', '#0a0a0f')} !important;
        --foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8ec')} !important;
        --card: ${getColor(bg as unknown as Record<string, unknown>, 'elevated', '#12121a')} !important;
        --card-foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8ec')} !important;
        --popover: ${getColor(bg as unknown as Record<string, unknown>, 'overlay', '#1a1a24')} !important;
        --popover-foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8ec')} !important;
        --primary: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#00d4aa')} !important;
        --primary-foreground: ${getColor(text as unknown as Record<string, unknown>, 'inverse', '#000000')} !important;
        --secondary: ${getColor(button as unknown as Record<string, unknown>, 'bg', '#1a1a24')} !important;
        --secondary-foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8ec')} !important;
        --muted: ${getColor(bg as unknown as Record<string, unknown>, 'surface', '#0e0e14')} !important;
        --muted-foreground: ${getColor(text as unknown as Record<string, unknown>, 'muted', '#606070')} !important;
        --accent: ${getColor(bg as unknown as Record<string, unknown>, 'elevated', '#12121a')} !important;
        --accent-foreground: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8ec')} !important;
        --destructive: ${getColor(accent as unknown as Record<string, unknown>, 'error', '#ef4444')} !important;
        --destructive-foreground: #ffffff !important;
        --border: ${getColor(bg as unknown as Record<string, unknown>, 'surface', '#0e0e14')} !important;
        --input: ${getColor(bg as unknown as Record<string, unknown>, 'surface', '#0e0e14')} !important;
        --ring: ${getColor(primary as unknown as Record<string, unknown>, 'DEFAULT', '#00d4aa')} !important;
      }
      body { background-color: ${getColor(bg as unknown as Record<string, unknown>, 'base', '#0a0a0f')} !important; color: ${getColor(text as unknown as Record<string, unknown>, 'primary', '#e8e8ec')} !important; }
      button:not(.color-picker):not([type="color"]), input[type="button"], .btn { 
        background-color: ${getColor(button as unknown as Record<string, unknown>, 'bg', '#1a1a24')} !important; 
        color: ${getColor(button as unknown as Record<string, unknown>, 'text', '#e8e8ec')} !important; 
        border: 1px solid ${getColor(button as unknown as Record<string, unknown>, 'border', '#3a3a44')} !important; 
      }
      button:not(.color-picker):not([type="color"]):hover, .btn:hover { background-color: ${getColor(button as unknown as Record<string, unknown>, 'bgHover', '#2a2a34')} !important; }
      button:not(.color-picker):not([type="color"]):active, .btn:active { background-color: ${getColor(button as unknown as Record<string, unknown>, 'bgActive', '#0a0a14')} !important; }
    `;
    applyCSS(css);
  }
}

function applyCSS(css: string) {
  let styleEl = document.getElementById('apex-theme-styles');
  if (!styleEl) {
    styleEl = document.createElement('style');
    styleEl.id = 'apex-theme-styles';
    document.head.appendChild(styleEl);
  }
  styleEl.textContent = css;
}
