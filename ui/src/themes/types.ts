export interface Theme {
  id: string;
  name: string;
  description: string;
  isBuiltIn: boolean;
  tokens: ThemeTokens;
}

export interface ThemeTokens {
  colors: ColorTokens;
}

export interface ColorTokens {
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
  accent: {
    success: string;
    warning: string;
    error: string;
    info: string;
  };
  agent: {
    idle: string;
    active: string;
    thinking: string;
    alert: string;
  };
  badge: {
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
