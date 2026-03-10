import { useState, useEffect } from 'react';
import { apiGet, apiPost, apiPut, apiDelete } from '../../lib/api';

interface LlmConfig {
  id: string;
  name: string;
  provider: string;
  url: string;
  model: string;
  has_api_key: boolean;
}

interface ProviderInfo {
  id: string;
  name: string;
  default_url: string;
  default_model: string;
  requires_api_key: boolean;
  api_type: string;
}

interface CreateLlmRequest {
  name: string;
  provider: string;
  url: string;
  model: string;
  api_key?: string;
}

interface TestResult {
  success: boolean;
  message: string;
  latency_ms?: number;
}

export function LlmManager() {
  const [llms, setLlms] = useState<LlmConfig[]>([]);
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [defaultLlm, setDefaultLlm] = useState<LlmConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showAddForm, setShowAddForm] = useState(false);
  const [testingId, setTestingId] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<Record<string, TestResult>>({});

  const [formData, setFormData] = useState<CreateLlmRequest>({
    name: '',
    provider: 'local',
    url: 'http://localhost:8080/v1',
    model: 'qwen3-4b',
    api_key: '',
  });

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [llmsRes, defaultRes, providersRes] = await Promise.all([
        apiGet('/api/v1/llms').then(r => r.json() as Promise<LlmConfig[]>),
        apiGet('/api/v1/llms/default').then(r => r.json() as Promise<LlmConfig | null>),
        apiGet('/api/v1/llms/providers').then(r => r.json() as Promise<ProviderInfo[]>),
      ]);
      setLlms(llmsRes || []);
      setDefaultLlm(defaultRes);
      setProviders(providersRes || []);
      
      // Set defaults from first provider
      if (providersRes && providersRes.length > 0) {
        const defaultProvider = providersRes.find(p => p.id === 'local') || providersRes[0];
        setFormData(prev => ({
          ...prev,
          provider: defaultProvider.id,
          url: defaultProvider.default_url,
          model: defaultProvider.default_model,
        }));
      }
    } catch (err) {
      console.error('Failed to load data:', err);
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

  const handleAddLlm = async () => {
    if (!formData.name || !formData.url || !formData.model) return;
    
    setSaving(true);
    try {
      const res = await apiPost('/api/v1/llms', {
        ...formData,
        api_key: formData.api_key || null,
      });
      const newLlm = await res.json() as LlmConfig;
      setLlms([...llms, newLlm]);
      setShowAddForm(false);
      setFormData({
        name: '',
        provider: 'local',
        url: 'http://localhost:8080/v1',
        model: 'qwen3-4b',
        api_key: '',
      });
    } catch (err) {
      console.error('Failed to add LLM:', err);
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
      if (defaultLlm?.id === id) {
        setDefaultLlm(remaining.length > 0 ? remaining[0] : null);
      }
    } catch (err) {
      console.error('Failed to delete LLM:', err);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      const res = await apiPut('/api/v1/llms/default', { id });
      const newDefault = await res.json() as LlmConfig;
      setDefaultLlm(newDefault);
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
      const result = await res.json() as TestResult;
      setTestResult({ ...testResult, [id]: result });
    } catch (err) {
      setTestResult({ ...testResult, [id]: { success: false, message: 'Failed to test connection' } });
    } finally {
      setTestingId(null);
    }
  };

  const currentProvider = providers.find(p => p.id === formData.provider);

  if (loading) {
    return <div className="p-4">Loading LLMs...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">LLM Configuration</h2>
          <p className="text-muted-foreground">Manage multiple LLM providers</p>
        </div>
        <button
          onClick={() => setShowAddForm(true)}
          className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90"
        >
          + Add LLM
        </button>
      </div>

      {/* Add LLM Form */}
      {showAddForm && (
        <div className="border rounded-lg p-4 bg-card">
          <h3 className="font-semibold mb-4">Add New LLM</h3>
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
                placeholder="https://api.example.com/v1"
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              />
            </div>
            <div className="grid grid-cols-4 gap-2 items-center">
              <label className="text-sm">Model</label>
              <input
                type="text"
                value={formData.model}
                onChange={(e) => setFormData({ ...formData, model: e.target.value })}
                placeholder="model-name"
                className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
              />
            </div>
            {currentProvider?.requires_api_key && (
              <div className="grid grid-cols-4 gap-2 items-center">
                <label className="text-sm">API Key</label>
                <input
                  type="password"
                  value={formData.api_key || ''}
                  onChange={(e) => setFormData({ ...formData, api_key: e.target.value })}
                  placeholder={currentProvider.id === 'azure' ? 'Azure API Key' : 'sk-...'}
                  className="col-span-3 px-2 py-1 rounded border bg-background text-foreground"
                />
              </div>
            )}
            <div className="flex gap-2 pt-2">
              <button
                onClick={handleAddLlm}
                disabled={saving || !formData.name || !formData.url || !formData.model}
                className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90 disabled:opacity-50"
              >
                {saving ? 'Adding...' : 'Add LLM'}
              </button>
              <button
                onClick={() => {
                  setShowAddForm(false);
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
      {llms.length === 0 ? (
        <div className="border rounded-lg p-8 text-center text-muted-foreground">
          No LLMs configured. Add an LLM to get started.
        </div>
      ) : (
        <div className="space-y-4">
          {llms.map((llm) => (
            <div key={llm.id} className="border rounded-lg p-4 bg-card">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold">{llm.name}</span>
                      {defaultLlm?.id === llm.id && (
                        <span className="text-xs bg-primary/20 text-primary px-2 py-0.5 rounded">
                          Default
                        </span>
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
                </div>
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => handleTestLlm(llm.id)}
                    disabled={testingId === llm.id}
                    className="px-3 py-1 text-sm border rounded hover:bg-muted disabled:opacity-50"
                  >
                    {testingId === llm.id ? 'Testing...' : 'Test'}
                  </button>
                  {defaultLlm?.id !== llm.id && (
                    <button
                      onClick={() => handleSetDefault(llm.id)}
                      className="px-3 py-1 text-sm border rounded hover:bg-muted"
                    >
                      Set Default
                    </button>
                  )}
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

      {/* Provider Tips */}
      <div className="border rounded-lg p-4 bg-muted/50">
        <h3 className="font-semibold mb-2">Supported Providers</h3>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-2 text-sm text-muted-foreground">
          {providers.slice(0, 12).map(p => (
            <div key={p.id} className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-primary"></span>
              {p.name}
            </div>
          ))}
        </div>
        <p className="text-xs text-muted-foreground mt-2">
          Plus many more cloud providers (Azure, Google, Cohere, etc.)
        </p>
      </div>
    </div>
  );
}
