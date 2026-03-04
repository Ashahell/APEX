import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../../src/types.js';

const InputSchema = z.object({
  command: z.enum(['apply', 'delete', 'get', 'describe', 'logs', 'exec', 'scale', 'rollout'])
    .describe('kubectl command to execute'),
  resource: z.string().describe('Kubernetes resource (e.g., deployment, service, pod)'),
  name: z.string().optional().describe('Specific resource name'),
  namespace: z.string().optional().default('default').describe('Kubernetes namespace'),
  manifest: z.string().optional().describe('YAML manifest for apply'),
  options: z.string().optional().describe('Additional kubectl options'),
});

const OutputSchema = z.object({
  success: z.boolean(),
  command: z.string(),
  output: z.string(),
  error: z.string().optional(),
});

const DANGEROUS_COMMANDS = ['delete', 'scale', 'rollout restart'];

export const skill: Skill = {
  name: 'deploy.kubectl',
  version: '0.1.0',
  tier: 'T3',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    if (ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T3 permission required for Kubernetes deployments',
      };
    }

    const { command, resource, name, namespace, manifest, options } = input;

    if (DANGEROUS_COMMANDS.includes(command) && ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'Dangerous kubectl commands require T3 permission',
      };
    }

    try {
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);

      let kubectlCmd = `kubectl ${command} ${resource}`;

      if (name) {
        kubectlCmd += ` ${name}`;
      }

      if (namespace && namespace !== 'default') {
        kubectlCmd += ` -n ${namespace}`;
      }

      if (options) {
        kubectlCmd += ` ${options}`;
      }

      if (command === 'apply' && manifest) {
        const fs = await import('fs/promises');
        const tmpFile = `/tmp/kube-manifest-${Date.now()}.yaml`;
        await fs.writeFile(tmpFile, manifest);
        kubectlCmd += ` -f ${tmpFile}`;
      }

      const { stdout, stderr } = await execAsync(kubectlCmd).catch((err) => ({
        stdout: '',
        stderr: err.message,
      }));

      const success = !stderr || !stderr.includes('Error');

      return {
        success,
        output: JSON.stringify({
          success,
          command: kubectlCmd,
          output: stdout || stderr,
          error: success ? undefined : stderr,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `kubectl execution failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    try {
      const { exec } = await import('child_process');
      const { promisify } = await import('util');
      const execAsync = promisify(exec);
      await execAsync('kubectl version --client');
      return true;
    } catch {
      return false;
    }
  },
};

export default skill;
