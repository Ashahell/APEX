'use client';

import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface FilterCondition {
  field: string;
  operator: 'equals' | 'not_equals' | 'contains' | 'not_contains' | 'starts_with' | 'ends_with' | 'regex' | 'in' | 'not_in' | 'greater_than' | 'less_than';
  value: string;
}

interface WebhookFilter {
  id: string;
  name: string;
  webhook_id: string;
  event_types: string[];
  conditions: FilterCondition[];
  condition_logic: 'all' | 'any';
  action: 'allow' | 'block' | 'transform';
  enabled: boolean;
  priority: number;
  created_at: string;
  updated_at: string;
}

const OPERATORS = [
  { value: 'equals', label: 'Equals' },
  { value: 'not_equals', label: 'Not Equals' },
  { value: 'contains', label: 'Contains' },
  { value: 'not_contains', label: 'Not Contains' },
  { value: 'starts_with', label: 'Starts With' },
  { value: 'ends_with', label: 'Ends With' },
  { value: 'regex', label: 'Regex' },
  { value: 'in', label: 'In' },
  { value: 'not_in', label: 'Not In' },
  { value: 'greater_than', label: 'Greater Than' },
  { value: 'less_than', label: 'Less Than' },
];

const EVENT_TYPES = ['task.created', 'task.updated', 'task.completed', 'task.failed', 'alert.created', 'alert.resolved', 'webhook.received'];

export function WebhookEventFilters() {
  const [filters, setFilters] = useState<WebhookFilter[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);

  // Form state
  const [formName, setFormName] = useState('');
  const [formWebhookId, setFormWebhookId] = useState('');
  const [formEventTypes, setFormEventTypes] = useState<string[]>([]);
  const [formLogic, setFormLogic] = useState<'all' | 'any'>('all');
  const [formAction, setFormAction] = useState<'allow' | 'block' | 'transform'>('allow');
  const [formConditions, setFormConditions] = useState<FilterCondition[]>([]);

  const loadData = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/webhook-filters');
      const data = await res.json();
      setFilters(Array.isArray(data) ? data : []);
    } catch (e) {
      console.error('Failed to load webhook filters:', e);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadData();
  }, []);

  const resetForm = () => {
    setFormName('');
    setFormWebhookId('');
    setFormEventTypes([]);
    setFormLogic('all');
    setFormAction('allow');
    setFormConditions([]);
  };

  const handleCreate = async () => {
    try {
      await apiPost('/api/v1/webhook-filters', {
        name: formName,
        webhook_id: formWebhookId,
        event_types: formEventTypes,
        conditions: formConditions,
        condition_logic: formLogic,
        action: formAction,
      });
      resetForm();
      setShowCreate(false);
      loadData();
    } catch (e) {
      console.error('Failed to create webhook filter:', e);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this webhook filter?')) return;
    try {
      await fetch(`/api/v1/webhook-filters/${id}`, { method: 'DELETE' });
      loadData();
    } catch (e) {
      console.error('Failed to delete webhook filter:', e);
    }
  };

  const handleToggle = async (filter: WebhookFilter) => {
    try {
      await apiPost(`/api/v1/webhook-filters/${filter.id}`, {
        enabled: !filter.enabled,
      });
      loadData();
    } catch (e) {
      console.error('Failed to toggle webhook filter:', e);
    }
  };

  const toggleEventType = (type: string) => {
    if (formEventTypes.includes(type)) {
      setFormEventTypes(formEventTypes.filter(t => t !== type));
    } else {
      setFormEventTypes([...formEventTypes, type]);
    }
  };

  const addCondition = () => {
    setFormConditions([...formConditions, { field: '', operator: 'equals', value: '' }]);
  };

  const updateCondition = (index: number, updates: Partial<FilterCondition>) => {
    const updated = [...formConditions];
    updated[index] = { ...updated[index], ...updates };
    setFormConditions(updated);
  };

  const removeCondition = (index: number) => {
    setFormConditions(formConditions.filter((_, i) => i !== index));
  };

  if (loading) {
    return <div className="text-muted-foreground">Loading webhook filters...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold">Webhook Event Filters</h2>
          <p className="text-muted-foreground text-sm mt-1">
            Filter and transform webhook events before processing
          </p>
        </div>
        <button
          onClick={() => { resetForm(); setShowCreate(true); }}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
        >
          + New Filter
        </button>
      </div>

      {/* Create Form */}
      {showCreate && (
        <div className="border rounded-lg p-4 bg-card">
          <h3 className="font-semibold mb-4">Create Webhook Filter</h3>
          <div className="grid gap-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Name</label>
                <input
                  type="text"
                  value={formName}
                  onChange={e => setFormName(e.target.value)}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  placeholder="Filter name"
                />
              </div>
              <div>
                <label className="text-sm font-medium">Webhook ID</label>
                <input
                  type="text"
                  value={formWebhookId}
                  onChange={e => setFormWebhookId(e.target.value)}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  placeholder="webhook-123"
                />
              </div>
            </div>

            <div>
              <label className="text-sm font-medium mb-2 block">Event Types</label>
              <div className="flex flex-wrap gap-2">
                {EVENT_TYPES.map(type => (
                  <label key={type} className="flex items-center gap-2 px-3 py-1 border rounded cursor-pointer hover:bg-muted">
                    <input
                      type="checkbox"
                      checked={formEventTypes.includes(type)}
                      onChange={() => toggleEventType(type)}
                      className="rounded"
                    />
                    <span className="text-sm font-mono">{type}</span>
                  </label>
                ))}
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Condition Logic</label>
                <select
                  value={formLogic}
                  onChange={e => setFormLogic(e.target.value as 'all' | 'any')}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                >
                  <option value="all">All conditions (AND)</option>
                  <option value="any">Any condition (OR)</option>
                </select>
              </div>
              <div>
                <label className="text-sm font-medium">Action</label>
                <select
                  value={formAction}
                  onChange={e => setFormAction(e.target.value as 'allow' | 'block' | 'transform')}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                >
                  <option value="allow">Allow</option>
                  <option value="block">Block</option>
                  <option value="transform">Transform</option>
                </select>
              </div>
            </div>

            <div className="border-t pt-4">
              <div className="flex justify-between items-center mb-2">
                <label className="text-sm font-medium">Conditions</label>
                <button
                  onClick={addCondition}
                  className="text-sm text-primary hover:underline"
                >
                  + Add Condition
                </button>
              </div>
              {formConditions.length === 0 ? (
                <p className="text-sm text-muted-foreground">No conditions. All matching events will be {formAction}.</p>
              ) : (
                <div className="space-y-2">
                  {formConditions.map((cond, idx) => (
                    <div key={idx} className="flex gap-2 items-center">
                      <input
                        type="text"
                        value={cond.field}
                        onChange={e => updateCondition(idx, { field: e.target.value })}
                        className="flex-1 px-3 py-2 rounded border bg-background text-sm"
                        placeholder="Field"
                      />
                      <select
                        value={cond.operator}
                        onChange={e => updateCondition(idx, { operator: e.target.value as FilterCondition['operator'] })}
                        className="px-3 py-2 rounded border bg-background text-sm"
                      >
                        {OPERATORS.map(op => (
                          <option key={op.value} value={op.value}>{op.label}</option>
                        ))}
                      </select>
                      <input
                        type="text"
                        value={cond.value}
                        onChange={e => updateCondition(idx, { value: e.target.value })}
                        className="flex-1 px-3 py-2 rounded border bg-background text-sm"
                        placeholder="Value"
                      />
                      <button
                        onClick={() => removeCondition(idx)}
                        className="text-red-500 hover:underline text-sm"
                      >
                        Remove
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          <div className="flex gap-2 mt-4">
            <button
              onClick={handleCreate}
              className="px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90"
            >
              Create
            </button>
            <button
              onClick={() => { setShowCreate(false); resetForm(); }}
              className="px-4 py-2 border rounded hover:bg-muted"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Filters List */}
      <div className="border rounded-lg overflow-hidden">
        <table className="w-full">
          <thead className="bg-muted">
            <tr>
              <th className="px-4 py-2 text-left text-sm font-medium">Name</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Webhook</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Event Types</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Conditions</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Action</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
              <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
            </tr>
          </thead>
          <tbody>
            {filters.length === 0 ? (
              <tr>
                <td colSpan={7} className="px-4 py-8 text-center text-muted-foreground">
                  No webhook filters configured. Create one to start filtering events.
                </td>
              </tr>
            ) : filters.map(filter => (
              <tr key={filter.id} className="border-t">
                <td className="px-4 py-3">
                  <div className="font-medium">{filter.name}</div>
                  <div className="text-xs text-muted-foreground">Priority: {filter.priority}</div>
                </td>
                <td className="px-4 py-3 font-mono text-sm">{filter.webhook_id}</td>
                <td className="px-4 py-3 text-sm">
                  <div className="flex flex-wrap gap-1">
                    {filter.event_types.slice(0, 3).map(type => (
                      <span key={type} className="px-1.5 py-0.5 bg-muted rounded text-xs">{type}</span>
                    ))}
                    {filter.event_types.length > 3 && (
                      <span className="text-xs text-muted-foreground">+{filter.event_types.length - 3}</span>
                    )}
                  </div>
                </td>
                <td className="px-4 py-3 text-sm">
                  {filter.conditions.length} condition{filter.conditions.length !== 1 ? 's' : ''}
                  <span className="text-muted-foreground ml-1">({filter.condition_logic})</span>
                </td>
                <td className="px-4 py-3">
                  <span className={`px-2 py-1 rounded text-xs ${
                    filter.action === 'allow' ? 'bg-green-100 text-green-800' :
                    filter.action === 'block' ? 'bg-red-100 text-red-800' :
                    'bg-blue-100 text-blue-800'
                  }`}>
                    {filter.action}
                  </span>
                </td>
                <td className="px-4 py-3">
                  <button
                    onClick={() => handleToggle(filter)}
                    className={`px-2 py-1 rounded text-xs cursor-pointer ${
                      filter.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                    }`}
                  >
                    {filter.enabled ? 'Enabled' : 'Disabled'}
                  </button>
                </td>
                <td className="px-4 py-3">
                  <button
                    onClick={() => handleDelete(filter.id)}
                    className="text-sm text-red-500 hover:underline"
                  >
                    Delete
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
