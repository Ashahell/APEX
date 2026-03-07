import { useState, useEffect } from 'react';
import { listAudit, getAuditChainStatus, AuditEntry, AuditChainStatus } from '../../lib/api';

type AuditFilter = 'all' | 'task' | 'workflow' | 'skill' | 'system';

export function AuditLog() {
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [chainStatus, setChainStatus] = useState<AuditChainStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState<AuditFilter>('all');

  useEffect(() => {
    loadAuditLog();
  }, []);

  const loadAuditLog = async () => {
    setLoading(true);
    try {
      const [data, status] = await Promise.all([
        listAudit(100, 0),
        getAuditChainStatus(),
      ]);
      setEntries(data);
      setChainStatus(status);
    } catch (err) {
      console.error('Failed to load audit log:', err);
    } finally {
      setLoading(false);
    }
  };

  const filteredEntries = entries.filter(entry => {
    if (filter === 'all') return true;
    return entry.entity_type?.toLowerCase() === filter;
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
    { id: 'task', label: 'Tasks' },
    { id: 'workflow', label: 'Workflows' },
    { id: 'skill', label: 'Skills' },
    { id: 'system', label: 'System' },
  ];

  const exportCsv = () => {
    const headers = ['Timestamp', 'Action', 'Entity Type', 'Entity ID', 'Details'];
    const rows = filteredEntries.map(e => [
      e.timestamp,
      e.action,
      e.entity_type,
      e.entity_id,
      e.details || '',
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
            <p className="text-muted-foreground text-sm">Immutable hash chain of all system actions</p>
          </div>
          <button
            onClick={exportCsv}
            className="px-4 py-2 border rounded-lg hover:bg-muted"
          >
            Export CSV
          </button>
        </div>
      </div>

      {chainStatus && (
        <div className="border-b p-4 bg-muted/30">
          <div className="flex items-center gap-4 text-sm">
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Chain Status:</span>
              {chainStatus.valid ? (
                <span className="text-green-600 font-medium">✓ Verified</span>
              ) : (
                <span className="text-red-600 font-medium">⚠ Invalid</span>
              )}
            </div>
            <div className="text-muted-foreground">|</div>
            <div className="text-muted-foreground">
              Total Entries: <span className="font-medium">{chainStatus.total_entries}</span>
            </div>
          </div>
        </div>
      )}

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
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Entity Type</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Entity ID</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-muted-foreground">Details</th>
              </tr>
            </thead>
            <tbody>
              {filteredEntries.map((entry, idx) => (
                <tr 
                  key={entry.id || idx} 
                  className="border-b hover:bg-muted/50"
                >
                  <td className="px-4 py-3 text-sm">
                    {formatDate(entry.timestamp)}
                  </td>
                  <td className="px-4 py-3">
                    <span className={`text-xs px-2 py-0.5 rounded ${
                      entry.action.includes('created') ? 'bg-green-100 text-green-800' :
                      entry.action.includes('deleted') ? 'bg-red-100 text-red-800' :
                      entry.action.includes('updated') ? 'bg-blue-100 text-blue-800' :
                      'bg-gray-100 text-gray-800'
                    }`}>
                      {entry.action}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm">
                    {entry.entity_type}
                  </td>
                  <td className="px-4 py-3 text-sm font-mono text-muted-foreground">
                    {entry.entity_id.slice(0, 16)}...
                  </td>
                  <td className="px-4 py-3 text-sm text-muted-foreground max-w-xs truncate">
                    {entry.details || '-'}
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
