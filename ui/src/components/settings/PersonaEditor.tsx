import { useState, useEffect } from 'react';
import { apiGet, apiPost } from '../../lib/api';

interface PromptPiece {
  piece_type: string;
  content: string;
  order: number;
  enabled: boolean;
}

interface Persona {
  id: string;
  name: string;
  description: string | null;
  prompt_pieces: PromptPiece[];
  tools: string[];
  voice_config: {
    tts_engine: string | null;
    voice_id: string | null;
    speed: number | null;
    pitch: number | null;
  };
  model_config: {
    provider: string | null;
    model: string | null;
    temperature: number | null;
    max_tokens: number | null;
  };
  is_active: boolean;
  assembled_prompt: string;
}

export function PersonaEditor() {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [activePersona, setActivePersona] = useState<Persona | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPersonas();
  }, []);

  const loadPersonas = async () => {
    setLoading(true);
    try {
      const res = await apiGet('/api/v1/personas');
      if (res.ok) {
        const data = await res.json();
        setPersonas(data);
        const active = data.find((p: Persona) => p.is_active);
        if (active) setActivePersona(active);
      }
    } catch (err) {
      console.error('Failed to load personas:', err);
    } finally {
      setLoading(false);
    }
  };

  const activatePersona = async (id: string) => {
    try {
      const res = await apiPost(`/api/v1/personas/${id}/activate`, {});
      if (res.ok) {
        const data = await res.json();
        setActivePersona(data);
        loadPersonas();
      }
    } catch (err) {
      console.error('Failed to activate persona:', err);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-[var(--color-text-muted)]">Loading personas...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="mb-6">
        <h2 className="text-2xl font-semibold">Persona Editor</h2>
        <p className="text-[var(--color-text-muted)]">
          Create and manage AI personas with custom prompts, tools, and settings
        </p>
      </div>

      {/* Persona List */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {personas.map((persona) => (
          <div
            key={persona.id}
            className={`p-4 rounded-lg border cursor-pointer transition-colors ${
              activePersona?.id === persona.id
                ? 'border-primary bg-primary/5'
                : 'border-border hover:border-primary/50'
            }`}
            onClick={() => activatePersona(persona.id)}
          >
            <div className="flex items-center justify-between mb-2">
              <h3 className="font-medium">{persona.name}</h3>
              {persona.is_active && (
                <span className="px-2 py-0.5 text-xs rounded-full bg-primary/10 text-primary">
                  Active
                </span>
              )}
            </div>
            {persona.description && (
              <p className="text-sm text-[var(--color-text-muted)] mb-2">
                {persona.description}
              </p>
            )}
            <div className="text-xs text-[var(--color-text-muted)]">
              {persona.prompt_pieces.length} prompt pieces, {persona.tools.length} tools
            </div>
          </div>
        ))}
      </div>

      {/* Active Persona Details */}
      {activePersona && (
        <div className="p-6 rounded-lg border bg-card">
          <h3 className="font-medium mb-4">Active Persona: {activePersona.name}</h3>
          
          {/* Assembled Prompt Preview */}
          <div className="mb-6">
            <h4 className="text-sm font-medium mb-2">System Prompt Preview</h4>
            <div className="p-3 rounded bg-secondary text-secondary-foreground text-sm max-h-40 overflow-y-auto">
              {activePersona.assembled_prompt || 'No prompt pieces configured'}
            </div>
          </div>

          {/* Prompt Pieces */}
          <div className="mb-6">
            <h4 className="text-sm font-medium mb-2">Prompt Pieces</h4>
            <div className="space-y-2">
              {activePersona.prompt_pieces.map((piece, idx) => (
                <div key={idx} className="flex items-center gap-2 text-sm">
                  <span className="px-2 py-0.5 rounded bg-secondary text-secondary-foreground">
                    {piece.piece_type}
                  </span>
                  <span className="truncate flex-1">{piece.content}</span>
                </div>
              ))}
            </div>
          </div>

          {/* Tools */}
          <div>
            <h4 className="text-sm font-medium mb-2">Available Tools</h4>
            <div className="flex flex-wrap gap-2">
              {activePersona.tools.length > 0 ? (
                activePersona.tools.map((tool, idx) => (
                  <span key={idx} className="px-2 py-1 text-xs rounded bg-secondary text-secondary-foreground">
                    {tool}
                  </span>
                ))
              ) : (
                <span className="text-sm text-[var(--color-text-muted)]">No tools assigned</span>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Info Box */}
      <div className="p-4 rounded-lg bg-blue-500/10 border border-blue-500/20">
        <h4 className="font-medium text-blue-500 mb-2">About Personas</h4>
        <p className="text-sm text-[var(--color-text-muted)]">
          Personas bundle system prompts, voice settings, tools, and model configuration
          into swappable profiles. Click a persona to activate it. The assembled prompt
          is used in the agent loop.
        </p>
      </div>
    </div>
  );
}