# APEX Skill Specification

## Overview

Skills are reusable, typed execution units that extend APEX's capabilities. Each skill is a self-contained module with input/output validation, health checks, and permission tiering.

## Skill Structure

```
skill-name/
├── package.json          # Skill metadata
├── src/
│   └── index.ts          # Skill implementation
├── input.schema.json     # Input validation schema
└── output.schema.json    # Output validation schema
```

## package.json Schema

```json
{
  "name": "skill-name",
  "version": "0.1.0",
  "apex": {
    "tier": "T1",
    "capabilities": ["code", "execution"]
  }
}
```

## Skill Interface (TypeScript)

```typescript
import { z } from 'zod';

interface SkillContext {
  taskId: string;
  userId: string;
  workspacePath: string;
  tier: 'T0' | 'T1' | 'T2' | 'T3';
}

interface SkillResult {
  success: boolean;
  output?: string;
  error?: string;
  artifacts?: SkillArtifact[];
}

interface SkillArtifact {
  path: string;
  mimeType: string;
  content?: string;
}

interface Skill {
  readonly name: string;
  readonly version: string;
  readonly tier: PermissionTier;
  readonly inputSchema: z.ZodSchema;
  readonly outputSchema: z.ZodSchema;
  
  execute(ctx: SkillContext, input: unknown): Promise<SkillResult>;
  healthCheck(): Promise<boolean>;
}
```

## Permission Tiers

| Tier | Description | Example Skills |
|------|-------------|----------------|
| T0 | Instant - no execution | docs.read |
| T1 | Lightweight - limited scope | code.generate, code.review |
| T2 | Moderate - full execution | shell.execute |
| T3 | Full - unrestricted | (reserved) |

## Input/Output Validation

All skills must define Zod schemas for input and output validation:

```typescript
const InputSchema = z.object({
  language: z.string(),
  description: z.string(),
});

const OutputSchema = z.object({
  files: z.array(z.object({
    path: z.string(),
    content: z.string(),
  })),
});
```

## Health Checks

Each skill must implement a `healthCheck()` method that returns `true` if the skill is operational.

## Registry

Skills are registered in the SQLite `skill_registry` table:

| Column | Type | Description |
|--------|------|-------------|
| name | TEXT | Unique skill name |
| version | TEXT | Semantic version |
| tier | TEXT | Permission tier (T0-T3) |
| enabled | INTEGER | 1 = active, 0 = disabled |
| health_status | TEXT | unknown, healthy, unhealthy |
| last_health_check | TEXT | ISO timestamp |

## Loading

Skills are loaded from the `skills/skills/` directory at startup. The loader:
1. Scans for directories with `dist/index.js`
2. Imports the skill module
3. Validates the skill interface
4. Registers in SQLite
5. Runs initial health check

## Execution Flow

1. **Task Classification** - Router determines if task needs skill
2. **Capability Check** - Verify user tier allows skill
3. **Input Validation** - Parse and validate against inputSchema
4. **Skill Execution** - Call skill.execute()
5. **Output Validation** - Validate result against outputSchema
6. **Result Storage** - Save to database

## Error Handling

| Error | Description | Action |
|-------|-------------|--------|
| SkillNotFound | Skill doesn't exist | Return error, don't queue |
| TierDenied | User tier too low | Return 403 |
| ValidationError | Input doesn't match schema | Return 400 |
| ExecutionError | Skill threw exception | Log, return error, mark unhealthy |

## Built-in Skills

| Skill | Tier | Description |
|-------|------|-------------|
| code.generate | T1 | Generate code from description |
| code.review | T1 | Review code for issues |
| shell.execute | T2 | Execute shell commands |
| docs.read | T0 | Read documentation |
| git.commit | T1 | Create git commits |
