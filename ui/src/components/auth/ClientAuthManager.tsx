import { useState, useEffect } from 'react';
import { apiGet, apiPost, apiDelete } from '../../lib/api';

interface Client {
  client_id: string;
  client_name: string;
  created_at: string;
  last_used: string | null;
  rate_limit: number;
}

export function ClientAuthManager() {
  const [clients, setClients] = useState<Client[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newClientName, setNewClientName] = useState('');
  const [newClientRateLimit, setNewClientRateLimit] = useState(60);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    loadClients();
  }, []);

  const loadClients = async () => {
    try {
      const res = await apiGet('/api/v1/clients');
      if (res.ok) {
        const data = await res.json();
        setClients(data);
      }
    } catch (err) {
      console.error('Failed to load clients:', err);
    } finally {
      setLoading(false);
    }
  };

  const createClient = async () => {
    if (!newClientName.trim()) {
      setError('Client name is required');
      return;
    }
    setCreating(true);
    setError(null);
    try {
      const res = await apiPost('/api/v1/clients', {
        client_name: newClientName,
        rate_limit: newClientRateLimit,
      });
      if (res.ok) {
        setSuccess('Client created successfully');
        setNewClientName('');
        setNewClientRateLimit(60);
        setShowCreate(false);
        await loadClients();
      } else {
        setError('Failed to create client');
      }
    } catch (err) {
      setError('Failed to create client');
    } finally {
      setCreating(false);
    }
  };

  const deleteClient = async (clientId: string) => {
    if (!confirm('Are you sure you want to delete this client?')) return;
    try {
      const res = await apiDelete(`/api/v1/clients/${clientId}`);
      if (res.ok) {
        setSuccess('Client deleted');
        await loadClients();
      }
    } catch (err) {
      setError('Failed to delete client');
    }
  };

  const formatDate = (dateStr: string | null) => {
    if (!dateStr) return 'Never';
    return new Date(dateStr).toLocaleString();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading...</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-4">
      <div className="max-w-4xl mx-auto space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold" style={{ color: '#4248f1' }}>Client Authentication</h2>
            <p className="text-sm text-[var(--color-text-muted)]">
              Manage API clients with per-client secrets and rate limiting
            </p>
          </div>
          <button
            onClick={() => setShowCreate(!showCreate)}
            className="px-4 py-2 rounded-xl bg-[#4248f1] text-white hover:bg-[#4248f1]/90 transition-colors"
          >
            {showCreate ? 'Cancel' : 'New Client'}
          </button>
        </div>

        {error && (
          <div className="p-3 bg-red-500/20 text-red-500 rounded-xl border border-red-500/30">
            {error}
          </div>
        )}

        {success && (
          <div className="p-3 bg-green-500/20 text-green-500 rounded-xl border border-green-500/30">
            {success}
          </div>
        )}

        {showCreate && (
          <div className="border border-border rounded-xl p-4 space-y-4 bg-[var(--color-panel)]">
            <h3 className="font-semibold" style={{ color: '#4248f1' }}>Create New Client</h3>
            <div>
              <label className="block text-sm font-medium mb-1">Client Name</label>
              <input
                type="text"
                value={newClientName}
                onChange={(e) => setNewClientName(e.target.value)}
                placeholder="My API Client"
                className="w-full px-3 py-2 rounded-xl border border-border bg-[var(--color-background)] focus:ring-2 focus:ring-[#4248f1]/50 focus:border-[#4248f1] outline-none transition-colors"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Rate Limit (requests/min)</label>
              <input
                type="number"
                value={newClientRateLimit}
                onChange={(e) => setNewClientRateLimit(parseInt(e.target.value) || 60)}
                min={1}
                max={10000}
                className="w-full px-3 py-2 rounded-xl border border-border bg-[var(--color-background)] focus:ring-2 focus:ring-[#4248f1]/50 focus:border-[#4248f1] outline-none transition-colors"
              />
            </div>
            <button
              onClick={createClient}
              disabled={creating}
              className="w-full px-4 py-2 rounded-xl bg-[#4248f1] text-white hover:bg-[#4248f1]/90 disabled:opacity-50 transition-colors"
            >
              {creating ? 'Creating...' : 'Create Client'}
            </button>
          </div>
        )}

        <div className="border border-border rounded-xl overflow-hidden">
          <div className="border-b border-border p-3 bg-[var(--color-panel)]">
            <h3 className="font-semibold" style={{ color: '#4248f1' }}>Registered Clients ({clients.length})</h3>
          </div>
          <div className="divide-y divide-border">
            {clients.length === 0 ? (
              <div className="p-8 text-center text-[var(--color-text-muted)]">
                No clients registered
              </div>
            ) : (
              clients.map((client) => (
                <div key={client.client_id} className="p-4 flex items-center justify-between hover:bg-[#4248f1]/5 transition-colors">
                  <div>
                    <div className="font-medium">{client.client_name}</div>
                    <div className="text-sm text-[var(--color-text-muted)] font-mono">
                      {client.client_id.slice(0, 8)}...
                    </div>
                    <div className="text-xs text-[var(--color-text-muted)] mt-1">
                      Created: {formatDate(client.created_at)} • 
                      Last used: {formatDate(client.last_used)} •
                      Rate: {client.rate_limit}/min
                    </div>
                  </div>
                  <button
                    onClick={() => deleteClient(client.client_id)}
                    className="px-3 py-1 rounded-xl border border-red-500/30 text-red-500 hover:bg-red-500/20 transition-colors"
                  >
                    Delete
                  </button>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="border border-border rounded-xl p-4 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-2" style={{ color: '#4248f1' }}>About Client Authentication</h3>
          <p className="text-sm text-[var(--color-text-muted)]">
            Each client has its own API key for authentication. Rate limiting helps prevent
            abuse. Client secrets are rotated by creating new clients and deleting old ones.
          </p>
        </div>
      </div>
    </div>
  );
}
