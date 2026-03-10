import { Theme } from './types';

export const agentzeroTheme: Theme = {
  id: 'agentzero',
  name: 'AgentZero',
  description: 'Dark theme inspired by AgentZero AI framework - navy blue with cyan accents',
  isBuiltIn: true,
  tokens: {
    colors: {
      bg: {
        base: '#0f0f1a',
        elevated: '#1a1a2e',
        overlay: '#252542',
        surface: '#16162a',
      },
      text: {
        primary: '#e8e8f0',
        secondary: '#9090a8',
        muted: '#606078',
        inverse: '#0f0f1a',
      },
      primary: {
        DEFAULT: '#00d4ff',
        hover: '#00e6ff',
        active: '#00b4d8',
        muted: 'rgba(0, 212, 255, 0.15)',
      },
      button: {
        bg: '#252542',
        bgHover: '#303055',
        bgActive: '#1a1a2e',
        text: '#e8e8f0',
        border: '#3a3a55',
      },
      accent: {
        success: '#22c55e',
        warning: '#f59e0b',
        error: '#ef4444',
        info: '#3b82f6',
      },
      agent: {
        idle: '#606078',
        active: '#00d4ff',
        thinking: '#a855f7',
        alert: '#ef4444',
      },
      badge: {
        gen: '#8b5cf6',
        use: '#00d4ff',
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
