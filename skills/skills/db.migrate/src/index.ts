import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

const InputSchema = z.object({
  currentSchema: z.string().describe('Current database schema'),
  desiredSchema: z.string().describe('Desired database schema'),
  database: z.enum(['postgresql', 'mysql', 'sqlite']).describe('Target database'),
});

const OutputSchema = z.object({
  migrations: z.array(z.object({
    up: z.string(),
    down: z.string(),
  })),
  summary: z.string(),
});

export const skill: Skill = {
  name: 'db.migrate',
  version: '0.1.0',
  tier: 'T2',
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { currentSchema, desiredSchema, database } = input;
    
    try {
      const migrations = await generateMigrations(currentSchema, desiredSchema, database);
      
      return {
        success: true,
        output: JSON.stringify({
          migrations,
          summary: 'Generated ' + migrations.length + ' migration(s) for ' + database,
        }),
      };
    } catch (error) {
      return {
        success: false,
        error: 'Migration generation failed: ' + error,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    return true;
  },
};

async function generateMigrations(current: string, desired: string, database: string): Promise<{ up: string; down: string }[]> {
  // Simple migration generator - parses schema diffs and generates ALTER statements
  const currentTables = parseSchema(current);
  const desiredTables = parseSchema(desired);
  const migrations: { up: string; down: string }[] = [];

  // Find new tables
  for (const [tableName, desiredCols] of Object.entries(desiredTables)) {
    if (!currentTables[tableName]) {
      // Create new table
      const createSql = generateCreateTable(tableName, desiredCols, database);
      migrations.push({
        up: createSql,
        down: `DROP TABLE IF EXISTS "${tableName}";`,
      });
    } else {
      // Find new columns
      const currentCols = currentTables[tableName];
      for (const [colName, colType] of Object.entries(desiredCols)) {
        if (!currentCols[colName]) {
          migrations.push({
            up: `ALTER TABLE "${tableName}" ADD COLUMN "${colName}" ${colType};`,
            down: `ALTER TABLE "${tableName}" DROP COLUMN "${colName}";`,
          });
        }
      }
    }
  }

  // If no migrations generated, return a placeholder
  if (migrations.length === 0) {
    return [
      {
        up: '-- Schemas are in sync. No migration needed.',
        down: '-- No migration to rollback.',
      },
    ];
  }

  return migrations;
}

function parseSchema(schema: string): Record<string, Record<string, string>> {
  const tables: Record<string, Record<string, string>> = {};
  const tableRegex = /CREATE TABLE (\w+)\s*\(([\s\S]*?)\);/gi;
  let match;
  
  while ((match = tableRegex.exec(schema)) !== null) {
    const tableName = match[1];
    const columnsStr = match[2];
    const columns: Record<string, string> = {};
    
    const colRegex = /(\w+)\s+(\w+(?:\(\d+\))?)/gi;
    let colMatch;
    
    while ((colMatch = colRegex.exec(columnsStr)) !== null) {
      columns[colMatch[1]] = colMatch[2];
    }
    
    tables[tableName] = columns;
  }
  
  return tables;
}

function generateCreateTable(tableName: string, columns: Record<string, string>, database: string): string {
  const colDefs = Object.entries(columns)
    .map(([name, type]) => `  "${name}" ${type}`)
    .join(',\n');
  
  return `CREATE TABLE "${tableName}" (\n${colDefs}\n);`;
}

export default skill;
