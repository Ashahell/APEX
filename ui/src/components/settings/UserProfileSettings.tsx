import { useState, useEffect } from 'react';
import { apiGet, apiPut } from '../../lib/api';

// Types matching the Rust backend
interface UserProfile {
  communication_style: 'formal' | 'casual' | 'technical' | 'concise';
  verbosity: 'brief' | 'moderate' | 'detailed' | 'comprehensive';
  preferred_categories: string[];
  preferred_tools: string[];
  response_format: 'plain' | 'markdown' | 'structured';
  include_reasoning: boolean;
  language: string;
  timezone: string;
}

const COMMUNICATION_STYLES = [
  { value: 'casual', label: 'Casual', description: 'Conversational and friendly' },
  { value: 'formal', label: 'Formal', description: 'Professional and structured' },
  { value: 'technical', label: 'Technical', description: 'Detailed with code examples' },
  { value: 'concise', label: 'Concise', description: 'Brief and to the point' },
];

const VERBOSITY_LEVELS = [
  { value: 'brief', label: 'Brief', description: 'Short answers, minimal explanation' },
  { value: 'moderate', label: 'Moderate', description: 'Balanced detail level' },
  { value: 'detailed', label: 'Detailed', description: 'Thorough explanations' },
  { value: 'comprehensive', label: 'Comprehensive', description: 'Maximum detail with context' },
];

const RESPONSE_FORMATS = [
  { value: 'plain', label: 'Plain Text', description: 'Simple text responses' },
  { value: 'markdown', label: 'Markdown', description: 'Formatted with markdown' },
  { value: 'structured', label: 'Structured', description: 'JSON or structured output' },
];

export function UserProfileSettings() {
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    loadProfile();
  }, []);

  const loadProfile = async () => {
    try {
      const res = await apiGet('/api/v1/user/profile');
      if (res.ok) {
        const data = await res.json();
        setProfile(data);
      } else {
        setError('Failed to load profile');
      }
    } catch (e) {
      setError('Failed to load profile');
    } finally {
      setLoading(false);
    }
  };

  const updateProfile = async (updates: Partial<UserProfile>) => {
    if (!profile) return;
    
    setSaving(true);
    setSuccess(false);
    setError(null);
    
    try {
      const res = await apiPut('/api/v1/user/profile', updates);
      if (res.ok) {
        const data = await res.json();
        setProfile(data);
        setSuccess(true);
        setTimeout(() => setSuccess(false), 3000);
      } else {
        setError('Failed to save profile');
      }
    } catch {
      setError('Failed to save profile');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-muted-foreground">Loading profile...</div>
      </div>
    );
  }

  if (!profile) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-destructive">Failed to load profile</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-4">User Profile Settings</h3>
        <p className="text-sm text-muted-foreground mb-6">
          Customize how the agent interacts with you based on your preferences.
        </p>
      </div>

      {error && (
        <div className="bg-destructive/10 border border-destructive/20 rounded-lg p-4 text-sm text-destructive">
          {error}
        </div>
      )}

      {success && (
        <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-4 text-sm text-green-600">
          Profile saved successfully!
        </div>
      )}

      {/* Communication Style */}
      <div className="space-y-3">
        <label className="text-sm font-medium">Communication Style</label>
        <div className="grid grid-cols-2 gap-3">
          {COMMUNICATION_STYLES.map((style) => (
            <button
              key={style.value}
              onClick={() => updateProfile({ communication_style: style.value as any })}
              disabled={saving}
              className={`p-4 rounded-lg border text-left transition-colors ${
                profile.communication_style === style.value
                  ? 'border-primary bg-primary/10'
                  : 'border-border hover:border-primary/50'
              }`}
            >
              <div className="font-medium">{style.label}</div>
              <div className="text-xs text-muted-foreground mt-1">{style.description}</div>
            </button>
          ))}
        </div>
      </div>

      {/* Verbosity */}
      <div className="space-y-3">
        <label className="text-sm font-medium">Response Verbosity</label>
        <div className="grid grid-cols-2 gap-3">
          {VERBOSITY_LEVELS.map((level) => (
            <button
              key={level.value}
              onClick={() => updateProfile({ verbosity: level.value as any })}
              disabled={saving}
              className={`p-4 rounded-lg border text-left transition-colors ${
                profile.verbosity === level.value
                  ? 'border-primary bg-primary/10'
                  : 'border-border hover:border-primary/50'
              }`}
            >
              <div className="font-medium">{level.label}</div>
              <div className="text-xs text-muted-foreground mt-1">{level.description}</div>
            </button>
          ))}
        </div>
      </div>

      {/* Response Format */}
      <div className="space-y-3">
        <label className="text-sm font-medium">Response Format</label>
        <div className="grid grid-cols-3 gap-3">
          {RESPONSE_FORMATS.map((format) => (
            <button
              key={format.value}
              onClick={() => updateProfile({ response_format: format.value as any })}
              disabled={saving}
              className={`p-3 rounded-lg border text-center transition-colors ${
                profile.response_format === format.value
                  ? 'border-primary bg-primary/10'
                  : 'border-border hover:border-primary/50'
              }`}
            >
              <div className="font-medium text-sm">{format.label}</div>
              <div className="text-xs text-muted-foreground mt-1">{format.description}</div>
            </button>
          ))}
        </div>
      </div>

      {/* Include Reasoning */}
      <div className="flex items-center justify-between p-4 rounded-lg border border-border">
        <div>
          <div className="font-medium">Include Reasoning</div>
          <div className="text-xs text-muted-foreground mt-1">
            Show the agent's reasoning process in responses
          </div>
        </div>
        <button
          onClick={() => updateProfile({ include_reasoning: !profile.include_reasoning })}
          disabled={saving}
          className={`relative w-12 h-6 rounded-full transition-colors ${
            profile.include_reasoning ? 'bg-primary' : 'bg-muted'
          }`}
        >
          <div
            className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${
              profile.include_reasoning ? 'translate-x-7' : 'translate-x-1'
            }`}
          />
        </button>
      </div>

      {/* Language */}
      <div className="space-y-2">
        <label className="text-sm font-medium">Language</label>
        <select
          value={profile.language}
          onChange={(e) => updateProfile({ language: e.target.value })}
          disabled={saving}
          className="w-full p-3 rounded-lg border border-border bg-background"
        >
          <option value="en">English</option>
          <option value="es">Spanish</option>
          <option value="fr">French</option>
          <option value="de">German</option>
          <option value="zh">Chinese</option>
          <option value="ja">Japanese</option>
          <option value="pt">Portuguese</option>
          <option value="ru">Russian</option>
        </select>
      </div>

      {/* Preferred Tools */}
      {profile.preferred_tools.length > 0 && (
        <div className="space-y-2">
          <label className="text-sm font-medium">Most Used Tools</label>
          <div className="flex flex-wrap gap-2">
            {profile.preferred_tools.map((tool) => (
              <span
                key={tool}
                className="px-3 py-1 rounded-full bg-primary/10 text-primary text-sm"
              >
                {tool}
              </span>
            ))}
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            These are your most frequently used tools, learned from your interactions.
          </p>
        </div>
      )}

      {/* Preferred Categories */}
      {profile.preferred_categories.length > 0 && (
        <div className="space-y-2">
          <label className="text-sm font-medium">Most Used Categories</label>
          <div className="flex flex-wrap gap-2">
            {profile.preferred_categories.map((cat) => (
              <span
                key={cat}
                className="px-3 py-1 rounded-full bg-secondary text-secondary-foreground text-sm"
              >
                {cat}
              </span>
            ))}
          </div>
          <p className="text-xs text-muted-foreground mt-2">
            Task categories you interact with most often.
          </p>
        </div>
      )}
    </div>
  );
}
