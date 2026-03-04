import { z } from 'zod';
import { exec } from 'child_process';
import { promisify } from 'util';
import type { Skill, SkillContext, SkillResult } from '../../types.js';
import { checkGit } from '../../../src/utils.js';

const execAsync = promisify(exec);

const InputSchema = z.object({
  message: z.string().describe('Commit message'),
  files: z.array(z.string()).optional().describe('Files to stage (default: all)'),
  push: z.boolean().optional().default(false).describe('Push after commit'),
  repoPath: z.string().optional().describe('Repository path'),
});

const OutputSchema = z.object({
  commitHash: z.string(),
  message: z.string(),
  pushed: z.boolean(),
});

export const skill: Skill = {
  name: 'git.commit',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    if (ctx.tier !== 'T2' && ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T2 permission required to commit to git',
      };
    }
    
    const { message, files, push, repoPath } = input;
    const options = repoPath ? { cwd: repoPath } : {};
    
    try {
      const stagedFiles = files || ['.'];
      await execAsync(`git add ${stagedFiles.join(' ')}`, options);
      
      const { stdout } = await execAsync(`git commit -m "${message}"`, options);
      
      let commitHash = '';
      try {
        const { stdout: hashOutput } = await execAsync('git rev-parse HEAD', options);
        commitHash = hashOutput.trim().substring(0, 7);
      } catch {
        commitHash = 'unknown';
      }
      
      let pushed = false;
      if (push) {
        await execAsync('git push', options);
        pushed = true;
      }
      
      return {
        success: true,
        output: JSON.stringify({
          commitHash,
          message,
          pushed,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Git commit failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return checkGit();
  },
};

export default skill;
