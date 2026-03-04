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
      case 'busy': return 'bg-blue-500';
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
        <div className="text-muted-foreground">Loading VM pool...</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-4">
      <div className="max-w-6xl mx-auto space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold">VM Pool</h2>
            <p className="text-sm text-muted-foreground">
              Backend: {stats?.backend || 'unknown'}
            </p>
          </div>
          <button
            onClick={triggerRefresh}
            disabled={refreshing}
            className="px-4 py-2 rounded-lg border hover:bg-muted disabled:opacity-50"
          >
            {refreshing ? 'Refreshing...' : 'Refresh'}
          </button>
        </div>

        <div className="grid grid-cols-5 gap-4">
          <div className="border rounded-lg p-4 text-center">
            <div className="text-3xl font-bold">{stats?.total_vms || 0}</div>
            <div className="text-sm text-muted-foreground">Total</div>
          </div>
          <div className="border rounded-lg p-4 text-center">
            <div className="text-3xl font-bold text-green-500">{stats?.idle_vms || 0}</div>
            <div className="text-sm text-muted-foreground">Idle</div>
          </div>
          <div className="border rounded-lg p-4 text-center">
            <div className="text-3xl font-bold text-blue-500">{stats?.active_vms || 0}</div>
            <div className="text-sm text-muted-foreground">Active</div>
          </div>
          <div className="border rounded-lg p-4 text-center">
            <div className="text-3xl font-bold text-yellow-500">{stats?.starting_vms || 0}</div>
            <div className="text-sm text-muted-foreground">Starting</div>
          </div>
          <div className="border rounded-lg p-4 text-center">
            <div className="text-3xl font-bold text-red-500">{stats?.failed_vms || 0}</div>
            <div className="text-sm text-muted-foreground">Failed</div>
          </div>
        </div>

        <div className="border rounded-lg">
          <div className="border-b p-3">
            <h3 className="font-semibold">VM Instances</h3>
          </div>
          <div className="divide-y max-h-96 overflow-auto">
            {instances.length === 0 ? (
              <div className="p-8 text-center text-muted-foreground">
                No VM instances running
              </div>
            ) : (
              instances.map((vm) => (
                <button
                  key={vm.id}
                  onClick={() => setSelectedVm(vm)}
                  className={`w-full p-3 text-left hover:bg-muted/50 transition-colors ${
                    selectedVm?.id === vm.id ? 'bg-muted' : ''
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <span className={`w-2 h-2 rounded-full ${getStateColor(vm.state)}`} />
                      <span className="font-mono text-sm">{vm.id}</span>
                    </div>
                    <div className="flex items-center gap-4 text-sm text-muted-foreground">
                      <span>{vm.backend}</span>
                      <span>{getStateText(vm.state)}</span>
                      <span>{vm.task_count} tasks</span>
                    </div>
                  </div>
                </button>
              ))
            )}
          </div>
        </div>

        {selectedVm && (
          <div className="border rounded-lg p-4">
            <h3 className="font-semibold mb-3">VM Details: {selectedVm.id}</h3>
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span className="text-muted-foreground">State:</span>{' '}
                <span className={`font-medium ${selectedVm.state === 'failed' ? 'text-red-500' : ''}`}>
                  {getStateText(selectedVm.state)}
                </span>
              </div>
              <div>
                <span className="text-muted-foreground">Backend:</span>{' '}
                <span className="font-medium">{selectedVm.backend}</span>
              </div>
              <div>
                <span className="text-muted-foreground">Started:</span>{' '}
                <span className="font-medium">
                  {selectedVm.started_at ? new Date(selectedVm.started_at).toLocaleString() : 'N/A'}
                </span>
              </div>
              <div>
                <span className="text-muted-foreground">Tasks:</span>{' '}
                <span className="font-medium">{selectedVm.task_count}</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
