import { useState, useEffect } from 'react';
import { apiGet, apiPut } from '../../lib/api';

interface ValidationResponse {
  allowed: boolean;
  blocked_imports: string[];
  validation_level: string;
  error?: string;
}

const VALIDATION_LEVELS = [
  { value: 'strict', label: 'Strict', description: 'Only safe stdlib modules (json, re, math, etc.)' },
  { value: 'moderate', label: 'Moderate', description: 'Includes network/parsing (urllib, csv, http.client)' },
  { value: 'permissive', label: 'Permissive', description: 'No restrictions - use with caution' },
];

export function ToolValidationSettings() {
  const [validationLevel, setValidationLevel] = useState<string>('strict');
  const [blockedImports, setBlockedImports] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  useEffect(() => {
    loadValidationLevel();
  }, []);

  const loadValidationLevel = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/tools/validation-level');
      if (res.ok) {
        const data: ValidationResponse = await res.json();
        setValidationLevel(data.validation_level);
        setBlockedImports(data.blocked_imports);
      }
    } catch (err) {
      console.error('Failed to load validation level:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleLevelChange = async (level: string) => {
    setSaving(true);
    setMessage(null);
    try {
      const res = await apiPut('/api/v1/tools/validation-level', {
        level,
      });
      if (res.ok) {
        const data: ValidationResponse = await res.json();
        setValidationLevel(data.validation_level);
        setBlockedImports(data.blocked_imports);
        setMessage({ type: 'success', text: `Validation level set to ${level}` });
      } else {
        setMessage({ type: 'error', text: 'Failed to update validation level' });
      }
    } catch (err) {
      setMessage({ type: 'error', text: 'Error updating validation level' });
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading tool validation settings...</div>
      </div>
    );
  }

  const currentLevel = VALIDATION_LEVELS.find(l => l.value === validationLevel);

  return (
    <>
      <div className="mb-6">
        <h2 className="text-2xl font-semibold">Tool Validation Settings</h2>
        <p className="text-[var(--color-text-muted)]">
          Configure import validation for dynamically generated tools
        </p>
      </div>

      {message && (
        <div className={`mb-4 p-3 rounded-lg ${
          message.type === 'success' 
            ? 'bg-green-500/10 text-green-500 border border-green-500/20' 
            : 'bg-red-500/10 text-red-500 border border-red-500/20'
        }`}>
          {message.text}
        </div>
      )}

      <div className="space-y-6">
        {/* Current Level */}
        <div className="p-4 rounded-lg border bg-card">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-medium">Current Validation Level</h3>
              <p className="text-sm text-[var(--color-text-muted)]">
                {currentLevel?.label || validationLevel}
              </p>
            </div>
            <span className="px-3 py-1 rounded-full text-sm font-medium bg-primary/10 text-primary">
              {validationLevel}
            </span>
          </div>
        </div>

        {/* Level Selection */}
        <div className="space-y-3">
          <h3 className="font-medium">Validation Levels</h3>
          {VALIDATION_LEVELS.map((level) => (
            <label
              key={level.value}
              className={`flex items-start p-4 rounded-lg border cursor-pointer transition-colors ${
                validationLevel === level.value
                  ? 'border-primary bg-primary/5'
                  : 'border-border hover:border-primary/50'
              }`}
            >
              <input
                type="radio"
                name="validationLevel"
                value={level.value}
                checked={validationLevel === level.value}
                onChange={() => handleLevelChange(level.value)}
                disabled={saving}
                className="mt-1 mr-3"
              />
              <div className="flex-1">
                <div className="font-medium">{level.label}</div>
                <div className="text-sm text-[var(--color-text-muted)]">{level.description}</div>
              </div>
              {saving && validationLevel !== level.value && (
                <span className="text-sm text-[var(--color-text-muted)]">Saving...</span>
              )}
            </label>
          ))}
        </div>

        {/* Allowed Imports */}
        <div className="p-4 rounded-lg border bg-card">
          <h3 className="font-medium mb-3">Allowed Imports ({validationLevel})</h3>
          <div className="flex flex-wrap gap-2">
            {blockedImports.length > 0 ? (
              blockedImports.map((imp, idx) => (
                <span
                  key={idx}
                  className="px-2 py-1 text-sm rounded bg-secondary text-secondary-foreground"
                >
                  {imp}
                </span>
              ))
            ) : (
              <span className="text-[var(--color-text-muted)]">No restrictions</span>
            )}
          </div>
        </div>

        {/* Info Box */}
        <div className="p-4 rounded-lg bg-blue-500/10 border border-blue-500/20">
          <h4 className="font-medium text-blue-500 mb-2">About Tool Validation</h4>
          <p className="text-sm text-[var(--color-text-muted)]">
            When dynamic tools are generated by the AI, they are validated against this import allowlist.
            Strict blocks dangerous modules like subprocess, os.system. Moderate adds network access.
            Permissive allows all imports but should only be used for trusted code.
          </p>
        </div>
      </div>
    </>
  );
}