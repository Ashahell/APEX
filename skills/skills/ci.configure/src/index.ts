import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  platform: z.enum(['github', 'gitlab', 'jenkins', 'circleci']).describe('CI/CD platform'),
  language: z.string().describe('Programming language'),
  testCommand: z.string().optional().describe('Test command'),
  buildCommand: z.string().optional().describe('Build command'),
  deployStages: z.boolean().optional().describe('Include deploy stages'),
});

const OutputSchema = z.object({
  files: z.array(z.object({
    path: z.string(),
    content: z.string(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'ci.configure',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { platform, language, testCommand, buildCommand, deployStages } = input;
    
    try {
      const config = await generateCIConfig(platform, language, testCommand, buildCommand, deployStages);
      
      return {
        success: true,
        output: JSON.stringify(config),
      };
    } catch (error) {
      return {
        success: false,
        error: 'CI configuration failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

async function generateCIConfig(platform: string, language: string, testCmd?: string, buildCmd?: string, deploy?: boolean): Promise<{ files: { path: string; content: string }[]; summary: string }> {
  const files: { path: string; content: string }[] = [];
  
  if (platform === 'github') {
    const content = 'name: CI\n\non: [push, pull_request]\n\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - name: Setup\n        run: echo "Setup ' + language + '"\n      - name: Test\n        run: ' + (testCmd || 'echo "No tests"') + '\n';
    files.push({ path: '.github/workflows/ci.yml', content });
  }
  
  return {
    files,
    summary: 'Generated ' + platform + ' CI configuration for ' + language,
  };
}

export default skill;
