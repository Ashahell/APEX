import { useState, useEffect, useCallback } from 'react';
import { apiGet, apiPost, apiPut, apiDelete } from '../../lib/api';

// ============ Types ============

export interface WatchPattern {
  id: string;
  name: string;
  pattern: string;
  watch_scope: { type: 'All' | 'Project' | 'TaskIds'; Project?: string; TaskIds?: string[] };
  notify_on: { type: 'Match' | 'Completion' | 'Error' | 'Timeout' | 'Threshold'; count?: number };
  notification_mode: 'All' | 'Result' | 'Error' | 'Off';
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface WatchPatternUpdate {
  name?: string;
  pattern?: string;
  watch_scope?: WatchPattern['watch_scope'];
  notify_on?: WatchPattern['notify_on'];
  notification_mode?: WatchPattern['notification_mode'];
  enabled?: boolean;
}

export interface MonitorEvent {
  id: string;
  event_type: string;
  task_id?: string;
  session_id?: string;
  payload: string;
  matched_watcher_id?: string;
  created_at: string;
}

export interface MonitoringStats {
  total_watchers: number;
  active_watchers: number;
  events_last_hour: number;
  patterns_matched: number;
  notifications_sent: number;
}

// ============ Component ============

export function MonitoringDashboard() {
  const [patterns, setPatterns] = useState<WatchPattern[]>([]);
  const [events, setEvents] = useState<MonitorEvent[]>([]);
  const [stats, setStats] = useState<MonitoringStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [activeView, setActiveView] = useState<'patterns' | 'events' | 'create'>('patterns');
  const [taskFilter, setTaskFilter] = useState('');
  
  // Create form state
  const [newPattern, setNewPattern] = useState<Partial<WatchPattern>>({
    name: '',
    pattern: '',
    watch_scope: { type: 'All' },
    notify_on: { type: 'Match' },
    notification_mode: 'Result',
    enabled: true,
  });

  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [patternsRes, eventsRes, statsRes] = await Promise.all([
        apiGet('/api/v1/monitor/watchers'),
        apiGet('/api/v1/monitor/events?limit=50'),
        apiGet('/api/v1/monitor/stats'),
      ]);
      
      if (patternsRes.ok) {
        const data = await patternsRes.json();
        setPatterns(Array.isArray(data) ? data : []);
      }
      if (eventsRes.ok) {
        const data = await eventsRes.json();
        setEvents(Array.isArray(data) ? data : []);
      }
      if (statsRes.ok) {
        const data = await statsRes.json();
        setStats(data);
      }
    } catch (err) {
      console.error('Failed to load monitoring data:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  const handleCreatePattern = async () => {
    if (!newPattern.name || !newPattern.pattern) return;
    
    try {
      const res = await apiPost('/api/v1/monitor/watchers', {
        name: newPattern.name,
        pattern: newPattern.pattern,
        watch_scope: newPattern.watch_scope,
        notify_on: newPattern.notify_on,
        notification_mode: newPattern.notification_mode,
        enabled: newPattern.enabled,
      });
      
      if (res.ok) {
        setNewPattern({
          name: '',
          pattern: '',
          watch_scope: { type: 'All' },
          notify_on: { type: 'Match' },
          notification_mode: 'Result',
          enabled: true,
        });
        setActiveView('patterns');
        loadData();
      }
    } catch (err) {
      console.error('Failed to create pattern:', err);
    }
  };

  const handleTogglePattern = async (id: string, enabled: boolean) => {
    try {
      await apiPut(`/api/v1/monitor/watchers/${id}`, { enabled });
      loadData();
    } catch (err) {
      console.error('Failed to toggle pattern:', err);
    }
  };

  const handleDeletePattern = async (id: string) => {
    try {
      await apiDelete(`/api/v1/monitor/watchers/${id}`);
      loadData();
    } catch (err) {
      console.error('Failed to delete pattern:', err);
    }
  };

  const filteredEvents = taskFilter
    ? events.filter(e => e.task_id?.includes(taskFilter))
    : events;

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading monitoring data...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-semibold">Background Process Monitoring</h2>
        <p className="text-[var(--color-text-muted)]">
          Watch patterns, track events, and monitor agent behavior
        </p>
      </div>

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-5 gap-4">
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold">{stats.total_watchers}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Total Patterns</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-green-400">{stats.active_watchers}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Active</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold">{stats.events_last_hour}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Events/Hour</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-yellow-400">{stats.patterns_matched}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Matched</div>
          </div>
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <div className="text-2xl font-bold text-blue-400">{stats.notifications_sent}</div>
            <div className="text-sm text-[var(--color-text-muted)]">Notifications</div>
          </div>
        </div>
      )}

      {/* View Tabs */}
      <div className="flex gap-4 border-b">
        <button
          onClick={() => setActiveView('patterns')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'patterns'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          Watch Patterns
        </button>
        <button
          onClick={() => setActiveView('events')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'events'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          Event Log
        </button>
        <button
          onClick={() => setActiveView('create')}
          className={`px-4 py-2 transition-colors ${
            activeView === 'create'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          + New Pattern
        </button>
      </div>

      {/* Patterns View */}
      {activeView === 'patterns' && (
        <div className="space-y-4">
          {patterns.length === 0 ? (
            <div className="border rounded-lg p-8 text-center text-[var(--color-text-muted)]">
              No watch patterns configured. Create one to start monitoring.
            </div>
          ) : (
            <div className="space-y-2">
              {patterns.map((p) => (
                <div
                  key={p.id}
                  className="border rounded-lg p-4 bg-[var(--color-panel)] flex items-start gap-4"
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-2">
                      <span className={`px-2 py-0.5 rounded text-xs ${
                        p.enabled ? 'bg-green-500/20 text-green-400' : 'bg-gray-500/20 text-gray-400'
                      }`}>
                        {p.enabled ? 'Active' : 'Disabled'}
                      </span>
                      <h3 className="font-medium">{p.name}</h3>
                    </div>
                    <div className="text-sm text-[var(--color-text-muted)] font-mono bg-[var(--color-muted)] px-2 py-1 rounded mb-2">
                      {p.pattern}
                    </div>
                    <div className="flex gap-4 text-xs text-[var(--color-text-muted)]">
                      <span>Scope: <span className="text-[var(--color-text)]">{p.watch_scope.type}</span></span>
                      <span>Notify: <span className="text-[var(--color-text)]">{p.notify_on.type}</span></span>
                      <span>Mode: <span className="text-[var(--color-text)]">{p.notification_mode}</span></span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input
                        type="checkbox"
                        checked={p.enabled}
                        onChange={(e) => handleTogglePattern(p.id, e.target.checked)}
                        className="sr-only peer"
                      />
                      <div className="w-9 h-5 bg-[var(--color-muted)] rounded-full peer peer-checked:bg-[#00d4ff] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
                    </label>
                    <button
                      onClick={() => handleDeletePattern(p.id)}
                      className="p-1.5 text-red-400 hover:bg-red-500/20 rounded transition-colors"
                      title="Delete pattern"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <path d="M3 6h18"></path>
                        <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
                        <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
                      </svg>
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Events View */}
      {activeView === 'events' && (
        <div className="space-y-4">
          <div className="flex items-center gap-4">
            <input
              type="text"
              placeholder="Filter by task ID..."
              value={taskFilter}
              onChange={(e) => setTaskFilter(e.target.value)}
              className="px-3 py-2 rounded border bg-[var(--color-panel)] border-[var(--color-border)] w-64"
            />
            <button
              onClick={loadData}
              className="px-3 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors text-sm"
            >
              Refresh
            </button>
          </div>
          
          <div className="border rounded-lg overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-[var(--color-muted)]">
                <tr>
                  <th className="text-left p-3 font-medium">Time</th>
                  <th className="text-left p-3 font-medium">Event</th>
                  <th className="text-left p-3 font-medium">Task ID</th>
                  <th className="text-left p-3 font-medium">Payload</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[var(--color-border)]">
                {filteredEvents.length === 0 ? (
                  <tr>
                    <td colSpan={4} className="p-8 text-center text-[var(--color-text-muted)]">
                      No events recorded yet
                    </td>
                  </tr>
                ) : (
                  filteredEvents.map((event) => (
                    <tr key={event.id} className="hover:bg-[var(--color-panel)]">
                      <td className="p-3 text-[var(--color-text-muted)] font-mono text-xs">
                        {new Date(event.created_at).toLocaleTimeString()}
                      </td>
                      <td className="p-3">
                        <span className="px-2 py-0.5 bg-[#00d4ff]/20 text-[#00d4ff] rounded text-xs">
                          {event.event_type}
                        </span>
                      </td>
                      <td className="p-3 font-mono text-xs">
                        {event.task_id || '-'}
                      </td>
                      <td className="p-3 text-xs max-w-xs truncate text-[var(--color-text-muted)]">
                        {event.payload}
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Create Pattern View */}
      {activeView === 'create' && (
        <div className="border rounded-lg p-6 bg-[var(--color-panel)] space-y-4">
          <h3 className="font-semibold">Create Watch Pattern</h3>
          
          <div className="grid gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Name</label>
              <input
                type="text"
                value={newPattern.name || ''}
                onChange={(e) => setNewPattern({ ...newPattern, name: e.target.value })}
                placeholder="Error Detection"
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Pattern (Regex)</label>
              <input
                type="text"
                value={newPattern.pattern || ''}
                onChange={(e) => setNewPattern({ ...newPattern, pattern: e.target.value })}
                placeholder="\berror\b"
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)] font-mono"
              />
              <p className="text-xs text-[var(--color-text-muted)] mt-1">
                Use (?i) prefix for case-insensitive matching
              </p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-2">Watch Scope</label>
                <select
                  value={newPattern.watch_scope?.type || 'All'}
                  onChange={(e) => setNewPattern({
                    ...newPattern,
                    watch_scope: { type: e.target.value as 'All' | 'Project' | 'TaskIds' }
                  })}
                  className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
                >
                  <option value="All">All Tasks</option>
                  <option value="Project">Specific Project</option>
                  <option value="TaskIds">Specific Task IDs</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Notify On</label>
                <select
                  value={newPattern.notify_on?.type || 'Match'}
                  onChange={(e) => setNewPattern({
                    ...newPattern,
                    notify_on: { type: e.target.value as 'Match' | 'Completion' | 'Error' | 'Timeout' | 'Threshold' }
                  })}
                  className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
                >
                  <option value="Match">Pattern Match</option>
                  <option value="Completion">Task Completion</option>
                  <option value="Error">Task Error</option>
                  <option value="Timeout">Task Timeout</option>
                  <option value="Threshold">Threshold Count</option>
                </select>
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Notification Mode</label>
              <select
                value={newPattern.notification_mode || 'Result'}
                onChange={(e) => setNewPattern({
                  ...newPattern,
                  notification_mode: e.target.value as 'All' | 'Result' | 'Error' | 'Off'
                })}
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
              >
                <option value="All">All Events</option>
                <option value="Result">Final Result Only</option>
                <option value="Error">Errors Only</option>
                <option value="Off">Disabled</option>
              </select>
            </div>

            <div className="flex items-center gap-3 pt-2">
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={newPattern.enabled ?? true}
                  onChange={(e) => setNewPattern({ ...newPattern, enabled: e.target.checked })}
                  className="sr-only peer"
                />
                <div className="w-9 h-5 bg-[var(--color-muted)] rounded-full peer peer-checked:bg-[#00d4ff] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
              </label>
              <span className="text-sm">Enabled immediately</span>
            </div>
          </div>

          <div className="flex gap-3 pt-4">
            <button
              onClick={handleCreatePattern}
              disabled={!newPattern.name || !newPattern.pattern}
              className="px-4 py-2 bg-[#00d4ff] text-[#0f0f1a] rounded font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
            >
              Create Pattern
            </button>
            <button
              onClick={() => setActiveView('patterns')}
              className="px-4 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
