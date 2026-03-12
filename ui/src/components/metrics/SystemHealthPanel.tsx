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
  const [health, setHealth] = useState< SystemHealth | null>(null);
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
          Loading system health...
        </div>
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
    <div className="h-full overflow-auto p-6">
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 12h-4l-3 9L9 3l-3 9H2"></path>
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold">System Health</h2>
            <p className="text-sm text-[var(--color-text-muted)]">Real-time system performance metrics</p>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-4 gap-4">
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Uptime</div>
            <div className="text-3xl font-bold mt-1 text-[#4248f1]">
              {formatUptime(health?.uptime_secs || 0)}
            </div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Total Requests</div>
            <div className="text-3xl font-bold mt-1">
              {totalRequests.toLocaleString()}
            </div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Errors</div>
            <div className={`text-3xl font-bold mt-1 ${errors > 0 ? 'text-red-500' : 'text-green-500'}`}>
              {errors.toLocaleString()}
            </div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Avg Response</div>
            <div className="text-3xl font-bold mt-1">
              {avgResponseTime.toFixed(1)}ms
            </div>
          </div>
        </div>

        {/* Error Rate & Last Error */}
        <div className="grid grid-cols-2 gap-4">
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <h3 className="font-semibold mb-4">Error Rate</h3>
            <div className="flex items-center gap-4">
              <div className="flex-1">
                <div className="h-4 bg-[var(--color-muted)] rounded-full overflow-hidden">
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
            <div className="border border-red-500/30 rounded-xl p-4 bg-red-500/5">
              <h3 className="font-semibold mb-2 text-red-500 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                  <line x1="15" y1="9" x2="9" y2="15"></line>
                  <line x1="9" y1="9" x2="15" y2="15"></line>
                </svg>
                Last Error
              </h3>
              <p className="text-sm text-[var(--color-text-muted)] line-clamp-2">
                {health.last_error}
              </p>
            </div>
          )}
        </div>

        {/* Requests by Endpoint */}
        <div className="border border-[var(--color-border)] rounded-xl bg-[var(--color-panel)]">
          <div className="border-b border-[var(--color-border)] p-3">
            <h3 className="font-semibold">Requests by Endpoint</h3>
          </div>
          <div className="p-4 space-y-3">
            {sortedEndpoints.length === 0 ? (
              <div className="text-[var(--color-text-muted)] text-sm">No requests recorded yet</div>
            ) : (
              sortedEndpoints.map(([endpoint, count]) => (
                <div key={endpoint}>
                  <div className="flex items-center justify-between text-sm mb-1.5">
                    <span className="font-mono text-xs">{endpoint}</span>
                    <span className="font-medium">{count.toLocaleString()}</span>
                  </div>
                  <div className="h-2 bg-[var(--color-muted)] rounded-full overflow-hidden">
                    <div 
                      className="h-full bg-[#4248f1] transition-all"
                      style={{ width: `${(count / maxRequests) * 100}%` }}
                    />
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Last Updated */}
        <div className="text-xs text-[var(--color-text-muted)] text-center">
          Last updated: {new Date().toLocaleTimeString()}
        </div>
      </div>
    </div>
  );
}
