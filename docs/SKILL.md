# APEX Skill Specification

## Overview

Skills are reusable, typed execution units that extend APEX's capabilities. Each skill is a self-contained module with input/output validation, health checks, and permission tiering.

APEX v0.2.0 supports two skill formats:
1. **TypeScript Skills** - Traditional skill format with package.json
2. **SKILL.md Plugins** - Markdown-based skill definition (new in v0.2.0)

## Skill Structure

### TypeScript Format
```
skill-name/
├── package.json          # Skill metadata
├── src/
│   └── index.ts          # Skill implementation
├── input.schema.json     # Input validation schema
└── output.schema.json    # Output validation schema
```

### SKILL.md Plugin Format (v0.2.0)
```
skill-name/
├── SKILL.md              # Skill definition
├── src/
│   └── index.ts          # Implementation (optional)
└── config.yaml           # Runtime configuration (optional)
```

## SKILL.md Plugin Format

```markdown
# skill.code.generate

**Version**: 1.2.0
**Author**: APEX Team
**Tier**: T1 (Tap to confirm)
**Runtime**: TypeScript

## Description
Generates code from natural language descriptions using AI.

## Input Schema
```json
{
  "type": "object",
  "properties": {
    "language": { "type": "string", "enum": ["python", "javascript", "rust", "go"] },
    "description": { "type": "string" }
  },
  "required": ["language", "description"]
}
```

## Output Schema
```json
{
  "type": "object",
  "properties": {
    "code": { "type": "string" },
    "files": { "type": "array" }
  }
}
```

## Capabilities
- code.generate
- file.write
- docs.read

## Security
- sandbox: true
- network: false
- timeout: 30s

## Example
```yaml
input:
  language: python
  description: "A function to calculate fibonacci numbers"
```
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
| shell.execute | T3 | Execute shell commands (requires TOTP) |
| docs.read | T0 | Read documentation |
| git.commit | T2 | Create git commits |

---

## SKILL.md Plugin System (v0.2.0)

APEX v0.2.0 introduces a new plugin system based on SKILL.md files.

### Plugin Manager

```rust
pub struct SkillPluginManager {
    plugins: HashMap<String, SkillPlugin>,
    skills_dir: PathBuf,
}

impl SkillPluginManager {
    pub async fn load_plugin(&mut self, name: &str) -> Result<&SkillPlugin, PluginError>;
    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError>;
    pub fn get_plugin(&self, name: &str) -> Option<&SkillPlugin>;
    pub fn list_plugins(&self) -> Vec<&SkillPlugin>;
}
```

### Plugin Manifest

```rust
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tier: PermissionTier,      // T0, T1, T2, T3
    pub runtime: PluginRuntime,    // TypeScript, Python, Bash
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub capabilities: Vec<String>,
    pub security: SkillSecurity,
}
```

### Plugin Runtime Types

| Runtime | Description |
|---------|-------------|
| TypeScript | Compiled JavaScript with node |
| Python | Python 3.11+ interpreter |
| Bash | Shell scripts (/bin/sh) |

### Security Configuration

```yaml
security:
  sandbox: true      # Run in isolated environment
  network: false    # Disable network access
  timeout: 30       # Max execution time in seconds
  max_memory_mb: 512
```

### Loading Plugins

Plugins are automatically loaded from the `skills/skills/` directory. Each subdirectory with a `SKILL.md` file is treated as a plugin.
