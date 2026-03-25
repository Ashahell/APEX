import { useState, useEffect } from 'react';

interface Persona {
  name: string;
  description?: string;
  prompt_template?: string;
  tools?: string[];
  model?: string;
  voice?: string;
  is_active?: boolean;
}

interface PersonaListProps {
  onSelect?: (persona: Persona) => void;
  onEdit?: (persona: Persona) => void;
  compact?: boolean;
}

const DEFAULT_PERSONAS: Persona[] = [
  {
    name: 'default',
    description: 'Balanced assistant for general tasks',
    is_active: true,
  },
  {
    name: 'coder',
    description: 'Specialized in code review and generation',
    tools: ['code.generate', 'code.review'],
  },
  {
    name: 'researcher',
    description: 'Deep research and analysis',
    tools: ['web.search', 'docs.read'],
  },
];

export function PersonaList({ onSelect, onEdit, compact = false }: PersonaListProps) {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [activePersona, setActivePersona] = useState<string>('default');
  const [isLoading, setIsLoading] = useState(true);
  const [showDropdown, setShowDropdown] = useState(false);

  useEffect(() => {
    loadPersonas();
  }, []);

  const loadPersonas = async () => {
    setIsLoading(true);
    try {
      const res = await fetch('/api/v1/personas', {
        headers: {
          'X-APEX-Signature': 'dev-signature',
          'X-APEX-Timestamp': Math.floor(Date.now() / 1000).toString(),
        },
      });
      
      if (res.ok) {
        const data = await res.json();
        setPersonas(data.personas || DEFAULT_PERSONAS);
      } else {
        setPersonas(DEFAULT_PERSONAS);
      }
    } catch (err) {
      console.warn('Failed to load personas:', err);
      setPersonas(DEFAULT_PERSONAS);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectPersona = (persona: Persona) => {
    setActivePersona(persona.name);
    onSelect?.(persona);
    setShowDropdown(false);
    localStorage.setItem('apex-active-persona', persona.name);
  };

  const currentPersona = personas.find(p => p.name === activePersona) || personas[0];

  const getPersonaIcon = (name: string) => {
    switch (name) {
      case 'default':
        return '🤖';
      case 'coder':
        return '💻';
      case 'researcher':
        return '🔬';
      default:
        return '👤';
    }
  };

  if (isLoading) {
    return (
      <div className="animate-pulse flex items-center gap-2">
        <div className="w-8 h-8 bg-gray-200 dark:bg-gray-700 rounded-full" />
        <div className="w-24 h-4 bg-gray-200 dark:bg-gray-700 rounded" />
      </div>
    );
  }

  if (compact) {
    return (
      <div className="relative">
        <button
          onClick={() => setShowDropdown(!showDropdown)}
          className="flex items-center gap-2 px-2 py-1 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800"
        >
          <span className="text-lg">{getPersonaIcon(currentPersona?.name || 'default')}</span>
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
            {currentPersona?.name || 'default'}
          </span>
          <svg className="w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </button>

        {showDropdown && (
          <>
            <div
              className="fixed inset-0 z-40"
              onClick={() => setShowDropdown(false)}
            />
            <div className="absolute left-0 mt-2 w-56 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50">
              {personas.map((persona) => (
                <button
                  key={persona.name}
                  onClick={() => handleSelectPersona(persona)}
                  className={`w-full flex items-center gap-3 px-3 py-2 text-left hover:bg-gray-50 dark:hover:bg-gray-700 ${
                    activePersona === persona.name ? 'bg-indigo-50 dark:bg-indigo-900/20' : ''
                  }`}
                >
                  <span className="text-lg">{getPersonaIcon(persona.name)}</span>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                      {persona.name}
                    </p>
                    {persona.description && (
                      <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                        {persona.description}
                      </p>
                    )}
                  </div>
                  {activePersona === persona.name && (
                    <svg className="w-4 h-4 text-indigo-500" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                  )}
                </button>
              ))}
              <div className="border-t border-gray-200 dark:border-gray-700 p-2">
                <button
                  onClick={() => { onEdit?.(currentPersona!); setShowDropdown(false); }}
                  className="w-full flex items-center gap-2 px-3 py-2 text-sm text-indigo-600 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 rounded"
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                  Create New Persona
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Personas
        </h3>
        <button
          onClick={() => onEdit?.({ name: '', description: '' })}
          className="text-xs text-indigo-600 hover:text-indigo-700 dark:text-indigo-400"
        >
          + New
        </button>
      </div>

      <div className="space-y-2">
        {personas.map((persona) => (
          <div
            key={persona.name}
            className={`p-3 rounded-lg border cursor-pointer transition-colors ${
              activePersona === persona.name
                ? 'border-indigo-500 bg-indigo-50 dark:bg-indigo-900/20'
                : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
            }`}
            onClick={() => handleSelectPersona(persona)}
          >
            <div className="flex items-start gap-3">
              <span className="text-2xl">{getPersonaIcon(persona.name)}</span>
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between">
                  <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                    {persona.name}
                  </p>
                  {activePersona === persona.name && (
                    <span className="text-xs text-indigo-600 dark:text-indigo-400">
                      Active
                    </span>
                  )}
                </div>
                {persona.description && (
                  <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                    {persona.description}
                  </p>
                )}
                {persona.tools && persona.tools.length > 0 && (
                  <div className="flex flex-wrap gap-1 mt-2">
                    {persona.tools.slice(0, 3).map((tool) => (
                      <span
                        key={tool}
                        className="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-xs text-gray-600 dark:text-gray-400"
                      >
                        {tool}
                      </span>
                    ))}
                    {persona.tools.length > 3 && (
                      <span className="text-xs text-gray-400">
                        +{persona.tools.length - 3}
                      </span>
                    )}
                  </div>
                )}
              </div>
              <button
                onClick={(e) => { e.stopPropagation(); onEdit?.(persona); }}
                className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
              >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
