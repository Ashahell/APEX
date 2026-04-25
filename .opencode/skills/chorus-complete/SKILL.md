---
name: chorus-complete
description: Signal Chorus that your task is complete by writing a validated done.json file. Required at the end of every Chorus task. Validates all artifacts exist and are within your sandbox before writing.
license: MIT
compatibility: opencode
metadata:
  chorus-version: "1.1"
  category: orchestration
---

# Chorus Task Completion

The correct and only way to end a Chorus task. Chorus watches for `.chorus/done.json`
and will not advance the workflow until it appears and is valid.

## Completion Protocol

Run through these steps in order. Do not write done.json until all pass.

```
Step 1  Read task ID from .chorus/current-task.md  → **Task ID:** line
Step 2  List all files in workspace/draft/{taskId}/
Step 3  Identify final artifacts vs. temp/scratch files (ask if uncertain)
Step 4  Validate every artifact path starts with workspace/draft/{taskId}/
Step 5  Verify every listed artifact exists on disk
Step 6  Confirm all expected outputs from current-task.md are present
Step 7  Write summary ≤ 200 chars describing what was produced
Step 8  Collect optional notes for human reviewer
Step 9  Write done.json atomically (temp file → rename)
Step 10 Confirm to user that Chorus has been signaled
```

## done.json Schema

```json
{
  "taskId": "T2",
  "status": "success",
  "summary": "Architecture.md + 2 ADRs for auth and database selection",
  "artifacts": [
    "workspace/draft/T2-architect/architecture.md",
    "workspace/draft/T2-architect/adrs/001-authentication.md",
    "workspace/draft/T2-architect/adrs/002-database.md"
  ],
  "notes": "ADR-001 flags auth approach for security review — see §3"
}
```

All artifact paths must be **relative to the project root**. Never use absolute paths.

## Atomic Write

Write done.json using a temp file and rename to prevent Chorus reading a partial file:

```bash
# Write to temp, then atomically rename
cat > .chorus/done.json.tmp << 'EOF'
{
  "taskId": "T2",
  ...
}
EOF
mv .chorus/done.json.tmp .chorus/done.json
```

On POSIX systems, `mv` within the same filesystem is atomic. This ensures Chorus
either sees a complete file or no file — never a partial write.

## Validation Checks

| Check | On failure |
|-------|-----------|
| Task ID readable from `current-task.md` | Abort — cannot identify task |
| All artifact paths within `workspace/draft/{taskId}/` | Abort — sandbox violation |
| All artifact paths exist on disk | Remove missing paths, warn user |
| All expected outputs from task brief present | Warn — ask user to confirm or explain |
| At least one artifact listed | Warn — completing with no artifacts is unusual |
| Summary ≤ 200 chars | Prompt user to shorten |

## Completion Statuses

| Status | When to use |
|--------|------------|
| `success` | Task completed, all expected outputs produced |
| `interrupted` | Task cannot be completed — blocker, unclear requirements, scope issue |
| `partial` | Usable work exists but task is not fully complete |

For `interrupted` or `partial`, still list any artifacts that exist — Chorus preserves
them, and `chorus resume` can continue from partial work.

## Interactive Walk-Through

When running this skill interactively, present the user with:

```
Chorus Task Completion — T2 (architect)
════════════════════════════════════════

Files in workspace/draft/T2-architect/:
  architecture.md             (4,211 bytes)  → include as artifact? [Y/n]
  adrs/001-authentication.md  (892 bytes)    → include as artifact? [Y/n]
  adrs/002-database.md        (741 bytes)    → include as artifact? [Y/n]
  scratch-notes.md            (203 bytes)    → temp file, exclude? [Y/n]

Expected outputs:
  [✓] architecture.md          present
  [✓] adrs/                    present (2 files)

Summary (≤200 chars):
> _

Notes for reviewer (optional):
> _
```

After user input, write done.json and confirm:
```
✓ .chorus/done.json written
  Chorus has been signaled. Waiting for workflow to advance.
  (Next: T3 — security-review will begin after human review)
```

## Error States

```
✗ Abort: cannot determine task ID
  current-task.md must contain: **Task ID:** T2

✗ Abort: sandbox violation
  workspace/staging/arch.md is outside workspace/draft/T2-architect/
  Correct path: workspace/draft/T2-architect/arch.md

✗ Abort: artifact declared but not found on disk
  workspace/draft/T2-architect/adrs/ is empty or missing
  
✗ Abort: done.json write failed
  Check disk space: df -h .
  Check permissions: ls -la .chorus/

⚠ Warning: completing with no artifacts
  done.json will list no files. Is this intentional? [y/N]
```

## Rules

- Never write done.json without running all validation checks first
- Never use absolute paths in artifacts — always relative to project root
- If any sandbox check fails, abort and explain — do not bypass
- If the user insists on bypassing a check, document it explicitly in `notes`
- After writing done.json, do not write any more files — the task is over