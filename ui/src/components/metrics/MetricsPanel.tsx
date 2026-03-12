import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface Metrics {
  tasks_total: Record<string, number>;
  tasks_by_tier: Record<string, number>;
  tasks_by_status: Record<string, number>;
  total_cost: number;
  total_cost_cents: number;
  average_task_duration_ms: number;
}

export function MetricsPanel() {
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState<'hour' | 'day' | 'week' | 'all'>('day');

  useEffect(() => {
    loadMetrics();
    const interval = setInterval(loadMetrics, 30000);
    return () => clearInterval(interval);
  }, [timeRange]);

  const loadMetrics = async () => {
    try {
      const res = await apiGet(`/api/v1/metrics?range=${timeRange}`);
      if (res.ok) {
        const data = await res.json();
        setMetrics(data);
      }
    } catch (err) {
      console.error('Failed to load metrics:', err);
    } finally {
      setLoading(false);
    }
  };

  const formatCost = (cents: number) => {
    return `$${(cents / 100).toFixed(2)}`;
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  };

  const getPercentage = (value: number, total: number) => {
    if (total === 0) return 0;
    return ((value / total) * 100).toFixed(1);
  };

  const totalTasks = metrics?.tasks_total?.total || 0;
  const tierData = metrics?.tasks_by_tier || {};
  const statusData = metrics?.tasks_by_status || {};

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading metrics...</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-6xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-12 h-12 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <line x1="18" y1="20" x2="18" y2="10"></line>
                <line x1="12" y1="20" x2="12" y2="4"></line>
                <line x1="6" y1="20" x2="6" y2="14"></line>
              </svg>
            </div>
            <div>
              <h2 className="text-2xl font-bold">Metrics</h2>
              <p className="text-sm text-[var(--color-text-muted)]">System performance and usage</p>
            </div>
          </div>
          <div className="flex gap-1 bg-[var(--color-muted)] p-1 rounded-lg">
            {(['hour', 'day', 'week', 'all'] as const).map((range) => (
              <button
                key={range}
                onClick={() => setTimeRange(range)}
                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                  timeRange === range
                    ? 'bg-[#4248f1] text-white'
                    : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
                }`}
              >
                {range.charAt(0).toUpperCase() + range.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-4 gap-4">
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Total Tasks</div>
            <div className="text-3xl font-bold mt-1">{totalTasks}</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Total Cost</div>
            <div className="text-3xl font-bold mt-1 text-green-500">
              {formatCost(metrics?.total_cost_cents || 0)}
            </div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Avg Duration</div>
            <div className="text-3xl font-bold mt-1">
              {formatDuration(metrics?.average_task_duration_ms || 0)}
            </div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="text-sm text-[var(--color-text-muted)]">Success Rate</div>
            <div className="text-3xl font-bold mt-1 text-[#4248f1]">
              {totalTasks > 0 
                ? getPercentage(statusData['completed'] || 0, totalTasks)
                : '0'
              }%
            </div>
          </div>
        </div>

        {/* Charts */}
        <div className="grid grid-cols-2 gap-4">
          <div className="border border-[var(--color-border)] rounded-xl overflow-hidden bg-[var(--color-panel)]">
            <div className="border-b border-[var(--color-border)] p-4">
              <h3 className="font-semibold flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="20" x2="18" y2="10"></line>
                  <line x1="12" y1="20" x2="12" y2="4"></line>
                  <line x1="6" y1="20" x2="6" y2="14"></line>
                </svg>
                Tasks by Tier
              </h3>
            </div>
            <div className="p-4 space-y-4">
              {Object.entries(tierData).map(([tier, count]) => (
                <div key={tier}>
                  <div className="flex items-center justify-between text-sm mb-2">
                    <span className="capitalize font-medium">{tier}</span>
                    <span className="text-[var(--color-text-muted)]">{count}</span>
                  </div>
                  <div className="h-2 bg-[var(--color-muted)] rounded-full overflow-hidden">
                    <div
                      className="h-full bg-[#4248f1] rounded-full"
                      style={{ width: `${getPercentage(count, totalTasks)}%` }}
                    />
                  </div>
                </div>
              ))}
              {Object.keys(tierData).length === 0 && (
                <div className="text-center text-[var(--color-text-muted)] py-4">No data</div>
              )}
            </div>
          </div>

          <div className="border border-[var(--color-border)] rounded-xl overflow-hidden bg-[var(--color-panel)]">
            <div className="border-b border-[var(--color-border)] p-4">
              <h3 className="font-semibold flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="20" x2="18" y2="10"></line>
                  <line x1="12" y1="20" x2="12" y2="4"></line>
                  <line x1="6" y1="20" x2="6" y2="14"></line>
                </svg>
                Tasks by Status
              </h3>
            </div>
            <div className="p-4 space-y-4">
              {Object.entries(statusData).map(([status, count]) => {
                const color = status === 'completed' ? 'bg-green-500' 
                  : status === 'failed' ? 'bg-red-500'
                  : status === 'running' ? 'bg-blue-500'
                  : 'bg-[var(--color-muted)]';
                return (
                  <div key={status}>
                    <div className="flex items-center justify-between text-sm mb-2">
                      <span className="capitalize font-medium">{status}</span>
                      <span className="text-[var(--color-text-muted)]">{count}</span>
                    </div>
                    <div className="h-2 bg-[var(--color-muted)] rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full ${color}`}
                        style={{ width: `${getPercentage(count, totalTasks)}%` }}
                      />
                    </div>
                  </div>
                );
              })}
              {Object.keys(statusData).length === 0 && (
                <div className="text-center text-[var(--color-text-muted)] py-4">No data</div>
              )}
            </div>
          </div>
        </div>

        {/* Cost Breakdown */}
        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="1" x2="12" y2="23"></line>
              <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"></path>
            </svg>
            Cost Breakdown
          </h3>
          <div className="grid grid-cols-3 gap-4">
            <div className="border border-[var(--color-border)] rounded-lg p-4 bg-[var(--color-background)]">
              <div className="text-sm text-[var(--color-text-muted)]">Total Cost</div>
              <div className="text-2xl font-bold text-green-500 mt-1">
                {formatCost(metrics?.total_cost_cents || 0)}
              </div>
            </div>
            <div className="border border-[var(--color-border)] rounded-lg p-4 bg-[var(--color-background)]">
              <div className="text-sm text-[var(--color-text-muted)]">Avg Cost/Task</div>
              <div className="text-2xl font-bold mt-1">
                {totalTasks > 0 
                  ? formatCost((metrics?.total_cost_cents || 0) / totalTasks)
                  : '$0.00'
                }
              </div>
            </div>
            <div className="border border-[var(--color-border)] rounded-lg p-4 bg-[var(--color-background)]">
              <div className="text-sm text-[var(--color-text-muted)]">Cost per Minute</div>
              <div className="text-2xl font-bold mt-1">
                {metrics?.total_cost ? `$${metrics.total_cost.toFixed(4)}` : '$0.00'}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
