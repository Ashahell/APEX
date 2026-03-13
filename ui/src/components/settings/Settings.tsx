import { useState, useEffect } from 'react';
import { apiGet, apiPost, setSetting, deleteSetting, Setting, API_BASE } from '../../lib/api';
import { ConfigViewer } from './ConfigViewer';
import { McpManager } from './McpManager';
import { ChatModelSettings } from './ChatModelSettings';
import { EmbedModelSettings } from './EmbedModelSettings';
import { ApiKeysManager } from './ApiKeysManager';
import { LiteLlmSettings } from './LiteLlmSettings';
import { SecretsManager } from './SecretsManager';
import { RuntimeSettings } from './RuntimeSettings';

interface VmStats {
  enabled: boolean;
  backend: string;
  total: number;
  ready: number;
  busy: number;
  available: number;
}

interface TotpStatus {
  configured: boolean;
}

type SettingsTab = 'agent' | 'external' | 'mcp' | 'skills' | 'security' | 'vm' | 'runtime' | 'config' | 'preferences' | 'about' | 'notifications' | 'developer' | 'backup' | 'speech' | 'a2a';
type AgentSubTab = 'chat' | 'embed' | 'util' | 'browser' | 'memory';
type ExternalSubTab = 'apikeys' | 'litellm' | 'secrets' | 'auth' | 'externalapi' | 'updatechecker' | 'tunnel';

export function Settings() {
  const [activeTab, setActiveTab] = useState<SettingsTab>('agent');
  const [agentSubTab, setAgentSubTab] = useState<AgentSubTab>('chat');
  const [externalSubTab, setExternalSubTab] = useState<ExternalSubTab>('apikeys');
  const [vmStats, setVmStats] = useState<VmStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [totpStatus, setTotpStatus] = useState<TotpStatus | null>(null);
  const [totpSetupLoading, setTotpSetupLoading] = useState(false);
  const [totpSecret, setTotpSecret] = useState<string | null>(null);
  const [totpVerifyCode, setTotpVerifyCode] = useState('');
  const [totpVerified, setTotpVerified] = useState(false);
  const [preferences, setPreferences] = useState<Setting[]>([]);
  const [prefSaving, setPrefSaving] = useState(false);
  const [newPrefKey, setNewPrefKey] = useState('');
  const [newPrefValue, setNewPrefValue] = useState('');
  const [newPrefEncrypt, setNewPrefEncrypt] = useState(false);
  const [enableTir, setEnableTir] = useState(false);
  const [enableSubagents, setEnableSubagents] = useState(true);

  // Load TIR and subagent settings on mount (from localStorage as fallback)
  useEffect(() => {
    try {
      const saved = localStorage.getItem('apex-task-config');
      if (saved) {
        const parsed = JSON.parse(saved);
        setEnableTir(parsed.useTir ?? false);
        setEnableSubagents(parsed.enableSubagents ?? true);
      }
    } catch (e) {
      console.warn('Failed to load agent settings:', e);
    }
  }, []);

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
    Promise.all([
      apiGet('/api/v1/vm/stats').then(r => r.json()),
    ])
      .then(([vm]) => {
        setVmStats(vm);
        loadTotpStatus();
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

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
    { id: 'agent', label: 'Agent' },
    { id: 'external', label: 'External' },
    { id: 'mcp', label: 'MCP' },
    { id: 'skills', label: 'Skills' },
    { id: 'security', label: 'Security' },
    { id: 'vm', label: 'VM Pool' },
    { id: 'runtime', label: 'Runtime' },
    { id: 'config', label: 'Config' },
    { id: 'preferences', label: 'Preferences' },
    { id: 'notifications', label: 'Notifications' },
    { id: 'developer', label: 'Developer' },
    { id: 'backup', label: 'Backup' },
    { id: 'speech', label: 'Speech' },
    { id: 'a2a', label: 'A2A' },
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
        {/* Agent Tab with Sub-tabs */}
        {activeTab === 'agent' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-semibold">Agent Settings</h2>
              <p className="text-muted-foreground">Configure model settings for APEX</p>
            </div>
            
            {/* Sub-tabs for Agent */}
            <div className="flex gap-1 border-b">
              <button
                onClick={() => setAgentSubTab('chat')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  agentSubTab === 'chat'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                Chat Model
              </button>
              <button
                onClick={() => setAgentSubTab('embed')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  agentSubTab === 'embed'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                Embed Model
              </button>
              <button
                onClick={() => setAgentSubTab('util')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  agentSubTab === 'util'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                Util Model
              </button>
              <button
                onClick={() => setAgentSubTab('browser')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  agentSubTab === 'browser'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                Browser Model
              </button>
              <button
                onClick={() => setAgentSubTab('memory')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  agentSubTab === 'memory'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                Memory
              </button>
            </div>
            
              {/* Sub-tab content */}
            <div className="border rounded-lg p-4">
              {agentSubTab === 'chat' && <ChatModelSettings />}
              {agentSubTab === 'embed' && <EmbedModelSettings />}
              {agentSubTab === 'util' && (
                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Util Model</h3>
                    <p className="text-sm text-muted-foreground mb-4">Model used for utility tasks like classification and routing</p>
                  </div>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div>
                      <label className="block text-sm font-medium mb-2">Provider</label>
                      <select className="w-full px-3 py-2 rounded-lg border bg-background">
                        <option value="openai">OpenAI</option>
                        <option value="anthropic">Anthropic</option>
                        <option value="ollama">Ollama</option>
                        <option value="litellm">LiteLLM</option>
                      </select>
                    </div>
                    <div>
                      <label className="block text-sm font-medium mb-2">Model</label>
                      <input type="text" placeholder="gpt-4o-mini" className="w-full px-3 py-2 rounded-lg border bg-background" />
                    </div>
                  </div>
                  <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                    <div>
                      <div className="font-medium">Use same as Chat Model</div>
                      <div className="text-xs text-muted-foreground">Use chat model for utility tasks</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                </div>
              )}
              {agentSubTab === 'browser' && (
                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Browser Model</h3>
                    <p className="text-sm text-muted-foreground mb-4">Model used for web browsing and scraping tasks</p>
                  </div>
                  <div className="grid gap-4 md:grid-cols-2">
                    <div>
                      <label className="block text-sm font-medium mb-2">Provider</label>
                      <select className="w-full px-3 py-2 rounded-lg border bg-background">
                        <option value="openai">OpenAI</option>
                        <option value="anthropic">Anthropic</option>
                        <option value="ollama">Ollama</option>
                        <option value="litellm">LiteLLM</option>
                      </select>
                    </div>
                    <div>
                      <label className="block text-sm font-medium mb-2">Model</label>
                      <input type="text" placeholder="gpt-4o" className="w-full px-3 py-2 rounded-lg border bg-background" />
                    </div>
                  </div>
                  <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                    <div>
                      <div className="font-medium">Use same as Chat Model</div>
                      <div className="text-xs text-muted-foreground">Use chat model for browser tasks</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                </div>
              )}
              {agentSubTab === 'memory' && (
                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Memory Settings</h3>
                    <p className="text-sm text-muted-foreground mb-4">Configure how the agent remembers context</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Context Window</label>
                    <input type="number" defaultValue="4096" className="w-full px-3 py-2 rounded-lg border bg-background" />
                    <p className="text-xs text-muted-foreground mt-1">Maximum tokens to keep in context</p>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Memory Type</label>
                    <select className="w-full px-3 py-2 rounded-lg border bg-background">
                      <option value="full">Full History</option>
                      <option value="summary">Summary Only</option>
                      <option value="semantic">Semantic (Vector)</option>
                      <option value="hybrid">Hybrid</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Embedding Provider</label>
                    <select className="w-full px-3 py-2 rounded-lg border bg-background">
                      <option value="local">Local (Ollama)</option>
                      <option value="openai">OpenAI</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                    <div>
                      <div className="font-medium">Enable Long-term Memory</div>
                      <div className="text-xs text-muted-foreground">Store memories between sessions</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* External Tab with Sub-tabs */}
        {activeTab === 'external' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-semibold">External Services</h2>
              <p className="text-muted-foreground">Configure API keys, secrets, and external services</p>
            </div>
            
            {/* Sub-tabs for External */}
            <div className="flex gap-1 border-b flex-wrap">
              <button
                onClick={() => setExternalSubTab('apikeys')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  externalSubTab === 'apikeys'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                API Keys
              </button>
              <button
                onClick={() => setExternalSubTab('litellm')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  externalSubTab === 'litellm'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                LiteLLM
              </button>
              <button
                onClick={() => setExternalSubTab('secrets')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  externalSubTab === 'secrets'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                Secrets
              </button>
              <button
                onClick={() => setExternalSubTab('auth')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  externalSubTab === 'auth'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                Authentication
              </button>
              <button
                onClick={() => setExternalSubTab('externalapi')}
                className={`px-4 py-2 rounded-t transition-colors ${
                  externalSubTab === 'externalapi'
                    ? 'bg-primary/10 text-primary border-b-2 border-primary'
                    : 'hover:bg-muted'
                }`}
              >
                External API
              </button>
            </div>
            
            {/* Sub-tab content */}
            <div className="border rounded-lg p-4">
              {externalSubTab === 'apikeys' && <ApiKeysManager />}
              {externalSubTab === 'litellm' && <LiteLlmSettings />}
              {externalSubTab === 'secrets' && <SecretsManager />}
              {externalSubTab === 'auth' && (
                <div className="text-muted-foreground">Authentication settings - Configure HMAC and TOTP in Security tab</div>
              )}
              {externalSubTab === 'externalapi' && (
                <div className="text-muted-foreground">External API configuration</div>
              )}
            </div>
          </div>
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

        {activeTab === 'runtime' && <RuntimeSettings />}

        {activeTab === 'mcp' && (
          <McpManager />
        )}

        {activeTab === 'notifications' && (
          <NotificationsSettings />
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

        {/* Developer Tab */}
        {activeTab === 'developer' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-semibold">Developer Settings</h2>
              <p className="text-muted-foreground">Configure developer options and debugging</p>
            </div>
            
            {/* Workspace Settings */}
            <div className="border rounded-lg p-6 space-y-6">
              <h3 className="font-semibold flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                Workspace
              </h3>
              <div>
                <label className="block text-sm font-medium mb-2">Working Directory</label>
                <input type="text" placeholder="C:\projects" className="w-full px-3 py-2 rounded-lg border bg-background" defaultValue="." />
                <p className="text-xs text-muted-foreground mt-1">The default directory for file operations</p>
              </div>
            </div>

            {/* Logging Settings */}
            <div className="border rounded-lg p-6 space-y-6">
              <h3 className="font-semibold flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                Logging
              </h3>
              <div>
                <label className="block text-sm font-medium mb-2">Log Level</label>
                <select className="w-full px-3 py-2 rounded-lg border bg-background">
                  <option value="trace">Trace</option>
                  <option value="debug">Debug</option>
                  <option value="info" selected>Info</option>
                  <option value="warn">Warning</option>
                  <option value="error">Error</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">Log Format</label>
                <select className="w-full px-3 py-2 rounded-lg border bg-background">
                  <option value="text">Text</option>
                  <option value="json" selected>JSON</option>
                </select>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">Enable Debug Mode</div>
                  <div className="text-xs text-muted-foreground">Verbose logging for debugging</div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" />
                  <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                </label>
              </div>
            </div>

            {/* Advanced Settings */}
            <div className="border rounded-lg p-6 space-y-6">
              <h3 className="font-semibold flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
                Advanced
              </h3>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">Enable LLM Streaming</div>
                  <div className="text-xs text-muted-foreground">Stream model responses in real-time</div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" defaultChecked />
                  <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                </label>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">Enable Execution Streaming</div>
                  <div className="text-xs text-muted-foreground">Stream tool execution output</div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" defaultChecked />
                  <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                </label>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">Enable Subagent Pool</div>
                  <div className="text-xs text-muted-foreground">Parallel task execution</div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input 
                    type="checkbox" 
                    className="sr-only peer" 
                    checked={enableSubagents}
                    onChange={async (e) => {
                      const value = e.target.checked;
                      setEnableSubagents(value);
                      // Save to API (database) for persistence, localStorage as fallback
                      try {
                        await setSetting('agent.enable_subagents', String(value), false);
                        // Also save to localStorage for Chat.tsx fallback
                        const saved = localStorage.getItem('apex-task-config');
                        const config = saved ? JSON.parse(saved) : {};
                        config.enableSubagents = value;
                        localStorage.setItem('apex-task-config', JSON.stringify(config));
                      } catch (err) {
                        console.warn('Failed to save subagent setting to API:', err);
                      }
                    }}
                  />
                  <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                </label>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">Enable TIR (Tool-Integrated Reasoning)</div>
                  <div className="text-xs text-muted-foreground">Interleave reasoning with tool execution</div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input 
                    type="checkbox" 
                    className="sr-only peer" 
                    checked={enableTir}
                    onChange={async (e) => {
                      const value = e.target.checked;
                      setEnableTir(value);
                      // Save to API (database) for persistence, localStorage as fallback
                      try {
                        await setSetting('agent.use_tir', String(value), false);
                        // Also save to localStorage for Chat.tsx fallback
                        const saved = localStorage.getItem('apex-task-config');
                        const config = saved ? JSON.parse(saved) : {};
                        config.useTir = value;
                        localStorage.setItem('apex-task-config', JSON.stringify(config));
                      } catch (err) {
                        console.warn('Failed to save TIR setting to API:', err);
                      }
                    }}
                  />
                  <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                </label>
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">Max Concurrent Tasks</label>
                <input type="number" defaultValue="4" min="1" max="16" className="w-full px-3 py-2 rounded-lg border bg-background" />
              </div>
            </div>

            {/* Danger Zone */}
            <div className="border border-red-500/30 rounded-lg p-6 space-y-4">
              <h3 className="font-semibold flex items-center gap-2 text-red-500">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg>
                Danger Zone
              </h3>
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-medium">Disable Authentication</div>
                  <div className="text-xs text-muted-foreground">Allow unauthenticated requests (DANGER)</div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" className="sr-only peer" />
                  <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-red-500 after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                </label>
              </div>
              <button className="px-4 py-2 bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors">
                Clear All Cache
              </button>
            </div>
          </div>
        )}

        {/* Backup Tab */}
        {activeTab === 'backup' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-semibold">Backup</h2>
              <p className="text-muted-foreground">Backup and restore APEX configuration</p>
            </div>
            <div className="border rounded-lg p-4 space-y-4">
              <div>
                <h3 className="font-semibold mb-2">Create Backup</h3>
                <p className="text-sm text-muted-foreground mb-3">
                  Download a backup of your APEX configuration, skills, and settings.
                </p>
                <button className="px-4 py-2 bg-primary text-primary-foreground rounded hover:opacity-90">
                  Download Backup
                </button>
              </div>
              <div className="border-t pt-4">
                <h3 className="font-semibold mb-2">Restore Backup</h3>
                <p className="text-sm text-muted-foreground mb-3">
                  Restore APEX from a previously saved backup file.
                </p>
                <input type="file" accept=".json" className="block w-full text-sm text-muted-foreground file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:bg-primary file:text-primary-foreground" />
              </div>
            </div>
          </div>
        )}

        {/* Skills Tab */}
        {activeTab === 'skills' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-semibold">Skills</h2>
              <p className="text-muted-foreground">Manage and configure agent skills</p>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-muted-foreground mb-4">
                Skills extend the agent's capabilities. Enable or disable skills based on your needs.
              </div>
              <div className="space-y-3">
                <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                  <div>
                    <div className="font-medium">Shell Execute</div>
                    <div className="text-xs text-muted-foreground">Execute shell commands (T3 - Requires TOTP)</div>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" className="sr-only peer" defaultChecked />
                    <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                  </label>
                </div>
                <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                  <div>
                    <div className="font-medium">Code Generation</div>
                    <div className="text-xs text-muted-foreground">Generate code from descriptions (T2)</div>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" className="sr-only peer" defaultChecked />
                    <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                  </label>
                </div>
                <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                  <div>
                    <div className="font-medium">Code Review</div>
                    <div className="text-xs text-muted-foreground">Review code for issues (T0)</div>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" className="sr-only peer" defaultChecked />
                    <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                  </label>
                </div>
                <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
                  <div>
                    <div className="font-medium">Git Operations</div>
                    <div className="text-xs text-muted-foreground">Commit, push, branch operations (T1/T2)</div>
                  </div>
                  <label className="relative inline-flex items-center cursor-pointer">
                    <input type="checkbox" className="sr-only peer" defaultChecked />
                    <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                  </label>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Speech Tab */}
        {activeTab === 'speech' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-semibold">Speech</h2>
              <p className="text-muted-foreground">Configure voice input and output settings</p>
            </div>
            <div className="border rounded-lg p-6 space-y-6">
              {/* Speech-to-Text Settings */}
              <div>
                <h3 className="font-semibold mb-4 flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path><line x1="12" y1="19" x2="12" y2="23"></line><line x1="8" y1="23" x2="16" y2="23"></line></svg>
                  Speech-to-Text (STT)
                </h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">Provider</label>
                    <select className="w-full px-3 py-2 rounded-lg border bg-background">
                      <option value="">Select provider...</option>
                      <option value="openai">OpenAI Whisper</option>
                      <option value="local">Local (Vosk)</option>
                      <option value="browser">Browser Native</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Language</label>
                    <select className="w-full px-3 py-2 rounded-lg border bg-background">
                      <option value="en">English</option>
                      <option value="auto">Auto-detect</option>
                    </select>
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">Enable Voice Input</div>
                      <div className="text-xs text-muted-foreground">Use microphone for voice input</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                </div>
              </div>

              {/* Text-to-Speech Settings */}
              <div className="border-t pt-6">
                <h3 className="font-semibold mb-4 flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon><path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path></svg>
                  Text-to-Speech (TTS)
                </h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">Provider</label>
                    <select className="w-full px-3 py-2 rounded-lg border bg-background">
                      <option value="">Select provider...</option>
                      <option value="openai">OpenAI TTS</option>
                      <option value="coqui">Coqui TTS</option>
                      <option value="browser">Browser Native</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Voice</label>
                    <select className="w-full px-3 py-2 rounded-lg border bg-background">
                      <option value="alloy">Alloy</option>
                      <option value="echo">Echo</option>
                      <option value="fable">Fable</option>
                      <option value="onyx">Onyx</option>
                      <option value="nova">Nova</option>
                      <option value="shimmer">Shimmer</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Speed</label>
                    <input type="range" min="0.5" max="2" step="0.1" defaultValue="1" className="w-full" />
                    <div className="flex justify-between text-xs text-muted-foreground">
                      <span>0.5x</span>
                      <span>1x</span>
                      <span>2x</span>
                    </div>
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">Enable Voice Output</div>
                      <div className="text-xs text-muted-foreground">Speak responses aloud</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* A2A Tab */}
        {activeTab === 'a2a' && (
          <div className="space-y-6">
            <div>
              <h2 className="text-2xl font-semibold">A2A (Agent-to-Agent)</h2>
              <p className="text-muted-foreground">Configure agent communication and collaboration</p>
            </div>
            <div className="border rounded-lg p-6 space-y-6">
              <div>
                <h3 className="font-semibold mb-4 flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
                  Agent Discovery
                </h3>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">Enable Agent Discovery</div>
                      <div className="text-xs text-muted-foreground">Allow other agents to discover this agent</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Agent ID</label>
                    <input type="text" placeholder="apex-agent-001" className="w-full px-3 py-2 rounded-lg border bg-background" />
                  </div>
                </div>
              </div>

              <div className="border-t pt-6">
                <h3 className="font-semibold mb-4 flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><line x1="2" y1="12" x2="22" y2="12"></line><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path></svg>
                  Network
                </h3>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium mb-2">Protocol</label>
                    <select className="w-full px-3 py-2 rounded-lg border bg-background">
                      <option value="a2a">A2A Protocol</option>
                      <option value="mcp">MCP</option>
                      <option value="stdio">STDIO</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Port</label>
                    <input type="number" defaultValue="8080" className="w-full px-3 py-2 rounded-lg border bg-background" />
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">Accept Tasks from Agents</div>
                      <div className="text-xs text-muted-foreground">Allow other agents to submit tasks</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                </div>
              </div>

              <div className="border-t pt-6">
                <h3 className="font-semibold mb-4 flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect><path d="M7 11V7a5 5 0 0 1 10 0v4"></path></svg>
                  Authentication
                </h3>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-medium">Require Authentication</div>
                      <div className="text-xs text-muted-foreground">Require HMAC signature for agent requests</div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input type="checkbox" className="sr-only peer" defaultChecked />
                      <div className="w-11 h-6 bg-muted rounded-full peer peer-checked:bg-[#4248f1] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                  </div>
                  <div>
                    <label className="block text-sm font-medium mb-2">Allowed Agents</label>
                    <textarea placeholder="Enter agent IDs (one per line)" rows={3} className="w-full px-3 py-2 rounded-lg border bg-background" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// Notifications Settings Component
interface ExternalNotificationConfig {
  discord_webhook_url?: string;
  telegram_bot_token?: string;
  telegram_chat_id?: string;
  enabled: boolean;
}

function NotificationsSettings() {
  const [config, setConfig] = useState<ExternalNotificationConfig>({
    discord_webhook_url: '',
    telegram_bot_token: '',
    telegram_chat_id: '',
    enabled: false,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/notifications/external');
      const data = await res.json();
      setConfig({
        discord_webhook_url: data.discord_webhook_url || '',
        telegram_bot_token: data.telegram_bot_token || '',
        telegram_chat_id: data.telegram_chat_id || '',
        enabled: data.enabled || false,
      });
    } catch (e) {
      console.error('Failed to load notification config:', e);
    }
    setLoading(false);
  };

  const saveConfig = async () => {
    setSaving(true);
    setMessage(null);
    try {
      await apiPost('/api/v1/notifications/external', {
        discord_webhook_url: config.discord_webhook_url || null,
        telegram_bot_token: config.telegram_bot_token || null,
        telegram_chat_id: config.telegram_chat_id || null,
        enabled: config.enabled,
      });
      setMessage({ type: 'success', text: 'Settings saved successfully!' });
    } catch (e) {
      setMessage({ type: 'error', text: 'Failed to save settings' });
    }
    setSaving(false);
  };

  const testNotification = async () => {
    setTesting(true);
    setMessage(null);
    try {
      const res = await apiPost('/api/v1/notifications/external/test', {});
      const data = await res.json();
      if (res.ok) {
        setMessage({ type: 'success', text: 'Test notification sent!' });
      } else {
        setMessage({ type: 'error', text: data.message || 'Failed to send test' });
      }
    } catch (e) {
      setMessage({ type: 'error', text: 'Failed to send test notification' });
    }
    setTesting(false);
  };

  if (loading) {
    return <div className="text-muted-foreground">Loading...</div>;
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-semibold">External Notifications</h2>
        <p className="text-muted-foreground mt-1">
          Configure Discord or Telegram notifications for task completion
        </p>
      </div>

      {message && (
        <div className={`p-3 rounded ${message.type === 'success' ? 'bg-green-50 text-green-800' : 'bg-red-50 text-red-800'}`}>
          {message.text}
        </div>
      )}

      <div className="border rounded-lg p-4 space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="font-medium">Enable External Notifications</h3>
            <p className="text-sm text-muted-foreground">Send notifications when tasks complete or fail</p>
          </div>
          <input
            type="checkbox"
            checked={config.enabled}
            onChange={(e) => setConfig({ ...config, enabled: e.target.checked })}
            className="h-5 w-5"
          />
        </div>

        <div className="border-t pt-4">
          <h3 className="font-medium mb-3">Discord Webhook</h3>
          <input
            type="text"
            placeholder="https://discord.com/api/webhooks/..."
            value={config.discord_webhook_url}
            onChange={(e) => setConfig({ ...config, discord_webhook_url: e.target.value })}
            className="w-full px-3 py-2 border rounded-md bg-background"
          />
        </div>

        <div className="border-t pt-4">
          <h3 className="font-medium mb-3">Telegram Bot</h3>
          <div className="space-y-3">
            <div>
              <label className="text-sm text-muted-foreground">Bot Token</label>
              <input
                type="password"
                placeholder="1234567890:ABCdefGHIjklMNOpqrsTUVwxyz"
                value={config.telegram_bot_token}
                onChange={(e) => setConfig({ ...config, telegram_bot_token: e.target.value })}
                className="w-full px-3 py-2 border rounded-md bg-background mt-1"
              />
            </div>
            <div>
              <label className="text-sm text-muted-foreground">Chat ID</label>
              <input
                type="text"
                placeholder="123456789"
                value={config.telegram_chat_id}
                onChange={(e) => setConfig({ ...config, telegram_chat_id: e.target.value })}
                className="w-full px-3 py-2 border rounded-md bg-background mt-1"
              />
            </div>
          </div>
        </div>

        <div className="border-t pt-4 flex gap-3">
          <button
            onClick={saveConfig}
            disabled={saving}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Settings'}
          </button>
          <button
            onClick={testNotification}
            disabled={testing || !config.enabled}
            className="px-4 py-2 border rounded-md hover:bg-muted disabled:opacity-50"
          >
            {testing ? 'Sending...' : 'Send Test'}
          </button>
        </div>
      </div>
    </div>
  );
}
