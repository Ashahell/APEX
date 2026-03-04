import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  repository: z.string().describe('Git repository path or URL'),
  remote: z.string().optional().default('origin').describe('Remote name'),
  branch: z.string().optional().default('main').describe('Branch to force push'),
  forceWithLease: z.boolean().optional().default(true).describe('Use --force-with-lease for safety'),
  message: z.string().optional().describe('Commit message if amend is needed'),
});

const OutputSchema = z.object({
  success: z.boolean(),
  remote: z.string(),
  branch: z.string(),
  newRef: z.string().optional(),
  warning: z.string().optional(),
});

export const skill: Skill = {
  name: 'git.force_push',
  version: '0.1.0',
  tier: 'T3',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    if (ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T3 permission required for force push',
      };
    }

    const { repository, remote, branch, forceWithLease, message } = input;

    try {
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);

      const cwd = repository.startsWith('http') ? undefined : repository;

      if (message) {
        await execAsync('git add -A', { cwd });
        await execAsync(`git commit -m "${message}"`, { cwd });
      }

      const forceFlag = forceWithLease ? '--force-with-lease' : '--force';
      const command = `git push ${remote} ${branch} ${forceFlag}`;

      const { stdout, stderr } = await execAsync(command, { cwd }).catch((err) => ({
        stdout: '',
        stderr: err.message,
      }));

      if (stderr && !stderr.includes('To ')) {
        return {
          success: false,
          error: `Force push failed: ${stderr}`,
        };
      }

      const warning = forceWithLease
        ? 'Used --force-with-lease for safety. This can be overridden by someone who has recently fetched.'
        : 'WARNING: Used --force. This is dangerous and can overwrite remote changes.';

      return {
        success: true,
        output: JSON.stringify({
          success: true,
          remote,
          branch,
          warning,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Force push failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    try {
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);
      await execAsync('git --version');
      return true;
    } catch {
      return false;
    }
  },
};

export default skill;
