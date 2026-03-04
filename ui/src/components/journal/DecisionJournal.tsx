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
      <div className="p-4 text-center text-muted-foreground">
        Loading journal entries...
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Decision Journal</h2>
        <button
          onClick={() => {
            setShowForm(!showForm);
            setEditingId(null);
            setFormData({ title: '', context: '', decision: '', rationale: '', outcome: '', tags: '', task_id: '' });
          }}
          className="px-3 py-1.5 bg-primary text-primary-foreground rounded-lg text-sm hover:bg-primary/90"
        >
          {showForm ? 'Cancel' : '+ New Entry'}
        </button>
      </div>

      <form onSubmit={handleSearch} className="flex gap-2">
        <input
          type="text"
          placeholder="Search decisions..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="flex-1 px-3 py-2 bg-background border rounded-lg"
        />
        <button
          type="submit"
          className="px-4 py-2 bg-secondary text-secondary-foreground rounded-lg text-sm hover:bg-secondary/90"
        >
          Search
        </button>
      </form>

      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-600 text-sm">
          {error}
        </div>
      )}

      {showForm && (
        <form onSubmit={handleSubmit} className="p-4 bg-muted rounded-lg space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <label className="block text-sm font-medium mb-1">Title *</label>
              <input
                type="text"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                className="w-full px-3 py-2 bg-background border rounded-lg"
                required
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Task ID</label>
              <input
                type="text"
                value={formData.task_id}
                onChange={(e) => setFormData({ ...formData, task_id: e.target.value })}
                className="w-full px-3 py-2 bg-background border rounded-lg"
                placeholder="Optional linked task"
              />
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Context</label>
            <textarea
              value={formData.context}
              onChange={(e) => setFormData({ ...formData, context: e.target.value })}
              className="w-full px-3 py-2 bg-background border rounded-lg"
              rows={2}
              placeholder="Background and context..."
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Decision *</label>
            <textarea
              value={formData.decision}
              onChange={(e) => setFormData({ ...formData, decision: e.target.value })}
              className="w-full px-3 py-2 bg-background border rounded-lg"
              rows={2}
              required
              placeholder="What decision was made..."
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Rationale</label>
            <textarea
              value={formData.rationale}
              onChange={(e) => setFormData({ ...formData, rationale: e.target.value })}
              className="w-full px-3 py-2 bg-background border rounded-lg"
              rows={2}
              placeholder="Why this decision was made..."
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Outcome</label>
            <textarea
              value={formData.outcome}
              onChange={(e) => setFormData({ ...formData, outcome: e.target.value })}
              className="w-full px-3 py-2 bg-background border rounded-lg"
              rows={2}
              placeholder="How it turned out..."
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Tags</label>
            <input
              type="text"
              value={formData.tags}
              onChange={(e) => setFormData({ ...formData, tags: e.target.value })}
              className="w-full px-3 py-2 bg-background border rounded-lg"
              placeholder="Comma-separated tags"
            />
          </div>
          <button
            type="submit"
            className="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm hover:bg-primary/90"
          >
            {editingId ? 'Update' : 'Create'}
          </button>
        </form>
      )}

      <div className="space-y-3">
        {entries.map((entry) => (
          <div
            key={entry.id}
            className="p-4 bg-card border rounded-lg"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="font-semibold">{entry.title}</div>
                {entry.task_id && (
                  <div className="text-xs text-muted-foreground mt-1">
                    Task: {entry.task_id}
                  </div>
                )}
                <div className="text-sm mt-2 p-2 bg-muted rounded">
                  <span className="font-medium">Decision:</span> {entry.decision}
                </div>
                {entry.context && (
                  <div className="text-sm mt-2 text-muted-foreground">
                    <span className="font-medium">Context:</span> {entry.context}
                  </div>
                )}
                {entry.rationale && (
                  <div className="text-sm mt-1 text-muted-foreground">
                    <span className="font-medium">Rationale:</span> {entry.rationale}
                  </div>
                )}
                {entry.outcome && (
                  <div className="text-sm mt-1 text-green-600">
                    <span className="font-medium">Outcome:</span> {entry.outcome}
                  </div>
                )}
                {entry.tags && (
                  <div className="flex gap-1 mt-2 flex-wrap">
                    {parseTags(entry.tags).map((tag, i) => (
                      <span key={i} className="px-2 py-0.5 bg-secondary text-secondary-foreground text-xs rounded-full">
                        {tag}
                      </span>
                    ))}
                  </div>
                )}
              </div>
              <div className="flex gap-2 ml-4">
                <button
                  onClick={() => startEdit(entry)}
                  className="p-2 text-muted-foreground hover:text-foreground"
                  title="Edit"
                >
                  ✏️
                </button>
                <button
                  onClick={() => handleDelete(entry.id)}
                  className="p-2 text-muted-foreground hover:text-red-500"
                  title="Delete"
                >
                  🗑️
                </button>
              </div>
            </div>
            <div className="text-xs text-muted-foreground mt-2">
              {new Date(entry.created_at).toLocaleString()}
            </div>
          </div>
        ))}
      </div>

      {entries.length === 0 && !showForm && (
        <div className="text-center text-muted-foreground py-8">
          No journal entries yet. Document your decisions to build institutional knowledge.
        </div>
      )}
    </div>
  );
}
