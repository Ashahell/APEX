import { useState, useEffect, useCallback } from 'react';
import {
  getMultimodalConfig,
  updateMultimodalConfig,
  getMultimodalStats,
  listMultimodalEmbeddings,
  searchMultimodal,
  type MultimodalConfig,
  type MultimodalStats,
  type MultimodalEmbedding,
  type MultimodalSearchResult,
} from '../../lib/api';

type Modality = 'all' | 'text' | 'image' | 'audio';

// ============ Multimodal Settings Component ============

export function MultimodalSettings() {
  const [config, setConfig] = useState<MultimodalConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getMultimodalConfig();
      setConfig(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load config');
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    setError(null);
    setSuccess(false);
    try {
      await updateMultimodalConfig(config);
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save config');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (!config) return null;

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold text-white mb-4">Multimodal Memory Settings</h3>
        
        {error && (
          <div className="mb-4 p-3 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
            {error}
          </div>
        )}
        
        {success && (
          <div className="mb-4 p-3 bg-green-900/30 border border-green-700/50 rounded text-green-400 text-sm">
            Settings saved successfully!
          </div>
        )}

        {/* Enable/Disable */}
        <div className="flex items-center justify-between p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
          <div>
            <div className="text-white font-medium">Enable Multimodal Memory</div>
            <div className="text-sm text-gray-400">Enable indexing and searching of images and audio</div>
          </div>
          <button
            onClick={() => setConfig({ ...config, enabled: !config.enabled })}
            className={`relative w-12 h-6 rounded-full transition-colors ${
              config.enabled ? 'bg-indigo-600' : 'bg-gray-600'
            }`}
          >
            <span
              className={`absolute top-1 w-4 h-4 bg-white rounded-full transition-transform ${
                config.enabled ? 'translate-x-7' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {/* Image Indexing */}
        <div className="flex items-center justify-between p-4 bg-gray-800/50 rounded-lg border border-gray-700/50 mt-4">
          <div>
            <div className="text-white font-medium">Image Indexing</div>
            <div className="text-sm text-gray-400">Automatically index images with embeddings</div>
          </div>
          <button
            onClick={() => setConfig({ ...config, image_indexing: !config.image_indexing })}
            disabled={!config.enabled}
            className={`relative w-12 h-6 rounded-full transition-colors ${
              config.image_indexing && config.enabled ? 'bg-indigo-600' : 'bg-gray-600'
            } disabled:opacity-50`}
          >
            <span
              className={`absolute top-1 w-4 h-4 bg-white rounded-full transition-transform ${
                config.image_indexing && config.enabled ? 'translate-x-7' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {/* Audio Indexing */}
        <div className="flex items-center justify-between p-4 bg-gray-800/50 rounded-lg border border-gray-700/50 mt-4">
          <div>
            <div className="text-white font-medium">Audio Indexing</div>
            <div className="text-sm text-gray-400">Automatically index audio with embeddings</div>
          </div>
          <button
            onClick={() => setConfig({ ...config, audio_indexing: !config.audio_indexing })}
            disabled={!config.enabled}
            className={`relative w-12 h-6 rounded-full transition-colors ${
              config.audio_indexing && config.enabled ? 'bg-indigo-600' : 'bg-gray-600'
            } disabled:opacity-50`}
          >
            <span
              className={`absolute top-1 w-4 h-4 bg-white rounded-full transition-transform ${
                config.audio_indexing && config.enabled ? 'translate-x-7' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {/* Embedding Model */}
        <div className="mt-4 p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
          <label className="block text-white font-medium mb-2">Embedding Model</label>
          <select
            value={config.embedding_model}
            onChange={(e) => setConfig({ ...config, embedding_model: e.target.value })}
            disabled={!config.enabled}
            className="w-full px-3 py-2 bg-gray-900 border border-gray-600 rounded text-white focus:outline-none focus:border-indigo-500 disabled:opacity-50"
          >
            <option value="gemini-embedding-2-preview">Gemini Embedding 2 Preview</option>
            <option value="text-embedding-3-small">OpenAI Text Embedding 3 Small</option>
            <option value="text-embedding-3-large">OpenAI Text Embedding 3 Large</option>
          </select>
          <div className="text-xs text-gray-500 mt-1">
            Dimension: {config.embedding_dim}
          </div>
        </div>

        {/* Save Button */}
        <button
          onClick={handleSave}
          disabled={saving || !config.enabled}
          className="w-full mt-6 px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded-lg transition-colors"
        >
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  );
}

// ============ Multimodal Stats Component ============

export function MultimodalStats() {
  const [stats, setStats] = useState<MultimodalStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadStats = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await getMultimodalStats();
      setStats(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load stats');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadStats();
  }, [loadStats]);

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
        {error}
      </div>
    );
  }

  if (!stats) return null;

  return (
    <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
      {/* Text Embeddings */}
      <div className="p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
        <div className="text-2xl mb-1">📝</div>
        <div className="text-2xl font-bold text-white">{stats.text_embeddings}</div>
        <div className="text-sm text-gray-400">Text Embeddings</div>
      </div>

      {/* Image Embeddings */}
      <div className="p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
        <div className="text-2xl mb-1">🖼️</div>
        <div className="text-2xl font-bold text-white">{stats.image_embeddings}</div>
        <div className="text-sm text-gray-400">Image Embeddings</div>
      </div>

      {/* Audio Embeddings */}
      <div className="p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
        <div className="text-2xl mb-1">🎵</div>
        <div className="text-2xl font-bold text-white">{stats.audio_embeddings}</div>
        <div className="text-sm text-gray-400">Audio Embeddings</div>
      </div>

      {/* Total Embeddings */}
      <div className="p-4 bg-gray-800/50 rounded-lg border border-gray-700/50 col-span-2 md:col-span-1">
        <div className="text-2xl mb-1">💾</div>
        <div className="text-2xl font-bold text-white">{stats.total_embeddings}</div>
        <div className="text-sm text-gray-400">Total Embeddings</div>
      </div>

      {/* Pending Jobs */}
      <div className="p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
        <div className="text-2xl mb-1">⏳</div>
        <div className="text-2xl font-bold text-yellow-400">{stats.pending_jobs}</div>
        <div className="text-sm text-gray-400">Pending Jobs</div>
      </div>

      {/* Processing Jobs */}
      <div className="p-4 bg-gray-800/50 rounded-lg border border-gray-700/50">
        <div className="text-2xl mb-1">⚙️</div>
        <div className="text-2xl font-bold text-blue-400">{stats.processing_jobs}</div>
        <div className="text-sm text-gray-400">Processing</div>
      </div>
    </div>
  );
}

// ============ Multimodal Search Component ============

export function MultimodalSearch() {
  const [query, setQuery] = useState('');
  const [modality, setModality] = useState<Modality>('all');
  const [results, setResults] = useState<MultimodalSearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  const handleSearch = async () => {
    setLoading(true);
    setError(null);
    setSearched(true);
    try {
      const data = await searchMultimodal({
        q: query || undefined,
        modality: modality === 'all' ? undefined : modality,
        limit: 20,
      });
      setResults(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
    } finally {
      setLoading(false);
    }
  };

  const getModalityIcon = (mod: string) => {
    switch (mod) {
      case 'image': return '🖼️';
      case 'audio': return '🎵';
      default: return '📝';
    }
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString();
  };

  return (
    <div className="space-y-4">
      {/* Search Input */}
      <div className="flex gap-2">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search memory..."
          className="flex-1 px-4 py-2 bg-gray-800 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500"
          onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
        />
        <button
          onClick={handleSearch}
          disabled={loading}
          className="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white rounded-lg transition-colors"
        >
          {loading ? 'Searching...' : 'Search'}
        </button>
      </div>

      {/* Modality Filter */}
      <div className="flex gap-2">
        {(['all', 'text', 'image', 'audio'] as Modality[]).map((m) => (
          <button
            key={m}
            onClick={() => setModality(m)}
            className={`px-3 py-1 text-sm rounded-full transition-colors ${
              modality === m
                ? 'bg-indigo-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            {m === 'all' ? 'All' : m.charAt(0).toUpperCase() + m.slice(1)} {getModalityIcon(m)}
          </button>
        ))}
      </div>

      {/* Error */}
      {error && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
          {error}
        </div>
      )}

      {/* Results */}
      {searched && !loading && (
        <div>
          <div className="text-sm text-gray-400 mb-2">
            {results.length} results found
          </div>
          {results.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              No results found
            </div>
          ) : (
            <ul className="space-y-2">
              {results.map((result, idx) => (
                <li
                  key={idx}
                  className="p-3 bg-gray-800/50 rounded border border-gray-700/50"
                >
                  <div className="flex items-center gap-2">
                    <span>{getModalityIcon(result.modality)}</span>
                    <span className="text-white font-medium">{result.memory_type}</span>
                    <span className="text-gray-500 text-xs">{formatDate(result.created_at)}</span>
                  </div>
                  {result.original_data && (
                    <div className="mt-2 text-sm text-gray-400 line-clamp-2">
                      {result.modality === 'image' ? '[Image data]' : 
                       result.modality === 'audio' ? '[Audio data]' : 
                       result.original_data.slice(0, 200)}
                    </div>
                  )}
                  <div className="mt-1 text-xs text-indigo-400">
                    Score: {result.score.toFixed(3)}
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

// ============ Multimodal Embeddings List Component ============

export function MultimodalEmbeddingsList() {
  const [modality, setModality] = useState<Modality>('all');
  const [embeddings, setEmbeddings] = useState<MultimodalEmbedding[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadEmbeddings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listMultimodalEmbeddings(
        modality === 'all' ? undefined : modality,
        50
      );
      setEmbeddings(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load embeddings');
    } finally {
      setLoading(false);
    }
  }, [modality]);

  useEffect(() => {
    loadEmbeddings();
  }, [loadEmbeddings]);

  const getModalityIcon = (mod: string) => {
    switch (mod) {
      case 'image': return '🖼️';
      case 'audio': return '🎵';
      default: return '📝';
    }
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString();
  };

  return (
    <div className="space-y-4">
      {/* Filter */}
      <div className="flex gap-2">
        {(['all', 'text', 'image', 'audio'] as Modality[]).map((m) => (
          <button
            key={m}
            onClick={() => setModality(m)}
            className={`px-3 py-1 text-sm rounded-full transition-colors ${
              modality === m
                ? 'bg-indigo-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            {m === 'all' ? 'All' : m.charAt(0).toUpperCase() + m.slice(1)} {getModalityIcon(m)}
          </button>
        ))}
      </div>

      {/* Loading/Error */}
      {loading && (
        <div className="flex items-center justify-center p-8">
          <div className="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
        </div>
      )}

      {error && (
        <div className="p-3 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
          {error}
        </div>
      )}

      {/* List */}
      {!loading && !error && (
        <>
          <div className="text-sm text-gray-400">
            {embeddings.length} embeddings
          </div>
          {embeddings.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              No embeddings yet
            </div>
          ) : (
            <ul className="space-y-2 max-h-96 overflow-y-auto">
              {embeddings.map((emb) => (
                <li
                  key={emb.id}
                  className="p-3 bg-gray-800/50 rounded border border-gray-700/50"
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span>{getModalityIcon(emb.modality)}</span>
                      <span className="text-white font-medium">{emb.memory_type}</span>
                    </div>
                    <span className="text-xs text-gray-500">{formatDate(emb.created_at)}</span>
                  </div>
                  <div className="mt-1 text-xs text-gray-500">
                    Model: {emb.embedding_model} • {emb.has_original_data ? 'Has data' : 'No data'}
                  </div>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </div>
  );
}
