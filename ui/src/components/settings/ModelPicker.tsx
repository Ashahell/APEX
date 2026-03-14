import { useState, useEffect } from 'react';
import { 
  listProviderPlugins, 
  createProviderPlugin, 
  ProviderPlugin,
  listModelFallbacks,
  ModelFallback 
} from '../../lib/api';

export function ModelPicker() {
  const [providers, setProviders] = useState<ProviderPlugin[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [showAddProvider, setShowAddProvider] = useState(false);

  useEffect(() => {
    loadProviders();
  }, []);

  const loadProviders = async () => {
    try {
      const data = await listProviderPlugins();
      setProviders(data);
      if (data.length > 0) {
        setSelectedProvider(data[0].id);
      }
    } catch (err) {
      console.error('Failed to load providers:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center">
        <div className="w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-white">Model Provider</h3>
        <button
          onClick={() => setShowAddProvider(true)}
          className="text-xs text-indigo-400 hover:text-indigo-300"
        >
          + Add Provider
        </button>
      </div>

      <div className="space-y-2">
        {providers.map((provider) => (
          <button
            key={provider.id}
            onClick={() => setSelectedProvider(provider.id)}
            className={`w-full p-3 rounded-lg border text-left transition-colors ${
              selectedProvider === provider.id
                ? 'border-indigo-500 bg-indigo-500/10'
                : 'border-gray-700 bg-gray-800/50 hover:border-gray-600'
            }`}
          >
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-white">{provider.name}</span>
              <span className="text-xs text-gray-500 uppercase">{provider.provider_type}</span>
            </div>
            <div className="mt-1 text-xs text-gray-400">
              {provider.default_model || 'No default model'}
            </div>
          </button>
        ))}
      </div>

      {providers.length === 0 && (
        <div className="text-center py-4 text-gray-500 text-sm">
          No providers configured. Add a provider to get started.
        </div>
      )}

      {showAddProvider && (
        <AddProviderModal onClose={() => setShowAddProvider(false)} onAdd={loadProviders} />
      )}
    </div>
  );
}

interface AddProviderModalProps {
  onClose: () => void;
  onAdd: () => void;
}

function AddProviderModal({ onClose, onAdd }: AddProviderModalProps) {
  const [providerType, setProviderType] = useState('ollama');
  const [name, setName] = useState('');
  const [baseUrl, setBaseUrl] = useState('http://localhost:11434');
  const [defaultModel, setDefaultModel] = useState('');
  const [saving, setSaving] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    try {
      await createProviderPlugin({
        provider_type: providerType,
        name: name || providerType,
        base_url: baseUrl,
        default_model: defaultModel || undefined,
      });
      onAdd();
      onClose();
    } catch (err) {
      console.error('Failed to add provider:', err);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
      <div
        className="w-full max-w-md bg-[#1a1a2e] border border-gray-700 rounded-lg p-4"
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="text-lg font-medium text-white mb-4">Add Provider</h3>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Provider Type</label>
            <select
              value={providerType}
              onChange={(e) => setProviderType(e.target.value)}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-md text-white text-sm focus:outline-none focus:border-indigo-500"
            >
              <option value="ollama">Ollama</option>
              <option value="vllm">vLLM</option>
              <option value="sglang">SGLang</option>
              <option value="minimax">MiniMax</option>
              <option value="openrouter">OpenRouter</option>
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={providerType}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-md text-white text-sm focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Base URL</label>
            <input
              type="text"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-md text-white text-sm focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Default Model</label>
            <input
              type="text"
              value={defaultModel}
              onChange={(e) => setDefaultModel(e.target.value)}
              placeholder="e.g., qwen2.5:7b"
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-md text-white text-sm focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm text-gray-400 hover:text-white"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={saving}
              className="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-indigo-600/50 text-white rounded-md text-sm font-medium"
            >
              {saving ? 'Adding...' : 'Add Provider'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// Model Fallbacks Panel
export function ModelFallbacks() {
  const [fallbacks, setFallbacks] = useState<ModelFallback[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadFallbacks();
  }, []);

  const loadFallbacks = async () => {
    try {
      const data = await listModelFallbacks();
      setFallbacks(data);
    } catch (err) {
      console.error('Failed to load fallbacks:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center">
        <div className="w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-white">Model Fallbacks</h3>
        <span className="text-xs text-gray-500">
          {fallbacks.length} configured
        </span>
      </div>

      {fallbacks.length > 0 ? (
        <div className="space-y-2">
          {fallbacks.map((fallback) => (
            <div
              key={fallback.id}
              className="p-3 bg-gray-800/50 border border-gray-700 rounded-lg"
            >
              <div className="flex items-center gap-2 text-sm">
                <span className="text-white">{fallback.primary_model}</span>
                <span className="text-gray-500">→</span>
                <span className="text-indigo-400">{fallback.fallback_model}</span>
              </div>
              {fallback.provider && (
                <span className="text-xs text-gray-500 mt-1 block">
                  via {fallback.provider}
                </span>
              )}
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-4 text-gray-500 text-sm">
          No fallbacks configured. Add a fallback to handle model failures.
        </div>
      )}
    </div>
  );
}
