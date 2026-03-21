import { useState, useEffect, useCallback } from 'react';
import { useAppStore } from '../../stores/appStore';
import {
  getBoundedMemoryStats,
  getMemoryEntries,
  getUserEntries,
  addMemoryEntry,
  addUserEntry,
  removeMemoryEntry,
  removeUserEntry,
  BoundedMemoryStats,
  BoundedMemoryEntry,
} from '../../lib/api';

interface UsageBarProps {
  used: number;
  limit: number;
  label: string;
  isWarning?: boolean;
  isCritical?: boolean;
}

function UsageBar({ used, limit, label, isWarning, isCritical }: UsageBarProps) {
  const percent = limit > 0 ? Math.min((used / limit) * 100, 100) : 0;
  
  let barColor = 'bg-emerald-500';
  if (isCritical) {
    barColor = 'bg-red-500';
  } else if (isWarning) {
    barColor = 'bg-amber-500';
  }
  
  return (
    <div className="space-y-2">
      <div className="flex justify-between items-center text-sm">
        <span className="font-medium text-[var(--color-text)]">{label}</span>
        <span className="text-[var(--color-text-muted)]">
          {used.toLocaleString()} / {limit.toLocaleString()} chars ({percent.toFixed(0)}%)
        </span>
      </div>
      <div className="h-3 bg-[var(--color-panel)] rounded-full overflow-hidden">
        <div
          className={`h-full ${barColor} transition-all duration-300 rounded-full`}
          style={{ width: `${percent}%` }}
        />
      </div>
    </div>
  );
}

interface MemoryEntryProps {
  entry: BoundedMemoryEntry;
  onDelete: (id: string) => void;
  canEdit: boolean;
}

function MemoryEntryCard({ entry, onDelete, canEdit }: MemoryEntryProps) {
  const [showDelete, setShowDelete] = useState(false);
  
  const formatDate = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };
  
  return (
    <div
      className="p-4 bg-[var(--color-panel)] rounded-lg border border-[var(--color-border)] hover:border-[var(--color-primary)] transition-colors group"
      onMouseEnter={() => setShowDelete(true)}
      onMouseLeave={() => setShowDelete(false)}
    >
      <div className="flex justify-between items-start gap-3">
        <p className="text-[var(--color-text)] text-sm flex-1 whitespace-pre-wrap break-words">
          {entry.content}
        </p>
        {canEdit && showDelete && (
          <button
            onClick={() => onDelete(entry.content.substring(0, 50))}
            className="shrink-0 p-1 text-red-400 hover:text-red-300 hover:bg-red-500/20 rounded transition-colors"
            title="Remove entry"
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </button>
        )}
      </div>
      <div className="mt-2 text-xs text-[var(--color-text-muted)]">
        {formatDate(entry.created_at)}
      </div>
    </div>
  );
}

interface AddEntryFormProps {
  storeType: 'memory' | 'user';
  onSubmit: (content: string) => Promise<void>;
  remaining: number;
}

function AddEntryForm({ storeType, onSubmit, remaining: _remaining }: AddEntryFormProps) {
  const [content, setContent] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const maxLength = 500;
  const minLength = 10;
  
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (content.length < minLength || content.length > maxLength) {
      setError(`Content must be between ${minLength} and ${maxLength} characters`);
      return;
    }
    
    setIsSubmitting(true);
    setError(null);
    
    try {
      await onSubmit(content);
      setContent('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add entry');
    } finally {
      setIsSubmitting(false);
    }
  };
  
  return (
    <form onSubmit={handleSubmit} className="space-y-3">
      <textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        placeholder={storeType === 'memory' 
          ? "Add a memory (e.g., 'User prefers dark mode in VS Code')"
          : "Add a user preference (e.g., 'User name: Alex')"
        }
        className="w-full p-3 bg-[var(--color-panel)] border border-[var(--color-border)] rounded-lg text-[var(--color-text)] text-sm resize-none focus:outline-none focus:border-[var(--color-primary)]"
        rows={3}
        maxLength={maxLength + 100}
      />
      {error && (
        <p className="text-red-400 text-sm">{error}</p>
      )}
      <div className="flex justify-between items-center">
        <span className={`text-xs ${content.length < minLength ? 'text-amber-400' : 'text-[var(--color-text-muted)]'}`}>
          {content.length} / {maxLength} chars (min: {minLength})
        </span>
        <button
          type="submit"
          disabled={isSubmitting || content.length < minLength || content.length > maxLength}
          className="px-4 py-2 bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] disabled:bg-[var(--color-panel)] disabled:text-[var(--color-text-muted)] text-white rounded-lg text-sm font-medium transition-colors"
        >
          {isSubmitting ? 'Adding...' : 'Add Entry'}
        </button>
      </div>
    </form>
  );
}

export function BoundedMemory() {
  const { addToast } = useAppStore();
  const [activeTab, setActiveTab] = useState<'memory' | 'user'>('memory');
  const [stats, setStats] = useState<BoundedMemoryStats | null>(null);
  const [memoryEntries, setMemoryEntries] = useState<BoundedMemoryEntry[]>([]);
  const [userEntries, setUserEntries] = useState<BoundedMemoryEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  
  const loadData = useCallback(async () => {
    try {
      const [statsData, memoryData, userData] = await Promise.all([
        getBoundedMemoryStats(),
        getMemoryEntries(),
        getUserEntries(),
      ]);
      setStats(statsData);
      setMemoryEntries(memoryData.entries);
      setUserEntries(userData.entries);
    } catch (err) {
      addToast({
        type: 'error',
        message: `Failed to load memory: ${err instanceof Error ? err.message : 'Unknown error'}`,
      });
    } finally {
      setIsLoading(false);
    }
  }, [addToast]);
  
  useEffect(() => {
    loadData();
  }, [loadData]);
  
  const handleAddMemory = async (content: string) => {
    await addMemoryEntry(content);
    await loadData();
    addToast({ type: 'success', message: 'Memory entry added' });
  };
  
  const handleAddUser = async (content: string) => {
    await addUserEntry(content);
    await loadData();
    addToast({ type: 'success', message: 'User preference added' });
  };
  
  const handleDeleteMemory = async (oldText: string) => {
    try {
      await removeMemoryEntry(oldText);
      await loadData();
      addToast({ type: 'success', message: 'Memory entry removed' });
    } catch (err) {
      addToast({
        type: 'error',
        message: `Failed to remove: ${err instanceof Error ? err.message : 'Unknown error'}`,
      });
    }
  };
  
  const handleDeleteUser = async (oldText: string) => {
    try {
      await removeUserEntry(oldText);
      await loadData();
      addToast({ type: 'success', message: 'User preference removed' });
    } catch (err) {
      addToast({
        type: 'error',
        message: `Failed to remove: ${err instanceof Error ? err.message : 'Unknown error'}`,
      });
    }
  };
  
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin w-8 h-8 border-2 border-[var(--color-primary)] border-t-transparent rounded-full" />
      </div>
    );
  }
  
  const currentEntries = activeTab === 'memory' ? memoryEntries : userEntries;
  const currentStats = activeTab === 'memory' ? stats?.memory : stats?.user;
  const remaining = currentStats ? currentStats.char_limit - currentStats.used_chars : 0;
  
  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-xl font-bold text-[var(--color-text)]">Bounded Memory</h2>
        <p className="text-sm text-[var(--color-text-muted)] mt-1">
          Hermes-style curated memory with character limits
        </p>
      </div>
      
      {/* Usage Bars */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="p-4 bg-[var(--color-panel)] rounded-lg border border-[var(--color-border)]">
        <UsageBar
          used={stats?.memory.used_chars || 0}
          limit={stats?.memory.char_limit || 2200}
          label="MEMORY.md"
          isWarning={stats?.memory.is_warning}
          isCritical={stats?.memory.is_critical}
        />
          <p className="mt-2 text-xs text-[var(--color-text-muted)]">
            {stats?.memory.entry_count || 0} entries
          </p>
        </div>
        <div className="p-4 bg-[var(--color-panel)] rounded-lg border border-[var(--color-border)]">
        <UsageBar
          used={stats?.user.used_chars || 0}
          limit={stats?.user.char_limit || 1375}
          label="USER.md"
          isWarning={stats?.user.is_warning}
          isCritical={stats?.user.is_critical}
        />
          <p className="mt-2 text-xs text-[var(--color-text-muted)]">
            {stats?.user.entry_count || 0} entries
          </p>
        </div>
      </div>
      
      {/* Tabs */}
      <div className="flex gap-2 border-b border-[var(--color-border)]">
        <button
          onClick={() => setActiveTab('memory')}
          className={`px-4 py-2 text-sm font-medium transition-colors ${
            activeTab === 'memory'
              ? 'text-[var(--color-primary)] border-b-2 border-[var(--color-primary)]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          Memory ({memoryEntries.length})
        </button>
        <button
          onClick={() => setActiveTab('user')}
          className={`px-4 py-2 text-sm font-medium transition-colors ${
            activeTab === 'user'
              ? 'text-[var(--color-primary)] border-b-2 border-[var(--color-primary)]'
              : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
          }`}
        >
          User Profile ({userEntries.length})
        </button>
      </div>
      
      {/* Add Entry Form */}
      <div className="p-4 bg-[var(--color-panel)] rounded-lg border border-[var(--color-border)]">
        <h3 className="text-sm font-medium text-[var(--color-text)] mb-3">
          Add New Entry
        </h3>
        {remaining < 100 ? (
          <div className="p-3 bg-amber-500/20 border border-amber-500/30 rounded-lg text-amber-400 text-sm">
            ⚠️ Memory is almost full ({remaining} chars remaining). Remove some entries first.
          </div>
        ) : (
          <AddEntryForm
            storeType={activeTab}
            onSubmit={activeTab === 'memory' ? handleAddMemory : handleAddUser}
            remaining={remaining}
          />
        )}
      </div>
      
      {/* Entry List */}
      <div className="space-y-3">
        <h3 className="text-sm font-medium text-[var(--color-text)]">
          {activeTab === 'memory' ? 'Memory Entries' : 'User Preferences'}
        </h3>
        {currentEntries.length === 0 ? (
          <div className="p-8 text-center text-[var(--color-text-muted)] bg-[var(--color-panel)] rounded-lg border border-[var(--color-border)]">
            <p>No entries yet.</p>
            <p className="text-sm mt-1">Add your first entry above.</p>
          </div>
        ) : (
          <div className="space-y-2">
            {currentEntries.map((entry) => (
              <MemoryEntryCard
                key={entry.id}
                entry={entry}
                onDelete={activeTab === 'memory' ? handleDeleteMemory : handleDeleteUser}
                canEdit={true}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export default BoundedMemory;
