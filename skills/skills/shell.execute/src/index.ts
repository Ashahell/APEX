import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  command: z.string().describe('Shell command to execute'),
  cwd: z.string().optional().describe('Working directory'),
  timeout: z.number().optional().default(30).describe('Timeout in seconds'),
});

const OutputSchema = z.object({
  stdout: z.string(),
  stderr: z.string(),
  exitCode: z.number(),
  timedOut: z.boolean(),
});

const BLOCKED_COMMANDS = [
  'rm -rf /',
  'mkfs',
  'dd if=/dev/zero',
  ':(){:|:&};:',
  'chmod -R 777',
  'wget | sh',
  'curl | sh',
];

export const skill: Skill = {
  name: 'shell.execute',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { command, cwd, timeout } = input;
    
    if (ctx.tier !== 'T2' && ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T2 permission required to execute shell commands',
      };
    }
    
    for (const blocked of BLOCKED_COMMANDS) {
      if (command.toLowerCase().includes(blocked.toLowerCase())) {
        return {
          success: false,
          error: `Command blocked: potentially dangerous command detected`,
        };
      }
    }
    
    try {
      const result = await executeCommand(command, cwd, timeout);
      
      return {
        success: result.exitCode === 0,
        output: JSON.stringify(result),
        artifacts: result.exitCode !== 0 ? undefined : [],
      };
    } catch (error) {
      return {
        success: false,
        error: `Command execution failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

interface CommandResult {
  stdout: string;
  stderr: string;
  exitCode: number;
  timedOut: boolean;
}

async function executeCommand(
  command: string,
  cwd?: string,
  timeout: number = 30
): Promise<CommandResult> {
  const { exec } = await import('child_process');
  
  return new Promise((resolve) => {
    const options = cwd ? { cwd } : {};
    const proc = exec(command, options, (error, stdout, stderr) => {
      resolve({
        stdout: stdout || '',
        stderr: stderr || '',
        exitCode: error?.code || 0,
        timedOut: false,
      });
    });
    
    setTimeout(() => {
      proc.kill();
      resolve({
        stdout: '',
        stderr: 'Command timed out',
        exitCode: 124,
        timedOut: true,
      });
    }, timeout * 1000);
  });
}

export default skill;
