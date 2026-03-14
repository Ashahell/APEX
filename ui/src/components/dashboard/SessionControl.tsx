import { useState, useEffect } from 'react';
import {
  yieldSession,
  getSessionYields,
  resumeSession,
  getResumeHistory,
  listCheckpoints,
  createCheckpoint,
  deleteCheckpoint,
  listAttachments,
  deleteAttachment,
  SessionYieldLog,
  SessionResumeHistory,
  SessionCheckpoint,
  SessionAttachment,
} from '../../lib/api';

interface SessionControlProps {
  sessionId: string;
  onYield?: (childSessionId: string) => void;
  onResume?: (sessionId: string) => void;
}

type TabType = 'control' | 'checkpoints' | 'attachments' | 'history';

export function SessionControl({ sessionId, onYield, onResume }: SessionControlProps) {
  const [activeTab, setActiveTab] = useState<TabType>('control');
  const [yields, setYields] = useState<SessionYieldLog[]>([]);
  const [resumeHistory, setResumeHistory] = useState<SessionResumeHistory[]>([]);
  const [checkpoints, setCheckpoints] = useState<SessionCheckpoint[]>([]);
  const [attachments, setAttachments] = useState<SessionAttachment[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showNewCheckpoint, setShowNewCheckpoint] = useState(false);
  const [newCheckpointName, setNewCheckpointName] = useState('');

  useEffect(() => {
    loadData();
  }, [sessionId, activeTab]);

  const loadData = async () => {
    setLoading(true);
    setError(null);
    try {
      switch (activeTab) {
        case 'control':
          const [yieldsData, resumeData] = await Promise.all([
            getSessionYields(sessionId),
            getResumeHistory(sessionId),
          ]);
          setYields(yieldsData);
          setResumeHistory(resumeData);
          break;
        case 'checkpoints':
          const checkpointsData = await listCheckpoints(sessionId);
          setCheckpoints(checkpointsData);
          break;
        case 'attachments':
          const attachmentsData = await listAttachments(sessionId);
          setAttachments(attachmentsData);
          break;
        case 'history':
          const [y, r] = await Promise.all([
            getSessionYields(sessionId),
            getResumeHistory(sessionId),
          ]);
          setYields(y);
          setResumeHistory(r);
          break;
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handleYield = async (skipToolWork: boolean = false) => {
    setLoading(true);
    setError(null);
    try {
      const result = await yieldSession(sessionId, {
        skipToolWork,
        reason: skipToolWork ? 'Skipping remaining tool work' : 'Yielding session',
      });
      onYield?.(result.child_session_id);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to yield session');
    } finally {
      setLoading(false);
    }
  };

  const handleResume = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await resumeSession({
        originalSessionId: sessionId,
        resumeType: 'manual',
        contextSummary: 'Resumed from UI',
      });
      onResume?.(result.session_id);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to resume session');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateCheckpoint = async () => {
    if (!newCheckpointName.trim()) return;
    setLoading(true);
    setError(null);
    try {
      await createCheckpoint(sessionId, newCheckpointName, JSON.stringify({}), 'Manual checkpoint');
      setNewCheckpointName('');
      setShowNewCheckpoint(false);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create checkpoint');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteCheckpoint = async (checkpointId: string) => {
    setLoading(true);
    setError(null);
    try {
      await deleteCheckpoint(sessionId, checkpointId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete checkpoint');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteAttachment = async (attachmentId: string) => {
    setLoading(true);
    setError(null);
    try {
      await deleteAttachment(sessionId, attachmentId);
      await loadData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete attachment');
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleString();
  };

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const tabs: { id: TabType; label: string }[] = [
    { id: 'control', label: 'Control' },
    { id: 'checkpoints', label: 'Checkpoints' },
    { id: 'attachments', label: 'Attachments' },
    { id: 'history', label: 'History' },
  ];

  return (
    <div className="bg-gray-900/80 border border-gray-700/50 rounded-lg overflow-hidden">
      {/* Tabs */}
      <div className="flex border-b border-gray-700/50">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? 'text-indigo-400 border-b-2 border-indigo-500 bg-gray-800/50'
                : 'text-gray-400 hover:text-gray-200 hover:bg-gray-800/30'
            }`}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="p-4">
        {error && (
          <div className="mb-4 p-3 bg-red-900/30 border border-red-700/50 rounded text-red-400 text-sm">
            {error}
          </div>
        )}

        {loading && (
          <div className="flex items-center justify-center py-8">
            <div className="w-5 h-5 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin" />
          </div>
        )}

        {!loading && activeTab === 'control' && (
          <div className="space-y-4">
            {/* Yield Buttons */}
            <div className="space-y-2">
              <h4 className="text-sm font-medium text-gray-300">Session Control</h4>
              <div className="flex gap-2">
                <button
                  onClick={() => handleYield(false)}
                  disabled={loading}
                  className="px-3 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white text-sm rounded transition-colors"
                >
                  ⏸️ Yield
                </button>
                <button
                  onClick={() => handleYield(true)}
                  disabled={loading}
                  className="px-3 py-2 bg-yellow-600 hover:bg-yellow-700 disabled:opacity-50 text-white text-sm rounded transition-colors"
                >
                  ⏭️ Skip Remaining
                </button>
                <button
                  onClick={handleResume}
                  disabled={loading}
                  className="px-3 py-2 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white text-sm rounded transition-colors"
                >
                  ▶️ Resume
                </button>
              </div>
            </div>

            {/* Quick Stats */}
            <div className="grid grid-cols-2 gap-4 pt-4 border-t border-gray-700/50">
              <div>
                <div className="text-xs text-gray-500">Total Yields</div>
                <div className="text-lg text-indigo-400 font-semibold">{yields.length}</div>
              </div>
              <div>
                <div className="text-xs text-gray-500">Total Resumes</div>
                <div className="text-lg text-green-400 font-semibold">{resumeHistory.length}</div>
              </div>
            </div>
          </div>
        )}

        {!loading && activeTab === 'checkpoints' && (
          <div className="space-y-4">
            {/* Create Checkpoint */}
            <div className="flex items-center justify-between">
              <h4 className="text-sm font-medium text-gray-300">Checkpoints</h4>
              <button
                onClick={() => setShowNewCheckpoint(!showNewCheckpoint)}
                className="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-700 text-white rounded transition-colors"
              >
                + New
              </button>
            </div>

            {showNewCheckpoint && (
              <div className="flex gap-2">
                <input
                  type="text"
                  value={newCheckpointName}
                  onChange={(e) => setNewCheckpointName(e.target.value)}
                  placeholder="Checkpoint name..."
                  className="flex-1 px-3 py-2 bg-gray-800 border border-gray-600 rounded text-white text-sm focus:outline-none focus:border-indigo-500"
                />
                <button
                  onClick={handleCreateCheckpoint}
                  disabled={loading || !newCheckpointName.trim()}
                  className="px-3 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50 text-white text-sm rounded transition-colors"
                >
                  Save
                </button>
              </div>
            )}

            {/* Checkpoint List */}
            {checkpoints.length === 0 ? (
              <p className="text-gray-500 text-sm text-center py-4">No checkpoints yet</p>
            ) : (
              <ul className="space-y-2">
                {checkpoints.map((cp) => (
                  <li
                    key={cp.id}
                    className="flex items-center justify-between p-3 bg-gray-800/50 rounded border border-gray-700/50"
                  >
                    <div>
                      <div className="text-sm text-white font-medium">{cp.checkpoint_name}</div>
                      <div className="text-xs text-gray-500">{formatDate(cp.created_at)}</div>
                      {cp.description && (
                        <div className="text-xs text-gray-400 mt-1">{cp.description}</div>
                      )}
                    </div>
                    <button
                      onClick={() => handleDeleteCheckpoint(cp.id)}
                      className="p-1 text-gray-500 hover:text-red-400 transition-colors"
                      title="Delete checkpoint"
                    >
                      🗑️
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {!loading && activeTab === 'attachments' && (
          <div className="space-y-4">
            <h4 className="text-sm font-medium text-gray-300">Session Attachments</h4>

            {attachments.length === 0 ? (
              <p className="text-gray-500 text-sm text-center py-4">No attachments yet</p>
            ) : (
              <ul className="space-y-2">
                {attachments.map((att) => (
                  <li
                    key={att.id}
                    className="flex items-center justify-between p-3 bg-gray-800/50 rounded border border-gray-700/50"
                  >
                    <div className="flex items-center gap-3">
                      <span className="text-lg">📎</span>
                      <div>
                        <div className="text-sm text-white">{att.file_name}</div>
                        <div className="text-xs text-gray-500">
                          {formatFileSize(att.file_size)} • {att.file_type}
                        </div>
                      </div>
                    </div>
                    <button
                      onClick={() => handleDeleteAttachment(att.id)}
                      className="p-1 text-gray-500 hover:text-red-400 transition-colors"
                      title="Delete attachment"
                    >
                      🗑️
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}

        {!loading && activeTab === 'history' && (
          <div className="space-y-4">
            {/* Yields */}
            <div>
              <h4 className="text-sm font-medium text-gray-300 mb-2">Yield History</h4>
              {yields.length === 0 ? (
                <p className="text-gray-500 text-sm">No yield history</p>
              ) : (
                <ul className="space-y-2">
                  {yields.map((y) => (
                    <li key={y.id} className="p-3 bg-gray-800/30 rounded border border-gray-700/30">
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-indigo-400">Yielded to:</span>
                        <span className="text-xs text-gray-500 font-mono">{y.child_session_id.slice(0, 8)}...</span>
                      </div>
                      {y.reason && (
                        <div className="text-xs text-gray-400 mt-1">{y.reason}</div>
                      )}
                      <div className="text-xs text-gray-500 mt-1">{formatDate(y.created_at)}</div>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            {/* Resumes */}
            <div>
              <h4 className="text-sm font-medium text-gray-300 mb-2">Resume History</h4>
              {resumeHistory.length === 0 ? (
                <p className="text-gray-500 text-sm">No resume history</p>
              ) : (
                <ul className="space-y-2">
                  {resumeHistory.map((r) => (
                    <li key={r.id} className="p-3 bg-gray-800/30 rounded border border-gray-700/30">
                      <div className="flex items-center justify-between">
                        <span className="text-sm text-green-400">Resumed from:</span>
                        <span className="text-xs text-gray-500 font-mono">{r.resumed_from.slice(0, 8)}...</span>
                      </div>
                      <div className="text-xs text-gray-400 mt-1">Type: {r.resume_type}</div>
                      {r.context_summary && (
                        <div className="text-xs text-gray-500 mt-1">{r.context_summary}</div>
                      )}
                      <div className="text-xs text-gray-500 mt-1">{formatDate(r.created_at)}</div>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
