import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface DeepTask {
  id: string;
  input_content: string;
  status: string;
  tier: string;
  max_steps: number;
  budget_usd: number;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
  output_content: string | null;
  error_message: string | null;
}

export function DeepTaskPanel() {
  const [tasks, setTasks] = useState<DeepTask[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedTask, setSelectedTask] = useState<DeepTask | null>(null);
  const [creating, setCreating] = useState(false);
  const [newTaskInput, setNewTaskInput] = useState('');
  const [newTaskMaxSteps, setNewTaskMaxSteps] = useState(50);
  const [newTaskBudget, setNewTaskBudget] = useState(1.0);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTasks();
    const interval = setInterval(loadTasks, 3000);
    return () => clearInterval(interval);
  }, []);

  const loadTasks = async () => {
    try {
      const res = await apiGet('/api/v1/tasks?limit=50&tier=deep');
      if (res.ok) {
        const data = await res.json();
        setTasks(data);
      }
    } catch (err) {
      console.error('Failed to load tasks:', err);
    } finally {
      setLoading(false);
    }
  };

  const createDeepTask = async () => {
    if (!newTaskInput.trim()) {
      setError('Task input is required');
      return;
    }
    setCreating(true);
    setError(null);
    try {
      const res = await apiPost('/api/v1/deep', {
        input_content: newTaskInput,
        max_steps: newTaskMaxSteps,
        budget_usd: newTaskBudget,
      });
      if (res.ok) {
        const task = await res.json();
        setSelectedTask(task);
        setNewTaskInput('');
        await loadTasks();
      } else {
        setError('Failed to create deep task');
      }
    } catch (err) {
      setError('Failed to create deep task');
    } finally {
      setCreating(false);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'completed': return 'bg-green-500';
      case 'failed': return 'bg-red-500';
      case 'running': return 'bg-blue-500';
      case 'pending': return 'bg-yellow-500';
      default: return 'bg-muted';
    }
  };

  const formatDuration = (start: string | null, end: string | null) => {
    if (!start) return '-';
    const startTime = new Date(start).getTime();
    const endTime = end ? new Date(end).getTime() : Date.now();
    const duration = (endTime - startTime) / 1000;
    if (duration < 60) return `${duration.toFixed(1)}s`;
    return `${(duration / 60).toFixed(1)}m`;
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  return (
    <div className="h-full flex">
      <div className="w-1/3 border-r flex flex-col">
        <div className="p-4 border-b space-y-3">
          <h2 className="font-bold">Deep Tasks</h2>
          <p className="text-sm text-muted-foreground">
            Execute complex tasks with full reasoning
          </p>
          
          <div className="space-y-2">
            <textarea
              value={newTaskInput}
              onChange={(e) => setNewTaskInput(e.target.value)}
              placeholder="Describe your complex task..."
              className="w-full px-3 py-2 rounded-lg border bg-background text-sm h-20 resize-none"
            />
            <div className="flex gap-2">
              <input
                type="number"
                value={newTaskMaxSteps}
                onChange={(e) => setNewTaskMaxSteps(parseInt(e.target.value) || 50)}
                min={1}
                max={500}
                className="w-24 px-2 py-1 rounded-lg border bg-background text-sm"
                title="Max Steps"
              />
              <input
                type="number"
                value={newTaskBudget}
                onChange={(e) => setNewTaskBudget(parseFloat(e.target.value) || 1.0)}
                min={0.01}
                max={100}
                step={0.1}
                className="w-24 px-2 py-1 rounded-lg border bg-background text-sm"
                title="Budget (USD)"
              />
              <button
                onClick={createDeepTask}
                disabled={creating || !newTaskInput.trim()}
                className="flex-1 px-3 py-1 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 text-sm"
              >
                {creating ? 'Starting...' : 'Start Deep Task'}
              </button>
            </div>
            {error && (
              <div className="text-sm text-red-500">{error}</div>
            )}
          </div>
        </div>

        <div className="flex-1 overflow-auto">
          {tasks.length === 0 ? (
            <div className="p-4 text-center text-muted-foreground">
              No deep tasks yet
            </div>
          ) : (
            tasks.map((task) => (
              <button
                key={task.id}
                onClick={() => setSelectedTask(task)}
                className={`w-full p-3 border-b text-left hover:bg-muted/50 transition-colors ${
                  selectedTask?.id === task.id ? 'bg-muted' : ''
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className={`w-2 h-2 rounded-full ${getStatusColor(task.status)}`} />
                  <span className="text-xs text-muted-foreground">
                    {formatDuration(task.started_at, task.completed_at)}
                  </span>
                </div>
                <p className="text-sm line-clamp-2 mb-1">
                  {task.input_content?.slice(0, 80)}
                </p>
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <span>{task.max_steps} steps</span>
                  <span>${task.budget_usd}</span>
                </div>
              </button>
            ))
          )}
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {!selectedTask ? (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            Select a task to view details
          </div>
        ) : (
          <div className="max-w-2xl mx-auto space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-bold">Task Details</h2>
              <span className={`px-3 py-1 rounded-full text-sm ${
                selectedTask.status === 'completed' ? 'bg-green-500/20 text-green-500' :
                selectedTask.status === 'failed' ? 'bg-red-500/20 text-red-500' :
                selectedTask.status === 'running' ? 'bg-blue-500/20 text-blue-500' :
                'bg-yellow-500/20 text-yellow-500'
              }`}>
                {selectedTask.status}
              </span>
            </div>

            <div className="grid grid-cols-3 gap-4">
              <div className="border rounded-lg p-3 text-center">
                <div className="text-2xl font-bold">{selectedTask.max_steps}</div>
                <div className="text-xs text-muted-foreground">Max Steps</div>
              </div>
              <div className="border rounded-lg p-3 text-center">
                <div className="text-2xl font-bold">${selectedTask.budget_usd}</div>
                <div className="text-xs text-muted-foreground">Budget</div>
              </div>
              <div className="border rounded-lg p-3 text-center">
                <div className="text-2xl font-bold">
                  {formatDuration(selectedTask.started_at, selectedTask.completed_at)}
                </div>
                <div className="text-xs text-muted-foreground">Duration</div>
              </div>
            </div>

            <div className="border rounded-lg p-4">
              <h3 className="font-semibold mb-2">Input</h3>
              <p className="text-sm whitespace-pre-wrap">{selectedTask.input_content}</p>
            </div>

            {selectedTask.error_message && (
              <div className="border border-red-500/30 rounded-lg p-4 bg-red-500/10">
                <h3 className="font-semibold mb-2 text-red-500">Error</h3>
                <p className="text-sm text-red-400">{selectedTask.error_message}</p>
              </div>
            )}

            {selectedTask.output_content && (
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Output</h3>
                <pre className="text-sm whitespace-pre-wrap bg-muted p-3 rounded-lg overflow-auto max-h-96">
                  {selectedTask.output_content}
                </pre>
              </div>
            )}

            <div className="text-sm text-muted-foreground">
              <div>ID: {selectedTask.id}</div>
              <div>Created: {new Date(selectedTask.created_at).toLocaleString()}</div>
              {selectedTask.started_at && (
                <div>Started: {new Date(selectedTask.started_at).toLocaleString()}</div>
              )}
              {selectedTask.completed_at && (
                <div>Completed: {new Date(selectedTask.completed_at).toLocaleString()}</div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
