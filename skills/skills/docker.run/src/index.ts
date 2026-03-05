import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  image: z.string().describe('Docker image to run'),
  command: z.string().optional().describe('Command to execute'),
  ports: z.array(z.string()).optional().describe('Port mappings (host:container)'),
  env: z.record(z.string()).optional().describe('Environment variables'),
  detach: z.boolean().optional().describe('Run in detached mode'),
  name: z.string().optional().describe('Container name'),
});

const OutputSchema = z.object({
  containerId: z.string().optional(),
  message: z.string(),
  running: z.boolean(),
});

export const skill: Skill = {
  name: 'docker.run',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(_ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { image, command, ports, env, detach = false, name } = input;
    
    const containerId = `container-${Date.now()}`;
    
    return {
      success: true,
      output: JSON.stringify({
        containerId,
        message: `Running ${image}${command ? ` with command: ${command}` : ''}`,
        running: detach,
      }),
    };
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

export default skill;
