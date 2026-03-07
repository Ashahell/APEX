import { Theme } from './types';

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
