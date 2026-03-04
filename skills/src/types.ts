import { z } from 'zod';

export type PermissionTier = 'T0' | 'T1' | 'T2' | 'T3';

export interface CapabilityToken {
  task_id: string;
  tier: PermissionTier;
  allowed_skills: string[];
  allowed_domains: string[];
  expires_at: string;
  max_cost_usd: number;
}

export interface SkillContext {
  taskId: string;
  userId: string;
  workspacePath: string;
  tier: PermissionTier;
  capabilityToken?: string;
}

export interface SkillResult {
  success: boolean;
  output?: string;
  error?: string;
  artifacts?: SkillArtifact[];
}

export interface SkillArtifact {
  path: string;
  mimeType: string;
  content?: string;
}

export interface Skill {
  readonly name: string;
  readonly version: string;
  readonly tier: PermissionTier;
  readonly inputSchema: z.ZodSchema;
  readonly outputSchema: z.ZodSchema;
  
  execute(ctx: SkillContext, input: unknown): Promise<SkillResult>;
  healthCheck(): Promise<boolean>;
}
