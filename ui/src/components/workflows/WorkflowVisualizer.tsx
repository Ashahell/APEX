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
    case 'trigger': return 'bg-green-500';
    case 'action': return 'bg-blue-500';
    case 'condition': return 'bg-yellow-500';
    case 'delay': return 'bg-purple-500';
    case 'end': return 'bg-red-500';
    default: return 'bg-gray-500';
  }
}

function getStatusColor(status: string): { bg: string; text: string; icon: string } {
  switch (status) {
    case 'completed': return { bg: 'bg-green-100', text: 'text-green-800', icon: '✓' };
    case 'running': return { bg: 'bg-blue-100', text: 'text-blue-800', icon: '⟳' };
    case 'failed': return { bg: 'bg-red-100', text: 'text-red-800', icon: '✕' };
    case 'pending': return { bg: 'bg-yellow-100', text: 'text-yellow-800', icon: '◷' };
    case 'cancelled': return { bg: 'bg-gray-100', text: 'text-gray-800', icon: '⊘' };
    default: return { bg: 'bg-gray-100', text: 'text-gray-800', icon: '•' };
  }
}

export function WorkflowVisualizer({ workflow, executions }: WorkflowVisualizerProps) {
  const [view, setView] = useState<'flowchart' | 'timeline'>('flowchart');
  const { nodes, connections } = parseWorkflowDefinition(workflow.definition);
  
  const maxDuration = Math.max(...executions.map(e => e.duration_secs || 0), 1);
  
  return (
    <div className="border rounded-xl overflow-hidden">
      <div className="bg-muted/30 p-3 border-b flex items-center justify-between">
        <div>
          <h3 className="font-semibold">{workflow.name}</h3>
          <p className="text-xs text-muted-foreground">
            {workflow.execution_count} executions • avg {workflow.avg_duration_secs?.toFixed(1) || '-'}s • {workflow.success_rate?.toFixed(0) || '-'}% success
          </p>
        </div>
        <div className="flex gap-1">
          <button
            onClick={() => setView('flowchart')}
            className={`px-3 py-1.5 text-sm rounded ${view === 'flowchart' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}`}
          >
            Flow
          </button>
          <button
            onClick={() => setView('timeline')}
            className={`px-3 py-1.5 text-sm rounded ${view === 'timeline' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'}`}
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
                  <div className={`w-3 h-3 rounded ${getNodeColor(type as Node['type'])}`} />
                  <span className="text-xs text-muted-foreground">{label}</span>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-3">
            {executions.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                <div className="text-2xl mb-2">📋</div>
                <p>No executions yet</p>
              </div>
            ) : (
              executions.slice(0, 10).map((exec) => {
                const status = getStatusColor(exec.status);
                return (
                  <div key={exec.id} className="flex items-center gap-3 p-3 border rounded-lg">
                    <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm ${status.bg} ${status.text}`}>
                      {status.icon}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className={`font-medium ${status.text}`}>
                          {exec.status.charAt(0).toUpperCase() + exec.status.slice(1)}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {exec.duration_secs ? `${exec.duration_secs}s` : '-'}
                        </span>
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {new Date(exec.started_at_ms).toLocaleString()}
                        {exec.triggered_by && ` • by ${exec.triggered_by}`}
                      </div>
                    </div>
                    {exec.duration_secs && (
                      <div className="w-24">
                        <div className="h-1.5 bg-muted rounded-full overflow-hidden">
                          <div
                            className="h-full bg-blue-500 rounded-full"
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
        <div className="p-3 border-t bg-muted/30">
          <div className="flex items-center justify-between text-xs text-muted-foreground">
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
