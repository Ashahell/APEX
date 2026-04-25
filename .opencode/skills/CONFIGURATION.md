# Chorus Skills Configuration Guide

## Overview

Chorus skills are OpenCode instruction sets that provide contextual awareness of Chorus workflow state. They enable OpenCode to:

- Understand task position in workflows
- Load prior artifacts with token budget management
- Validate sandbox boundaries before writing
- Signal task completion via `done.json`
- Navigate Party Mode collaborative sessions

## Installation

### Option 1: Global Installation (Recommended for personal use)

Install once, available in all projects:

```bash
# From the skills-global directory
cd skills-global
node install-skills.js

# Or manually:
# Windows
xcopy /E /I skills-global\* %USERPROFILE%\.config\opencode\skills\

# macOS/Linux  
cp -r skills-global/* ~/.config/opencode/skills/
```

### Option 2: Project-Local Installation (Recommended for team/CI use)

Install in each project for version control and reproducibility:

```bash
# Copy skills to your project
cp -r skills-global/* .opencode/skills/

# Add to your project repo
git add .opencode/skills/
git commit -m "Add Chorus skills"
```

## Configuration

### 1. opencode.json Permissions

Create `opencode.json` in your project root:

```json
{
  "permission": {
    "skill": {
      "chorus-task": "allow",
      "chorus-context": "allow",
      "chorus-artifacts": "allow",
      "chorus-complete": "allow",
      "chorus-party": "allow"
    }
  }
}
```

**Permission levels:**
- `allow` — Skill loads automatically when relevant
- `ask` — User prompted for approval before loading
- `deny` — Skill hidden from agent entirely

**Wildcard support:**
```json
{
  "permission": {
    "skill": {
      "chorus-*": "allow",
      "*": "ask"
    }
  }
}
```

### 2. Chorus Configuration

Create `.chorus/config.json` with your retention policies and settings.

## How Skills Work

### Auto-Loading Mechanism

Skills are **not** user commands. OpenCode agents load them via the skill() tool when their description signals relevance:

1. **Session starts** → OpenCode detects `.chorus/current-task.md`
2. **Skill loaded** → Based on description match
3. **Instructions applied** → Skill content guides agent behavior
4. **Tools execute** → Agent uses existing file access to do the work

### Skill Discovery

Chorus automatically appends to `current-task.md`:

```markdown
## Chorus Skills

The following OpenCode skills are available:
- **chorus-task** — Workflow position and task state
- **chorus-context** — Load prior artifacts
- **chorus-artifacts** — Validate write paths
- **chorus-complete** — Signal task completion
```

## Skill Reference

### chorus-task
**When loaded:** Start of any Chorus session  
**Key actions:**
- Read `.chorus/current-task.md` for task brief
- Read `.chorus/task-graph.json` for workflow position
- Display task status and downstream dependencies
- Answer workflow state questions

**Never:**
- Modify `.chorus/task-graph.json`
- Write to `.chorus/` except via `chorus-complete`

### chorus-context
**When loaded:** Need prior context before starting work  
**Key actions:**
- Load immediate predecessor artifacts (up to 3,000 tokens)
- Score other artifacts by keyword relevance
- Truncate from top when budget exceeded
- Report loaded vs. omitted artifacts

### chorus-artifacts
**When loaded:** Before writing files  
**Key actions:**
- Validate paths are within `workspace/draft/{taskId}/`
- Check expected outputs exist before completion
- Identify temp files vs. deliverables
- Prevent sandbox violations

### chorus-complete
**When loaded:** Task is finished  
**Key actions:**
- Run 10-step validation protocol
- Write atomic `done.json` (temp → rename)
- Validate all artifacts exist and are within sandbox
- Support success/interrupted/partial statuses

### chorus-party
**When loaded:** Party Mode session detected  
**Key actions:**
- Navigate multi-agent transcript
- Structure contributions (Agreement/Concerns/Proposal)
- Reach consensus in Round 3
- Document final decisions and overridden concerns

## Workflows

### Typical Session Flow

```
1. User runs: chorus run create-architecture
2. Chorus writes: .chorus/current-task.md
3. Chorus spawns: opencode
4. OpenCode loads: chorus-task (auto-detected)
5. OpenCode reads: current-task.md, task-graph.json
6. OpenCode loads: chorus-context (needs prior artifacts)
7. OpenCode works: creates outputs
8. OpenCode loads: chorus-complete
9. OpenCode writes: validated done.json
10. Chorus detects: done.json
11. Chorus advances: to next task
```

## Troubleshooting

### Skills Not Loading

**Check file naming:**
- File must be named exactly `SKILL.md` (uppercase)
- Directory name must match `name` field in frontmatter

**Check permissions:**
```bash
# Verify opencode.json allows the skill
grep chorus opencode.json
```

**Check location:**
```bash
# Global
ls ~/.config/opencode/skills/chorus-task/SKILL.md

# Project-local
ls .opencode/skills/chorus-task/SKILL.md
```

### done.json Not Detected

**Check file was written:**
```bash
ls -la .chorus/done.json
```

**Check file is complete:**
```bash
chorus status
```

## Best Practices

1. **Use global skills for personal work** — Install once, use everywhere
2. **Use project-local skills for teams** — Version controlled, reproducible
3. **Always include opencode.json** — Explicit permissions prevent surprises
4. **Start with `allow`** — Trust Chorus skills, they're well-tested
5. **Review loaded skills** — OpenCode shows which skills it loaded
6. **Check done.json** — Before ending session, verify it was written correctly

## Security Considerations

- Skills are **instruction sets**, not executable code
- Skills cannot declare tool permissions (use `opencode.json`)
- Skills cannot bypass file access controls
- Chorus validates all paths before promotion
- Sandbox violations are caught and reported

## Getting Help

- Run: `chorus help "skills"` for contextual help
- Check: `.chorus/task-graph.json` for current workflow state
- Review: `chorus history` for event log
- Verify: `chorus status` for active tasks