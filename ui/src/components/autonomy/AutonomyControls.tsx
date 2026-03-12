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
          Loading...
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-2xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <polyline points="12 6 12 12 16 14"></polyline>
              </svg>
            </div>
            <div>
              <h2 className="text-2xl font-bold">Autonomy Controls</h2>
              <p className="text-sm text-[var(--color-text-muted)]">
                Configure heartbeat daemon and autonomous behavior
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <span className={`text-sm font-medium ${config.enabled ? 'text-green-500' : 'text-[var(--color-text-muted)]'}`}>
              {config.enabled ? 'Active' : 'Disabled'}
            </span>
            <button
              onClick={toggleHeartbeat}
              className={`relative inline-flex h-7 w-12 items-center rounded-full transition-colors ${
                config.enabled ? 'bg-green-500' : 'bg-[var(--color-muted)]'
              }`}
            >
              <span
                className={`inline-block h-5 w-5 transform rounded-full bg-white shadow-md transition-transform ${
                  config.enabled ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>
        </div>

        {/* Notifications */}
        {error && (
          <div className="p-3 bg-red-500/10 text-red-500 rounded-lg border border-red-500/20 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="15" y1="9" x2="9" y2="15"></line>
              <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
            {error}
          </div>
        )}

        {success && (
          <div className="p-3 bg-green-500/10 text-green-500 rounded-lg border border-green-500/20 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
              <polyline points="22 4 12 14.01 9 11.01"></polyline>
            </svg>
            {success}
          </div>
        )}

        {/* Stats Grid */}
        {stats && (
          <div className="grid grid-cols-4 gap-4">
            <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
              <div className="text-2xl font-bold text-[#4248f1]">{stats.wake_count}</div>
              <div className="text-xs text-[var(--color-text-muted)]">Total Wakes</div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
              <div className="text-2xl font-bold">{stats.actions_performed}</div>
              <div className="text-xs text-[var(--color-text-muted)]">Actions</div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
              <div className="text-2xl font-bold text-green-500">{stats.autonomous_actions}</div>
              <div className="text-xs text-[var(--color-text-muted)]">Autonomous</div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
              <div className="text-2xl font-bold">
                {stats.last_wake ? new Date(stats.last_wake).toLocaleTimeString() : 'Never'}
              </div>
              <div className="text-xs text-[var(--color-text-muted)]">Last Wake</div>
            </div>
          </div>
        )}

        {/* Configuration Card */}
        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="3"></circle>
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
            </svg>
            Heartbeat Configuration
          </h3>
          
          <div className="space-y-5">
            <div>
              <label className="block text-sm font-medium mb-2">
                Wake Interval (minutes)
              </label>
              <input
                type="number"
                value={config.interval_minutes}
                onChange={(e) => setConfig({ ...config, interval_minutes: parseInt(e.target.value) || 60 })}
                min={1}
                max={1440}
                className="w-full px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
              />
              <p className="text-xs text-[var(--color-text-muted)] mt-1.5">
                How often the agent wakes to check for tasks
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Jitter (%)
              </label>
              <input
                type="number"
                value={config.jitter_percent}
                onChange={(e) => setConfig({ ...config, jitter_percent: parseInt(e.target.value) || 10 })}
                min={0}
                max={50}
                className="w-full px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
              />
              <p className="text-xs text-[var(--color-text-muted)] mt-1.5">
                Random variation in wake time to avoid predictable patterns
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Cooldown (seconds)
              </label>
              <input
                type="number"
                value={config.cooldown_seconds}
                onChange={(e) => setConfig({ ...config, cooldown_seconds: parseInt(e.target.value) || 300 })}
                min={60}
                max={3600}
                className="w-full px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
              />
              <p className="text-xs text-[var(--color-text-muted)] mt-1.5">
                Minimum time between wake cycles
              </p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">
                Max Actions Per Wake
              </label>
              <input
                type="number"
                value={config.max_actions_per_wake}
                onChange={(e) => setConfig({ ...config, max_actions_per_wake: parseInt(e.target.value) || 3 })}
                min={1}
                max={10}
                className="w-full px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
              />
              <p className="text-xs text-[var(--color-text-muted)] mt-1.5">
                Maximum autonomous actions per wake cycle
              </p>
            </div>

            <div className="flex items-center justify-between p-4 bg-[var(--color-muted)]/30 rounded-lg">
              <div>
                <label className="block text-sm font-medium mb-1">
                  Require Approval for T1+
                </label>
                <p className="text-xs text-[var(--color-text-muted)]">
                  Require user confirmation for T1 and above actions
                </p>
              </div>
              <button
                onClick={() => setConfig({ ...config, require_approval_t1_plus: !config.require_approval_t1_plus })}
                className={`relative inline-flex h-7 w-12 items-center rounded-full transition-colors ${
                  config.require_approval_t1_plus ? 'bg-[#4248f1]' : 'bg-[var(--color-muted)]'
                }`}
              >
                <span
                  className={`inline-block h-5 w-5 transform rounded-full bg-white shadow-md transition-transform ${
                    config.require_approval_t1_plus ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <button
            onClick={saveConfig}
            disabled={saving}
            className="flex-1 px-4 py-2.5 rounded-lg bg-[#4248f1] text-white hover:bg-[#353bc5] transition-colors disabled:opacity-50 flex items-center justify-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
              <polyline points="17 21 17 13 7 13 7 21"></polyline>
              <polyline points="7 3 7 8 15 8"></polyline>
            </svg>
            {saving ? 'Saving...' : 'Save Configuration'}
          </button>
          <button
            onClick={triggerWake}
            className="px-4 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-panel)] hover:bg-[var(--color-muted)] transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="5 3 19 12 5 21 5 3"></polygon>
            </svg>
            Trigger Wake
          </button>
        </div>
      </div>
    </div>
  );
}
