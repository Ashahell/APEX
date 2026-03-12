import { useState, useEffect, useMemo, useRef } from 'react';

interface Command {
  id: string;
  label: string;
  category: 'navigation' | 'action' | 'task' | 'settings';
  shortcut?: string;
  action: () => void;
}

interface QuickCommandBarProps {
  onNavigate?: (tab: string) => void;
  onRunTask?: (content: string) => void;
  onOpenSettings?: () => void;
}

export function QuickCommandBar({ onNavigate, onRunTask, onOpenSettings }: QuickCommandBarProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  const commands: Command[] = useMemo(() => [
    { id: 'nav-chat', label: 'Go to Chat', category: 'navigation', shortcut: 'Ctrl+1', action: () => onNavigate?.('chat') },
    { id: 'nav-skills', label: 'Go to Skills', category: 'navigation', shortcut: 'Ctrl+2', action: () => onNavigate?.('skills') },
    { id: 'nav-memory', label: 'Go to Memory', category: 'navigation', action: () => onNavigate?.('memory') },
    { id: 'nav-kanban', label: 'Go to Board', category: 'navigation', shortcut: 'Ctrl+8', action: () => onNavigate?.('kanban') },
    { id: 'nav-workflows', label: 'Go to Workflows', category: 'navigation', action: () => onNavigate?.('workflows') },
    { id: 'nav-journal', label: 'Go to Journal', category: 'navigation', action: () => onNavigate?.('journal') },
    { id: 'nav-settings', label: 'Go to Settings', category: 'navigation', shortcut: 'Ctrl+0', action: () => onNavigate?.('settings') },
    { id: 'action-new-task', label: 'Create New Task', category: 'action', shortcut: 'Ctrl+N', action: () => onNavigate?.('chat') },
    { id: 'action-kill', label: 'Toggle Theme', category: 'action', action: () => {} },
    { id: 'settings-config', label: 'Open Config Settings', category: 'settings', action: () => onOpenSettings?.() },
    { id: 'settings-metrics', label: 'View Metrics', category: 'settings', action: () => onNavigate?.('metrics') },
    { id: 'settings-health', label: 'Check System Health', category: 'settings', action: () => onNavigate?.('health') },
  ], [onNavigate, onOpenSettings]);

  const filteredCommands = useMemo(() => {
    if (!query.trim()) return commands;
    const lowerQuery = query.toLowerCase();
    return commands.filter(cmd => 
      cmd.label.toLowerCase().includes(lowerQuery) ||
      cmd.category.toLowerCase().includes(lowerQuery)
    );
  }, [commands, query]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'p') {
        e.preventDefault();
        setIsOpen(true);
      }
      if (e.key === 'Escape' && isOpen) {
        setIsOpen(false);
        setQuery('');
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen]);

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(i => Math.min(i + 1, filteredCommands.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(i => Math.max(i - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          filteredCommands[selectedIndex].action();
          setIsOpen(false);
          setQuery('');
        }
        break;
    }
  };

  const handleTaskSubmit = () => {
    if (query.trim().startsWith('>')) {
      const taskContent = query.slice(1).trim();
      if (taskContent && onRunTask) {
        onRunTask(taskContent);
        setIsOpen(false);
        setQuery('');
      }
    } else if (query.trim()) {
      const matched = filteredCommands[selectedIndex];
      if (matched) {
        matched.action();
        setIsOpen(false);
        setQuery('');
      }
    }
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="flex items-center gap-2 px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:bg-[#4248f1]/10 rounded-lg transition-colors"
        title="Quick Command (Ctrl+P)"
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <span className="hidden md:inline text-xs border border-border rounded-lg px-1">Ctrl+P</span>
      </button>
    );
  }

  const groupedCommands = filteredCommands.reduce((acc, cmd) => {
    if (!acc[cmd.category]) acc[cmd.category] = [];
    acc[cmd.category].push(cmd);
    return acc;
  }, {} as Record<string, Command[]>);

  let globalIndex = 0;

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24">
      <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={() => { setIsOpen(false); setQuery(''); }} />
      
      <div className="relative w-full max-w-xl mx-4 bg-[var(--color-panel)] rounded-2xl shadow-2xl border border-border overflow-hidden">
        <div className="flex items-center gap-3 p-4 border-b border-border">
          <svg className="w-5 h-5 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            ref={inputRef}
            type="text"
            placeholder="Type a command, {'>'} to run task, or search..."
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            onKeyDownCapture={(e) => {
              if (e.key === 'Enter') {
                handleTaskSubmit();
              }
            }}
            className="flex-1 bg-transparent outline-none text-sm placeholder:text-[var(--color-text-muted)]"
            autoFocus
          />
          <button
            onClick={() => { setIsOpen(false); setQuery(''); }}
            className="p-1.5 hover:bg-[#4248f1]/20 rounded-lg transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="max-h-96 overflow-y-auto p-2">
          {query.trim().startsWith('>') ? (
            <div className="p-3">
              <div className="text-xs text-[var(--color-text-muted)] mb-2">Run as task</div>
              <button
                onClick={handleTaskSubmit}
                className="w-full text-left p-3 bg-[#4248f1]/10 hover:bg-[#4248f1]/20 rounded-xl transition-colors"
              >
                <span className="font-medium">{query.slice(1).trim() || 'Empty task'}</span>
                <span className="block text-xs text-muted-foreground mt-1">Press Enter to execute</span>
              </button>
            </div>
          ) : filteredCommands.length === 0 ? (
            <div className="p-4 text-center text-muted-foreground">
              No commands found
            </div>
          ) : (
            <div className="space-y-4">
              {Object.entries(groupedCommands).map(([category, cmds]) => (
                <div key={category}>
                  <div className="text-xs font-medium text-muted-foreground uppercase px-3 mb-2">
                    {category}
                  </div>
                  {cmds.map((cmd) => {
                    const currentIndex = globalIndex++;
                    return (
                      <button
                        key={cmd.id}
                        onClick={() => { cmd.action(); setIsOpen(false); setQuery(''); }}
                        className={`w-full flex items-center justify-between p-3 rounded-lg transition-colors text-left ${
                          currentIndex === selectedIndex ? 'bg-primary/10' : 'hover:bg-muted'
                        }`}
                      >
                        <span className="font-medium">{cmd.label}</span>
                        {cmd.shortcut && (
                          <span className="text-xs text-muted-foreground border rounded px-1.5 py-0.5">
                            {cmd.shortcut}
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="p-3 border-t bg-muted/30 text-xs text-muted-foreground flex justify-between">
          <span>↑↓ Navigate • Enter Select • {'>'} Run Task</span>
          <span>Ctrl+P to open • ESC to close</span>
        </div>
      </div>
    </div>
  );
}
