import { useEffect, useCallback } from 'react';

type KeyboardShortcut = {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  action: () => void;
  description?: string;
};

export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[], enabled = true) {
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    if (!enabled) return;
    
    const target = event.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      if (event.key !== 'Escape') return;
    }

    for (const shortcut of shortcuts) {
      const ctrlMatch = shortcut.ctrl ? (event.ctrlKey || event.metaKey) : true;
      const metaMatch = shortcut.meta ? event.metaKey : true;
      const shiftMatch = shortcut.shift ? event.shiftKey : !event.shiftKey;
      
      if (event.key.toLowerCase() === shortcut.key.toLowerCase() && ctrlMatch && metaMatch && shiftMatch) {
        event.preventDefault();
        shortcut.action();
        break;
      }
    }
  }, [shortcuts, enabled]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}

export const DEFAULT_SHORTCUTS = [
  { key: 'k', ctrl: true, description: 'Open command palette' },
  { key: '/', ctrl: true, description: 'Focus message input' },
  { key: '1', ctrl: true, description: 'Go to Chat' },
  { key: '2', ctrl: true, description: 'Go to Skills' },
  { key: '3', ctrl: true, description: 'Go to Memory' },
  { key: '4', ctrl: true, description: 'Go to Board' },
  { key: '5', ctrl: true, description: 'Go to Settings' },
  { key: 'Escape', description: 'Close modal/panel' },
  { key: 'b', ctrl: true, description: 'Toggle sidebar' },
];
