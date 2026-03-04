import { readdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { Skill, SkillContext, SkillResult, CapabilityToken, PermissionTier } from './types.js';
import Pino from 'pino';

const logger = Pino({ name: 'apex-skills' });

const TIER_ORDER: Record<PermissionTier, number> = {
  'T0': 0,
  'T1': 1,
  'T2': 2,
  'T3': 3,
};

function base64Decode(str: string): string {
  const buffer = Buffer.from(str, 'base64');
  return buffer.toString('utf-8');
}

export function decodeCapabilityToken(token: string): CapabilityToken | null {
  try {
    const json = base64Decode(token);
    return JSON.parse(json) as CapabilityToken;
  } catch (error) {
    logger.error({ error }, 'Failed to decode capability token');
    return null;
  }
}

export function verifyCapabilityToken(
  token: CapabilityToken,
  skillName: string,
  requiredTier: PermissionTier
): { valid: boolean; error?: string } {
  if (TIER_ORDER[token.tier] < TIER_ORDER[requiredTier]) {
    return { 
      valid: false, 
      error: `Skill ${skillName} requires ${requiredTier} but token has ${token.tier}` 
    };
  }

  const now = Date.now();
  const expiresAt = new Date(token.expires_at).getTime();
  if (now > expiresAt) {
    return { valid: false, error: 'Capability token has expired' };
  }

  const skillAllowed = token.allowed_skills.some(
    s => s === skillName || s === '*'
  );
  if (!skillAllowed) {
    return { 
      valid: false, 
      error: `Skill ${skillName} not in allowed_skills` 
    };
  }

  return { valid: true };
}

export class SkillLoader {
  private skills: Map<string, Skill> = new Map();

  async loadFromDirectory(dirPath: string): Promise<void> {
    logger.info({ path: dirPath }, 'Loading skills from directory');
    
    const entries = readdirSync(dirPath, { withFileTypes: true });
    
    for (const entry of entries) {
      if (!entry.isDirectory()) continue;
      
      const skillPath = join(dirPath, entry.name);
      await this.loadSkill(skillPath);
    }
    
    logger.info({ count: this.skills.size }, 'Skills loaded');
  }

  private async loadSkill(skillPath: string): Promise<void> {
    try {
      const skillFile = join(skillPath, 'src', 'index.ts');
      const fileUrl = 'file:///' + skillFile.replace(/\\/g, '/');
      
      const module = await import(fileUrl);
      const skill: Skill = module.default || module.skill;
      
      if (!skill || !skill.name) {
        logger.warn({ path: skillPath }, 'Invalid skill module');
        return;
      }
      
      this.skills.set(skill.name, skill);
      logger.info({ skill: skill.name, version: skill.version }, 'Loaded skill');
      
    } catch (error) {
      logger.error({ error, path: skillPath }, 'Failed to load skill');
    }
  }

  get(name: string): Skill | undefined {
    return this.skills.get(name);
  }

  getAll(): Skill[] {
    return Array.from(this.skills.values());
  }

  async execute(name: string, ctx: SkillContext, input: unknown): Promise<SkillResult> {
    const skill = this.skills.get(name);
    
    if (!skill) {
      return {
        success: false,
        error: `Skill not found: ${name}`,
      };
    }
    
    if (ctx.capabilityToken) {
      const decoded = decodeCapabilityToken(ctx.capabilityToken);
      if (!decoded) {
        return {
          success: false,
          error: 'Invalid capability token format',
        };
      }
      
      const verification = verifyCapabilityToken(decoded, name, skill.tier);
      if (!verification.valid) {
        logger.warn({ skill: name, error: verification.error }, 'Capability token verification failed');
        return {
          success: false,
          error: verification.error,
        };
      }
    } else if (ctx.tier) {
      const tierCheck = TIER_ORDER[ctx.tier] >= TIER_ORDER[skill.tier];
      if (!tierCheck) {
        return {
          success: false,
          error: `Skill ${name} requires ${skill.tier} but context has ${ctx.tier}`,
        };
      }
    }
    
    try {
      const validatedInput = skill.inputSchema.parse(input);
      return await skill.execute(ctx, validatedInput);
    } catch (error) {
      return {
        success: false,
        error: `Skill execution failed: ${error}`,
      };
    }
  }

  async healthCheck(): Promise<Map<string, boolean>> {
    const results = new Map<string, boolean>();
    
    for (const [name, skill] of this.skills) {
      try {
        results.set(name, await skill.healthCheck());
      } catch (error) {
        results.set(name, false);
      }
    }
    
    return results;
  }
}
