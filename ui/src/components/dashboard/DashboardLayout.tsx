import { useState, useEffect, useCallback } from 'react';
import { useAppStore } from '../../stores/appStore';
import { Chat } from '../chat/Chat';
import { CommandPalette } from '../ui/CommandPalette';
import { PinnedMessages } from './PinnedMessages';
import { SessionManager } from './SessionManager';

export function DashboardLayout() {
  const [showCommandPalette, setShowCommandPalette] = useState(false);
  const [showPinned, setShowPinned] = useState(true);
  const [showSessions, setShowSessions] = useState(true);
  const { tasks } = useAppStore();
  
  const runningTasks = tasks.filter(t => t.status === 'running').length;

  // Keyboard shortcut for command palette
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault();
      setShowCommandPalette(true);
    }
  }, []);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  return (
    <div className="flex h-full bg-[#0f0f1a]">
      {/* Main Content Area */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Dashboard Toolbar */}
        <div className="flex items-center justify-between px-4 py-2 border-b border-gray-700 bg-[#1a1a2e]">
          <div className="flex items-center gap-4">
            <h1 className="text-lg font-semibold text-white">Dashboard</h1>
            <span className="text-sm text-gray-400">
              {runningTasks > 0 ? `${runningTasks} running` : 'Idle'}
            </span>
          </div>
          
          <div className="flex items-center gap-2">
            <SearchBar />
            <button
              onClick={() => setShowCommandPalette(true)}
              className="px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 text-gray-200 rounded-md flex items-center gap-2 transition-colors"
            >
              <span>⌘</span>
              <span>K</span>
            </button>
          </div>
        </div>

        {/* Chat View */}
        <div className="flex-1 overflow-hidden">
          <Chat />
        </div>
      </div>

      {/* Sidebar */}
      {showPinned || showSessions ? (
        <div className="w-72 border-l border-gray-700 bg-[#16162a] flex flex-col">
          {/* Pinned Messages */}
          {showPinned && (
            <div className="flex-1 overflow-hidden border-b border-gray-700">
              <div className="px-3 py-2 border-b border-gray-700 flex items-center justify-between">
                <h2 className="text-sm font-medium text-gray-300">Pinned</h2>
                <button
                  onClick={() => setShowPinned(false)}
                  className="text-gray-500 hover:text-gray-300"
                >
                  ×
                </button>
              </div>
              <PinnedMessages />
            </div>
          )}

          {/* Session Manager */}
          {showSessions && (
            <div className="flex-1 overflow-hidden">
              <div className="px-3 py-2 border-b border-gray-700 flex items-center justify-between">
                <h2 className="text-sm font-medium text-gray-300">Sessions</h2>
                <button
                  onClick={() => setShowSessions(false)}
                  className="text-gray-500 hover:text-gray-300"
                >
                  ×
                </button>
              </div>
              <SessionManager />
            </div>
          )}

          {/* Toggle Buttons when collapsed */}
          {!showPinned && (
            <button
              onClick={() => { setShowPinned(true); setShowSessions(true); }}
              className="p-2 text-sm text-gray-400 hover:text-white border-t border-gray-700"
            >
              Show Sidebar
            </button>
          )}
        </div>
      ) : (
        <button
          onClick={() => { setShowPinned(true); setShowSessions(true); }}
          className="w-10 border-l border-gray-700 bg-[#16162a] flex flex-col items-center py-2 gap-2"
        >
          <span className="text-gray-500">📌</span>
          <span className="text-gray-500">💬</span>
        </button>
      )}

      {/* Command Palette Modal */}
      {showCommandPalette && (
        <CommandPalette onClose={() => setShowCommandPalette(false)} />
      )}
    </div>
  );
}

// Search Bar Component
function SearchBar() {
  const [query, setQuery] = useState('');
  const [isSearching, setIsSearching] = useState(false);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;
    
    setIsSearching(true);
    // Search will be handled by the dashboard API
    try {
      const { searchMessages } = await import('../../lib/dashboard');
      const results = await searchMessages(query);
      console.log('Search results:', results);
      // TODO: Display results in a dropdown or modal
    } catch (err) {
      console.error('Search failed:', err);
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <form onSubmit={handleSearch} className="relative">
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search messages..."
        className="w-64 px-3 py-1.5 text-sm bg-gray-800 border border-gray-600 rounded-md text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
      />
      {isSearching && (
        <div className="absolute right-3 top-1/2 -translate-y-1/2">
          <div className="w-4 h-4 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
        </div>
      )}
    </form>
  );
}
