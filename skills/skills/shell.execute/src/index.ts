import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';
import { checkCommandExists } from '../../../src/utils.js';

const InputSchema = z.object({
  command: z.string().describe('Shell command to execute'),
  cwd: z.string().optional().describe('Working directory'),
  timeout: z.number().optional().default(30).describe('Timeout in seconds'),
}).refine(
  (data) => data.timeout <= 300,
  { message: "Timeout cannot exceed 300 seconds" }
);

const OutputSchema = z.object({
  stdout: z.string(),
  stderr: z.string(),
  exitCode: z.number(),
  timedOut: z.boolean(),
});

const BLOCKED_COMMANDS = [
  'rm -rf /',
  'rm -rf /*',
  'mkfs',
  'dd if=/dev/zero',
  ':(){:|:&};:',
  'chmod -R 777',
  'wget | sh',
  'curl | sh',
  'sudo rm',
  '> /dev/sda',
  'dd of=/dev/sd',
];

const BLOCKED_PATTERNS = [
  /\$\(.*\)/,  // Command substitution
  /`.*`/,      // Backtick substitution
  /\|\s*sh$/,  // Pipe to shell
  /\s\&\s*$/,  // Background execution
  /;\s*rm\s+/, // Command chaining with rm
  /&&\s*rm\s+/, // AND with rm
  /\|\|\s*rm\s+/, // OR with rm
];

const DANGEROUS_PATHS = [
  '/etc/passwd',
  '/etc/shadow',
  '/etc/sudoers',
  '/.ssh/',
  '/.aws/',
  '/.env',
];

function validateCommand(command: string): { valid: boolean; error?: string } {
  // Check blocked commands
  for (const blocked of BLOCKED_COMMANDS) {
    if (command.toLowerCase().includes(blocked.toLowerCase())) {
      return { valid: false, error: `Blocked command pattern: ${blocked}` };
    }
  }
  
  // Check blocked patterns
  for (const pattern of BLOCKED_PATTERNS) {
    if (pattern.test(command)) {
      return { valid: false, error: 'Command contains dangerous pattern (command substitution, pipe to shell, etc.)' };
    }
  }
  
  // Check for dangerous path access
  for (const path of DANGEROUS_PATHS) {
    if (command.includes(path)) {
      return { valid: false, error: `Command attempts to access protected path: ${path}` };
    }
  }
  
  return { valid: true };
}

export const skill: Skill = {
  name: 'shell.execute',
  version: '0.1.0',
  tier: 'T3',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { command, cwd, timeout } = input;
    
    if (ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T3 permission required to execute shell commands',
      };
    }
    
    const validation = validateCommand(command);
    if (!validation.valid) {
      return {
        success: false,
        error: `Command blocked: ${validation.error}`,
      };
    }
    
    // Validate cwd if provided
    if (cwd) {
      const pathTraversal = cwd.includes('..');
      if (pathTraversal) {
        return {
          success: false,
          error: 'Path traversal not allowed in cwd',
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
    return checkCommandExists('sh');
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
