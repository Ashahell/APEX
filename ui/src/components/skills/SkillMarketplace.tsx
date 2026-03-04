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
      T0: 'bg-green-500/20 text-green-500',
      T1: 'bg-blue-500/20 text-blue-500',
      T2: 'bg-yellow-500/20 text-yellow-500',
      T3: 'bg-red-500/20 text-red-500',
    };
    return colors[tier] || 'bg-muted text-muted-foreground';
  };

  const getHealthBadge = (status?: string) => {
    switch (status) {
      case 'healthy': return 'bg-green-500/20 text-green-500';
      case 'degraded': return 'bg-yellow-500/20 text-yellow-500';
      case 'unhealthy': return 'bg-red-500/20 text-red-500';
      default: return 'bg-muted text-muted-foreground';
    }
  };

  if (selectedSkill) {
    return (
      <div className="h-full overflow-auto p-4">
        <div className="max-w-2xl mx-auto space-y-4">
          <button
            onClick={() => setSelectedSkill(null)}
            className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
          >
            ← Back to marketplace
          </button>

          <div className="border rounded-lg p-4">
            <div className="flex items-center justify-between mb-2">
              <h2 className="text-2xl font-bold">{selectedSkill.name}</h2>
              <span className={`px-2 py-1 rounded text-xs ${getTierBadge(selectedSkill.tier)}`}>
                {selectedSkill.tier}
              </span>
            </div>
            <p className="text-muted-foreground">{selectedSkill.description}</p>
            <div className="flex items-center gap-4 mt-2 text-sm text-muted-foreground">
              <span>v{selectedSkill.version}</span>
              <span>{selectedSkill.runtime}</span>
              {selectedSkill.author && <span>by {selectedSkill.author}</span>}
            </div>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="border rounded-lg p-4">
              <h3 className="font-semibold mb-2">Health Status</h3>
              <span className={`px-2 py-1 rounded text-sm ${getHealthBadge(selectedSkill.health_status)}`}>
                {selectedSkill.health_status || 'unknown'}
              </span>
            </div>
            <div className="border rounded-lg p-4">
              <h3 className="font-semibold mb-2">Capabilities</h3>
              <div className="flex flex-wrap gap-1">
                {selectedSkill.capabilities?.map(cap => (
                  <span key={cap} className="text-xs px-2 py-0.5 bg-muted rounded">
                    {cap}
                  </span>
                )) || <span className="text-muted-foreground">None</span>}
              </div>
            </div>
          </div>

          <div className="border rounded-lg p-4">
            <h3 className="font-semibold mb-2">Input Schema</h3>
            <pre className="text-xs bg-muted p-3 rounded-lg overflow-auto">
              {JSON.stringify(selectedSkill.input_schema, null, 2)}
            </pre>
          </div>

          <div className="border rounded-lg p-4">
            <h3 className="font-semibold mb-2">Output Schema</h3>
            <pre className="text-xs bg-muted p-3 rounded-lg overflow-auto">
              {JSON.stringify(selectedSkill.output_schema, null, 2)}
            </pre>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-xl font-bold">Skill Marketplace</h2>
            <p className="text-sm text-muted-foreground">
              Browse and manage available skills
            </p>
          </div>
          <button
            onClick={loadSkills}
            className="px-4 py-2 rounded-lg border hover:bg-muted"
          >
            Refresh
          </button>
        </div>
        
        <div className="flex gap-4">
          <input
            type="text"
            placeholder="Search skills..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 px-3 py-2 rounded-lg border bg-background"
          />
          <select
            value={filterTier}
            onChange={(e) => setFilterTier(e.target.value)}
            className="px-3 py-2 rounded-lg border bg-background"
          >
            <option value="all">All Tiers</option>
            <option value="T0">T0 (Read-only)</option>
            <option value="T1">T1 (Tap to confirm)</option>
            <option value="T2">T2 (Type to confirm)</option>
            <option value="T3">T3 (TOTP required)</option>
          </select>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="text-center text-muted-foreground py-8">Loading...</div>
        ) : filteredSkills.length === 0 ? (
          <div className="text-center text-muted-foreground py-8">
            No skills found
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredSkills.map((skill) => (
              <button
                key={skill.name}
                onClick={() => viewSkill(skill.name)}
                className="border rounded-lg p-4 text-left hover:bg-muted/50 transition-colors"
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="font-medium">{skill.name}</span>
                  <span className={`px-2 py-0.5 rounded text-xs ${getTierBadge(skill.tier)}`}>
                    {skill.tier}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground line-clamp-2 mb-2">
                  {skill.description || 'No description'}
                </p>
                <div className="flex items-center justify-between text-xs text-muted-foreground">
                  <span>v{skill.version}</span>
                  <span className={getHealthBadge(skill.health_status)}>
                    {skill.health_status || 'unknown'}
                  </span>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

      <div className="p-4 border-t text-sm text-muted-foreground">
        Showing {filteredSkills.length} of {skills.length} skills
      </div>
    </div>
  );
}
