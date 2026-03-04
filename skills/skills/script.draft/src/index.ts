import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  outline: z.array(z.object({
    section: z.string(),
    keyPoints: z.array(z.string()),
    duration: z.number(),
  })).describe('Script outline from script.outline skill'),
  wordCount: z.number().optional().describe('Target word count'),
  format: z.enum(['markdown', 'plain', 'html']).optional(),
});

const OutputSchema = z.object({
  script: z.string(),
  wordCount: z.number(),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'script.draft',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { outline, wordCount, format } = input;
    
    try {
      const script = generateScript(outline, wordCount || 500, format || 'markdown');
      
      return {
        success: true,
        output: JSON.stringify({
          script,
          wordCount: script.split(/\s+/).length,
          summary: 'Drafted script with ' + outline.length + ' sections',
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Script drafting failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function generateScript(outline: any[], wordCount: number, format: string): string {
  let script = format === 'markdown' ? '# Script\n\n' : '';
  
  for (const section of outline) {
    if (format === 'markdown') {
      script += '\n## ' + section.section + '\n\n';
    }
    for (const point of section.keyPoints) {
      script += '- ' + point + '\n';
    }
  }
  
  return script;
}

export default skill;
