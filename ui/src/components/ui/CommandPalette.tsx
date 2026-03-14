import { useState, useEffect } from 'react';
import { listCommands, recordCommand } from '../../lib/dashboard';

interface Command {
  id: string;
  label: string;
  description: string;
  action: () => void;
  category: 'action' | 'skill' | 'navigation' | 'search';
}

const DEFAULT_COMMANDS: Command[] = [
  { id: 'new-task', label: 'New Task', description: 'Create a new task', action: () => {}, category: 'action' },
  { id: 'settings', label: 'Settings', description: 'Open settings', action: () => {}, category: 'navigation' },
  { id: 'skills', label: 'Skills', description: 'Browse skills', action: () => {}, category: 'navigation' },
  { id: 'memory', label: 'Memory', description: 'View memory', action: () => {}, category: 'navigation' },
  { id: 'search', label: 'Search', description: 'Search messages', action: () => {}, category: 'search' },
];

interface CommandPaletteProps {
  onClose: () => void;
}

export function CommandPalette({ onClose }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);

  // Load recent commands on mount
  useEffect(() => {
    listCommands(undefined, 5).then((cmds) => {
      console.log('Recent commands:', cmds);
    });
  }, []);

  const filteredCommands = query
    ? DEFAULT_COMMANDS.filter(
        (cmd) =>
          cmd.label.toLowerCase().includes(query.toLowerCase()) ||
          cmd.description.toLowerCase().includes(query.toLowerCase())
      )
    : DEFAULT_COMMANDS;

  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, filteredCommands.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex]) {
          executeCommand(filteredCommands[selectedIndex]);
        }
        break;
      case 'Escape':
        onClose();
        break;
    }
  };

  const executeCommand = async (cmd: Command) => {
    try {
      await recordCommand(cmd.label, cmd.category);
    } catch (err) {
      console.error('Failed to record command:', err);
    }
    cmd.action();
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/50" onClick={onClose}>
      <div
        className="w-full max-w-xl bg-[#1a1a2e] border border-gray-700 rounded-lg shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input */}
        <div className="p-4 border-b border-gray-700">
          <input
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Type a command..."
            autoFocus
            className="w-full px-4 py-3 text-lg bg-transparent text-white placeholder-gray-500 focus:outline-none"
          />
        </div>

        {/* Command List */}
        <div className="max-h-80 overflow-y-auto">
          {filteredCommands.length === 0 ? (
            <div className="p-4 text-center text-gray-500">No commands found</div>
          ) : (
            <ul className="py-2">
              {filteredCommands.map((cmd, index) => (
                <li key={cmd.id}>
                  <button
                    onClick={() => executeCommand(cmd)}
                    className={`w-full px-4 py-3 flex items-center justify-between text-left transition-colors ${
                      index === selectedIndex ? 'bg-indigo-600/30' : 'hover:bg-gray-800'
                    }`}
                  >
                    <div>
                      <div className="text-white font-medium">{cmd.label}</div>
                      <div className="text-sm text-gray-400">{cmd.description}</div>
                    </div>
                    <span className="text-xs text-gray-500 capitalize">{cmd.category}</span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* Footer */}
        <div className="px-4 py-2 border-t border-gray-700 bg-[#16162a] flex items-center justify-between text-xs text-gray-500">
          <div className="flex items-center gap-4">
            <span>↑↓ Navigate</span>
            <span>↵ Select</span>
            <span>Esc Close</span>
          </div>
        </div>
      </div>
    </div>
  );
}
