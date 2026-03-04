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

interface Metrics {
  tasks: Record<string, number>;
  by_tier: Record<string, number>;
  by_status: Record<string, number>;
  total_cost_usd: number;
  tasks_completed: number;
  tasks_failed: number;
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
  const [health, setHealth] = useState<SystemHealth | null>(null);
  const [cache, setCache] = useState<CacheStats | null>(null);
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [activePanel, setActivePanel] = useState<'overview' | 'health' | 'cache' | 'tasks'>('overview');

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
        <div className="text-muted-foreground">Loading monitoring data...</div>
      </div>
    );
  }

  const totalTasks = metrics?.tasks?.total || 0;
  const completedTasks = metrics?.tasks_completed || 0;
  const failedTasks = metrics?.tasks_failed || 0;
  const successRate = totalTasks > 0 ? ((completedTasks / totalTasks) * 100).toFixed(1) : '0';

  return (
    <div className="h-full overflow-auto p-4">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold">Monitoring Dashboard</h2>
            <p className="text-sm text-muted-foreground">System health, cache, and task metrics</p>
          </div>
          <div className="flex gap-1 border rounded-lg p-1">
            {(['overview', 'health', 'cache', 'tasks'] as const).map((panel) => (
              <button
                key={panel}
                onClick={() => setActivePanel(panel)}
                className={`px-3 py-1 rounded text-sm transition-colors ${
                  activePanel === panel
                    ? 'bg-primary text-primary-foreground'
                    : 'hover:bg-muted'
                }`}
              >
                {panel.charAt(0).toUpperCase() + panel.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {(activePanel === 'overview' || activePanel === 'health') && (
          <div className="grid grid-cols-4 gap-4">
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Uptime</div>
              <div className="text-2xl font-bold mt-1">
                {formatUptime(health?.uptime_secs || 0)}
              </div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Total Requests</div>
              <div className="text-2xl font-bold mt-1">
                {(health?.requests_total || 0).toLocaleString()}
              </div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Error Rate</div>
              <div className={`text-2xl font-bold mt-1 ${(health?.error_rate || 0) > 5 ? 'text-red-500' : (health?.error_rate || 0) > 1 ? 'text-yellow-500' : 'text-green-500'}`}>
                {(health?.error_rate || 0).toFixed(2)}%
              </div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Avg Response</div>
              <div className="text-2xl font-bold mt-1">
                {(health?.avg_response_time_ms || 0).toFixed(1)}ms
              </div>
            </div>
          </div>
        )}

        {(activePanel === 'overview' || activePanel === 'cache') && (
          <div className="grid grid-cols-3 gap-4">
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Cache Entries</div>
              <div className="text-2xl font-bold mt-1">
                {cache?.active_entries || 0}
              </div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Expired</div>
              <div className="text-2xl font-bold mt-1 text-yellow-500">
                {cache?.expired_entries || 0}
              </div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Total Cost</div>
              <div className="text-2xl font-bold mt-1 text-green-500">
                ${(metrics?.total_cost_usd || 0).toFixed(2)}
              </div>
            </div>
          </div>
        )}

        {(activePanel === 'overview' || activePanel === 'tasks') && (
          <div className="grid grid-cols-4 gap-4">
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Total Tasks</div>
              <div className="text-2xl font-bold mt-1">{totalTasks}</div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Completed</div>
              <div className="text-2xl font-bold mt-1 text-green-500">{completedTasks}</div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Failed</div>
              <div className="text-2xl font-bold mt-1 text-red-500">{failedTasks}</div>
            </div>
            <div className="border rounded-lg p-4">
              <div className="text-sm text-muted-foreground">Success Rate</div>
              <div className="text-2xl font-bold mt-1 text-green-500">{successRate}%</div>
            </div>
          </div>
        )}

        {activePanel === 'overview' && (
          <>
            <div className="border rounded-lg">
              <div className="border-b p-3">
                <h3 className="font-semibold">Tasks by Tier</h3>
              </div>
              <div className="p-4 grid grid-cols-3 gap-4">
                {Object.entries(metrics?.by_tier || {}).map(([tier, count]) => (
                  <div key={tier} className="bg-muted/50 rounded-lg p-3">
                    <div className="text-sm text-muted-foreground capitalize">{tier}</div>
                    <div className="text-xl font-bold">{count}</div>
                  </div>
                ))}
              </div>
            </div>

            <div className="border rounded-lg">
              <div className="border-b p-3">
                <h3 className="font-semibold">Tasks by Status</h3>
              </div>
              <div className="p-4 grid grid-cols-5 gap-4">
                {Object.entries(metrics?.by_status || {}).map(([status, count]) => (
                  <div key={status} className="bg-muted/50 rounded-lg p-3">
                    <div className="text-sm text-muted-foreground capitalize">{status}</div>
                    <div className="text-xl font-bold">{count}</div>
                  </div>
                ))}
              </div>
            </div>

            <div className="border rounded-lg">
              <div className="border-b p-3">
                <h3 className="font-semibold">Top Endpoints</h3>
              </div>
              <div className="p-4 space-y-2">
                {Object.entries(health?.requests_by_endpoint || {})
                  .sort(([, a], [, b]) => b - a)
                  .slice(0, 10)
                  .map(([endpoint, count]) => (
                    <div key={endpoint} className="flex items-center justify-between">
                      <span className="font-mono text-sm">{endpoint}</span>
                      <span className="font-medium">{count.toLocaleString()}</span>
                    </div>
                  ))}
              </div>
            </div>
          </>
        )}

        {activePanel === 'health' && health?.last_error && (
          <div className="border border-red-500/50 rounded-lg p-4 bg-red-500/10">
            <h3 className="font-semibold text-red-400 mb-2">Last Error</h3>
            <p className="text-sm text-red-300/80">{health.last_error}</p>
          </div>
        )}

        <div className="text-xs text-muted-foreground text-center">
          Last updated: {new Date().toLocaleTimeString()}
        </div>
      </div>
    </div>
  );
}
