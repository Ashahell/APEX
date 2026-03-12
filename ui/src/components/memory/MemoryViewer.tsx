import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface MemoryEntry {
  task_id: string;
  input_content: string;
  output_content: string | null;
  status: string;
  created_at: string;
  project: string | null;
  tier?: 'session' | 'project' | 'longterm';
}

type MemoryTab = 'session' | 'project' | 'longterm';

export function MemoryViewer() {
  const [memories, setMemories] = useState<MemoryEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<MemoryTab>('session');
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    loadMemories();
  }, []);

  const loadMemories = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/tasks?limit=100');
      const data = await res.json();
      setMemories(data);
    } catch (err) {
      console.error('Failed to load memories:', err);
    } finally {
      setLoading(false);
    }
  };

  const getFilteredMemories = () => {
    let filtered = memories;
    
    if (activeTab === 'session') {
      const oneHourAgo = Date.now() - 60 * 60 * 1000;
      filtered = filtered.filter(m => new Date(m.created_at).getTime() > oneHourAgo);
    } else if (activeTab === 'project') {
      const oneHourAgo = Date.now() - 24 * 60 * 60 * 1000;
      filtered = filtered.filter(m => {
        const time = new Date(m.created_at).getTime();
        return time > oneHourAgo && m.project;
      });
    } else {
      filtered = filtered.filter(m => !m.project);
    }

    if (searchQuery) {
      filtered = filtered.filter(m => 
        m.input_content?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        m.output_content?.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }

    return filtered;
  };

  const filteredMemories = getFilteredMemories();

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const tabs: { id: MemoryTab; label: string; description: string }[] = [
    { id: 'session', label: 'Session', description: 'Recent interactions (last hour)' },
    { id: 'project', label: 'Project', description: 'Project-specific memories' },
    { id: 'longterm', label: 'Long-term', description: 'Persistent knowledge' },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading memories...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-border p-4 bg-[var(--color-panel)]">
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-lg bg-[#4248f1]/10 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2a10 10 0 1 0 10 10H12V2z"></path>
              <path d="M12 2a10 10 0 0 1 10 10"></path>
              <circle cx="12" cy="12" r="4"></circle>
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold" style={{ color: '#4248f1' }}>Memory</h2>
            <p className="text-sm text-[var(--color-text-muted)]">View agent memories and knowledge</p>
          </div>
        </div>
        
        {/* Tabs */}
        <div className="flex gap-1 bg-[var(--color-muted)] p-1 rounded-lg w-fit">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'bg-[#4248f1] text-white'
                  : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Search */}
      <div className="p-4 border-b border-border bg-[var(--color-panel)]">
        <div className="flex items-center gap-3">
          <div className="flex-1 relative">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
              <circle cx="11" cy="11" r="8"></circle>
              <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
            <input
              type="text"
              placeholder="Search memories..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-3 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)]"
            />
          </div>
          <button
            onClick={loadMemories}
            className="px-4 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] hover:bg-[var(--color-muted)] transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            Refresh
          </button>
        </div>
        <p className="text-sm text-[var(--color-text-muted)] mt-2">
          {tabs.find(t => t.id === activeTab)?.description}
        </p>
      </div>

      {/* Memory List */}
      <div className="flex-1 overflow-auto p-4">
        {filteredMemories.length === 0 ? (
          <div className="text-center text-[var(--color-text-muted)] py-12">
            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="mx-auto opacity-50">
              <path d="M12 2a10 10 0 1 0 10 10H12V2z"></path>
              <path d="M12 2a10 10 0 0 1 10 10"></path>
              <circle cx="12" cy="12" r="4"></circle>
            </svg>
            <p className="mt-4">No {activeTab} memories found</p>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredMemories.map((memory) => (
              <div 
                key={memory.task_id} 
                className="border border-[var(--color-border)] rounded-xl p-4 hover:border-[#4248f1]/30 hover:bg-[#4248f1]/5 transition-all cursor-pointer"
              >
                <div className="flex items-center justify-between mb-3">
                  <span className="text-xs text-[var(--color-text-muted)] font-mono bg-[var(--color-muted)] px-2 py-1 rounded">
                    {memory.task_id.slice(0, 8)}...
                  </span>
                  <span className="text-xs text-[var(--color-text-muted)]">
                    {formatDate(memory.created_at)}
                  </span>
                </div>
                <p className="text-sm mb-2 line-clamp-2 text-[var(--color-text)]">
                  {memory.input_content?.slice(0, 150) || '(No input)'}
                  {(memory.input_content?.length || 0) > 150 && '...'}
                </p>
                {memory.output_content && (
                  <p className="text-xs text-[var(--color-text-muted)] line-clamp-2 mb-3">
                    → {(() => {
                      try {
                        const parsed = JSON.parse(memory.output_content);
                        return parsed.output?.slice(0, 100) || '';
                      } catch {
                        return memory.output_content.slice(0, 100);
                      }
                    })()}
                  </p>
                )}
                <div className="flex items-center gap-2">
                  <span className={`text-xs px-2.5 py-1 rounded-full font-medium ${
                    memory.status === 'completed'
                      ? 'bg-green-500/10 text-green-500 border border-green-500/20'
                      : memory.status === 'failed'
                      ? 'bg-red-500/10 text-red-500 border border-red-500/20'
                      : 'bg-amber-500/10 text-amber-500 border border-amber-500/20'
                  }`}>
                    {memory.status}
                  </span>
                  {memory.project && (
                    <span className="text-xs px-2.5 py-1 rounded-full bg-[#4248f1]/10 text-[#4248f1] border border-[#4248f1]/20">
                      {memory.project}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="p-4 border-t bg-[var(--color-panel)] text-xs text-[var(--color-text-muted)]">
        Showing {filteredMemories.length} of {memories.length} total entries
      </div>
    </div>
  );
}
