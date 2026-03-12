import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface ImmutableValue {
  name: string;
  description: string;
  priority: number;
}

interface EmergencyProtocol {
  name: string;
  trigger_condition: string;
  actions: string[];
  notify_human: boolean;
}

interface GovernancePolicy {
  constitution_hash: string;
  immutable_values: ImmutableValue[];
  emergency_protocols: EmergencyProtocol[];
}

export function GovernanceControls() {
  const [policy, setPolicy] = useState<GovernancePolicy | null>(null);
  const [oracleMode, setOracleMode] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    loadPolicy();
  }, []);

  const loadPolicy = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/governance/policy');
      if (res.ok) {
        const data = await res.json();
        setPolicy(data.policy);
        setOracleMode(data.oracle_mode);
      }
    } catch (err) {
      console.error('Failed to load policy:', err);
    } finally {
      setLoading(false);
    }
  };

  const toggleOracleMode = async () => {
    setError(null);
    setSuccess(null);
    try {
      const res = await apiPost('/api/v1/governance/oracle', { enable: !oracleMode });
      if (res.ok) {
        const data = await res.json();
        setOracleMode(data.oracle_mode);
        setSuccess(`Oracle mode ${data.oracle_mode ? 'enabled' : 'disabled'}`);
        setTimeout(() => setSuccess(null), 3000);
      } else {
        setError('Failed to toggle oracle mode');
      }
    } catch (err) {
      setError('Failed to toggle oracle mode');
    }
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
          Loading governance policy...
        </div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-3xl mx-auto space-y-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
              </svg>
            </div>
            <div>
              <h2 className="text-2xl font-bold">Governance</h2>
              <p className="text-sm text-[var(--color-text-muted)]">
                Agent constitution and safety protocols
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <span className={`px-3 py-1.5 rounded-full text-sm font-medium ${
              oracleMode ? 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30' : 'bg-green-500/20 text-green-400 border border-green-500/30'
            }`}>
              {oracleMode ? 'Oracle Mode' : 'Normal Mode'}
            </span>
          </div>
        </div>

        {/* Notifications */}
        {error && (
          <div className="p-3 bg-red-500/10 text-red-500 rounded-lg border border-red-500/20 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="15" y1="9" x2="9" y2="15"></line>
              <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
            {error}
          </div>
        )}

        {success && (
          <div className="p-3 bg-green-500/10 text-green-500 rounded-lg border border-green-500/20 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
              <polyline points="22 4 12 14.01 9 11.01"></polyline>
            </svg>
            {success}
          </div>
        )}

        {/* Oracle Mode Card */}
        <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="16" x2="12" y2="12"></line>
                <line x1="12" y1="8" x2="12.01" y2="8"></line>
              </svg>
              Oracle Mode
            </h3>
            <button
              onClick={toggleOracleMode}
              className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                oracleMode
                  ? 'bg-[var(--color-muted)] hover:bg-[var(--color-muted)]/80 text-[var(--color-text)] border border-[var(--color-border)]'
                  : 'bg-yellow-600 hover:bg-yellow-700 text-white'
              }`}
            >
              {oracleMode ? 'Exit Oracle Mode' : 'Enter Oracle Mode'}
            </button>
          </div>
          <p className="text-[var(--color-text-muted)] text-sm">
            In Oracle Mode, the agent can only perform read-only operations. 
            All write actions are blocked. This is useful for safe observation mode.
          </p>
        </div>

        {policy && (
          <>
            {/* Immutable Values Card */}
            <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
              <h3 className="font-semibold mb-2 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z"></path>
                  <line x1="4" y1="22" x2="4" y2="15"></line>
                </svg>
                Immutable Values
              </h3>
              <p className="text-[var(--color-text-muted)] text-sm mb-4">
                These values cannot be modified without proper approval (T3 + hardware token).
              </p>
              <div className="space-y-3">
                {policy.immutable_values.map((value, idx) => (
                  <div key={idx} className="flex items-start gap-3 p-3 bg-[var(--color-muted)]/30 rounded-lg">
                    <div className="flex-shrink-0 w-8 h-8 bg-red-500/10 rounded-full flex items-center justify-center text-red-500 font-bold border border-red-500/20">
                      {value.priority}
                    </div>
                    <div>
                      <div className="font-medium capitalize">{value.name.replace(/_/g, ' ')}</div>
                      <div className="text-[var(--color-text-muted)] text-sm">{value.description}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Emergency Protocols Card */}
            <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
              <h3 className="font-semibold mb-4 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <polygon points="7.86 2 16.14 2 22 7.86 22 16.14 16.14 22 7.86 22 2 16.14 2 7.86 7.86 2"></polygon>
                  <line x1="12" y1="8" x2="12" y2="12"></line>
                  <line x1="12" y1="16" x2="12.01" y2="16"></line>
                </svg>
                Emergency Protocols
              </h3>
              <div className="space-y-3">
                {policy.emergency_protocols.map((protocol, idx) => (
                  <div key={idx} className="p-4 bg-[var(--color-muted)]/30 rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <div className="font-medium capitalize">{protocol.name.replace(/_/g, ' ')}</div>
                      {protocol.notify_human && (
                        <span className="px-2 py-1 bg-orange-500/10 text-orange-400 text-xs rounded-full border border-orange-500/20">
                          Notifies Human
                        </span>
                      )}
                    </div>
                    <div className="text-[var(--color-text-muted)] text-sm mb-2">
                      <span className="text-red-400 font-medium">Trigger:</span> {protocol.trigger_condition}
                    </div>
                    <div className="text-[var(--color-text-muted)] text-sm">
                      <span className="text-[#4248f1] font-medium">Actions:</span> {protocol.actions.join(' → ')}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Constitution Card */}
            <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
              <h3 className="font-semibold mb-4 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                  <polyline points="14 2 14 8 20 8"></polyline>
                  <line x1="16" y1="13" x2="8" y2="13"></line>
                  <line x1="16" y1="17" x2="8" y2="17"></line>
                  <polyline points="10 9 9 9 8 9"></polyline>
                </svg>
                Constitution
              </h3>
              <div className="text-[var(--color-text-muted)] text-sm">
                <div className="mb-3">
                  <span className="text-[var(--color-text)] font-medium">Hash:</span>{' '}
                  <code className="bg-[var(--color-muted)] px-2 py-1 rounded text-xs font-mono">
                    {policy.constitution_hash || 'Not set'}
                  </code>
                </div>
                <p className="text-xs">
                  The constitution defines the core principles that govern agent behavior.
                  Modifications require T3 approval + hardware token + 24 hour delay.
                </p>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
