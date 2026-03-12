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
          Loading webhooks...
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-[var(--color-border)] p-4 bg-[var(--color-panel)]">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
              </svg>
            </div>
            <div>
              <h2 className="text-xl font-semibold">Webhooks</h2>
              <p className="text-sm text-[var(--color-text-muted)]">Configure external integrations</p>
            </div>
          </div>
          <button
            onClick={() => setShowForm(true)}
            className="px-4 py-2 bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
            Add Webhook
          </button>
        </div>
      </div>

      {/* Webhooks List */}
      <div className="flex-1 overflow-auto p-4">
        {webhooks.length === 0 ? (
          <div className="text-center text-[var(--color-text-muted)] py-12">
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--color-muted)] flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
              </svg>
            </div>
            <p className="mb-2 font-medium">No webhooks configured</p>
            <p className="text-sm">Add a webhook to receive notifications</p>
          </div>
        ) : (
          <div className="space-y-3">
            {webhooks.map((webhook) => (
              <div key={webhook.id} className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)] hover:border-[#4248f1]/30 transition-colors">
                <div className="flex items-center justify-between mb-2">
                  <h3 className="font-medium">{webhook.name}</h3>
                  <span className={`text-xs px-3 py-1 rounded-full font-medium ${
                    webhook.enabled 
                      ? 'bg-green-500/10 text-green-500 border border-green-500/20' 
                      : 'bg-[var(--color-muted)] text-[var(--color-text-muted)]'
                  }`}>
                    {webhook.enabled ? 'Active' : 'Disabled'}
                  </span>
                </div>
                <p className="text-sm text-[var(--color-text-muted)] mb-3 font-mono">{webhook.url}</p>
                <div className="flex flex-wrap gap-1 mb-3">
                  {webhook.events.map(event => (
                    <span key={event} className="text-xs px-2 py-1 bg-[var(--color-muted)] rounded-full">
                      {event}
                    </span>
                  ))}
                </div>
                <div className="flex items-center gap-4 text-xs text-[var(--color-text-muted)] mb-3">
                  <span>Failures: {webhook.failure_count}</span>
                  {webhook.last_triggered_ms && (
                    <span>Last: {new Date(webhook.last_triggered_ms).toLocaleString()}</span>
                  )}
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => handleToggle(webhook.id)}
                    className="px-3 py-1.5 text-sm border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors"
                  >
                    {webhook.enabled ? 'Disable' : 'Enable'}
                  </button>
                  <button
                    onClick={() => handleDelete(webhook.id)}
                    className="px-3 py-1.5 text-sm border border-[var(--color-border)] rounded-lg hover:bg-red-500/10 hover:text-red-500 transition-colors"
                  >
                    Delete
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Add Form Modal */}
      {showForm && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50 backdrop-blur-sm" onClick={() => setShowForm(false)}>
          <div className="bg-[var(--color-panel)] border border-[var(--color-border)] rounded-xl p-6 max-w-md w-full mx-4 shadow-2xl" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
              Add Webhook
            </h3>
            <form onSubmit={handleCreate} className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1.5">Name</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={e => setFormData({ ...formData, name: e.target.value })}
                  className="w-full px-3 py-2.5 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)]"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1.5">URL</label>
                <input
                  type="url"
                  value={formData.url}
                  onChange={e => setFormData({ ...formData, url: e.target.value })}
                  className="w-full px-3 py-2.5 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)]"
                  placeholder="https://example.com/webhook"
                  required
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1.5">Events</label>
                <div className="flex flex-wrap gap-2">
                  {AVAILABLE_EVENTS.map(event => (
                    <button
                      key={event}
                      type="button"
                      onClick={() => toggleEvent(event)}
                      className={`px-3 py-1.5 text-xs rounded-lg border transition-colors ${
                        formData.events.includes(event)
                          ? 'bg-[#4248f1] text-white border-[#4248f1]'
                          : 'bg-[var(--color-muted)] border-[var(--color-border)]'
                      }`}
                    >
                      {event}
                    </button>
                  ))}
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium mb-1.5">Secret (optional)</label>
                <input
                  type="password"
                  value={formData.secret}
                  onChange={e => setFormData({ ...formData, secret: e.target.value })}
                  className="w-full px-3 py-2.5 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)]"
                  placeholder="Webhook secret for verification"
                />
              </div>
              <div className="flex gap-2 justify-end pt-2">
                <button
                  type="button"
                  onClick={() => setShowForm(false)}
                  className="px-4 py-2 border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="px-4 py-2 bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] transition-colors"
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
