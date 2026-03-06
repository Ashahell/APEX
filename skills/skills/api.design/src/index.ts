import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  name: z.string().describe('API name'),
  resources: z.array(z.object({
    name: z.string(),
    operations: z.array(z.enum(['GET', 'POST', 'PUT', 'PATCH', 'DELETE'])),
    fields: z.array(z.object({
      name: z.string(),
      type: z.string(),
      required: z.boolean(),
    })),
  })),
  style: z.enum(['rest', 'graphql', 'grpc']).optional().describe('API style'),
});

const OutputSchema = z.object({
  spec: z.object({
    openapi: z.string().optional(),
    schema: z.string().optional(),
  }),
  files: z.array(z.object({
    path: z.string(),
    content: z.string(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'api.design',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { name, resources, style } = input;
    
    try {
      const spec = await designApi(name, resources, style || 'rest');
      
      return {
        success: true,
        output: JSON.stringify(spec),
      };
    } catch (error) {
      return {
        success: false,
        error: 'API design failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

interface ApiResource {
  name: string;
  operations: ('GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE')[];
  fields: { name: string; type: string; required: boolean }[];
}

async function designApi(name: string, resources: ApiResource[], style: string): Promise<{ spec: Record<string, unknown>; files: { path: string; content: string }[]; summary: string }> {
  const spec: Record<string, unknown> = {
    openapi: '3.0.0',
    info: { title: name, version: '1.0.0' },
    paths: {} as Record<string, unknown>,
  };
  
  for (const resource of resources) {
    for (const op of resource.operations) {
      const path = '/' + resource.name.toLowerCase();
      (spec.paths as Record<string, unknown>)[path] = {
        [op.toLowerCase()]: {
          summary: op + ' ' + resource.name,
          responses: { '200': { description: 'OK' } },
        },
      };
    }
  }
  
  return {
    spec,
    files: [{ path: 'api spec.yaml', content: YAML.stringify(spec) }],
    summary: 'Designed ' + style + ' API with ' + resources.length + ' resources',
  };
}

const YAML = {
  stringify: (obj: any): string => JSON.stringify(obj, null, 2),
};

export default skill;
