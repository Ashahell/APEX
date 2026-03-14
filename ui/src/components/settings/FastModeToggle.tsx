import { useState, useEffect } from 'react';
import { getSessionFastMode, setSessionFastMode, FastModeState } from '../../lib/api';

interface FastModeToggleProps {
  sessionId: string;
}

export function FastModeToggle({ sessionId }: FastModeToggleProps) {
  const [fastMode, setFastMode] = useState<FastModeState | null>(null);
  const [loading, setLoading] = useState(true);
  const [toggling, setToggling] = useState(false);

  useEffect(() => {
    loadFastMode();
  }, [sessionId]);

  const loadFastMode = async () => {
    try {
      const data = await getSessionFastMode(sessionId);
      setFastMode(data);
    } catch (err) {
      console.error('Failed to load fast mode:', err);
      // Set default if not found
      setFastMode({
        session_id: sessionId,
        fast_enabled: false,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = async () => {
    if (!fastMode || toggling) return;
    
    setToggling(true);
    try {
      const newState = await setSessionFastMode(
        sessionId,
        !fastMode.fast_enabled,
        fastMode.fast_model,
        fastMode.fast_config,
        fastMode.toggles
      );
      setFastMode(newState);
    } catch (err) {
      console.error('Failed to toggle fast mode:', err);
    } finally {
      setToggling(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center gap-2">
        <div className="w-8 h-5 bg-gray-700 rounded-full animate-pulse" />
        <span className="text-sm text-gray-500">Loading...</span>
      </div>
    );
  }

  return (
    <button
      onClick={handleToggle}
      disabled={toggling}
      className={`flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200 ${
        fastMode?.fast_enabled
          ? 'bg-yellow-500/20 text-yellow-400 border border-yellow-500/50 hover:bg-yellow-500/30'
          : 'bg-gray-700/50 text-gray-400 border border-gray-600 hover:bg-gray-700'
      } ${toggling ? 'opacity-50 cursor-wait' : ''}`}
      title={fastMode?.fast_enabled ? 'Disable Fast Mode' : 'Enable Fast Mode'}
    >
      <span className={fastMode?.fast_enabled ? 'animate-pulse' : ''}>⚡</span>
      <span>Fast</span>
      {fastMode?.fast_enabled && fastMode.fast_model && (
        <span className="text-xs text-yellow-400/70 ml-1">
          {fastMode.fast_model}
        </span>
      )}
    </button>
  );
}

// Fast Mode Settings Panel
export function FastModeSettings({ sessionId }: FastModeToggleProps) {
  const [fastMode, setFastMode] = useState<FastModeState | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadFastMode();
  }, [sessionId]);

  const loadFastMode = async () => {
    try {
      const data = await getSessionFastMode(sessionId);
      setFastMode(data);
    } catch (err) {
      console.error('Failed to load fast mode:', err);
      setFastMode({
        session_id: sessionId,
        fast_enabled: false,
      });
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    if (!fastMode || saving) return;
    
    setSaving(true);
    try {
      const newState = await setSessionFastMode(
        sessionId,
        fastMode.fast_enabled,
        fastMode.fast_model,
        fastMode.fast_config,
        fastMode.toggles
      );
      setFastMode(newState);
    } catch (err) {
      console.error('Failed to save fast mode:', err);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="p-4 flex items-center justify-center">
        <div className="w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-sm font-medium text-white">Fast Mode</h3>
          <p className="text-xs text-gray-500">
            Use faster, smaller models for quick responses
          </p>
        </div>
        <FastModeToggle sessionId={sessionId} />
      </div>

      {fastMode?.fast_enabled && (
        <>
          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Fast Model
            </label>
            <input
              type="text"
              value={fastMode.fast_model || ''}
              onChange={(e) => setFastMode({ ...fastMode, fast_model: e.target.value })}
              placeholder="e.g., qwen2.5:0.5b"
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded-md text-white text-sm focus:outline-none focus:border-indigo-500"
            />
          </div>

          <div className="flex items-center gap-4">
            <label className="flex items-center gap-2 text-sm text-gray-400">
              <input
                type="checkbox"
                checked={fastMode.toggles?.thinking ?? false}
                onChange={(e) => setFastMode({
                  ...fastMode,
                  toggles: { ...fastMode.toggles, thinking: e.target.checked }
                })}
                className="w-4 h-4 rounded bg-gray-700 border-gray-600 text-indigo-500 focus:ring-indigo-500"
              />
              Disable Thinking
            </label>
            <label className="flex items-center gap-2 text-sm text-gray-400">
              <input
                type="checkbox"
                checked={fastMode.toggles?.verbose ?? false}
                onChange={(e) => setFastMode({
                  ...fastMode,
                  toggles: { ...fastMode.toggles, verbose: e.target.checked }
                })}
                className="w-4 h-4 rounded bg-gray-700 border-gray-600 text-indigo-500 focus:ring-indigo-500"
              />
              Verbose Output
            </label>
          </div>
        </>
      )}

      <button
        onClick={handleSave}
        disabled={saving}
        className="w-full py-2 bg-indigo-600 hover:bg-indigo-700 disabled:bg-indigo-600/50 text-white rounded-md text-sm font-medium transition-colors"
      >
        {saving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  );
}
