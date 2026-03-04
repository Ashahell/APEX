import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  filePaths: z.array(z.string()).describe('Array of file paths to refactor'),
  goal: z.string().describe('Goal of refactoring (e.g., "improve performance", "reduce complexity")'),
  language: z.string().describe('Programming language'),
  constraints: z.string().optional().describe('Additional constraints or requirements'),
});

const OutputSchema = z.object({
  refactoredFiles: z.array(z.object({
    path: z.string(),
    originalContent: z.string(),
    newContent: z.string(),
    changes: z.string(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'code.refactor',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { filePaths, goal, language, constraints } = input;
    
    const prompt = buildPrompt(filePaths, goal, language, constraints);
    
    try {
      const refactoredCode = await callLLM(prompt);
      
      return {
        success: true,
        output: JSON.stringify({
          refactoredFiles: filePaths.map((path, i) => ({
            path,
            originalContent: '// Original content here',
            newContent: refactoredCode,
            changes: `Refactored to ${goal}`,
          })),
          summary: `Refactored ${filePaths.length} files for: ${goal}`,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Refactoring failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function buildPrompt(filePaths: string[], goal: string, language: string, constraints?: string): string {
  let prompt = `Refactor the following ${language} files to ${goal}:\n\nFiles: ${filePaths.join(', ')}`;
  if (constraints) prompt += `\nConstraints: ${constraints}`;
  return prompt;
}

async function callLLM(prompt: string): Promise<string> {
  return `// Refactored code for: ${prompt}\n// Improved structure here`;
}

export default skill;
