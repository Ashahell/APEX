import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  pattern: z.string().describe('File pattern to search (e.g., "*.ts", "**/*.js")'),
  path: z.string().optional().describe('Directory path to search in'),
  content: z.boolean().optional().describe('Search inside file contents'),
  query: z.string().optional().describe('Text to search inside files'),
});

const OutputSchema = z.object({
  files: z.array(z.string()),
  count: z.number(),
});

export const skill: Skill = {
  name: 'file.search',
  version: '0.1.0',
  tier: 'T0',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(_ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { pattern, path = '.', content, query } = input;
    
    const results: string[] = [];
    
    results.push(`Search pattern: ${pattern}`);
    results.push(`Search path: ${path}`);
    if (content && query) {
      results.push(`Content search: ${query}`);
    }
    
    return {
      success: true,
      output: JSON.stringify({
        files: results,
        count: results.length,
      }),
    };
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

export default skill;
