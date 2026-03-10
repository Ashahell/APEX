import { parseArgs } from 'util';
import { SkillLoader } from './loader.js';
import { SkillContext } from './types.js';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

async function main() {
  try {
    const { values, positionals } = parseArgs({
      options: {
        skill: { type: 'string' },
        input: { type: 'string' },
        'task-id': { type: 'string' },
        'user-id': { type: 'string' },
        'workspace': { type: 'string' },
        tier: { type: 'string' },
      },
      strict: false,
    });

    const command = positionals[0];
    
    // Handle skill create command
    if (command === 'create') {
      const skillName = positionals[1];
      if (!skillName) {
        console.error(JSON.stringify({ error: 'Missing skill name. Usage: apex-skills create <name>' }));
        process.exit(1);
      }
      await createSkill(skillName);
      console.log(JSON.stringify({ success: true, message: `Skill '${skillName}' created` }));
      process.exit(0);
    }

    // Handle skill list command
    if (command === 'list') {
      // Simple directory listing for now
      console.log(JSON.stringify({ skills: [] }));
      process.exit(0);
    }

    // Original execution logic
    const skillName = values.skill as string | undefined;
    const inputStr = values.input as string | undefined;
    const taskId = (values['task-id'] as string) || 'unknown';
    const userId = (values['user-id'] as string) || 'local';
    const workspace = (values['workspace'] as string) || '.';
    const tier = (values.tier as string) || 'T1';

    if (!skillName) {
      console.error(JSON.stringify({ error: 'Missing --skill argument' }));
      process.exit(1);
    }

    if (!inputStr) {
      console.error(JSON.stringify({ error: 'Missing --input argument' }));
      process.exit(1);
    }

    const input = JSON.parse(inputStr);

    // Load skills from the skills/ directory using tsx-friendly paths
    const skillsDir = process.cwd() + '/skills/';
    const loader = new SkillLoader();
    await loader.loadFromDirectory(skillsDir);

    const ctx: SkillContext = {
      taskId,
      userId,
      workspacePath: workspace,
      tier: tier as 'T0' | 'T1' | 'T2' | 'T3',
    };

    const result = await loader.execute(skillName, ctx, input);

    console.log(JSON.stringify(result));
    process.exit(result.success ? 0 : 1);
  } catch (error) {
    console.error(JSON.stringify({
      success: false,
      error: error instanceof Error ? error.message : String(error)
    }));
    process.exit(1);
  }
}

main();

async function createSkill(name: string) {
  const skillDir = join(process.cwd(), 'skills', name);
  const srcDir = join(skillDir, 'src');
  
  if (existsSync(skillDir)) {
    throw new Error(`Skill '${name}' already exists`);
  }
  
  mkdirSync(srcDir, { recursive: true });
  
  // Create package.json
  const packageJson = {
    name,
    version: '0.1.0',
    type: 'module',
    main: './dist/index.js',
    types: './dist/index.d.ts',
    scripts: {
      build: 'tsup src/index.ts --dts',
      dev: 'tsx watch src/index.ts',
      test: 'vitest run',
    },
    dependencies: {
      '@apex/skills': 'workspace:*',
      zod: '^3.22.0',
    },
    devDependencies: {
      '@types/node': '^20.10.0',
      tsx: '^4.7.0',
      tsup: '^8.0.0',
      typescript: '^5.3.0',
      vitest: '^1.2.0',
    },
  };
  
  writeFileSync(join(skillDir, 'package.json'), JSON.stringify(packageJson, null, 2));
  
  // Create skill.yaml
  const skillYaml = `name: ${name}
version: 0.1.0
description: A new skill for APEX
tier: T1
categories:
  - utility
keywords:
  - automation
`;
  
  writeFileSync(join(skillDir, 'skill.yaml'), skillYaml);
  
  // Create index.ts
  const indexTs = `import { z } from 'zod';
import type { Skill, SkillContext, SkillResult, PermissionTier } from '@apex/skills';

const TIER: PermissionTier = 'T1';

const InputSchema = z.object({
  input: z.string().min(1).describe('Input value'),
});

const OutputSchema = z.object({
  success: z.boolean(),
  output: z.string().optional(),
  error: z.string().optional(),
});

export const name = '${name}';
export const version = '0.1.0';
export const tier = TIER;
export const inputSchema = InputSchema;
export const outputSchema = OutputSchema;

export async function execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
  try {
    // Your skill logic here
    const output = \`Processed: \${input.input}\`;
    
    return {
      success: true,
      output,
    };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function healthCheck(): Promise<boolean> {
  return true;
}
`;
  
  writeFileSync(join(srcDir, 'index.ts'), indexTs);
  
  // Create README.md
  const readme = `# ${name}

A skill for APEX.

## Usage

\`\`\`bash
apex-skills execute --skill ${name} --input '{"input": "hello"}'
\`\`\`

## Tier

T1
`;
  
  writeFileSync(join(skillDir, 'README.md'), readme);
}
