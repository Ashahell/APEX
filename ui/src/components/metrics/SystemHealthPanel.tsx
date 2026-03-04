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

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  
  if (days > 0) return `${days}d ${hours}h ${minutes}m`;
  if (hours > 0) return `${hours}h ${minutes}m ${secs}s`;
  if (minutes > 0) return `${minutes}m ${secs}s`;
  return `${secs}s`;
}

export function SystemHealthPanel() {
  const [health, setHealth] = useState<SystemHealth | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadHealth();
    const interval = setInterval(loadHealth, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadHealth = async () => {
    try {
      const res = await apiGet('/api/v1/system/health');
      if (res.ok) {
        const data = await res.json();
        setHealth(data);
      }
    } catch (err) {
      console.error('Failed to load system health:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading system health...</div>
      </div>
    );
  }

  const totalRequests = health?.requests_total || 0;
  const errors = health?.errors_total || 0;
  const errorRate = health?.error_rate || 0;
  const avgResponseTime = health?.avg_response_time_ms || 0;
  const endpointData = health?.requests_by_endpoint || {};

  const sortedEndpoints = Object.entries(endpointData)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 10);

  const maxRequests = Math.max(...Object.values(endpointData), 1);

  return (
    <div className="h-full overflow-auto p-4">
      <div className="max-w-6xl mx-auto space-y-6">
        <div>
          <h2 className="text-2xl font-bold">System Health</h2>
          <p className="text-sm text-muted-foreground">Real-time system performance metrics</p>
        </div>

        <div className="grid grid-cols-4 gap-4">
          <div className="border rounded-lg p-4">
            <div className="text-sm text-muted-foreground">Uptime</div>
            <div className="text-3xl font-bold mt-1">
              {formatUptime(health?.uptime_secs || 0)}
            </div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-sm text-muted-foreground">Total Requests</div>
            <div className="text-3xl font-bold mt-1">
              {totalRequests.toLocaleString()}
            </div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-sm text-muted-foreground">Errors</div>
            <div className={`text-3xl font-bold mt-1 ${errors > 0 ? 'text-red-500' : 'text-green-500'}`}>
              {errors.toLocaleString()}
            </div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-sm text-muted-foreground">Avg Response</div>
            <div className="text-3xl font-bold mt-1">
              {avgResponseTime.toFixed(1)}ms
            </div>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="border rounded-lg p-4">
            <h3 className="font-semibold mb-4">Error Rate</h3>
            <div className="flex items-center gap-4">
              <div className="flex-1">
                <div className="h-4 bg-gray-700 rounded-full overflow-hidden">
                  <div 
                    className={`h-full transition-all ${errorRate > 5 ? 'bg-red-500' : errorRate > 1 ? 'bg-yellow-500' : 'bg-green-500'}`}
                    style={{ width: `${Math.min(errorRate, 100)}%` }}
                  />
                </div>
              </div>
              <div className="text-2xl font-bold">
                {errorRate.toFixed(2)}%
              </div>
            </div>
          </div>

          {health?.last_error && (
            <div className="border border-red-500/50 rounded-lg p-4 bg-red-500/10">
              <h3 className="font-semibold mb-2 text-red-400">Last Error</h3>
              <p className="text-sm text-red-300/80 line-clamp-2">
                {health.last_error}
              </p>
            </div>
          )}
        </div>

        <div className="border rounded-lg">
          <div className="border-b p-3">
            <h3 className="font-semibold">Requests by Endpoint</h3>
          </div>
          <div className="p-4 space-y-3">
            {sortedEndpoints.length === 0 ? (
              <div className="text-muted-foreground text-sm">No requests recorded yet</div>
            ) : (
              sortedEndpoints.map(([endpoint, count]) => (
                <div key={endpoint}>
                  <div className="flex items-center justify-between text-sm mb-1">
                    <span className="font-mono text-xs">{endpoint}</span>
                    <span className="font-medium">{count.toLocaleString()}</span>
                  </div>
                  <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
                    <div 
                      className="h-full bg-primary transition-all"
                      style={{ width: `${(count / maxRequests) * 100}%` }}
                    />
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="text-xs text-muted-foreground">
          Last updated: {new Date().toLocaleTimeString()}
        </div>
      </div>
    </div>
  );
}
