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
        <div className="text-[var(--color-text-muted)] flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
            <line x1="12" y1="2" x2="12" y2="6"></line>
            <line x1="12" y1="18" x2="12" y2="22"></line>
            <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
            <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
            <line x1="2" y1="12" x2="6" y2="12"></line>
            <line x1="18" y1="12" x2="22" y2="12"></line>
            <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
            <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
          </svg>
          Loading audit log...
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-[var(--color-border)] p-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
                <line x1="16" y1="13" x2="8" y2="13"></line>
                <line x1="16" y1="17" x2="8" y2="17"></line>
                <polyline points="10 9 9 9 8 9"></polyline>
              </svg>
            </div>
            <div>
              <h2 className="text-xl font-semibold">Audit Log</h2>
              <p className="text-sm text-[var(--color-text-muted)]">Immutable hash chain of all system actions</p>
            </div>
          </div>
          <button
            onClick={exportCsv}
            className="px-4 py-2 border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            Export CSV
          </button>
        </div>
      </div>

      {/* Chain Status */}
      {chainStatus && (
        <div className="border-b border-[var(--color-border)] p-4 bg-[var(--color-muted)]/20">
          <div className="flex items-center gap-4 text-sm">
            <div className="flex items-center gap-2">
              <span className="text-[var(--color-text-muted)]">Chain Status:</span>
              {chainStatus.valid ? (
                <span className="text-green-500 font-medium flex items-center gap-1">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                    <polyline points="22 4 12 14.01 9 11.01"></polyline>
                  </svg>
                  Verified
                </span>
              ) : (
                <span className="text-red-500 font-medium flex items-center gap-1">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
                    <line x1="12" y1="9" x2="12" y2="13"></line>
                    <line x1="12" y1="17" x2="12.01" y2="17"></line>
                  </svg>
                  Invalid
                </span>
              )}
            </div>
            <div className="text-[var(--color-text-muted)]">|</div>
            <div className="text-[var(--color-text-muted)]">
              Total Entries: <span className="font-medium text-[var(--color-text)]">{chainStatus.total_entries}</span>
            </div>
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="border-b border-[var(--color-border)] p-4">
        <div className="flex gap-2">
          {filters.map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id)}
              className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
                filter === f.id
                  ? 'bg-[#4248f1] text-white'
                  : 'border border-[var(--color-border)] hover:bg-[var(--color-muted)]'
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      {/* Table */}
      <div className="flex-1 overflow-auto">
        {filteredEntries.length === 0 ? (
          <div className="text-center text-[var(--color-text-muted)] py-12">
            No audit entries found
          </div>
        ) : (
          <table className="w-full">
            <thead className="border-b border-[var(--color-border)] bg-[var(--color-muted)]/30 sticky top-0">
              <tr>
                <th className="text-left px-4 py-3 text-sm font-medium text-[var(--color-text-muted)]">Timestamp</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-[var(--color-text-muted)]">Action</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-[var(--color-text-muted)]">Entity Type</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-[var(--color-text-muted)]">Entity ID</th>
                <th className="text-left px-4 py-3 text-sm font-medium text-[var(--color-text-muted)]">Details</th>
              </tr>
            </thead>
            <tbody>
              {filteredEntries.map((entry, idx) => (
                <tr 
                  key={entry.id || idx} 
                  className="border-b border-[var(--color-border)] hover:bg-[var(--color-muted)]/30"
                >
                  <td className="px-4 py-3 text-sm text-[var(--color-text)]">
                    {formatDate(entry.timestamp)}
                  </td>
                  <td className="px-4 py-3">
                    <span className={`text-xs px-2 py-0.5 rounded font-medium ${
                      entry.action.includes('created') ? 'bg-green-500/10 text-green-500 border border-green-500/20' :
                      entry.action.includes('deleted') ? 'bg-red-500/10 text-red-500 border border-red-500/20' :
                      entry.action.includes('updated') ? 'bg-[#4248f1]/10 text-[#4248f1] border border-[#4248f1]/20' :
                      'bg-[var(--color-muted)] text-[var(--color-text-muted)]'
                    }`}>
                      {entry.action}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-text)]">
                    {entry.entity_type}
                  </td>
                  <td className="px-4 py-3 text-sm font-mono text-[var(--color-text-muted)]">
                    {entry.entity_id.slice(0, 16)}...
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-text-muted)] max-w-xs truncate">
                    {entry.details || '-'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Footer */}
      <div className="border-t border-[var(--color-border)] p-4 text-sm text-[var(--color-text-muted)]">
        Showing {filteredEntries.length} entries
      </div>
    </div>
  );
}
