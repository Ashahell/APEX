import { useEffect, useState } from 'react';
import { apiFetch } from '../../lib/api';

interface AutoCreatedSkill {
  name: string;
  description: string;
  version: string;
  category: string;
  platforms: string[];
  created_at: number;
  trigger_conditions: string[];
  auto_created: boolean;
  source_task_id?: string;
}

interface SkillSuggestion {
  task_id: string;
  skill_name: string;
  task_content: string;
  skill_content: string;
  tools_used: string[];
  suggested_at: string;
}

type TabType = 'skills' | 'suggestions';

export function AutoCreatedSkills() {
  const [skills, setSkills] = useState<AutoCreatedSkill[]>([]);
  const [suggestions, setSuggestions] = useState<SkillSuggestion[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedSkill, setSelectedSkill] = useState<AutoCreatedSkill | null>(null);
  const [skillContent, setSkillContent] = useState<string | null>(null);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newSkill, setNewSkill] = useState({ name: '', description: '', content: '' });
  const [creating, setCreating] = useState(false);
  const [activeTab, setActiveTab] = useState<TabType>('skills');

  useEffect(() => {
    loadSkills();
    loadSuggestions();
  }, []);

  const loadSkills = async () => {
    try {
      const res = await apiFetch('/api/v1/skills/auto-created');
      const data = await res.json();
      setSkills(Array.isArray(data) ? data : []);
      setError(null);
    } catch (err: unknown) {
      console.error('Failed to load skills:', err);
    }
  };

  const loadSuggestions = async () => {
    try {
      setLoading(true);
      const res = await apiFetch('/api/v1/skills/suggestions');
      const data = await res.json();
      setSuggestions(Array.isArray(data) ? data : []);
    } catch (err: unknown) {
      console.error('Failed to load suggestions:', err);
    } finally {
      setLoading(false);
    }
  };

  const loadSkillContent = async (name: string) => {
    try {
      const res = await apiFetch(`/api/v1/skills/auto-created/${name}/content`);
      const data = await res.json();
      setSkillContent(data.content || null);
    } catch {
      setSkillContent(null);
    }
  };

  const handleSelectSkill = async (skill: AutoCreatedSkill) => {
    setSelectedSkill(skill);
    await loadSkillContent(skill.name);
  };

  const handleDeleteSkill = async (name: string) => {
    if (!confirm(`Are you sure you want to delete "${name}"?`)) return;
    
    try {
      await apiFetch(`/api/v1/skills/auto-created/${name}`, { method: 'DELETE' });
      await loadSkills();
      if (selectedSkill?.name === name) {
        setSelectedSkill(null);
        setSkillContent(null);
      }
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      alert(`Failed to delete skill: ${message}`);
    }
  };

  const handleCreateSkillFromSuggestion = async (suggestion: SkillSuggestion) => {
    const name = suggestion.skill_name.toLowerCase().replace(/\s+/g, '-');
    
    try {
      await apiFetch('/api/v1/skills/auto-created', {
        method: 'POST',
        body: JSON.stringify({
          name,
          content: suggestion.skill_content,
          description: suggestion.task_content.substring(0, 100),
          source_task_id: suggestion.task_id,
        }),
      });
      
      await apiFetch(`/api/v1/skills/suggestions/${suggestion.task_id}`, { method: 'DELETE' });
      
      await loadSkills();
      await loadSuggestions();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      alert(`Failed to create skill: ${message}`);
    }
  };

  const handleDeleteSuggestion = async (taskId: string) => {
    if (!confirm('Dismiss this suggestion?')) return;
    
    try {
      await apiFetch(`/api/v1/skills/suggestions/${taskId}`, { method: 'DELETE' });
      await loadSuggestions();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      alert(`Failed to delete suggestion: ${message}`);
    }
  };

  const handleCreateSkill = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newSkill.name || !newSkill.content) return;
    
    setCreating(true);
    try {
      await apiFetch('/api/v1/skills/auto-created', {
        method: 'POST',
        body: JSON.stringify({
          name: newSkill.name.toLowerCase().replace(/\s+/g, '-'),
          content: newSkill.content,
          description: newSkill.description || undefined,
        }),
      });
      setShowCreateForm(false);
      setNewSkill({ name: '', description: '', content: '' });
      await loadSkills();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Unknown error';
      alert(`Failed to create skill: ${message}`);
    } finally {
      setCreating(false);
    }
  };

  const filteredSkills = skills.filter((skill) =>
    skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    skill.description.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const formatDate = (timestamp: number) => {
    if (!timestamp) return 'Unknown';
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  const formatSuggestionDate = (dateStr: string) => {
    try {
      return new Date(dateStr).toLocaleString();
    } catch {
      return dateStr;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading...</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Tab Header */}
      <div className="border-b border-[var(--color-border)] p-4">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-xl font-semibold text-[var(--color-text)]">Agent Skills</h2>
          <button
            onClick={() => setShowCreateForm(true)}
            className="p-1.5 rounded-lg bg-[#4248f1] text-white hover:bg-[#4248f1]/90 transition-colors"
            title="Create new skill"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="12" y1="5" x2="12" y2="19"></line>
              <line x1="5" y1="12" x2="19" y2="12"></line>
            </svg>
          </button>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setActiveTab('skills')}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
              activeTab === 'skills'
                ? 'bg-[#4248f1] text-white'
                : 'text-[var(--color-text-muted)] hover:bg-[var(--color-muted)]'
            }`}
          >
            Skills ({skills.length})
          </button>
          <button
            onClick={() => setActiveTab('suggestions')}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors flex items-center gap-2 ${
              activeTab === 'suggestions'
                ? 'bg-[#4248f1] text-white'
                : 'text-[var(--color-text-muted)] hover:bg-[var(--color-muted)]'
            }`}
          >
            Suggestions
            {suggestions.length > 0 && (
              <span className="px-1.5 py-0.5 rounded-full bg-amber-500 text-white text-xs">
                {suggestions.length}
              </span>
            )}
          </button>
        </div>
      </div>

      {activeTab === 'skills' ? (
        <div className="flex-1 flex overflow-hidden">
          {/* Skills List */}
          <div className="w-80 border-r border-[var(--color-border)] flex flex-col">
            <div className="p-4 border-b border-[var(--color-border)]">
              <div className="relative">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
                  <circle cx="11" cy="11" r="8"></circle>
                  <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
                </svg>
                <input
                  type="text"
                  placeholder="Search skills..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full pl-9 pr-3 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] text-sm focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                />
              </div>
            </div>

            <div className="flex-1 overflow-y-auto">
              {filteredSkills.length === 0 ? (
                <div className="p-4 text-center text-[var(--color-text-muted)] text-sm">
                  {skills.length === 0 ? (
                    <>
                      <p>No auto-created skills yet.</p>
                      <p className="text-xs mt-1">Skills are created by the agent after complex tasks.</p>
                    </>
                  ) : (
                    <p>No skills match your search.</p>
                  )}
                </div>
              ) : (
                <div className="p-2">
                  {filteredSkills.map((skill) => (
                    <button
                      key={skill.name}
                      onClick={() => handleSelectSkill(skill)}
                      className={`w-full text-left p-3 rounded-lg mb-1 transition-colors ${
                        selectedSkill?.name === skill.name
                          ? 'bg-[#4248f1]/10 border border-[#4248f1]/30'
                          : 'hover:bg-[var(--color-muted)] border border-transparent'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-medium text-[var(--color-text)]">{skill.name}</span>
                        <span className="text-xs text-[var(--color-text-muted)]">{skill.version}</span>
                      </div>
                      <p className="text-xs text-[var(--color-text-muted)] mt-1 line-clamp-2">
                        {skill.description || 'No description'}
                      </p>
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Skill Detail */}
          <div className="flex-1 flex flex-col">
            {selectedSkill ? (
              <>
                <div className="p-4 border-b border-[var(--color-border)] flex items-center justify-between">
                  <div>
                    <h3 className="text-lg font-semibold text-[var(--color-text)]">{selectedSkill.name}</h3>
                    <div className="flex items-center gap-3 mt-1">
                      <span className="text-xs px-2 py-0.5 rounded bg-[#4248f1]/10 text-[#4248f1]">
                        {selectedSkill.category}
                      </span>
                      <span className="text-xs text-[var(--color-text-muted)]">
                        v{selectedSkill.version}
                      </span>
                      <span className="text-xs text-[var(--color-text-muted)]">
                        {formatDate(selectedSkill.created_at)}
                      </span>
                    </div>
                  </div>
                  <button
                    onClick={() => handleDeleteSkill(selectedSkill.name)}
                    className="p-2 rounded-lg text-red-500 hover:bg-red-500/10 transition-colors"
                    title="Delete skill"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <polyline points="3 6 5 6 21 6"></polyline>
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    </svg>
                  </button>
                </div>

                <div className="flex-1 overflow-y-auto p-4">
                  <div className="mb-4">
                    <h4 className="text-sm font-medium text-[var(--color-text-muted)] mb-1">Description</h4>
                    <p className="text-[var(--color-text)]">{selectedSkill.description || 'No description'}</p>
                  </div>

                  {selectedSkill.source_task_id && (
                    <div className="mb-4">
                      <h4 className="text-sm font-medium text-[var(--color-text-muted)] mb-1">Source Task</h4>
                      <p className="text-[var(--color-text)] font-mono text-sm">{selectedSkill.source_task_id}</p>
                    </div>
                  )}

                  <div className="mb-4">
                    <h4 className="text-sm font-medium text-[var(--color-text-muted)] mb-1">Platforms</h4>
                    <div className="flex gap-2">
                      {selectedSkill.platforms.map((platform) => (
                        <span key={platform} className="text-xs px-2 py-1 rounded bg-[var(--color-muted)] text-[var(--color-text)]">
                          {platform}
                        </span>
                      ))}
                    </div>
                  </div>

                  <div>
                    <h4 className="text-sm font-medium text-[var(--color-text-muted)] mb-1">Content</h4>
                    <pre className="text-sm bg-[var(--color-muted)] rounded-lg p-4 overflow-x-auto whitespace-pre-wrap font-mono">
                      {skillContent || 'Loading...'}
                    </pre>
                  </div>
                </div>
              </>
            ) : (
              <div className="flex-1 flex items-center justify-center text-[var(--color-text-muted)]">
                <div className="text-center">
                  <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="mx-auto opacity-50">
                    <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
                    <polyline points="2 17 12 22 22 17"></polyline>
                    <polyline points="2 12 12 17 22 12"></polyline>
                  </svg>
                  <p className="mt-2">Select a skill to view details</p>
                </div>
              </div>
            )}
          </div>
        </div>
      ) : (
        /* Suggestions Tab */
        <div className="flex-1 overflow-y-auto p-4">
          {suggestions.length === 0 ? (
            <div className="text-center py-12 text-[var(--color-text-muted)]">
              <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="mx-auto opacity-50">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="8" x2="12" y2="12"></line>
                <line x1="12" y1="16" x2="12.01" y2="16"></line>
              </svg>
              <p className="mt-4">No skill suggestions yet.</p>
              <p className="text-sm mt-1">Complete complex tasks with 5+ tool calls to generate suggestions.</p>
            </div>
          ) : (
            <div className="space-y-4">
              {suggestions.map((suggestion) => (
                <div key={suggestion.task_id} className="border border-[var(--color-border)] rounded-lg p-4 bg-[var(--color-panel)]">
                  <div className="flex items-start justify-between mb-3">
                    <div>
                      <h4 className="font-medium text-[var(--color-text)]">{suggestion.skill_name}</h4>
                      <p className="text-xs text-[var(--color-text-muted)] mt-1">
                        Suggested {formatSuggestionDate(suggestion.suggested_at)}
                      </p>
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={() => handleCreateSkillFromSuggestion(suggestion)}
                        className="px-3 py-1.5 rounded-lg bg-[#4248f1] text-white text-sm hover:bg-[#4248f1]/90 transition-colors"
                      >
                        Create Skill
                      </button>
                      <button
                        onClick={() => handleDeleteSuggestion(suggestion.task_id)}
                        className="px-3 py-1.5 rounded-lg border border-[var(--color-border)] text-[var(--color-text-muted)] text-sm hover:bg-[var(--color-muted)] transition-colors"
                      >
                        Dismiss
                      </button>
                    </div>
                  </div>
                  
                  <div className="mb-3">
                    <p className="text-sm font-medium text-[var(--color-text-muted)] mb-1">Task:</p>
                    <p className="text-sm text-[var(--color-text)] line-clamp-2">{suggestion.task_content}</p>
                  </div>
                  
                  <div className="mb-3">
                    <p className="text-sm font-medium text-[var(--color-text-muted)] mb-1">Tools Used:</p>
                    <div className="flex flex-wrap gap-1">
                      {suggestion.tools_used.map((tool, i) => (
                        <span key={i} className="text-xs px-2 py-0.5 rounded bg-[var(--color-muted)] text-[var(--color-text)]">
                          {tool}
                        </span>
                      ))}
                    </div>
                  </div>
                  
                  <details className="group">
                    <summary className="text-sm text-[#4248f1] cursor-pointer hover:underline">
                      View Generated Content
                    </summary>
                    <pre className="text-xs bg-[var(--color-muted)] rounded-lg p-3 mt-2 overflow-x-auto whitespace-pre-wrap font-mono">
                      {suggestion.skill_content}
                    </pre>
                  </details>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Create Skill Modal */}
      {showCreateForm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-[var(--color-panel)] rounded-xl w-full max-w-lg mx-4 shadow-xl">
            <div className="p-4 border-b border-[var(--color-border)] flex items-center justify-between">
              <h3 className="font-semibold text-[var(--color-text)]">Create Auto-Created Skill</h3>
              <button onClick={() => setShowCreateForm(false)} className="text-[var(--color-text-muted)] hover:text-[var(--color-text)]">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
            <form onSubmit={handleCreateSkill} className="p-4 space-y-4">
              <div>
                <label className="block text-sm font-medium text-[var(--color-text)] mb-1">Skill Name</label>
                <input
                  type="text"
                  value={newSkill.name}
                  onChange={(e) => setNewSkill({ ...newSkill, name: e.target.value })}
                  placeholder="my-new-skill"
                  className="w-full px-3 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                  required
                />
                <p className="text-xs text-[var(--color-text-muted)] mt-1">Lowercase with hyphens</p>
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-text)] mb-1">Description</label>
                <input
                  type="text"
                  value={newSkill.description}
                  onChange={(e) => setNewSkill({ ...newSkill, description: e.target.value })}
                  placeholder="What this skill does"
                  className="w-full px-3 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-[var(--color-text)] mb-1">Procedure Content</label>
                <textarea
                  value={newSkill.content}
                  onChange={(e) => setNewSkill({ ...newSkill, content: e.target.value })}
                  placeholder="Step-by-step instructions..."
                  rows={6}
                  className="w-full px-3 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50 font-mono text-sm"
                  required
                />
              </div>
              <div className="flex justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setShowCreateForm(false)}
                  className="px-4 py-2 rounded-lg border border-[var(--color-border)] text-[var(--color-text)] hover:bg-[var(--color-muted)] transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={creating || !newSkill.name || !newSkill.content}
                  className="px-4 py-2 rounded-lg bg-[#4248f1] text-white hover:bg-[#4248f1]/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {creating ? 'Creating...' : 'Create Skill'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {error && (
        <div className="fixed bottom-4 right-4 bg-red-500/10 border border-red-500/30 rounded-lg p-4 text-red-500">
          {error}
        </div>
      )}
    </div>
  );
}
