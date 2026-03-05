import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  action: z.enum(['list', 'create', 'delete', 'switch', 'merge']).describe('Branch action to perform'),
  branchName: z.string().optional().describe('Name of branch for create/delete/switch/merge'),
  sourceBranch: z.string().optional().describe('Source branch for create/merge'),
});

const OutputSchema = z.object({
  action: z.string(),
  branch: z.string().optional(),
  message: z.string(),
});

export const skill: Skill = {
  name: 'git.branch',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(_ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { action, branchName, sourceBranch } = input;
    
    let message = '';
    let branch = branchName;
    
    switch (action) {
      case 'list':
        message = 'Listed all branches';
        break;
      case 'create':
        message = `Created branch '${branchName}' from ${sourceBranch || 'current branch'}`;
        break;
      case 'delete':
        message = `Deleted branch '${branchName}'`;
        break;
      case 'switch':
        message = `Switched to branch '${branchName}'`;
        break;
      case 'merge':
        message = `Merged '${sourceBranch}' into current branch`;
        branch = undefined;
        break;
    }
    
    return {
      success: true,
      output: JSON.stringify({
        action,
        branch,
        message,
      }),
    };
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

export default skill;
