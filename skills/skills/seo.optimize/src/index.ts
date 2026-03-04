import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  content: z.string().describe('Content to optimize'),
  keywords: z.array(z.string()).describe('Target keywords'),
  url: z.string().optional().describe('Page URL'),
});

const OutputSchema = z.object({
  optimizedContent: z.string(),
  score: z.number(),
  suggestions: z.array(z.string()),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'seo.optimize',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { content, keywords, url } = input;
    
    try {
      const result = analyzeAndOptimize(content, keywords);
      
      return {
        success: true,
        output: JSON.stringify(result),
      };
    } catch (error) {
      return {
        success: false,
        error: 'SEO optimization failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function analyzeAndOptimize(content: string, keywords: string[]) {
  const optimizedContent = content;
  const keywordCount = keywords.length;
  const score = Math.min(100, 50 + keywordCount * 10);
  
  const suggestions: string[] = [];
  if (score < 80) {
    suggestions.push('Add more keywords to the content');
    suggestions.push('Include keywords in headings');
  }
  if (!content.toLowerCase().includes(keywords[0] || '')) {
    suggestions.push('Add primary keyword in first paragraph');
  }
  
  return {
    optimizedContent,
    score,
    suggestions,
    summary: 'SEO score: ' + score + '/100',
  };
}

export default skill;
