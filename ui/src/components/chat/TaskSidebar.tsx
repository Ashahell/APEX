import { useAppStore, Task } from '../../stores/appStore';
import { motion, AnimatePresence } from 'framer-motion';
import { useState } from 'react';
import { ThoughtPanel } from './ThoughtPanel';

interface TaskSidebarProps {
  onTaskClick?: (taskId: string) => void;
}

export function TaskSidebar({ onTaskClick }: TaskSidebarProps) {
  const tasks = useAppStore((s) => s.tasks);
  const [showThoughts, setShowThoughts] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  const activeTasks = tasks
    .filter(t => t.status === 'running' || t.status === 'pending')
    .slice(0, 10);
  
  const recentTasks = tasks
    .filter(t => t.status === 'completed' || t.status === 'failed')
    .slice(0, 5);

  const handleTaskClick = (taskId: string) => {
    setSelectedTaskId(taskId);
    setShowThoughts(true);
    onTaskClick?.(taskId);
  };

  const getStatusIcon = (status: Task['status']) => {
    switch (status) {
      case 'running':
        return (
          <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#f59e0b" strokeWidth="2" strokeDasharray="4 2" />
            <circle cx="12" cy="12" r="4" fill="#f59e0b" className="animate-pulse" />
          </svg>
        );
      case 'pending':
        return (
          <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="8" stroke="#3b82f6" strokeWidth="2" />
            <circle cx="12" cy="12" r="3" fill="#3b82f6" />
          </svg>
        );
      case 'completed':
        return (
          <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#22c55e" strokeWidth="2" />
            <path d="M8 12l2.5 2.5L16 9" stroke="#22c55e" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        );
      case 'failed':
        return (
          <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#ef4444" strokeWidth="2" />
            <path d="M15 9l-6 6M9 9l6 6" stroke="#ef4444" strokeWidth="2" strokeLinecap="round" />
          </svg>
        );
      default:
        return (
          <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#6b7280" strokeWidth="2" />
          </svg>
        );
    }
  };

  const getTierBadge = (tier: Task['tier']) => {
    const colors = {
      instant: 'bg-green-500/20 text-green-400 border border-green-500/30',
      shallow: 'bg-blue-500/20 text-blue-400 border border-blue-500/30',
      deep: 'bg-[#4248f1]/20 text-[#4248f1] border border-[#4248f1]/30',
    };
    return (
      <span className={`text-xs px-1.5 py-0.5 rounded-md ${colors[tier]}`}>
        {tier}
      </span>
    );
  };

  const formatElapsed = (createdAt: Date): string => {
    const now = new Date();
    const diff = Math.floor((now.getTime() - createdAt.getTime()) / 1000);
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    return `${Math.floor(diff / 3600)}h`;
  };

  return (
    <div className="w-72 border-l border-[var(--color-border)] bg-[var(--color-panel)] flex flex-col h-full">
      <div className="p-4 border-b border-[var(--color-border)]">
        <h3 className="font-semibold text-[var(--color-text)]">Tasks</h3>
        <p className="text-sm text-[var(--color-text-muted)]">
          {activeTasks.length} active
        </p>
      </div>

      <div className="flex-1 overflow-y-auto">
        {activeTasks.length > 0 && (
          <div className="p-2">
            <h4 className="text-xs font-medium text-[var(--color-text-muted)] uppercase px-2 mb-2">
              Active
            </h4>
            <AnimatePresence>
              {activeTasks.map((task) => (
               <motion.button
                   key={task.id}
                   initial={{ opacity: 0, x: 20 }}
                   animate={{ opacity: 1, x: 0 }}
                   exit={{ opacity: 0, x: -20 }}
                   onClick={() => handleTaskClick(task.id)}
                   className="w-full text-left p-2 rounded-xl border border-transparent hover:border-[var(--color-border)] hover:bg-[var(--color-muted)]/50 transition-colors"
                 >
                  <div className="flex items-center gap-2 mb-1">
                    {getStatusIcon(task.status)}
                    {getTierBadge(task.tier)}
                  </div>
                  <p className="text-sm font-medium truncate text-[var(--color-text)]">
                    {task.skillName || task.input.slice(0, 30)}
                  </p>
                  <div className="flex items-center justify-between mt-1">
                    <span className="text-xs text-[var(--color-text-muted)]">
                      {formatElapsed(task.createdAt)}
                    </span>
                    {task.cost && (
                      <span className="text-xs text-[var(--color-text-muted)]">
                        ${task.cost.toFixed(3)}
                      </span>
                    )}
                  </div>
                </motion.button>
              ))}
            </AnimatePresence>
          </div>
        )}

        {recentTasks.length > 0 && (
          <div className="p-2 border-t border-[var(--color-border)]">
            <h4 className="text-xs font-medium text-[var(--color-text-muted)] uppercase px-2 mb-2">
              Recent
            </h4>
            {recentTasks.map((task) => (
                <button
                 key={task.id}
                 onClick={() => handleTaskClick(task.id)}
                 className="w-full text-left p-2 rounded-xl border border-transparent hover:border-[var(--color-border)] hover:bg-[var(--color-muted)]/50 transition-colors"
               >
                <div className="flex items-center gap-2 mb-1">
                  {getStatusIcon(task.status)}
                  <span className="text-xs text-[var(--color-text-muted)]">
                    {task.status}
                  </span>
                </div>
                <p className="text-sm text-[var(--color-text-muted)] truncate">
                  {task.skillName || task.input.slice(0, 30)}
                </p>
              </button>
            ))}
          </div>
        )}

        {tasks.length === 0 && (
          <div className="p-4 text-center text-[var(--color-text-muted)]">
            <p className="text-sm">No tasks yet</p>
            <p className="text-xs mt-1">Start a conversation to begin</p>
          </div>
        )}
      </div>
      
      <AnimatePresence>
        {showThoughts && selectedTaskId && (
          <ThoughtPanel
            taskId={selectedTaskId}
            onClose={() => setShowThoughts(false)}
          />
        )}
      </AnimatePresence>
    </div>
  );
}
