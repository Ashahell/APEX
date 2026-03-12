import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { StepDetailModal } from './StepDetailModal';

export type StepType = 'GEN' | 'USE' | 'EXE' | 'WWW' | 'SUB' | 'MEM' | 'AUD' | 'MCP';

export interface ProcessStep {
  id: string;
  type: StepType;
  name: string;
  input?: Record<string, unknown>;
  output?: string;
  expanded?: boolean;
}

export interface ProcessGroupProps {
  id: string;
  title: string;
  status: 'running' | 'awaiting_confirmation' | 'completed' | 'failed' | 'timed_out';
  steps: ProcessStep[];
  response?: string;
  elapsed?: string;
  cost?: number;
  onConfirm?: () => void;
  onCancel?: () => void;
}

const STEP_COLORS: Record<StepType, { bg: string; text: string; border: string }> = {
  GEN: { bg: 'bg-[#4248f1]/20', text: 'text-[#4248f1]', border: 'border-l-[#4248f1]' },
  USE: { bg: 'bg-teal-500/20', text: 'text-teal-400', border: 'border-l-teal-500' },
  EXE: { bg: 'bg-amber-500/20', text: 'text-amber-400', border: 'border-l-amber-500' },
  WWW: { bg: 'bg-purple-500/20', text: 'text-purple-400', border: 'border-l-purple-500' },
  SUB: { bg: 'bg-indigo-500/20', text: 'text-indigo-400', border: 'border-l-indigo-500' },
  MEM: { bg: 'bg-green-500/20', text: 'text-green-400', border: 'border-l-green-500' },
  AUD: { bg: 'bg-red-500/20', text: 'text-red-400', border: 'border-l-red-500' },
  MCP: { bg: 'bg-cyan-500/20', text: 'text-cyan-400', border: 'border-l-cyan-500' },
};

const STATUS_COLORS: Record<string, { border: string; bg: string; icon: React.ReactNode }> = {
  running: { border: 'border-l-[#4248f1]', bg: 'bg-[#4248f1]/5', icon: <svg className="w-4 h-4 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg> },
  awaiting_confirmation: { border: 'border-l-amber-500', bg: 'bg-amber-500/5', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" /></svg> },
  completed: { border: 'border-l-green-500', bg: 'bg-green-500/5', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg> },
  failed: { border: 'border-l-red-500', bg: 'bg-red-500/5', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg> },
  timed_out: { border: 'border-l-gray-500', bg: 'bg-gray-500/5', icon: <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" /></svg> },
};

export function ProcessGroup({
  id: _id,
  title,
  status,
  steps,
  response,
  elapsed,
  cost,
  onConfirm,
  onCancel,
}: ProcessGroupProps) {
  const [expanded, setExpanded] = useState(false);
  const [expandedSteps, setExpandedSteps] = useState<Set<string>>(new Set());
  const [selectedStep, setSelectedStep] = useState<ProcessStep | null>(null);

  const statusStyle = STATUS_COLORS[status] || STATUS_COLORS.running;
  const isRunning = status === 'running';

  const toggleStep = (stepId: string) => {
    setExpandedSteps((prev) => {
      const next = new Set(prev);
      if (next.has(stepId)) {
        next.delete(stepId);
      } else {
        next.add(stepId);
      }
      return next;
    });
  };

  const handleStepClick = (step: ProcessStep, e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedStep(step);
  };

  const stepCount = steps.length;

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className={`border-l-4 ${statusStyle.border} ${statusStyle.bg} rounded-lg overflow-hidden mb-4`}
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full p-4 flex items-center justify-between hover:bg-muted/50 transition-colors"
      >
        <div className="flex items-center gap-3">
          <span className="text-lg">{statusStyle.icon}</span>
          <div className="text-left">
            <h4 className="font-medium">{title}</h4>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <span>{stepCount} steps</span>
              {elapsed && <span>• {elapsed}</span>}
              {cost !== undefined && <span>• ${cost.toFixed(3)}</span>}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {isRunning && (
            <span className="animate-pulse text-primary">Running...</span>
          )}
          <span className="text-muted-foreground">
            {expanded ? '▼' : '▶'}
          </span>
        </div>
      </button>

      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="border-t"
          >
            <div className="p-4 space-y-2">
              {steps.map((step) => {
                const stepStyle = STEP_COLORS[step.type];
                const isStepExpanded = expandedSteps.has(step.id);

                return (
                  <div
                    key={step.id}
                    className={`border-l-2 ${stepStyle.border} pl-3 py-2 rounded-r ${stepStyle.bg}`}
                  >
                    <button
                      onClick={() => toggleStep(step.id)}
                      onDoubleClick={(e) => handleStepClick(step, e)}
                      className="w-full flex items-center gap-3 hover:bg-muted/30 rounded transition-colors"
                      title="Double-click for full details"
                    >
                      <span className={`text-xs font-bold px-1.5 py-0.5 rounded ${stepStyle.bg} ${stepStyle.text}`}>
                        {step.type}
                      </span>
                      <span className="flex-1 text-left text-sm">{step.name}</span>
                      <span className="text-muted-foreground text-xs">
                        {isStepExpanded ? '▼' : '▶'}
                      </span>
                    </button>

                    <AnimatePresence>
                      {isStepExpanded && step.input && (
                        <motion.div
                          initial={{ height: 0, opacity: 0 }}
                          animate={{ height: 'auto', opacity: 1 }}
                          exit={{ height: 0, opacity: 0 }}
                          className="mt-2 text-xs font-mono bg-muted p-2 rounded overflow-x-auto"
                        >
                          <pre className="whitespace-pre-wrap">
                            {JSON.stringify(step.input, null, 2)}
                          </pre>
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>
                );
              })}
            </div>

            {response && (
              <div className="border-t p-4 bg-background">
                <h5 className="text-sm font-medium mb-2">Response</h5>
                <div className="text-sm whitespace-pre-wrap">{response}</div>
              </div>
            )}

            {status === 'awaiting_confirmation' && (
              <div className="border-t p-4 bg-amber-500/10">
                <p className="text-sm text-amber-700 dark:text-amber-300 mb-3">
                  ⚠️ This action requires confirmation before proceeding.
                </p>
                <div className="flex gap-2">
                  <button
                    onClick={onConfirm}
                    className="px-4 py-2 bg-amber-500 text-white rounded hover:bg-amber-600 transition-colors"
                  >
                    Confirm
                  </button>
                  <button
                    onClick={onCancel}
                    className="px-4 py-2 border border-amber-500 text-amber-700 rounded hover:bg-amber-50 dark:hover:bg-amber-900/20 transition-colors"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>

      <StepDetailModal step={selectedStep} onClose={() => setSelectedStep(null)} />
    </motion.div>
  );
}
