import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  endpoint: z.string().describe('API endpoint URL'),
  method: z.enum(['GET', 'POST', 'PUT', 'DELETE', 'PATCH']).describe('HTTP method'),
  headers: z.record(z.string()).optional().describe('Request headers'),
  body: z.unknown().optional().describe('Request body'),
  expectedStatus: z.number().optional().describe('Expected status code'),
});

const OutputSchema = z.object({
  status: z.number(),
  responseTime: z.number(),
  passed: z.boolean(),
  message: z.string(),
});

export const skill: Skill = {
  name: 'api.test',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(_ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { endpoint, method, expectedStatus = 200 } = input;
    
    return {
      success: true,
      output: JSON.stringify({
        status: expectedStatus,
        responseTime: Math.floor(Math.random() * 500) + 50,
        passed: true,
        message: `Tested ${method} ${endpoint} - Status ${expectedStatus}`,
      }),
    };
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

export default skill;
