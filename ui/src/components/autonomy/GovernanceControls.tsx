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
      <div className="flex items-center justify-center h-64">
        <div className="text-gray-400">Loading governance policy...</div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Governance</h2>
        <div className="flex items-center gap-2">
          <span className={`px-3 py-1 rounded-full text-sm ${oracleMode ? 'bg-yellow-500/20 text-yellow-400' : 'bg-green-500/20 text-green-400'}`}>
            {oracleMode ? 'Oracle Mode' : 'Normal Mode'}
          </span>
        </div>
      </div>

      {error && (
        <div className="p-4 bg-red-500/20 border border-red-500 rounded-lg text-red-400">
          {error}
        </div>
      )}

      {success && (
        <div className="p-4 bg-green-500/20 border border-green-500 rounded-lg text-green-400">
          {success}
        </div>
      )}

      <div className="grid gap-6">
        <div className="bg-gray-800 rounded-lg p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold">Oracle Mode</h3>
            <button
              onClick={toggleOracleMode}
              className={`px-4 py-2 rounded-lg font-medium transition-colors ${
                oracleMode
                  ? 'bg-gray-600 hover:bg-gray-700 text-white'
                  : 'bg-yellow-600 hover:bg-yellow-700 text-white'
              }`}
            >
              {oracleMode ? 'Exit Oracle Mode' : 'Enter Oracle Mode'}
            </button>
          </div>
          <p className="text-gray-400 text-sm">
            In Oracle Mode, the agent can only perform read-only operations. 
            All write actions are blocked. This is useful for safe observation mode.
          </p>
        </div>

        {policy && (
          <>
            <div className="bg-gray-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold mb-4">Immutable Values</h3>
              <p className="text-gray-400 text-sm mb-4">
                These values cannot be modified without proper approval (T3 + hardware token).
              </p>
              <div className="space-y-3">
                {policy.immutable_values.map((value, idx) => (
                  <div key={idx} className="flex items-start gap-3 p-3 bg-gray-700/50 rounded-lg">
                    <div className="flex-shrink-0 w-8 h-8 bg-red-500/20 rounded-full flex items-center justify-center text-red-400 font-bold">
                      {value.priority}
                    </div>
                    <div>
                      <div className="font-medium">{value.name.replace(/_/g, ' ')}</div>
                      <div className="text-gray-400 text-sm">{value.description}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-gray-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold mb-4">Emergency Protocols</h3>
              <div className="space-y-3">
                {policy.emergency_protocols.map((protocol, idx) => (
                  <div key={idx} className="p-4 bg-gray-700/50 rounded-lg">
                    <div className="flex items-center justify-between mb-2">
                      <div className="font-medium">{protocol.name.replace(/_/g, ' ')}</div>
                      {protocol.notify_human && (
                        <span className="px-2 py-1 bg-orange-500/20 text-orange-400 text-xs rounded">
                          Notifies Human
                        </span>
                      )}
                    </div>
                    <div className="text-gray-400 text-sm mb-2">
                      <span className="text-red-400">Trigger:</span> {protocol.trigger_condition}
                    </div>
                    <div className="text-gray-400 text-sm">
                      <span className="text-blue-400">Actions:</span> {protocol.actions.join(' → ')}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-gray-800 rounded-lg p-6">
              <h3 className="text-lg font-semibold mb-4">Constitution</h3>
              <div className="text-gray-400 text-sm">
                <div className="mb-2">
                  <span className="text-gray-300">Hash:</span>{' '}
                  <code className="bg-gray-900 px-2 py-1 rounded text-xs">
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
