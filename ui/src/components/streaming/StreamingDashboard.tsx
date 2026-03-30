import React, { useState, useEffect } from 'react';
import { useStreaming } from '../../hooks/useStreaming';
import { apiGet } from '../../lib/api';

// ============================================================================
// Types - SSE Envelope and Payloads
// ============================================================================

// Phase 1: Extended event types
export type StreamEventType = 
  | 'connected' 
  | 'disconnected' 
  | 'hands' 
  | 'mcp' 
  | 'task' 
  | 'stats' 
  | 'heartbeat' 
  | 'error'
  // Phase 1: Richer event types
  | 'session_start'
  | 'session_end'
  | 'checkpoint'
  | 'user_intervention';

export interface SseEnvelope<T> {
  type: StreamEventType;
  timestamp: number;
  trace_id?: string;
  payload: T;
}

export interface ConnectionPayload {
  task_id: string;
  connection_id: string;
  message: string;
}

export interface HeartbeatPayload {
  server_time: number;
  active_connections: number;
}

export interface HandsPayload {
  thought?: string;
  action?: string;
  target?: string;
}

export interface McpEventPayload {
  type: 'tooldiscovery' | 'toolstart' | 'toolprogress' | 'toolresult' | 'toolerror';
  tool?: string;
  id?: string;
  input?: Record<string, unknown>;
  progress?: Record<string, unknown>;
  result?: Record<string, unknown>;
  error?: string;
}

export interface TaskPayload {
  status: 'pending' | 'running' | 'completed' | 'failed';
  step?: number;
  max_steps?: number;
  output?: string;
}

export interface StatsPayload {
  active_connections: number;
  total_connections: number;
  events: {
    thought: number;
    tool_call: number;
    tool_progress: number;
    tool_result: number;
    approval_needed: number;
    error: number;
    complete: number;
    // Phase 1: Additional event types
    session_start: number;
    session_end: number;
    checkpoint: number;
    user_intervention: number;
    total: number;
  };
  errors: {
    auth: number;
    replay: number;
    internal: number;
    total: number;
  };
  // Phase 1: Performance metrics
  performance: {
    connection_duration_total_ms: number;
    events_per_second_sum: number;
    avg_connection_duration_ms: number;
  };
}

// Phase 1: New payload types for richer events
export interface SessionStartPayload {
  session_id: string;
  task_id: string;
  started_at: number;
}

export interface SessionEndPayload {
  session_id: string;
  task_id: string;
  ended_at: number;
  duration_ms: number;
  final_status: 'completed' | 'failed' | 'cancelled';
}

export interface CheckpointPayload {
  session_id: string;
  checkpoint_id: string;
  step: number;
  timestamp: number;
  state_summary?: Record<string, unknown>;
}

export interface UserInterventionPayload {
  session_id: string;
  intervention_type: 'approval' | 'input' | 'choice';
  message: string;
  options?: string[];
  pending: boolean;
}

// ============================================================================
// Stats Panel Component
// ============================================================================

interface StatsPanelProps {
  endpoint: string;
}

const StatsPanel: React.FC<StatsPanelProps> = ({ endpoint }) => {
  const { events, connected, error } = useStreaming(endpoint);

  const stats: StatsPayload | null = events
    .filter((e) => e.type === 'stats')
    .map((e) => e.payload as StatsPayload)
    .pop() || null;

  return (
    <div 
      className="p-4 bg-gray-800 rounded-lg" 
      role="region" 
      aria-label="Streaming Statistics Panel"
    >
      <h3 className="text-lg font-semibold mb-4">Streaming Stats</h3>
      <div 
        className="flex items-center mb-4" 
        role="status" 
        aria-live="polite"
      >
        <span
          className={`w-3 h-3 rounded-full mr-2 ${
            connected ? 'bg-green-500' : error ? 'bg-red-500' : 'bg-gray-500'
          }`}
          aria-label={connected ? 'Connected' : error ? 'Error' : 'Disconnected'}
        />
        <span className="text-sm">{connected ? 'Connected' : error ? 'Error' : 'Disconnected'}</span>
      </div>
      {error && (
        <div 
          className="p-2 mb-4 bg-red-900/50 border border-red-500 rounded text-red-200 text-sm"
          role="alert"
        >
          Connection error. Attempting to reconnect...
        </div>
      )}
      {stats ? (
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span>Active Connections:</span>
            <span className="font-mono">{stats.active_connections}</span>
          </div>
          <div className="flex justify-between">
            <span>Total Connections:</span>
            <span className="font-mono">{stats.total_connections}</span>
          </div>
          <div className="flex justify-between">
            <span>Total Events:</span>
            <span className="font-mono">{stats.events.total}</span>
          </div>
          <div className="flex justify-between">
            <span>Total Errors:</span>
            <span className="font-mono text-red-400">{stats.errors.total}</span>
          </div>
        </div>
      ) : (
        <p className="text-gray-400 text-sm" aria-live="polite">Waiting for stats...</p>
      )}
    </div>
  );
};

// ============================================================================
// Hands Panel Component
// ============================================================================

interface HandsPanelProps {
  taskId: string;
}

const HandsPanel: React.FC<HandsPanelProps> = ({ taskId }) => {
  const endpoint = `/stream/hands/${taskId}`;
  const { events, connected, error } = useStreaming(endpoint);

  const handsEvents = events.filter((e) => e.type === 'hands');

  return (
    <div 
      className="p-4 bg-gray-800 rounded-lg"
      role="region" 
      aria-label={`Hands Agent Panel for task ${taskId}`}
    >
      <h3 className="text-lg font-semibold mb-4">Hands Agent</h3>
      <div 
        className="flex items-center mb-4" 
        role="status" 
        aria-live="polite"
      >
        <span
          className={`w-3 h-3 rounded-full mr-2 ${
            connected ? 'bg-green-500' : error ? 'bg-red-500' : 'bg-gray-500'
          }`}
        />
        <span className="text-sm">{connected ? 'Connected' : error ? 'Error' : 'Disconnected'}</span>
      </div>
      {error && (
        <div 
          className="p-2 mb-4 bg-red-900/50 border border-red-500 rounded text-red-200 text-sm"
          role="alert"
        >
          Connection error. Attempting to reconnect...
        </div>
      )}
      <div 
        className="space-y-2 max-h-64 overflow-y-auto"
        role="log" 
        aria-label="Hands agent events"
        aria-live="polite"
      >
        {handsEvents.length === 0 ? (
          <p className="text-gray-400 text-sm">Waiting for hands events...</p>
        ) : (
          handsEvents.map((event, idx) => {
            const payload = event.payload as HandsPayload;
            return (
              <div key={idx} className="p-2 bg-gray-700 rounded text-sm">
                {payload.thought && <p className="text-cyan-400">{payload.thought}</p>}
                {payload.action && (
                  <p className="text-yellow-400">
                    Action: {payload.action}
                    {payload.target && ` -> ${payload.target}`}
                  </p>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};

// ============================================================================
// MCP Panel Component
// ============================================================================

interface McpPanelProps {
  taskId: string;
}

const McpPanel: React.FC<McpPanelProps> = ({ taskId }) => {
  const endpoint = `/stream/mcp/${taskId}`;
  const { events, connected, error } = useStreaming(endpoint);

  const mcpEvents = events.filter((e) => e.type === 'mcp');

  return (
    <div 
      className="p-4 bg-gray-800 rounded-lg"
      role="region" 
      aria-label={`MCP Tools Panel for task ${taskId}`}
    >
      <h3 className="text-lg font-semibold mb-4">MCP Tools</h3>
      <div 
        className="flex items-center mb-4" 
        role="status" 
        aria-live="polite"
      >
        <span
          className={`w-3 h-3 rounded-full mr-2 ${
            connected ? 'bg-green-500' : error ? 'bg-red-500' : 'bg-gray-500'
          }`}
        />
        <span className="text-sm">{connected ? 'Connected' : error ? 'Error' : 'Disconnected'}</span>
      </div>
      {error && (
        <div 
          className="p-2 mb-4 bg-red-900/50 border border-red-500 rounded text-red-200 text-sm"
          role="alert"
        >
          Connection error. Attempting to reconnect...
        </div>
      )}
      <div 
        className="space-y-2 max-h-64 overflow-y-auto"
        role="log" 
        aria-label="MCP tool events"
        aria-live="polite"
      >
        {mcpEvents.length === 0 ? (
          <p className="text-gray-400 text-sm">Waiting for MCP events...</p>
        ) : (
          mcpEvents.map((event, idx) => {
            const payload = event.payload as McpEventPayload;
            return (
              <div key={idx} className="p-2 bg-gray-700 rounded text-sm">
                <span className="text-purple-400 uppercase text-xs">{payload.type}</span>
                {payload.tool && <p className="text-white">Tool: {payload.tool}</p>}
                {payload.result && (
                  <pre className="text-green-400 text-xs mt-1">
                    {JSON.stringify(payload.result, null, 2)}
                  </pre>
                )}
                {payload.error && <p className="text-red-400">Error: {payload.error}</p>}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};

// ============================================================================
// Task Panel Component
// ============================================================================

interface TaskPanelProps {
  taskId: string;
}

const TaskPanel: React.FC<TaskPanelProps> = ({ taskId }) => {
  const endpoint = `/stream/task/${taskId}`;
  const { events, connected, error } = useStreaming(endpoint);

  const taskEvents = events.filter((e) => e.type === 'task' || e.type === 'heartbeat');

  return (
    <div 
      className="p-4 bg-gray-800 rounded-lg"
      role="region" 
      aria-label={`Task Execution Panel for task ${taskId}`}
    >
      <h3 className="text-lg font-semibold mb-4">Task Execution</h3>
      <div 
        className="flex items-center mb-4" 
        role="status" 
        aria-live="polite"
      >
        <span
          className={`w-3 h-3 rounded-full mr-2 ${
            connected ? 'bg-green-500' : error ? 'bg-red-500' : 'bg-gray-500'
          }`}
        />
        <span className="text-sm">{connected ? 'Connected' : error ? 'Error' : 'Disconnected'}</span>
      </div>
      {error && (
        <div 
          className="p-2 mb-4 bg-red-900/50 border border-red-500 rounded text-red-200 text-sm"
          role="alert"
        >
          Connection error. Attempting to reconnect...
        </div>
      )}
      <div 
        className="space-y-2 max-h-64 overflow-y-auto"
        role="log" 
        aria-label="Task execution events"
        aria-live="polite"
      >
        {taskEvents.length === 0 ? (
          <p className="text-gray-400 text-sm">Waiting for task events...</p>
        ) : (
          taskEvents.map((event, idx) => {
            const payload = event.payload as TaskPayload | HeartbeatPayload;
            if (event.type === 'heartbeat') {
              const hb = payload as HeartbeatPayload;
              return (
                <div key={idx} className="p-2 bg-gray-700 rounded text-sm">
                  <span className="text-blue-400">Heartbeat</span>
                  <p className="text-gray-400 text-xs">
                    Server: {new Date(hb.server_time).toISOString()} | Active: {hb.active_connections}
                  </p>
                </div>
              );
            }
            const tp = payload as TaskPayload;
            return (
              <div key={idx} className="p-2 bg-gray-700 rounded text-sm">
                <span
                  className={`uppercase text-xs ${
                    tp.status === 'completed'
                      ? 'text-green-400'
                      : tp.status === 'failed'
                      ? 'text-red-400'
                      : tp.status === 'running'
                      ? 'text-yellow-400'
                      : 'text-gray-400'
                  }`}
                >
                  {tp.status}
                </span>
                {tp.step !== undefined && tp.max_steps !== undefined && (
                  <p className="text-white">
                    Step {tp.step} / {tp.max_steps}
                  </p>
                )}
                {tp.output && <p className="text-gray-300 mt-1">{tp.output}</p>}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};

// ============================================================================
// Main StreamingDashboard Component
// ============================================================================

interface StreamingDashboardProps {
  taskId?: string;
}

interface TaskItem {
  task_id: string;
  status: string;
  created_at?: string;
}

// Sample task IDs for demo (in production, fetch from API)
const SAMPLE_TASKS = [
  { task_id: 'default', status: 'demo', created_at: new Date().toISOString() },
];

export const StreamingDashboard: React.FC<StreamingDashboardProps> = ({ taskId: initialTaskId }) => {
  const [activeTab, setActiveTab] = useState<'stats' | 'hands' | 'mcp' | 'task'>('stats');
  const [selectedTaskId, setSelectedTaskId] = useState<string>(initialTaskId || 'default');
  const [availableTasks, setAvailableTasks] = useState<TaskItem[]>(SAMPLE_TASKS);
  const [tasksLoading, setTasksLoading] = useState(false);

  // Fetch available tasks from API
  useEffect(() => {
    const fetchTasks = async () => {
      setTasksLoading(true);
      try {
        const response = await apiGet('/api/v1/tasks?limit=20');
        if (response.ok) {
          const data = await response.json();
          // Handle different response formats
          const tasks = Array.isArray(data) ? data : (data.tasks || []);
          setAvailableTasks([
            { task_id: 'default', status: 'demo', created_at: new Date().toISOString() },
            ...tasks.slice(0, 19).map((t: { task_id: string; status: string; created_at?: string }) => ({
              task_id: t.task_id,
              status: t.status,
              created_at: t.created_at,
            })),
          ]);
        }
      } catch (err) {
        console.warn('Failed to fetch tasks:', err);
      } finally {
        setTasksLoading(false);
      }
    };

    fetchTasks();
  }, []);

  return (
    <div className="h-full flex flex-col bg-gray-900 text-white">
      {/* Header */}
      <div className="p-4 border-b border-gray-700">
        <h2 className="text-xl font-bold">Streaming Dashboard</h2>
        <p className="text-gray-400 text-sm">Real-time agent execution monitoring</p>
      </div>

      {/* Task Selector */}
      <div className="p-4 border-b border-gray-700 flex items-center gap-4">
        <label htmlFor="task-selector" className="text-sm font-medium text-gray-300">
          Monitor Task:
        </label>
        <select
          id="task-selector"
          value={selectedTaskId}
          onChange={(e) => setSelectedTaskId(e.target.value)}
          disabled={tasksLoading}
          className="px-3 py-2 bg-gray-800 border border-gray-600 rounded text-white text-sm focus:outline-none focus:border-cyan-500 disabled:opacity-50"
        >
          {availableTasks.map((task) => (
            <option key={task.task_id} value={task.task_id}>
              {task.task_id.slice(0, 12)}... ({task.status})
            </option>
          ))}
        </select>
        <span className="text-xs text-gray-500">
          {tasksLoading ? 'Loading...' : `${availableTasks.length} tasks`}
        </span>
      </div>

      {/* Tabs */}
      <div className="flex border-b border-gray-700">
        {(['stats', 'hands', 'mcp', 'task'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-6 py-3 text-sm font-medium uppercase ${
              activeTab === tab
                ? 'border-b-2 border-cyan-500 text-cyan-400'
                : 'text-gray-400 hover:text-white'
            }`}
          >
            {tab}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 p-4 overflow-auto">
        {activeTab === 'stats' && <StatsPanel endpoint="/stream/stats" />}
        {activeTab === 'hands' && <HandsPanel taskId={selectedTaskId} />}
        {activeTab === 'mcp' && <McpPanel taskId={selectedTaskId} />}
        {activeTab === 'task' && <TaskPanel taskId={selectedTaskId} />}
      </div>

      {/* Footer */}
      <div className="p-2 border-t border-gray-700 text-xs text-gray-500">
        APEX Streaming v1.7.0 | SSE Events | Heartbeat: 30s
      </div>
    </div>
  );
};

export default StreamingDashboard;
