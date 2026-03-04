import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  imageName: z.string().describe('Docker image name'),
  language: z.string().describe('Programming language'),
  port: z.number().optional().describe('Exposed port'),
  entrypoint: z.string().optional().describe('Entrypoint command'),
  multiStage: z.boolean().optional().describe('Use multi-stage build'),
});

const OutputSchema = z.object({
  files: z.array(z.object({
    path: z.string(),
    content: z.string(),
  })),
  buildCommand: z.string(),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'docker.build',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { imageName, language, port, entrypoint, multiStage } = input;
    
    try {
      const dockerfile = await generateDockerfile(imageName, language, port, entrypoint, multiStage);
      
      return {
        success: true,
        output: JSON.stringify(dockerfile),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Dockerfile generation failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

async function generateDockerfile(imageName: string, language: string, port?: number, entrypoint?: string, multiStage?: boolean): Promise<{ files: { path: string; content: string }[]; buildCommand: string; summary: string }> {
  let content = 'FROM node:20-alpine\n\nWORKDIR /app\n\nCOPY package*.json ./\nRUN npm ci --only=production\n\nCOPY . .\n';
  
  if (port) {
    content += 'EXPOSE ' + port + '\n';
  }
  
  if (entrypoint) {
    content += 'CMD [' + entrypoint + ']\n';
  } else {
    content += 'CMD ["node", "index.js"]\n';
  }
  
  return {
    files: [
      { path: 'Dockerfile', content },
      { path: '.dockerignore', content: 'node_modules\n.git\n' },
    ],
    buildCommand: 'docker build -t ' + imageName + ' .',
    summary: 'Generated Dockerfile for ' + language + ' application',
  };
}

export default skill;
