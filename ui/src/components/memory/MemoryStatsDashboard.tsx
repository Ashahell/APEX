import { useState, useEffect } from 'react';
import { apiFetch } from '../../lib/api';

interface MemoryStats {
  total_entities: number;
  total_knowledge: number;
  total_reflections: number;
  recent_reflections: Array<{
    id: number;
    content: string;
    importance: number;
    created_at: string;
  }>;
}

export function MemoryStatsDashboard() {
  const [stats, setStats] = useState<MemoryStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    apiFetch('/api/v1/memory/stats')
      .then((res) => {
        if (!res.ok) throw new Error('Failed to fetch memory stats');
        return res.json();
      })
      .then((data) => {
        setStats(data);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message);
        setLoading(false);
      });
  }, []);

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading memory stats...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 flex items-center justify-center h-full">
        <div className="text-red-500">Error: {error}</div>
      </div>
    );
  }

  if (!stats) return null;

  const maxImportance = Math.max(...stats.recent_reflections.map(r => r.importance), 1);

  return (
    <div className="p-4 h-full overflow-y-auto">
      <div className="mb-6">
        <h2 className="text-2xl font-semibold">Memory Dashboard</h2>
        <p className="text-muted-foreground">Agent memory statistics and insights</p>
      </div>

      <div className="grid gap-4 md:grid-cols-3 mb-6">
        <div className="bg-gradient-to-br from-blue-500/20 to-blue-600/10 border rounded-xl p-4">
          <div className="text-sm text-muted-foreground mb-1">Entities</div>
          <div className="text-3xl font-bold">{stats.total_entities}</div>
          <div className="text-xs text-muted-foreground mt-1">Stored memories</div>
        </div>
        
        <div className="bg-gradient-to-br from-green-500/20 to-green-600/10 border rounded-xl p-4">
          <div className="text-sm text-muted-foreground mb-1">Knowledge</div>
          <div className="text-3xl font-bold">{stats.total_knowledge}</div>
          <div className="text-xs text-muted-foreground mt-1">Learned facts</div>
        </div>
        
        <div className="bg-gradient-to-br from-purple-500/20 to-purple-600/10 border rounded-xl p-4">
          <div className="text-sm text-muted-foreground mb-1">Reflections</div>
          <div className="text-3xl font-bold">{stats.total_reflections}</div>
          <div className="text-xs text-muted-foreground mt-1">Self-analyses</div>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 mb-6">
        <div className="border rounded-xl p-4">
          <h3 className="font-semibold mb-3">Memory Distribution</h3>
          <div className="space-y-3">
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>Entities</span>
                <span className="text-muted-foreground">{stats.total_entities}</span>
              </div>
              <div className="h-2 bg-muted rounded-full overflow-hidden">
                <div 
                  className="h-full bg-blue-500 rounded-full"
                  style={{ width: `${(stats.total_entities / (stats.total_entities + stats.total_knowledge + stats.total_reflections)) * 100}%` }}
                />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>Knowledge</span>
                <span className="text-muted-foreground">{stats.total_knowledge}</span>
              </div>
              <div className="h-2 bg-muted rounded-full overflow-hidden">
                <div 
                  className="h-full bg-green-500 rounded-full"
                  style={{ width: `${(stats.total_knowledge / (stats.total_entities + stats.total_knowledge + stats.total_reflections)) * 100}%` }}
                />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-sm mb-1">
                <span>Reflections</span>
                <span className="text-muted-foreground">{stats.total_reflections}</span>
              </div>
              <div className="h-2 bg-muted rounded-full overflow-hidden">
                <div 
                  className="h-full bg-purple-500 rounded-full"
                  style={{ width: `${(stats.total_reflections / (stats.total_entities + stats.total_knowledge + stats.total_reflections)) * 100}%` }}
                />
              </div>
            </div>
          </div>
        </div>

        <div className="border rounded-xl p-4">
          <h3 className="font-semibold mb-3">Memory Health</h3>
          <div className="space-y-2">
            {stats.total_entities > 0 ? (
              <div className="flex items-center gap-2 text-green-600">
                <span className="text-lg">✓</span>
                <span className="text-sm">Entity memory active</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-muted-foreground">
                <span className="text-lg">○</span>
                <span className="text-sm">No entities yet</span>
              </div>
            )}
            {stats.total_knowledge > 0 ? (
              <div className="flex items-center gap-2 text-green-600">
                <span className="text-lg">✓</span>
                <span className="text-sm">Knowledge base populated</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-muted-foreground">
                <span className="text-lg">○</span>
                <span className="text-sm">No knowledge yet</span>
              </div>
            )}
            {stats.total_reflections > 0 ? (
              <div className="flex items-center gap-2 text-green-600">
                <span className="text-lg">✓</span>
                <span className="text-sm">Self-reflection enabled</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-muted-foreground">
                <span className="text-lg">○</span>
                <span className="text-sm">No reflections yet</span>
              </div>
            )}
          </div>
          
          <div className="mt-4 pt-4 border-t">
            <div className="text-sm font-medium mb-2">Memory Usage Score</div>
            <div className="flex items-center gap-3">
              <div className="flex-1 h-2 bg-muted rounded-full overflow-hidden">
                <div 
                  className="h-full bg-gradient-to-r from-green-500 to-blue-500 rounded-full transition-all"
                  style={{ width: `${Math.min(100, ((stats.total_entities + stats.total_knowledge + stats.total_reflections) / 100) * 100)}%` }}
                />
              </div>
              <span className="text-sm font-medium">
                {Math.min(100, Math.round(((stats.total_entities + stats.total_knowledge + stats.total_reflections) / 100) * 100))}%
              </span>
            </div>
          </div>
        </div>
      </div>

      {stats.recent_reflections.length > 0 && (
        <div className="border rounded-xl p-4">
          <h3 className="font-semibold mb-3">Recent Reflections</h3>
          <div className="space-y-3">
            {stats.recent_reflections.map((reflection) => (
              <div key={reflection.id} className="p-3 bg-muted/50 rounded-lg">
                <div className="flex items-start justify-between gap-2">
                  <p className="text-sm flex-1">{reflection.content}</p>
                  <div className="flex items-center gap-2 shrink-0">
                    <div className="w-16 h-1.5 bg-muted rounded-full overflow-hidden">
                      <div 
                        className="h-full bg-purple-500 rounded-full"
                        style={{ width: `${(reflection.importance / maxImportance) * 100}%` }}
                      />
                    </div>
                    <span className="text-xs text-muted-foreground w-10">
                      {reflection.importance}
                    </span>
                  </div>
                </div>
                <div className="text-xs text-muted-foreground mt-2">
                  {new Date(reflection.created_at).toLocaleString()}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {stats.recent_reflections.length === 0 && (
        <div className="border rounded-xl p-8 text-center">
          <div className="text-4xl mb-2">🧠</div>
          <h3 className="font-semibold mb-1">No Reflections Yet</h3>
          <p className="text-sm text-muted-foreground">
            The agent will generate reflections as it learns and processes information.
          </p>
        </div>
      )}
    </div>
  );
}
