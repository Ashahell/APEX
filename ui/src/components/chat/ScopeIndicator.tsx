import { useState, useEffect } from 'react';

type ScopeType = 'global' | 'session' | 'channel';

interface ScopeData {
  scope: ScopeType;
  label: string;
  description: string;
}

interface ScopeIndicatorProps {
  onScopeChange?: (scope: ScopeType) => void;
  compact?: boolean;
}

const SCOPE_INFO: Record<ScopeType, ScopeData> = {
  global: {
    scope: 'global',
    label: 'Global',
    description: 'Data accessible across all sessions and channels',
  },
  session: {
    scope: 'session',
    label: 'Session',
    description: 'Data available only in the current session',
  },
  channel: {
    scope: 'channel',
    label: 'Channel',
    description: 'Data restricted to the current channel',
  },
};

export function ScopeIndicator({ onScopeChange, compact = false }: ScopeIndicatorProps) {
  const [currentScope, setCurrentScope] = useState<ScopeType>('global');
  const [showDropdown, setShowDropdown] = useState(false);

  useEffect(() => {
    // Load current scope from localStorage or context
    const saved = localStorage.getItem('apex-context-scope');
    if (saved && ['global', 'session', 'channel'].includes(saved)) {
      setCurrentScope(saved as ScopeType);
    }
  }, []);

  const handleScopeChange = (newScope: ScopeType) => {
    setCurrentScope(newScope);
    localStorage.setItem('apex-context-scope', newScope);
    onScopeChange?.(newScope);
    setShowDropdown(false);
  };

  const getScopeIcon = (scope: ScopeType) => {
    switch (scope) {
      case 'global':
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
      case 'session':
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
          </svg>
        );
      case 'channel':
        return (
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
          </svg>
        );
    }
  };

  const getScopeColor = (scope: ScopeType) => {
    switch (scope) {
      case 'global':
        return 'text-blue-500 bg-blue-100 dark:bg-blue-900/30';
      case 'session':
        return 'text-green-500 bg-green-100 dark:bg-green-900/30';
      case 'channel':
        return 'text-purple-500 bg-purple-100 dark:bg-purple-900/30';
    }
  };

  const info = SCOPE_INFO[currentScope];

  if (compact) {
    return (
      <button
        onClick={() => setShowDropdown(!showDropdown)}
        className={`inline-flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium ${getScopeColor(currentScope)}`}
      >
        {getScopeIcon(currentScope)}
        <span>{info.label}</span>
        <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>

        {showDropdown && (
          <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50">
            {(Object.keys(SCOPE_INFO) as ScopeType[]).map((scope) => {
              const scopeInfo = SCOPE_INFO[scope];
              return (
                <button
                  key={scope}
                  onClick={() => handleScopeChange(scope)}
                  className={`w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-700 ${
                    currentScope === scope ? 'bg-gray-50 dark:bg-gray-700' : ''
                  }`}
                >
                  <span className={getScopeColor(scope)}>{getScopeIcon(scope)}</span>
                  <span className="text-sm text-gray-700 dark:text-gray-300">
                    {scopeInfo.label}
                  </span>
                </button>
              );
            })}
          </div>
        )}
      </button>
    );
  }

  return (
    <div className="relative">
      <button
        onClick={() => setShowDropdown(!showDropdown)}
        className={`flex items-center gap-2 px-3 py-2 rounded-lg border transition-colors ${
          showDropdown
            ? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20'
            : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
        }`}
      >
        <span className={`p-1.5 rounded ${getScopeColor(currentScope)}`}>
          {getScopeIcon(currentScope)}
        </span>
        <div className="text-left">
          <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
            {info.label} Scope
          </p>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            {info.description}
          </p>
        </div>
        <svg className="w-4 h-4 text-gray-400 ml-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {showDropdown && (
        <>
          <div
            className="fixed inset-0 z-40"
            onClick={() => setShowDropdown(false)}
          />
          <div className="absolute left-0 mt-2 w-72 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50">
            <div className="p-3 border-b border-gray-200 dark:border-gray-700">
              <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100">
                Context Scope
              </h3>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                Choose which contexts can access your data
              </p>
            </div>
            <div className="p-2">
              {(Object.keys(SCOPE_INFO) as ScopeType[]).map((scope) => {
                const scopeInfo = SCOPE_INFO[scope];
                const isActive = currentScope === scope;
                return (
                  <button
                    key={scope}
                    onClick={() => handleScopeChange(scope)}
                    className={`w-full flex items-start gap-3 p-3 rounded-lg text-left transition-colors ${
                      isActive
                        ? 'bg-indigo-50 dark:bg-indigo-900/20 border border-indigo-200 dark:border-indigo-800'
                        : 'hover:bg-gray-50 dark:hover:bg-gray-700'
                    }`}
                  >
                    <span className={`p-2 rounded-lg ${getScopeColor(scope)}`}>
                      {getScopeIcon(scope)}
                    </span>
                    <div className="flex-1">
                      <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                        {scopeInfo.label}
                        {isActive && (
                          <span className="ml-2 text-xs text-indigo-600 dark:text-indigo-400">
                            Active
                          </span>
                        )}
                      </p>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        {scopeInfo.description}
                      </p>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
