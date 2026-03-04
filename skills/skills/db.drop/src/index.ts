import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  connectionString: z.string().describe('Database connection string'),
  target: z.enum(['table', 'database', 'schema']).describe('What to drop'),
  name: z.string().describe('Name of table/database/schema to drop'),
  cascade: z.boolean().optional().default(false).describe('Use CASCADE for drop'),
  ifExists: z.boolean().optional().default(true).describe('Use IF EXISTS'),
});

const OutputSchema = z.object({
  success: z.boolean(),
  target: z.string(),
  name: z.string(),
  sql: z.string(),
});

const BLOCKED_DATABASES = [
  'information_schema',
  'performance_schema',
  'mysql',
  'sys',
  'postgres',
  'template0',
  'template1',
];

export const skill: Skill = {
  name: 'db.drop',
  version: '0.1.0',
  tier: 'T3',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    if (ctx.tier !== 'T3') {
      return {
        success: false,
        error: 'T3 permission required to drop database objects',
      };
    }

    const { connectionString, target, name, cascade, ifExists } = input;

    if (target === 'database') {
      const lowerName = name.toLowerCase();
      for (const blocked of BLOCKED_DATABASES) {
        if (lowerName === blocked || lowerName.includes(blocked)) {
          return {
            success: false,
            error: `Cannot drop protected database: ${blocked}`,
          };
        }
      }
    }

    try {
      const ifExistsClause = ifExists ? 'IF EXISTS ' : '';
      const cascadeClause = cascade ? ' CASCADE' : '';

      let sql = '';
      switch (target) {
        case 'table':
          sql = `DROP TABLE ${ifExistsClause}"${name}"${cascadeClause}`;
          break;
        case 'database':
          sql = `DROP DATABASE ${ifExistsClause}"${name}"${cascadeClause}`;
          break;
        case 'schema':
          sql = `DROP SCHEMA ${ifExistsClause}"${name}"${cascadeClause}`;
          break;
      }

      const { default: pg } = await import('pg');
      const { Pool } = pg;

      const pool = new Pool({ connectionString });
      await pool.query(sql);
      await pool.end();

      return {
        success: true,
        output: JSON.stringify({
          success: true,
          target,
          name,
          sql,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Drop failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

export default skill;
