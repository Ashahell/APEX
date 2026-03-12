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
          Loading memory stats...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-red-500 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="15" y1="9" x2="9" y2="15"></line>
            <line x1="9" y1="9" x2="15" y2="15"></line>
          </svg>
          Error: {error}
        </div>
      </div>
    );
  }

  if (!stats) return null;

  const maxImportance = Math.max(...stats.recent_reflections.map(r => r.importance), 1);
  const total = stats.total_entities + stats.total_knowledge + stats.total_reflections;

  return (
    <div className="h-full overflow-y-auto p-6">
      {/* Header */}
      <div className="flex items-center gap-3 mb-6">
        <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 2a10 10 0 1 0 10 10 4 4 0 0 1-5-5 4 4 0 0 1-5-5"></path>
            <path d="M8.5 8.5v.01"></path>
            <path d="M16 15.5v.01"></path>
            <path d="M12 12v.01"></path>
            <path d="M11 17v.01"></path>
            <path d="M7 14v.01"></path>
          </svg>
        </div>
        <div>
          <h2 className="text-xl font-semibold">Memory Dashboard</h2>
          <p className="text-sm text-[var(--color-text-muted)]">Agent memory statistics and insights</p>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-3 mb-6">
        <div className="bg-gradient-to-br from-[#4248f1]/20 to-[#4248f1]/5 border border-[var(--color-border)] rounded-xl p-4">
          <div className="text-sm text-[var(--color-text-muted)] mb-1 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
              <circle cx="12" cy="7" r="4"></circle>
            </svg>
            Entities
          </div>
          <div className="text-3xl font-bold text-[#4248f1]">{stats.total_entities}</div>
          <div className="text-xs text-[var(--color-text-muted)] mt-1">Stored memories</div>
        </div>
        
        <div className="bg-gradient-to-br from-green-500/20 to-green-500/5 border border-[var(--color-border)] rounded-xl p-4">
          <div className="text-sm text-[var(--color-text-muted)] mb-1 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"></path>
            </svg>
            Knowledge
          </div>
          <div className="text-3xl font-bold text-green-500">{stats.total_knowledge}</div>
          <div className="text-xs text-[var(--color-text-muted)] mt-1">Learned facts</div>
        </div>
        
        <div className="bg-gradient-to-br from-purple-500/20 to-purple-500/5 border border-[var(--color-border)] rounded-xl p-4">
          <div className="text-sm text-[var(--color-text-muted)] mb-1 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
            Reflections
          </div>
          <div className="text-3xl font-bold text-purple-500">{stats.total_reflections}</div>
          <div className="text-xs text-[var(--color-text-muted)] mt-1">Self-analyses</div>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 mb-6">
        {/* Memory Distribution */}
        <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="20" x2="18" y2="10"></line>
              <line x1="12" y1="20" x2="12" y2="4"></line>
              <line x1="6" y1="20" x2="6" y2="14"></line>
            </svg>
            Memory Distribution
          </h3>
          <div className="space-y-3">
            <div>
              <div className="flex justify-between text-sm mb-1.5">
                <span className="text-[var(--color-text)]">Entities</span>
                <span className="text-[var(--color-text-muted)]">{stats.total_entities}</span>
              </div>
              <div className="h-2 bg-[var(--color-muted)] rounded-full overflow-hidden">
                <div 
                  className="h-full bg-[#4248f1] rounded-full transition-all"
                  style={{ width: `${total > 0 ? (stats.total_entities / total) * 100 : 0}%` }}
                />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-sm mb-1.5">
                <span className="text-[var(--color-text)]">Knowledge</span>
                <span className="text-[var(--color-text-muted)]">{stats.total_knowledge}</span>
              </div>
              <div className="h-2 bg-[var(--color-muted)] rounded-full overflow-hidden">
                <div 
                  className="h-full bg-green-500 rounded-full transition-all"
                  style={{ width: `${total > 0 ? (stats.total_knowledge / total) * 100 : 0}%` }}
                />
              </div>
            </div>
            <div>
              <div className="flex justify-between text-sm mb-1.5">
                <span className="text-[var(--color-text)]">Reflections</span>
                <span className="text-[var(--color-text-muted)]">{stats.total_reflections}</span>
              </div>
              <div className="h-2 bg-[var(--color-muted)] rounded-full overflow-hidden">
                <div 
                  className="h-full bg-purple-500 rounded-full transition-all"
                  style={{ width: `${total > 0 ? (stats.total_reflections / total) * 100 : 0}%` }}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Memory Health */}
        <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 12h-4l-3 9L9 3l-3 9H2"></path>
            </svg>
            Memory Health
          </h3>
          <div className="space-y-2">
            {stats.total_entities > 0 ? (
              <div className="flex items-center gap-2 text-green-500">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
                <span className="text-sm">Entity memory active</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-[var(--color-text-muted)]">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                </svg>
                <span className="text-sm">No entities yet</span>
              </div>
            )}
            {stats.total_knowledge > 0 ? (
              <div className="flex items-center gap-2 text-green-500">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
                <span className="text-sm">Knowledge base populated</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-[var(--color-text-muted)]">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                </svg>
                <span className="text-sm">No knowledge yet</span>
              </div>
            )}
            {stats.total_reflections > 0 ? (
              <div className="flex items-center gap-2 text-green-500">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
                <span className="text-sm">Self-reflection enabled</span>
              </div>
            ) : (
              <div className="flex items-center gap-2 text-[var(--color-text-muted)]">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                </svg>
                <span className="text-sm">No reflections yet</span>
              </div>
            )}
          </div>
          
          <div className="mt-4 pt-4 border-t border-[var(--color-border)]">
            <div className="text-sm font-medium mb-2">Memory Usage Score</div>
            <div className="flex items-center gap-3">
              <div className="flex-1 h-2 bg-[var(--color-muted)] rounded-full overflow-hidden">
                <div 
                  className="h-full bg-gradient-to-r from-green-500 to-[#4248f1] rounded-full transition-all"
                  style={{ width: `${Math.min(100, total)}%` }}
                />
              </div>
              <span className="text-sm font-medium">
                {Math.min(100, total)}%
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Recent Reflections */}
      {stats.recent_reflections.length > 0 && (
        <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
            Recent Reflections
          </h3>
          <div className="space-y-3">
            {stats.recent_reflections.map((reflection) => (
              <div key={reflection.id} className="p-3 bg-[var(--color-muted)]/30 rounded-lg">
                <div className="flex items-start justify-between gap-2">
                  <p className="text-sm flex-1">{reflection.content}</p>
                  <div className="flex items-center gap-2 shrink-0">
                    <div className="w-16 h-1.5 bg-[var(--color-muted)] rounded-full overflow-hidden">
                      <div 
                        className="h-full bg-purple-500 rounded-full"
                        style={{ width: `${(reflection.importance / maxImportance) * 100}%` }}
                      />
                    </div>
                    <span className="text-xs text-[var(--color-text-muted)] w-10">
                      {reflection.importance}
                    </span>
                  </div>
                </div>
                <div className="text-xs text-[var(--color-text-muted)] mt-2">
                  {new Date(reflection.created_at).toLocaleString()}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {stats.recent_reflections.length === 0 && (
        <div className="border border-[var(--color-border)] rounded-xl p-8 text-center bg-[var(--color-panel)]">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--color-muted)] flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-[var(--color-text-muted)]">
              <path d="M12 2a10 10 0 1 0 10 10 4 4 0 0 1-5-5 4 4 0 0 1-5-5"></path>
              <path d="M8.5 8.5v.01"></path>
              <path d="M16 15.5v.01"></path>
              <path d="M12 12v.01"></path>
              <path d="M11 17v.01"></path>
              <path d="M7 14v.01"></path>
            </svg>
          </div>
          <h3 className="font-semibold mb-1">No Reflections Yet</h3>
          <p className="text-sm text-[var(--color-text-muted)]">
            The agent will generate reflections as it learns and processes information.
          </p>
        </div>
      )}
    </div>
  );
}
