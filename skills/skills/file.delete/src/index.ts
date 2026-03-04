import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  path: z.string().describe('File path to delete'),
  recursive: z.boolean().optional().default(false).describe('Delete directories recursively'),
  force: z.boolean().optional().default(false).describe('Force deletion without confirmation'),
});

const OutputSchema = z.object({
  deleted: z.boolean(),
  path: z.string(),
  size: z.number().optional(),
});

const BLOCKED_PATHS = [
  '/',
  '/home',
  '/root',
  '/etc',
  '/var',
  'C:\\',
  'D:\\',
  'E:\\projects',
];

const DANGEROUS_PATTERNS = [
  /\.ssh\/known_hosts$/,
  /\.aws\/credentials$/,
  /\.env$/,
  /\/etc\/passwd$/,
  /\/etc\/shadow$/,
];

export const skill: Skill = {
  name: 'file.delete',
  version: '0.1.0',
  tier: 'T3',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    if (ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T3 permission required to delete files',
      };
    }

    const { path, recursive, force } = input;

    for (const blocked of BLOCKED_PATHS) {
      if (path.toLowerCase().startsWith(blocked.toLowerCase())) {
        return {
          success: false,
          error: `Cannot delete protected path: ${blocked}`,
        };
      }
    }

    for (const pattern of DANGEROUS_PATTERNS) {
      if (pattern.test(path)) {
        return {
          success: false,
          error: `Cannot delete sensitive file: ${path}`,
        };
      }
    }

    try {
      const fs = await import('fs/promises');
      const pathModule = await import('path');

      const stats = await fs.stat(path).catch(() => null);
      if (!stats) {
        return {
          success: false,
          error: `File not found: ${path}`,
        };
      }

      const size = stats.size;

      if (stats.isDirectory()) {
        if (!recursive) {
          return {
            success: false,
            error: 'Use recursive=true to delete directories',
          };
        }
        await fs.rm(path, { recursive: true, force });
      } else {
        await fs.unlink(path);
      }

      return {
        success: true,
        output: JSON.stringify({
          deleted: true,
          path,
          size,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Delete failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

export default skill;
