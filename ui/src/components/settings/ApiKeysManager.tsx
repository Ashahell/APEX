import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface ProviderInfo {
  id: string;
  name: string;
  default_url: string;
  default_model: string;
  requires_api_key: boolean;
  api_type: string;
}

interface ApiKeyProvider {
  value: string;
  label: string;
}

export function ApiKeysManager() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [apiKeys, setApiKeys] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const res = await apiGet('/api/v1/llms/providers');
      const data = await res.json() as ProviderInfo[];
      setProviders(data || []);
      
      // Load saved API keys from preferences
      const keysRes = await apiGet('/api/v1/settings/api_keys');
      if (keysRes.ok) {
        const keysData = await keysRes.json();
        if (keysData.value) {
          try {
            setApiKeys(JSON.parse(keysData.value));
          } catch {}
        }
      }
    } catch (err) {
      console.error('Failed to load providers:', err);
    } finally {
      setLoading(false);
    }
  };

  // Get unique providers that require API keys
  const apiKeyProviders: ApiKeyProvider[] = (() => {
    const seen = new Set<string>();
    const options: ApiKeyProvider[] = [];
    
    providers.forEach(p => {
      if (p.requires_api_key && !seen.has(p.id)) {
        seen.add(p.id);
        options.push({ value: p.id, label: p.name });
      }
    });
    
    return options.sort((a, b) => a.label.localeCompare(b.label));
  })();

  const handleSave = async () => {
    setSaving(true);
    try {
      // Save API keys to preferences (encrypted)
      await apiPost('/api/v1/settings/api_keys', {
        value: JSON.stringify(apiKeys),
        encrypt: true,
      });
    } catch (err) {
      console.error('Failed to save API keys:', err);
    } finally {
      setSaving(false);
    }
  };

  const handleKeyChange = (providerId: string, value: string) => {
    setApiKeys(prev => ({ ...prev, [providerId]: value }));
  };

  if (loading) {
    return <div className="p-4">Loading API keys...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">API Keys</h3>
        <p className="text-sm text-muted-foreground">
          API keys for model providers and services. Keys are stored securely.
        </p>
      </div>

      <div className="space-y-4 max-w-2xl">
        {apiKeyProviders.map(provider => (
          <div key={provider.value} className="field">
            <div className="field-label">
              <div className="field-title">{provider.label}</div>
            </div>
            <div className="field-control">
              <input
                type="password"
                value={apiKeys[provider.value] || ''}
                onChange={(e) => handleKeyChange(provider.value, e.target.value)}
                placeholder={provider.value === 'azure' ? 'Azure API Key' : 'sk-...'}
                className="w-full px-3 py-2 bg-background border rounded-md"
              />
            </div>
          </div>
        ))}

        {apiKeyProviders.length === 0 && (
          <p className="text-muted-foreground">No providers require API keys.</p>
        )}

        <div className="flex gap-3 pt-4">
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save API Keys'}
          </button>
        </div>
      </div>
    </div>
  );
}
