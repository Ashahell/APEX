import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface AuditEntry {
  id: string;
  timestamp: string;
  created_at?: string;
  action: string;
  skill_name?: string;
  tier: string;
  status: string;
  task_id: string;
  user: string;
  details: string | null;
}

type AuditFilter = 'all' | 't0' | 't1' | 't2' | 't3';

export function AuditLog() {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<AuditFilter>('all');

  useEffect(() => {
    loadAuditLog();
  }, []);

  const loadAuditLog = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/tasks?limit=100');
      const data = await res.json();
      setEntries(data);
    } catch (err) {
      console.error('Failed to load audit log:', err);
    } finally {
      setLoading(false);
    }
  };

  const filteredEntries = entries.filter(entry => {
    if (filter === 'all') return true;
    return entry.tier?.toLowerCase() === filter;
  });

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const filters: { id: AuditFilter; label: string }[] = [
    { id: 'all', label: 'All' },
    { id: 't0', label: 'T0' },
    { id: 't1', label: 'T1' },
    { id: 't2', label: 'T2' },
    { id: 't3', label: 'T3' },
  ];

  const exportCsv = () => {
    const headers = ['Timestamp', 'Action', 'Tier', 'Status', 'Task ID'];
    const rows = filteredEntries.map(e => [
      e.created_at || e.timestamp,
      e.action || e.skill_name || 'task',
      e.tier || 'N/A',
      e.status,
      e.task_id,
    ]);
    const csv = [headers, ...rows].map(r => r.join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `audit-log-${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading audit log...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold mb-1">Audit Log</h2>
            <p className="text-muted-foreground text-sm">Track all actions and security events</p>
          </div>
          <button
            onClick={exportCsv}
            className="px-4 py-2 border rounded-lg hover:bg-muted"
          >
            Export CSV
          </button>
        </div>
      </div>

      <div className="border-b p-4">
        <div className="flex gap-2">
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id)}
              className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
                filter === f.id
                  ? 'bg-primary text-primary-foreground'
                  : 'border hover:bg-muted'
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        {filteredEntries.length === 0 ? (
          <div className="text-center text-muted-foreground py-12">
            No audit entries found
          </div>
        ) : (
          <table className="w-full">
            <thead className="border-b bg-muted/50 sticky top-0">
              <tr>
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Timestamp</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Action</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Tier</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Status</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Task</th>
              </tr>
            </thead>
            <tbody>
              {filteredEntries.map((entry, idx) => (
                <tr 
                  key={entry.task_id || idx} 
                  className="border-b hover:bg-muted/50"
                >
                  <td className="px-4 py-3 text-sm">
                    {formatDate(entry.created_at || entry.timestamp)}
                  </td>
                  <td className="px-4 py-3 text-sm">
                    {entry.skill_name || entry.action || 'task'}
                  </td>
                  <td className="px-4 py-3">
                    {entry.tier && (
                      <span className={`text-xs px-2 py-0.5 rounded ${
                        entry.tier === 'T0' ? 'bg-green-100 text-green-800' :
                        entry.tier === 'T1' ? 'bg-blue-100 text-blue-800' :
                        entry.tier === 'T2' ? 'bg-orange-100 text-orange-800' :
                        'bg-red-100 text-red-800'
                      }`}>
                        {entry.tier}
                      </span>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <span className={`text-xs px-2 py-0.5 rounded ${
                      entry.status === 'completed' ? 'bg-green-100 text-green-800' :
                      entry.status === 'failed' ? 'bg-red-100 text-red-800' :
                      entry.status === 'running' ? 'bg-blue-100 text-blue-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {entry.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm font-mono text-muted-foreground">
                    {entry.task_id?.slice(0, 12)}...
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className="border-t p-4 text-sm text-muted-foreground">
        Showing {filteredEntries.length} entries
      </div>
    </div>
  );
}
