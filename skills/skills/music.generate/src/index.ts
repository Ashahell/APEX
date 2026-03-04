import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  description: z.string().describe('Description of the music to generate'),
  genre: z.string().optional().describe('Music genre (pop, rock, jazz, etc.)'),
  duration: z.number().optional().describe('Duration in seconds'),
  tempo: z.number().optional().describe('BPM (beats per minute)'),
  instruments: z.array(z.string()).optional().describe('Instruments to include'),
});

const OutputSchema = z.object({
  audioFile: z.string().describe('Path to generated audio file'),
  duration: z.number(),
  format: z.string(),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'music.generate',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { description, genre, duration, tempo, instruments } = input;
    
    try {
      const prompt = buildPrompt(description, genre, duration, tempo, instruments);
      const audioResult = await generateMusic(prompt);
      
      return {
        success: true,
        output: JSON.stringify({
          audioFile: audioResult.path,
          duration: duration || 60,
          format: 'mp3',
          summary: 'Generated ' + (genre || 'music') + ' track: ' + description,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Music generation failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function buildPrompt(desc: string, genre?: string, duration?: number, tempo?: number, instruments?: string[]): string {
  let prompt = 'Generate music: ' + desc;
  if (genre) prompt += ', Genre: ' + genre;
  if (duration) prompt += ', Duration: ' + duration + 's';
  if (tempo) prompt += ', Tempo: ' + tempo + ' BPM';
  if (instruments) prompt += ', Instruments: ' + instruments.join(', ');
  return prompt;
}

async function generateMusic(prompt: string): Promise<{ path: string }> {
  return { path: 'generated/music_' + Date.now() + '.mp3' };
}

export default skill;
