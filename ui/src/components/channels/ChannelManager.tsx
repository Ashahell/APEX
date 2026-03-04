import { useState, useEffect } from 'react';
import { listChannels, createChannel, updateChannel, deleteChannel, Channel } from '../../lib/api';

export function ChannelManager() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formData, setFormData] = useState({ name: '', description: '' });

  useEffect(() => {
    loadChannels();
  }, []);

  async function loadChannels() {
    try {
      setLoading(true);
      const data = await listChannels();
      setChannels(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load channels');
    } finally {
      setLoading(false);
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      if (editingId) {
        await updateChannel(editingId, formData.name, formData.description || undefined);
      } else {
        await createChannel(formData.name, formData.description || undefined);
      }
      setFormData({ name: '', description: '' });
      setShowForm(false);
      setEditingId(null);
      loadChannels();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to save channel');
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this channel?')) return;
    try {
      await deleteChannel(id);
      loadChannels();
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to delete channel');
    }
  }

  function startEdit(channel: Channel) {
    setEditingId(channel.id);
    setFormData({ name: channel.name, description: channel.description || '' });
    setShowForm(true);
  }

  if (loading) {
    return (
      <div className="p-4 text-center text-muted-foreground">
        Loading channels...
      </div>
    );
  }

  return (
    <div className="p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Channels</h2>
        <button
          onClick={() => {
            setShowForm(!showForm);
            setEditingId(null);
            setFormData({ name: '', description: '' });
          }}
          className="px-3 py-1.5 bg-primary text-primary-foreground rounded-lg text-sm hover:bg-primary/90"
        >
          {showForm ? 'Cancel' : '+ New Channel'}
        </button>
      </div>

      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-600 text-sm">
          {error}
        </div>
      )}

      {showForm && (
        <form onSubmit={handleSubmit} className="p-4 bg-muted rounded-lg space-y-3">
          <div>
            <label className="block text-sm font-medium mb-1">Name</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 bg-background border rounded-lg"
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              className="w-full px-3 py-2 bg-background border rounded-lg"
              rows={2}
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

      <div className="space-y-2">
        {channels.map((channel) => (
          <div
            key={channel.id}
            className="p-3 bg-card border rounded-lg flex items-center justify-between"
          >
            <div>
              <div className="font-medium">{channel.name}</div>
              {channel.description && (
                <div className="text-sm text-muted-foreground">{channel.description}</div>
              )}
              <div className="text-xs text-muted-foreground mt-1">
                Created: {new Date(channel.created_at).toLocaleDateString()}
              </div>
            </div>
            <div className="flex gap-2">
              <button
                onClick={() => startEdit(channel)}
                className="p-2 text-muted-foreground hover:text-foreground"
                title="Edit"
              >
                ✏️
              </button>
              <button
                onClick={() => handleDelete(channel.id)}
                className="p-2 text-muted-foreground hover:text-red-500"
                title="Delete"
              >
                🗑️
              </button>
            </div>
          </div>
        ))}
      </div>

      {channels.length === 0 && !showForm && (
        <div className="text-center text-muted-foreground py-8">
          No channels yet. Create one to get started.
        </div>
      )}
    </div>
  );
}
