import { useState, useEffect } from 'react';
import { listSessions, SessionMetadata } from '../../lib/dashboard';
import { SessionControl } from './SessionControl';

export function SessionManager() {
  const [sessions, setSessions] = useState<SessionMetadata[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedSession, setSelectedSession] = useState<string | null>(null);
  const [showControl, setShowControl] = useState(false);

  useEffect(() => {
    loadSessions();
  }, []);

  const loadSessions = async () => {
    try {
      const data = await listSessions();
      setSessions(data);
    } catch (err) {
      console.error('Failed to load sessions:', err);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center">
        <div className="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  if (sessions.length === 0) {
    return (
      <div className="p-4 text-center text-gray-500 text-sm">
        No active sessions
      </div>
    );
  }

  // When a session is selected, show the control panel
  if (selectedSession && showControl) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center justify-between p-2 border-b border-gray-700/50">
          <button
            onClick={() => { setSelectedSession(null); setShowControl(false); }}
            className="text-sm text-gray-400 hover:text-white"
          >
            ← Back
          </button>
          <button
            onClick={() => setShowControl(false)}
            className="text-gray-500 hover:text-gray-300"
          >
            ×
          </button>
        </div>
        <div className="flex-1 overflow-y-auto">
          <SessionControl 
            sessionId={selectedSession}
            onYield={(childId) => console.log('Yielded to:', childId)}
            onResume={(sessionId) => console.log('Resumed:', sessionId)}
          />
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <ul className="flex-1 overflow-y-auto">
        {sessions.map((session) => (
          <li key={session.session_id} className="border-b border-gray-700/50 last:border-0">
            <button
              onClick={() => { setSelectedSession(session.session_id); setShowControl(true); }}
              className={`w-full p-3 text-left hover:bg-gray-800/50 transition-colors ${
                selectedSession === session.session_id ? 'bg-indigo-600/20' : ''
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-sm text-white font-mono truncate">
                  {session.session_id.slice(0, 8)}...
                </span>
                {session.fast_mode && (
                  <span className="text-xs text-yellow-400" title="Fast Mode">
                    ⚡
                  </span>
                )}
              </div>
              <div className="flex items-center gap-2 mt-1">
                {session.model && (
                  <span className="text-xs text-gray-400 truncate">{session.model}</span>
                )}
              </div>
              <div className="flex items-center gap-2 mt-1">
                <span className="text-xs text-gray-500">
                  {session.thinking_level}
                </span>
                <span className="text-gray-600">•</span>
                <span className="text-xs text-gray-500">
                  {session.send_policy}
                </span>
              </div>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
