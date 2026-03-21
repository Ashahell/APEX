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
  use_chat_model: boolean;
}

const DEFAULT_FORM: FormData = {
  name: '',
  provider: 'local',
  url: 'http://localhost:8080/v1',
  model: 'qwen3-4b',
  api_key: '',
  use_chat_model: false,
};

export function BrowserModelSettings() {
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [llms, setLlms] = useState<LlmConfig[]>([]);
  const [defaultLlmId, setDefaultLlmId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [formData, setFormData] = useState<FormData>(DEFAULT_FORM);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<Record<string, { success: boolean; message: string; latency_ms?: number }>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [llmsRes, defaultRes, providersRes] = await Promise.all([
        apiGet('/api/v1/llms').then(r => r.ok ? r.json() as Promise<LlmConfig[]> : []).catch(() => []),
        apiGet('/api/v1/llms/default').then(r => r.ok ? r.json() as Promise<LlmConfig | null> : null).catch(() => null),
        apiGet('/api/v1/llms/providers').then(r => r.ok ? r.json() as Promise<ProviderInfo[]> : FALLBACK_PROVIDERS).catch(() => FALLBACK_PROVIDERS),
      ]);
      setLlms(llmsRes || []);
      setDefaultLlmId(defaultRes?.id || null);
      setProviders(providersRes || FALLBACK_PROVIDERS);
    } catch (err) {
      console.error('Failed to load data:', err);
      setProviders(FALLBACK_PROVIDERS);
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
    }
  };

  const handleTestLlm = async (id: string) => {
    setTestingId(id);
    try {
      const res = await apiPost(`/api/v1/llms/${id}/test`, {});
      const data = await res.json();
      setTestResult(prev => ({ ...prev, [id]: data }));
    } catch (err) {
      setTestResult(prev => ({ ...prev, [id]: { success: false, message: 'Failed to test connection' } }));
    } finally {
      setTestingId(null);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await apiPut('/api/v1/llms/default', { id });
      setDefaultLlmId(id);
    } catch (err) {
      console.error('Failed to set default:', err);
    }
  };

  const handleDeleteLlm = async (id: string) => {
    if (!confirm('Are you sure you want to delete this LLM?')) return;
    try {
      await apiDelete(`/api/v1/llms/${id}`);
      setLlms(llms.filter(l => l.id !== id));
      if (defaultLlmId === id) {
        setDefaultLlmId(null);
      }
    } catch (err) {
      console.error('Failed to delete LLM:', err);
    }
  };

  const handleSave = async () => {
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
        ctx_length: 4096,
        ctx_history: 0.3,
        vision: false,
        rl_requests: 0,
        rl_input: 0,
        rl_output: 0,
        kwargs: null,
      });
      
      if (!res.ok) {
        const errData = await res.json().catch(() => ({ message: 'Failed to save LLM' }));
        setError(errData.message || 'Failed to save LLM');
        return;
      }
      
      const newLlm = await res.json() as LlmConfig;
      setLlms([...llms, newLlm]);
      setFormData(DEFAULT_FORM);
      
      // Set as default
      setDefaultLlmId(newLlm.id);
      await apiPut('/api/v1/llms/default', { id: newLlm.id });
    } catch (err) {
      console.error('Failed to add LLM:', err);
      setError('Failed to add LLM. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="p-4">Loading browser model settings...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold">Browser Model</h3>
        <p className="text-sm text-muted-foreground">
          Model used for web browsing and scraping tasks
        </p>
      </div>

      {/* Add New LLM Form */}
      <div className="border rounded-lg p-4 bg-card space-y-4">
        <h4 className="font-semibold">Add Browser Model</h4>
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
              placeholder="My Browser Model"
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
            <input
              type="text"
              value={formData.model}
              onChange={(e) => setFormData({ ...formData, model: e.target.value })}
              className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
            />
          </div>
          {providers.find(p => p.id === formData.provider)?.requires_api_key && (
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">API Key</label>
              <input
                type="password"
                value={formData.api_key}
                onChange={(e) => setFormData({ ...formData, api_key: e.target.value })}
                placeholder="sk-..."
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              />
            </div>
          )}
          <div className="flex gap-2 pt-2">
            <button
              onClick={handleSave}
              disabled={saving || !formData.name || !formData.url || !formData.model}
              className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90 disabled:opacity-50"
            >
              {saving ? 'Saving...' : 'Add Browser Model'}
            </button>
          </div>
        </div>
      </div>

      {/* LLM List */}
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
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleTestLlm(llm.id)}
                    disabled={testingId === llm.id}
                    className="px-3 py-1 text-sm rounded border hover:bg-muted disabled:opacity-50"
                  >
                    {testingId === llm.id ? 'Testing...' : testResult[llm.id] ? (testResult[llm.id].success ? '✓' : '✗') : 'Test'}
                  </button>
                  {defaultLlmId !== llm.id && (
                    <button
                      onClick={() => handleSetDefault(llm.id)}
                      className="px-3 py-1 text-sm rounded border hover:bg-muted"
                    >
                      Set Default
                    </button>
                  )}
                  <button
                    onClick={() => handleDeleteLlm(llm.id)}
                    className="px-3 py-1 text-sm rounded border hover:bg-muted text-red-500"
                  >
                    Delete
                  </button>
                </div>
              </div>
              {testResult[llm.id] && (
                <div className={`mt-2 text-sm ${testResult[llm.id].success ? 'text-green-500' : 'text-red-500'}`}>
                  {testResult[llm.id].message}
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
