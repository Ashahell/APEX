---
name: chorus-artifacts
description: Validate that files you are about to write are within your allowed Chorus draft directory, and check that all expected task outputs exist before signaling completion. Load before writing any files in a Chorus session.
license: MIT
compatibility: opencode
metadata:
  chorus-version: "1.1"
  category: orchestration
---

# Chorus Artifact Validation

Chorus enforces a sandbox: each task may only write to its own draft directory.
Violations cause `done.json` to be rejected at promotion time. Validate proactively.

## Your Sandbox

Your allowed write directory is specified in `current-task.md`:
```
Write ONLY to: workspace/draft/{taskId}/
```

Determine your taskId by reading the `**Task ID:**` line from `.chorus/current-task.md`.

## Path Validation

Before writing any file, verify the resolved path starts with your draft directory.

```
Validation logic:
  allowed = path.resolve("workspace/draft/{taskId}/")
  target  = path.resolve(proposed_path)
  valid   = target.startsWith(allowed)
```

**Path check results:**

```
workspace/draft/T2-architect/architecture.md   ✓ within your sandbox
workspace/draft/T2-architect/adrs/001.md       ✓ within your sandbox
workspace/draft/T1-scout/findings.md           ✗ FORBIDDEN — another task's draft
workspace/staging/architecture.md             ✗ FORBIDDEN — staging is read-only
.chorus/config.json                            ✗ FORBIDDEN — .chorus/ is read-only
../../../etc/passwd                            ✗ FORBIDDEN — resolves outside workspace
```

If a path is forbidden, determine the correct path and suggest it.
Do not write to forbidden paths under any circumstances.

## Completeness Check

Before running chorus-complete, verify all expected outputs exist.

1. Parse the `**Expected outputs**` section from `.chorus/current-task.md`
2. Check each listed path for existence
3. Report status:

```
Expected outputs for T2 (architect):

  [✓] workspace/draft/T2-architect/architecture.md    (exists, 4,211 bytes)
  [✗] workspace/draft/T2-architect/adrs/              (directory missing or empty)

Ready to complete: NO
Missing: adrs/ directory — create at least one ADR file before completing.
```

## Identifying Temp Files

Flag files in your draft directory that appear to be working notes rather than
deliverable artifacts. These will be included in done.json artifacts list if not filtered:

- Names containing: `scratch`, `notes`, `tmp`, `wip`, `draft-`
- Extensions: `.bak`, `.tmp`, `.swp`

When found, ask the user whether to include or exclude them from the artifact list.

## Error States

```
Cannot determine task ID from current-task.md
→ Expected line format: **Task ID:** T2
→ File may be malformed. Run: chorus status

workspace/draft/{taskId}/ does not exist
→ Create the directory before writing any files.

current-task.md does not contain expected outputs section
→ Check task definition in .chorus/workflows/. The task may not specify outputs.
```

## Rules

- Check paths before writing — not after
- Never silently write to a forbidden location
- If unsure whether a path is allowed, check it explicitly using the validation logic above
- Report all violations clearly with the correct alternative path