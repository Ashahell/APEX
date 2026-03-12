import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface VmPoolStats {
  total_vms: number;
  active_vms: number;
  idle_vms: number;
  starting_vms: number;
  failed_vms: number;
  backend: string;
  max_vms: number;
}

interface VmInstance {
  id: string;
  state: string;
  backend: string;
  started_at: string;
  task_count: number;
}

export function VmPoolDashboard() {
  const [stats, setStats] = useState<VmPoolStats | null>(null);
  const [instances, setInstances] = useState<VmInstance[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [selectedVm, setSelectedVm] = useState<VmInstance | null>(null);

  useEffect(() => {
    loadStats();
    const interval = setInterval(loadStats, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadStats = async () => {
    try {
      const res = await apiGet('/api/v1/vm/stats');
      if (res.ok) {
        const data = await res.json();
        setStats(data);
        setInstances(data.instances || []);
      }
    } catch (err) {
      console.error('Failed to load VM stats:', err);
    } finally {
      setLoading(false);
      setRefreshing(false);
    }
  };

  const triggerRefresh = () => {
    setRefreshing(true);
    loadStats();
  };

  const getStateColor = (state: string) => {
    switch (state) {
      case 'ready': return 'bg-green-500';
      case 'busy': return 'bg-[#4248f1]';
      case 'starting': return 'bg-yellow-500';
      case 'failed': return 'bg-red-500';
      default: return 'bg-gray-500';
    }
  };

  const getStateText = (state: string) => {
    switch (state) {
      case 'ready': return 'Ready';
      case 'busy': return 'Busy';
      case 'starting': return 'Starting';
      case 'failed': return 'Failed';
      default: return state;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading VM pool...</div>
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
                <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
                <line x1="8" y1="21" x2="16" y2="21"></line>
                <line x1="12" y1="17" x2="12" y2="21"></line>
              </svg>
            </div>
            <div>
              <h2 className="text-2xl font-bold" style={{ color: '#4248f1' }}>VM Pool</h2>
              <p className="text-sm text-[var(--color-text-muted)]">
                Backend: {stats?.backend || 'unknown'} • Max: {stats?.max_vms || 0} VMs
              </p>
            </div>
          </div>
          <button
            onClick={triggerRefresh}
            disabled={refreshing}
            className="px-4 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] hover:bg-[var(--color-muted)] transition-colors flex items-center gap-2 disabled:opacity-50"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={refreshing ? 'animate-spin' : ''}>
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            {refreshing ? 'Refreshing...' : 'Refresh'}
          </button>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-5 gap-4">
          <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
            <div className="text-3xl font-bold">{stats?.total_vms || 0}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Total</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
            <div className="text-3xl font-bold text-green-500">{stats?.idle_vms || 0}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Idle</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
            <div className="text-3xl font-bold text-blue-500">{stats?.active_vms || 0}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Active</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
            <div className="text-3xl font-bold text-amber-500">{stats?.starting_vms || 0}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Starting</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-xl p-4 text-center bg-[var(--color-panel)]">
            <div className="text-3xl font-bold text-red-500">{stats?.failed_vms || 0}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Failed</div>
          </div>
        </div>

        {/* VM Instances */}
        <div className="border border-[var(--color-border)] rounded-xl overflow-hidden bg-[var(--color-panel)]">
          <div className="border-b border-[var(--color-border)] p-4">
            <h3 className="font-semibold flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
                <line x1="8" y1="21" x2="16" y2="21"></line>
                <line x1="12" y1="17" x2="12" y2="21"></line>
              </svg>
              VM Instances
            </h3>
          </div>
          <div className="divide-y divide-[var(--color-border)] max-h-96 overflow-auto">
            {instances.length === 0 ? (
              <div className="p-8 text-center text-[var(--color-text-muted)]">
                No VM instances running
              </div>
            ) : (
              instances.map((vm) => (
                <button
                  key={vm.id}
                  onClick={() => setSelectedVm(vm)}
                  className={`w-full p-4 text-left hover:bg-[var(--color-muted)]/30 transition-colors ${
                    selectedVm?.id === vm.id ? 'bg-[#4248f1]/5' : ''
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span className={`w-2.5 h-2.5 rounded-full ${getStateColor(vm.state)}`} />
                      <span className="font-mono text-sm">{vm.id}</span>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-[var(--color-text-muted)]">
                      <span className="px-2 py-1 rounded-full bg-[var(--color-muted)]">{vm.backend}</span>
                      <span className={`px-2 py-1 rounded-full ${
                        vm.state === 'ready' ? 'bg-green-500/10 text-green-500' :
                        vm.state === 'busy' ? 'bg-[#4248f1]/10 text-[#4248f1]' :
                        vm.state === 'starting' ? 'bg-amber-500/10 text-amber-500' :
                        'bg-red-500/10 text-red-500'
                      }`}>
                        {getStateText(vm.state)}
                      </span>
                      <span className="flex items-center gap-1">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
                        {vm.task_count} tasks
                      </span>
                    </div>
                  </div>
                </button>
              ))
            )}
          </div>
        </div>

        {selectedVm && (
          <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
            <h3 className="font-semibold mb-4 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="3"></circle>
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
              </svg>
              VM Details: {selectedVm.id}
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div className="p-3 bg-[var(--color-muted)]/30 rounded-lg">
                <span className="text-[var(--color-text-muted)] text-sm">State</span>
                <div className={`font-medium ${selectedVm.state === 'failed' ? 'text-red-500' : ''}`}>
                  {getStateText(selectedVm.state)}
                </div>
              </div>
              <div className="p-3 bg-[var(--color-muted)]/30 rounded-lg">
                <span className="text-[var(--color-text-muted)] text-sm">Backend</span>
                <div className="font-medium">{selectedVm.backend}</div>
              </div>
              <div className="p-3 bg-[var(--color-muted)]/30 rounded-lg">
                <span className="text-[var(--color-text-muted)] text-sm">Started</span>
                <div className="font-medium">
                  {selectedVm.started_at ? new Date(selectedVm.started_at).toLocaleString() : 'N/A'}
                </div>
              </div>
              <div className="p-3 bg-[var(--color-muted)]/30 rounded-lg">
                <span className="text-[var(--color-text-muted)] text-sm">Tasks</span>
                <div className="font-medium">{selectedVm.task_count}</div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
