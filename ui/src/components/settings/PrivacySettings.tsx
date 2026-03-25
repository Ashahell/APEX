import { useState, useEffect } from 'react';

// Privacy settings - stores in localStorage for now, API integration pending
const CLOUD_PROVIDERS = [
  { id: 'openai', name: 'OpenAI', description: 'GPT models' },
  { id: 'anthropic', name: 'Anthropic', description: 'Claude models' },
  { id: 'google', name: 'Google', description: 'Gemini models' },
  { id: 'cohere', name: 'Cohere', description: 'Command models' },
  { id: 'fireworks', name: 'Fireworks', description: 'Fireworks AI' },
  { id: 'azure', name: 'Azure OpenAI', description: 'Microsoft Azure' },
  { id: 'aws_bedrock', name: 'AWS Bedrock', description: 'Amazon Web Services' },
  { id: 'huggingface', name: 'Hugging Face', description: 'Inference Endpoints' },
];

interface PrivacyState {
  enabled: boolean;
  blockedProviders: string[];
  auditLogEnabled: boolean;
}

export function PrivacySettings() {
  const [privacyEnabled, setPrivacyEnabled] = useState(false);
  const [blockedProviders, setBlockedProviders] = useState<string[]>([]);
  const [auditEnabled, setAuditEnabled] = useState(true);
  const [showWarning, setShowWarning] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

  // Load settings on mount
  useEffect(() => {
    loadPrivacySettings();
  }, []);

  const loadPrivacySettings = async () => {
    setIsLoading(true);
    try {
      // Try to load from API first
      const res = await fetch('/api/v1/privacy/status', {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      
      if (res.ok) {
        const data = await res.json();
        setPrivacyEnabled(data.enabled);
        setBlockedProviders(data.blocked_providers || []);
        setAuditEnabled(data.audit_log_enabled ?? true);
      } else {
        // Fallback to localStorage
        const saved = localStorage.getItem('apex-privacy-settings');
        if (saved) {
          const parsed: PrivacyState = JSON.parse(saved);
          setPrivacyEnabled(parsed.enabled);
          setBlockedProviders(parsed.blockedProviders);
          setAuditEnabled(parsed.auditLogEnabled);
        } else {
          // Default: block all cloud providers
          setBlockedProviders(CLOUD_PROVIDERS.map(p => p.id));
        }
      }
    } catch (err) {
      console.warn('Failed to load privacy settings, using defaults:', err);
      setBlockedProviders(CLOUD_PROVIDERS.map(p => p.id));
    } finally {
      setIsLoading(false);
    }
  };

  const savePrivacySettings = async (settings: PrivacyState) => {
    setIsSaving(true);
    try {
      // Try API first
      await fetch('/api/v1/privacy/config', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
        body: JSON.stringify(settings),
      });
    } catch (err) {
      console.warn('Failed to save to API, saving locally:', err);
    }
    
    // Always save to localStorage as backup
    localStorage.setItem('apex-privacy-settings', JSON.stringify(settings));
    setIsSaving(false);
  };

  const handleTogglePrivacy = async () => {
    const newEnabled = !privacyEnabled;
    setPrivacyEnabled(newEnabled);
    if (newEnabled) {
      setShowWarning(true);
    }
    await savePrivacySettings({
      enabled: newEnabled,
      blockedProviders,
      auditLogEnabled: auditEnabled,
    });
  };

  const handleProviderToggle = async (providerId: string) => {
    const newBlocked = blockedProviders.includes(providerId)
      ? blockedProviders.filter(p => p !== providerId)
      : [...blockedProviders, providerId];
    
    setBlockedProviders(newBlocked);
    await savePrivacySettings({
      enabled: privacyEnabled,
      blockedProviders: newBlocked,
      auditLogEnabled: auditEnabled,
    });
  };

  const handleBlockAll = async () => {
    setBlockedProviders(CLOUD_PROVIDERS.map(p => p.id));
    await savePrivacySettings({
      enabled: privacyEnabled,
      blockedProviders: CLOUD_PROVIDERS.map(p => p.id),
      auditLogEnabled: auditEnabled,
    });
  };

  const handleAllowAll = async () => {
    setBlockedProviders([]);
    await savePrivacySettings({
      enabled: privacyEnabled,
      blockedProviders: [],
      auditLogEnabled: auditEnabled,
    });
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            Privacy Mode
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Block cloud connections and use local models only
          </p>
        </div>
        
        {/* Main Toggle */}
        <button
          onClick={handleTogglePrivacy}
          disabled={isSaving}
          className={`
            relative inline-flex h-8 w-14 items-center rounded-full transition-colors
            focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2
            ${privacyEnabled ? 'bg-indigo-600' : 'bg-gray-200 dark:bg-gray-700'}
            ${isSaving ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
          `}
        >
          <span
            className={`
              inline-block h-6 w-6 transform rounded-full bg-white shadow-lg transition-transform
              ${privacyEnabled ? 'translate-x-7' : 'translate-x-1'}
            `}
          />
        </button>
      </div>

      {/* Warning Banner */}
      {showWarning && privacyEnabled && (
        <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700 rounded-lg p-4">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg className="h-5 w-5 text-amber-400" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
              </svg>
            </div>
            <div className="ml-3">
              <h4 className="text-sm font-medium text-amber-800 dark:text-amber-200">
                Privacy Mode Enabled
              </h4>
              <p className="mt-1 text-sm text-amber-700 dark:text-amber-300">
                Only local models will be used. Cloud LLM providers will be blocked.
              </p>
            </div>
            <button
              onClick={() => setShowWarning(false)}
              className="ml-auto text-amber-400 hover:text-amber-500"
            >
              <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
              </svg>
            </button>
          </div>
        </div>
      )}

      {/* Provider List */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Cloud Providers
          </h4>
          <div className="flex gap-2">
            <button
              onClick={handleBlockAll}
              className="text-xs px-2 py-1 text-red-600 hover:text-red-700 dark:text-red-400"
            >
              Block All
            </button>
            <span className="text-gray-300">|</span>
            <button
              onClick={handleAllowAll}
              className="text-xs px-2 py-1 text-green-600 hover:text-green-700 dark:text-green-400"
            >
              Allow All
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {CLOUD_PROVIDERS.map((provider) => {
            const isBlocked = blockedProviders.includes(provider.id);
            return (
              <div
                key={provider.id}
                className={`
                  flex items-center justify-between p-3 rounded-lg border
                  ${isBlocked 
                    ? 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800' 
                    : 'bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700'}
                `}
              >
                <div className="flex items-center gap-3">
                  <div className={`
                    w-3 h-3 rounded-full
                    ${isBlocked ? 'bg-red-500' : 'bg-green-500'}
                  `} />
                  <div>
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {provider.name}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400">
                      {provider.description}
                    </p>
                  </div>
                </div>
                <button
                  onClick={() => handleProviderToggle(provider.id)}
                  className={`
                    text-xs px-3 py-1 rounded-full font-medium transition-colors
                    ${isBlocked 
                      ? 'bg-red-100 text-red-700 dark:bg-red-900/50 dark:text-red-300 hover:bg-red-200' 
                      : 'bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300 hover:bg-green-200'}
                  `}
                >
                  {isBlocked ? 'Blocked' : 'Allowed'}
                </button>
              </div>
            );
          })}
        </div>
      </div>

      {/* Audit Log Toggle */}
      <div className="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-700">
        <div>
          <h4 className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Audit Log
          </h4>
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
            Record blocked connection attempts
          </p>
        </div>
        <button
          onClick={() => {
            setAuditEnabled(!auditEnabled);
            savePrivacySettings({
              enabled: privacyEnabled,
              blockedProviders,
              auditLogEnabled: !auditEnabled,
            });
          }}
          className={`
            relative inline-flex h-6 w-10 items-center rounded-full transition-colors
            ${auditEnabled ? 'bg-indigo-600' : 'bg-gray-200 dark:bg-gray-700'}
          `}
        >
          <span
            className={`
              inline-block h-4 w-4 transform rounded-full bg-white transition-transform
              ${auditEnabled ? 'translate-x-5' : 'translate-x-1'}
            `}
          />
        </button>
      </div>

      {/* Local-only notice */}
      {privacyEnabled && (
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
          <div className="flex items-start gap-3">
            <svg className="h-5 w-5 text-blue-400 mt-0.5" viewBox="0 0 20 20" fill="currentColor">
              <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clipRule="evenodd" />
            </svg>
            <div>
              <h4 className="text-sm font-medium text-blue-800 dark:text-blue-200">
                Local Models Only
              </h4>
              <p className="mt-1 text-sm text-blue-700 dark:text-blue-300">
                When privacy mode is enabled, APEX will only use local models like Qwen3-4B 
                running on llama-server. No data will be sent to cloud providers.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
