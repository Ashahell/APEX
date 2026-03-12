import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface EmbedModelConfig {
  provider: string;
  model_name: string;
  api_key: string;
  api_base: string;
  ctx_length: number;
  kwargs: string;
}

const DEFAULT_CONFIG: EmbedModelConfig = {
  provider: 'local',
  model_name: 'nomic-embed-text',
  api_key: '',
  api_base: 'http://localhost:8081',
  ctx_length: 8192,
  kwargs: '',
};

export function EmbedModelSettings() {
  const [config, setConfig] = useState<EmbedModelConfig>(DEFAULT_CONFIG);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      // Load embed model config from preferences API
      const res = await apiGet('/api/v1/settings/embed_model');
      if (res.ok) {
        const data = await res.json();
        if (data.value) {
          try {
            setConfig({ ...DEFAULT_CONFIG, ...JSON.parse(data.value) });
          } catch {}
        }
      }
    } catch (err) {
      console.error('Failed to load embed model config:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await apiPost('/api/v1/settings/embed_model', {
        value: JSON.stringify(config),
        encrypt: false,
      });
    } catch (err) {
      console.error('Failed to save embed model config:', err);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="p-4">Loading embedding model settings...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">Embedding Model</h3>
        <p className="text-sm text-muted-foreground">
          Selection and settings for embedding model used by memory search
        </p>
      </div>

      <div className="space-y-4 max-w-2xl">
        {/* Model Name */}
        <div className="field">
          <div className="field-label">
            <div className="field-title">Embedding model name</div>
            <div className="field-description">Exact name of embedding model</div>
          </div>
          <div className="field-control">
            <input
              type="text"
              value={config.model_name}
              onChange={(e) => setConfig({ ...config, model_name: e.target.value })}
              placeholder="e.g., nomic-embed-text, text-embedding-3-small"
              className="w-full px-3 py-2 bg-background border rounded-md"
            />
          </div>
        </div>

        {/* API Base URL */}
        <div className="field">
          <div className="field-label">
            <div className="field-title">Embedding model API base URL</div>
            <div className="field-description">API base URL for embedding model</div>
          </div>
          <div className="field-control">
            <input
              type="text"
              value={config.api_base}
              onChange={(e) => setConfig({ ...config, api_base: e.target.value })}
              placeholder="https://api.example.com/v1"
              className="w-full px-3 py-2 bg-background border rounded-md"
            />
          </div>
        </div>

        {/* API Key */}
        <div className="field">
          <div className="field-label">
            <div className="field-title">API key (optional)</div>
            <div className="field-description">API key for the embedding provider</div>
          </div>
          <div className="field-control">
            <input
              type="password"
              value={config.api_key}
              onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
              placeholder="sk-..."
              className="w-full px-3 py-2 bg-background border rounded-md"
            />
          </div>
        </div>

        {/* Context Length */}
        <div className="field">
          <div className="field-label">
            <div className="field-title">Embedding model context length</div>
            <div className="field-description">Maximum tokens for embedding input</div>
          </div>
          <div className="field-control">
            <input
              type="number"
              value={config.ctx_length}
              onChange={(e) => setConfig({ ...config, ctx_length: parseInt(e.target.value) || 8192 })}
              min={512}
              className="w-full px-3 py-2 bg-background border rounded-md"
            />
          </div>
        </div>

        {/* Additional Parameters */}
        <div className="field field-full">
          <div className="field-label">
            <div className="field-title">Embedding model additional parameters</div>
            <div className="field-description">
              Any other parameters in KEY=VALUE format (one per line)
            </div>
          </div>
          <div className="field-control">
            <textarea
              value={config.kwargs}
              onChange={(e) => setConfig({ ...config, kwargs: e.target.value })}
              placeholder="dimension=768&#10;normalize=true"
              className="w-full px-3 py-2 bg-background border rounded-md font-mono text-sm"
              rows={3}
            />
          </div>
        </div>

        {/* Actions */}
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
