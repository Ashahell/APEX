import { describe, it, expect, vi, beforeEach } from 'vitest';
import { SkillLoader } from './loader';
import { Skill, SkillContext, SkillResult, PermissionTier } from './types';
import { z } from 'zod';

const mockSkill: Skill = {
  name: 'test.skill',
  version: '1.0.0',
  tier: 'T1' as PermissionTier,
  inputSchema: z.object({
    content: z.string(),
  }),
  outputSchema: z.object({
    success: z.boolean(),
    output: z.string().optional(),
  }),
  execute: vi.fn().mockResolvedValue({
    success: true,
    output: 'test output',
  }),
  healthCheck: vi.fn().mockResolvedValue(true),
};

describe('SkillLoader', () => {
  let loader: SkillLoader;

  beforeEach(() => {
    loader = new SkillLoader();
  });

  describe('get', () => {
    it('should return undefined for non-existent skill', () => {
      const skill = loader.get('nonexistent');
      expect(skill).toBeUndefined();
    });
  });

  describe('getAll', () => {
    it('should return empty array when no skills loaded', () => {
      const skills = loader.getAll();
      expect(skills).toEqual([]);
    });
  });

  describe('execute', () => {
    it('should return error for non-existent skill', async () => {
      const ctx: SkillContext = {
        taskId: 'task-1',
        userId: 'user-1',
        workspacePath: '/tmp/workspace',
        tier: 'T1',
      };

      const result = await loader.execute('nonexistent', ctx, { content: 'test' });

      expect(result.success).toBe(false);
      expect(result.error).toContain('Skill not found');
    });
  });

  describe('healthCheck', () => {
    it('should return empty map when no skills loaded', async () => {
      const results = await loader.healthCheck();
      expect(results.size).toBe(0);
    });
  });
});

describe('Skill types', () => {
  describe('SkillContext', () => {
    it('should have correct shape', () => {
      const ctx: SkillContext = {
        taskId: 'task-123',
        userId: 'user-456',
        workspacePath: '/workspace/project',
        tier: 'T2',
      };

      expect(ctx.taskId).toBe('task-123');
      expect(ctx.userId).toBe('user-456');
      expect(ctx.workspacePath).toBe('/workspace/project');
      expect(ctx.tier).toBe('T2');
    });
  });

  describe('SkillResult', () => {
    it('should have correct shape for success', () => {
      const result: SkillResult = {
        success: true,
        output: 'Hello world',
        artifacts: [
          {
            path: '/tmp/file.txt',
            mimeType: 'text/plain',
            content: 'file content',
          },
        ],
      };

      expect(result.success).toBe(true);
      expect(result.output).toBe('Hello world');
      expect(result.artifacts).toHaveLength(1);
    });

    it('should have correct shape for error', () => {
      const result: SkillResult = {
        success: false,
        error: 'Something went wrong',
      };

      expect(result.success).toBe(false);
      expect(result.error).toBe('Something went wrong');
    });
  });

  describe('PermissionTier', () => {
    it('should allow all tier values', () => {
      const tiers: PermissionTier[] = ['T0', 'T1', 'T2', 'T3'];

      for (const tier of tiers) {
        const ctx: SkillContext = {
          taskId: 'task-1',
          userId: 'user-1',
          workspacePath: '/tmp',
          tier,
        };
        expect(ctx.tier).toBe(tier);
      }
    });
  });
});
