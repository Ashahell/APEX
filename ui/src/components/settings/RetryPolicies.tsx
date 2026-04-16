'use client';

import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface RetryPolicy {
  id: string;
  name: string;
  description?: string;
  max_attempts: number;
  initial_delay_secs: number;
  backoff_multiplier: number;
  max_delay_secs: number;
  jitter: boolean;
  retry_on_statuses: string[];
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

interface RetryAttempt {
  id: string;
  task_id: string;
  policy_id: string;
  attempt_number: number;
  status: string;
  error_message?: string;
  delay_used_secs: number;
  started_at: string;
  completed_at?: string;
}

export function RetryPolicies() {
  const [policies, setPolicies] = useState<RetryPolicy[]>([]);
  const [attempts, setAttempts] = useState<RetryAttempt[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [editing, setEditing] = useState<RetryPolicy | null>(null);

  // Form state
  const [formName, setFormName] = useState('');
  const [formDesc, setFormDesc] = useState('');
  const [formMaxAttempts, setFormMaxAttempts] = useState(3);
  const [formInitialDelay, setFormInitialDelay] = useState(5);
  const [formBackoff, setFormBackoff] = useState(2.0);
  const [formMaxDelay, setFormMaxDelay] = useState(300);
  const [formJitter, setFormJitter] = useState(true);
  const [formStatuses, setFormStatuses] = useState<string[]>(['failed', 'timeout']);

  const loadData = async () => {
    setLoading(true);
    try {
      const [polRes, attRes] = await Promise.all([
        apiGet('/api/v1/retry-policies'),
        apiGet('/api/v1/retry-attempts'),
      ]);
      const polData = await polRes.json();
      const attData = await attRes.json();
      setPolicies(polData);
      setAttempts(Array.isArray(attData) ? attData : []);
    } catch (e) {
      console.error('Failed to load retry policies:', e);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadData();
  }, []);

  const resetForm = () => {
    setFormName('');
    setFormDesc('');
    setFormMaxAttempts(3);
    setFormInitialDelay(5);
    setFormBackoff(2.0);
    setFormMaxDelay(300);
    setFormJitter(true);
    setFormStatuses(['failed', 'timeout']);
  };

  const handleCreate = async () => {
    try {
      await apiPost('/api/v1/retry-policies', {
        name: formName,
        description: formDesc || null,
        max_attempts: formMaxAttempts,
        initial_delay_secs: formInitialDelay,
        backoff_multiplier: formBackoff,
        max_delay_secs: formMaxDelay,
        jitter: formJitter,
        retry_on_statuses: formStatuses,
      });
      resetForm();
      setShowCreate(false);
      loadData();
    } catch (e) {
      console.error('Failed to create retry policy:', e);
    }
  };

  const handleUpdate = async () => {
    if (!editing) return;
    try {
      await apiPost(`/api/v1/retry-policies/${editing.id}`, {
        name: formName,
        description: formDesc || null,
        max_attempts: formMaxAttempts,
        initial_delay_secs: formInitialDelay,
        backoff_multiplier: formBackoff,
        max_delay_secs: formMaxDelay,
        jitter: formJitter,
        retry_on_statuses: formStatuses,
      });
      resetForm();
      setEditing(null);
      loadData();
    } catch (e) {
      console.error('Failed to update retry policy:', e);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this retry policy?')) return;
    try {
      await fetch(`/api/v1/retry-policies/${id}`, { method: 'DELETE' });
      loadData();
    } catch (e) {
      console.error('Failed to delete retry policy:', e);
    }
  };

  const openEdit = (policy: RetryPolicy) => {
    setEditing(policy);
    setFormName(policy.name);
    setFormDesc(policy.description || '');
    setFormMaxAttempts(policy.max_attempts);
    setFormInitialDelay(policy.initial_delay_secs);
    setFormBackoff(policy.backoff_multiplier);
    setFormMaxDelay(policy.max_delay_secs);
    setFormJitter(policy.jitter);
    setFormStatuses(policy.retry_on_statuses);
  };

  const toggleStatus = (status: string) => {
    if (formStatuses.includes(status)) {
      setFormStatuses(formStatuses.filter(s => s !== status));
    } else {
      setFormStatuses([...formStatuses, status]);
    }
  };

  if (loading) {
    return <div className="text-muted-foreground">Loading retry policies...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold">Task Retry Policies</h2>
          <p className="text-muted-foreground text-sm mt-1">
            Configure automatic retry behavior for failed tasks
          </p>
        </div>
        <button
          onClick={() => { resetForm(); setShowCreate(true); setEditing(null); }}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
        >
          + New Policy
        </button>
      </div>

      {/* Create/Edit Form Modal */}
      {(showCreate || editing) && (
        <div className="border rounded-lg p-4 bg-card">
          <h3 className="font-semibold mb-4">{editing ? 'Edit Policy' : 'Create Policy'}</h3>
          <div className="grid gap-4">
            <div>
              <label className="text-sm font-medium">Name</label>
              <input
                type="text"
                value={formName}
                onChange={e => setFormName(e.target.value)}
                className="w-full mt-1 px-3 py-2 rounded border bg-background"
                placeholder="Policy name"
              />
            </div>
            <div>
              <label className="text-sm font-medium">Description</label>
              <input
                type="text"
                value={formDesc}
                onChange={e => setFormDesc(e.target.value)}
                className="w-full mt-1 px-3 py-2 rounded border bg-background"
                placeholder="Optional description"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Max Attempts</label>
                <input
                  type="number"
                  value={formMaxAttempts}
                  onChange={e => setFormMaxAttempts(parseInt(e.target.value))}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  min={1}
                  max={10}
                />
              </div>
              <div>
                <label className="text-sm font-medium">Initial Delay (sec)</label>
                <input
                  type="number"
                  value={formInitialDelay}
                  onChange={e => setFormInitialDelay(parseInt(e.target.value))}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  min={1}
                />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Backoff Multiplier</label>
                <input
                  type="number"
                  value={formBackoff}
                  onChange={e => setFormBackoff(parseFloat(e.target.value))}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  step={0.1}
                  min={1}
                />
              </div>
              <div>
                <label className="text-sm font-medium">Max Delay (sec)</label>
                <input
                  type="number"
                  value={formMaxDelay}
                  onChange={e => setFormMaxDelay(parseInt(e.target.value))}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  min={1}
                />
              </div>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="jitter"
                checked={formJitter}
                onChange={e => setFormJitter(e.target.checked)}
                className="rounded"
              />
              <label htmlFor="jitter" className="text-sm">Enable Jitter (random delay variation)</label>
            </div>
            <div>
              <label className="text-sm font-medium mb-2 block">Retry On Statuses</label>
              <div className="flex gap-4">
                {['failed', 'timeout', 'error'].map(status => (
                  <label key={status} className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={formStatuses.includes(status)}
                      onChange={() => toggleStatus(status)}
                      className="rounded"
                    />
                    <span className="text-sm">{status}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>
          <div className="flex gap-2 mt-4">
            <button
              onClick={editing ? handleUpdate : handleCreate}
              className="px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90"
            >
              {editing ? 'Update' : 'Create'}
            </button>
            <button
              onClick={() => { setShowCreate(false); setEditing(null); resetForm(); }}
              className="px-4 py-2 border rounded hover:bg-muted"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Policy List */}
      <div className="border rounded-lg overflow-hidden">
        <table className="w-full">
          <thead className="bg-muted">
            <tr>
              <th className="px-4 py-2 text-left text-sm font-medium">Name</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Attempts</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Delay</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Backoff</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            {policies.length === 0 ? (
              <tr>
                <td colSpan={6} className="px-4 py-8 text-center text-muted-foreground">
                  No retry policies configured. Create one to get started.
                </td>
              </tr>
            ) : policies.map(policy => (
              <tr key={policy.id} className="border-t">
                <td className="px-4 py-3">
                  <div className="font-medium">{policy.name}</div>
                  {policy.description && (
                    <div className="text-xs text-muted-foreground">{policy.description}</div>
                  )}
                </td>
                <td className="px-4 py-3 text-sm">{policy.max_attempts}</td>
                <td className="px-4 py-3 text-sm">{policy.initial_delay_secs}s</td>
                <td className="px-4 py-3 text-sm">{policy.backoff_multiplier}x</td>
                <td className="px-4 py-3">
                  <span className={`px-2 py-1 rounded text-xs ${
                    policy.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                  }`}>
                    {policy.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </td>
                <td className="px-4 py-3">
                  <div className="flex gap-2">
                    <button
                      onClick={() => openEdit(policy)}
                      className="text-sm text-primary hover:underline"
                    >
                      Edit
                    </button>
                    <button
                      onClick={() => handleDelete(policy.id)}
                      className="text-sm text-red-500 hover:underline"
                    >
                      Delete
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Recent Attempts */}
      {attempts.length > 0 && (
        <div>
          <h3 className="font-semibold mb-2">Recent Retry Attempts</h3>
          <div className="border rounded-lg p-4 space-y-2">
            {attempts.slice(0, 10).map(attempt => (
              <div key={attempt.id} className="flex justify-between items-center text-sm">
                <div>
                  <span className="font-mono text-xs">{attempt.task_id.slice(0, 8)}...</span>
                  <span className="ml-2">Attempt #{attempt.attempt_number}</span>
                </div>
                <span className={`px-2 py-0.5 rounded text-xs ${
                  attempt.status === 'Success' ? 'bg-green-100 text-green-800' :
                  attempt.status === 'Failed' ? 'bg-red-100 text-red-800' :
                  'bg-gray-100 text-gray-600'
                }`}>
                  {attempt.status}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
