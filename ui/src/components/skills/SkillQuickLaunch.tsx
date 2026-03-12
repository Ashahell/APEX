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
        return <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20"><path fillRule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clipRule="evenodd" /></svg>;
      case 'T1':
        return <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20"><path fillRule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clipRule="evenodd" /></svg>;
      case 'T2':
        return <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20"><path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" /></svg>;
      case 'T3':
        return <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20"><path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" /></svg>;
      default:
        return <span className="w-3 h-3">•</span>;
    }
  };

  const getTierColor = (tier: string) => {
    switch (tier) {
      case 'T0':
        return 'bg-[#4248f1]/20 text-[#4248f1] border-[#4248f1]/30';
      case 'T1':
        return 'bg-blue-500/20 text-blue-500 border-blue-500/30';
      case 'T2':
        return 'bg-yellow-500/20 text-yellow-500 border-yellow-500/30';
      case 'T3':
        return 'bg-red-500/20 text-red-500 border-red-500/30';
      default:
        return 'bg-gray-500/20 text-gray-500 border-gray-500/30';
    }
  };

  if (!isOpen) {
    return (
      <button
        onClick={() => setIsOpen(true)}
        className="flex items-center gap-2 px-3 py-1.5 text-sm bg-[#4248f1]/20 hover:bg-[#4248f1]/30 rounded-xl transition-colors"
        title="Quick Launch Skills (Ctrl+K)"
      >
        <svg className="w-4 h-4" style={{ color: '#4248f1' }} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
        <span className="hidden sm:inline">Skills</span>
      </button>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-16">
      <div
        className="absolute inset-0 bg-black/60 backdrop-blur-sm"
        onClick={() => {
          setIsOpen(false);
          setSearchQuery('');
        }}
      />
      <div className="relative w-full max-w-lg mx-4 bg-[var(--color-panel)] rounded-2xl shadow-2xl border border-border overflow-hidden">
        <div className="flex items-center gap-3 p-4 border-b border-border">
          <svg className="w-5 h-5 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            placeholder="Search skills..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="flex-1 bg-transparent outline-none text-sm placeholder:text-[var(--color-text-muted)]"
            autoFocus
          />
          <button
            onClick={() => {
              setIsOpen(false);
              setSearchQuery('');
            }}
            className="p-1.5 hover:bg-[#4248f1]/20 rounded-lg transition-colors"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <div className="max-h-80 overflow-y-auto p-2">
          {loading ? (
            <div className="p-4 text-center text-[var(--color-text-muted)]">
              Loading skills...
            </div>
          ) : filteredSkills.length === 0 ? (
            <div className="p-4 text-center text-[var(--color-text-muted)]">
              No skills found
            </div>
          ) : (
            <div className="space-y-1">
              {filteredSkills.map((skill) => (
                <button
                  key={skill.name}
                  onClick={() => handleSkillClick(skill.name)}
                  className="w-full flex items-center justify-between p-3 hover:bg-[#4248f1]/10 rounded-xl transition-colors text-left"
                >
                  <div className="flex items-center gap-3">
                    <span className="font-medium">{skill.name}</span>
                    <span className="text-xs text-[var(--color-text-muted)]">
                      v{skill.version}
                    </span>
                  </div>
                  <span
                    className={`flex items-center gap-1.5 px-2 py-0.5 rounded-lg text-xs font-medium border ${getTierColor(
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

        <div className="p-3 border-t border-border bg-[var(--color-background)]/50 text-xs text-[var(--color-text-muted)] flex justify-between">
          <span>
            {filteredSkills.length} skill{filteredSkills.length !== 1 ? 's' : ''}
          </span>
          <span>Ctrl+K to open • ESC to close</span>
        </div>
      </div>
    </div>
  );
}
