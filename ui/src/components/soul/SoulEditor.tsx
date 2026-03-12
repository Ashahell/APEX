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
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="w-14 h-14 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path>
                <circle cx="12" cy="7" r="4"></circle>
              </svg>
            </div>
            <div>
              <h2 className="text-2xl font-bold">{identity.name}</h2>
              <p className="text-sm text-[var(--color-text-muted)]">Wake Count: {identity.wake_count}</p>
            </div>
          </div>
          <button
            onClick={() => setEditing(true)}
            className="px-4 py-2 rounded-lg bg-[#4248f1] text-white hover:bg-[#353bc5] transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
            </svg>
            Edit SOUL
          </button>
        </div>

        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            Purpose
          </h3>
          <p className="text-[var(--color-text-muted)]">{identity.purpose}</p>
        </div>

        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"></path>
            </svg>
            Values
          </h3>
          <div className="flex flex-wrap gap-2">
            {identity.values.map((value) => (
              <span key={value} className="px-3 py-1.5 bg-[var(--color-muted)] rounded-full text-sm">
                {value}
              </span>
            ))}
          </div>
        </div>

        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-3 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
            </svg>
            Capabilities
          </h3>
          <div className="flex flex-wrap gap-2">
            {identity.capabilities.map((cap) => (
              <span key={cap} className="px-3 py-1.5 bg-[#4248f1]/10 text-[#4248f1] rounded-full text-sm border border-[#4248f1]/20">
                {cap}
              </span>
            ))}
          </div>
        </div>

        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <polyline points="12 6 12 12 16 14"></polyline>
            </svg>
            Autonomy Configuration
          </h3>
          <div className="grid grid-cols-3 gap-4">
            <div className="p-4 bg-[var(--color-muted)]/30 rounded-lg text-center">
              <div className="text-2xl font-bold text-[#4248f1]">{identity.autonomy_config.heartbeat_interval_minutes}</div>
              <div className="text-xs text-[var(--color-text-muted)]">Minutes Interval</div>
            </div>
            <div className="p-4 bg-[var(--color-muted)]/30 rounded-lg text-center">
              <div className="text-2xl font-bold text-[#4248f1]">{identity.autonomy_config.max_actions_per_wake}</div>
              <div className="text-xs text-[var(--color-text-muted)]">Max Actions/Wake</div>
            </div>
            <div className="p-4 bg-[var(--color-muted)]/30 rounded-lg text-center">
              <div className="text-2xl font-bold">{identity.autonomy_config.require_approval_t1_plus ? 'T1+' : 'None'}</div>
              <div className="text-xs text-[var(--color-text-muted)]">Approval Required</div>
            </div>
          </div>
        </div>

        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
              <polyline points="2 17 12 22 22 17"></polyline>
              <polyline points="2 12 12 17 22 12"></polyline>
            </svg>
            Current Goals
          </h3>
          <div className="space-y-3">
            {identity.goals.map((goal, idx) => (
              <div key={idx} className="flex items-center justify-between p-3 bg-[var(--color-muted)]/30 rounded-lg">
                <span className="text-[var(--color-text)]">{goal.text}</span>
                <div className="flex items-center gap-2">
                  <span className={`text-xs px-2.5 py-1 rounded-full font-medium ${
                    goal.priority === 'high' ? 'bg-red-500/10 text-red-500 border border-red-500/20' :
                    goal.priority === 'medium' ? 'bg-amber-500/10 text-amber-500 border border-amber-500/20' :
                    'bg-green-500/10 text-green-500 border border-green-500/20'
                  }`}>
                    {goal.priority}
                  </span>
                  <span className={`text-xs px-2.5 py-1 rounded-full font-medium ${
                    goal.status === 'active' ? 'bg-[#4248f1]/10 text-[#4248f1] border border-[#4248f1]/20' :
                    'bg-[var(--color-muted)] text-[var(--color-text-muted)]'
                  }`}>
                    {goal.status}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <h3 className="font-semibold mb-4 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path>
              <circle cx="9" cy="7" r="4"></circle>
              <path d="M23 21v-2a4 4 0 0 0-3-3.87"></path>
              <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
            </svg>
            Relationships
          </h3>
          <div className="space-y-3">
            {identity.relationships.map((rel) => (
              <div key={rel.entity} className="flex items-center justify-between p-3 bg-[var(--color-muted)]/30 rounded-lg">
                <span className="font-medium">{rel.entity}</span>
                <div className="flex items-center gap-2">
                  <span className="text-[var(--color-text-muted)] text-sm">Trust:</span>
                  <span className={`text-sm font-bold ${
                    rel.trust >= 0.8 ? 'text-green-500' :
                    rel.trust >= 0.5 ? 'text-amber-500' :
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
            Loading...
        </div>
      </div>
    );
  }

  if (editing) {
    return (
      <div className="flex flex-col h-full p-6">
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
              </svg>
            </div>
            <h2 className="text-xl font-bold">Edit SOUL</h2>
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setEditing(false)}
              className="px-4 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] hover:bg-[var(--color-muted)] transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={saveSoul}
              disabled={saving}
              className="px-4 py-2 rounded-lg bg-[#4248f1] text-white hover:bg-[#353bc5] transition-colors disabled:opacity-50 flex items-center gap-2"
            >
              {saving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </div>
        {error && (
          <div className="mb-4 p-3 bg-red-500/10 text-red-500 rounded-lg border border-red-500/20 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="15" y1="9" x2="9" y2="15"></line>
              <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
            {error}
          </div>
        )}
        <textarea
          value={editedContent}
          onChange={(e) => setEditedContent(e.target.value)}
          className="flex-1 p-4 font-mono text-sm bg-[var(--color-muted)]/30 rounded-lg border border-[var(--color-border)] resize-none text-[var(--color-text)]"
        />
        <p className="text-xs text-[var(--color-text-muted)] mt-3 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
            <line x1="12" y1="9" x2="12" y2="13"></line>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          Changes to core identity require T3 + hardware token approval
        </p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto">
        {renderSoulView()}
      </div>
    </div>
  );
}
