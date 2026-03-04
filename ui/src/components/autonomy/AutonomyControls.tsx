import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface HeartbeatConfig {
  enabled: boolean;
  interval_minutes: number;
  jitter_percent: number;
  cooldown_seconds: number;
  max_actions_per_wake: number;
  require_approval_t1_plus: boolean;
}

interface AutonomyStats {
  wake_count: number;
  last_wake: string | null;
  actions_performed: number;
  autonomous_actions: number;
}

export function AutonomyControls() {
  const [config, setConfig] = useState<HeartbeatConfig | null>(null);
  const [stats, setStats] = useState<AutonomyStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
    loadStats();
  }, []);

  const loadConfig = async () => {
    try {
      const res = await apiGet('/api/v1/heartbeat/config');
      if (res.ok) {
        const data = await res.json();
        setConfig(data);
      }
    } catch (err) {
      console.error('Failed to load config:', err);
    }
  };

  const loadStats = async () => {
    try {
      const res = await apiGet('/api/v1/heartbeat/stats');
      if (res.ok) {
        const data = await res.json();
        setStats(data);
      }
    } catch (err) {
      console.error('Failed to load stats:', err);
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    if (!config) return;
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      const res = await apiPost('/api/v1/heartbeat/config', config);
      if (res.ok) {
        setSuccess('Configuration saved successfully');
        setTimeout(() => setSuccess(null), 3000);
      } else {
        setError('Failed to save configuration');
      }
    } catch (err) {
      setError('Failed to save configuration');
    } finally {
      setSaving(false);
    }
  };

  const triggerWake = async () => {
    setError(null);
    try {
      const res = await apiPost('/api/v1/heartbeat/trigger', {});
      if (res.ok) {
        setSuccess('Wake cycle triggered');
        setTimeout(() => setSuccess(null), 3000);
        loadStats();
      } else {
        setError('Failed to trigger wake');
      }
    } catch (err) {
      setError('Failed to trigger wake');
    }
  };

  const toggleHeartbeat = async () => {
    if (!config) return;
    setError(null);
    try {
      const res = await apiPost('/api/v1/heartbeat/toggle', { enabled: !config.enabled });
      if (res.ok) {
        setConfig({ ...config, enabled: !config.enabled });
        setSuccess(`Heartbeat ${config.enabled ? 'disabled' : 'enabled'}`);
        setTimeout(() => setSuccess(null), 3000);
      } else {
        setError('Failed to toggle heartbeat');
      }
    } catch (err) {
      setError('Failed to toggle heartbeat');
    }
  };

  if (loading || !config) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-4">
      <div className="max-w-2xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold">Autonomy Controls</h2>
            <p className="text-sm text-muted-foreground">
              Configure heartbeat daemon and autonomous behavior
            </p>
          </div>
          <div className="flex items-center gap-2">
            <span className={`text-sm ${config.enabled ? 'text-green-500' : 'text-muted-foreground'}`}>
              {config.enabled ? 'Active' : 'Disabled'}
            </span>
            <button
              onClick={toggleHeartbeat}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                config.enabled ? 'bg-green-500' : 'bg-muted'
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  config.enabled ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>
        </div>

        {error && (
          <div className="p-3 bg-red-500/20 text-red-500 rounded-lg">
            {error}
          </div>
        )}

        {success && (
          <div className="p-3 bg-green-500/20 text-green-500 rounded-lg">
            {success}
          </div>
        )}

        {stats && (
          <div className="grid grid-cols-4 gap-4">
            <div className="border rounded-lg p-4 text-center">
              <div className="text-2xl font-bold">{stats.wake_count}</div>
              <div className="text-xs text-muted-foreground">Total Wakes</div>
            </div>
            <div className="border rounded-lg p-4 text-center">
              <div className="text-2xl font-bold">{stats.actions_performed}</div>
              <div className="text-xs text-muted-foreground">Actions</div>
            </div>
            <div className="border rounded-lg p-4 text-center">
              <div className="text-2xl font-bold">{stats.autonomous_actions}</div>
              <div className="text-xs text-muted-foreground">Autonomous</div>
            </div>
            <div className="border rounded-lg p-4 text-center">
              <div className="text-2xl font-bold">
                {stats.last_wake ? new Date(stats.last_wake).toLocaleTimeString() : 'Never'}
              </div>
              <div className="text-xs text-muted-foreground">Last Wake</div>
            </div>
          </div>
        )}

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Heartbeat Configuration</h3>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1">
                Wake Interval (minutes)
              </label>
              <input
                type="number"
                value={config.interval_minutes}
                onChange={(e) => setConfig({ ...config, interval_minutes: parseInt(e.target.value) || 60 })}
                min={1}
                max={1440}
                className="w-full px-3 py-2 rounded-lg border bg-background"
              />
              <p className="text-xs text-muted-foreground mt-1">
                How often the agent wakes to check for tasks
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">
                Jitter (%)
              </label>
              <input
                type="number"
                value={config.jitter_percent}
                onChange={(e) => setConfig({ ...config, jitter_percent: parseInt(e.target.value) || 10 })}
                min={0}
                max={50}
                className="w-full px-3 py-2 rounded-lg border bg-background"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Random variation in wake time to avoid predictable patterns
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">
                Cooldown (seconds)
              </label>
              <input
                type="number"
                value={config.cooldown_seconds}
                onChange={(e) => setConfig({ ...config, cooldown_seconds: parseInt(e.target.value) || 300 })}
                min={60}
                max={3600}
                className="w-full px-3 py-2 rounded-lg border bg-background"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Minimum time between wake cycles
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-1">
                Max Actions Per Wake
              </label>
              <input
                type="number"
                value={config.max_actions_per_wake}
                onChange={(e) => setConfig({ ...config, max_actions_per_wake: parseInt(e.target.value) || 3 })}
                min={1}
                max={10}
                className="w-full px-3 py-2 rounded-lg border bg-background"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Maximum autonomous actions per wake cycle
              </p>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <label className="block text-sm font-medium mb-1">
                  Require Approval for T1+
                </label>
                <p className="text-xs text-muted-foreground">
                  Require user confirmation for T1 and above actions
                </p>
              </div>
              <button
                onClick={() => setConfig({ ...config, require_approval_t1_plus: !config.require_approval_t1_plus })}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                  config.require_approval_t1_plus ? 'bg-primary' : 'bg-muted'
                }`}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    config.require_approval_t1_plus ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>
          </div>
        </div>

        <div className="flex gap-2">
          <button
            onClick={saveConfig}
            disabled={saving}
            className="flex-1 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
          <button
            onClick={triggerWake}
            className="px-4 py-2 rounded-lg border hover:bg-muted"
          >
            Trigger Wake
          </button>
        </div>
      </div>
    </div>
  );
}
