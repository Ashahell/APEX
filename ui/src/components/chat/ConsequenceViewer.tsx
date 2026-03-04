import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface Consequence {
  files_read: string[];
  files_written: string[];
  commands_executed: string[];
  network_access: string[];
  blast_radius: string;
  risk_level: string;
  estimated_impact: string;
}

interface Task {
  id: string;
  input_content: string;
  status: string;
  created_at: string;
}

export function ConsequenceViewer() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [consequences, setConsequences] = useState<Consequence | null>(null);
  const [loading, setLoading] = useState(true);
  const [analyzing, setAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadTasks();
  }, []);

  const loadTasks = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/tasks?limit=50');
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

  const analyzeConsequences = async (taskId: string) => {
    setAnalyzing(true);
    setError(null);
    setConsequences(null);
    try {
      const res = await apiPost(`/api/v1/tasks/${taskId}/consequences`, {});
      if (res.ok) {
        const data = await res.json();
        setConsequences(data);
      } else {
        setError('Failed to analyze consequences');
      }
    } catch (err) {
      setError('Failed to analyze consequences');
    } finally {
      setAnalyzing(false);
    }
  };

  const getBlastRadiusColor = (radius: string) => {
    switch (radius) {
      case 'minimal': return 'text-green-500';
      case 'limited': return 'text-yellow-500';
      case 'extensive': return 'text-red-500';
      default: return 'text-muted-foreground';
    }
  };

  const getRiskColor = (risk: string) => {
    switch (risk) {
      case 'low': return 'bg-green-500/20 text-green-500';
      case 'medium': return 'bg-yellow-500/20 text-yellow-500';
      case 'high': return 'bg-red-500/20 text-red-500';
      default: return 'bg-muted text-muted-foreground';
    }
  };

  return (
    <div className="h-full flex">
      <div className="w-1/3 border-r flex flex-col">
        <div className="p-4 border-b">
          <h2 className="font-bold mb-2">Select Task</h2>
          <p className="text-sm text-muted-foreground">
            Choose a task to preview its consequences
          </p>
        </div>
        <div className="flex-1 overflow-auto">
          {loading ? (
            <div className="p-4 text-center text-muted-foreground">Loading...</div>
          ) : tasks.length === 0 ? (
            <div className="p-4 text-center text-muted-foreground">No tasks found</div>
          ) : (
            tasks.map((task) => (
              <button
                key={task.id}
                onClick={() => {
                  setSelectedTask(task);
                  analyzeConsequences(task.id);
                }}
                className={`w-full p-3 border-b text-left hover:bg-muted/50 transition-colors ${
                  selectedTask?.id === task.id ? 'bg-muted' : ''
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="font-mono text-sm">{task.id.slice(0, 8)}</span>
                  <span className={`text-xs px-2 py-0.5 rounded ${
                    task.status === 'completed' ? 'bg-green-500/20 text-green-500' :
                    task.status === 'failed' ? 'bg-red-500/20 text-red-500' :
                    task.status === 'running' ? 'bg-blue-500/20 text-blue-500' :
                    'bg-muted text-muted-foreground'
                  }`}>
                    {task.status}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground line-clamp-2">
                  {task.input_content?.slice(0, 100)}
                </p>
              </button>
            ))
          )}
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {!selectedTask ? (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            Select a task to view consequences
          </div>
        ) : analyzing ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4" />
              <div className="text-muted-foreground">Analyzing consequences...</div>
            </div>
          </div>
        ) : error ? (
          <div className="p-4 bg-red-500/20 text-red-500 rounded-lg">
            {error}
          </div>
        ) : consequences ? (
          <div className="space-y-4">
            <div>
              <h2 className="text-2xl font-bold mb-2">Consequence Preview</h2>
              <p className="text-sm text-muted-foreground">
                Task: {selectedTask.id}
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Blast Radius</h3>
                <div className={`text-2xl font-bold capitalize ${getBlastRadiusColor(consequences.blast_radius)}`}>
                  {consequences.blast_radius}
                </div>
              </div>
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Risk Level</h3>
                <span className={`px-3 py-1 rounded text-sm capitalize ${getRiskColor(consequences.risk_level)}`}>
                  {consequences.risk_level}
                </span>
              </div>
            </div>

            <div className="border rounded-lg p-4">
              <h3 className="font-semibold mb-2">Estimated Impact</h3>
              <p className="text-muted-foreground">{consequences.estimated_impact}</p>
            </div>

            {consequences.files_read.length > 0 && (
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Files to Read</h3>
                <div className="space-y-1">
                  {consequences.files_read.map((file, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <span className="text-blue-500">📖</span>
                      <code className="bg-muted px-2 py-0.5 rounded">{file}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.files_written.length > 0 && (
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Files to Write</h3>
                <div className="space-y-1">
                  {consequences.files_written.map((file, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <span className="text-yellow-500">📝</span>
                      <code className="bg-muted px-2 py-0.5 rounded">{file}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.commands_executed.length > 0 && (
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Commands to Execute</h3>
                <div className="space-y-1">
                  {consequences.commands_executed.map((cmd, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <span className="text-red-500">⚡</span>
                      <code className="bg-muted px-2 py-0.5 rounded">{cmd}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.network_access.length > 0 && (
              <div className="border rounded-lg p-4">
                <h3 className="font-semibold mb-2">Network Access</h3>
                <div className="space-y-1">
                  {consequences.network_access.map((host, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <span className="text-purple-500">🌐</span>
                      <code className="bg-muted px-2 py-0.5 rounded">{host}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.files_read.length === 0 && 
             consequences.files_written.length === 0 && 
             consequences.commands_executed.length === 0 &&
             consequences.network_access.length === 0 && (
              <div className="border rounded-lg p-8 text-center">
                <div className="text-4xl mb-2">✓</div>
                <h3 className="font-semibold">No Predicted Consequences</h3>
                <p className="text-sm text-muted-foreground">
                  This task is expected to have minimal impact
                </p>
              </div>
            )}
          </div>
        ) : null}
      </div>
    </div>
  );
}
