import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  filePaths: z.array(z.string()).describe('Source files to generate tests for'),
  language: z.string().describe('Programming language'),
  testFramework: z.string().describe('Test framework (e.g., jest, pytest, vitest)'),
  coverage: z.boolean().optional().describe('Include coverage setup'),
});

const OutputSchema = z.object({
  testFiles: z.array(z.object({
    path: z.string(),
    content: z.string(),
  })),
  runCommand: z.string(),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'code.test',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { filePaths, language, testFramework, coverage } = input;
    
    const prompt = buildPrompt(filePaths, language, testFramework, coverage);
    
    try {
      const testCode = await callLLM(prompt);
      const testExt = getTestExtension(testFramework, language);
      const testPath = `tests/test${testExt}`;
      
      return {
        success: true,
        output: JSON.stringify({
          testFiles: [
            { path: testPath, content: testCode },
          ],
          runCommand: getRunCommand(testFramework),
          summary: `Generated tests for ${filePaths.length} files using ${testFramework}`,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Test generation failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function getTestExtension(framework: string, language: string): string {
  const map: Record<string, Record<string, string>> = {
    jest: { javascript: '.test.js', typescript: '.test.ts' },
    vitest: { javascript: '.test.js', typescript: '.test.ts' },
    pytest: { python: '.py' },
    mocha: { javascript: '.test.js' },
  };
  return map[framework.toLowerCase()]?.[language.toLowerCase()] || '.test.js';
}

function getRunCommand(framework: string): string {
  const commands: Record<string, string> = {
    jest: 'npm test',
    vitest: 'npx vitest run',
    pytest: 'pytest',
    mocha: 'npm test',
  };
  return commands[framework.toLowerCase()] || 'npm test';
}

function buildPrompt(filePaths: string[], language: string, framework: string, coverage?: boolean): string {
  let prompt = `Generate ${framework} tests for ${language} files: ${filePaths.join(', ')}`;
  if (coverage) prompt += '\nInclude coverage configuration.';
  return prompt;
}

async function callLLM(prompt: string): Promise<string> {
  return `// Test file for: ${prompt}\ndescribe('tests', () => {\n  it('should work', () => {\n    expect(true).toBe(true);\n  });\n});`;
}

export default skill;
