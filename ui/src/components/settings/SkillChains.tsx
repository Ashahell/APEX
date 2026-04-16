'use client';

import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface ChainStep {
  id: string;
  skill_name: string;
  input_template: string;
  output_variable?: string;
  on_success: 'next' | 'end';
  on_failure: 'retry' | 'next' | 'end';
  timeout_secs: number;
  retry_on_failure: boolean;
}

interface SkillChain {
  id: string;
  name: string;
  description?: string;
  steps: ChainStep[];
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

interface ChainExecution {
  id: string;
  chain_id: string;
  chain_name: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  current_step?: number;
  started_at: string;
  completed_at?: string;
  output?: string;
  error?: string;
}

export function SkillChains() {
  const [chains, setChains] = useState<SkillChain[]>([]);
  const [executions, setExecutions] = useState<ChainExecution[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [activeTab, setActiveTab] = useState<'chains' | 'executions'>('chains');

  // Form state
  const [formName, setFormName] = useState('');
  const [formDesc, setFormDesc] = useState('');
  const [formSteps, setFormSteps] = useState<ChainStep[]>([]);

  const loadData = async () => {
    setLoading(true);
    try {
      const [chainRes, execRes] = await Promise.all([
        apiGet('/api/v1/skill-chains'),
        apiGet('/api/v1/skill-chains/executions'),
      ]);
      const chainData = await chainRes.json();
      const execData = await execRes.json();
      setChains(Array.isArray(chainData) ? chainData : []);
      setExecutions(Array.isArray(execData) ? execData.slice(0, 20) : []);
    } catch (e) {
      console.error('Failed to load skill chains:', e);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadData();
  }, []);

  const resetForm = () => {
    setFormName('');
    setFormDesc('');
    setFormSteps([]);
  };

  const handleCreate = async () => {
    try {
      await apiPost('/api/v1/skill-chains', {
        name: formName,
        description: formDesc || null,
        steps: formSteps,
      });
      resetForm();
      setShowCreate(false);
      loadData();
    } catch (e) {
      console.error('Failed to create skill chain:', e);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this skill chain?')) return;
    try {
      await fetch(`/api/v1/skill-chains/${id}`, { method: 'DELETE' });
      loadData();
    } catch (e) {
      console.error('Failed to delete skill chain:', e);
    }
  };

  const handleExecute = async (id: string) => {
    try {
      await apiPost(`/api/v1/skill-chains/${id}/execute`, {});
      loadData();
    } catch (e) {
      console.error('Failed to execute skill chain:', e);
    }
  };

  const handleCancel = async (id: string) => {
    try {
      await apiPost(`/api/v1/skill-chains/${id}/cancel`, {});
      loadData();
    } catch (e) {
      console.error('Failed to cancel skill chain:', e);
    }
  };

  const addStep = () => {
    setFormSteps([...formSteps, {
      id: crypto.randomUUID(),
      skill_name: '',
      input_template: '',
      output_variable: undefined,
      on_success: 'next',
      on_failure: 'end',
      timeout_secs: 300,
      retry_on_failure: true,
    }]);
  };

  const updateStep = (index: number, updates: Partial<ChainStep>) => {
    const updated = [...formSteps];
    updated[index] = { ...updated[index], ...updates };
    setFormSteps(updated);
  };

  const removeStep = (index: number) => {
    setFormSteps(formSteps.filter((_, i) => i !== index));
  };

  if (loading) {
    return <div className="text-muted-foreground">Loading skill chains...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold">Skill Chains</h2>
          <p className="text-muted-foreground text-sm mt-1">
            Chain multiple skills together for complex workflows
          </p>
        </div>
        <button
          onClick={() => { resetForm(); setShowCreate(true); }}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
        >
          + New Chain
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b">
        <button
          onClick={() => setActiveTab('chains')}
          className={`px-4 py-2 ${activeTab === 'chains' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground'}`}
        >
          Chains ({chains.length})
        </button>
        <button
          onClick={() => setActiveTab('executions')}
          className={`px-4 py-2 ${activeTab === 'executions' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground'}`}
        >
          Recent Executions
        </button>
      </div>

      {/* Create Form */}
      {showCreate && (
        <div className="border rounded-lg p-4 bg-card">
          <h3 className="font-semibold mb-4">Create Skill Chain</h3>
          <div className="grid gap-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Name</label>
                <input
                  type="text"
                  value={formName}
                  onChange={e => setFormName(e.target.value)}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  placeholder="Chain name"
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
              <div className="flex justify-between items-center mb-2">
                <label className="text-sm font-medium">Steps</label>
                <button
                  onClick={addStep}
                  className="text-sm text-primary hover:underline"
                >
                  + Add Step
                </button>
              </div>
              {formSteps.length === 0 ? (
                <p className="text-sm text-muted-foreground">Add steps to create your skill chain.</p>
              ) : (
                <div className="space-y-4">
                  {formSteps.map((step, idx) => (
                    <div key={step.id} className="border rounded-lg p-4 bg-muted/30">
                      <div className="flex justify-between items-center mb-3">
                        <span className="font-medium text-sm">Step {idx + 1}</span>
                        <button
                          onClick={() => removeStep(idx)}
                          className="text-sm text-red-500 hover:underline"
                        >
                          Remove
                        </button>
                      </div>
                      <div className="grid gap-3">
                        <div>
                          <label className="text-xs font-medium">Skill Name</label>
                          <input
                            type="text"
                            value={step.skill_name}
                            onChange={e => updateStep(idx, { skill_name: e.target.value })}
                            className="w-full mt-1 px-3 py-2 rounded border bg-background text-sm"
                            placeholder="e.g., code.generate, git.commit"
                          />
                        </div>
                        <div>
                          <label className="text-xs font-medium">Input Template</label>
                          <textarea
                            value={step.input_template}
                            onChange={e => updateStep(idx, { input_template: e.target.value })}
                            className="w-full mt-1 px-3 py-2 rounded border bg-background text-sm font-mono"
                            placeholder="Generate code for {{ task }}"
                            rows={2}
                          />
                        </div>
                        <div className="grid grid-cols-2 gap-3">
                          <div>
                            <label className="text-xs font-medium">On Success</label>
                            <select
                              value={step.on_success}
                              onChange={e => updateStep(idx, { on_success: e.target.value as 'next' | 'end' })}
                              className="w-full mt-1 px-3 py-2 rounded border bg-background text-sm"
                            >
                              <option value="next">Go to Next Step</option>
                              <option value="end">End Chain</option>
                            </select>
                          </div>
                          <div>
                            <label className="text-xs font-medium">On Failure</label>
                            <select
                              value={step.on_failure}
                              onChange={e => updateStep(idx, { on_failure: e.target.value as 'retry' | 'next' | 'end' })}
                              className="w-full mt-1 px-3 py-2 rounded border bg-background text-sm"
                            >
                              <option value="retry">Retry</option>
                              <option value="next">Go to Next Step</option>
                              <option value="end">End Chain</option>
                            </select>
                          </div>
                        </div>
                        <div className="grid grid-cols-2 gap-3">
                          <div>
                            <label className="text-xs font-medium">Timeout (sec)</label>
                            <input
                              type="number"
                              value={step.timeout_secs}
                              onChange={e => updateStep(idx, { timeout_secs: parseInt(e.target.value) })}
                              className="w-full mt-1 px-3 py-2 rounded border bg-background text-sm"
                              min={10}
                            />
                          </div>
                          <div className="flex items-center gap-2 pt-5">
                            <input
                              type="checkbox"
                              id={`retry-${idx}`}
                              checked={step.retry_on_failure}
                              onChange={e => updateStep(idx, { retry_on_failure: e.target.checked })}
                              className="rounded"
                            />
                            <label htmlFor={`retry-${idx}`} className="text-xs">Retry on failure</label>
                          </div>
                        </div>
                      </div>
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

      {/* Chains List */}
      {activeTab === 'chains' && (
        <div className="border rounded-lg overflow-hidden">
          <table className="w-full">
            <thead className="bg-muted">
              <tr>
                <th className="px-4 py-2 text-left text-sm font-medium">Name</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Steps</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {chains.length === 0 ? (
                <tr>
                  <td colSpan={4} className="px-4 py-8 text-center text-muted-foreground">
                    No skill chains configured. Create one to chain skills together.
                  </td>
                </tr>
              ) : chains.map(chain => (
                <tr key={chain.id} className="border-t">
                  <td className="px-4 py-3">
                    <div className="font-medium">{chain.name}</div>
                    {chain.description && (
                      <div className="text-xs text-muted-foreground">{chain.description}</div>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <span className="px-2 py-1 bg-muted rounded text-xs">
                      {chain.steps.length} step{chain.steps.length !== 1 ? 's' : ''}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 rounded text-xs ${
                      chain.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                    }`}>
                      {chain.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleExecute(chain.id)}
                        className="text-sm text-primary hover:underline"
                      >
                        Execute
                      </button>
                      <button
                        onClick={() => handleDelete(chain.id)}
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
      )}

      {/* Executions List */}
      {activeTab === 'executions' && (
        <div className="border rounded-lg overflow-hidden">
          <table className="w-full">
            <thead className="bg-muted">
              <tr>
                <th className="px-4 py-2 text-left text-sm font-medium">Chain</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Step</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Started</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {executions.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-4 py-8 text-center text-muted-foreground">
                    No executions yet. Execute a chain to see history.
                  </td>
                </tr>
              ) : executions.map(exec => (
                <tr key={exec.id} className="border-t">
                  <td className="px-4 py-3 font-medium">{exec.chain_name}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 rounded text-xs ${
                      exec.status === 'completed' ? 'bg-green-100 text-green-800' :
                      exec.status === 'failed' ? 'bg-red-100 text-red-800' :
                      exec.status === 'running' ? 'bg-blue-100 text-blue-800' :
                      exec.status === 'cancelled' ? 'bg-yellow-100 text-yellow-800' :
                      'bg-gray-100 text-gray-600'
                    }`}>
                      {exec.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm">
                    {exec.current_step !== undefined ? `Step ${exec.current_step + 1}` : '-'}
                  </td>
                  <td className="px-4 py-3 text-sm">{new Date(exec.started_at).toLocaleString()}</td>
                  <td className="px-4 py-3">
                    {exec.status === 'running' && (
                      <button
                        onClick={() => handleCancel(exec.id)}
                        className="text-sm text-red-500 hover:underline"
                      >
                        Cancel
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
