import { useState, useEffect } from 'react';
import { apiGet, apiPost, apiDelete, apiPut } from '../../lib/api';

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
      <div className="border-b p-4">
        <h2 className="text-2xl font-semibold mb-2">Workflows</h2>
        <p className="text-muted-foreground">Manage YAML workflow definitions and track executions</p>
      </div>

      <div className="border-b">
        <nav className="flex px-4">
          <button
            onClick={() => setActiveTab('list')}
            className={`px-4 py-3 text-sm font-medium transition-colors relative ${
              activeTab === 'list' ? 'text-primary' : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            List
            {activeTab === 'list' && <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />}
          </button>
          <button
            onClick={() => setActiveTab('create')}
            className={`px-4 py-3 text-sm font-medium transition-colors relative ${
              activeTab === 'create' ? 'text-primary' : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            Create
            {activeTab === 'create' && <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />}
          </button>
          {activeTab === 'executions' && (
            <button
              className="px-4 py-3 text-sm font-medium transition-colors relative text-primary"
            >
              Executions
              <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-primary" />
            </button>
          )}
        </nav>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {activeTab === 'list' && (
          <>
            <div className="flex gap-4 mb-4">
              <select
                value={categoryFilter}
                onChange={(e) => setCategoryFilter(e.target.value)}
                className="px-3 py-2 border rounded-lg bg-background"
              >
                <option value="">All Categories</option>
                {filterOptions.categories.map((cat) => (
                  <option key={cat} value={cat}>{cat}</option>
                ))}
              </select>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={showActiveOnly}
                  onChange={(e) => setShowActiveOnly(e.target.checked)}
                />
                Active only
              </label>
              <button
                onClick={() => setActiveTab('create')}
                className="px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 ml-auto"
              >
                + New Workflow
              </button>
            </div>

            {loading ? (
              <div className="text-center py-12">Loading...</div>
            ) : workflows.length === 0 ? (
              <div className="text-center text-muted-foreground py-12">
                <p className="mb-2">No workflows found</p>
                <p className="text-sm">Create a workflow to get started</p>
              </div>
            ) : (
              <div className="space-y-3">
                {workflows.map((workflow) => (
                  <div key={workflow.id} className="border rounded-lg p-4">
                    <div className="flex items-center justify-between mb-2">
                      <h3 className="font-medium">{workflow.name}</h3>
                      <div className="flex items-center gap-2">
                        <span className={`text-xs px-2 py-1 rounded ${
                          workflow.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                        }`}>
                          {workflow.is_active ? 'Active' : 'Inactive'}
                        </span>
                        {workflow.category && (
                          <span className="text-xs px-2 py-1 rounded bg-blue-100 text-blue-800">
                            {workflow.category}
                          </span>
                        )}
                      </div>
                    </div>
                    {workflow.description && (
                      <p className="text-sm text-muted-foreground mb-2">{workflow.description}</p>
                    )}
                    <div className="flex items-center gap-4 text-xs text-muted-foreground mb-3">
                      <span>v{workflow.version}</span>
                      <span>Executions: {workflow.execution_count}</span>
                      {workflow.avg_duration_secs && (
                        <span>Avg: {workflow.avg_duration_secs.toFixed(1)}s</span>
                      )}
                      {workflow.success_rate !== null && (
                        <span>Success: {(workflow.success_rate * 100).toFixed(1)}%</span>
                      )}
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleToggleActive(workflow)}
                        className="px-3 py-1 text-sm border rounded hover:bg-muted"
                      >
                        {workflow.is_active ? 'Pause' : 'Activate'}
                      </button>
                      <button
                        onClick={() => viewExecutions(workflow)}
                        className="px-3 py-1 text-sm border rounded hover:bg-muted"
                      >
                        View Executions
                      </button>
                      <button
                        onClick={() => handleDeleteWorkflow(workflow.id)}
                        className="px-3 py-1 text-sm border rounded hover:bg-muted text-red-500"
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
            <h3 className="text-lg font-medium mb-4">
              Executions: {selectedWorkflow.name}
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
