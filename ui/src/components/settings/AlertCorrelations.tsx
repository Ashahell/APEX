'use client';

import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface CorrelationRule {
  id: string;
  name: string;
  description?: string;
  condition: {
    source_pattern?: string;
    message_pattern?: string;
    severity_in?: string[];
    time_window_secs?: number;
  };
  action: {
    suppress: boolean;
    aggregate: boolean;
    notify: boolean;
    auto_resolve: boolean;
  };
  enabled: boolean;
  priority: number;
  created_at: string;
  updated_at: string;
}

interface AlertGroup {
  id: string;
  rule_id: string;
  group_key: string;
  alert_count: number;
  first_seen: string;
  last_seen: string;
  resolved: boolean;
  resolved_at?: string;
}

export function AlertCorrelations() {
  const [rules, setRules] = useState<CorrelationRule[]>([]);
  const [groups, setGroups] = useState<AlertGroup[]>([]);
  const [stats, setStats] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [activeTab, setActiveTab] = useState<'rules' | 'groups'>('rules');

  // Form state
  const [formName, setFormName] = useState('');
  const [formDesc, setFormDesc] = useState('');
  const [formSourcePattern, setFormSourcePattern] = useState('');
  const [formMessagePattern, setFormMessagePattern] = useState('');
  const [formSeverities, setFormSeverities] = useState<string[]>(['critical', 'high', 'medium', 'low']);
  const [formTimeWindow, setFormTimeWindow] = useState(300);
  const [formSuppress, setFormSuppress] = useState(false);
  const [formAggregate, setFormAggregate] = useState(true);
  const [formNotify, setFormNotify] = useState(true);
  const [formAutoResolve, setFormAutoResolve] = useState(false);

  const loadData = async () => {
    setLoading(true);
    try {
      const [rulesRes, groupsRes, statsRes] = await Promise.all([
        apiGet('/api/v1/alert-correlations'),
        apiGet('/api/v1/alert-correlations/groups'),
        apiGet('/api/v1/alert-correlations/stats'),
      ]);
      const rulesData = await rulesRes.json();
      const groupsData = await groupsRes.json();
      const statsData = await statsRes.json();
      setRules(rulesData);
      setGroups(Array.isArray(groupsData) ? groupsData : []);
      setStats(statsData);
    } catch (e) {
      console.error('Failed to load correlation data:', e);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadData();
  }, []);

  const resetForm = () => {
    setFormName('');
    setFormDesc('');
    setFormSourcePattern('');
    setFormMessagePattern('');
    setFormSeverities(['critical', 'high', 'medium', 'low']);
    setFormTimeWindow(300);
    setFormSuppress(false);
    setFormAggregate(true);
    setFormNotify(true);
    setFormAutoResolve(false);
  };

  const handleCreate = async () => {
    try {
      await apiPost('/api/v1/alert-correlations', {
        name: formName,
        description: formDesc || null,
        condition: {
          source_pattern: formSourcePattern || null,
          message_pattern: formMessagePattern || null,
          severity_in: formSeverities,
          time_window_secs: formTimeWindow,
        },
        action: {
          suppress: formSuppress,
          aggregate: formAggregate,
          notify: formNotify,
          auto_resolve: formAutoResolve,
        },
      });
      resetForm();
      setShowCreate(false);
      loadData();
    } catch (e) {
      console.error('Failed to create correlation rule:', e);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this correlation rule?')) return;
    try {
      await fetch(`/api/v1/alert-correlations/${id}`, { method: 'DELETE' });
      loadData();
    } catch (e) {
      console.error('Failed to delete correlation rule:', e);
    }
  };

  const handleResolveGroup = async (id: string) => {
    try {
      await fetch(`/api/v1/alert-correlations/groups/${id}/resolve`, { method: 'POST' });
      loadData();
    } catch (e) {
      console.error('Failed to resolve group:', e);
    }
  };

  const toggleSeverity = (severity: string) => {
    if (formSeverities.includes(severity)) {
      setFormSeverities(formSeverities.filter(s => s !== severity));
    } else {
      setFormSeverities([...formSeverities, severity]);
    }
  };

  if (loading) {
    return <div className="text-muted-foreground">Loading alert correlations...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold">Alert Correlations</h2>
          <p className="text-muted-foreground text-sm mt-1">
            Reduce alert fatigue by grouping related alerts
          </p>
        </div>
        <button
          onClick={() => { resetForm(); setShowCreate(true); }}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
        >
          + New Rule
        </button>
      </div>

      {/* Stats */}
      {stats && (
        <div className="grid grid-cols-4 gap-4">
          <div className="border rounded-lg p-4">
            <div className="text-2xl font-bold">{stats.total_rules}</div>
            <div className="text-sm text-muted-foreground">Total Rules</div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-2xl font-bold">{stats.active_groups}</div>
            <div className="text-sm text-muted-foreground">Active Groups</div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-2xl font-bold">{stats.alerts_processed}</div>
            <div className="text-sm text-muted-foreground">Alerts Processed</div>
          </div>
          <div className="border rounded-lg p-4">
            <div className="text-2xl font-bold">{stats.alerts_suppressed}</div>
            <div className="text-sm text-muted-foreground">Alerts Suppressed</div>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="flex border-b">
        <button
          onClick={() => setActiveTab('rules')}
          className={`px-4 py-2 ${activeTab === 'rules' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground'}`}
        >
          Correlation Rules
        </button>
        <button
          onClick={() => setActiveTab('groups')}
          className={`px-4 py-2 ${activeTab === 'groups' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground'}`}
        >
          Alert Groups
        </button>
      </div>

      {/* Create Form */}
      {showCreate && (
        <div className="border rounded-lg p-4 bg-card">
          <h3 className="font-semibold mb-4">Create Correlation Rule</h3>
          <div className="grid gap-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Name</label>
                <input
                  type="text"
                  value={formName}
                  onChange={e => setFormName(e.target.value)}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  placeholder="Rule name"
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
            </div>

            <div className="border-t pt-4">
              <h4 className="font-medium mb-2">Conditions</h4>
              <div className="grid gap-4">
                <div>
                  <label className="text-sm font-medium">Source Pattern</label>
                  <input
                    type="text"
                    value={formSourcePattern}
                    onChange={e => setFormSourcePattern(e.target.value)}
                    className="w-full mt-1 px-3 py-2 rounded border bg-background"
                    placeholder="e.g., api-gateway, database"
                  />
                </div>
                <div>
                  <label className="text-sm font-medium">Message Pattern</label>
                  <input
                    type="text"
                    value={formMessagePattern}
                    onChange={e => setFormMessagePattern(e.target.value)}
                    className="w-full mt-1 px-3 py-2 rounded border bg-background"
                    placeholder="e.g., timeout, error"
                  />
                </div>
                <div>
                  <label className="text-sm font-medium mb-2 block">Severities</label>
                  <div className="flex gap-4">
                    {['critical', 'high', 'medium', 'low'].map(severity => (
                      <label key={severity} className="flex items-center gap-2">
                        <input
                          type="checkbox"
                          checked={formSeverities.includes(severity)}
                          onChange={() => toggleSeverity(severity)}
                          className="rounded"
                        />
                        <span className="text-sm">{severity}</span>
                      </label>
                    ))}
                  </div>
                </div>
                <div>
                  <label className="text-sm font-medium">Time Window (sec)</label>
                  <input
                    type="number"
                    value={formTimeWindow}
                    onChange={e => setFormTimeWindow(parseInt(e.target.value))}
                    className="w-full mt-1 px-3 py-2 rounded border bg-background"
                    min={60}
                  />
                </div>
              </div>
            </div>

            <div className="border-t pt-4">
              <h4 className="font-medium mb-2">Actions</h4>
              <div className="space-y-2">
                {[
                  { key: 'suppress', label: 'Suppress', desc: "Don't show individual alerts" },
                  { key: 'aggregate', label: 'Aggregate', desc: 'Group similar alerts together' },
                  { key: 'notify', label: 'Notify', desc: 'Send notification when group forms' },
                  { key: 'auto_resolve', label: 'Auto-resolve', desc: 'Auto-resolve after time window' },
                ].map(({ key, label, desc }) => (
                  <label key={key} className="flex items-center gap-3">
                    <input
                      type="checkbox"
                      checked={key === 'suppress' ? formSuppress : key === 'aggregate' ? formAggregate : key === 'notify' ? formNotify : formAutoResolve}
                      onChange={e => {
                        if (key === 'suppress') setFormSuppress(e.target.checked);
                        if (key === 'aggregate') setFormAggregate(e.target.checked);
                        if (key === 'notify') setFormNotify(e.target.checked);
                        if (key === 'auto_resolve') setFormAutoResolve(e.target.checked);
                      }}
                      className="rounded"
                    />
                    <div>
                      <span className="text-sm font-medium">{label}</span>
                      <span className="text-xs text-muted-foreground ml-2">{desc}</span>
                    </div>
                  </label>
                ))}
              </div>
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

      {/* Rules List */}
      {activeTab === 'rules' && (
        <div className="border rounded-lg overflow-hidden">
          <table className="w-full">
            <thead className="bg-muted">
              <tr>
                <th className="px-4 py-2 text-left text-sm font-medium">Name</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Conditions</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {rules.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                    No correlation rules configured.
                  </td>
                </tr>
              ) : rules.map(rule => (
                <tr key={rule.id} className="border-t">
                  <td className="px-4 py-3">
                    <div className="font-medium">{rule.name}</div>
                    {rule.description && (
                      <div className="text-xs text-muted-foreground">{rule.description}</div>
                    )}
                  </td>
                  <td className="px-4 py-3 text-sm">
                    {rule.condition.source_pattern && <span>Source: {rule.condition.source_pattern}</span>}
                    {rule.condition.message_pattern && <span> Msg: {rule.condition.message_pattern}</span>}
                  </td>
                  <td className="px-4 py-3 text-sm">
                    {Object.entries(rule.action).filter(([, v]) => v).map(([k]) => k).join(', ')}
                  </td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 rounded text-xs ${
                      rule.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                    }`}>
                      {rule.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <button
                      onClick={() => handleDelete(rule.id)}
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
      )}

      {/* Groups List */}
      {activeTab === 'groups' && (
        <div className="border rounded-lg overflow-hidden">
          <table className="w-full">
            <thead className="bg-muted">
              <tr>
                <th className="px-4 py-2 text-left text-sm font-medium">Group Key</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Alert Count</th>
                <th className="px-4 py-2 text-left text-sm font-medium">First Seen</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Last Seen</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {groups.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-4 py-8 text-center text-muted-foreground">
                    No alert groups yet. Groups form when alerts match correlation rules.
                  </td>
                </tr>
              ) : groups.map(group => (
                <tr key={group.id} className="border-t">
                  <td className="px-4 py-3 font-mono text-sm">{group.group_key}</td>
                  <td className="px-4 py-3 text-sm">{group.alert_count}</td>
                  <td className="px-4 py-3 text-sm">{new Date(group.first_seen).toLocaleString()}</td>
                  <td className="px-4 py-3 text-sm">{new Date(group.last_seen).toLocaleString()}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 rounded text-xs ${
                      group.resolved ? 'bg-gray-100 text-gray-600' : 'bg-yellow-100 text-yellow-800'
                    }`}>
                      {group.resolved ? 'Resolved' : 'Active'}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    {!group.resolved && (
                      <button
                        onClick={() => handleResolveGroup(group.id)}
                        className="text-sm text-primary hover:underline"
                      >
                        Resolve
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
