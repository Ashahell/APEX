import { Theme } from './types';

export const highContrastTheme: Theme = {
  id: 'high-contrast',
  name: 'High Contrast',
  description: 'WCAG AAA compliant high contrast theme for accessibility',
  isBuiltIn: true,
  tokens: {
    colors: {
      bg: {
        base: '#000000',
        elevated: '#1a1a1a',
        overlay: '#2d2d2d',
        surface: '#0d0d0d',
      },
      text: {
        primary: '#ffffff',
        secondary: '#d0d0d0',
        muted: '#a0a0a0',
        inverse: '#000000',
      },
      primary: {
        DEFAULT: '#00ffff',
        hover: '#33ffff',
        active: '#00cccc',
        muted: 'rgba(0, 255, 255, 0.2)',
      },
      button: {
        bg: '#1a1a1a',
        bgHover: '#2d2d2d',
        bgActive: '#000000',
        text: '#ffffff',
        border: '#ffffff',
      },
      accent: {
        success: '#00ff00',
        warning: '#ffff00',
        error: '#ff0000',
        info: '#00bfff',
      },
      agent: {
        idle: '#a0a0a0',
        active: '#00ffff',
        thinking: '#ffff00',
        alert: '#ff0000',
      },
      badge: {
        gen: '#ff00ff',
        use: '#00ffff',
        exe: '#00bfff',
        www: '#ffff00',
        sub: '#ff69b4',
        mem: '#00ff00',
        aud: '#ff0000',
        mcp: '#00ffff',
      },
      chrome: {
        titleBarActive: '#000000',
        titleBarInactive: '#1a1a1a',
        buttonRaised: '#2d2d2d',
        buttonDepressed: '#0d0d0d',
        windowBorder: '#ffffff',
      },
    },
  },
};
