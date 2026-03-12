import { useState, useEffect } from 'react';
import { apiGet, apiPost, apiDelete, apiPut } from '../../lib/api';
import { WorkflowVisualizer } from './WorkflowVisualizer';

interface Workflow {
  id: string;
  name: string;
  description: string | null;
  definition: string;
  category: string | null;
  version: number;
  is_active: boolean;
  created_at_ms: number;
  updated_at_ms: number;
  last_executed_at_ms: number | null;
  execution_count: number;
  avg_duration_secs: number | null;
  success_rate: number | null;
}

interface WorkflowExecution {
  id: string;
  workflow_id: string;
  status: string;
  started_at_ms: number;
  completed_at_ms: number | null;
  duration_secs: number | null;
  input_data: string | null;
  output_data: string | null;
  error_message: string | null;
  triggered_by: string | null;
}

interface FilterOptions {
  categories: string[];
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}

export function Workflows() {
  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [filterOptions, setFilterOptions] = useState<FilterOptions>({ categories: [] });
  const [activeTab, setActiveTab] = useState<'list' | 'create' | 'executions'>('list');
  const [selectedWorkflow, setSelectedWorkflow] = useState<Workflow | null>(null);
  const [executions, setExecutions] = useState<WorkflowExecution[]>([]);
  const [loading, setLoading] = useState(false);
  const [categoryFilter, setCategoryFilter] = useState<string>('');
  const [showActiveOnly, setShowActiveOnly] = useState(false);
  
  const [newWorkflow, setNewWorkflow] = useState({
    name: '',
    description: '',
    definition: '',
    category: '',
  });

  useEffect(() => {
    loadWorkflows();
    loadFilterOptions();
  }, [categoryFilter, showActiveOnly]);

  async function loadWorkflows() {
    setLoading(true);
    try {
      const params = new URLSearchParams();
      if (categoryFilter) params.append('category', categoryFilter);
      if (showActiveOnly) params.append('active_only', 'true');
      
      const response = await apiGet(`/api/v1/workflows?${params}`);
      if (response.ok) {
        const data = await response.json();
        setWorkflows(data);
      }
    } catch (error) {
      console.error('Failed to load workflows:', error);
    } finally {
      setLoading(false);
    }
  }

  async function loadFilterOptions() {
    try {
      const response = await apiGet('/api/v1/workflows/filter-options');
      if (response.ok) {
        const data = await response.json();
        setFilterOptions(data);
      }
    } catch (error) {
      console.error('Failed to load filter options:', error);
    }
  }

  async function loadExecutions(workflowId: string) {
    try {
      const response = await apiGet(`/api/v1/workflows/${workflowId}/executions`);
      if (response.ok) {
        const data = await response.json();
        setExecutions(data);
      }
    } catch (error) {
      console.error('Failed to load executions:', error);
    }
  }

  async function handleCreateWorkflow() {
    try {
      const response = await apiPost('/api/v1/workflows', {
        name: newWorkflow.name,
        description: newWorkflow.description || null,
        definition: newWorkflow.definition,
        category: newWorkflow.category || null,
      });
      if (response.ok) {
        setNewWorkflow({ name: '', description: '', definition: '', category: '' });
        setActiveTab('list');
        loadWorkflows();
      }
    } catch (error) {
      console.error('Failed to create workflow:', error);
    }
  }

  async function handleDeleteWorkflow(id: string) {
    if (!confirm('Are you sure you want to delete this workflow?')) return;
    try {
      const response = await apiDelete(`/api/v1/workflows/${id}`);
      if (response.ok) {
        loadWorkflows();
      }
    } catch (error) {
      console.error('Failed to delete workflow:', error);
    }
  }

  async function handleToggleActive(workflow: Workflow) {
    try {
      const response = await apiPut(`/api/v1/workflows/${workflow.id}`, {
        is_active: !workflow.is_active,
      });
      if (response.ok) {
        loadWorkflows();
      }
    } catch (error) {
      console.error('Failed to toggle workflow:', error);
    }
  }

  function viewExecutions(workflow: Workflow) {
    setSelectedWorkflow(workflow);
    loadExecutions(workflow.id);
    setActiveTab('executions');
  }

  return (
    <div className="flex flex-col h-full">
      <div className="border-b p-4 bg-[var(--color-panel)]">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-[#4248f1]/10 flex items-center justify-center">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
            </svg>
          </div>
          <div>
            <h2 className="text-xl font-semibold">Workflows</h2>
            <p className="text-sm text-[var(--color-text-muted)]">Manage YAML workflow definitions and track executions</p>
          </div>
        </div>
      </div>

      <div className="border-b bg-[var(--color-panel)]">
        <nav className="flex px-4 gap-1">
          <button
            onClick={() => setActiveTab('list')}
            className={`px-4 py-3 text-sm font-medium transition-colors relative ${
              activeTab === 'list' ? 'text-[#4248f1]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
            }`}
          >
            <span className="flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <line x1="8" y1="6" x2="21" y2="6"></line>
                <line x1="8" y1="12" x2="21" y2="12"></line>
                <line x1="8" y1="18" x2="21" y2="18"></line>
                <line x1="3" y1="6" x2="3.01" y2="6"></line>
                <line x1="3" y1="12" x2="3.01" y2="12"></line>
                <line x1="3" y1="18" x2="3.01" y2="18"></line>
              </svg>
              List
            </span>
            {activeTab === 'list' && <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-[#4248f1]" />}
          </button>
          <button
            onClick={() => setActiveTab('create')}
            className={`px-4 py-3 text-sm font-medium transition-colors relative ${
              activeTab === 'create' ? 'text-[#4248f1]' : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
            }`}
          >
            <span className="flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
              Create
            </span>
            {activeTab === 'create' && <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-[#4248f1]" />}
          </button>
          {activeTab === 'executions' && (
            <button
              className="px-4 py-3 text-sm font-medium transition-colors relative text-[#4248f1]"
            >
              <span className="flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                  <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
                Executions
              </span>
              <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-[#4248f1]" />
            </button>
          )}
        </nav>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {activeTab === 'list' && (
          <>
            <div className="flex gap-4 mb-4 items-center flex-wrap">
              <select
                value={categoryFilter}
                onChange={(e) => setCategoryFilter(e.target.value)}
                className="px-3 py-2 border border-[var(--color-border)] rounded-lg bg-[var(--color-background)] text-[var(--color-text)]"
              >
                <option value="">All Categories</option>
                {filterOptions.categories.map((cat) => (
                  <option key={cat} value={cat}>{cat}</option>
                ))}
              </select>
              <label className="flex items-center gap-2 text-sm text-[var(--color-text)]">
                <input
                  type="checkbox"
                  checked={showActiveOnly}
                  onChange={(e) => setShowActiveOnly(e.target.checked)}
                  className="rounded"
                />
                Active only
              </label>
              <button
                onClick={() => setActiveTab('create')}
                className="px-4 py-2 bg-[#4248f1] text-white rounded-lg hover:bg-[#353bc5] transition-colors ml-auto flex items-center gap-2"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="12" y1="5" x2="12" y2="19"></line>
                  <line x1="5" y1="12" x2="19" y2="12"></line>
                </svg>
                New Workflow
              </button>
            </div>

            {loading ? (
              <div className="text-center py-12 text-[var(--color-text-muted)]">Loading...</div>
            ) : workflows.length === 0 ? (
              <div className="text-center text-[var(--color-text-muted)] py-12">
                <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="mx-auto opacity-50">
                  <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"></polyline>
                </svg>
                <p className="mt-4 mb-2">No workflows found</p>
                <p className="text-sm">Create a workflow to get started</p>
              </div>
            ) : (
              <div className="space-y-3">
                {workflows.map((workflow) => (
                  <div key={workflow.id} className="border border-[var(--color-border)] rounded-xl p-4 hover:border-[#4248f1]/30 transition-colors">
                    <div className="flex items-center justify-between mb-3">
                      <h3 className="font-semibold text-[var(--color-text)]">{workflow.name}</h3>
                      <div className="flex items-center gap-2">
                        <span className={`text-xs px-2.5 py-1 rounded-full font-medium ${
                          workflow.is_active ? 'bg-green-500/10 text-green-500 border border-green-500/20' : 'bg-muted text-[var(--color-text-muted)]'
                        }`}>
                          {workflow.is_active ? 'Active' : 'Inactive'}
                        </span>
                        {workflow.category && (
                          <span className="text-xs px-2.5 py-1 rounded-full bg-[#4248f1]/10 text-[#4248f1] border border-[#4248f1]/20">
                            {workflow.category}
                          </span>
                        )}
                      </div>
                    </div>
                    {workflow.description && (
                      <p className="text-sm text-[var(--color-text-muted)] mb-3">{workflow.description}</p>
                    )}
                    <div className="flex items-center gap-4 text-xs text-[var(--color-text-muted)] mb-4">
                      <span className="flex items-center gap-1">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle></svg>
                        v{workflow.version}
                      </span>
                      <span className="flex items-center gap-1">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
                        {workflow.execution_count} executions
                      </span>
                      {workflow.avg_duration_secs && (
                        <span className="flex items-center gap-1">
                          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>
                          {workflow.avg_duration_secs.toFixed(1)}s avg
                        </span>
                      )}
                      {workflow.success_rate !== null && (
                        <span className="flex items-center gap-1">
                          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
                          {(workflow.success_rate * 100).toFixed(1)}% success
                        </span>
                      )}
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleToggleActive(workflow)}
                        className="px-3 py-1.5 text-sm border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors"
                      >
                        {workflow.is_active ? 'Pause' : 'Activate'}
                      </button>
                      <button
                        onClick={() => viewExecutions(workflow)}
                        className="px-3 py-1.5 text-sm border border-[var(--color-border)] rounded-lg hover:bg-[var(--color-muted)] transition-colors"
                      >
                        View Executions
                      </button>
                      <button
                        onClick={() => handleDeleteWorkflow(workflow.id)}
                        className="px-3 py-1.5 text-sm border border-[var(--color-border)] rounded-lg hover:bg-red-500/10 text-red-500 transition-colors"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </>
        )}

        {activeTab === 'create' && (
          <div className="max-w-2xl mx-auto">
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Name</label>
                <input
                  type="text"
                  value={newWorkflow.name}
                  onChange={(e) => setNewWorkflow({ ...newWorkflow, name: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background"
                  placeholder="My Workflow"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Description</label>
                <input
                  type="text"
                  value={newWorkflow.description}
                  onChange={(e) => setNewWorkflow({ ...newWorkflow, description: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background"
                  placeholder="Optional description"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Category</label>
                <input
                  type="text"
                  value={newWorkflow.category}
                  onChange={(e) => setNewWorkflow({ ...newWorkflow, category: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background"
                  placeholder="e.g., deployment, data-processing"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">YAML Definition</label>
                <textarea
                  value={newWorkflow.definition}
                  onChange={(e) => setNewWorkflow({ ...newWorkflow, definition: e.target.value })}
                  className="w-full px-3 py-2 border rounded-lg bg-background font-mono text-sm"
                  rows={15}
                  placeholder={`steps:
  - name: Step 1
    action: some-action
    input:
      key: value`}
                />
              </div>
              <div className="flex gap-2">
                <button
                  onClick={handleCreateWorkflow}
                  disabled={!newWorkflow.name || !newWorkflow.definition}
                  className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50"
                >
                  Create Workflow
                </button>
                <button
                  onClick={() => setActiveTab('list')}
                  className="px-4 py-2 border rounded-lg hover:bg-muted"
                >
                  Cancel
                </button>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'executions' && selectedWorkflow && (
          <>
            <button
              onClick={() => setActiveTab('list')}
              className="mb-4 px-3 py-1 text-sm border rounded hover:bg-muted"
            >
              ← Back to List
            </button>
            <WorkflowVisualizer workflow={selectedWorkflow} executions={executions} />
            <h3 className="text-lg font-medium mt-6 mb-4">
              Execution History: {selectedWorkflow.name}
            </h3>
            {executions.length === 0 ? (
              <div className="text-center text-muted-foreground py-12">
                No executions yet
              </div>
            ) : (
              <div className="space-y-3">
                {executions.map((exec) => (
                  <div key={exec.id} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <span className={`text-xs px-2 py-1 rounded ${
                        exec.status === 'completed' ? 'bg-green-100 text-green-800' :
                        exec.status === 'failed' ? 'bg-red-100 text-red-800' :
                        exec.status === 'running' ? 'bg-blue-100 text-blue-800' :
                        'bg-gray-100 text-gray-800'
                      }`}>
                        {exec.status}
                      </span>
                      <span className="text-xs text-muted-foreground">
                        {formatDate(exec.started_at_ms)}
                      </span>
                    </div>
                    <div className="flex items-center gap-4 text-xs text-muted-foreground">
                      {exec.duration_secs && <span>Duration: {exec.duration_secs.toFixed(2)}s</span>}
                      {exec.triggered_by && <span>Triggered by: {exec.triggered_by}</span>}
                    </div>
                    {exec.error_message && (
                      <p className="text-sm text-red-500 mt-2">{exec.error_message}</p>
                    )}
                  </div>
                ))}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
