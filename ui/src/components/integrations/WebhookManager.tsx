import { useState, useEffect } from 'react';
import { apiGet, apiPost, apiDelete } from '../../lib/api';

interface Webhook {
  id: string;
  name: string;
  url: string;
  events: string[];
  enabled: boolean;
  created_at_ms: number;
  last_triggered_ms: number | null;
  failure_count: number;
}

const AVAILABLE_EVENTS = [
  'task.completed',
  'task.failed',
  'task.created',
  'task.confirmed',
];

export function WebhookManager() {
  const [webhooks, setWebhooks] = useState<Webhook[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    name: '',
    url: '',
    events: [] as string[],
    secret: '',
  });

  useEffect(() => {
    loadWebhooks();
  }, []);

  async function loadWebhooks() {
    try {
      setLoading(true);
      const response = await apiGet('/api/v1/webhooks');
      if (response.ok) {
        const data = await response.json();
        setWebhooks(data);
      }
    } catch (error) {
      console.error('Failed to load webhooks:', error);
    } finally {
      setLoading(false);
    }
  }

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    try {
      const response = await apiPost('/api/v1/webhooks', {
        name: formData.name,
        url: formData.url,
        events: formData.events,
        secret: formData.secret || null,
      });
      if (response.ok) {
        setShowForm(false);
        setFormData({ name: '', url: '', events: [], secret: '' });
        loadWebhooks();
      }
    } catch (error) {
      console.error('Failed to create webhook:', error);
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Delete this webhook?')) return;
    try {
      const response = await apiDelete(`/api/v1/webhooks/${id}`);
      if (response.ok) {
        loadWebhooks();
      }
    } catch (error) {
      console.error('Failed to delete webhook:', error);
    }
  }

  async function handleToggle(id: string) {
    try {
      const response = await apiPost(`/api/v1/webhooks/${id}/toggle`, {});
      if (response.ok) {
        loadWebhooks();
      }
    } catch (error) {
      console.error('Failed to toggle webhook:', error);
    }
  }

  function toggleEvent(event: string) {
    setFormData(prev => ({
      ...prev,
      events: prev.events.includes(event)
        ? prev.events.filter(e => e !== event)
        : [...prev.events, event]
    }));
  }

  if (loading) {
    return <div className="p-4">Loading webhooks...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-semibold mb-1">Webhooks</h2>
            <p className="text-muted-foreground">Configure external integrations</p>
          </div>
          <button
            onClick={() => setShowForm(true)}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
          >
            + Add Webhook
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {webhooks.length === 0 ? (
          <div className="text-center text-muted-foreground py-12">
            <p className="mb-2">No webhooks configured</p>
            <p className="text-sm">Add a webhook to receive notifications</p>
          </div>
        ) : (
          <div className="space-y-3">
            {webhooks.map((webhook) => (
              <div key={webhook.id} className="border rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <h3 className="font-medium">{webhook.name}</h3>
                  <span className={`text-xs px-2 py-1 rounded ${
                    webhook.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                  }`}>
                    {webhook.enabled ? 'Active' : 'Disabled'}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground mb-2 font-mono">{webhook.url}</p>
                <div className="flex flex-wrap gap-1 mb-3">
                  {webhook.events.map(event => (
                    <span key={event} className="text-xs px-2 py-0.5 bg-muted rounded">
                      {event}
                    </span>
                  ))}
                </div>
                <div className="flex items-center gap-4 text-xs text-muted-foreground mb-3">
                  <span>Failures: {webhook.failure_count}</span>
                  {webhook.last_triggered_ms && (
                    <span>Last: {new Date(webhook.last_triggered_ms).toLocaleString()}</span>
                  )}
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleToggle(webhook.id)}
                    className="px-3 py-1 text-sm border rounded hover:bg-muted"
                  >
                    {webhook.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    onClick={() => handleDelete(webhook.id)}
                    className="px-3 py-1 text-sm border rounded hover:bg-muted text-red-500"
                  >
                    Delete
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {showForm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-background border rounded-lg p-6 max-w-md w-full mx-4">
            <h3 className="text-lg font-semibold mb-4">Add Webhook</h3>
            <form onSubmit={handleCreate} className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Name</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={e => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">URL</label>
                <input
                  type="url"
                  value={formData.url}
                  onChange={e => setFormData({ ...formData, url: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background"
                  placeholder="https://example.com/webhook"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Events</label>
                <div className="flex flex-wrap gap-2">
                  {AVAILABLE_EVENTS.map(event => (
                    <button
                      key={event}
                      type="button"
                      onClick={() => toggleEvent(event)}
                      className={`px-2 py-1 text-xs rounded border ${
                        formData.events.includes(event)
                          ? 'bg-primary text-primary-foreground'
                          : 'bg-muted'
                      }`}
                    >
                      {event}
                    </button>
                  ))}
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Secret (optional)</label>
                <input
                  type="password"
                  value={formData.secret}
                  onChange={e => setFormData({ ...formData, secret: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background"
                  placeholder="Webhook secret for verification"
                />
              </div>
              <div className="flex gap-2 justify-end">
                <button
                  type="button"
                  onClick={() => setShowForm(false)}
                  className="px-4 py-2 border rounded-lg hover:bg-muted"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
                >
                  Create
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
