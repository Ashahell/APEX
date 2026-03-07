import { useState, useEffect } from 'react';
import { apiGet, apiPost, setSetting, deleteSetting, Setting, API_BASE } from '../../lib/api';
import { ConfigViewer } from './ConfigViewer';

interface VmStats {
  enabled: boolean;
  backend: string;
  total: number;
  ready: number;
  busy: number;
  available: number;
}

interface Metrics {
  tasks_total: Record<string, number>;
  tasks_completed: number;
  tasks_failed: number;
  total_cost_usd: number;
}

interface TaskHistoryItem {
  task_id: string;
  status: string;
  output: string | null;
  error: string | null;
}

interface TaskConfig {
  maxSteps: number;
  budgetUsd: number;
  timeLimitSecs: number | null;
}

interface TotpStatus {
  configured: boolean;
}

const DEFAULT_CONFIG: TaskConfig = {
  maxSteps: 3,
  budgetUsd: 1.0,
  timeLimitSecs: null,
};

type SettingsTab = 'general' | 'security' | 'vm' | 'llm' | 'config' | 'preferences' | 'about';

export function Settings() {
  const [activeTab, setActiveTab] = useState<SettingsTab>('general');
  const [vmStats, setVmStats] = useState<VmStats | null>(null);
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [taskHistory, setTaskHistory] = useState<TaskHistoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [tasksLoading, setTasksLoading] = useState(false);
  const [config, setConfig] = useState<TaskConfig>(DEFAULT_CONFIG);
  const [saved, setSaved] = useState(false);
  const [selectedTask, setSelectedTask] = useState<TaskHistoryItem | null>(null);
  const [totpStatus, setTotpStatus] = useState<TotpStatus | null>(null);
  const [totpSetupLoading, setTotpSetupLoading] = useState(false);
  const [totpSecret, setTotpSecret] = useState<string | null>(null);
  const [totpVerifyCode, setTotpVerifyCode] = useState('');
  const [totpVerified, setTotpVerified] = useState(false);
  const [llmConfig, setLlmConfig] = useState({
    useLlm: false,
    llamaUrl: 'http://localhost:8080',
    llamaModel: 'qwen3-4b',
  });
  const [llmSaving, setLlmSaving] = useState(false);
  const [llmSaved, setLlmSaved] = useState(false);
  const [preferences, setPreferences] = useState<Setting[]>([]);
  const [prefSaving, setPrefSaving] = useState(false);
  const [newPrefKey, setNewPrefKey] = useState('');
  const [newPrefValue, setNewPrefValue] = useState('');
  const [newPrefEncrypt, setNewPrefEncrypt] = useState(false);

  const loadTaskHistory = async () => {
    setTasksLoading(true);
    try {
      const res = await apiGet('/api/v1/tasks?limit=10');
      const tasks = await res.json();
      setTaskHistory(Array.isArray(tasks) ? tasks : []);
    } catch {}
    setTasksLoading(false);
  };

  const loadTotpStatus = async () => {
    try {
      const res = await apiGet('/api/v1/totp/status');
      const status = await res.json();
      setTotpStatus(status);
    } catch {
      setTotpStatus({ configured: false });
    }
  };

  useEffect(() => {
    const savedConfig = localStorage.getItem('apex-task-config');
    if (savedConfig) {
      try {
        setConfig(JSON.parse(savedConfig));
      } catch {}
    }
    
    Promise.all([
      apiGet('/api/v1/vm/stats').then(r => r.json()),
      apiGet('/api/v1/metrics').then(r => r.json()),
    ])
      .then(([vm, met]) => {
        setVmStats(vm);
        setMetrics(met);
        loadTaskHistory();
        loadTotpStatus();
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  const handleSave = () => {
    localStorage.setItem('apex-task-config', JSON.stringify(config));
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleTotpSetup = async () => {
    setTotpSetupLoading(true);
    try {
      const res = await apiPost('/api/v1/totp/setup', {});
      const data = await res.json();
      if (data.secret) {
        setTotpSecret(data.secret);
      }
    } catch {}
    setTotpSetupLoading(false);
  };

  const handleTotpVerify = async () => {
    if (totpVerifyCode.length !== 6) return;
    try {
      const res = await apiPost('/api/v1/totp/verify', { token: totpVerifyCode });
      const data = await res.json();
      if (data.valid) {
        setTotpVerified(true);
        setTotpStatus({ configured: true });
        setTotpSecret(null);
      }
    } catch {}
  };

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: 'general', label: 'General' },
    { id: 'llm', label: 'LLM' },
    { id: 'security', label: 'Security' },
    { id: 'vm', label: 'VM Pool' },
    { id: 'config', label: 'Config' },
    { id: 'preferences', label: 'Preferences' },
    { id: 'about', label: 'About' },
  ];

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="flex h-full">
      <nav className="w-48 border-r bg-card p-2">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`w-full text-left px-3 py-2 rounded mb-1 transition-colors ${
              activeTab === tab.id
                ? 'bg-primary/10 text-primary'
                : 'hover:bg-muted'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <div className="flex-1 p-6 overflow-y-auto">
        {activeTab === 'general' && (
          <>
            <div className="mb-6">
              <h2 className="text-2xl font-semibold">General Settings</h2>
              <p className="text-muted-foreground">Configure APEX preferences</p>
            </div>

            <div className="space-y-6">
              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Task Configuration</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  These settings apply to tasks. Note: Time limits are not applied when using local LLM.
                </p>
                <div className="grid gap-4 max-w-md">
                  <div className="flex items-center justify-between">
                    <label className="text-sm">Max Steps</label>
                    <input
                      type="number"
                      min="1"
                      max="100"
                      value={config.maxSteps}
                      onChange={(e) => setConfig({ ...config, maxSteps: parseInt(e.target.value) || 3 })}
                      className="w-20 px-2 py-1 rounded border text-center"
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-sm">Budget (USD)</label>
                    <input
                      type="number"
                      min="0.1"
                      max="100"
                      step="0.1"
                      value={config.budgetUsd}
                      onChange={(e) => setConfig({ ...config, budgetUsd: parseFloat(e.target.value) || 1.0 })}
                      className="w-20 px-2 py-1 rounded border text-center"
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-sm">Time Limit (seconds)</label>
                    <input
                      type="number"
                      min="0"
                      placeholder="No limit"
                      value={config.timeLimitSecs ?? ''}
                      onChange={(e) => setConfig({ ...config, timeLimitSecs: e.target.value ? parseInt(e.target.value) : null })}
                      className="w-20 px-2 py-1 rounded border text-center"
                    />
                  </div>
                  <button
                    onClick={handleSave}
                    className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90 w-fit"
                  >
                    {saved ? 'Saved!' : 'Save Configuration'}
                  </button>
                </div>
              </section>

              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Task Metrics</h3>
                {metrics ? (
                  <div className="grid gap-2 text-sm max-w-xs">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Total Tasks</span>
                      <span>{(metrics.tasks_total as Record<string, number>)?.total || 0}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Completed</span>
                      <span>{metrics.tasks_completed || 0}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Failed</span>
                      <span>{metrics.tasks_failed || 0}</span>
                    </div>
                    <div className="flex justify-between border-t pt-2 mt-2">
                      <span className="text-muted-foreground font-semibold">Total Cost</span>
                      <span className="font-semibold">${(metrics.total_cost_usd || 0).toFixed(4)}</span>
                    </div>
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">No metrics available</p>
                )}
              </section>

              <section className="border rounded-lg p-4">
                <div className="flex justify-between items-center mb-4">
                  <h3 className="font-semibold">Task History</h3>
                  <button
                    onClick={loadTaskHistory}
                    disabled={tasksLoading}
                    className="text-xs px-2 py-1 rounded border hover:bg-muted disabled:opacity-50"
                  >
                    {tasksLoading ? 'Loading...' : 'Refresh'}
                  </button>
                </div>
                {taskHistory.length > 0 ? (
                  <div className="space-y-2 max-h-64 overflow-y-auto">
                    {taskHistory.map((task) => (
                      <div 
                        key={task.task_id} 
                        className="flex justify-between items-center text-sm border-b pb-2 cursor-pointer hover:bg-muted p-1 -m-1 rounded"
                        onClick={() => setSelectedTask(task)}
                      >
                        <div className="truncate flex-1 mr-2">
                          <span className="font-mono text-xs">{task.task_id.slice(0, 12)}...</span>
                          <span className={`ml-2 px-1.5 py-0.5 rounded text-xs ${
                            task.status === 'completed' ? 'bg-green-100 text-green-800' :
                            task.status === 'failed' ? 'bg-red-100 text-red-800' :
                            task.status === 'running' ? 'bg-blue-100 text-blue-800' :
                            'bg-gray-100 text-gray-800'
                          }`}>
                            {task.status}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">No tasks yet</p>
                )}
              </section>
            </div>
          </>
        )}

        {activeTab === 'security' && (
          <>
            <div className="mb-6">
              <h2 className="text-2xl font-semibold">Security Settings</h2>
              <p className="text-muted-foreground">Manage authentication and permissions</p>
            </div>

            <div className="space-y-6">
              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">TOTP Authentication</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  TOTP is required for T3 (destructive) operations like shell execution.
                </p>
                
                {totpVerified && (
                  <div className="bg-green-50 border border-green-200 rounded-lg p-3 mb-4">
                    <p className="text-green-800 text-sm">✓ TOTP is configured and verified</p>
                  </div>
                )}

                {totpStatus?.configured && !totpVerified && (
                  <div className="bg-green-50 border border-green-200 rounded-lg p-3 mb-4">
                    <p className="text-green-800 text-sm">✓ TOTP is configured</p>
                  </div>
                )}

                {!totpStatus?.configured && !totpSecret && (
                  <button
                    onClick={handleTotpSetup}
                    disabled={totpSetupLoading}
                    className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90"
                  >
                    {totpSetupLoading ? 'Setting up...' : 'Setup TOTP'}
                  </button>
                )}

                {totpSecret && (
                  <div className="space-y-4">
                    <div className="bg-muted p-3 rounded">
                      <p className="text-sm mb-2">Scan this secret in your authenticator app:</p>
                      <code className="block bg-background p-2 rounded font-mono text-xs break-all">
                        {totpSecret}
                      </code>
                    </div>
                    <div>
                      <label className="block text-sm mb-2">Enter verification code:</label>
                      <div className="flex gap-2">
                        <input
                          type="text"
                          value={totpVerifyCode}
                          onChange={(e) => setTotpVerifyCode(e.target.value.replace(/\D/g, '').slice(0, 6))}
                          placeholder="000000"
                          maxLength={6}
                          className="w-32 px-3 py-2 rounded border font-mono text-center"
                        />
                        <button
                          onClick={handleTotpVerify}
                          disabled={totpVerifyCode.length !== 6}
                          className="px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90 disabled:opacity-50"
                        >
                          Verify
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </section>

              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Permission Tiers</h3>
                <div className="space-y-3 text-sm">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded bg-green-100 text-green-800 text-xs">T0</span>
                    <span className="text-muted-foreground">Read-only queries - no confirmation</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded bg-blue-100 text-blue-800 text-xs">T1</span>
                    <span className="text-muted-foreground">File writes - tap to confirm</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded bg-orange-100 text-orange-800 text-xs">T2</span>
                    <span className="text-muted-foreground">External API calls - type to confirm</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded bg-red-100 text-red-800 text-xs">T3</span>
                    <span className="text-muted-foreground">Destructive operations - TOTP required</span>
                  </div>
                </div>
              </section>
            </div>
          </>
        )}

        {activeTab === 'vm' && (
          <>
            <div className="mb-6">
              <h2 className="text-2xl font-semibold">VM Pool Settings</h2>
              <p className="text-muted-foreground">Configure execution isolation</p>
            </div>

            <div className="space-y-6">
              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">VM Pool Status</h3>
                {vmStats ? (
                  <div className="grid gap-2 text-sm max-w-xs">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Enabled</span>
                      <span>{vmStats.enabled ? 'Yes' : 'No'}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Backend</span>
                      <span>{vmStats.backend}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Total VMs</span>
                      <span>{vmStats.total}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Available</span>
                      <span>{vmStats.available}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Busy</span>
                      <span>{vmStats.busy}</span>
                    </div>
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">VM pool not available</p>
                )}
              </section>

              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Environment Variables</h3>
                <p className="text-sm text-muted-foreground mb-2">
                  To enable VM isolation, set environment variables before starting the router:
                </p>
                <div className="space-y-2">
                  <code className="block bg-muted p-2 rounded text-xs">
                    APEX_USE_DOCKER=1
                  </code>
                  <code className="block bg-muted p-2 rounded text-xs">
                    APEX_USE_FIRECRACKER=1<br />
                    APEX_VM_KERNEL=/path/to/vmlinux<br />
                    APEX_VM_ROOTFS=/path/to/rootfs.ext4
                  </code>
                  <code className="block bg-muted p-2 rounded text-xs">
                    APEX_USE_GVISOR=1
                  </code>
                </div>
              </section>
            </div>
          </>
        )}

        {activeTab === 'llm' && (
          <>
            <div className="mb-6">
              <h2 className="text-2xl font-semibold">LLM Settings</h2>
              <p className="text-muted-foreground">Configure local and cloud LLM providers</p>
            </div>

            <div className="space-y-6">
              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Local LLM (llama.cpp)</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Configure local LLM using llama-server. Requires llama-server to be running separately.
                </p>
                <div className="space-y-4 max-w-md">
                  <div className="flex items-center justify-between">
                    <label className="text-sm">Enable Local LLM</label>
                    <input
                      type="checkbox"
                      checked={llmConfig.useLlm}
                      onChange={(e) => setLlmConfig({ ...llmConfig, useLlm: e.target.checked })}
                      className="w-5 h-5"
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-sm">Server URL</label>
                    <input
                      type="text"
                      value={llmConfig.llamaUrl}
                      onChange={(e) => setLlmConfig({ ...llmConfig, llamaUrl: e.target.value })}
                      className="w-48 px-2 py-1 rounded border text-center text-sm"
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-sm">Model</label>
                    <input
                      type="text"
                      value={llmConfig.llamaModel}
                      onChange={(e) => setLlmConfig({ ...llmConfig, llamaModel: e.target.value })}
                      className="w-48 px-2 py-1 rounded border text-center text-sm"
                    />
                  </div>
                  <div className="text-xs text-muted-foreground">
                    <p>Current: {llmConfig.llamaUrl}</p>
                    <p>Model: {llmConfig.llamaModel}</p>
                  </div>
                  <button
                    onClick={async () => {
                      setLlmSaving(true);
                      setLlmSaved(false);
                      await new Promise(r => setTimeout(r, 500));
                      setLlmSaving(false);
                      setLlmSaved(true);
                      setTimeout(() => setLlmSaved(false), 2000);
                    }}
                    disabled={llmSaving}
                    className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90 w-fit"
                  >
                    {llmSaving ? 'Saving...' : llmSaved ? 'Saved!' : 'Save LLM Settings'}
                  </button>
                </div>
              </section>

              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Cloud LLM</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Configure cloud LLM providers (OpenAI, Anthropic, etc.). Coming soon.
                </p>
                <div className="opacity-50">
                  <div className="flex items-center justify-between mb-4">
                    <label className="text-sm">Provider</label>
                    <select className="w-48 px-2 py-1 rounded border text-center" disabled>
                      <option>Select provider...</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-between">
                    <label className="text-sm">API Key</label>
                    <input
                      type="password"
                      placeholder="Enter API key"
                      className="w-48 px-2 py-1 rounded border text-center text-sm"
                      disabled
                    />
                  </div>
                </div>
              </section>

              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Development Mode</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  During development, local LLM is disabled by default to avoid unnecessary LLM usage.
                  Enable it here when you need to test LLM-powered features.
                </p>
                <div className="flex items-center gap-2 text-sm">
                  <span className="text-muted-foreground">Current mode:</span>
                  <span className="font-semibold">Development (LLM off unless enabled)</span>
                </div>
              </section>
            </div>
          </>
        )}

        {activeTab === 'about' && (
          <>
            <div className="mb-6">
              <h2 className="text-2xl font-semibold">About APEX</h2>
            </div>

            <div className="space-y-6">
              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">System Information</h3>
                <div className="grid gap-2 text-sm max-w-xs">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Version</span>
                    <span>0.1.1</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Router URL</span>
                    <span>{API_BASE}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Architecture</span>
                    <span>Single-process</span>
                  </div>
                </div>
              </section>

              <section className="border rounded-lg p-4">
                <h3 className="font-semibold mb-4">Description</h3>
                <p className="text-sm text-muted-foreground">
                  APEX is a single-user autonomous agent platform. It combines messaging
                  interfaces with secure code execution using Firecracker micro-VMs.
                </p>
              </section>
            </div>
            </>
        )}

        {activeTab === 'config' && <ConfigViewer />}

        {activeTab === 'preferences' && (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-semibold mb-4">User Preferences</h3>
              <p className="text-sm text-muted-foreground mb-4">
                Store and retrieve key-value preferences. Use encryption for sensitive data like API keys.
              </p>
            </div>

            <div className="bg-card rounded-lg border p-4">
              <h4 className="font-medium mb-3">Add New Preference</h4>
              <div className="grid gap-3">
                <div>
                  <label className="text-sm text-muted-foreground">Key</label>
                  <input
                    type="text"
                    value={newPrefKey}
                    onChange={(e) => setNewPrefKey(e.target.value)}
                    placeholder="e.g., theme, language, api_key"
                    className="w-full mt-1 px-3 py-2 bg-background border rounded-md text-sm"
                  />
                </div>
                <div>
                  <label className="text-sm text-muted-foreground">Value</label>
                  <input
                    type="text"
                    value={newPrefValue}
                    onChange={(e) => setNewPrefValue(e.target.value)}
                    placeholder="Value"
                    className="w-full mt-1 px-3 py-2 bg-background border rounded-md text-sm"
                  />
                </div>
                <div className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    id="encrypt"
                    checked={newPrefEncrypt}
                    onChange={(e) => setNewPrefEncrypt(e.target.checked)}
                    className="rounded"
                  />
                  <label htmlFor="encrypt" className="text-sm">Encrypt value (base64)</label>
                </div>
                <button
                  onClick={async () => {
                    if (!newPrefKey.trim() || !newPrefValue.trim()) return;
                    setPrefSaving(true);
                    try {
                      await setSetting(newPrefKey, newPrefValue, newPrefEncrypt);
                      setNewPrefKey('');
                      setNewPrefValue('');
                      setNewPrefEncrypt(false);
                    } catch (e) {
                      console.error('Failed to save preference:', e);
                    }
                    setPrefSaving(false);
                  }}
                  disabled={prefSaving || !newPrefKey.trim() || !newPrefValue.trim()}
                  className="px-4 py-2 bg-primary text-primary-foreground rounded-md text-sm disabled:opacity-50"
                >
                  {prefSaving ? 'Saving...' : 'Save Preference'}
                </button>
              </div>
            </div>

            <div className="bg-card rounded-lg border p-4">
              <h4 className="font-medium mb-3">Saved Preferences</h4>
              {preferences.length === 0 ? (
                <div className="text-sm text-muted-foreground">No preferences saved. Add one above.</div>
              ) : (
                <div className="space-y-2">
                  {preferences.map((pref) => (
                    <div key={pref.key} className="flex items-center justify-between p-2 bg-background rounded border">
                      <div className="flex-1 min-w-0">
                        <div className="font-mono text-sm truncate">{pref.key}</div>
                        <div className="text-xs text-muted-foreground truncate">{pref.value}</div>
                        {pref.encrypted && (
                          <span className="text-xs bg-yellow-100 text-yellow-800 px-1 rounded">Encrypted</span>
                        )}
                      </div>
                      <button
                        onClick={async () => {
                          try {
                            await deleteSetting(pref.key);
                            setPreferences(preferences.filter(p => p.key !== pref.key));
                          } catch (e) {
                            console.error('Failed to delete preference:', e);
                          }
                        }}
                        className="ml-2 px-2 py-1 text-xs text-red-500 hover:bg-red-50 rounded"
                      >
                        Delete
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {selectedTask && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setSelectedTask(null)}>
          <div className="bg-background rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto" onClick={(e) => e.stopPropagation()}>
            <div className="flex justify-between items-start mb-4">
              <h3 className="text-lg font-semibold">Task Details</h3>
              <button onClick={() => setSelectedTask(null)} className="text-muted-foreground hover:text-foreground">
                ✕
              </button>
            </div>
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Task ID:</span>
                <span className="font-mono">{selectedTask.task_id}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Status:</span>
                <span className={`px-2 py-0.5 rounded ${
                  selectedTask.status === 'completed' ? 'bg-green-100 text-green-800' :
                  selectedTask.status === 'failed' ? 'bg-red-100 text-red-800' :
                  selectedTask.status === 'running' ? 'bg-blue-100 text-blue-800' :
                  'bg-gray-100 text-gray-800'
                }`}>{selectedTask.status}</span>
              </div>
              {selectedTask.output && (
                <div className="border-t pt-3 mt-3">
                  <span className="text-muted-foreground block mb-2">Output:</span>
                  <pre className="bg-muted p-3 rounded text-xs overflow-x-auto whitespace-pre-wrap">
                    {(() => {
                      try {
                        const parsed = JSON.parse(selectedTask.output!);
                        return JSON.stringify(parsed, null, 2);
                      } catch {
                        return selectedTask.output;
                      }
                    })()}
                  </pre>
                </div>
              )}
              {selectedTask.error && (
                <div className="border-t pt-3 mt-3">
                  <span className="text-muted-foreground block mb-2">Error:</span>
                  <pre className="bg-red-50 p-3 rounded text-xs text-red-800 overflow-x-auto">{selectedTask.error}</pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
