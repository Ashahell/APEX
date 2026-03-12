import { useState, useEffect } from 'react';
import { apiGet } from '../../lib/api';

interface Skill {
  name: string;
  version: string;
  description: string;
  tier: string;
  runtime: string;
  author?: string;
  health_status?: string;
  capabilities?: string[];
}

interface SkillDetail {
  name: string;
  version: string;
  description: string;
  tier: string;
  input_schema: Record<string, unknown>;
  output_schema: Record<string, unknown>;
  capabilities: string[];
  runtime: string;
  author?: string;
  health_status?: string;
}

export function SkillMarketplace() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedSkill, setSelectedSkill] = useState<SkillDetail | null>(null);
  const [filterTier, setFilterTier] = useState<string>('all');

  useEffect(() => {
    loadSkills();
  }, []);

  const loadSkills = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/skills');
      if (res.ok) {
        const data = await res.json();
        setSkills(data);
      }
    } catch (err) {
      console.error('Failed to load skills:', err);
    } finally {
      setLoading(false);
    }
  };

  const viewSkill = async (name: string) => {
    try {
      const res = await apiGet(`/api/v1/skills/${name}`);
      if (res.ok) {
        const data = await res.json();
        setSelectedSkill(data);
      }
    } catch (err) {
      console.error('Failed to load skill:', err);
    }
  };

  const filteredSkills = skills.filter(skill => {
    const matchesSearch = skill.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      skill.description?.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesTier = filterTier === 'all' || skill.tier === filterTier;
    return matchesSearch && matchesTier;
  });

  const getTierBadge = (tier: string) => {
    const colors: Record<string, string> = {
      T0: 'bg-green-500/10 text-green-500 border border-green-500/20',
      T1: 'bg-blue-500/10 text-blue-500 border border-blue-500/20',
      T2: 'bg-yellow-500/10 text-yellow-500 border border-yellow-500/20',
      T3: 'bg-red-500/10 text-red-500 border border-red-500/20',
    };
    return colors[tier] || 'bg-[var(--color-muted)] text-[var(--color-text-muted)] border border-[var(--color-border)]';
  };

  const getHealthBadge = (status?: string) => {
    switch (status) {
      case 'healthy': return 'bg-green-500/10 text-green-500 border border-green-500/20';
      case 'degraded': return 'bg-yellow-500/10 text-yellow-500 border border-yellow-500/20';
      case 'unhealthy': return 'bg-red-500/10 text-red-500 border border-red-500/20';
      default: return 'bg-[var(--color-muted)] text-[var(--color-text-muted)] border border-[var(--color-border)]';
    }
  };

  if (selectedSkill) {
    return (
      <div className="h-full overflow-auto p-6">
        <div className="max-w-2xl mx-auto space-y-4">
          <button
            onClick={() => setSelectedSkill(null)}
            className="flex items-center gap-2 text-sm text-[var(--color-text-muted)] hover:text-[#4248f1] transition-colors"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="19" y1="12" x2="5" y2="12"></line>
              <polyline points="12 19 5 12 12 5"></polyline>
            </svg>
            Back to marketplace
          </button>

          <div className="border border-[var(--color-border)] rounded-xl p-6 bg-[var(--color-panel)]">
            <div className="flex items-center justify-between mb-2">
              <h2 className="text-2xl font-bold">{selectedSkill.name}</h2>
              <span className={`px-3 py-1 rounded-full text-xs font-medium ${getTierBadge(selectedSkill.tier)}`}>
                {selectedSkill.tier}
              </span>
            </div>
            <p className="text-[var(--color-text-muted)]">{selectedSkill.description}</p>
            <div className="flex items-center gap-4 mt-3 text-sm text-[var(--color-text-muted)]">
              <span className="flex items-center gap-1">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="12" y1="20" x2="12" y2="10"></line>
                  <line x1="18" y1="20" x2="18" y2="4"></line>
                  <line x1="6" y1="20" x2="6" y2="16"></line>
                </svg>
                v{selectedSkill.version}
              </span>
              <span className="flex items-center gap-1">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="4 17 10 11 4 5"></polyline>
                  <line x1="12" y1="19" x2="20" y2="19"></line>
                </svg>
                {selectedSkill.runtime}
              </span>
              {selectedSkill.author && <span>by {selectedSkill.author}</span>}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <h3 className="font-semibold mb-2 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
                Health Status
              </h3>
              <span className={`px-3 py-1.5 rounded-lg text-sm font-medium ${getHealthBadge(selectedSkill.health_status)}`}>
                {selectedSkill.health_status || 'unknown'}
              </span>
            </div>
            <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
              <h3 className="font-semibold mb-2 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
                </svg>
                Capabilities
              </h3>
              <div className="flex flex-wrap gap-1">
                {selectedSkill.capabilities?.map(cap => (
                  <span key={cap} className="text-xs px-2 py-1 bg-[var(--color-muted)] rounded-full">
                    {cap}
                  </span>
                )) || <span className="text-[var(--color-text-muted)]">None</span>}
              </div>
            </div>
          </div>

          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <h3 className="font-semibold mb-2 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
                <line x1="16" y1="13" x2="8" y2="13"></line>
                <line x1="16" y1="17" x2="8" y2="17"></line>
              </svg>
              Input Schema
            </h3>
            <pre className="text-xs bg-[var(--color-muted)]/30 p-4 rounded-lg overflow-auto border border-[var(--color-border)]">
              {JSON.stringify(selectedSkill.input_schema, null, 2)}
            </pre>
          </div>

          <div className="border border-[var(--color-border)] rounded-xl p-4 bg-[var(--color-panel)]">
            <h3 className="font-semibold mb-2 flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
                <line x1="12" y1="18" x2="12" y2="12"></line>
                <line x1="9" y1="15" x2="15" y2="15"></line>
              </svg>
              Output Schema
            </h3>
            <pre className="text-xs bg-[var(--color-muted)]/30 p-4 rounded-lg overflow-auto border border-[var(--color-border)]">
              {JSON.stringify(selectedSkill.output_schema, null, 2)}
            </pre>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-[var(--color-border)]">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-[#4248f1]/10 flex items-center justify-center">
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                <line x1="7" y1="7" x2="7.01" y2="7"></line>
              </svg>
            </div>
            <div>
              <h2 className="text-xl font-bold">Skill Marketplace</h2>
              <p className="text-sm text-[var(--color-text-muted)]">
                Browse and manage available skills
              </p>
            </div>
          </div>
          <button
            onClick={loadSkills}
            className="px-4 py-2 rounded-lg border border-[var(--color-border)] hover:bg-[var(--color-muted)] transition-colors flex items-center gap-2"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            Refresh
          </button>
        </div>
        
        <div className="flex gap-3">
          <input
            type="text"
            placeholder="Search skills..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
          />
          <select
            value={filterTier}
            onChange={(e) => setFilterTier(e.target.value)}
            className="px-3 py-2.5 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
          >
            <option value="all">All Tiers</option>
            <option value="T0">T0 (Read-only)</option>
            <option value="T1">T1 (Tap to confirm)</option>
            <option value="T2">T2 (Type to confirm)</option>
            <option value="T3">T3 (TOTP required)</option>
          </select>
        </div>
      </div>

      {/* Skills Grid */}
      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="text-center text-[var(--color-text-muted)] py-8 flex items-center justify-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="animate-spin">
              <line x1="12" y1="2" x2="12" y2="6"></line>
              <line x1="12" y1="18" x2="12" y2="22"></line>
              <line x1="4.93" y1="4.93" x2="7.76" y2="7.76"></line>
              <line x1="16.24" y1="16.24" x2="19.07" y2="19.07"></line>
              <line x1="2" y1="12" x2="6" y2="12"></line>
              <line x1="18" y1="12" x2="22" y2="12"></line>
              <line x1="4.93" y1="19.07" x2="7.76" y2="16.24"></line>
              <line x1="16.24" y1="7.76" x2="19.07" y2="4.93"></line>
            </svg>
            Loading...
          </div>
        ) : filteredSkills.length === 0 ? (
          <div className="text-center text-[var(--color-text-muted)] py-8">
            No skills found
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredSkills.map((skill) => (
              <button
                key={skill.name}
                onClick={() => viewSkill(skill.name)}
                className="border border-[var(--color-border)] rounded-xl p-4 text-left hover:bg-[var(--color-muted)]/30 hover:border-[#4248f1]/30 transition-all group"
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="font-medium group-hover:text-[#4248f1] transition-colors">{skill.name}</span>
                  <span className={`px-2 py-0.5 rounded-full text-xs font-medium ${getTierBadge(skill.tier)}`}>
                    {skill.tier}
                  </span>
                </div>
                <p className="text-sm text-[var(--color-text-muted)] line-clamp-2 mb-3">
                  {skill.description || 'No description'}
                </p>
                <div className="flex items-center justify-between text-xs text-[var(--color-text-muted)]">
                  <span className="flex items-center gap-1">
                    <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                      <line x1="12" y1="20" x2="12" y2="10"></line>
                      <line x1="18" y1="20" x2="18" y2="4"></line>
                      <line x1="6" y1="20" x2="6" y2="16"></line>
                    </svg>
                    v{skill.version}
                  </span>
                  <span className={`px-2 py-0.5 rounded-full ${getHealthBadge(skill.health_status)}`}>
                    {skill.health_status || 'unknown'}
                  </span>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="p-4 border-t border-[var(--color-border)] text-sm text-[var(--color-text-muted)]">
        Showing {filteredSkills.length} of {skills.length} skills
      </div>
    </div>
  );
}
