import { useState, useEffect, useCallback } from 'react';
import { apiFetch, apiPost, apiPut } from '../../lib/api';

interface Task {
  task_id: string;
  status: string;
  content: string | null;
  output: string | null;
  error: string | null;
  project: string | null;
  priority: string | null;
  category: string | null;
  created_at: string | null;
}

interface FilterOptions {
  projects: string[];
  categories: string[];
  priorities: string[];
  statuses: string[];
}

const STATUS_COLUMNS = [
  { id: 'pending', label: 'Pending', color: 'bg-yellow-100 border-yellow-300' },
  { id: 'running', label: 'Running', color: 'bg-blue-100 border-blue-300' },
  { id: 'completed', label: 'Completed', color: 'bg-green-100 border-green-300' },
  { id: 'failed', label: 'Failed', color: 'bg-red-100 border-red-300' },
  { id: 'cancelled', label: 'Cancelled', color: 'bg-gray-100 border-gray-300' },
];

const PRIORITY_COLORS: Record<string, string> = {
  low: 'bg-gray-200 text-gray-700',
  medium: 'bg-blue-100 text-blue-700',
  high: 'bg-orange-100 text-orange-700',
  urgent: 'bg-red-100 text-red-700',
};

export function KanbanBoard() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [filterOptions, setFilterOptions] = useState<FilterOptions | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedProject, setSelectedProject] = useState<string>('');
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [editModal, setEditModal] = useState<{task: Task; field: string} | null>(null);
  const [newTaskModal, setNewTaskModal] = useState(false);
  const [newTask, setNewTask] = useState({ content: '', project: '', priority: 'medium', category: '' });
  const [runningTask, setRunningTask] = useState<string | null>(null);

  const fetchTasks = useCallback(async () => {
    try {
      const params = new URLSearchParams();
      if (selectedProject) params.set('project', selectedProject);
      params.set('limit', '100');
      
      const res = await apiFetch(`/api/v1/tasks?${params}`);
      const data = await res.json();
      setTasks(Array.isArray(data) ? data : []);
    } catch (err) {
      console.error('Failed to fetch tasks:', err);
    }
  }, [selectedProject]);

  const fetchFilterOptions = async () => {
    try {
      const res = await apiFetch('/api/v1/tasks/filter-options');
      const data = await res.json();
      setFilterOptions(data);
    } catch (err) {
      console.error('Failed to fetch filter options:', err);
    }
  };

  useEffect(() => {
    const load = async () => {
      setLoading(true);
      await Promise.all([fetchTasks(), fetchFilterOptions()]);
      setLoading(false);
    };
    load();
  }, [fetchTasks]);

  useEffect(() => {
    const interval = setInterval(fetchTasks, 5000);
    return () => clearInterval(interval);
  }, [fetchTasks]);

  const updateTask = async (taskId: string, updates: Record<string, string>) => {
    try {
      const res = await apiPut(`/api/v1/tasks/${taskId}`, updates);
      if (res.ok) {
        await fetchTasks();
      }
    } catch (err) {
      console.error('Failed to update task:', err);
    }
  };

  const createTask = async () => {
    try {
      const res = await apiPost('/api/v1/tasks', {
        content: newTask.content,
        project: newTask.project || undefined,
        priority: newTask.priority || undefined,
        category: newTask.category || undefined,
      });
      if (res.ok) {
        await fetchTasks();
        await fetchFilterOptions();
        setNewTaskModal(false);
        setNewTask({ content: '', project: '', priority: 'medium', category: '' });
      }
    } catch (err) {
      console.error('Failed to create task:', err);
    }
  };

  const runTask = async (taskId: string) => {
    setRunningTask(taskId);
    try {
      const res = await apiPost('/api/v1/tasks', { content: `Execute task ${taskId}` });
      if (res.ok) {
        await fetchTasks();
      }
    } catch (err) {
      console.error('Failed to run task:', err);
    } finally {
      setRunningTask(null);
    }
  };

  const handleTaskClick = (task: Task) => {
    setSelectedTask(task);
  };

  const handleStatusChange = (taskId: string, newStatus: string) => {
    updateTask(taskId, { status: newStatus });
  };

  const handleFieldEdit = (task: Task, field: string) => {
    setEditModal({ task, field });
  };

  const handleFieldSave = (value: string) => {
    if (editModal) {
      updateTask(editModal.task.task_id, { [editModal.field]: value });
      setEditModal(null);
    }
  };

  const getPriorityBadge = (priority: string | null) => {
    if (!priority) return null;
    return (
      <span className={`text-xs px-1.5 py-0.5 rounded ${PRIORITY_COLORS[priority] || 'bg-gray-100'}`}>
        {priority}
      </span>
    );
  };

  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  const getColumnTasks = (status: string) => {
    return tasks.filter(t => t.status === status);
  };

  if (loading && tasks.length === 0) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading kanban board...</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="border-b p-4 flex items-center justify-between bg-background">
        <h2 className="text-xl font-semibold">Task Board</h2>
        <div className="flex items-center gap-4">
          <button
            onClick={() => setNewTaskModal(true)}
            className="px-3 py-1.5 rounded bg-primary text-primary-foreground text-sm hover:bg-primary/90"
          >
            + New Task
          </button>
          <select
            value={selectedProject}
            onChange={(e) => setSelectedProject(e.target.value)}
            className="px-3 py-1.5 rounded border bg-background text-sm"
          >
            <option value="">All Projects</option>
            {filterOptions?.projects.map(p => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
          <button
            onClick={fetchTasks}
            className="px-3 py-1.5 rounded border bg-background text-sm hover:bg-muted"
          >
            Refresh
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-x-auto overflow-y-hidden p-4">
        <div className="flex gap-4 h-full min-w-max">
          {STATUS_COLUMNS.map(column => {
            const columnTasks = getColumnTasks(column.id);
            return (
              <div key={column.id} className="w-72 flex-shrink-0 flex flex-col">
                <div className={`px-3 py-2 rounded-t-lg border-t border-l border-r ${column.color}`}>
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-sm">{column.label}</span>
                    <span className="text-xs bg-white/50 px-2 py-0.5 rounded-full">
                      {columnTasks.length}
                    </span>
                  </div>
                </div>
                <div className="flex-1 border-l border-r border-b rounded-b-lg bg-gray-50 overflow-y-auto p-2 space-y-2">
                  {columnTasks.map(task => (
                    <div
                      key={task.task_id}
                      className="bg-white rounded-lg border p-3 shadow-sm cursor-pointer hover:shadow-md transition-shadow"
                      onClick={() => handleTaskClick(task)}
                    >
                      <div className="flex items-start justify-between mb-2">
                        <span className="font-mono text-xs text-muted-foreground">
                          {task.task_id.slice(0, 8)}...
                        </span>
                        {getPriorityBadge(task.priority)}
                      </div>
                      <p className="text-sm line-clamp-2 mb-2">
                        {task.output ? JSON.parse(task.output).input_content || task.output.slice(0, 100) : task.task_id}
                      </p>
                      <div className="flex items-center justify-between text-xs text-muted-foreground">
                        <span>{task.project || 'No project'}</span>
                        {task.category && (
                          <span className="bg-purple-100 text-purple-700 px-1.5 py-0.5 rounded">
                            {task.category}
                          </span>
                        )}
                      </div>
                      <div className="mt-2 flex gap-1">
                        {task.status === 'pending' && (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              runTask(task.task_id);
                            }}
                            disabled={runningTask === task.task_id}
                            className="text-xs px-2 py-1 rounded bg-green-100 hover:bg-green-200 text-green-700 disabled:opacity-50"
                          >
                            {runningTask === task.task_id ? '⏳' : '▶'} Run
                          </button>
                        )}
                        {STATUS_COLUMNS
                          .filter(c => c.id !== task.status)
                          .slice(0, 3)
                          .map(c => (
                            <button
                              key={c.id}
                              onClick={(e) => {
                                e.stopPropagation();
                                handleStatusChange(task.task_id, c.id);
                              }}
                              className="text-xs px-2 py-1 rounded bg-gray-100 hover:bg-gray-200 text-gray-600"
                            >
                              → {c.label}
                            </button>
                          ))}
                      </div>
                    </div>
                  ))}
                  {columnTasks.length === 0 && (
                    <div className="text-center text-muted-foreground text-sm py-8">
                      No tasks
                    </div>
                  )}
                </div>
              </div>
            );
          })}
        </div>
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
              {selectedTask.content && (
                <div className="mt-2">
                  <span className="text-muted-foreground">Content:</span>
                  <p className="mt-1 p-2 bg-muted rounded text-xs font-mono whitespace-pre-wrap">{selectedTask.content}</p>
                </div>
              )}
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">Status:</span>
                <select
                  value={selectedTask.status}
                  onChange={(e) => handleStatusChange(selectedTask.task_id, e.target.value)}
                  className="px-2 py-1 rounded border"
                >
                  {STATUS_COLUMNS.map(c => (
                    <option key={c.id} value={c.id}>{c.label}</option>
                  ))}
                </select>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">Project:</span>
                <button
                  onClick={() => handleFieldEdit(selectedTask, 'project')}
                  className="text-blue-600 hover:underline"
                >
                  {selectedTask.project || 'Set project'}
                </button>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">Priority:</span>
                <select
                  value={selectedTask.priority || 'medium'}
                  onChange={(e) => updateTask(selectedTask.task_id, { priority: e.target.value })}
                  className="px-2 py-1 rounded border"
                >
                  {filterOptions?.priorities.map(p => (
                    <option key={p} value={p}>{p}</option>
                  ))}
                </select>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-muted-foreground">Category:</span>
                <button
                  onClick={() => handleFieldEdit(selectedTask, 'category')}
                  className="text-blue-600 hover:underline"
                >
                  {selectedTask.category || 'Set category'}
                </button>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created:</span>
                <span>{formatDate(selectedTask.created_at)}</span>
              </div>
              {selectedTask.output && (
                <div className="border-t pt-3 mt-3">
                  <span className="text-muted-foreground block mb-2">Output:</span>
                  <pre className="bg-muted p-3 rounded text-xs overflow-x-auto whitespace-pre-wrap max-h-64">
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

      {editModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-[60]" onClick={() => setEditModal(null)}>
          <div className="bg-background rounded-lg p-6 max-w-md w-full mx-4" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-semibold mb-4">
              Edit {editModal.field.charAt(0).toUpperCase() + editModal.field.slice(1)}
            </h3>
            <input
              type="text"
              autoFocus
              defaultValue={editModal.task[editModal.field as keyof Task] as string || ''}
              placeholder={`Enter ${editModal.field}`}
              className="w-full px-3 py-2 rounded border mb-4"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  handleFieldSave((e.target as HTMLInputElement).value);
                }
              }}
            />
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => setEditModal(null)}
                className="px-4 py-2 rounded border"
              >
                Cancel
              </button>
              <button
                onClick={(e) => handleFieldSave((e.target as HTMLButtonElement).previousElementSibling?.querySelector('input')?.value || '')}
                className="px-4 py-2 rounded bg-primary text-primary-foreground"
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}

      {newTaskModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setNewTaskModal(false)}>
          <div className="bg-background rounded-lg p-6 max-w-lg w-full mx-4" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-semibold mb-4">Create New Task</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Task Description</label>
                <textarea
                  value={newTask.content}
                  onChange={(e) => setNewTask({ ...newTask, content: e.target.value })}
                  placeholder="What do you want to do?"
                  className="w-full px-3 py-2 rounded border bg-background h-24 resize-none"
                  autoFocus
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium mb-1">Project</label>
                  <input
                    type="text"
                    value={newTask.project}
                    onChange={(e) => setNewTask({ ...newTask, project: e.target.value })}
                    placeholder="project-name"
                    list="project-list"
                    className="w-full px-3 py-2 rounded border bg-background"
                  />
                  <datalist id="project-list">
                    {filterOptions?.projects.map(p => (
                      <option key={p} value={p} />
                    ))}
                  </datalist>
                </div>
                <div>
                  <label className="block text-sm font-medium mb-1">Priority</label>
                  <select
                    value={newTask.priority}
                    onChange={(e) => setNewTask({ ...newTask, priority: e.target.value })}
                    className="w-full px-3 py-2 rounded border bg-background"
                  >
                    <option value="low">Low</option>
                    <option value="medium">Medium</option>
                    <option value="high">High</option>
                    <option value="urgent">Urgent</option>
                  </select>
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Category</label>
                <input
                  type="text"
                  value={newTask.category}
                  onChange={(e) => setNewTask({ ...newTask, category: e.target.value })}
                  placeholder="bug, feature, research..."
                  list="category-list"
                  className="w-full px-3 py-2 rounded border bg-background"
                />
                <datalist id="category-list">
                  {filterOptions?.categories.map(c => (
                    <option key={c} value={c} />
                  ))}
                </datalist>
              </div>
            </div>
            <div className="flex gap-2 justify-end mt-6">
              <button
                onClick={() => setNewTaskModal(false)}
                className="px-4 py-2 rounded border"
              >
                Cancel
              </button>
              <button
                onClick={createTask}
                disabled={!newTask.content.trim()}
                className="px-4 py-2 rounded bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                Create Task
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
