import { useState, useEffect, useCallback, useMemo } from 'react';
import { useTheme } from '../../hooks/useTheme';
import { Theme, ColorTokens } from '../../themes';

const defaultColors: ColorTokens = {
  bg: { base: '#0a0a0f', elevated: '#12121a', overlay: '#1a1a24', surface: '#0e0e14' },
  text: { primary: '#e8e8ec', secondary: '#9090a0', muted: '#606070', inverse: '#0a0a0f' },
  primary: { DEFAULT: '#00d4aa', hover: '#00e6bb', active: '#00c29a', muted: 'rgba(0, 212, 170, 0.15)' },
  button: { bg: '#1a1a24', bgHover: '#2a2a34', bgActive: '#0a0a14', text: '#e8e8ec', border: '#3a3a44' },
  accent: { success: '#22c55e', warning: '#f59e0b', error: '#ef4444', info: '#3b82f6' },
  agent: { idle: '#606070', active: '#00d4aa', thinking: '#f59e0b', alert: '#ef4444' },
  badge: { gen: '#8b5cf6', use: '#00d4aa', exe: '#3b82f6', www: '#f59e0b', sub: '#ec4899', mem: '#22c55e', aud: '#ef4444', mcp: '#06b6d4' },
};

function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj));
}

function isValidHex(v: string): boolean {
  return /^#[0-9A-Fa-f]{6}$/.test(v);
}

function getValidColor(value: string, fallback: string): string {
  return isValidHex(value) ? value : fallback;
}

interface ColorInputProps {
  label: string;
  description?: string;
  value: string;
  onChange: (value: string) => void;
}

function ColorInput({ label, description, value, onChange }: ColorInputProps) {
  const [textValue, setTextValue] = useState(value);
  const [isInvalid, setIsInvalid] = useState(false);
  
  useEffect(() => {
    setTextValue(value);
    setIsInvalid(false);
  }, [value]);

  const handleColorChange = useCallback((newColor: string) => {
    setTextValue(newColor);
    setIsInvalid(false);
    if (isValidHex(newColor)) {
      onChange(newColor);
    }
  }, [onChange]);

  const validateAndSubmit = useCallback(() => {
    if (isValidHex(textValue)) {
      onChange(textValue);
      setIsInvalid(false);
    } else {
      setTextValue(value);
      setIsInvalid(true);
    }
  }, [textValue, value, onChange]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      validateAndSubmit();
    } else if (e.key === 'Escape') {
      setTextValue(value);
      setIsInvalid(false);
    }
  }, [validateAndSubmit, value]);

  const safeColor = useMemo(() => getValidColor(value, '#000000'), [value]);

  return (
    <div className="flex items-center gap-2">
      <label htmlFor={`color-${label}`} className="sr-only">Color picker for {label}</label>
      <input
        id={`color-${label}`}
        type="color"
        value={safeColor}
        onChange={(e) => handleColorChange(e.target.value)}
        className="w-8 h-8 rounded cursor-pointer border-0 shrink-0"
        aria-label={`Select color for ${label}`}
      />
      <div className="flex-1 min-w-0">
        <input
          type="text"
          value={textValue}
          onChange={(e) => {
            setTextValue(e.target.value);
            setIsInvalid(false);
          }}
          onBlur={validateAndSubmit}
          onKeyDown={handleKeyDown}
          className={`w-full px-2 py-1 text-sm rounded border bg-background text-foreground font-mono ${
            isInvalid ? 'border-red-500 focus:border-red-500' : ''
          }`}
          aria-invalid={isInvalid}
          aria-describedby={description ? `desc-${label}` : undefined}
        />
        {description && (
          <div id={`desc-${label}`} className="text-xs text-muted-foreground truncate">
            {description}
          </div>
        )}
        {isInvalid && (
          <div className="text-xs text-red-500">Invalid hex color</div>
        )}
      </div>
      <span className="text-xs text-muted-foreground shrink-0 w-20">{label}</span>
    </div>
  );
}

interface SectionProps {
  title: string;
  children: React.ReactNode;
}

function Section({ title, children }: SectionProps) {
  return (
    <div className="border rounded-lg p-4 bg-card">
      <h3 className="font-semibold mb-3 text-foreground">{title}</h3>
      <div className="space-y-2">{children}</div>
    </div>
  );
}

export function ThemeEditor() {
  const { theme, updateTheme, resetTheme, isCustom, availableThemes, setTheme, themeId, applyPreviewTheme } = useTheme();
  const [activeTab, setActiveTab] = useState<'presets' | 'editor'>('presets');
  const [newThemeName, setNewThemeName] = useState('');
  const [previewColors, setPreviewColors] = useState<ColorTokens | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // Ensure we always have valid colors to work with
  const currentColors = useMemo(() => {
    return theme?.tokens?.colors ?? defaultColors;
  }, [theme]);

  const colors = previewColors ?? currentColors;

  // Apply preview colors to DOM when they change
  useEffect(() => {
    if (previewColors) {
      applyPreviewTheme(previewColors);
    }
  }, [previewColors, applyPreviewTheme]);

  const updatePreviewColor = useCallback((path: string, value: string) => {
    setPreviewColors(prev => {
      const base = prev ? deepClone(prev) : deepClone(currentColors);
      const keys = path.split('.');
      let obj: Record<string, unknown> = base as unknown as Record<string, unknown>;
      
      for (let i = 0; i < keys.length - 1; i++) {
        const key = keys[i];
        if (!(key in obj) || typeof obj[key] !== 'object' || obj[key] === null) {
          obj[key] = {};
        }
        obj = obj[key] as Record<string, unknown>;
      }
      
      obj[keys[keys.length - 1]] = value;
      return base;
    });
    setHasChanges(true);
  }, [currentColors]);

  const handleApply = useCallback(() => {
    if (previewColors) {
      updateTheme({ tokens: { colors: previewColors } });
      applyPreviewTheme(previewColors);
      setHasChanges(false);
    }
  }, [previewColors, updateTheme, applyPreviewTheme]);

  const handleCancel = useCallback(() => {
    setPreviewColors(null);
    setHasChanges(false);
    applyPreviewTheme(currentColors);
  }, [applyPreviewTheme, currentColors]);

  const handlePresetSelect = useCallback((presetId: string) => {
    setPreviewColors(null);
    setHasChanges(false);
    setTheme(presetId as 'modern-2026' | 'amiga');
  }, [setTheme]);

  const handleSaveAs = useCallback(() => {
    const colorsToSave = previewColors ?? currentColors;
    const trimmedName = newThemeName.trim();
    if (!trimmedName) return;

    const customTheme: Theme = {
      ...theme,
      id: `custom-${Date.now()}`,
      name: trimmedName,
      isBuiltIn: false,
      tokens: { colors: deepClone(colorsToSave) },
    };
    
    updateTheme(customTheme);
    setPreviewColors(null);
    setHasChanges(false);
    setNewThemeName('');
  }, [previewColors, currentColors, newThemeName, theme, updateTheme]);

  const builtInThemes = useMemo(() => 
    availableThemes?.filter((t: Theme) => t.isBuiltIn) ?? [], 
    [availableThemes]
  );

  const handleResetToDefault = useCallback(() => {
    setPreviewColors(null);
    setHasChanges(false);
    applyPreviewTheme(defaultColors);
  }, [applyPreviewTheme]);

  // Ensure theme is loaded before rendering editor
  if (!theme) {
    return <div className="p-4 text-muted-foreground">Loading theme editor...</div>;
  }

  return (
    <div className="p-4 space-y-4">
      <div className="flex gap-2 border-b pb-2 items-center flex-wrap">
        <button
          onClick={() => setActiveTab('presets')}
          className={`px-4 py-2 rounded-t transition-colors ${
            activeTab === 'presets' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
          }`}
        >
          Presets
        </button>
        <button
          onClick={() => setActiveTab('editor')}
          className={`px-4 py-2 rounded-t transition-colors ${
            activeTab === 'editor' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
          }`}
        >
          Editor
        </button>
        {hasChanges && (
          <div className="ml-auto flex gap-2">
            <button
              onClick={handleCancel}
              className="px-3 py-1 text-sm bg-muted rounded hover:bg-muted/80 transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleApply}
              className="px-3 py-1 text-sm bg-primary text-primary-foreground rounded hover:opacity-90 transition-opacity"
            >
              Apply
            </button>
          </div>
        )}
        {!hasChanges && activeTab === 'editor' && (
          <span className="ml-auto text-sm text-muted-foreground">
            Make changes and click Apply to preview
          </span>
        )}
      </div>

      {activeTab === 'presets' && (
        <div className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {builtInThemes.map((preset: Theme) => (
              <button
                key={preset.id}
                onClick={() => handlePresetSelect(preset.id)}
                className={`p-4 rounded-lg border-2 text-left transition-all hover:shadow-md ${
                  themeId === preset.id ? 'border-primary ring-1 ring-primary' : 'border-transparent hover:border-muted'
                }`}
              >
                <div
                  className="h-16 rounded mb-2 flex items-center justify-center text-xs font-medium"
                  style={{
                    background: preset.id === 'amiga' ? '#b8b8b8' : preset.tokens?.colors?.bg?.base ?? '#0a0a0f',
                    color: preset.id === 'amiga' ? '#000' : preset.tokens?.colors?.text?.primary ?? '#e8e8ec',
                    border: '2px solid #00007a'
                  }}
                >
                  {preset.name}
                </div>
                <div className="font-medium">{preset.name}</div>
                {preset.description && (
                  <div className="text-sm text-muted-foreground">{preset.description}</div>
                )}
              </button>
            ))}
          </div>

          {isCustom && (
            <div className="mt-4 p-4 bg-muted rounded-lg">
              <h3 className="font-semibold mb-2">Custom Theme</h3>
              <p className="text-sm text-muted-foreground mb-3">
                You have unsaved changes to a custom theme.
              </p>
              <div className="flex gap-2 flex-wrap">
                <input
                  type="text"
                  value={newThemeName}
                  onChange={(e) => setNewThemeName(e.target.value)}
                  placeholder="Theme name..."
                  className="flex-1 min-w-[200px] px-3 py-2 rounded border bg-background text-foreground"
                />
                <button
                  onClick={handleSaveAs}
                  disabled={!newThemeName.trim()}
                  className="px-4 py-2 bg-primary text-primary-foreground rounded disabled:opacity-50 disabled:cursor-not-allowed transition-opacity"
                >
                  Save As
                </button>
                <button
                  onClick={resetTheme}
                  className="px-4 py-2 bg-destructive text-destructive-foreground rounded hover:opacity-90 transition-opacity"
                >
                  Reset
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {activeTab === 'editor' && (
        <div className="space-y-4">
          <Section title="Background Colors">
            <ColorInput
              label="Base"
              description="Main page background"
              value={colors.bg.base}
              onChange={(v) => updatePreviewColor('bg.base', v)}
            />
            <ColorInput
              label="Elevated"
              description="Cards, panels, dialogs"
              value={colors.bg.elevated}
              onChange={(v) => updatePreviewColor('bg.elevated', v)}
            />
            <ColorInput
              label="Overlay"
              description="Dropdowns, popovers"
              value={colors.bg.overlay}
              onChange={(v) => updatePreviewColor('bg.overlay', v)}
            />
            <ColorInput
              label="Surface"
              description="Input backgrounds, dividers"
              value={colors.bg.surface || defaultColors.bg.surface}
              onChange={(v) => updatePreviewColor('bg.surface', v)}
            />
          </Section>

          <Section title="Text Colors">
            <ColorInput
              label="Primary"
              description="Main text color"
              value={colors.text?.primary ?? defaultColors.text.primary}
              onChange={(v) => updatePreviewColor('text.primary', v)}
            />
            <ColorInput
              label="Secondary"
              description="Secondary text"
              value={colors.text?.secondary ?? defaultColors.text.secondary}
              onChange={(v) => updatePreviewColor('text.secondary', v)}
            />
            <ColorInput
              label="Muted"
              description="Disabled, placeholder text"
              value={colors.text.muted}
              onChange={(v) => updatePreviewColor('text.muted', v)}
            />
            <ColorInput
              label="Inverse"
              description="Text on dark backgrounds"
              value={colors.text.inverse || defaultColors.text.inverse}
              onChange={(v) => updatePreviewColor('text.inverse', v)}
            />
          </Section>

          <Section title="Primary / Accent">
            <ColorInput
              label="Primary"
              description="Main accent color, links"
              value={colors.primary?.DEFAULT ?? defaultColors.primary.DEFAULT}
              onChange={(v) => updatePreviewColor('primary.DEFAULT', v)}
            />
            <ColorInput
              label="Primary Hover"
              description="Hover state for primary"
              value={colors.primary?.hover ?? defaultColors.primary.hover}
              onChange={(v) => updatePreviewColor('primary.hover', v)}
            />
            <ColorInput
              label="Primary Active"
              description="Active/pressed state"
              value={colors.primary?.active ?? defaultColors.primary.active}
              onChange={(v) => updatePreviewColor('primary.active', v)}
            />
          </Section>

          <Section title="Button Colors">
            <ColorInput
              label="Background"
              description="Default button background"
              value={colors.button?.bg ?? defaultColors.button.bg}
              onChange={(v) => updatePreviewColor('button.bg', v)}
            />
            <ColorInput
              label="Hover"
              description="Button hover state"
              value={colors.button?.bgHover ?? defaultColors.button.bgHover}
              onChange={(v) => updatePreviewColor('button.bgHover', v)}
            />
            <ColorInput
              label="Active"
              description="Button pressed state"
              value={colors.button?.bgActive ?? defaultColors.button.bgActive}
              onChange={(v) => updatePreviewColor('button.bgActive', v)}
            />
            <ColorInput
              label="Text"
              description="Button text color"
              value={colors.button?.text ?? defaultColors.button.text}
              onChange={(v) => updatePreviewColor('button.text', v)}
            />
            <ColorInput
              label="Border"
              description="Button border color"
              value={colors.button?.border ?? defaultColors.button.border}
              onChange={(v) => updatePreviewColor('button.border', v)}
            />
          </Section>

          <Section title="Status Colors">
            <ColorInput
              label="Success"
              description="Success messages, confirmations"
              value={colors.accent?.success ?? defaultColors.accent.success}
              onChange={(v) => updatePreviewColor('accent.success', v)}
            />
            <ColorInput
              label="Warning"
              description="Warning messages"
              value={colors.accent?.warning ?? defaultColors.accent.warning}
              onChange={(v) => updatePreviewColor('accent.warning', v)}
            />
            <ColorInput
              label="Error"
              description="Error messages, destructive"
              value={colors.accent?.error ?? defaultColors.accent.error}
              onChange={(v) => updatePreviewColor('accent.error', v)}
            />
            <ColorInput
              label="Info"
              description="Informational messages"
              value={colors.accent?.info ?? defaultColors.accent.info}
              onChange={(v) => updatePreviewColor('accent.info', v)}
            />
          </Section>

          <Section title="Agent States">
            <ColorInput
              label="Idle"
              description="Agent not running"
              value={colors.agent?.idle ?? defaultColors.agent.idle}
              onChange={(v) => updatePreviewColor('agent.idle', v)}
            />
            <ColorInput
              label="Active"
              description="Agent executing task"
              value={colors.agent?.active ?? defaultColors.agent.active}
              onChange={(v) => updatePreviewColor('agent.active', v)}
            />
            <ColorInput
              label="Thinking"
              description="Agent processing with LLM"
              value={colors.agent?.thinking ?? defaultColors.agent.thinking}
              onChange={(v) => updatePreviewColor('agent.thinking', v)}
            />
            <ColorInput
              label="Alert"
              description="Requires user attention"
              value={colors.agent?.alert ?? defaultColors.agent.alert}
              onChange={(v) => updatePreviewColor('agent.alert', v)}
            />
          </Section>

          <Section title="Badge Colors">
            <div className="grid grid-cols-2 gap-2">
              <ColorInput label="GEN" value={colors.badge?.gen ?? defaultColors.badge.gen} onChange={(v) => updatePreviewColor('badge.gen', v)} />
              <ColorInput label="USE" value={colors.badge?.use ?? defaultColors.badge.use} onChange={(v) => updatePreviewColor('badge.use', v)} />
              <ColorInput label="EXE" value={colors.badge?.exe ?? defaultColors.badge.exe} onChange={(v) => updatePreviewColor('badge.exe', v)} />
              <ColorInput label="WWW" value={colors.badge?.www ?? defaultColors.badge.www} onChange={(v) => updatePreviewColor('badge.www', v)} />
              <ColorInput label="SUB" value={colors.badge?.sub ?? defaultColors.badge.sub} onChange={(v) => updatePreviewColor('badge.sub', v)} />
              <ColorInput label="MEM" value={colors.badge?.mem ?? defaultColors.badge.mem} onChange={(v) => updatePreviewColor('badge.mem', v)} />
              <ColorInput label="AUD" value={colors.badge?.aud ?? defaultColors.badge.aud} onChange={(v) => updatePreviewColor('badge.aud', v)} />
              <ColorInput label="MCP" value={colors.badge?.mcp ?? defaultColors.badge.mcp} onChange={(v) => updatePreviewColor('badge.mcp', v)} />
            </div>
          </Section>

          <div className="flex gap-2 pt-4">
            <button
              onClick={handleResetToDefault}
              className="px-4 py-2 bg-destructive text-destructive-foreground rounded hover:opacity-90 transition-opacity"
            >
              Reset to Default
            </button>
          </div>
        </div>
      )}
    </div>
  );
}