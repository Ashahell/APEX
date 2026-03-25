import React, { useEffect, useState } from 'react';
import { useAppStore, type ExecutionStep } from '../../stores/appStore';
import { WSClient } from '../../lib/ws';
import { motion, AnimatePresence } from 'framer-motion';

interface HandInfo {
  name: string;
  status: 'idle' | 'running' | 'completed' | 'failed';
  lastUpdated: string;
  taskId?: string;
}

interface HandMonitorProps {
  /** Task IDs for hands to monitor */
  taskIds: string[];
}

const STEP_COLORS: Record<string, { bg: string; text: string }> = {
  Thought: { bg: 'bg-[#4248f1]/20', text: 'text-[#4248f1]' },
  ToolCall: { bg: 'bg-teal-500/20', text: 'text-teal-400' },
  ToolProgress: { bg: 'bg-amber-500/20', text: 'text-amber-400' },
  ToolResult: { bg: 'bg-green-500/20', text: 'text-green-400' },
  ApprovalNeeded: { bg: 'bg-orange-500/20', text: 'text-orange-400' },
  Error: { bg: 'bg-red-500/20', text: 'text-red-400' },
  Complete: { bg: 'bg-green-500/20', text: 'text-green-400' },
};

const HandMonitor: React.FC<HandMonitorProps> = ({ taskIds }) => {
  const [hands, setHands] = useState<HandInfo[]>([]);
  const [expandedHand, setExpandedHand] = useState<string | null>(null);
  const executionSteps = useAppStore((s) => s.executionSteps);

  // Connect WebSocket for each task (Patch 14: replaces SSE)
  useEffect(() => {
    const clients: WSClient[] = [];

    for (const taskId of taskIds) {
      const client = new WSClient(taskId, {
        maxRetries: 3,
        onConnect: () => {
          setHands((prev) =>
            prev.map((h) =>
              h.taskId === taskId ? { ...h, status: 'running' as const, lastUpdated: new Date().toLocaleTimeString() } : h
            )
          );
        },
        onDone: (_tid, reason) => {
          setHands((prev) =>
            prev.map((h) =>
              h.taskId === _tid
                ? { ...h, status: reason === 'complete' ? 'completed' as const : 'failed' as const, lastUpdated: new Date().toLocaleTimeString() }
                : h
            )
          );
          setExpandedHand((prev) => (prev === _tid ? null : prev));
        },
        onError: (_tid) => {
          setHands((prev) =>
            prev.map((h) =>
              h.taskId === _tid ? { ...h, status: 'failed' as const, lastUpdated: new Date().toLocaleTimeString() } : h
            )
          );
        },
      });

      clients.push(client);
      client.connect();

      // Add hand to list if not present
      setHands((prev) => {
        if (prev.some((h) => h.taskId === taskId)) return prev;
        return [
          ...prev,
          { name: `Hand ${prev.length + 1}`, status: 'idle', lastUpdated: '—', taskId },
        ];
      });
    }

    return () => {
      clients.forEach((c) => c.close('cancelled'));
    };
  }, [taskIds.join(',')]); // eslint-disable-line react-hooks/exhaustive-deps

  // Get steps for a specific task
  const getTaskSteps = (taskId: string): ExecutionStep[] =>
    executionSteps.filter((s) => s.taskId === taskId);

  const statusColor = (status: HandInfo['status']) => {
    switch (status) {
      case 'running': return 'text-[#4248f1]';
      case 'completed': return 'text-green-400';
      case 'failed': return 'text-red-400';
      default: return 'text-gray-400';
    }
  };

  const statusDot = (status: HandInfo['status']) => {
    const colors = {
      idle: 'bg-gray-400',
      running: 'bg-[#4248f1] animate-pulse',
      completed: 'bg-green-400',
      failed: 'bg-red-400',
    };
    return <span className={`inline-block w-2 h-2 rounded-full ${colors[status]}`} />;
  };

  return (
    <div
      style={{
        border: '1px solid var(--color-border, #374151)',
        padding: 12,
        borderRadius: 8,
        background: 'var(--color-panel, #1f2937)',
      }}
    >
      <div style={{ fontWeight: 700, marginBottom: 12, fontSize: 14, color: 'var(--color-text, #f9fafb)' }}>
        Hands Monitor
        {hands.some((h) => h.status === 'running') && (
          <span className="ml-2 text-xs text-[#4248f1] animate-pulse">● live</span>
        )}
      </div>

      {hands.length === 0 && (
        <div className="text-sm text-gray-500">No active hands</div>
      )}

      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        <AnimatePresence>
          {hands.map((hand) => {
            const steps = hand.taskId ? getTaskSteps(hand.taskId) : [];
            const isExpanded = expandedHand === hand.taskId;

            return (
              <motion.div
                key={hand.taskId || hand.name}
                initial={{ opacity: 0, y: -4 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -4 }}
              >
                {/* Hand row */}
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    cursor: 'pointer',
                    padding: '4px 0',
                  }}
                  onClick={() => setExpandedHand(isExpanded ? null : hand.taskId || null)}
                >
                  {statusDot(hand.status)}
                  <span style={{ flex: 1, fontSize: 13, color: 'var(--color-text, #f9fafb)' }}>
                    {hand.name}
                  </span>
                  <span className={`text-xs ${statusColor(hand.status)}`}>{hand.status}</span>
                  <span style={{ fontSize: 11, color: '#6b7280' }}>{hand.lastUpdated}</span>
                  <span style={{ fontSize: 11, color: '#6b7280' }}>
                    {isExpanded ? '▲' : '▼'}
                  </span>
                </div>

                {/* Expanded steps */}
                <AnimatePresence>
                  {isExpanded && steps.length > 0 && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: 'auto', opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      style={{ overflow: 'hidden', marginTop: 6, paddingLeft: 12 }}
                    >
                      <div
                        style={{
                          borderLeft: '2px solid var(--color-border, #374151)',
                          paddingLeft: 10,
                          maxHeight: 240,
                          overflowY: 'auto',
                        }}
                      >
                        {steps.map((step) => {
                          const colors = STEP_COLORS[step.type] || STEP_COLORS.Thought;
                          return (
                            <div
                              key={step.id}
                              style={{
                                marginBottom: 6,
                                padding: '4px 6px',
                                borderRadius: 4,
                                background: colors.bg,
                              }}
                            >
                              <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                                <span
                                  className={`text-xs font-bold px-1 py-0.5 rounded ${colors.bg} ${colors.text}`}
                                >
                                  {step.type}
                                </span>
                                <span className="text-xs text-gray-400">Step {step.step}</span>
                              </div>
                              {step.content && (
                                <p
                                  style={{
                                    fontSize: 11,
                                    color: 'var(--color-text-muted, #9ca3af)',
                                    marginTop: 2,
                                    overflow: 'hidden',
                                    textOverflow: 'ellipsis',
                                    display: '-webkit-box',
                                    WebkitLineClamp: 2,
                                    WebkitBoxOrient: 'vertical',
                                  }}
                                >
                                  {step.content}
                                </p>
                              )}
                              {step.tool && (
                                <span className="text-xs text-gray-500">tool: {step.tool}</span>
                              )}
                            </div>
                          );
                        })}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>
    </div>
  );
};

export default HandMonitor;
