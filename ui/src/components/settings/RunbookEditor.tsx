import { useState, useEffect, useCallback } from 'react';
import { apiGet, apiPost, apiPut, apiDelete } from '../../lib/api';

// ============ Types ============

export interface RunbookStep {
  id: string;
  name: string;
  action: 'Log' | 'Notify' | 'ExecuteCommand' | 'Webhook' | 'Delay' | 'CheckCondition' | 'PauseTask' | 'CancelTask' | 'Escalate';
  command?: string;
  url?: string;
  delay_secs?: number;
  condition?: string;
  on_failure?: 'abort' | 'continue' | 'retry';
  retry_count?: number;
}

export interface Runbook {
  id: string;
  name: string;
  description?: string;
  trigger_alert_type?: string;
  steps: RunbookStep[];
  enabled: boolean;
  created_at: string;
  updated_at: string;
}

export interface RunbookExecution {
  id: string;
  runbook_id: string;
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';
  started_at: string;
  completed_at?: string;
  alert_id?: string;
  step_results?: StepResult[];
  error?: string;
}

export interface StepResult {
  step_id: string;
  step_name: string;
  status: 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Skipped';
  started_at?: string;
  completed_at?: string;
  output?: string;
  error?: string;
}

export interface RunbookTemplate {
  name: string;
  description: string;
  steps: RunbookStep[];
  trigger_alert_type?: string;
}

export interface CreateRunbookRequest {
  name: string;
  description?: string;
  trigger_alert_type?: string;
  steps: RunbookStep[];
}

export interface UpdateRunbookRequest {
  name?: string;
  description?: string;
  trigger_alert_type?: string;
  steps?: RunbookStep[];
  enabled?: boolean;
}

export interface ExecuteRunbookRequest {
  alert_id?: string;
}

// ============ Templates ============

const TEMPLATES: RunbookTemplate[] = [
  {
    name: 'Restart Failed Task',
    description: 'Restart a task that failed due to transient errors',
    trigger_alert_type: 'NoProgress',
    steps: [
      { id: '1', name: 'Log restart attempt', action: 'Log', command: 'Restarting failed task' },
      { id: '2', name: 'Send notification', action: 'Notify' },
      { id: '3', name: 'Wait before restart', action: 'Delay', delay_secs: 5 },
      { id: '4', name: 'Pause task', action: 'PauseTask' },
    ],
  },
  {
    name: 'Clear Resource',
    description: 'Clear a stuck resource and retry',
    trigger_alert_type: 'ResourceExhaustion',
    steps: [
      { id: '1', name: 'Log resource clearing', action: 'Log', command: 'Clearing stuck resource' },
      { id: '2', name: 'Execute cleanup', action: 'ExecuteCommand', command: 'echo "Cleanup command"' },
      { id: '3', name: 'Wait', action: 'Delay', delay_secs: 3 },
      { id: '4', name: 'Notify completion', action: 'Notify' },
    ],
  },
  {
    name: 'Escalate Notification',
    description: 'Send escalation notification when alert is unacknowledged',
    trigger_alert_type: 'TimeoutWarning',
    steps: [
      { id: '1', name: 'Log escalation', action: 'Log', command: 'Alert escalation triggered' },
      { id: '2', name: 'Send email', action: 'Notify' },
      { id: '3', name: 'Wait for response', action: 'Delay', delay_secs: 300 },
      { id: '4', name: 'Escalate further', action: 'Escalate' },
    ],
  },
];

// ============ Component ============

export function RunbookEditor() {
  const [runbooks, setRunbooks] = useState<Runbook[]>([]);
  const [executions, setExecutions] = useState<Map<string, RunbookExecution[]>>(new Map());
  const [loading, setLoading] = useState(true);
  const [activeView, setActiveView] = useState<'list' | 'create' | 'edit'>('list');
  const [selectedRunbook, setSelectedRunbook] = useState<Runbook | null>(null);
  const [templates] = useState<RunbookTemplate[]>(TEMPLATES);

  // Form state
  const [formName, setFormName] = useState('');
  const [formDescription, setFormDescription] = useState('');
  const [formTriggerType, setFormTriggerType] = useState('');
  const [formSteps, setFormSteps] = useState<RunbookStep[]>([]);
  const [formEnabled, setFormEnabled] = useState(true);

  const loadRunbooks = useCallback(async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/runbooks');
      if (res.ok) {
        const data = await res.json();
        setRunbooks(Array.isArray(data) ? data : []);
      }
    } catch (err) {
      console.error('Failed to load runbooks:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  const loadExecutions = useCallback(async (runbookId: string) => {
    try {
      const res = await apiGet(`/api/v1/runbooks/${runbookId}/executions`);
      if (res.ok) {
        const data = await res.json();
        setExecutions(prev => new Map(prev).set(runbookId, data));
      }
    } catch (err) {
      console.error('Failed to load executions:', err);
    }
  }, []);

  useEffect(() => {
    loadRunbooks();
  }, [loadRunbooks]);

  useEffect(() => {
    runbooks.forEach(rb => loadExecutions(rb.id));
  }, [runbooks, loadExecutions]);

  const resetForm = () => {
    setFormName('');
    setFormDescription('');
    setFormTriggerType('');
    setFormSteps([]);
    setFormEnabled(true);
    setSelectedRunbook(null);
  };

  const handleCreateFromTemplate = (template: RunbookTemplate) => {
    setFormName(template.name);
    setFormDescription(template.description);
    setFormTriggerType(template.trigger_alert_type || '');
    setFormSteps(template.steps.map((s, i) => ({ ...s, id: String(i + 1) })));
    setActiveView('create');
  };

  const handleCreate = async () => {
    if (!formName || formSteps.length === 0) return;

    try {
      const res = await apiPost('/api/v1/runbooks', {
        name: formName,
        description: formDescription || undefined,
        trigger_alert_type: formTriggerType || undefined,
        steps: formSteps,
      });

      if (res.ok) {
        resetForm();
        setActiveView('list');
        loadRunbooks();
      }
    } catch (err) {
      console.error('Failed to create runbook:', err);
    }
  };

  const handleUpdate = async () => {
    if (!selectedRunbook || !formName || formSteps.length === 0) return;

    try {
      const res = await apiPut(`/api/v1/runbooks/${selectedRunbook.id}`, {
        name: formName,
        description: formDescription || undefined,
        trigger_alert_type: formTriggerType || undefined,
        steps: formSteps,
        enabled: formEnabled,
      });

      if (res.ok) {
        resetForm();
        setActiveView('list');
        loadRunbooks();
      }
    } catch (err) {
      console.error('Failed to update runbook:', err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await apiDelete(`/api/v1/runbooks/${id}`);
      loadRunbooks();
    } catch (err) {
      console.error('Failed to delete runbook:', err);
    }
  };

  const handleExecute = async (runbook: Runbook) => {
    try {
      const res = await apiPost(`/api/v1/runbooks/${runbook.id}/execute`, {
        alert_id: undefined,
      });
      if (res.ok) {
        loadExecutions(runbook.id);
      }
    } catch (err) {
      console.error('Failed to execute runbook:', err);
    }
  };

  const handleToggle = async (runbook: Runbook, enabled: boolean) => {
    try {
      await apiPut(`/api/v1/runbooks/${runbook.id}`, { enabled });
      loadRunbooks();
    } catch (err) {
      console.error('Failed to toggle runbook:', err);
    }
  };

  const handleEdit = (runbook: Runbook) => {
    setSelectedRunbook(runbook);
    setFormName(runbook.name);
    setFormDescription(runbook.description || '');
    setFormTriggerType(runbook.trigger_alert_type || '');
    setFormSteps([...runbook.steps]);
    setFormEnabled(runbook.enabled);
    setActiveView('edit');
  };

  const addStep = () => {
    const newStep: RunbookStep = {
      id: String(formSteps.length + 1),
      name: `Step ${formSteps.length + 1}`,
      action: 'Log',
      on_failure: 'abort',
    };
    setFormSteps([...formSteps, newStep]);
  };

  const updateStep = (index: number, updates: Partial<RunbookStep>) => {
    const updated = [...formSteps];
    updated[index] = { ...updated[index], ...updates };
    setFormSteps(updated);
  };

  const removeStep = (index: number) => {
    setFormSteps(formSteps.filter((_, i) => i !== index));
  };

  const getActionColor = (action: RunbookStep['action']) => {
    switch (action) {
      case 'Log': return 'bg-blue-500/20 text-blue-400';
      case 'Notify': return 'bg-purple-500/20 text-purple-400';
      case 'ExecuteCommand': return 'bg-orange-500/20 text-orange-400';
      case 'Webhook': return 'bg-green-500/20 text-green-400';
      case 'Delay': return 'bg-gray-500/20 text-gray-400';
      case 'CheckCondition': return 'bg-yellow-500/20 text-yellow-400';
      case 'PauseTask': return 'bg-cyan-500/20 text-cyan-400';
      case 'CancelTask': return 'bg-red-500/20 text-red-400';
      case 'Escalate': return 'bg-pink-500/20 text-pink-400';
    }
  };

  const getStatusColor = (status: RunbookExecution['status']) => {
    switch (status) {
      case 'Pending': return 'bg-gray-500/20 text-gray-400';
      case 'Running': return 'bg-blue-500/20 text-blue-400';
      case 'Completed': return 'bg-green-500/20 text-green-400';
      case 'Failed': return 'bg-red-500/20 text-red-400';
      case 'Cancelled': return 'bg-yellow-500/20 text-yellow-400';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading runbooks...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-semibold">Runbooks</h2>
        <p className="text-[var(--color-text-muted)]">
          Automated remediation workflows triggered by alerts
        </p>
      </div>

      {/* View Tabs */}
      <div className="flex flex-wrap gap-4 border-b">
        <button
          onClick={() => { resetForm(); setActiveView('list'); }}
          className={`px-4 py-2 transition-colors ${
            activeView === 'list'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          Runbooks ({runbooks.length})
        </button>
        <button
          onClick={() => { resetForm(); setActiveView('create'); }}
          className={`px-4 py-2 transition-colors ${
            activeView === 'create'
              ? 'border-b-2 border-[#00d4ff] text-[#00d4ff]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          + New Runbook
        </button>
      </div>

      {/* List View */}
      {activeView === 'list' && (
        <div className="space-y-4">
          {/* Templates Section */}
          <div className="border rounded-lg p-4 bg-[var(--color-panel)]">
            <h3 className="font-semibold mb-3">Templates</h3>
            <div className="grid grid-cols-3 gap-3">
              {templates.map((template, idx) => (
                <button
                  key={idx}
                  onClick={() => handleCreateFromTemplate(template)}
                  className="p-3 border rounded-lg text-left hover:border-[#00d4ff] transition-colors"
                >
                  <div className="font-medium text-sm">{template.name}</div>
                  <div className="text-xs text-[var(--color-text-muted)]">{template.description}</div>
                  <div className="text-xs text-[#00d4ff] mt-1">{template.steps.length} steps</div>
                </button>
              ))}
            </div>
          </div>

          {/* Runbooks List */}
          {runbooks.length === 0 ? (
            <div className="border rounded-lg p-8 text-center text-[var(--color-text-muted)]">
              No runbooks configured. Create one from a template or from scratch.
            </div>
          ) : (
            <div className="space-y-3">
              {runbooks.map((runbook) => (
                <div key={runbook.id} className="border rounded-lg p-4 bg-[var(--color-panel)]">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`px-2 py-0.5 rounded text-xs ${
                          runbook.enabled ? 'bg-green-500/20 text-green-400' : 'bg-gray-500/20 text-gray-400'
                        }`}>
                          {runbook.enabled ? 'Active' : 'Disabled'}
                        </span>
                        {runbook.trigger_alert_type && (
                          <span className="px-2 py-0.5 rounded text-xs bg-[#00d4ff]/20 text-[#00d4ff]">
                            Trigger: {runbook.trigger_alert_type}
                          </span>
                        )}
                        <h3 className="font-medium">{runbook.name}</h3>
                      </div>
                      {runbook.description && (
                        <p className="text-sm text-[var(--color-text-muted)] mb-2">{runbook.description}</p>
                      )}
                      <div className="flex flex-wrap gap-1">
                        {runbook.steps.map((step) => (
                          <span key={step.id} className={`px-2 py-0.5 rounded text-xs ${getActionColor(step.action)}`}>
                            {step.action}
                          </span>
                        ))}
                      </div>
                      <div className="text-xs text-[var(--color-text-muted)] mt-2">
                        {runbook.steps.length} steps • Updated {new Date(runbook.updated_at).toLocaleDateString()}
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleExecute(runbook)}
                        disabled={!runbook.enabled}
                        className="px-3 py-1.5 bg-[#00d4ff]/20 text-[#00d4ff] rounded hover:bg-[#00d4ff]/30 disabled:opacity-50 transition-colors text-sm"
                        title="Execute runbook"
                      >
                        ▶ Run
                      </button>
                      <button
                        onClick={() => handleEdit(runbook)}
                        className="px-3 py-1.5 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors text-sm"
                        title="Edit runbook"
                      >
                        Edit
                      </button>
                      <label className="flex items-center cursor-pointer">
                        <input
                          type="checkbox"
                          checked={runbook.enabled}
                          onChange={(e) => handleToggle(runbook, e.target.checked)}
                          className="sr-only peer"
                        />
                        <div className="w-9 h-5 bg-[var(--color-muted)] rounded-full peer peer-checked:bg-[#00d4ff] after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full relative"></div>
                      </label>
                      <button
                        onClick={() => handleDelete(runbook.id)}
                        className="p-1.5 text-red-400 hover:bg-red-500/20 rounded transition-colors"
                        title="Delete runbook"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M3 6h18"></path>
                          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
                          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
                        </svg>
                      </button>
                    </div>
                  </div>

                  {/* Recent Executions */}
                  {(() => {
                    const execs = executions.get(runbook.id);
                    if (!execs || execs.length === 0) return null;
                    return (
                      <div className="mt-3 pt-3 border-t">
                        <h4 className="text-xs font-medium text-[var(--color-text-muted)] mb-2">Recent Executions</h4>
                        <div className="flex gap-2 overflow-x-auto">
                          {execs.slice(0, 5).map((exec) => (
                            <div key={exec.id} className={`px-2 py-1 rounded text-xs whitespace-nowrap ${getStatusColor(exec.status)}`}>
                              {exec.status} • {new Date(exec.started_at).toLocaleTimeString()}
                            </div>
                          ))}
                        </div>
                      </div>
                    );
                  })()}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Create/Edit View */}
      {(activeView === 'create' || activeView === 'edit') && (
        <div className="border rounded-lg p-6 bg-[var(--color-panel)] space-y-6">
          <h3 className="font-semibold">{activeView === 'create' ? 'Create Runbook' : 'Edit Runbook'}</h3>

          {/* Basic Info */}
          <div className="grid gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Name *</label>
              <input
                type="text"
                value={formName}
                onChange={(e) => setFormName(e.target.value)}
                placeholder="Restart Failed Task"
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Description</label>
              <input
                type="text"
                value={formDescription}
                onChange={(e) => setFormDescription(e.target.value)}
                placeholder="What this runbook does..."
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Trigger Alert Type</label>
              <select
                value={formTriggerType}
                onChange={(e) => setFormTriggerType(e.target.value)}
                className="w-full px-3 py-2 rounded border bg-[var(--color-bg)] border-[var(--color-border)]"
              >
                <option value="">None (manual only)</option>
                <option value="InfiniteLoop">Infinite Loop</option>
                <option value="NoProgress">No Progress</option>
                <option value="ResourceExhaustion">Resource Exhaustion</option>
                <option value="TimeoutWarning">Timeout Warning</option>
                <option value="PatternDetected">Pattern Detected</option>
                <option value="ErrorSpike">Error Spike</option>
              </select>
              <p className="text-xs text-[var(--color-text-muted)] mt-1">
                Automatically trigger this runbook when matching alerts are created
              </p>
            </div>
          </div>

          {/* Steps */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <label className="block text-sm font-medium">Steps *</label>
              <button
                onClick={addStep}
                className="px-3 py-1 text-sm text-[#00d4ff] hover:bg-[#00d4ff]/10 rounded transition-colors"
              >
                + Add Step
              </button>
            </div>

            {formSteps.length === 0 ? (
              <div className="border rounded-lg p-6 text-center text-[var(--color-text-muted)]">
                No steps yet. Add steps to define the runbook workflow.
              </div>
            ) : (
              <div className="space-y-3">
                {formSteps.map((step, index) => (
                  <div key={step.id} className="border rounded-lg p-4 bg-[var(--color-muted)]/20">
                    <div className="flex items-start gap-3">
                      <div className="flex-shrink-0 w-8 h-8 rounded-full bg-[#00d4ff]/20 text-[#00d4ff] flex items-center justify-center font-medium">
                        {index + 1}
                      </div>
                      <div className="flex-1 space-y-3">
                        <div className="grid grid-cols-2 gap-3">
                          <div>
                            <label className="block text-xs text-[var(--color-text-muted)] mb-1">Name</label>
                            <input
                              type="text"
                              value={step.name}
                              onChange={(e) => updateStep(index, { name: e.target.value })}
                              className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                            />
                          </div>
                          <div>
                            <label className="block text-xs text-[var(--color-text-muted)] mb-1">Action</label>
                            <select
                              value={step.action}
                              onChange={(e) => updateStep(index, { action: e.target.value as RunbookStep['action'] })}
                              className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                            >
                              <option value="Log">Log</option>
                              <option value="Notify">Notify</option>
                              <option value="ExecuteCommand">Execute Command</option>
                              <option value="Webhook">Webhook</option>
                              <option value="Delay">Delay</option>
                              <option value="CheckCondition">Check Condition</option>
                              <option value="PauseTask">Pause Task</option>
                              <option value="CancelTask">Cancel Task</option>
                              <option value="Escalate">Escalate</option>
                            </select>
                          </div>
                        </div>

                        {/* Action-specific fields */}
                        {step.action === 'ExecuteCommand' && (
                          <div>
                            <label className="block text-xs text-[var(--color-text-muted)] mb-1">Command</label>
                            <input
                              type="text"
                              value={step.command || ''}
                              onChange={(e) => updateStep(index, { command: e.target.value })}
                              placeholder="echo 'Hello'"
                              className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm font-mono"
                            />
                          </div>
                        )}

                        {step.action === 'Webhook' && (
                          <div>
                            <label className="block text-xs text-[var(--color-text-muted)] mb-1">URL</label>
                            <input
                              type="text"
                              value={step.url || ''}
                              onChange={(e) => updateStep(index, { url: e.target.value })}
                              placeholder="https://example.com/webhook"
                              className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                            />
                          </div>
                        )}

                        {step.action === 'Delay' && (
                          <div>
                            <label className="block text-xs text-[var(--color-text-muted)] mb-1">Delay (seconds)</label>
                            <input
                              type="number"
                              value={step.delay_secs || 0}
                              onChange={(e) => updateStep(index, { delay_secs: parseInt(e.target.value) || 0 })}
                              min={0}
                              className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                            />
                          </div>
                        )}

                        {step.action === 'CheckCondition' && (
                          <div>
                            <label className="block text-xs text-[var(--color-text-muted)] mb-1">Condition</label>
                            <input
                              type="text"
                              value={step.condition || ''}
                              onChange={(e) => updateStep(index, { condition: e.target.value })}
                              placeholder="task.status == 'failed'"
                              className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm font-mono"
                            />
                          </div>
                        )}

                        <div className="grid grid-cols-2 gap-3">
                          <div>
                            <label className="block text-xs text-[var(--color-text-muted)] mb-1">On Failure</label>
                            <select
                              value={step.on_failure || 'abort'}
                              onChange={(e) => updateStep(index, { on_failure: e.target.value as RunbookStep['on_failure'] })}
                              className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                            >
                              <option value="abort">Abort runbook</option>
                              <option value="continue">Continue to next step</option>
                              <option value="retry">Retry this step</option>
                            </select>
                          </div>
                          {step.on_failure === 'retry' && (
                            <div>
                              <label className="block text-xs text-[var(--color-text-muted)] mb-1">Retry Count</label>
                              <input
                                type="number"
                                value={step.retry_count || 1}
                                onChange={(e) => updateStep(index, { retry_count: parseInt(e.target.value) || 1 })}
                                min={1}
                                max={10}
                                className="w-full px-2 py-1.5 rounded border bg-[var(--color-bg)] border-[var(--color-border)] text-sm"
                              />
                            </div>
                          )}
                        </div>
                      </div>
                      <button
                        onClick={() => removeStep(index)}
                        className="p-1.5 text-red-400 hover:bg-red-500/20 rounded transition-colors"
                        title="Remove step"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                          <path d="M18 6L6 18M6 6l12 12"></path>
                        </svg>
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Edit mode only */}
          {activeView === 'edit' && (
            <div>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={formEnabled}
                  onChange={(e) => setFormEnabled(e.target.checked)}
                  className="rounded"
                />
                <span className="text-sm">Enabled</span>
              </label>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-3 pt-4">
            <button
              onClick={activeView === 'create' ? handleCreate : handleUpdate}
              disabled={!formName || formSteps.length === 0}
              className="px-4 py-2 bg-[#00d4ff] text-[#0f0f1a] rounded font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
            >
              {activeView === 'create' ? 'Create Runbook' : 'Update Runbook'}
            </button>
            <button
              onClick={() => { resetForm(); setActiveView('list'); }}
              className="px-4 py-2 bg-[var(--color-muted)] rounded hover:bg-[var(--color-muted)]/80 transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
