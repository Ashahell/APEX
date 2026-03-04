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
        <div className="text-muted-foreground">Loading memories...</div>
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
              {activeTab === tab.id && (
                <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
              )}
            </button>
          ))}
        </nav>
      </div>

      <div className="p-4 border-b">
        <div className="flex items-center gap-4">
          <input
            type="text"
            placeholder="Search memories..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 px-3 py-2 rounded-lg border bg-background"
          />
          <button
            onClick={loadMemories}
            className="px-4 py-2 rounded-lg border hover:bg-muted"
          >
            Refresh
          </button>
        </div>
        <p className="text-sm text-muted-foreground mt-2">
          {tabs.find(t => t.id === activeTab)?.description}
        </p>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {filteredMemories.length === 0 ? (
          <div className="text-center text-muted-foreground py-8">
            No {activeTab} memories found
          </div>
        ) : (
          <div className="space-y-2">
            {filteredMemories.map((memory) => (
              <div 
                key={memory.task_id} 
                className="border rounded-lg p-3 hover:bg-muted/50 transition-colors"
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-xs text-muted-foreground font-mono">
                    {memory.task_id.slice(0, 8)}...
                  </span>
                  <span className="text-xs text-muted-foreground">
                    {formatDate(memory.created_at)}
                  </span>
                </div>
                <p className="text-sm mb-2 line-clamp-2">
                  {memory.input_content?.slice(0, 150) || '(No input)'}
                  {(memory.input_content?.length || 0) > 150 && '...'}
                </p>
                {memory.output_content && (
                  <p className="text-xs text-muted-foreground line-clamp-2">
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
                <div className="flex items-center gap-2 mt-2">
                  <span className={`text-xs px-2 py-0.5 rounded ${
                    memory.status === 'completed'
                      ? 'bg-green-500/20 text-green-500'
                      : memory.status === 'failed'
                      ? 'bg-red-500/20 text-red-500'
                      : 'bg-yellow-500/20 text-yellow-500'
                  }`}>
                    {memory.status}
                  </span>
                  {memory.project && (
                    <span className="text-xs text-blue-500">{memory.project}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="p-4 border-t text-xs text-muted-foreground">
        Showing {filteredMemories.length} of {memories.length} total entries
      </div>
    </div>
  );
}
