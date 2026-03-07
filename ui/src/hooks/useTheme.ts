import { useState, useEffect } from 'react';
import { modern2026Theme, amigaTheme, Theme } from '../themes';

export type ThemeId = 'modern-2026' | 'amiga';

const themes: Record<ThemeId, Theme> = {
  'modern-2026': modern2026Theme,
  'amiga': amigaTheme,
};

function applyThemeTokens(theme: Theme) {
  const root = document.documentElement;
  const { colors } = theme.tokens;
  
  // Background colors
  root.style.setProperty('--color-bg-base', colors.bg.base);
  root.style.setProperty('--color-bg-elevated', colors.bg.elevated);
  root.style.setProperty('--color-bg-overlay', colors.bg.overlay);
  if (colors.bg.surface) {
    root.style.setProperty('--color-bg-surface', colors.bg.surface);
  }
  
  // Text colors
  root.style.setProperty('--color-text-primary', colors.text.primary);
  root.style.setProperty('--color-text-secondary', colors.text.secondary);
  root.style.setProperty('--color-text-muted', colors.text.muted);
  if (colors.text.inverse) {
    root.style.setProperty('--color-text-inverse', colors.text.inverse);
  }
  
  // Primary colors
  root.style.setProperty('--color-primary', colors.primary.DEFAULT);
  root.style.setProperty('--color-primary-hover', colors.primary.hover);
  root.style.setProperty('--color-primary-active', colors.primary.active);
  if (colors.primary.muted) {
    root.style.setProperty('--color-primary-muted', colors.primary.muted);
  }
  
  // Accent colors
  root.style.setProperty('--color-accent-success', colors.accent.success);
  root.style.setProperty('--color-accent-warning', colors.accent.warning);
  root.style.setProperty('--color-accent-error', colors.accent.error);
  root.style.setProperty('--color-accent-info', colors.accent.info);
  
  // Agent state colors
  root.style.setProperty('--color-agent-idle', colors.agent.idle);
  root.style.setProperty('--color-agent-active', colors.agent.active);
  root.style.setProperty('--color-agent-thinking', colors.agent.thinking);
  root.style.setProperty('--color-agent-alert', colors.agent.alert);
  
  // Badge colors
  root.style.setProperty('--color-badge-gen', colors.badge.gen);
  root.style.setProperty('--color-badge-use', colors.badge.use);
  root.style.setProperty('--color-badge-exe', colors.badge.exe);
  root.style.setProperty('--color-badge-www', colors.badge.www);
  root.style.setProperty('--color-badge-sub', colors.badge.sub);
  root.style.setProperty('--color-badge-mem', colors.badge.mem);
  root.style.setProperty('--color-badge-aud', colors.badge.aud);
  
  // Amiga-specific chrome tokens
  if (colors.chrome) {
    if (colors.chrome.titleBarActive) {
      root.style.setProperty('--color-chrome-titlebar-active', colors.chrome.titleBarActive);
    }
    if (colors.chrome.titleBarInactive) {
      root.style.setProperty('--color-chrome-titlebar-inactive', colors.chrome.titleBarInactive);
    }
    if (colors.chrome.buttonRaised) {
      root.style.setProperty('--color-chrome-button-raised', colors.chrome.buttonRaised);
    }
    if (colors.chrome.buttonDepressed) {
      root.style.setProperty('--color-chrome-button-depressed', colors.chrome.buttonDepressed);
    }
    if (colors.chrome.windowBorder) {
      root.style.setProperty('--color-chrome-window-border', colors.chrome.windowBorder);
    }
  }
}

export function useTheme() {
  const [themeId, setThemeId] = useState<ThemeId>(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('apex-theme-id') as ThemeId;
      return saved && themes[saved] ? saved : 'modern-2026';
    }
    return 'modern-2026';
  });

  const theme = themes[themeId];

  useEffect(() => {
    applyThemeTokens(theme);
    localStorage.setItem('apex-theme-id', themeId);
  }, [theme, themeId]);

  const setTheme = (id: ThemeId) => {
    setThemeId(id);
  };

  const toggleTheme = () => {
    setThemeId(prev => prev === 'modern-2026' ? 'amiga' : 'modern-2026');
  };

  return { 
    theme, 
    themeId, 
    setTheme, 
    toggleTheme,
    availableThemes: Object.values(themes) 
  };
}
