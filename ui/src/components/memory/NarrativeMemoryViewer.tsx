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
        <div className="border-b p-4">
          <button
            onClick={() => {
              setSelectedEntry(null);
              setEntryContent(null);
            }}
            className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
          >
            ← Back to list
          </button>
        </div>
        <div className="flex-1 overflow-auto p-4">
          <div className="border rounded-lg p-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold">{selectedEntry.task_id}</h3>
              <span className="text-sm text-muted-foreground">
                {formatDate(selectedEntry.created_at)}
              </span>
            </div>
            <pre className="whitespace-pre-wrap text-sm font-mono bg-muted p-4 rounded-lg overflow-auto">
              {entryContent}
            </pre>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b">
        <nav className="flex">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-3 text-sm font-medium transition-colors relative ${
                activeTab === tab.id
                  ? 'text-primary'
                  : 'text-muted-foreground hover:text-foreground'
              }`}
            >
              {tab.label}
              <span className="ml-2 text-xs bg-muted px-2 py-0.5 rounded-full">
                {tab.count}
              </span>
              {activeTab === tab.id && (
                <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
              )}
            </button>
          ))}
        </nav>
      </div>

      <div className="p-4 border-b">
        <div className="grid grid-cols-4 gap-4">
          <div className="border rounded-lg p-3 text-center">
            <div className="text-2xl font-bold">{stats?.total_files || 0}</div>
            <div className="text-xs text-muted-foreground">Total Files</div>
          </div>
          <div className="border rounded-lg p-3 text-center">
            <div className="text-2xl font-bold">{stats?.journal_entries || 0}</div>
            <div className="text-xs text-muted-foreground">Journal</div>
          </div>
          <div className="border rounded-lg p-3 text-center">
            <div className="text-2xl font-bold">{stats?.entities || 0}</div>
            <div className="text-xs text-muted-foreground">Entities</div>
          </div>
          <div className="border rounded-lg p-3 text-center">
            <div className="text-2xl font-bold">{stats?.reflections || 0}</div>
            <div className="text-xs text-muted-foreground">Reflections</div>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="text-center text-muted-foreground py-8">
            Loading...
          </div>
        ) : entries.length === 0 ? (
          <div className="text-center text-muted-foreground py-8">
            No {activeTab} entries found
          </div>
        ) : (
          <div className="space-y-2">
            {entries.map((entry) => (
              <button
                key={entry.id}
                onClick={() => viewEntry(entry)}
                className="w-full border rounded-lg p-3 hover:bg-muted/50 transition-colors text-left"
              >
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-medium">{entry.task_id}</span>
                  <span className="text-xs text-muted-foreground">
                    {formatDate(entry.created_at)}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground line-clamp-2">
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
