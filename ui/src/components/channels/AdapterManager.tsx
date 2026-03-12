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
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)] flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
            <line x1="12" y1="2" x2="12" y2="6"></line>
            <line x1="12" y1="18" x2="12" y2="22"></line>
            <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
            <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
            <line x1="2" y1="12" x2="6" y2="12"></line>
            <line x1="18" y1="12" x2="22" y2="12"></line>
            <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
            <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
          </svg>
          Loading adapters...
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-[var(--color-border)] p-4 bg-[var(--color-panel)]">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92s2.92-1.31 2.92-2.92-1.31-2.92-2.92-2.92z"></path>
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold">Adapters</h2>
            <p className="text-sm text-[var(--color-text-muted)]">Configure messaging channel integrations</p>
          </div>
        </div>
      </div>

      {/* Adapters Grid */}
      <div className="flex-1 overflow-auto p-4">
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {adapters.map((adapter) => {
            const info = ADAPTER_INFO[adapter.name] || { label: adapter.name, description: '', icon: '🔌' };
            return (
              <div key={adapter.name} className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)] hover:border-[#4248f1]/30 transition-colors">
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center gap-3">
                    <span className="text-2xl">{info.icon}</span>
                    <div>
                      <h3 className="font-medium">{info.label}</h3>
                      <p className="text-xs text-[var(--color-text-muted)]">{adapter.adapter_type}</p>
                    </div>
                  </div>
                  <span className={`text-xs px-3 py-1 rounded-full font-medium ${
                    adapter.enabled 
                      ? 'bg-green-500/10 text-green-500 border border-green-500/20' 
                      : 'bg-[var(--color-muted)] text-[var(--color-text-muted)]'
                  }`}>
                    {adapter.enabled ? 'Active' : 'Disabled'}
                  </span>
                </div>
                
                <p className="text-sm text-[var(--color-text-muted)] mb-4">{info.description}</p>
                
                <div className="flex gap-2">
                  <button
                    onClick={() => toggleAdapter(adapter.name)}
                    className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
                      adapter.enabled 
                        ? 'bg-red-500/10 text-red-500 border border-red-500/20 hover:bg-red-500/20' 
                        : 'bg-green-500/10 text-green-500 border border-green-500/20 hover:bg-green-500/20'
                    }`}
                  >
                    {adapter.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    onClick={() => openConfig(adapter)}
                    className="px-3 py-1.5 text-sm border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors"
                  >
                    Configure
                  </button>
                </div>
              </div>
            );
          })}
        </div>

        {adapters.length === 0 && (
          <div className="text-center text-[var(--color-text-muted)] py-12">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--color-muted)] flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M18 16.08c-.76 0-1.44.3-1.96.77L8.91 12.7c.05-.23.09-.46.09-.7s-.04-.47-.09-.7l7.05-4.11c.54.5 1.25.81 2.04.81 1.66 0 3-1.34 3-3s-1.34-3-3-3-3 1.34-3 3c0 .24.04.47.09.7L8.04 9.81C7.5 9.31 6.79 9 6 9c-1.66 0-3 1.34-3 3s1.34 3 3 3c.79 0 1.5-.31 2.04-.81l7.12 4.16c-.05.21-.08.43-.08.65 0 1.61 1.31 2.92 2.92 2.92s2.92-1.31 2.92-2.92-1.31-2.92-2.92-2.92z"></path>
              </svg>
            </div>
            No adapters available
          </div>
        )}
      </div>

      {/* Config Modal */}
      {showConfig && selectedAdapter && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" onClick={() => setShowConfig(false)}>
          <div className="bg-[var(--color-panel)] border border-[var(--color-border)] rounded-xl p-6 max-w-md w-full mx-4 shadow-2xl" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
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
          <label className="block text-sm font-medium mb-1.5 text-[var(--color-text)]">
            {key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())}
          </label>
          <input
            type={key.includes('pass') || key.includes('token') ? 'password' : 'text'}
            value={value}
            onChange={(e) => setConfig({ ...config, [key]: e.target.value })}
            className="w-full px-3 py-2.5 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)]"
          />
        </div>
      ))}
      
      <div className="flex gap-2 justify-end pt-2">
        <button
          type="button"
          onClick={onCancel}
          className="px-4 py-2 border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors"
        >
          Cancel
        </button>
        <button
          type="submit"
          className="px-4 py-2 bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] transition-colors"
        >
          Save
        </button>
      </div>
    </form>
  );
}
