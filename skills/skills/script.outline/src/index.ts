import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  topic: z.string().describe('Topic for the script'),
  type: z.enum(['blog', 'youtube', 'podcast', 'marketing', 'educational']).describe('Script type'),
  duration: z.number().optional().describe('Approximate duration in minutes'),
  tone: z.string().optional().describe('Tone (humorous, professional, casual, etc.)'),
});

const OutputSchema = z.object({
  outline: z.array(z.object({
    section: z.string(),
    keyPoints: z.array(z.string()),
    duration: z.number(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'script.outline',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { topic, type, duration, tone } = input;
    
    try {
      const outline = generateOutline(topic, type, duration, tone);
      
      return {
        success: true,
        output: JSON.stringify({
          outline,
          summary: 'Generated ' + type + ' outline for: ' + topic,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Outline generation failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function generateOutline(topic: string, type: string, duration?: number, tone?: string) {
  const sections = [
    { section: 'Introduction', keyPoints: ['Hook', 'Topic overview'], duration: 1 },
    { section: 'Main Content', keyPoints: ['Point 1', 'Point 2', 'Point 3'], duration: duration ? duration - 2 : 3 },
    { section: 'Conclusion', keyPoints: ['Summary', 'Call to action'], duration: 1 },
  ];
  return sections;
}

export default skill;
