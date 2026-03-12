import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface MemoryStats {
  journal_entries: number;
  entities: number;
  knowledge_items: number;
  reflections: number;
  total_files: number;
}

interface NarrativeEntry {
  id: string;
  task_id: string;
  path: string;
  created_at: string;
  summary: string;
}

type MemoryTab = 'journal' | 'entities' | 'knowledge' | 'reflections';

export function NarrativeMemoryViewer() {
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [entries, setEntries] = useState<NarrativeEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<MemoryTab>('journal');
  const [selectedEntry, setSelectedEntry] = useState<NarrativeEntry | null>(null);
  const [entryContent, setEntryContent] = useState<string | null>(null);

  useEffect(() => {
    loadStats();
    loadEntries(activeTab);
  }, []);

  useEffect(() => {
    loadEntries(activeTab);
  }, [activeTab]);

  const loadStats = async () => {
    try {
      const res = await apiGet('/api/v1/memory/stats');
      if (res.ok) {
        const data = await res.json();
        setStats(data);
      }
    } catch (err) {
      console.error('Failed to load stats:', err);
    }
  };

  const loadEntries = async (tab: MemoryTab) => {
    setLoading(true);
    try {
      const res = await apiGet(`/api/v1/memory/${tab}`);
      if (res.ok) {
        const data = await res.json();
        setEntries(data);
      }
    } catch (err) {
      console.error('Failed to load entries:', err);
    } finally {
      setLoading(false);
    }
  };

  const viewEntry = async (entry: NarrativeEntry) => {
    setSelectedEntry(entry);
    try {
      const res = await apiGet(`/api/v1/memory/content?path=${encodeURIComponent(entry.path)}`);
      if (res.ok) {
        const data = await res.json();
        setEntryContent(data.content);
      }
    } catch (err) {
      console.error('Failed to load entry content:', err);
      setEntryContent('Failed to load content');
    }
  };

  const tabs: { id: MemoryTab; label: string; count: number }[] = [
    { id: 'journal', label: 'Journal', count: stats?.journal_entries || 0 },
    { id: 'entities', label: 'Entities', count: stats?.entities || 0 },
    { id: 'knowledge', label: 'Knowledge', count: stats?.knowledge_items || 0 },
    { id: 'reflections', label: 'Reflections', count: stats?.reflections || 0 },
  ];

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (selectedEntry && entryContent !== null) {
    return (
      <div className="flex flex-col h-full">
        {/* Header */}
        <div className="border-b border-[var(--color-border)] p-4">
          <button
            onClick={() => {
              setSelectedEntry(null);
              setEntryContent(null);
            }}
            className="flex items-center gap-2 text-sm text-[var(--color-text-muted)] hover:text-[#4248f1] transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="19" y1="12" x2="5" y2="12"></line>
              <polyline points="12 19 5 12 12 5"></polyline>
            </svg>
            Back to list
          </button>
        </div>
        {/* Content */}
        <div className="flex-1 overflow-auto p-4">
          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold">{selectedEntry.task_id}</h3>
              <span className="text-sm text-[var(--color-text-muted)]">
                {formatDate(selectedEntry.created_at)}
              </span>
            </div>
            <pre className="whitespace-pre-wrap text-sm font-mono bg-[var(--color-muted)]/30 p-4 rounded-lg border border-[var(--color-border)] overflow-auto">
              {entryContent}
            </pre>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Tabs */}
      <div className="border-b border-[var(--color-border)]">
        <nav className="flex">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium transition-colors relative ${
                activeTab === tab.id
                  ? 'text-[#4248f1]'
                  : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
              }`}
            >
              {tab.label}
              <span className="ml-2 text-xs bg-[var(--color-muted)] px-2 py-0.5 rounded-full">
                {tab.count}
              </span>
              {activeTab === tab.id && (
                <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-[#4248f1]" />
              )}
            </button>
          ))}
        </nav>
      </div>

      {/* Stats */}
      <div className="border-b border-[var(--color-border)] p-4 bg-[var(--color-panel)]">
        <div className="grid grid-cols-4 gap-3">
          <div className="border border-[var(--color-border)] rounded-lg p-3 text-center bg-[var(--color-background)]">
            <div className="text-2xl font-bold text-[#4248f1]">{stats?.total_files || 0}</div>
            <div className="text-xs text-[var(--color-text-muted)]">Total Files</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-lg p-3 text-center bg-[var(--color-background)]">
            <div className="text-2xl font-bold">{stats?.journal_entries || 0}</div>
            <div className="text-xs text-[var(--color-text-muted)]">Journal</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-lg p-3 text-center bg-[var(--color-background)]">
            <div className="text-2xl font-bold">{stats?.entities || 0}</div>
            <div className="text-xs text-[var(--color-text-muted)]">Entities</div>
          </div>
          <div className="border border-[var(--color-border)] rounded-lg p-3 text-center bg-[var(--color-background)]">
            <div className="text-2xl font-bold">{stats?.reflections || 0}</div>
            <div className="text-xs text-[var(--color-text-muted)]">Reflections</div>
          </div>
        </div>
      </div>

      {/* Entries List */}
      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="text-center text-[var(--color-text-muted)] py-8 flex items-center justify-center gap-2">
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
            Loading...
          </div>
        ) : entries.length === 0 ? (
          <div className="text-center text-[var(--color-text-muted)] py-8">
            No {activeTab} entries found
          </div>
        ) : (
          <div className="space-y-2">
            {entries.map((entry) => (
              <button
                key={entry.id}
                onClick={() => viewEntry(entry)}
                className="w-full border border-[var(--color-border)] rounded-lg p-3 hover:bg-[var(--color-muted)]/30 hover:border-[#4248f1]/30 transition-colors text-left"
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-medium">{entry.task_id}</span>
                  <span className="text-xs text-[var(--color-text-muted)]">
                    {formatDate(entry.created_at)}
                  </span>
                </div>
                <p className="text-sm text-[var(--color-text-muted)] line-clamp-2">
                  {entry.summary}
                </p>
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
