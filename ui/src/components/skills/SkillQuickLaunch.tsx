import { useState, useEffect } from 'react';
import { apiFetch } from '../../lib/api';

interface Skill {
  name: string;
  version: string;
  tier: string;
  status: string;
}

interface SkillQuickLaunchProps {
  onSelectSkill?: (skillName: string) => void;
}

export function SkillQuickLaunch({ onSelectSkill }: SkillQuickLaunchProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [skills, setSkills] = useState<Skill[]>([]);
  const [filteredSkills, setFilteredSkills] = useState<Skill[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    apiFetch('/api/v1/skills')
      .then((res) => res.json())
      .then((data) => {
        setSkills(data);
        setFilteredSkills(data);
        setLoading(false);
      })
      .catch(() => {
        setLoading(false);
      });
  }, []);

  useEffect(() => {
    if (searchQuery.trim() === '') {
      setFilteredSkills(skills);
    } else {
      const query = searchQuery.toLowerCase();
      setFilteredSkills(
        skills.filter((skill) =>
          skill.name.toLowerCase().includes(query)
        )
      );
    }
  }, [searchQuery, skills]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        setIsOpen(false);
        setSearchQuery('');
      }
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        setIsOpen(true);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen]);

  const handleSkillClick = (skillName: string) => {
    if (onSelectSkill) {
      onSelectSkill(skillName);
    }
    setIsOpen(false);
    setSearchQuery('');
  };

  const getTierIcon = (tier: string) => {
    switch (tier) {
      case 'T0':
        return '⚡';
      case 'T1':
        return '✨';
      case 'T2':
        return '⚠️';
      case 'T3':
        return '🔒';
      default:
        return '•';
    }
  };

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'T0':
        return 'bg-green-100 text-green-800 border-green-200';
      case 'T1':
        return 'bg-blue-100 text-blue-800 border-blue-200';
      case 'T2':
        return 'bg-yellow-100 text-yellow-800 border-yellow-200';
      case 'T3':
        return 'bg-red-100 text-red-800 border-red-200';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="flex items-center gap-2 px-3 py-1.5 text-sm bg-primary/10 hover:bg-primary/20 rounded-md transition-colors"
        title="Quick Launch Skills (Ctrl+K)"
      >
        <span>✨</span>
        <span className="hidden sm:inline">Skills</span>
      </button>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-16">
      <div
        className="absolute inset-0 bg-black/50"
        onClick={() => {
          setIsOpen(false);
          setSearchQuery('');
        }}
      />
      <div className="relative w-full max-w-lg mx-4 bg-background rounded-xl shadow-2xl border overflow-hidden">
        <div className="flex items-center gap-3 p-4 border-b">
          <span className="text-muted-foreground">🔍</span>
          <input
            type="text"
            placeholder="Search skills..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 bg-transparent outline-none text-sm placeholder:text-muted-foreground"
            autoFocus
          />
          <button
            onClick={() => {
              setIsOpen(false);
              setSearchQuery('');
            }}
            className="p-1 hover:bg-muted rounded text-sm"
          >
            ✕
          </button>
        </div>

        <div className="max-h-80 overflow-y-auto p-2">
          {loading ? (
            <div className="p-4 text-center text-muted-foreground">
              Loading skills...
            </div>
          ) : filteredSkills.length === 0 ? (
            <div className="p-4 text-center text-muted-foreground">
              No skills found
            </div>
          ) : (
            <div className="space-y-1">
              {filteredSkills.map((skill) => (
                <button
                  key={skill.name}
                  onClick={() => handleSkillClick(skill.name)}
                  className="w-full flex items-center justify-between p-3 hover:bg-muted rounded-lg transition-colors text-left"
                >
                  <div className="flex items-center gap-3">
                    <span className="font-medium">{skill.name}</span>
                    <span className="text-xs text-muted-foreground">
                      v{skill.version}
                    </span>
                  </div>
                  <span
                    className={`flex items-center gap-1.5 px-2 py-0.5 rounded text-xs font-medium border ${getTierColor(
                      skill.tier
                    )}`}
                  >
                    {getTierIcon(skill.tier)} {skill.tier}
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="p-3 border-t bg-muted/30 text-xs text-muted-foreground flex justify-between">
          <span>
            {filteredSkills.length} skill{filteredSkills.length !== 1 ? 's' : ''}
          </span>
          <span>Ctrl+K to open • ESC to close</span>
        </div>
      </div>
    </div>
  );
}
