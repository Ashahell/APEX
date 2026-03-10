import { Theme } from './types';

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
      button: {
        bg: '#1a1a24',
        bgHover: '#2a2a34',
        bgActive: '#0a0a14',
        text: '#e8e8ec',
        border: '#3a3a44',
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
        mcp: '#06b6d4',
      },
    },
  },
};
