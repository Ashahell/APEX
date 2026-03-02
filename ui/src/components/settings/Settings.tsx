import { useState, useEffect } from 'react';

interface VmStats {
  enabled: boolean;
  backend: string;
  total: number;
  ready: number;
  busy: number;
  available: number;
}

interface Metrics {
  tasks_total: Record<string, number>;
  tasks_completed: number;
  tasks_failed: number;
  total_cost_usd: number;
}

interface TaskHistoryItem {
  task_id: string;
  status: string;
  output: string | null;
  error: string | null;
}

interface TaskConfig {
  maxSteps: number;
  budgetUsd: number;
  timeLimitSecs: number | null;
}

const DEFAULT_CONFIG: TaskConfig = {
  maxSteps: 3,
  budgetUsd: 1.0,
  timeLimitSecs: null,
};

export function Settings() {
  const [vmStats, setVmStats] = useState<VmStats | null>(null);
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [taskHistory, setTaskHistory] = useState<TaskHistoryItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [tasksLoading, setTasksLoading] = useState(false);
  const [config, setConfig] = useState<TaskConfig>(DEFAULT_CONFIG);
  const [saved, setSaved] = useState(false);
  const [selectedTask, setSelectedTask] = useState<TaskHistoryItem | null>(null);

  const loadTaskHistory = async () => {
    setTasksLoading(true);
    try {
      const res = await fetch('http://localhost:3000/api/v1/tasks');
      const tasks = await res.json();
      setTaskHistory(Array.isArray(tasks) ? tasks.slice(0, 10) : []);
    } catch {}
    setTasksLoading(false);
  };

  useEffect(() => {
    // Load config from localStorage
    const savedConfig = localStorage.getItem('apex-task-config');
    if (savedConfig) {
      try {
        setConfig(JSON.parse(savedConfig));
      } catch {}
    }
    
    Promise.all([
      fetch('http://localhost:3000/api/v1/vm/stats').then((r) => r.json()),
      fetch('http://localhost:3000/api/v1/metrics').then((r) => r.json()),
    ])
      .then(([vm, met]) => {
        setVmStats(vm);
        setMetrics(met);
        loadTaskHistory();
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  const handleSave = () => {
    localStorage.setItem('apex-task-config', JSON.stringify(config));
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading settings...</div>
      </div>
    );
  }

  return (
    <div className="p-4 h-full overflow-y-auto">
      <div className="mb-6">
        <h2 className="text-2xl font-semibold">Settings</h2>
        <p className="text-muted-foreground">Configure APEX preferences</p>
      </div>

      <div className="space-y-6">
        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">System Information</h3>
          <div className="grid gap-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Version</span>
              <span>0.1.0</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Router URL</span>
              <span>http://localhost:3000</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Mode</span>
              <span>Local</span>
            </div>
          </div>
        </section>

        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Task Configuration</h3>
          <p className="text-sm text-muted-foreground mb-4">
            These settings apply to tasks. Note: Time limits are not applied when using local LLM.
          </p>
          <div className="grid gap-4">
            <div className="flex items-center justify-between">
              <label className="text-sm">Max Steps</label>
              <input
                type="number"
                min="1"
                max="100"
                value={config.maxSteps}
                onChange={(e) => setConfig({ ...config, maxSteps: parseInt(e.target.value) || 3 })}
                className="w-20 px-2 py-1 rounded border text-center"
              />
            </div>
            <div className="flex items-center justify-between">
              <label className="text-sm">Budget (USD)</label>
              <input
                type="number"
                min="0.1"
                max="100"
                step="0.1"
                value={config.budgetUsd}
                onChange={(e) => setConfig({ ...config, budgetUsd: parseFloat(e.target.value) || 1.0 })}
                className="w-20 px-2 py-1 rounded border text-center"
              />
            </div>
            <div className="flex items-center justify-between">
              <label className="text-sm">Time Limit (seconds)</label>
              <input
                type="number"
                min="0"
                placeholder="No limit"
                value={config.timeLimitSecs ?? ''}
                onChange={(e) => setConfig({ ...config, timeLimitSecs: e.target.value ? parseInt(e.target.value) : null })}
                className="w-20 px-2 py-1 rounded border text-center"
              />
            </div>
            <button
              onClick={handleSave}
              className="bg-primary text-primary-foreground px-4 py-2 rounded hover:bg-primary/90"
            >
              {saved ? 'Saved!' : 'Save Configuration'}
            </button>
          </div>
        </section>

        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">VM Pool</h3>
          {vmStats ? (
            <div className="grid gap-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Enabled</span>
                <span>{vmStats.enabled ? 'Yes' : 'No'}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Backend</span>
                <span>{vmStats.backend}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Total VMs</span>
                <span>{vmStats.total}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Available</span>
                <span>{vmStats.available}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Busy</span>
                <span>{vmStats.busy}</span>
              </div>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">VM pool not available</p>
          )}
        </section>

        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Task Metrics</h3>
          {metrics ? (
            <div className="grid gap-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Total Tasks</span>
                <span>{(metrics.tasks_total as any)?.total || 0}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Completed</span>
                <span>{metrics.tasks_completed || 0}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Failed</span>
                <span>{metrics.tasks_failed || 0}</span>
              </div>
              <div className="flex justify-between border-t pt-2 mt-2">
                <span className="text-muted-foreground font-semibold">Total Cost</span>
                <span className="font-semibold">${(metrics.total_cost_usd || 0).toFixed(4)}</span>
              </div>
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No metrics available</p>
          )}
        </section>

        <section className="border rounded-lg p-4">
          <div className="flex justify-between items-center mb-4">
            <h3 className="font-semibold">Task History</h3>
            <button
              onClick={loadTaskHistory}
              disabled={tasksLoading}
              className="text-xs px-2 py-1 rounded border hover:bg-muted disabled:opacity-50"
            >
              {tasksLoading ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          {taskHistory.length > 0 ? (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {taskHistory.map((task) => (
                <div 
                  key={task.task_id} 
                  className="flex justify-between items-center text-sm border-b pb-2 cursor-pointer hover:bg-muted p-1 -m-1 rounded"
                  onClick={() => setSelectedTask(task)}
                >
                  <div className="truncate flex-1 mr-2">
                    <span className="font-mono text-xs">{task.task_id.slice(0, 12)}...</span>
                    <span className={`ml-2 px-1.5 py-0.5 rounded text-xs ${
                      task.status === 'completed' ? 'bg-green-100 text-green-800' :
                      task.status === 'failed' ? 'bg-red-100 text-red-800' :
                      task.status === 'running' ? 'bg-blue-100 text-blue-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {task.status}
                    </span>
                  </div>
                  {task.output && (
                    <span className="text-muted-foreground text-xs">
                      {(() => {
                        try {
                          const parsed = JSON.parse(task.output);
                          return `$${parsed.total_cost_usd?.toFixed(4) || '0.0000'}`;
                        } catch {
                          return '';
                        }
                      })()}
                    </span>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">No tasks yet</p>
          )}
        </section>

        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">Environment Variables</h3>
          <div className="text-sm space-y-1">
            <p className="text-muted-foreground">
              To enable real VMs, set environment variables before starting the router:
            </p>
            <code className="block bg-muted p-2 rounded text-xs mt-2">
              APEX_USE_FIRECRACKER=1<br />
              APEX_VM_KERNEL=/path/to/vmlinux<br />
              APEX_VM_ROOTFS=/path/to/rootfs.ext4
            </code>
            <p className="text-muted-foreground mt-2">Or for gVisor:</p>
            <code className="block bg-muted p-2 rounded text-xs">
              APEX_USE_GVISOR=1
            </code>
          </div>
        </section>

        <section className="border rounded-lg p-4">
          <h3 className="font-semibold mb-4">About</h3>
          <p className="text-sm text-muted-foreground">
            APEX is a single-user autonomous agent platform. It combines messaging
            interfaces with secure code execution using Firecracker micro-VMs.
          </p>
        </section>
      </div>

      {selectedTask && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setSelectedTask(null)}>
          <div className="bg-background rounded-lg p-6 max-w-2xl w-full mx-4 max-h-[80vh] overflow-y-auto" onClick={(e) => e.stopPropagation()}>
            <div className="flex justify-between items-start mb-4">
              <h3 className="text-lg font-semibold">Task Details</h3>
              <button onClick={() => setSelectedTask(null)} className="text-muted-foreground hover:text-foreground">
                ✕
              </button>
            </div>
            <div className="space-y-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">Task ID:</span>
                <span className="font-mono">{selectedTask.task_id}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Status:</span>
                <span className={`px-2 py-0.5 rounded ${
                  selectedTask.status === 'completed' ? 'bg-green-100 text-green-800' :
                  selectedTask.status === 'failed' ? 'bg-red-100 text-red-800' :
                  selectedTask.status === 'running' ? 'bg-blue-100 text-blue-800' :
                  'bg-gray-100 text-gray-800'
                }`}>{selectedTask.status}</span>
              </div>
              {selectedTask.output && (
                <>
                  <div className="border-t pt-3 mt-3">
                    <span className="text-muted-foreground block mb-2">Output:</span>
                    <pre className="bg-muted p-3 rounded text-xs overflow-x-auto whitespace-pre-wrap">
{(() => {
  try {
    const parsed = JSON.parse(selectedTask.output);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return selectedTask.output;
  }
})()}
                    </pre>
                  </div>
                </>
              )}
              {selectedTask.error && (
                <div className="border-t pt-3 mt-3">
                  <span className="text-muted-foreground block mb-2">Error:</span>
                  <pre className="bg-red-50 p-3 rounded text-xs text-red-800 overflow-x-auto">{selectedTask.error}</pre>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
