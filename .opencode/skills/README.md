# Chorus Skills for OpenCode

OpenCode skills that give Chorus contextual awareness of workflow state. These skills allow OpenCode agents to understand task position, load prior context, validate outputs, and signal task completion.

## Installation

### Global Installation (Recommended)

Install to your OpenCode global skills directory:

```bash
# Windows
xcopy /E /I skills-global\* %USERPROFILE%\.config\opencode\skills\

# macOS/Linux
cp -r skills-global/* ~/.config/opencode/skills/
```

Or use the provided install script:

```bash
npm run install:skills
```

### Project-Local Installation

Copy to your project's `.opencode/skills/` directory:

```bash
cp -r skills-global/* .opencode/skills/
```

## Skills Included

### chorus-task
**Purpose:** Task awareness and workflow navigation  
**Load when:** Starting any Chorus session  
**What it does:**
- Reads current task from `.chorus/current-task.md`
- Understands workflow position via `.chorus/task-graph.json`
- Identifies downstream tasks that depend on your output
- Answers questions about task state and what to do next

### chorus-context
**Purpose:** Prior artifact loading with token budget  
**Load when:** Need context from previous workflow steps  
**What it does:**
- Loads artifacts from `workspace/staging/` with priority scoring
- Manages 8,000 token budget across multiple sources
- Truncates intelligently (keeps end of files, most recent content)
- Reports what was loaded and what was omitted

### chorus-artifacts
**Purpose:** Write path validation and output checking  
**Load when:** Before writing any files in a Chorus session  
**What it does:**
- Validates paths are within your sandbox (`workspace/draft/{taskId}/`)
- Checks all expected outputs exist before completion
- Identifies temp/scratch files vs. deliverable artifacts
- Prevents sandbox violations that would cause promotion failures

### chorus-complete
**Purpose:** Task completion signal  
**Load when:** Task is finished (required at end of every task)  
**What it does:**
- Writes validated `.chorus/done.json` to signal Chorus
- Validates all artifacts exist and are within sandbox
- Handles atomic writes (temp file → rename)
- Supports success, interrupted, and partial completion statuses

### chorus-party
**Purpose:** Party Mode collaboration  
**Load when:** current-task.md mentions party session or round number  
**What it does:**
- Navigates multi-agent discussion transcripts
- Structures contributions for opening/response/consensus rounds
- Helps reach alignment in collaborative decision-making
- Uses authority levels for conflict resolution

### karpathy-guidelines
**Purpose:** Karpathy-inspired coding guidelines  
**Load when:** Writing or editing code  
**What it does:**
- Applies 4 principles: Think Before Coding, Simplicity First, Surgical Changes, Goal-Driven Execution
- Reduces common LLM mistakes (wrong assumptions, overcomplication, unnecessary changes)
- Provides success criteria for verifying task completion
- Source: [forrestchang/andrej-karpathy-skills](https://github.com/forrestchang/andrej-karpathy-skills)

## Configuration

Add to your project's `opencode.json`:

```json
{
  "permission": {
    "skill": {
      "chorus-task": "allow",
      "chorus-context": "allow",
      "chorus-artifacts": "allow",
      "chorus-complete": "allow",
      "chorus-party": "allow",
      "karpathy-guidelines": "allow"
    }
  }
}
```

Or allow all Chorus skills with wildcard:

```json
{
  "permission": {
    "skill": {
      "chorus-*": "allow",
      "karpathy-*": "allow",
      "*": "ask"
    }
  }
}
```

## How Skills Work

Skills are **not** user-invoked commands. OpenCode agents load them autonomously via the `skill()` tool when their `description` signals relevance.

Example:
```javascript
// Agent loads skill based on description
skill({ name: "chorus-task" })
```

The skill provides instructions that guide the agent's behavior. The agent uses its existing file access tools to actually read/write files.

## Skill Discovery

Chorus automatically appends a skill hint to `current-task.md`:

```markdown
## Chorus Skills

The following OpenCode skills are available:
- **chorus-task** — Workflow position and task state
- **chorus-context** — Load prior artifacts
- **chorus-artifacts** — Validate write paths
- **chorus-complete** — Signal task completion
- **chorus-party** — Party Mode (if applicable)
```

This tells OpenCode that skills are available without requiring user action.

## Troubleshooting

**Skill not loading:**
1. Check file is named exactly `SKILL.md` (uppercase)
2. Verify directory name matches `name` field in frontmatter
3. Ensure `opencode.json` permissions allow the skill
4. Check OpenCode can access the skills directory

**done.json not detected:**
- Verify `.chorus/` directory exists and is writable
- Ensure file was written completely (atomic write recommended)
- Check Chorus file watcher is running: `chorus status`

## License

MIT