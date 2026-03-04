import { useAppStore, Task } from '../../stores/appStore';
import { motion, AnimatePresence } from 'framer-motion';

interface TaskSidebarProps {
  onTaskClick?: (taskId: string) => void;
}

export function TaskSidebar({ onTaskClick }: TaskSidebarProps) {
  const tasks = useAppStore((s) => s.tasks);

  const activeTasks = tasks
    .filter(t => t.status === 'running' || t.status === 'pending')
    .slice(0, 10);
  
  const recentTasks = tasks
    .filter(t => t.status === 'completed' || t.status === 'failed')
    .slice(0, 5);

  const getStatusIcon = (status: Task['status']) => {
    switch (status) {
      case 'running':
        return <span className="w-2 h-2 bg-amber-500 rounded-full animate-pulse" />;
      case 'pending':
        return <span className="w-2 h-2 bg-blue-500 rounded-full" />;
      case 'completed':
        return <span className="w-2 h-2 bg-green-500 rounded-full" />;
      case 'failed':
        return <span className="w-2 h-2 bg-red-500 rounded-full" />;
      default:
        return <span className="w-2 h-2 bg-gray-500 rounded-full" />;
    }
  };

  const getTierBadge = (tier: Task['tier']) => {
    const colors = {
      instant: 'bg-green-500/20 text-green-400',
      shallow: 'bg-blue-500/20 text-blue-400',
      deep: 'bg-purple-500/20 text-purple-400',
    };
    return (
      <span className={`text-xs px-1.5 py-0.5 rounded ${colors[tier]}`}>
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
    <div className="w-72 border-l bg-card flex flex-col h-full">
      <div className="p-4 border-b">
        <h3 className="font-semibold">Tasks</h3>
        <p className="text-sm text-muted-foreground">
          {activeTasks.length} active
        </p>
      </div>

      <div className="flex-1 overflow-y-auto">
        {activeTasks.length > 0 && (
          <div className="p-2">
            <h4 className="text-xs font-medium text-muted-foreground uppercase px-2 mb-2">
              Active
            </h4>
            <AnimatePresence>
              {activeTasks.map((task) => (
                <motion.button
                  key={task.id}
                  initial={{ opacity: 0, x: 20 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: -20 }}
                  onClick={() => onTaskClick?.(task.id)}
                  className="w-full text-left p-2 rounded-lg hover:bg-muted transition-colors"
                >
                  <div className="flex items-center gap-2 mb-1">
                    {getStatusIcon(task.status)}
                    {getTierBadge(task.tier)}
                  </div>
                  <p className="text-sm font-medium truncate">
                    {task.skillName || task.input.slice(0, 30)}
                  </p>
                  <div className="flex items-center justify-between mt-1">
                    <span className="text-xs text-muted-foreground">
                      {formatElapsed(task.createdAt)}
                    </span>
                    {task.cost && (
                      <span className="text-xs text-muted-foreground">
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
          <div className="p-2 border-t">
            <h4 className="text-xs font-medium text-muted-foreground uppercase px-2 mb-2">
              Recent
            </h4>
            {recentTasks.map((task) => (
              <button
                key={task.id}
                onClick={() => onTaskClick?.(task.id)}
                className="w-full text-left p-2 rounded-lg hover:bg-muted transition-colors"
              >
                <div className="flex items-center gap-2 mb-1">
                  {getStatusIcon(task.status)}
                  <span className="text-xs text-muted-foreground">
                    {task.status}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground truncate">
                  {task.skillName || task.input.slice(0, 30)}
                </p>
              </button>
            ))}
          </div>
        )}

        {tasks.length === 0 && (
          <div className="p-4 text-center text-muted-foreground">
            <p className="text-sm">No tasks yet</p>
            <p className="text-xs mt-1">Start a conversation to begin</p>
          </div>
        )}
      </div>
    </div>
  );
}
