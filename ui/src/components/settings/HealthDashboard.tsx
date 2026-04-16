import { useState, useEffect, useCallback } from 'react';
import { apiGet, apiPost } from '../../lib/api';

// ============ Types ============

export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown' | 'checking';

export interface ComponentHealth {
  component: string;
  status: HealthStatus;
  last_check: string | null;
  last_success: string | null;
  last_failure: string | null;
  response_time_ms: number | null;
  error_message: string | null;
  consecutive_failures: number;
  check_interval_secs: number;
  metadata: Record<string, string>;
}

export interface HealthSummary {
  overall_status: HealthStatus;
  healthy_count: number;
  degraded_count: number;
  unhealthy_count: number;
  unknown_count: number;
  components: ComponentHealth[];
  last_updated: string;
}

export interface HealthHistoryEntry {
  id: string;
  component: string;
  status: HealthStatus;
  response_time_ms: number | null;
  error_message: string | null;
  timestamp: string;
}

export interface RunHealthCheckResponse {
  component: string;
  status: HealthStatus;
  response_time_ms: number | null;
  error_message: string | null;
  timestamp: string;
}

// ============ Component ============

export function HealthDashboard() {
  const [summary, setSummary] = useState<HealthSummary | null>(null);
  const [history, setHistory] = useState<HealthHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [activeView, setActiveView] = useState<'overview' | 'history' | 'details'>('overview');
  const [selectedComponent, setSelectedComponent] = useState<string | null>(null);
  const [runningChecks, setRunningChecks] = useState<Set<string>>(new Set());

  const loadSummary = useCallback(async () => {
    try {
      const res = await apiGet('/api/v1/health/summary');
      if (res.ok) {
        const data = await res.json();
        setSummary(data);
      }
    } catch (err) {
      console.error('Failed to load health summary:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadHistory = useCallback(async (component?: string) => {
    setHistoryLoading(true);
    try {
      const url = component 
        ? `/api/v1/health/history?component=${component}&limit=100`
        : '/api/v1/health/history?limit=100';
      const res = await apiGet(url);
      if (res.ok) {
        const data = await res.json();
        setHistory(Array.isArray(data) ? data : []);
      }
    } catch (err) {
      console.error('Failed to load health history:', err);
    } finally {
      setHistoryLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSummary();
    const interval = setInterval(loadSummary, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, [loadSummary]);

  useEffect(() => {
    if (activeView === 'history') {
      loadHistory(selectedComponent || undefined);
    }
  }, [activeView, selectedComponent, loadHistory]);

  const runHealthCheck = async (component: string) => {
    setRunningChecks(prev => new Set(prev).add(component));
    try {
      await apiPost(`/api/v1/health/checks/${component}/run`, {});
      loadSummary();
    } catch (err) {
      console.error('Failed to run health check:', err);
    } finally {
      setRunningChecks(prev => {
        const next = new Set(prev);
        next.delete(component);
        return next;
      });
    }
  };

  const getStatusColor = (status: HealthStatus) => {
    switch (status) {
      case 'healthy': return 'bg-green-500/20 text-green-400 border-green-500/50';
      case 'degraded': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/50';
      case 'unhealthy': return 'bg-red-500/20 text-red-400 border-red-500/50';
      case 'checking': return 'bg-blue-500/20 text-blue-400 border-blue-500/50';
      default: return 'bg-gray-500/20 text-gray-400 border-gray-500/50';
    }
  };

  const getStatusIcon = (status: HealthStatus) => {
    switch (status) {
      case 'healthy': return '✓';
      case 'degraded': return '⚠';
      case 'unhealthy': return '✗';
      case 'checking': return '⟳';
      default: return '?';
    }
  };

  const formatTimestamp = (ts: string | null) => {
    if (!ts) return 'Never';
    const date = new Date(ts);
    return date.toLocaleTimeString();
  };

  const formatUptime = (lastSuccess: string | null) => {
    if (!lastSuccess) return 'N/A';
    const diff = Date.now() - new Date(lastSuccess).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'Just now';
    if (mins < 60) return `${mins}m`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ${mins % 60}m`;
    return `${Math.floor(hours / 24)}d ${hours % 24}h`;
  };

  const componentNames: Record<string, string> = {
    llm: 'LLM Server',
    database: 'Database',
    skill_pool: 'Skill Pool',
    memory_indexer: 'Memory Indexer',
    vm_pool: 'VM Pool',
    message_bus: 'Message Bus',
    websocket: 'WebSocket',
    skill_worker: 'Skill Worker',
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading health data...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold">Health Dashboard</h2>
          <p className="text-[var(--color-text-muted)]">
            Monitor system component health and status
          </p>
        </div>
        <button
          onClick={loadSummary}
          className="px-4 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors text-sm"
        >
          Refresh
        </button>
      </div>

      {/* Overall Status Card */}
      {summary && (
        <div className={`border rounded-lg p-6 ${getStatusColor(summary.overall_status)}`}>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="text-4xl">{getStatusIcon(summary.overall_status)}</div>
              <div>
                <div className="text-2xl font-bold capitalize">{summary.overall_status}</div>
                <div className="text-sm opacity-80">Overall System Health</div>
              </div>
            </div>
            <div className="text-right text-sm">
              <div className="opacity-80">Last updated: {formatTimestamp(summary.last_updated)}</div>
            </div>
          </div>
        </div>
      )}

      {/* Stats Grid */}
      {summary && (
        <div className="grid grid-cols-4 gap-4">
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-green-400">{summary.healthy_count}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Healthy</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-yellow-400">{summary.degraded_count}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Degraded</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-red-400">{summary.unhealthy_count}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Unhealthy</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-gray-400">{summary.unknown_count}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Unknown</div>
          </div>
        </div>
      )}

      {/* View Tabs */}
      <div className="flex flex-wrap gap-4 border-b">
        <button
          onClick={() => setActiveView('overview')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'overview'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          Component Overview
        </button>
        <button
          onClick={() => setActiveView('history')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'history'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          History
        </button>
      </div>

      {/* Component Overview */}
      {activeView === 'overview' && summary && (
        <div className="space-y-3">
          {summary.components.map((comp) => (
            <div
              key={comp.component}
              className="border rounded-lg p-4 bg-[var(--color-panel)] hover:border-[#00d4ff]/50 transition-colors"
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className={`w-3 h-3 rounded-full ${getStatusColor(comp.status).split(' ')[0]}`} />
                  <div>
                    <h3 className="font-medium">{componentNames[comp.component] || comp.component}</h3>
                    <p className="text-sm text-[var(--color-text-muted)]">
                      Interval: {comp.check_interval_secs}s
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-4">
                  <div className="text-right">
                    <div className={`px-2 py-1 rounded text-sm font-medium capitalize ${getStatusColor(comp.status)}`}>
                      {comp.status}
                    </div>
                    {comp.response_time_ms && (
                      <div className="text-xs text-[var(--color-text-muted)] mt-1">
                        {comp.response_time_ms}ms
                      </div>
                    )}
                  </div>
                  <button
                    onClick={() => runHealthCheck(comp.component)}
                    disabled={runningChecks.has(comp.component)}
                    className="px-3 py-1.5 bg-[#00d4ff]/20 text-[#00d4ff] rounded hover:bg-[#00d4ff]/30 disabled:opacity-50 transition-colors text-sm"
                  >
                    {runningChecks.has(comp.component) ? 'Running...' : 'Check'}
                  </button>
                </div>
              </div>
              
              {comp.error_message && (
                <div className="mt-3 p-2 rounded bg-red-500/10 text-red-400 text-sm">
                  {comp.error_message}
                </div>
              )}
              
              <div className="mt-3 flex gap-6 text-xs text-[var(--color-text-muted)]">
                <span>Last check: {formatTimestamp(comp.last_check)}</span>
                <span>Uptime: {formatUptime(comp.last_success)}</span>
                {comp.consecutive_failures > 0 && (
                  <span className="text-red-400">Failures: {comp.consecutive_failures}</span>
                )}
              </div>
            </div>
          ))}
          
          {summary.components.length === 0 && (
            <div className="border rounded-lg p-8 text-center text-[var(--color-text-muted)]">
              No components registered. Health checks will appear as they run.
            </div>
          )}
        </div>
      )}

      {/* History View */}
      {activeView === 'history' && (
        <div className="space-y-4">
          {/* Filter */}
          <div className="flex items-center gap-4">
            <select
              value={selectedComponent || ''}
              onChange={(e) => setSelectedComponent(e.target.value || null)}
              className="px-3 py-2 rounded border bg-[var(--color-panel)] border-[var(--color-border)]"
            >
              <option value="">All Components</option>
              {summary?.components.map((comp) => (
                <option key={comp.component} value={comp.component}>
                  {componentNames[comp.component] || comp.component}
                </option>
              ))}
            </select>
            <button
              onClick={() => loadHistory(selectedComponent || undefined)}
              disabled={historyLoading}
              className="px-3 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors text-sm"
            >
              {historyLoading ? 'Loading...' : 'Refresh'}
            </button>
          </div>

          {/* History Table */}
          <div className="border rounded-lg bg-[var(--color-panel)] overflow-hidden">
            <table className="w-full">
              <thead className="bg-[var(--color-muted)]">
                <tr>
                  <th className="px-4 py-3 text-left text-sm font-medium">Time</th>
                  <th className="px-4 py-3 text-left text-sm font-medium">Component</th>
                  <th className="px-4 py-3 text-left text-sm font-medium">Status</th>
                  <th className="px-4 py-3 text-left text-sm font-medium">Response Time</th>
                  <th className="px-4 py-3 text-left text-sm font-medium">Error</th>
                </tr>
              </thead>
              <tbody className="divide-y">
                {historyLoading ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-8 text-center text-[var(--color-text-muted)]">
                      Loading history...
                    </td>
                  </tr>
                ) : history.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="px-4 py-8 text-center text-[var(--color-text-muted)]">
                      No history entries found
                    </td>
                  </tr>
                ) : (
                  history.map((entry) => (
                    <tr key={entry.id} className="hover:bg-[var(--color-muted)]/30">
                      <td className="px-4 py-3 text-sm">
                        {new Date(entry.timestamp).toLocaleString()}
                      </td>
                      <td className="px-4 py-3 text-sm">
                        {componentNames[entry.component] || entry.component}
                      </td>
                      <td className="px-4 py-3">
                        <span className={`px-2 py-1 rounded text-xs font-medium capitalize ${getStatusColor(entry.status)}`}>
                          {entry.status}
                        </span>
                      </td>
                      <td className="px-4 py-3 text-sm">
                        {entry.response_time_ms ? `${entry.response_time_ms}ms` : '-'}
                      </td>
                      <td className="px-4 py-3 text-sm text-red-400">
                        {entry.error_message || '-'}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
