import { motion, AnimatePresence } from 'framer-motion';
import { ProcessStep } from './ProcessGroup';

interface StepDetailModalProps {
  step: ProcessStep | null;
  onClose: () => void;
}

const STEP_TYPE_LABELS: Record<string, string> = {
  GEN: 'Generate (LLM)',
  USE: 'Use Skill',
  EXE: 'Execute Code',
  WWW: 'Web Search',
  SUB: 'Subagent',
  MEM: 'Memory',
  AUD: 'Audit',
};

export function StepDetailModal({ step, onClose }: StepDetailModalProps) {
  if (!step) return null;

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.95, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.95, opacity: 0 }}
          className="bg-background border rounded-lg shadow-xl w-full max-w-3xl max-h-[90vh] overflow-hidden flex flex-col"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b bg-muted/30">
            <div className="flex items-center gap-3">
              <span className={`text-xs font-bold px-2 py-1 rounded ${
                step.type === 'GEN' ? 'bg-blue-500/20 text-blue-400' :
                step.type === 'USE' ? 'bg-teal-500/20 text-teal-400' :
                step.type === 'EXE' ? 'bg-amber-500/20 text-amber-400' :
                step.type === 'WWW' ? 'bg-purple-500/20 text-purple-400' :
                step.type === 'SUB' ? 'bg-indigo-500/20 text-indigo-400' :
                step.type === 'MEM' ? 'bg-green-500/20 text-green-400' :
                'bg-red-500/20 text-red-400'
              }`}>
                {step.type}
              </span>
              <h2 className="text-lg font-semibold">{step.name}</h2>
            </div>
            <button
              onClick={onClose}
              className="p-2 hover:bg-muted rounded-lg transition-colors"
            >
              ✕
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {/* Step Type Info */}
            <div className="bg-muted/30 rounded-lg p-3">
              <div className="flex items-center gap-2 text-sm">
                <span className="text-muted-foreground">Type:</span>
                <span className="font-medium">{STEP_TYPE_LABELS[step.type] || step.type}</span>
              </div>
              <div className="flex items-center gap-2 text-sm mt-1">
                <span className="text-muted-foreground">ID:</span>
                <code className="text-xs bg-muted px-1 py-0.5 rounded">{step.id}</code>
              </div>
            </div>

            {/* Input Section */}
            {step.input && Object.keys(step.input).length > 0 && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <h3 className="font-medium text-sm text-muted-foreground uppercase tracking-wide">
                    Input
                  </h3>
                  <button
                    onClick={() => handleCopy(JSON.stringify(step.input, null, 2))}
                    className="text-xs px-2 py-1 hover:bg-muted rounded transition-colors"
                  >
                    📋 Copy
                  </button>
                </div>
                <div className="bg-muted/50 rounded-lg p-3 max-h-64 overflow-auto">
                  <pre className="text-xs font-mono whitespace-pre-wrap">
                    {JSON.stringify(step.input, null, 2)}
                  </pre>
                </div>
              </div>
            )}

            {/* Output Section */}
            {step.output && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <h3 className="font-medium text-sm text-muted-foreground uppercase tracking-wide">
                    Output
                  </h3>
                  <button
                    onClick={() => handleCopy(step.output || '')}
                    className="text-xs px-2 py-1 hover:bg-muted rounded transition-colors"
                  >
                    📋 Copy
                  </button>
                </div>
                <div className="bg-muted/50 rounded-lg p-3 max-h-64 overflow-auto">
                  <pre className="text-xs font-mono whitespace-pre-wrap">
                    {step.output}
                  </pre>
                </div>
              </div>
            )}

            {/* Empty State */}
            {!step.input && !step.output && (
              <div className="text-center py-8 text-muted-foreground">
                No input or output data available for this step.
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-2 p-4 border-t bg-muted/30">
            <button
              onClick={() => handleCopy(JSON.stringify(step, null, 2))}
              className="px-4 py-2 text-sm border rounded-lg hover:bg-muted transition-colors"
            >
              Copy All as JSON
            </button>
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg hover:opacity-90 transition-colors"
            >
              Close
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
