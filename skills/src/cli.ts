import { parseArgs } from 'util';
import { SkillLoader } from './loader.js';
import { SkillContext } from './types.js';

async function main() {
  try {
    const { values } = parseArgs({
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
