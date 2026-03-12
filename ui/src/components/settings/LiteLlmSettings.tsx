import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

export function LiteLlmSettings() {
  const [globalParams, setGlobalParams] = useState('');
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const res = await apiGet('/api/v1/settings/litellm_global_params');
      if (res.ok) {
        const data = await res.json();
        if (data.value) {
          setGlobalParams(data.value);
        }
      }
    } catch (err) {
      console.error('Failed to load LiteLLM params:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await apiPost('/api/v1/settings/litellm_global_params', {
        value: globalParams,
        encrypt: false,
      });
    } catch (err) {
      console.error('Failed to save LiteLLM params:', err);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="p-4">Loading LiteLLM settings...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">LiteLLM Global Settings</h3>
        <p className="text-sm text-muted-foreground">
          Configure global parameters passed to LiteLLM for all providers.
        </p>
      </div>

      <div className="space-y-4 max-w-2xl">
        <div className="field field-full">
          <div className="field-label">
            <div className="field-title">LiteLLM global parameters</div>
            <div className="field-description">
              Global LiteLLM params (e.g., timeout, stream_timeout) in .env format: one KEY=VALUE per line.
              Example: <code className="bg-muted px-1 rounded">stream_timeout=30</code>
            </div>
          </div>
          <div className="field-control">
            <textarea
              value={globalParams}
              onChange={(e) => setGlobalParams(e.target.value)}
              placeholder="timeout=60&#10;stream_timeout=30&#10;max_parallel_requests=100"
              className="w-full px-3 py-2 bg-background border rounded-md font-mono text-sm"
              rows={8}
            />
          </div>
        </div>

        <div className="flex gap-3 pt-4">
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>
    </div>
  );
}
