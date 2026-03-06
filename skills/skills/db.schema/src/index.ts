import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

interface TableDefinition {
  name: string;
  columns: {
    name: string;
    type: string;
    nullable: boolean;
    primaryKey?: boolean;
    references?: { table: string; column: string };
  }[];
}

const InputSchema = z.object({
  tables: z.array(z.object({
    name: z.string(),
    columns: z.array(z.object({
      name: z.string(),
      type: z.string(),
      nullable: z.boolean(),
      primaryKey: z.boolean().optional(),
      references: z.object({
        table: z.string(),
        column: z.string(),
      }).optional(),
    })),
  })),
  database: z.enum(['postgresql', 'mysql', 'sqlite', 'mongodb']).describe('Target database'),
});

const OutputSchema = z.object({
  schema: z.string(),
  entities: z.array(z.object({
    name: z.string(),
    sql: z.string(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'db.schema',
  version: '0.1.0',
  tier: 'T1',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { tables, database } = input;
    
    try {
      const schema = await designSchema(tables, database);
      
      return {
        success: true,
        output: JSON.stringify(schema),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Schema design failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    // db.schema generates SQL locally, no external dependencies needed
    return true;
  },
};

async function designSchema(tables: TableDefinition[], database: string): Promise<{ schema: string; entities: { name: string; sql: string }[]; summary: string }> {
  const entities: { name: string; sql: string }[] = [];
  
  for (const table of tables) {
    let sql = 'CREATE TABLE ' + table.name + ' (\n';
    const cols: string[] = [];
    
    for (const col of table.columns) {
      let colDef = '  ' + col.name + ' ' + col.type;
      if (col.primaryKey) colDef += ' PRIMARY KEY';
      if (!col.nullable) colDef += ' NOT NULL';
      cols.push(colDef);
    }
    
    sql += cols.join(',\n') + '\n);';
    entities.push({ name: table.name, sql });
  }
  
  return {
    schema: entities.map(e => e.sql).join('\n\n'),
    entities,
    summary: 'Designed ' + database + ' schema with ' + tables.length + ' tables',
  };
}

export default skill;
