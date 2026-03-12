import { useState, useEffect } from 'react';
import { listJournalEntries, createJournalEntry, updateJournalEntry, deleteJournalEntry, searchJournal, DecisionJournalEntry } from '../../lib/api';

export function DecisionJournal() {
  const [entries, setEntries] = useState<DecisionJournalEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [formData, setFormData] = useState({
    title: '',
    context: '',
    decision: '',
    rationale: '',
    outcome: '',
    tags: '',
    task_id: '',
  });

  useEffect(() => {
    loadEntries();
  }, []);

  async function loadEntries() {
    try {
      setLoading(true);
      const data = await listJournalEntries();
      setEntries(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load journal entries');
    } finally {
      setLoading(false);
    }
  }

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault();
    if (!searchQuery.trim()) {
      loadEntries();
      return;
    }
    try {
      setLoading(true);
      const data = await searchJournal(searchQuery);
      setEntries(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to search journal');
    } finally {
      setLoading(false);
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const tags = formData.tags || undefined;
    const entry = {
      title: formData.title,
      context: formData.context || undefined,
      decision: formData.decision,
      rationale: formData.rationale || undefined,
      outcome: formData.outcome || undefined,
      tags,
      task_id: formData.task_id || undefined,
    };
    try {
      if (editingId) {
        await updateJournalEntry(editingId, entry);
      } else {
        await createJournalEntry(entry);
      }
      setFormData({ title: '', context: '', decision: '', rationale: '', outcome: '', tags: '', task_id: '' });
      setShowForm(false);
      setEditingId(null);
      loadEntries();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save entry');
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this entry?')) return;
    try {
      await deleteJournalEntry(id);
      loadEntries();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete entry');
    }
  }

  function startEdit(entry: DecisionJournalEntry) {
    setEditingId(entry.id);
    setFormData({
      title: entry.title,
      context: entry.context || '',
      decision: entry.decision,
      rationale: entry.rationale || '',
      outcome: entry.outcome || '',
      tags: entry.tags || '',
      task_id: entry.task_id || '',
    });
    setShowForm(true);
  }

  function parseTags(tags?: string): string[] {
    if (!tags) return [];
    return tags.split(',').map(t => t.trim()).filter(Boolean);
  }

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
          Loading journal entries...
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-3xl mx-auto space-y-4">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
              </svg>
            </div>
            <h2 className="text-xl font-semibold">Decision Journal</h2>
          </div>
          <button
            onClick={() => {
              setShowForm(!showForm);
              setEditingId(null);
              setFormData({ title: '', context: '', decision: '', rationale: '', outcome: '', tags: '', task_id: '' });
            }}
            className="px-3 py-1.5 bg-[#4248f1] text-white rounded-lg text-sm hover:bg-[#353bc5] transition-colors flex items-center gap-2"
          >
            {showForm ? (
              <>
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
                Cancel
              </>
            ) : (
              <>
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="12" y1="5" x2="12" y2="19"></line>
                  <line x1="5" y1="12" x2="19" y2="12"></line>
                </svg>
                + New Entry
              </>
            )}
          </button>
        </div>

        {/* Search */}
        <form onSubmit={handleSearch} className="flex gap-2">
          <input
            type="text"
            placeholder="Search decisions..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 px-3 py-2.5 bg-[var(--color-panel)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
          />
          <button
            type="submit"
            className="px-4 py-2.5 bg-[var(--color-muted)] text-[var(--color-text)] rounded-lg text-sm hover:bg-[var(--color-muted)]/80 transition-colors"
          >
            Search
          </button>
        </form>

        {/* Error */}
        {error && (
          <div className="p-3 bg-red-500/10 text-red-500 rounded-lg border border-red-500/20 text-sm flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="15" y1="9" x2="9" y2="15"></line>
              <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
            {error}
          </div>
        )}

        {/* Form */}
        {showForm && (
          <form onSubmit={handleSubmit} className="p-4 bg-[var(--color-panel)] border border-[var(--color-border)] rounded-xl space-y-3">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div>
                <label className="block text-sm font-medium mb-1.5">Title *</label>
                <input
                  type="text"
                  value={formData.title}
                  onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                  className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1.5">Task ID</label>
                <input
                  type="text"
                  value={formData.task_id}
                  onChange={(e) => setFormData({ ...formData, task_id: e.target.value })}
                  className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                  placeholder="Optional linked task"
                />
              </div>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1.5">Context</label>
              <textarea
                value={formData.context}
                onChange={(e) => setFormData({ ...formData, context: e.target.value })}
                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                rows={2}
                placeholder="Background and context..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1.5">Decision *</label>
              <textarea
                value={formData.decision}
                onChange={(e) => setFormData({ ...formData, decision: e.target.value })}
                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                rows={2}
                required
                placeholder="What decision was made..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1.5">Rationale</label>
              <textarea
                value={formData.rationale}
                onChange={(e) => setFormData({ ...formData, rationale: e.target.value })}
                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                rows={2}
                placeholder="Why this decision was made..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1.5">Outcome</label>
              <textarea
                value={formData.outcome}
                onChange={(e) => setFormData({ ...formData, outcome: e.target.value })}
                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                rows={2}
                placeholder="How it turned out..."
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1.5">Tags</label>
              <input
                type="text"
                value={formData.tags}
                onChange={(e) => setFormData({ ...formData, tags: e.target.value })}
                className="w-full px-3 py-2 bg-[var(--color-background)] border border-[var(--color-border)] rounded-lg text-sm text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                placeholder="Comma-separated tags"
              />
            </div>
            <button
              type="submit"
              className="px-4 py-2 bg-[#4248f1] text-white rounded-lg text-sm hover:bg-[#353bc5] transition-colors"
            >
              {editingId ? 'Update' : 'Create'}
            </button>
          </form>
        )}

        {/* Entries List */}
        <div className="space-y-3">
          {entries.map((entry) => (
            <div
              key={entry.id}
              className="p-4 bg-[var(--color-panel)] border border-[var(--color-border)] rounded-xl hover:border-[#4248f1]/30 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="font-semibold">{entry.title}</div>
                  {entry.task_id && (
                    <div className="text-xs text-[var(--color-text-muted)] mt-1">
                      Task: {entry.task_id}
                    </div>
                  )}
                  <div className="text-sm mt-2 p-2 bg-[var(--color-muted)]/30 rounded-lg">
                    <span className="font-medium">Decision:</span> {entry.decision}
                  </div>
                  {entry.context && (
                    <div className="text-sm mt-2 text-[var(--color-text-muted)]">
                      <span className="font-medium">Context:</span> {entry.context}
                    </div>
                  )}
                  {entry.rationale && (
                    <div className="text-sm mt-1 text-[var(--color-text-muted)]">
                      <span className="font-medium">Rationale:</span> {entry.rationale}
                    </div>
                  )}
                  {entry.outcome && (
                    <div className="text-sm mt-1 text-green-500">
                      <span className="font-medium">Outcome:</span> {entry.outcome}
                    </div>
                  )}
                  {entry.tags && (
                    <div className="flex gap-1 mt-2 flex-wrap">
                      {parseTags(entry.tags).map((tag, i) => (
                        <span key={i} className="px-2 py-0.5 bg-[#4248f1]/10 text-[#4248f1] text-xs rounded-full border border-[#4248f1]/20">
                          {tag}
                        </span>
                      ))}
                    </div>
                  )}
                </div>
                <div className="flex gap-1 ml-4">
                  <button
                    onClick={() => startEdit(entry)}
                    className="p-2 text-[var(--color-text-muted)] hover:text-[#4248f1] transition-colors"
                    title="Edit"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                    </svg>
                  </button>
                  <button
                    onClick={() => handleDelete(entry.id)}
                    className="p-2 text-[var(--color-text-muted)] hover:text-red-500 transition-colors"
                    title="Delete"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <polyline points="3 6 5 6 21 6"></polyline>
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    </svg>
                  </button>
                </div>
              </div>
              <div className="text-xs text-[var(--color-text-muted)] mt-3">
                {new Date(entry.created_at).toLocaleString()}
              </div>
            </div>
          ))}
        </div>

        {entries.length === 0 && !showForm && (
          <div className="text-center text-[var(--color-text-muted)] py-8">
            No journal entries yet. Document your decisions to build institutional knowledge.
          </div>
        )}
      </div>
    </div>
  );
}
