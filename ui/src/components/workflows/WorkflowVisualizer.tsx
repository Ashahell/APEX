import { useState } from 'react';

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

interface WorkflowVisualizerProps {
  workflow: Workflow;
  executions: WorkflowExecution[];
}

interface Node {
  id: string;
  label: string;
  type: 'trigger' | 'action' | 'condition' | 'delay' | 'end';
  x: number;
  y: number;
}

interface Connection {
  from: string;
  to: string;
  label?: string;
}

function parseWorkflowDefinition(definition: string): { nodes: Node[]; connections: Connection[] } {
  const nodes: Node[] = [];
  const connections: Connection[] = [];
  
  try {
    const parsed = JSON.parse(definition);
    const steps = parsed.steps || parsed.nodes || [];
    
    let y = 0;
    steps.forEach((step: { id?: string; name?: string; type?: string; action?: string; condition?: string; delay?: string }, index: number) => {
      const id = step.id || `step-${index}`;
      const label = step.name || step.action || step.condition || step.delay || 'Step';
      const type = (step.type as Node['type']) || 'action';
      
      nodes.push({
        id,
        label,
        type: type === 'trigger' ? 'trigger' : type === 'condition' ? 'condition' : type === 'delay' ? 'delay' : type === 'end' ? 'end' : 'action',
        x: 150,
        y: y * 80 + 40,
      });
      
      if (index > 0) {
        connections.push({
          from: nodes[index - 1].id,
          to: id,
        });
      }
    });
    
    if (nodes.length === 0) {
      nodes.push({ id: 'start', label: 'Start', type: 'trigger', x: 150, y: 40 });
      nodes.push({ id: 'end', label: 'End', type: 'end', x: 150, y: 120 });
      connections.push({ from: 'start', to: 'end' });
    }
  } catch {
    nodes.push({ id: 'start', label: 'Start', type: 'trigger', x: 150, y: 40 });
    nodes.push({ id: 'end', label: 'End', type: 'end', x: 150, y: 120 });
    connections.push({ from: 'start', to: 'end' });
  }
  
  return { nodes, connections };
}

function getNodeColor(type: Node['type']): string {
  switch (type) {
    case 'trigger': return '#4248f1';
    case 'action': return '#3b82f6';
    case 'condition': return '#f59e0b';
    case 'delay': return '#8b5cf6';
    case 'end': return '#ef4444';
    default: return '#6b7280';
  }
}

function getStatusColor(status: string): { bg: string; text: string; icon: React.ReactNode } {
  switch (status) {
    case 'completed': return { bg: 'bg-green-500/20', text: 'text-green-500', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg> };
    case 'running': return { bg: 'bg-[#4248f1]/20', text: 'text-[#4248f1]', icon: <svg className="w-4 h-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg> };
    case 'failed': return { bg: 'bg-red-500/20', text: 'text-red-500', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg> };
    case 'pending': return { bg: 'bg-yellow-500/20', text: 'text-yellow-500', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg> };
    case 'cancelled': return { bg: 'bg-gray-500/20', text: 'text-gray-500', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" /></svg> };
    default: return { bg: 'bg-gray-500/20', text: 'text-gray-500', icon: <span className="w-4 h-4">•</span> };
  }
}

export function WorkflowVisualizer({ workflow, executions }: WorkflowVisualizerProps) {
  const [view, setView] = useState<'flowchart' | 'timeline'>('flowchart');
  const { nodes, connections } = parseWorkflowDefinition(workflow.definition);
  
  const maxDuration = Math.max(...executions.map(e => e.duration_secs || 0), 1);
  
  return (
    <div className="border border-border rounded-xl overflow-hidden bg-[var(--color-panel)]">
      <div className="p-3 border-b border-border flex items-center justify-between">
        <div>
          <h3 className="font-semibold">{workflow.name}</h3>
          <p className="text-xs text-[var(--color-text-muted)]">
            {workflow.execution_count} executions • avg {workflow.avg_duration_secs?.toFixed(1) || '-'}s • {workflow.success_rate?.toFixed(0) || '-'}% success
          </p>
        </div>
        <div className="flex gap-1">
          <button
            onClick={() => setView('flowchart')}
            className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
              view === 'flowchart' ? 'bg-[#4248f1] text-white' : 'hover:bg-[#4248f1]/20'
            }`}
          >
            Flow
          </button>
          <button
            onClick={() => setView('timeline')}
            className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${
              view === 'timeline' ? 'bg-[#4248f1] text-white' : 'hover:bg-[#4248f1]/20'
            }`}
          >
            Timeline
          </button>
        </div>
      </div>
      
      <div className="p-4 min-h-[300px]">
        {view === 'flowchart' ? (
          <div className="relative">
            <svg viewBox="0 0 300 400" className="w-full h-[300px]">
              {connections.map((conn, i) => {
                const fromNode = nodes.find(n => n.id === conn.from);
                const toNode = nodes.find(n => n.id === conn.to);
                if (!fromNode || !toNode) return null;
                return (
                  <g key={i}>
                    <line
                      x1={fromNode.x}
                      y1={fromNode.y}
                      x2={toNode.x}
                      y2={toNode.y}
                      stroke="#94a3b8"
                      strokeWidth="2"
                      markerEnd="url(#arrowhead)"
                    />
                    {conn.label && (
                      <text
                        x={(fromNode.x + toNode.x) / 2}
                        y={(fromNode.y + toNode.y) / 2 - 10}
                        fill="#64748b"
                        fontSize="10"
                        textAnchor="middle"
                      >
                        {conn.label}
                      </text>
                    )}
                  </g>
                );
              })}
              <defs>
                <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                  <polygon points="0 0, 10 3.5, 0 7" fill="#94a3b8" />
                </marker>
              </defs>
              {nodes.map((node) => (
                <g key={node.id} transform={`translate(${node.x - 40}, ${node.y - 20})`}>
                  <rect
                    width="80"
                    height="40"
                    rx="8"
                    className={getNodeColor(node.type)}
                    fillOpacity="0.2"
                    stroke={getNodeColor(node.type)}
                    strokeWidth="2"
                  />
                  <text
                    x="40"
                    y="25"
                    textAnchor="middle"
                    fill="currentColor"
                    fontSize="11"
                    fontWeight="500"
                    className="fill-foreground"
                  >
                    {node.label.length > 10 ? node.label.slice(0, 10) + '...' : node.label}
                  </text>
                </g>
              ))}
            </svg>
            
            <div className="flex flex-wrap gap-3 mt-4 justify-center">
              {[
                { type: 'trigger', label: 'Trigger' },
                { type: 'action', label: 'Action' },
                { type: 'condition', label: 'Condition' },
                { type: 'delay', label: 'Delay' },
                { type: 'end', label: 'End' },
              ].map(({ type, label }) => (
                <div key={type} className="flex items-center gap-2">
                  <div 
                    className="w-3 h-3 rounded" 
                    style={{ backgroundColor: getNodeColor(type as Node['type']) }}
                  />
                  <span className="text-xs text-[var(--color-text-muted)]">{label}</span>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {executions.length === 0 ? (
              <div className="text-center py-8 text-[var(--color-text-muted)]">
                <svg className="w-12 h-12 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                </svg>
                <p>No executions yet</p>
              </div>
            ) : (
              executions.slice(0, 10).map((exec) => {
                const status = getStatusColor(exec.status);
                return (
                  <div key={exec.id} className="flex items-center gap-3 p-3 border border-border rounded-xl hover:bg-[#4248f1]/5 transition-colors">
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm ${status.bg} ${status.text}`}>
                      {status.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className={`font-medium ${status.text}`}>
                          {exec.status.charAt(0).toUpperCase() + exec.status.slice(1)}
                        </span>
                        <span className="text-xs text-[var(--color-text-muted)]">
                          {exec.duration_secs ? `${exec.duration_secs}s` : '-'}
                        </span>
                      </div>
                      <div className="text-xs text-[var(--color-text-muted)]">
                        {new Date(exec.started_at_ms).toLocaleString()}
                        {exec.triggered_by && ` • by ${exec.triggered_by}`}
                      </div>
                    </div>
                    {exec.duration_secs && (
                      <div className="w-24">
                        <div className="h-1.5 bg-[var(--color-background)] rounded-full overflow-hidden">
                          <div
                            className="h-full bg-[#4248f1] rounded-full"
                            style={{ width: `${(exec.duration_secs / maxDuration) * 100}%` }}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                );
              })
            )}
          </div>
        )}
      </div>
      
      {executions.length > 0 && (
        <div className="p-3 border-t border-border">
          <div className="flex items-center justify-between text-xs text-[var(--color-text-muted)]">
            <span>
              {executions.filter(e => e.status === 'completed').length} completed
              {' • '}
              {executions.filter(e => e.status === 'failed').length} failed
            </span>
            <span>
              Success rate: {workflow.success_rate?.toFixed(1) || '-'}%
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
