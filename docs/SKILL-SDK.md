# APEX Skill Development Kit (SDK)

Welcome to the APEX Skill SDK! This guide will help you create new skills for APEX.

## Quick Start

```bash
# Create a new skill from template
apex skill create my-new-skill

# Or manually create the structure
mkdir skills/my-new-skill/src
```

## Skill Structure

A skill must follow this structure:

```
my-skill/
├── package.json
└── src/
    └── index.ts
```

## Example Skill

```typescript
import { z } from 'zod';
import type { Skill, SkillContext, SkillResult } from '../../types.js';

// 1. Define input schema
const InputSchema = z.object({
  name: z.string().describe('Name of the item'),
  action: z.string().describe('Action to perform'),
});

// 2. Define output schema  
const OutputSchema = z.object({
  result: z.string(),
  success: z.boolean(),
});

// 3. Create and export the skill
export const skill: Skill = {
  name: 'my-skill',
  version: '0.1.0',
  tier: 'T1',  // T0, T1, T2, or T3
  inputSchema: InputSchema,
  outputSchema: OutputSchema,

  async execute(ctx: SkillContext, input: z.infer<typeof InputSchema>): Promise<SkillResult> {
    const { name, action } = input;
    
    try {
      // Your skill logic here
      const result = await performAction(name, action);
      
      return {
        success: true,
        output: JSON.stringify({ result, success: true }),
      };
    } catch (error) {
      return {
        success: false,
        error: `Skill failed: ${error}`,
      };
    }
  },

  async healthCheck(): Promise<boolean> {
    // Check if external dependencies are available
    return true;
  },
};

export default skill;
```

## Permission Tiers

| Tier | Description | Confirmation |
|------|-------------|--------------|
| T0 | Read-only queries | None |
| T1 | File writes, drafts | Tap to confirm |
| T2 | External API calls | Type to confirm |
| T3 | Destructive ops | 5-second delay |

## Package.json Template

```json
{
  "name": "my-skill",
  "version": "0.1.0",
  "type": "module",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "scripts": {
    "build": "tsup src/index.ts --dts",
    "dev": "tsx watch src/index.ts",
    "test": "vitest run"
  },
  "dependencies": {
    "@apex/skills": "workspace:*",
    "zod": "^3.22.0"
  },
  "devDependencies": {
    "@types/node": "^20.10.0",
    "tsx": "^4.7.0",
    "tsup": "^8.0.0",
    "typescript": "^5.3.0",
    "vitest": "^1.2.0"
  }
}
```

## Testing Your Skill

```typescript
// src/index.test.ts
import { describe, it, expect } from 'vitest';
import { skill } from './index.js';

describe('my-skill', () => {
  it('should execute successfully', async () => {
    const result = await skill.execute(
      { taskId: 'test', userId: 'test', workspacePath: '.', tier: 'T1' },
      { name: 'test', action: 'create' }
    );
    
    expect(result.success).toBe(true);
  });
});
```

## Running Tests

```bash
cd skills
pnpm install
pnpm test
```

## Registering Your Skill

After building, register with APEX:

```bash
curl -X POST http://localhost:3000/api/v1/skills \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-skill",
    "version": "0.1.0",
    "tier": "T1"
  }'
```

## Best Practices

1. **Always validate input** - Use Zod schemas
2. **Handle errors gracefully** - Return success: false with error message
3. **Add health checks** - Verify external dependencies
4. **Use proper tiers** - Don't use T2 for simple reads
5. **Document thoroughly** - Add descriptions to schema fields

## CLI Commands

```bash
# Create new skill
apex skill create <name>

# Build skill
apex skill build <name>

# Test skill
apex skill test <name>

# Register skill
apex skill register <name>
```

## Support

- GitHub: https://github.com/anomalyco/apex
- Discord: https://discord.gg/apex
