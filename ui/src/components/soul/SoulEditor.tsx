import { useState, useEffect } from 'react';
import { apiGet, apiPut } from '../../lib/api';

interface SoulIdentity {
  name: string;
  purpose: string;
  values: string[];
  capabilities: string[];
  autonomy_config: {
    heartbeat_interval_minutes: number;
    max_actions_per_wake: number;
    require_approval_t1_plus: boolean;
  };
  goals: { text: string; priority: string; status: string }[];
  relationships: { entity: string; trust: number }[];
  wake_count: number;
}

export function SoulEditor() {
  const [identity, setIdentity] = useState<SoulIdentity | null>(null);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState(false);
  const [editedContent, setEditedContent] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSoul();
  }, []);

  const loadSoul = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/soul');
      if (res.ok) {
        const data = await res.json();
        setIdentity(data);
        setEditedContent(JSON.stringify(data, null, 2));
      }
    } catch (err) {
      console.error('Failed to load SOUL:', err);
    } finally {
      setLoading(false);
    }
  };

  const saveSoul = async () => {
    setSaving(true);
    setError(null);
    try {
      const res = await apiPut('/api/v1/soul', { content: editedContent });
      if (!res.ok) {
        throw new Error('Failed to save SOUL');
      }
      setEditing(false);
      await loadSoul();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Save failed');
    } finally {
      setSaving(false);
    }
  };

  const renderSoulView = () => {
    if (!identity) return null;

    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold">{identity.name}</h2>
            <p className="text-sm text-muted-foreground">Wake Count: {identity.wake_count}</p>
          </div>
          <button
            onClick={() => setEditing(true)}
            className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
          >
            Edit SOUL
          </button>
        </div>

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">Purpose</h3>
          <p className="text-muted-foreground">{identity.purpose}</p>
        </div>

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">Values</h3>
          <div className="flex flex-wrap gap-2">
            {identity.values.map((value) => (
              <span key={value} className="px-3 py-1 bg-secondary rounded-full text-sm">
                {value}
              </span>
            ))}
          </div>
        </div>

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">Capabilities</h3>
          <div className="flex flex-wrap gap-2">
            {identity.capabilities.map((cap) => (
              <span key={cap} className="px-3 py-1 bg-blue-500/20 text-blue-500 rounded-full text-sm">
                {cap}
              </span>
            ))}
          </div>
        </div>

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">Autonomy Configuration</h3>
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">Heartbeat Interval:</span>{' '}
              {identity.autonomy_config.heartbeat_interval_minutes} minutes
            </div>
            <div>
              <span className="text-muted-foreground">Max Actions/Wake:</span>{' '}
              {identity.autonomy_config.max_actions_per_wake}
            </div>
            <div>
              <span className="text-muted-foreground">Require Approval:</span>{' '}
              {identity.autonomy_config.require_approval_t1_plus ? 'T1+' : 'None'}
            </div>
          </div>
        </div>

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">Current Goals</h3>
          <div className="space-y-2">
            {identity.goals.map((goal, idx) => (
              <div key={idx} className="flex items-center justify-between">
                <span>{goal.text}</span>
                <div className="flex items-center gap-2">
                  <span className={`text-xs px-2 py-0.5 rounded ${
                    goal.priority === 'high' ? 'bg-red-500/20 text-red-500' :
                    goal.priority === 'medium' ? 'bg-yellow-500/20 text-yellow-500' :
                    'bg-green-500/20 text-green-500'
                  }`}>
                    {goal.priority}
                  </span>
                  <span className={`text-xs px-2 py-0.5 rounded ${
                    goal.status === 'active' ? 'bg-blue-500/20 text-blue-500' :
                    'bg-gray-500/20 text-gray-500'
                  }`}>
                    {goal.status}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="border rounded-lg p-4">
          <h3 className="font-semibold mb-2">Relationships</h3>
          <div className="space-y-2">
            {identity.relationships.map((rel) => (
              <div key={rel.entity} className="flex items-center justify-between">
                <span>{rel.entity}</span>
                <div className="flex items-center gap-2">
                  <span className="text-muted-foreground text-sm">Trust:</span>
                  <span className={`text-sm font-medium ${
                    rel.trust >= 0.8 ? 'text-green-500' :
                    rel.trust >= 0.5 ? 'text-yellow-500' :
                    'text-red-500'
                  }`}>
                    {(rel.trust * 100).toFixed(0)}%
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-muted-foreground">Loading SOUL...</div>
      </div>
    );
  }

  if (editing) {
    return (
      <div className="flex flex-col h-full p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-bold">Edit SOUL</h2>
          <div className="flex gap-2">
            <button
              onClick={() => setEditing(false)}
              className="px-4 py-2 rounded-lg border hover:bg-muted"
            >
              Cancel
            </button>
            <button
              onClick={saveSoul}
              disabled={saving}
              className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {saving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </div>
        {error && (
          <div className="mb-4 p-3 bg-red-500/20 text-red-500 rounded-lg">
            {error}
          </div>
        )}
        <textarea
          value={editedContent}
          onChange={(e) => setEditedContent(e.target.value)}
          className="flex-1 p-4 font-mono text-sm bg-muted rounded-lg border resize-none"
        />
        <p className="text-xs text-muted-foreground mt-2">
          ⚠️ Changes to core identity require T3 + hardware token approval
        </p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-4">
      {renderSoulView()}
    </div>
  );
}
