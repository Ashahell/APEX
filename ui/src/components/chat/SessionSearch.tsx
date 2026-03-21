import { useState, useCallback } from 'react';
import { apiGet, apiPost } from '../../lib/api';

// Types matching the Rust backend
interface SearchResult {
  task_id: string;
  content: string;
  matched_content: string;
  rank: number;
  context_before: string;
  context_after: string;
}

interface SearchStats {
  total_results: number;
  query_time_ms: number;
  fts_enabled: boolean;
}

interface SearchResponse {
  results: SearchResult[];
  total: number;
}

interface SessionSearchProps {
  onSelectResult?: (result: SearchResult) => void;
}

export function SessionSearch({ onSelectResult }: SessionSearchProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [stats, setStats] = useState<SearchStats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  const performSearch = useCallback(async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([]);
      setTotal(0);
      setSearched(false);
      return;
    }

    setLoading(true);
    setError(null);
    setSearched(true);

    try {
      const params = new URLSearchParams({
        q: searchQuery,
        limit: '10',
        include_context: 'true',
      });
      
      const res = await apiGet(`/api/v1/search/sessions?${params}`);
      if (res.ok) {
        const data: SearchResponse = await res.json();
        setResults(data.results);
        setTotal(data.total);
      } else {
        setError('Search failed');
      }

      // Get stats
      const statsRes = await apiGet('/api/v1/search/sessions/stats');
      if (statsRes.ok) {
        const statsData: SearchStats = await statsRes.json();
        setStats(statsData);
      }
    } catch {
      setError('Search failed');
    } finally {
      setLoading(false);
    }
  }, []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    performSearch(query);
  };

  const handleReindex = async () => {
    setLoading(true);
    try {
      await apiPost('/api/v1/search/reindex', {});
      // Refresh stats
      const statsRes = await apiGet('/api/v1/search/sessions/stats');
      if (statsRes.ok) {
        const statsData: SearchStats = await statsRes.json();
        setStats(statsData);
      }
    } catch {
      setError('Reindex failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* Search form */}
      <form onSubmit={handleSubmit} className="flex gap-2">
        <div className="relative flex-1">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search conversations..."
            className="w-full px-4 py-2 pl-10 rounded-lg border border-border bg-background focus:outline-none focus:ring-2 focus:ring-primary/50"
          />
          <svg
            className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
            />
          </svg>
        </div>
        <button
          type="submit"
          disabled={loading || !query.trim()}
          className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50 transition-colors"
        >
          {loading ? 'Searching...' : 'Search'}
        </button>
      </form>

      {/* Reindex button */}
      <div className="flex items-center justify-between">
        <div className="text-sm text-muted-foreground">
          {stats && (
            <span>
              {stats.total_results} conversations indexed
              {stats.fts_enabled ? ' (FTS5 enabled)' : ' (basic search)'}
            </span>
          )}
        </div>
        <button
          onClick={handleReindex}
          disabled={loading}
          className="text-sm text-primary hover:underline disabled:opacity-50"
        >
          Rebuild Index
        </button>
      </div>

      {/* Error */}
      {error && (
        <div className="bg-destructive/10 border border-destructive/20 rounded-lg p-4 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Results */}
      {searched && !loading && (
        <div className="space-y-3">
          <div className="text-sm text-muted-foreground">
            Found {total} result{total !== 1 ? 's' : ''}
          </div>

          {results.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              No results found for "{query}"
            </div>
          ) : (
            <div className="space-y-2">
              {results.map((result, index) => (
                <button
                  key={`${result.task_id}-${index}`}
                  onClick={() => onSelectResult?.(result)}
                  className="w-full text-left p-4 rounded-lg border border-border hover:border-primary/50 hover:bg-muted/50 transition-colors"
                >
                  {/* Context before */}
                  {result.context_before && (
                    <div className="text-xs text-muted-foreground mb-2 line-clamp-2">
                      ...{result.context_before}
                    </div>
                  )}

                  {/* Matched content */}
                  <div className="text-sm">
                    <span
                      dangerouslySetInnerHTML={{
                        __html: result.matched_content.replace(
                          new RegExp(`(${query})`, 'gi'),
                          '<mark class="bg-yellow-200 dark:bg-yellow-800 px-0.5 rounded">$1</mark>'
                        ),
                      }}
                    />
                  </div>

                  {/* Context after */}
                  {result.context_after && (
                    <div className="text-xs text-muted-foreground mt-2 line-clamp-2">
                      {result.context_after}...
                    </div>
                  )}

                  {/* Task ID */}
                  <div className="text-xs text-muted-foreground mt-2">
                    Task: {result.task_id.slice(0, 8)}... • Relevance: {(result.rank * 100).toFixed(0)}%
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Loading state */}
      {loading && searched && (
        <div className="text-center py-8">
          <div className="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
          <div className="mt-2 text-sm text-muted-foreground">Searching...</div>
        </div>
      )}
    </div>
  );
}

// Compact search bar component for chat header
export function SessionSearchBar() {
  const [showSearch, setShowSearch] = useState(false);
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);

  const performSearch = useCallback(async (searchQuery: string) => {
    if (!searchQuery.trim()) {
      setResults([]);
      return;
    }

    setLoading(true);
    try {
      const params = new URLSearchParams({
        q: searchQuery,
        limit: '5',
        include_context: 'false',
      });
      
      const res = await apiGet(`/api/v1/search/sessions?${params}`);
      if (res.ok) {
        const data: SearchResponse = await res.json();
        setResults(data.results);
      }
    } catch {
      // Silently fail
    } finally {
      setLoading(false);
    }
  }, []);

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setQuery(e.target.value);
    performSearch(e.target.value);
  };

  if (!showSearch) {
    return (
      <button
        onClick={() => setShowSearch(true)}
        className="p-2 rounded-lg hover:bg-muted transition-colors"
        title="Search conversations"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
      </button>
    );
  }

  return (
    <div className="relative">
      <input
        type="text"
        value={query}
        onChange={handleSearch}
        placeholder="Search..."
        className="w-48 px-3 py-1.5 text-sm rounded-lg border border-border bg-background focus:outline-none focus:ring-2 focus:ring-primary/50"
        autoFocus
      />
      <button
        onClick={() => {
          setShowSearch(false);
          setQuery('');
          setResults([]);
        }}
        className="absolute right-2 top-1/2 -translate-y-1/2 p-1 hover:bg-muted rounded"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>

      {/* Dropdown results */}
      {(results.length > 0 || loading) && (
        <div className="absolute top-full left-0 right-0 mt-2 bg-background border border-border rounded-lg shadow-lg max-h-64 overflow-y-auto z-50">
          {loading ? (
            <div className="p-3 text-sm text-muted-foreground text-center">Searching...</div>
          ) : (
            results.map((result, index) => (
              <button
                key={`${result.task_id}-${index}`}
                className="w-full text-left p-3 hover:bg-muted border-b border-border last:border-b-0"
              >
                <div
                  className="text-sm line-clamp-2"
                  dangerouslySetInnerHTML={{
                    __html: result.matched_content.replace(
                      new RegExp(`(${query})`, 'gi'),
                      '<mark class="bg-yellow-200 dark:bg-yellow-800 px-0.5 rounded">$1</mark>'
                    ),
                  }}
                />
                <div className="text-xs text-muted-foreground mt-1">
                  {result.task_id.slice(0, 8)}...
                </div>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}
