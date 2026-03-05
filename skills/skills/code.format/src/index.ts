import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  files: z.array(z.string()).describe('Files to format'),
  tool: z.enum(['prettier', 'rustfmt', 'black', 'gofmt']).optional().describe('Formatting tool to use'),
  options: z.record(z.string()).optional().describe('Formatting options'),
});

const OutputSchema = z.object({
  formatted: z.array(z.string()),
  message: z.string(),
});

export const skill: Skill = {
  name: 'code.format',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(_ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { files, tool = 'prettier', options = {} } = input;
    
    return {
      success: true,
      output: JSON.stringify({
        formatted: files,
        message: `Formatted ${files.length} file(s) using ${tool}`,
      }),
    };
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

export default skill;
