'use client';

import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface ScheduleConfig {
  interval_secs: number;
  cron_expr?: string;
  run_at?: string;
}

interface ScheduledTemplate {
  id: string;
  name: string;
  description?: string;
  task_content: string;
  schedule_type: 'interval' | 'cron' | 'onetime';
  schedule_config: ScheduleConfig;
  enabled: boolean;
  max_runs?: number;
  run_count: number;
  last_run_at?: string;
  next_run_at?: string;
  created_at: string;
  updated_at: string;
}

interface ScheduledExecution {
  id: string;
  template_id: string;
  template_name: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  started_at: string;
  completed_at?: string;
  output?: string;
  error?: string;
}

export function ScheduledTaskTemplates() {
  const [templates, setTemplates] = useState<ScheduledTemplate[]>([]);
  const [executions, setExecutions] = useState<ScheduledExecution[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [activeTab, setActiveTab] = useState<'templates' | 'executions'>('templates');

  // Form state
  const [formName, setFormName] = useState('');
  const [formDesc, setFormDesc] = useState('');
  const [formContent, setFormContent] = useState('');
  const [formType, setFormType] = useState<'interval' | 'cron' | 'onetime'>('interval');
  const [formInterval, setFormInterval] = useState(3600);
  const [formMaxRuns, setFormMaxRuns] = useState<number | undefined>(undefined);

  const loadData = async () => {
    setLoading(true);
    try {
      const [tmplRes, execRes] = await Promise.all([
        apiGet('/api/v1/scheduled-templates'),
        apiGet('/api/v1/scheduled-templates/executions'),
      ]);
      const tmplData = await tmplRes.json();
      const execData = await execRes.json();
      setTemplates(Array.isArray(tmplData) ? tmplData : []);
      setExecutions(Array.isArray(execData) ? execData.slice(0, 20) : []);
    } catch (e) {
      console.error('Failed to load scheduled templates:', e);
    }
    setLoading(false);
  };

  useEffect(() => {
    loadData();
  }, []);

  const resetForm = () => {
    setFormName('');
    setFormDesc('');
    setFormContent('');
    setFormType('interval');
    setFormInterval(3600);
    setFormMaxRuns(undefined);
  };

  const handleCreate = async () => {
    try {
      await apiPost('/api/v1/scheduled-templates', {
        name: formName,
        description: formDesc || null,
        task_content: formContent,
        schedule_type: formType,
        interval_secs: formType === 'interval' ? formInterval : undefined,
        max_runs: formMaxRuns || null,
      });
      resetForm();
      setShowCreate(false);
      loadData();
    } catch (e) {
      console.error('Failed to create scheduled template:', e);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Delete this scheduled template?')) return;
    try {
      await fetch(`/api/v1/scheduled-templates/${id}`, { method: 'DELETE' });
      loadData();
    } catch (e) {
      console.error('Failed to delete scheduled template:', e);
    }
  };

  const handleTrigger = async (id: string) => {
    try {
      await apiPost(`/api/v1/scheduled-templates/${id}/trigger`, {});
      loadData();
    } catch (e) {
      console.error('Failed to trigger template:', e);
    }
  };

  const handleToggle = async (template: ScheduledTemplate) => {
    try {
      await apiPost(`/api/v1/scheduled-templates/${template.id}`, {
        enabled: !template.enabled,
      });
      loadData();
    } catch (e) {
      console.error('Failed to toggle template:', e);
    }
  };

  const formatInterval = (secs: number): string => {
    if (secs < 60) return `${secs}s`;
    if (secs < 3600) return `${Math.floor(secs / 60)}m`;
    if (secs < 86400) return `${Math.floor(secs / 3600)}h`;
    return `${Math.floor(secs / 86400)}d`;
  };

  if (loading) {
    return <div className="text-muted-foreground">Loading scheduled templates...</div>;
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-semibold">Scheduled Task Templates</h2>
          <p className="text-muted-foreground text-sm mt-1">
            Automate recurring task execution on a schedule
          </p>
        </div>
        <button
          onClick={() => { resetForm(); setShowCreate(true); }}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90"
        >
          + New Template
        </button>
      </div>

      {/* Tabs */}
      <div className="flex border-b">
        <button
          onClick={() => setActiveTab('templates')}
          className={`px-4 py-2 ${activeTab === 'templates' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground'}`}
        >
          Templates ({templates.length})
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
          <h3 className="font-semibold mb-4">Create Scheduled Template</h3>
          <div className="grid gap-4">
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Name</label>
                <input
                  type="text"
                  value={formName}
                  onChange={e => setFormName(e.target.value)}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  placeholder="Template name"
                />
              </div>
              <div>
                <label className="text-sm font-medium">Schedule Type</label>
                <select
                  value={formType}
                  onChange={e => setFormType(e.target.value as 'interval' | 'cron' | 'onetime')}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                >
                  <option value="interval">Interval</option>
                  <option value="cron">Cron Expression</option>
                  <option value="onetime">One Time</option>
                </select>
              </div>
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
            {formType === 'interval' && (
              <div>
                <label className="text-sm font-medium">Interval (seconds)</label>
                <input
                  type="number"
                  value={formInterval}
                  onChange={e => setFormInterval(parseInt(e.target.value))}
                  className="w-full mt-1 px-3 py-2 rounded border bg-background"
                  min={60}
                />
                <p className="text-xs text-muted-foreground mt-1">
                  Common values: 60 (1m), 300 (5m), 3600 (1h), 86400 (1d)
                </p>
              </div>
            )}
            <div>
              <label className="text-sm font-medium">Max Runs (optional)</label>
              <input
                type="number"
                value={formMaxRuns || ''}
                onChange={e => setFormMaxRuns(e.target.value ? parseInt(e.target.value) : undefined)}
                className="w-full mt-1 px-3 py-2 rounded border bg-background"
                placeholder="Unlimited"
                min={1}
              />
            </div>
            <div>
              <label className="text-sm font-medium">Task Content</label>
              <textarea
                value={formContent}
                onChange={e => setFormContent(e.target.value)}
                className="w-full mt-1 px-3 py-2 rounded border bg-background font-mono text-sm"
                placeholder="echo 'Hello World'"
                rows={4}
              />
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

      {/* Templates List */}
      {activeTab === 'templates' && (
        <div className="border rounded-lg overflow-hidden">
          <table className="w-full">
            <thead className="bg-muted">
              <tr>
                <th className="px-4 py-2 text-left text-sm font-medium">Name</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Schedule</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Runs</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Next Run</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Actions</th>
              </tr>
            </thead>
            <tbody>
              {templates.length === 0 ? (
                <tr>
                  <td colSpan={6} className="px-4 py-8 text-center text-muted-foreground">
                    No scheduled templates configured. Create one to automate recurring tasks.
                  </td>
                </tr>
              ) : templates.map(template => (
                <tr key={template.id} className="border-t">
                  <td className="px-4 py-3">
                    <div className="font-medium">{template.name}</div>
                    {template.description && (
                      <div className="text-xs text-muted-foreground">{template.description}</div>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <span className="px-2 py-1 bg-muted rounded text-xs font-mono">
                      {template.schedule_type}
                    </span>
                    {template.schedule_type === 'interval' && (
                      <span className="ml-2 text-sm">{formatInterval(template.schedule_config.interval_secs)}</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-sm">
                    {template.run_count}
                    {template.max_runs && ` / ${template.max_runs}`}
                  </td>
                  <td className="px-4 py-3 text-sm">
                    {template.next_run_at ? (
                      new Date(template.next_run_at).toLocaleString()
                    ) : (
                      <span className="text-muted-foreground">N/A</span>
                    )}
                  </td>
                  <td className="px-4 py-3">
                    <button
                      onClick={() => handleToggle(template)}
                      className={`px-2 py-1 rounded text-xs cursor-pointer ${
                        template.enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-600'
                      }`}
                    >
                      {template.enabled ? 'Enabled' : 'Disabled'}
                    </button>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleTrigger(template.id)}
                        className="text-sm text-primary hover:underline"
                      >
                        Run Now
                      </button>
                      <button
                        onClick={() => handleDelete(template.id)}
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
                <th className="px-4 py-2 text-left text-sm font-medium">Template</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Status</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Started</th>
                <th className="px-4 py-2 text-left text-sm font-medium">Completed</th>
              </tr>
            </thead>
            <tbody>
              {executions.length === 0 ? (
                <tr>
                  <td colSpan={4} className="px-4 py-8 text-center text-muted-foreground">
                    No executions yet. Trigger a template to see execution history.
                  </td>
                </tr>
              ) : executions.map(exec => (
                <tr key={exec.id} className="border-t">
                  <td className="px-4 py-3 font-medium">{exec.template_name}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 rounded text-xs ${
                      exec.status === 'completed' ? 'bg-green-100 text-green-800' :
                      exec.status === 'failed' ? 'bg-red-100 text-red-800' :
                      exec.status === 'running' ? 'bg-blue-100 text-blue-800' :
                      'bg-gray-100 text-gray-600'
                    }`}>
                      {exec.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm">{new Date(exec.started_at).toLocaleString()}</td>
                  <td className="px-4 py-3 text-sm">
                    {exec.completed_at ? new Date(exec.completed_at).toLocaleString() : '-'}
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
