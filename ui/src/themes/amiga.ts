import { Theme } from './types';

export const amigaTheme: Theme = {
  id: 'amiga',
  name: 'Amiga Workbench',
  description: 'Classic Amiga Workbench (1985) - Black desktop, gray windows',
  isBuiltIn: true,
  tokens: {
    colors: {
      bg: {
        base: '#000000',
        elevated: '#aaaaaa',
        overlay: '#cccccc',
        surface: '#888888',
      },
      text: {
        primary: '#000000',
        secondary: '#444444',
        muted: '#666666',
        inverse: '#ffffff',
      },
      primary: {
        DEFAULT: '#00007a',
        hover: '#0000ee',
        active: '#00006a',
        muted: 'rgba(0, 0, 122, 0.3)',
      },
      button: {
        bg: '#c0c0c0',
        bgHover: '#d0d0d0',
        bgActive: '#a0a0a0',
        text: '#000000',
        border: '#808080',
      },
      accent: {
        success: '#00aa00',
        warning: '#aa6600',
        error: '#cc0000',
        info: '#0066cc',
      },
      agent: {
        idle: '#888888',
        active: '#00007a',
        thinking: '#aa6600',
        alert: '#cc0000',
      },
      badge: {
        gen: '#6600cc',
        use: '#00007a',
        exe: '#0066cc',
        www: '#aa6600',
        sub: '#cc0066',
        mem: '#00aa00',
        aud: '#cc0000',
        mcp: '#008080',
      },
      chrome: {
        titleBarActive: 'linear-gradient(180deg, #0000ee 0%, #00007a 50%, #000055 100%)',
        titleBarInactive: 'linear-gradient(180deg, #888888 0%, #666666 50%, #444444 100%)',
        buttonRaised: 'linear-gradient(180deg, #cccccc 0%, #aaaaaa 50%, #888888 100%)',
        buttonDepressed: 'inset 2px 2px 4px rgba(0,0,0,0.4)',
        windowBorder: '#00007a',
      },
    },
  },
};
