import { useState, useEffect } from 'react';
import { apiGet, apiPost, apiPut, apiDelete } from '../../lib/api';

interface ProviderInfo {
  id: string;
  name: string;
  default_url: string;
  default_model: string;
  requires_api_key: boolean;
  api_type: string;
}

// Fallback providers in case API fails
const FALLBACK_PROVIDERS: ProviderInfo[] = [
  { id: "local", name: "Local (llama.cpp)", default_url: "http://localhost:8080/v1", default_model: "qwen3-4b", requires_api_key: false, api_type: "openai" },
  { id: "ollama", name: "Ollama", default_url: "http://localhost:11434/v1", default_model: "llama3.1", requires_api_key: false, api_type: "openai" },
  { id: "openai", name: "OpenAI", default_url: "https://api.openai.com/v1", default_model: "gpt-4o", requires_api_key: true, api_type: "openai" },
  { id: "anthropic", name: "Anthropic (Claude)", default_url: "https://api.anthropic.com", default_model: "claude-sonnet-4-20250514", requires_api_key: true, api_type: "anthropic" },
  { id: "google", name: "Google (Gemini)", default_url: "https://generativelanguage.googleapis.com/v1", default_model: "gemini-2.0-flash", requires_api_key: true, api_type: "google" },
  { id: "opencode", name: "OpenCode Zen", default_url: "https://opencode.ai/zen", default_model: "big-pickle", requires_api_key: true, api_type: "openai" },
  { id: "openrouter", name: "OpenRouter", default_url: "https://openrouter.ai/api/v1", default_model: "anthropic/claude-sonnet-4-20250514", requires_api_key: true, api_type: "openai" },
  { id: "groq", name: "Groq", default_url: "https://api.groq.com/openai/v1", default_model: "llama-3.3-70b-versatile", requires_api_key: true, api_type: "openai" },
  { id: "together", name: "Together AI", default_url: "https://api.together.ai/v1", default_model: "meta-llama/Llama-3.3-70B-Instruct", requires_api_key: true, api_type: "openai" },
  { id: "custom", name: "Custom (OpenAI-compatible)", default_url: "https://your-api.example.com/v1", default_model: "model-name", requires_api_key: false, api_type: "openai" },
];

interface LlmConfig {
  id: string;
  name: string;
  provider: string;
  url: string;
  model: string;
  has_api_key: boolean;
  ctx_length: number | null;
  ctx_history: number | null;
  vision: boolean | null;
  rl_requests: number | null;
  rl_input: number | null;
  rl_output: number | null;
  kwargs: string | null;
}

interface FormData {
  name: string;
  provider: string;
  url: string;
  model: string;
  api_key: string;
  ctx_length: number;
  ctx_history: number;
  vision: boolean;
  rl_requests: number;
  rl_input: number;
  rl_output: number;
  kwargs: string;
}

const DEFAULT_FORM: FormData = {
  name: '',
  provider: 'local',
  url: 'http://localhost:8080/v1',
  model: 'qwen3-4b',
  api_key: '',
  ctx_length: 4096,
  ctx_history: 0.3,
  vision: false,
  rl_requests: 0,
  rl_input: 0,
  rl_output: 0,
  kwargs: '',
};

export function ChatModelSettings() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [llms, setLlms] = useState<LlmConfig[]>([]);
  const [defaultLlmId, setDefaultLlmId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formData, setFormData] = useState<FormData>(DEFAULT_FORM);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<Record<string, { success: boolean; message: string; latency_ms?: number }>>({});
  const [availableModels, setAvailableModels] = useState<{id: string, name: string}[]>([]);
  const [loadingModels, setLoadingModels] = useState(false);
  const [showModelPicker, setShowModelPicker] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      // Check if router is reachable first (silently fail if not)
      await fetch('/api/v1/llms/providers', { method: 'GET' });
    } catch {
      // Silently fail - we'll get errors from the actual API calls anyway
    }
    
    try {
      const [llmsRes, defaultRes, providersRes] = await Promise.all([
        apiGet('/api/v1/llms').then(r => { if (!r.ok) throw new Error('Failed'); return r.json() as Promise<LlmConfig[]>; }).catch(() => []),
        apiGet('/api/v1/llms/default').then(r => r.ok ? r.json() as Promise<LlmConfig | null> : null).catch(() => null),
        apiGet('/api/v1/llms/providers').then(r => r.ok ? r.json() as Promise<ProviderInfo[]> : FALLBACK_PROVIDERS).catch(() => FALLBACK_PROVIDERS),
      ]);
      setLlms(llmsRes || []);
      setDefaultLlmId(defaultRes?.id || null);
      
      // Use API providers or fallback
      const providers = (providersRes && providersRes.length > 0) ? providersRes : FALLBACK_PROVIDERS;
      setProviders(providers);
      
      // Find default LLM or use first provider
      let defaultProvider = providers.find(p => p.id === 'local') || providers[0];
      
      // If there's a default LLM in the system, use its config
      if (defaultRes) {
        setFormData(prev => ({
          ...prev,
          name: defaultRes.name,
          provider: defaultRes.provider,
          url: defaultRes.url,
          model: defaultRes.model,
        }));
      } else if (defaultProvider) {
        setFormData(prev => ({
          ...prev,
          provider: defaultProvider.id,
          url: defaultProvider.default_url,
          model: defaultProvider.default_model,
        }));
      }
    } catch (err) {
      console.error('Failed to load data:', err);
      // Use fallback providers on error
      setProviders(FALLBACK_PROVIDERS);
      setError('Could not connect to router. Using offline mode.');
    } finally {
      setLoading(false);
    }
  };

  const handleProviderChange = (providerId: string) => {
    const provider = providers.find(p => p.id === providerId);
    if (provider) {
      setFormData(prev => ({
        ...prev,
        provider: providerId,
        url: provider.default_url,
        model: provider.default_model,
        api_key: provider.requires_api_key ? prev.api_key : '',
      }));
      // Reset model picker when provider changes
      setAvailableModels([]);
      setShowModelPicker(false);
    }
  };

  const handleBrowseModels = async () => {
    const provider = providers.find(p => p.id === formData.provider);
    if (!provider) return;
    
    setLoadingModels(true);
    setShowModelPicker(true);
    try {
      const res = await apiPost('/api/v1/llms/list-models', {
        url: formData.url,
        api_key: formData.api_key || null,
        provider: formData.provider,
      });
      const models = await res.json() as {id: string, name: string}[];
      setAvailableModels(models || []);
    } catch (err) {
      console.error('Failed to load models:', err);
      setAvailableModels([]);
    } finally {
      setLoadingModels(false);
    }
  };

  const handleSelectModel = (modelId: string) => {
    setFormData(prev => ({ ...prev, model: modelId }));
    setShowModelPicker(false);
  };

  const handleAddLlm = async () => {
    if (!formData.name || !formData.url || !formData.model) return;
    
    setSaving(true);
    setError(null);
    try {
      const res = await apiPost('/api/v1/llms', {
        name: formData.name,
        provider: formData.provider,
        url: formData.url,
        model: formData.model,
        api_key: formData.api_key || null,
        ctx_length: formData.ctx_length,
        ctx_history: formData.ctx_history,
        vision: formData.vision,
        rl_requests: formData.rl_requests,
        rl_input: formData.rl_input,
        rl_output: formData.rl_output,
        kwargs: formData.kwargs || null,
      });
      
      if (!res.ok) {
        const errData = await res.json().catch(() => ({ message: 'Failed to save LLM' }));
        setError(errData.message || 'Failed to save LLM');
        return;
      }
      
      const newLlm = await res.json() as LlmConfig;
      setLlms([...llms, newLlm]);
      setShowAddForm(false);
      setFormData(DEFAULT_FORM);
      // Set as default if first LLM
      if (llms.length === 0) {
        setDefaultLlmId(newLlm.id);
        await apiPut('/api/v1/llms/default', { id: newLlm.id });
      }
    } catch (err) {
      console.error('Failed to add LLM:', err);
      setError('Failed to add LLM. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  const handleUpdateLlm = async () => {
    if (!editingId || !formData.name) return;
    
    setSaving(true);
    try {
      const res = await apiPut(`/api/v1/llms/${editingId}`, {
        name: formData.name,
        provider: formData.provider,
        url: formData.url,
        model: formData.model,
        api_key: formData.api_key || null,
        ctx_length: formData.ctx_length,
        ctx_history: formData.ctx_history,
        vision: formData.vision,
        rl_requests: formData.rl_requests,
        rl_input: formData.rl_input,
        rl_output: formData.rl_output,
        kwargs: formData.kwargs || null,
      });
      const updated = await res.json() as LlmConfig;
      setLlms(llms.map(l => l.id === editingId ? updated : l));
      setEditingId(null);
      setFormData(DEFAULT_FORM);
    } catch (err) {
      console.error('Failed to update LLM:', err);
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteLlm = async (id: string) => {
    if (!confirm('Are you sure you want to delete this LLM?')) return;
    
    try {
      await apiDelete(`/api/v1/llms/${id}`);
      const remaining = llms.filter(l => l.id !== id);
      setLlms(remaining);
      if (defaultLlmId === id) {
        setDefaultLlmId(remaining.length > 0 ? remaining[0].id : null);
      }
    } catch (err) {
      console.error('Failed to delete LLM:', err);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      const res = await apiPut('/api/v1/llms/default', { id });
      const newDefault = await res.json() as LlmConfig;
      setDefaultLlmId(newDefault.id);
    } catch (err) {
      console.error('Failed to set default LLM:', err);
    }
  };

  const handleTestLlm = async (id: string) => {
    setTestingId(id);
    const newResults = { ...testResult };
    delete newResults[id];
    setTestResult(newResults);
    try {
      const res = await apiPost(`/api/v1/llms/${id}/test`, {});
      const result = await res.json();
      setTestResult({ ...testResult, [id]: result });
    } catch (err) {
      setTestResult({ ...testResult, [id]: { success: false, message: 'Failed to test connection' } });
    } finally {
      setTestingId(null);
    }
  };

  const startEdit = (llm: LlmConfig) => {
    setEditingId(llm.id);
    setFormData({
      name: llm.name,
      provider: llm.provider,
      url: llm.url,
      model: llm.model,
      api_key: '',
      ctx_length: llm.ctx_length || 4096,
      ctx_history: llm.ctx_history || 0.3,
      vision: llm.vision || false,
      rl_requests: llm.rl_requests || 0,
      rl_input: llm.rl_input || 0,
      rl_output: llm.rl_output || 0,
      kwargs: llm.kwargs || '',
    });
    setShowAddForm(false);
  };

  const currentProvider = providers.find(p => p.id === formData.provider);

  if (loading) {
    return <div className="p-4">Loading chat model settings...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">Chat Model</h3>
        <p className="text-sm text-muted-foreground">
          Selection and settings for main chat model used by APEX
        </p>
      </div>

      {/* Add/Edit Form */}
      {(showAddForm || editingId) && (
        <div className="border rounded-lg p-4 bg-card space-y-4">
          <h4 className="font-semibold">{editingId ? 'Edit LLM' : 'Add New LLM'}</h4>
          {error && (
            <div className="bg-red-500/10 border border-red-500 text-red-500 px-3 py-2 rounded text-sm">
              {error}
            </div>
          )}
          <div className="grid gap-4 max-w-2xl">
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Name</label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="My LLM"
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              />
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Provider</label>
              <select
                value={formData.provider}
                onChange={(e) => handleProviderChange(e.target.value)}
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              >
                <optgroup label="Local">
                  {providers.filter(p => ['local', 'ollama', 'vllm', 'lmstudio'].includes(p.id)).map(p => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </optgroup>
                <optgroup label="Cloud Providers">
                  {providers.filter(p => !['local', 'ollama', 'vllm', 'lmstudio'].includes(p.id)).map(p => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </optgroup>
              </select>
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">API URL</label>
              <input
                type="text"
                value={formData.url}
                onChange={(e) => setFormData({ ...formData, url: e.target.value })}
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              />
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Model</label>
              <div className="col-span-3 flex gap-2">
                <input
                  type="text"
                  value={formData.model}
                  onChange={(e) => setFormData({ ...formData, model: e.target.value })}
                  className="flex-1 px-2 py-1 rounded border bg-background text-foreground"
                  placeholder="Enter model name or browse..."
                />
                <button
                  type="button"
                  onClick={handleBrowseModels}
                  disabled={loadingModels || !formData.url}
                  className="px-3 py-1 text-sm border rounded hover:bg-muted disabled:opacity-50 whitespace-nowrap"
                >
                  {loadingModels ? 'Loading...' : 'Browse'}
                </button>
              </div>
            </div>
            {showModelPicker && (
              <div className="grid grid-cols-4 gap-2 items-start">
                <label className="text-sm pt-1">Available Models</label>
                <div className="col-span-3 border rounded max-h-48 overflow-y-auto bg-background">
                  {availableModels.length === 0 ? (
                    <div className="p-3 text-sm text-muted-foreground">
                      {loadingModels ? 'Loading models...' : 'No models found. Make sure URL and API key are correct.'}
                    </div>
                  ) : (
                    <div className="divide-y">
                      {availableModels.map((model) => (
                        <button
                          key={model.id}
                          type="button"
                          onClick={() => handleSelectModel(model.id)}
                          className="w-full text-left px-3 py-2 text-sm hover:bg-muted truncate"
                          title={model.name}
                        >
                          {model.id}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}
            {currentProvider?.requires_api_key && (
              <div className="grid grid-cols-4 gap-2 items-center">
                <label className="text-sm">API Key</label>
                <input
                  type="password"
                  value={formData.api_key}
                  onChange={(e) => setFormData({ ...formData, api_key: e.target.value })}
                  placeholder={currentProvider.id === 'azure' ? 'Azure API Key' : 'sk-...'}
                  className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
                />
              </div>
            )}
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Context Length</label>
              <input
                type="number"
                value={formData.ctx_length}
                onChange={(e) => setFormData({ ...formData, ctx_length: parseInt(e.target.value) || 4096 })}
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              />
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">History Ratio</label>
              <div className="col-span-3 flex items-center gap-2">
                <input
                  type="range"
                  min="0.01"
                  max="1"
                  step="0.01"
                  value={formData.ctx_history}
                  onChange={(e) => setFormData({ ...formData, ctx_history: parseFloat(e.target.value) })}
                  className="flex-1"
                />
                <span className="text-sm font-mono w-12">{formData.ctx_history.toFixed(2)}</span>
              </div>
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Vision</label>
              <input
                type="checkbox"
                checked={formData.vision}
                onChange={(e) => setFormData({ ...formData, vision: e.target.checked })}
                className="col-span-3"
              />
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Rate Limit (req/min)</label>
              <input
                type="number"
                value={formData.rl_requests}
                onChange={(e) => setFormData({ ...formData, rl_requests: parseInt(e.target.value) || 0 })}
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              />
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Additional Params</label>
              <textarea
                value={formData.kwargs}
                onChange={(e) => setFormData({ ...formData, kwargs: e.target.value })}
                placeholder="temperature=0.7&#10;top_p=0.9"
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground font-mono text-sm"
                rows={3}
              />
            </div>
            <div className="flex gap-2 pt-2">
              <button
                onClick={editingId ? handleUpdateLlm : handleAddLlm}
                disabled={saving || !formData.name || !formData.url || !formData.model}
                className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90 disabled:opacity-50"
              >
                {saving ? 'Saving...' : editingId ? 'Update' : 'Add LLM'}
              </button>
              <button
                onClick={() => {
                  setShowAddForm(false);
                  setEditingId(null);
                  // Reset to default provider or first available
                  const defaultProvider = providers.find(p => p.id === 'local') || providers[0] || FALLBACK_PROVIDERS[0];
                  setFormData({
                    ...DEFAULT_FORM,
                    provider: defaultProvider?.id || 'local',
                    url: defaultProvider?.default_url || 'http://localhost:8080/v1',
                    model: defaultProvider?.default_model || 'qwen3-4b',
                  });
                  setError(null);
                }}
                className="px-4 py-2 rounded border hover:bg-muted"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {/* LLM List */}
      <div className="flex justify-between items-center">
        <button
          onClick={() => { setShowAddForm(true); setEditingId(null); setFormData(DEFAULT_FORM); setError(null); }}
          className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90"
        >
          + Add LLM
        </button>
      </div>

      {llms.length === 0 ? (
        <div className="border rounded-lg p-8 text-center text-muted-foreground">
          No LLMs configured. Add an LLM to get started.
        </div>
      ) : (
        <div className="space-y-4">
          {llms.map((llm) => (
            <div key={llm.id} className="border rounded-lg p-4 bg-card">
              <div className="flex items-center justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span className="font-semibold">{llm.name}</span>
                    {defaultLlmId === llm.id && (
                      <span className="text-xs bg-primary/20 text-primary px-2 py-0.5 rounded">Default</span>
                    )}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    {providers.find(p => p.id === llm.provider)?.name || llm.provider} • {llm.model}
                  </div>
                  <div className="text-xs text-muted-foreground mt-1">
                    {llm.url}
                    {llm.has_api_key && ' • API Key configured'}
                    {llm.ctx_length && ` • Context: ${llm.ctx_length}`}
                    {llm.vision && ' • Vision'}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleTestLlm(llm.id)}
                    disabled={testingId === llm.id}
                    className="px-3 py-1 text-sm border rounded hover:bg-muted disabled:opacity-50"
                  >
                    {testingId === llm.id ? 'Testing...' : 'Test'}
                  </button>
                  {defaultLlmId !== llm.id && (
                    <button
                      onClick={() => handleSetDefault(llm.id)}
                      className="px-3 py-1 text-sm border rounded hover:bg-muted"
                    >
                      Set Default
                    </button>
                  )}
                  <button
                    onClick={() => startEdit(llm)}
                    className="px-3 py-1 text-sm border rounded hover:bg-muted"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => handleDeleteLlm(llm.id)}
                    className="px-3 py-1 text-sm border border-destructive text-destructive rounded hover:bg-destructive/10"
                  >
                    Delete
                  </button>
                </div>
              </div>
              
              {testResult[llm.id] && (
                <div className={`mt-3 text-sm ${testResult[llm.id].success ? 'text-green-500' : 'text-red-500'}`}>
                  {testResult[llm.id].success ? '✓' : '✗'} {testResult[llm.id].message}
                  {testResult[llm.id].latency_ms && ` (${testResult[llm.id].latency_ms}ms)`}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
