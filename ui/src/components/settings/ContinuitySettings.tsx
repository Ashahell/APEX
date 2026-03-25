import { useState, useEffect } from 'react';

// Task types for continuity scheduler
const TASK_TYPES = [
  { id: 'morning_greeting', name: 'Morning Greeting', description: 'Daily wake-up message', icon: '🌅' },
  { id: 'evening_checkin', name: 'Evening Check-in', description: 'Daily end-of-day review', icon: '🌙' },
  { id: 'weekly_summary', name: 'Weekly Summary', description: 'Weekly progress report', icon: '📊' },
  { id: 'dream_mode', name: 'Dream Mode', description: 'Background thought processing', icon: '💭' },
  { id: 'alarm', name: 'Alarm', description: 'Timed reminder', icon: '⏰' },
  { id: 'random_checkin', name: 'Random Check-in', description: 'Surprise interaction', icon: '🎲' },
  { id: 'custom', name: 'Custom', description: 'User-defined task', icon: '⚙️' },
];

interface ScheduledTask {
  id: string;
  name: string;
  task_type: string;
  schedule: string;
  enabled: boolean;
  last_run: string | null;
  next_run: string | null;
}

interface TaskHistoryEntry {
  id: string;
  task_id: string;
  task_name: string;
  started_at: string;
  completed_at: string | null;
  status: 'running' | 'success' | 'failed';
  error?: string;
}

export function ContinuitySettings() {
  const [tasks, setTasks] = useState<ScheduledTask[]>([]);
  const [history, setHistory] = useState<TaskHistoryEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [showEditor, setShowEditor] = useState(false);
  const [editingTask, setEditingTask] = useState<ScheduledTask | null>(null);
  const [activeTab, setActiveTab] = useState<'tasks' | 'history'>('tasks');

  // Form state
  const [formName, setFormName] = useState('');
  const [formType, setFormType] = useState('morning_greeting');
  const [formSchedule, setFormSchedule] = useState('0 8 * * *');

  useEffect(() => {
    loadScheduledTasks();
  }, []);

  const loadScheduledTasks = async () => {
    setIsLoading(true);
    try {
      // Try API first
      const res = await fetch('/api/v1/continuity/tasks', {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      
      if (res.ok) {
        const data = await res.json();
        setTasks(data.tasks || []);
      } else {
        // Fallback to localStorage
        const saved = localStorage.getItem('apex-continuity-tasks');
        if (saved) {
          setTasks(JSON.parse(saved));
        }
      }

      // Load history
      const historyRes = await fetch('/api/v1/continuity/history', {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      if (historyRes.ok) {
        const historyData = await historyRes.json();
        setHistory(historyData.history || []);
      }
    } catch (err) {
      console.warn('Failed to load continuity tasks:', err);
      const saved = localStorage.getItem('apex-continuity-tasks');
      if (saved) {
        setTasks(JSON.parse(saved));
      }
    } finally {
      setIsLoading(false);
    }
  };

  const saveTasks = async (newTasks: ScheduledTask[]) => {
    setIsSaving(true);
    try {
      await fetch('/api/v1/continuity/tasks/bulk', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
        body: JSON.stringify({ tasks: newTasks }),
      });
    } catch (err) {
      console.warn('Failed to save to API:', err);
    }
    localStorage.setItem('apex-continuity-tasks', JSON.stringify(newTasks));
    setTasks(newTasks);
    setIsSaving(false);
  };

  const handleToggleTask = async (taskId: string) => {
    const newTasks = tasks.map(t => 
      t.id === taskId ? { ...t, enabled: !t.enabled } : t
    );
    await saveTasks(newTasks);
  };

  const handleDeleteTask = async (taskId: string) => {
    const newTasks = tasks.filter(t => t.id !== taskId);
    await saveTasks(newTasks);
  };

  const handleCreateTask = async () => {
    const newTask: ScheduledTask = {
      id: crypto.randomUUID(),
      name: formName || TASK_TYPES.find(t => t.id === formType)?.name || 'New Task',
      task_type: formType,
      schedule: formSchedule,
      enabled: true,
      last_run: null,
      next_run: null,
    };
    await saveTasks([...tasks, newTask]);
    setShowEditor(false);
    resetForm();
  };

  const handleEditTask = (task: ScheduledTask) => {
    setEditingTask(task);
    setFormName(task.name);
    setFormType(task.task_type);
    setFormSchedule(task.schedule);
    setShowEditor(true);
  };

  const handleUpdateTask = async () => {
    if (!editingTask) return;
    
    const newTasks = tasks.map(t => 
      t.id === editingTask.id 
        ? { ...t, name: formName, task_type: formType, schedule: formSchedule }
        : t
    );
    await saveTasks(newTasks);
    setShowEditor(false);
    setEditingTask(null);
    resetForm();
  };

  const resetForm = () => {
    setFormName('');
    setFormType('morning_greeting');
    setFormSchedule('0 8 * * *');
  };

  const getTaskIcon = (taskType: string) => {
    return TASK_TYPES.find(t => t.id === taskType)?.icon || '📋';
  };

  const formatCron = (cron: string) => {
    // Simple cron format display
    const parts = cron.split(' ');
    if (parts.length >= 3) {
      const [min, hour] = parts;
      return `Daily at ${hour}:${min.padStart(2, '0')}`;
    }
    return cron;
  };

  const formatDateTime = (iso: string | null) => {
    if (!iso) return 'Never';
    const date = new Date(iso);
    return date.toLocaleString();
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            Continuity Scheduler
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Schedule autonomous tasks and check-ins
          </p>
        </div>
        <button
          onClick={() => { setShowEditor(true); setEditingTask(null); resetForm(); }}
          className="px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700 transition-colors"
        >
          + New Task
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-200 dark:border-gray-700">
        <button
          onClick={() => setActiveTab('tasks')}
          className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
            activeTab === 'tasks'
              ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400'
              : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400'
          }`}
        >
          Scheduled Tasks ({tasks.length})
        </button>
        <button
          onClick={() => setActiveTab('history')}
          className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
            activeTab === 'history'
              ? 'border-indigo-500 text-indigo-600 dark:text-indigo-400'
              : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400'
          }`}
        >
          History
        </button>
      </div>

      {/* Task Editor Modal */}
      {showEditor && (
        <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
          <h4 className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-4">
            {editingTask ? 'Edit Task' : 'Create New Task'}
          </h4>
          
          <div className="space-y-4">
            <div>
              <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                Task Name
              </label>
              <input
                type="text"
                value={formName}
                onChange={(e) => setFormName(e.target.value)}
                placeholder="My Morning Task"
                className="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:ring-indigo-500 focus:border-indigo-500"
              />
            </div>

            <div>
              <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                Task Type
              </label>
              <select
                value={formType}
                onChange={(e) => setFormType(e.target.value)}
                className="w-full px-3 py-2 text-sm rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:ring-indigo-500 focus:border-indigo-500"
              >
                {TASK_TYPES.map(type => (
                  <option key={type.id} value={type.id}>
                    {type.icon} {type.name}
                  </option>
                ))}
              </select>
            </div>

            <div>
              <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                Schedule (Cron Format)
              </label>
              <input
                type="text"
                value={formSchedule}
                onChange={(e) => setFormSchedule(e.target.value)}
                placeholder="0 8 * * *"
                className="w-full px-3 py-2 text-sm font-mono rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900 focus:ring-indigo-500 focus:border-indigo-500"
              />
              <p className="text-xs text-gray-400 mt-1">
                Format: minute hour day month weekday
              </p>
            </div>

            <div className="flex gap-2">
              <button
                onClick={editingTask ? handleUpdateTask : handleCreateTask}
                disabled={isSaving}
                className="px-4 py-2 bg-indigo-600 text-white text-sm font-medium rounded-lg hover:bg-indigo-700 disabled:opacity-50"
              >
                {editingTask ? 'Update' : 'Create'}
              </button>
              <button
                onClick={() => { setShowEditor(false); setEditingTask(null); resetForm(); }}
                className="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-lg hover:bg-gray-300"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Tasks List */}
      {activeTab === 'tasks' && (
        <div className="space-y-3">
          {tasks.length === 0 ? (
            <div className="text-center py-12 text-gray-500 dark:text-gray-400">
              <div className="text-4xl mb-4">📅</div>
              <p className="text-sm">No scheduled tasks yet</p>
              <p className="text-xs mt-1">Click "New Task" to create one</p>
            </div>
          ) : (
            tasks.map(task => (
              <div
                key={task.id}
                className={`flex items-center justify-between p-4 rounded-lg border ${
                  task.enabled 
                    ? 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700' 
                    : 'bg-gray-50 dark:bg-gray-900 border-gray-100 dark:border-gray-800'
                }`}
              >
                <div className="flex items-center gap-4">
                  <div className="text-2xl">{getTaskIcon(task.task_type)}</div>
                  <div>
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {task.name}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {formatCron(task.schedule)} • Next: {formatDateTime(task.next_run)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <button
                    onClick={() => handleEditTask(task)}
                    className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    title="Edit"
                  >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                    </svg>
                  </button>
                  <button
                    onClick={() => handleDeleteTask(task.id)}
                    className="p-2 text-gray-400 hover:text-red-500"
                    title="Delete"
                  >
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                  <button
                    onClick={() => handleToggleTask(task.id)}
                    className={`relative inline-flex h-6 w-10 items-center rounded-full transition-colors ${
                      task.enabled ? 'bg-indigo-600' : 'bg-gray-200 dark:bg-gray-700'
                    }`}
                  >
                    <span
                      className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                        task.enabled ? 'translate-x-5' : 'translate-x-1'
                      }`}
                    />
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      )}

      {/* History List */}
      {activeTab === 'history' && (
        <div className="space-y-2">
          {history.length === 0 ? (
            <div className="text-center py-12 text-gray-500 dark:text-gray-400">
              <div className="text-4xl mb-4">📜</div>
              <p className="text-sm">No task history yet</p>
            </div>
          ) : (
            history.map(entry => (
              <div
                key={entry.id}
                className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-100 dark:border-gray-700"
              >
                <div className="flex items-center gap-3">
                  <div className={`w-2 h-2 rounded-full ${
                    entry.status === 'success' ? 'bg-green-500' :
                    entry.status === 'failed' ? 'bg-red-500' : 'bg-yellow-500'
                  }`} />
                  <div>
                    <p className="text-sm text-gray-900 dark:text-gray-100">
                      {entry.task_name}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {formatDateTime(entry.started_at)}
                    </p>
                  </div>
                </div>
                <span className={`text-xs px-2 py-1 rounded-full ${
                  entry.status === 'success' ? 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300' :
                  entry.status === 'failed' ? 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300' :
                  'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/50 dark:text-yellow-300'
                }`}>
                  {entry.status}
                </span>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
