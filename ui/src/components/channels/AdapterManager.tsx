import { useState, useEffect } from 'react';
import { apiGet, apiPut, apiPost } from '../../lib/api';

interface AdapterConfig {
  name: string;
  adapter_type: string;
  enabled: boolean;
  config: Record<string, unknown>;
}

const ADAPTER_INFO: Record<string, { label: string; description: string; icon: string }> = {
  slack: { label: 'Slack', description: 'Connect to Slack workspaces', icon: '💬' },
  telegram: { label: 'Telegram', description: 'Telegram bot integration', icon: '✈️' },
  discord: { label: 'Discord', description: 'Discord bot integration', icon: '🎮' },
  email: { label: 'Email', description: 'Receive tasks via email (SMTP)', icon: '📧' },
  whatsapp: { label: 'WhatsApp', description: 'WhatsApp Business API', icon: '📱' },
};

export function AdapterManager() {
  const [adapters, setAdapters] = useState<AdapterConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedAdapter, setSelectedAdapter] = useState<AdapterConfig | null>(null);
  const [showConfig, setShowConfig] = useState(false);

  useEffect(() => {
    loadAdapters();
  }, []);

  async function loadAdapters() {
    try {
      setLoading(true);
      const response = await apiGet('/api/v1/adapters');
      if (response.ok) {
        const data = await response.json();
        setAdapters(data);
      }
    } catch (error) {
      console.error('Failed to load adapters:', error);
    } finally {
      setLoading(false);
    }
  }

  async function toggleAdapter(name: string) {
    try {
      const response = await apiPost(`/api/v1/adapters/${name}/toggle`, {});
      if (response.ok) {
        loadAdapters();
      }
    } catch (error) {
      console.error('Failed to toggle adapter:', error);
    }
  }

  async function updateAdapterConfig(name: string, config: Record<string, unknown>) {
    try {
      const response = await apiPut(`/api/v1/adapters/${name}`, { config });
      if (response.ok) {
        loadAdapters();
        setShowConfig(false);
        setSelectedAdapter(null);
      }
    } catch (error) {
      console.error('Failed to update adapter:', error);
    }
  }

  function openConfig(adapter: AdapterConfig) {
    setSelectedAdapter(adapter);
    setShowConfig(true);
  }

  if (loading) {
    return <div className="p-4">Loading adapters...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4">
        <h2 className="text-2xl font-semibold mb-2">Adapters</h2>
        <p className="text-muted-foreground">Configure messaging channel integrations</p>
      </div>

      <div className="flex-1 overflow-auto p-4">
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {adapters.map((adapter) => {
            const info = ADAPTER_INFO[adapter.name] || { label: adapter.name, description: '', icon: '🔌' };
            return (
              <div key={adapter.name} className="border rounded-lg p-4">
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <span className="text-2xl">{info.icon}</span>
                    <div>
                      <h3 className="font-medium">{info.label}</h3>
                      <p className="text-xs text-muted-foreground">{adapter.adapter_type}</p>
                    </div>
                  </div>
                  <span className={`text-xs px-2 py-1 rounded ${
                    adapter.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                  }`}>
                    {adapter.enabled ? 'Active' : 'Disabled'}
                  </span>
                </div>
                
                <p className="text-sm text-muted-foreground mb-4">{info.description}</p>
                
                <div className="flex gap-2">
                  <button
                    onClick={() => toggleAdapter(adapter.name)}
                    className={`px-3 py-1.5 text-sm rounded ${
                      adapter.enabled 
                        ? 'bg-red-100 text-red-700 hover:bg-red-200' 
                        : 'bg-green-100 text-green-700 hover:bg-green-200'
                    }`}
                  >
                    {adapter.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    onClick={() => openConfig(adapter)}
                    className="px-3 py-1.5 text-sm border rounded hover:bg-muted"
                  >
                    Configure
                  </button>
                </div>
              </div>
            );
          })}
        </div>

        {adapters.length === 0 && (
          <div className="text-center text-muted-foreground py-12">
            No adapters available
          </div>
        )}
      </div>

      {showConfig && selectedAdapter && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background border rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold mb-4">
              Configure {ADAPTER_INFO[selectedAdapter.name]?.label || selectedAdapter.name}
            </h3>
            
            <AdapterConfigForm
              adapter={selectedAdapter}
              onSave={(config) => updateAdapterConfig(selectedAdapter.name, config)}
              onCancel={() => {
                setShowConfig(false);
                setSelectedAdapter(null);
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
}

function AdapterConfigForm({ 
  adapter, 
  onSave, 
  onCancel 
}: { 
  adapter: AdapterConfig; 
  onSave: (config: Record<string, unknown>) => void;
  onCancel: () => void;
}) {
  const [config, setConfig] = useState<Record<string, string>>(() => {
    const result: Record<string, string> = {};
    Object.entries(adapter.config).forEach(([key, value]) => {
      result[key] = String(value);
    });
    return result;
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave(config);
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {Object.entries(config).map(([key, value]) => (
        <div key={key}>
          <label className="block text-sm font-medium mb-1">
            {key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
          </label>
          <input
            type={key.includes('pass') || key.includes('token') ? 'password' : 'text'}
            value={value}
            onChange={(e) => setConfig({ ...config, [key]: e.target.value })}
            className="w-full px-3 py-2 border rounded-lg bg-background"
          />
        </div>
      ))}
      
      <div className="flex gap-2 justify-end">
        <button
          type="button"
          onClick={onCancel}
          className="px-4 py-2 border rounded-lg hover:bg-muted"
        >
          Cancel
        </button>
        <button
          type="submit"
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
        >
          Save
        </button>
      </div>
    </form>
  );
}
