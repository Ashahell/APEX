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
      case 'low': return 'bg-green-500/20 text-green-500 border-green-500/30';
      case 'medium': return 'bg-yellow-500/20 text-yellow-500 border-yellow-500/30';
      case 'high': return 'bg-red-500/20 text-red-500 border-red-500/30';
      default: return 'bg-[var(--color-text-muted)]/20 text-[var(--color-text-muted)] border-[var(--color-text-muted)]/30';
    }
  };

  return (
    <div className="h-full flex">
      <div className="w-1/3 border-r border-border flex flex-col">
        <div className="p-4 border-b border-border">
          <h2 className="font-bold mb-2 text-xl" style={{ color: '#4248f1' }}>Select Task</h2>
          <p className="text-sm text-[var(--color-text-muted)]">
            Choose a task to preview its consequences
          </p>
        </div>
        <div className="flex-1 overflow-auto">
          {loading ? (
            <div className="p-4 text-center text-[var(--color-text-muted)]">Loading...</div>
          ) : tasks.length === 0 ? (
            <div className="p-4 text-center text-[var(--color-text-muted)]">No tasks found</div>
          ) : (
            tasks.map((task) => (
              <button
                key={task.id}
                onClick={() => {
                  setSelectedTask(task);
                  analyzeConsequences(task.id);
                }}
                className={`w-full p-3 border-b border-border text-left hover:bg-[#4248f1]/10 transition-colors ${
                  selectedTask?.id === task.id ? 'bg-[#4248f1]/10' : ''
                }`}
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="font-mono text-sm">{task.id.slice(0, 8)}</span>
                  <span className={`text-xs px-2 py-0.5 rounded-lg ${
                    task.status === 'completed' ? 'bg-green-500/20 text-green-500' :
                    task.status === 'failed' ? 'bg-red-500/20 text-red-500' :
                    task.status === 'running' ? 'bg-[#4248f1]/20 text-[#4248f1]' :
                    'bg-[var(--color-text-muted)]/20 text-[var(--color-text-muted)]'
                  }`}>
                    {task.status}
                  </span>
                </div>
                <p className="text-sm text-[var(--color-text-muted)] line-clamp-2">
                  {task.input_content?.slice(0, 100)}
                </p>
              </button>
            ))
          )}
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {!selectedTask ? (
          <div className="flex items-center justify-center h-full text-[var(--color-text-muted)]">
            Select a task to view consequences
          </div>
        ) : analyzing ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="animate-spin w-8 h-8 border-2 border-[#4248f1] border-t-transparent rounded-full mx-auto mb-4" />
              <div className="text-[var(--color-text-muted)]">Analyzing consequences...</div>
            </div>
          </div>
        ) : error ? (
          <div className="p-4 bg-red-500/20 text-red-500 rounded-xl border border-red-500/30">
            {error}
          </div>
        ) : consequences ? (
          <div className="space-y-4">
            <div>
              <h2 className="text-2xl font-bold mb-2" style={{ color: '#4248f1' }}>Consequence Preview</h2>
              <p className="text-sm text-[var(--color-text-muted)]">
                Task: {selectedTask.id}
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
                <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Blast Radius</h3>
                <div className={`text-2xl font-bold capitalize ${getBlastRadiusColor(consequences.blast_radius)}`}>
                  {consequences.blast_radius}
                </div>
              </div>
              <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
                <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Risk Level</h3>
                <span className={`px-3 py-1 rounded-lg text-sm capitalize border ${getRiskColor(consequences.risk_level)}`}>
                  {consequences.risk_level}
                </span>
              </div>
            </div>

            <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
              <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Estimated Impact</h3>
              <p className="text-[var(--color-text-muted)]">{consequences.estimated_impact}</p>
            </div>

            {consequences.files_read.length > 0 && (
              <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
                <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Files to Read</h3>
                <div className="space-y-1">
                  {consequences.files_read.map((file, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <svg className="w-4 h-4 text-[#4248f1]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                      </svg>
                      <code className="bg-[var(--color-background)] px-2 py-0.5 rounded-lg border border-border">{file}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.files_written.length > 0 && (
              <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
                <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Files to Write</h3>
                <div className="space-y-1">
                  {consequences.files_written.map((file, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <svg className="w-4 h-4 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                      <code className="bg-[var(--color-background)] px-2 py-0.5 rounded-lg border border-border">{file}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.commands_executed.length > 0 && (
              <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
                <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Commands to Execute</h3>
                <div className="space-y-1">
                  {consequences.commands_executed.map((cmd, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <svg className="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                      </svg>
                      <code className="bg-[var(--color-background)] px-2 py-0.5 rounded-lg border border-border">{cmd}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.network_access.length > 0 && (
              <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
                <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>Network Access</h3>
                <div className="space-y-1">
                  {consequences.network_access.map((host, i) => (
                    <div key={i} className="flex items-center gap-2 text-sm">
                      <svg className="w-4 h-4 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                      </svg>
                      <code className="bg-[var(--color-background)] px-2 py-0.5 rounded-lg border border-border">{host}</code>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {consequences.files_read.length === 0 && 
             consequences.files_written.length === 0 && 
             consequences.commands_executed.length === 0 &&
             consequences.network_access.length === 0 && (
              <div className="border border-border rounded-xl p-8 text-center bg-[var(--color-panel)]">
                <svg className="w-12 h-12 mx-auto mb-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <h3 className="font-semibold">No Predicted Consequences</h3>
                <p className="text-sm text-[var(--color-text-muted)]">
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
