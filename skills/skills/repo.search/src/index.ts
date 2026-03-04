import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  query: z.string().describe('Search query'),
  language: z.string().optional().describe('Filter by programming language'),
  path: z.string().optional().describe('Search within specific path'),
  maxResults: z.number().optional().describe('Maximum results to return'),
});

const OutputSchema = z.object({
  results: z.array(z.object({
    file: z.string(),
    line: z.number(),
    content: z.string(),
    score: z.number(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'repo.search',
  version: '0.1.0',
  tier: 'T0',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { query, language, path, maxResults } = input;
    
    try {
      const results = await searchCode(query, language, path, maxResults || 10);
      
      return {
        success: true,
        output: JSON.stringify({
          results,
          summary: 'Found ' + results.length + ' results for: ' + query,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Search failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

async function searchCode(query: string, language?: string, path?: string, maxResults?: number): Promise<{ file: string; line: number; content: string; score: number }[]> {
  return [
    { file: 'src/index.ts', line: 10, content: '// Code matching: ' + query, score: 0.9 },
  ];
}

export default skill;
