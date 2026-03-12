import { useEffect, useState } from 'react';
import { apiFetch } from '../../lib/api';

interface Skill {
  name: string;
  version: string;
  tier: string;
  status: string;
  description?: string;
}

type SkillTier = 'all' | 'T0' | 'T1' | 'T2' | 'T3';

export function Skills() {
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterTier, setFilterTier] = useState<SkillTier>('all');
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    apiFetch('/api/v1/skills')
      .then((res) => res.json())
      .then((data) => {
        setSkills(data);
        setLoading(false);
      })
      .catch((err) => {
        setError(err.message);
        setLoading(false);
      });
  }, []);

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'T0':
        return 'bg-[#4248f1]/20 text-[#4248f1] border-[#4248f1]/30';
      case 'T1':
        return 'bg-blue-500/20 text-blue-500 border-blue-500/30';
      case 'T2':
        return 'bg-amber-500/20 text-amber-500 border-amber-500/30';
      case 'T3':
        return 'bg-red-500/20 text-red-500 border-red-500/30';
      default:
        return 'bg-[var(--color-text-muted)]/20 text-[var(--color-text-muted)] border-[var(--color-text-muted)]/30';
    }
  };

  const getTierDescription = (tier: string) => {
    switch (tier) {
      case 'T0':
        return 'Read-only queries';
      case 'T1':
        return 'Tap to confirm';
      case 'T2':
        return 'Type to confirm';
      case 'T3':
        return 'TOTP verification required';
      default:
        return '';
    }
  };

  const filteredSkills = skills.filter((skill) => {
    const matchesTier = filterTier === 'all' || skill.tier === filterTier;
    const matchesSearch = skill.name.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesTier && matchesSearch;
  });

  const tierCounts = {
    all: skills.length,
    T0: skills.filter((s) => s.tier === 'T0').length,
    T1: skills.filter((s) => s.tier === 'T1').length,
    T2: skills.filter((s) => s.tier === 'T2').length,
    T3: skills.filter((s) => s.tier === 'T3').length,
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading skills...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-red-500">Error: {error}</div>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      {/* Header */}
      <div className="border-b border-border p-6">
        <h2 className="text-2xl font-semibold" style={{ color: '#4248f1' }}>Skills</h2>
        <p className="text-[var(--color-text-muted)] mt-1">Browse and manage available skills</p>
      </div>

      {/* Search and Filter */}
      <div className="p-4 border-b bg-[var(--color-panel)]">
        <div className="flex gap-4 flex-wrap">
          {/* Search */}
          <div className="flex-1 min-w-[200px]">
            <div className="relative">
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
              </svg>
              <input
                type="text"
                placeholder="Search skills..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-2 rounded-lg border border-[var(--color-border)] bg-[var(--color-background)] text-[var(--color-text)] focus:outline-none focus:ring-2 focus:ring-[#4248f1]/50"
              />
            </div>
          </div>

          {/* Tier Filter Tabs */}
          <div className="flex gap-1 bg-[var(--color-muted)] p-1 rounded-xl">
            {(['all', 'T0', 'T1', 'T2', 'T3'] as SkillTier[]).map((tier) => (
              <button
                key={tier}
                onClick={() => setFilterTier(tier)}
                className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                  filterTier === tier
                    ? 'bg-[#4248f1] text-white'
                    : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[#4248f1]/10'
                }`}
              >
                {tier === 'all' ? 'All' : tier}
                <span className="ml-1 text-xs opacity-70">({tierCounts[tier]})</span>
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Skills Grid */}
      <div className="p-6">
        {filteredSkills.length === 0 ? (
          <div className="text-center py-12">
            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="mx-auto text-[var(--color-text-muted)] opacity-50">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            <p className="text-[var(--color-text-muted)] mt-4">No skills found</p>
            <p className="text-sm text-[var(--color-text-muted)] mt-1">
              {searchQuery ? 'Try a different search term' : 'Skills can be registered via the API'}
            </p>
          </div>
        ) : (
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {filteredSkills.map((skill) => (
              <div
                key={skill.name}
                className="border border-[var(--color-border)] rounded-xl p-4 hover:shadow-lg hover:border-[#4248f1]/30 transition-all cursor-pointer group"
              >
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-[#4248f1]/10 flex items-center justify-center">
                      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#4248f1" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                        <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
                        <polyline points="2 17 12 22 22 17"></polyline>
                        <polyline points="2 12 12 17 22 12"></polyline>
                      </svg>
                    </div>
                    <div>
                      <h3 className="font-semibold text-[var(--color-text)] group-hover:text-[#4248f1] transition-colors">{skill.name}</h3>
                      <p className="text-xs text-[var(--color-text-muted)]">v{skill.version}</p>
                    </div>
                  </div>
                  <span className={`px-2.5 py-1 rounded-full text-xs font-medium border ${getTierColor(skill.tier)}`}>
                    {skill.tier}
                  </span>
                </div>
                
                <p className="text-xs text-[var(--color-text-muted)] mb-3">
                  {getTierDescription(skill.tier)}
                </p>
                
                <div className="flex items-center justify-between pt-3 border-t border-[var(--color-border)]">
                  <div className="flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-full ${skill.status === 'active' ? 'bg-green-500' : 'bg-gray-400'}`}></span>
                    <span className="text-xs text-[var(--color-text-muted)] capitalize">{skill.status || 'active'}</span>
                  </div>
                  <button className="text-xs text-[#4248f1] hover:underline opacity-0 group-hover:opacity-100 transition-opacity">
                    Configure
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
