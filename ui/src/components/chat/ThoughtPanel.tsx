import { useAppStore } from '../../stores/appStore';
import { motion, AnimatePresence } from 'framer-motion';

interface ThoughtPanelProps {
  taskId: string;
  onClose: () => void;
}

export function ThoughtPanel({ taskId, onClose }: ThoughtPanelProps) {
  const executionSteps = useAppStore((state) => 
    state.executionSteps.filter((step) => step.taskId === taskId && step.type === 'Thought')
  );

  if (executionSteps.length === 0) {
    return null;
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      className="absolute top-0 right-0 w-80 h-full bg-[var(--color-panel)] border-l border-[var(--color-border)] shadow-lg z-50 overflow-hidden flex flex-col"
    >
      <div className="p-4 border-b border-[var(--color-border)] flex items-center justify-between">
        <h3 className="font-semibold text-[var(--color-text)]">Agent Thoughts</h3>
        <button
          onClick={onClose}
          className="p-1 rounded hover:bg-[var(--color-muted)] transition-colors"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <AnimatePresence mode="popLayout">
          {executionSteps.map((step, index) => (
            <motion.div
              key={step.id}
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              transition={{ delay: index * 0.05 }}
              className="bg-[var(--color-muted)]/30 rounded-lg p-3 border-l-2 border-[#4248f1]"
            >
              <div className="flex items-center gap-2 mb-2">
                <span className="text-xs text-[var(--color-text-muted)]">
                  Step {step.step}
                </span>
                <span className="text-xs text-[var(--color-text-muted)]">
                  {new Date(step.timestamp).toLocaleTimeString()}
                </span>
              </div>
              <p className="text-sm text-[var(--color-text)] leading-relaxed">
                {step.content}
              </p>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </motion.div>
  );
}

export function useThoughts(taskId: string) {
  const executionSteps = useAppStore((state) => 
    state.executionSteps.filter((step) => step.taskId === taskId && step.type === 'Thought')
  );
  return executionSteps;
}
