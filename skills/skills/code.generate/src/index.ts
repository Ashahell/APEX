import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  language: z.string().describe('Programming language'),
  description: z.string().describe('Description of code to generate'),
  framework: z.string().optional().describe('Framework or library to use'),
  tests: z.boolean().optional().describe('Include test file'),
});

const OutputSchema = z.object({
  files: z.array(z.object({
    path: z.string(),
    content: z.string(),
  })),
  explanation: z.string(),
});

export const skill: Skill = {
  name: 'code.generate',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { language, description, framework, tests } = input;
    
    const prompt = buildPrompt(language, description, framework, tests);
    
    try {
      const generatedCode = await callLLM(prompt);
      
      return {
        success: true,
        output: JSON.stringify({
          files: [
            { path: `generated/main.${getExtension(language)}`, content: generatedCode },
            ...(tests ? [{ path: 'generated/main.test.' + getExtension(language), content: '# Tests' }] : []),
          ],
          explanation: `Generated ${language} code for: ${description}`,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Code generation failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

function getExtension(language: string): string {
  const extensions: Record<string, string> = {
    javascript: 'js',
    typescript: 'ts',
    python: 'py',
    rust: 'rs',
    go: 'go',
  };
  return extensions[language.toLowerCase()] || 'txt';
}

function buildPrompt(language: string, description: string, framework?: string, tests?: boolean): string {
  let prompt = `Write ${language} code that ${description}.`;
  if (framework) prompt += ` Use ${framework}.`;
  if (tests) prompt += ' Include tests.';
  return prompt;
}

async function callLLM(prompt: string): Promise<string> {
  return `// Generated code for: ${prompt}\nconsole.log("Hello, World!");`;
}

export default skill;
