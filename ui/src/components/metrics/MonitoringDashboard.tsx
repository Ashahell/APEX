import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface SystemHealth {
  uptime_secs: number;
  requests_total: number;
  errors_total: number;
  error_rate: number;
  avg_response_time_ms: number;
  requests_by_endpoint: Record<string, number>;
  last_error: string | null;
}

interface CacheStats {
  total_entries: number;
  expired_entries: number;
  active_entries: number;
}

interface TelemetryEndpoint {
  count: number;
  avg_ms: number;
  min_ms: number;
  max_ms: number;
  p50_ms: number;
  p95_ms: number;
  p99_ms: number;
}

interface TelemetryError {
  requests: number;
  errors: number;
  error_rate_pct: number;
  error_types: Record<string, number>;
}

interface TelemetryData {
  endpoint_latencies: Record<string, TelemetryEndpoint>;
  endpoint_errors: Record<string, TelemetryError>;
}

interface Metrics {
  tasks: Record<string, number>;
  by_tier: Record<string, number>;
  by_status: Record<string, number>;
  total_cost_usd: number;
  tasks_completed: number;
  tasks_failed: number;
  telemetry?: TelemetryData;
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  
  if (days > 0) return `${days}d ${hours}h ${minutes}m`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

export function MonitoringDashboard() {
  const [health, setHealth] = useState< SystemHealth | null>(null);
  const [cache, setCache] = useState<CacheStats | null>(null);
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [activePanel, setActivePanel] = useState<'overview' | 'health' | 'cache' | 'tasks' | 'telemetry'>('overview');

  useEffect(() => {
    loadAllData();
    const interval = setInterval(loadAllData, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadAllData = async () => {
    try {
      const [healthRes, cacheRes, metricsRes] = await Promise.all([
        apiGet('/api/v1/system/health'),
        apiGet('/api/v1/system/cache'),
        apiGet('/api/v1/metrics'),
      ]);
      
      if (healthRes.ok) setHealth(await healthRes.json());
      if (cacheRes.ok) setCache(await cacheRes.json());
      if (metricsRes.ok) setMetrics(await metricsRes.json());
    } catch (err) {
      console.error('Failed to load monitoring data:', err);
    } finally {
      setLoading(false);
    }
  };

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
          Loading monitoring data...
        </div>
      </div>
    );
  }

  const totalTasks = metrics?.tasks?.total || 0;
  const completedTasks = metrics?.tasks_completed || 0;
  const failedTasks = metrics?.tasks_failed || 0;
  const successRate = totalTasks > 0 ? ((completedTasks / totalTasks) * 100).toFixed(1) : '0';

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M22 12h-4l-3 9L9 3l-3 9H2"></path>
              </svg>
            </div>
            <div>
              <h2 className="text-xl font-semibold">Monitoring Dashboard</h2>
              <p className="text-sm text-[var(--color-text-muted)]">System health, cache, and task metrics</p>
            </div>
          </div>
          <div className="flex gap-1 border border-[var(--color-border)] rounded-lg p-1 bg-[var(--color-panel)]">
            {(['overview', 'health', 'cache', 'tasks', 'telemetry'] as const).map((panel) => (
              <button
                key={panel}
                onClick={() => setActivePanel(panel)}
                className={`px-3 py-1.5 rounded text-sm transition-colors ${
                  activePanel === panel
                    ? 'bg-[#4248f1] text-white'
                    : 'hover:bg-[var(--color-muted)] text-[var(--color-text-muted)]'
                }`}
              >
                {panel.charAt(0).toUpperCase() + panel.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {/* Health Stats */}
        {(activePanel === 'overview' || activePanel === 'health') && (
          <div className="grid grid-cols-4 gap-4">
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Uptime</div>
              <div className="text-2xl font-bold mt-1">
                {formatUptime(health?.uptime_secs || 0)}
              </div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Total Requests</div>
              <div className="text-2xl font-bold mt-1">
                {(health?.requests_total || 0).toLocaleString()}
              </div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Error Rate</div>
              <div className={`text-2xl font-bold mt-1 ${(health?.error_rate || 0) > 5 ? 'text-red-500' : (health?.error_rate || 0) > 1 ? 'text-yellow-500' : 'text-green-500'}`}>
                {(health?.error_rate || 0).toFixed(2)}%
              </div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Avg Response</div>
              <div className="text-2xl font-bold mt-1">
                {(health?.avg_response_time_ms || 0).toFixed(1)}ms
              </div>
            </div>
          </div>
        )}

        {/* Cache & Cost Stats */}
        {(activePanel === 'overview' || activePanel === 'cache') && (
          <div className="grid grid-cols-3 gap-4">
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Cache Entries</div>
              <div className="text-2xl font-bold mt-1 text-[#4248f1]">
                {cache?.active_entries || 0}
              </div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Expired</div>
              <div className="text-2xl font-bold mt-1 text-yellow-500">
                {cache?.expired_entries || 0}
              </div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Total Cost</div>
              <div className="text-2xl font-bold mt-1 text-green-500">
                ${(metrics?.total_cost_usd || 0).toFixed(2)}
              </div>
            </div>
          </div>
        )}

        {/* Task Stats */}
        {(activePanel === 'overview' || activePanel === 'tasks') && (
          <div className="grid grid-cols-4 gap-4">
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Total Tasks</div>
              <div className="text-2xl font-bold mt-1">{totalTasks}</div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Completed</div>
              <div className="text-2xl font-bold mt-1 text-green-500">{completedTasks}</div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Failed</div>
              <div className="text-2xl font-bold mt-1 text-red-500">{failedTasks}</div>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <div className="text-sm text-[var(--color-text-muted)]">Success Rate</div>
              <div className="text-2xl font-bold mt-1 text-green-500">{successRate}%</div>
            </div>
          </div>
        )}

        {/* Overview Panels */}
        {activePanel === 'overview' && (
          <>
            {/* Tasks by Tier */}
            <div className="border border-[var(--color-border)] rounded-xl bg-[var(--color-panel)]">
              <div className="border-b border-[var(--color-border)] p-3">
                <h3 className="font-semibold">Tasks by Tier</h3>
              </div>
              <div className="p-4 grid grid-cols-3 gap-4">
                {Object.entries(metrics?.by_tier || {}).map(([tier, count]) => (
                  <div key={tier} className="bg-[var(--color-muted)]/30 rounded-lg p-3">
                    <div className="text-sm text-[var(--color-text-muted)] capitalize">{tier}</div>
                    <div className="text-xl font-bold">{count}</div>
                  </div>
                ))}
              </div>
            </div>

            {/* Tasks by Status */}
            <div className="border border-[var(--color-border)] rounded-xl bg-[var(--color-panel)]">
              <div className="border-b border-[var(--color-border)] p-3">
                <h3 className="font-semibold">Tasks by Status</h3>
              </div>
              <div className="p-4 grid grid-cols-5 gap-4">
                {Object.entries(metrics?.by_status || {}).map(([status, count]) => (
                  <div key={status} className="bg-[var(--color-muted)]/30 rounded-lg p-3">
                    <div className="text-sm text-[var(--color-text-muted)] capitalize">{status}</div>
                    <div className="text-xl font-bold">{count}</div>
                  </div>
                ))}
              </div>
            </div>

            {/* Top Endpoints */}
            <div className="border border-[var(--color-border)] rounded-xl bg-[var(--color-panel)]">
              <div className="border-b border-[var(--color-border)] p-3">
                <h3 className="font-semibold">Top Endpoints</h3>
              </div>
              <div className="p-4 space-y-2">
                {Object.entries(health?.requests_by_endpoint || {})
                  .sort(([, a], [, b]) => b - a)
                  .slice(0, 10)
                  .map(([endpoint, count]) => (
                    <div key={endpoint} className="flex items-center justify-between p-2 hover:bg-[var(--color-muted)]/30 rounded-lg transition-colors">
                      <span className="font-mono text-sm">{endpoint}</span>
                      <span className="font-medium">{count.toLocaleString()}</span>
                    </div>
                  ))}
              </div>
            </div>
          </>
        )}

        {/* Health Error */}
        {activePanel === 'health' && health?.last_error && (
          <div className="border border-red-500/30 rounded-xl p-4 bg-red-500/5">
            <h3 className="font-semibold text-red-500 mb-2 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="15" y1="9" x2="9" y2="15"></line>
                <line x1="9" y1="9" x2="15" y2="15"></line>
              </svg>
              Last Error
            </h3>
            <p className="text-sm text-[var(--color-text-muted)]">{health.last_error}</p>
          </div>
        )}

        {/* Telemetry Panel (Phase 2) */}
        {activePanel === 'telemetry' && (
          <>
            {/* Endpoint Latencies */}
            <div className="border border-[var(--color-border)] rounded-xl bg-[var(--color-panel)]">
              <div className="border-b border-[var(--color-border)] p-3 flex items-center justify-between">
                <h3 className="font-semibold">Endpoint Latencies</h3>
                <span className="text-xs text-[var(--color-text-muted)]">
                  {Object.keys(metrics?.telemetry?.endpoint_latencies || {}).length} endpoints
                </span>
              </div>
              <div className="p-4 overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-[var(--color-text-muted)] border-b border-[var(--color-border)]">
                      <th className="text-left py-2 px-3">Endpoint</th>
                      <th className="text-right py-2 px-3">Count</th>
                      <th className="text-right py-2 px-3">Avg</th>
                      <th className="text-right py-2 px-3">P50</th>
                      <th className="text-right py-2 px-3">P95</th>
                      <th className="text-right py-2 px-3">P99</th>
                      <th className="text-right py-2 px-3">Min</th>
                      <th className="text-right py-2 px-3">Max</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(metrics?.telemetry?.endpoint_latencies || {})
                      .sort(([, a], [, b]) => b.avg_ms - a.avg_ms)
                      .map(([endpoint, stats]) => (
                        <tr key={endpoint} className="border-b border-[var(--color-border)]/50 hover:bg-[var(--color-muted)]/20">
                          <td className="py-2 px-3 font-mono text-xs">{endpoint}</td>
                          <td className="text-right py-2 px-3">{stats.count}</td>
                          <td className="text-right py-2 px-3">{stats.avg_ms}ms</td>
                          <td className="text-right py-2 px-3">{stats.p50_ms}ms</td>
                          <td className="text-right py-2 px-3">
                            <span className={stats.p95_ms > 500 ? 'text-yellow-500' : ''}>
                              {stats.p95_ms}ms
                            </span>
                          </td>
                          <td className="text-right py-2 px-3">
                            <span className={stats.p99_ms > 1000 ? 'text-red-500' : ''}>
                              {stats.p99_ms}ms
                            </span>
                          </td>
                          <td className="text-right py-2 px-3 text-green-500">{stats.min_ms}ms</td>
                          <td className="text-right py-2 px-3">{stats.max_ms}ms</td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              </div>
            </div>

            {/* Endpoint Error Rates */}
            <div className="border border-[var(--color-border)] rounded-xl bg-[var(--color-panel)]">
              <div className="border-b border-[var(--color-border)] p-3 flex items-center justify-between">
                <h3 className="font-semibold">Endpoint Error Rates</h3>
                <span className="text-xs text-[var(--color-text-muted)]">
                  {Object.keys(metrics?.telemetry?.endpoint_errors || {}).length} endpoints
                </span>
              </div>
              <div className="p-4 overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="text-[var(--color-text-muted)] border-b border-[var(--color-border)]">
                      <th className="text-left py-2 px-3">Endpoint</th>
                      <th className="text-right py-2 px-3">Requests</th>
                      <th className="text-right py-2 px-3">Errors</th>
                      <th className="text-right py-2 px-3">Error Rate</th>
                      <th className="text-left py-2 px-3">Error Types</th>
                    </tr>
                  </thead>
                  <tbody>
                    {Object.entries(metrics?.telemetry?.endpoint_errors || {})
                      .filter(([, stats]) => stats.requests > 0)
                      .sort(([, a], [, b]) => b.error_rate_pct - a.error_rate_pct)
                      .map(([endpoint, stats]) => (
                        <tr key={endpoint} className="border-b border-[var(--color-border)]/50 hover:bg-[var(--color-muted)]/20">
                          <td className="py-2 px-3 font-mono text-xs">{endpoint}</td>
                          <td className="text-right py-2 px-3">{stats.requests}</td>
                          <td className="text-right py-2 px-3">{stats.errors}</td>
                          <td className="text-right py-2 px-3">
                            <span className={stats.error_rate_pct > 5 ? 'text-red-500' : stats.error_rate_pct > 1 ? 'text-yellow-500' : 'text-green-500'}>
                              {stats.error_rate_pct.toFixed(2)}%
                            </span>
                          </td>
                          <td className="py-2 px-3">
                            <div className="flex gap-1 flex-wrap">
                              {Object.entries(stats.error_types).map(([type, count]) => (
                                <span key={type} className="text-xs bg-red-500/10 text-red-500 px-2 py-0.5 rounded">
                                  {type}: {count}
                                </span>
                              ))}
                            </div>
                          </td>
                        </tr>
                      ))}
                  </tbody>
                </table>
              </div>
            </div>
          </>
        )}

        {/* Last Updated */}
        <div className="text-xs text-[var(--color-text-muted)] text-center">
          Last updated: {new Date().toLocaleTimeString()}
        </div>
      </div>
    </div>
  );
}
