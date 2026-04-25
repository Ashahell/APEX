---
name: chorus-task
description: Read and understand the current Chorus orchestration task, workflow position, dependencies, and what downstream tasks need from your output. Load this at the start of any session where .chorus/current-task.md exists.
license: MIT
compatibility: opencode
metadata:
  chorus-version: "1.1"
  category: orchestration
---

# Chorus Task Awareness

You are in a Chorus-orchestrated session. Chorus is a workflow orchestration system
that sequences AI work across multiple roles. Your task, role, and expected outputs
are defined in `.chorus/current-task.md`.

## On Session Start

1. Read `.chorus/current-task.md` — your complete task brief
2. Read `.chorus/task-graph.json` — understand where you sit in the pipeline
3. Identify your output directory from the task brief (always `workspace/draft/{taskId}/`)

## State Files

| File | Purpose | Access |
|------|---------|--------|
| `.chorus/current-task.md` | Your role, task, expected outputs, available context | Read |
| `.chorus/task-graph.json` | Full workflow pipeline, all task statuses | Read |
| `.chorus/event-log.jsonl` | History of completed tasks and artifact paths | Read |
| `workspace/staging/` | Promoted artifacts from prior tasks | Read |
| `workspace/draft/{taskId}/` | Your exclusive output directory | Write |

## Understanding Your Position

Parse `.chorus/task-graph.json` to find your task and understand:

```json
// Look for your taskId in the tasks array
// Check dependsOn to know what prior work you build on
// Note which tasks list your taskId in their dependsOn — these are downstream
```

Present your position clearly:
```
Workflow: create-architecture (standard complexity)

  T1  scout              ✓ complete   → workspace/staging/T1/findings.md
  T2  architect          ◉ YOU        → workspace/draft/T2-architect/
  T3  security-review    ○ pending    (waiting for your output)
  T4  tech-lead-review   ○ pending    (waiting for T3)
```

## What Downstream Tasks Need From You

Before starting work, check what T3 (or whichever task follows yours) expects.
Read its entry in `task-graph.json`. Its `inputArtifacts` or description will
tell you what to produce. Produce what they need, not just what seems complete.

## Answering Questions About State

When asked anything about the current task, workflow, or what to do next:
- Read the actual files — do not rely on memory or assumption
- Cite which file you found the answer in
- If a file is missing, say so and suggest the recovery command

## Error States

If expected files are missing, report clearly and do not proceed blindly:

```
.chorus/current-task.md not found
→ Is a Chorus workflow active? Run: chorus status

.chorus/task-graph.json not found or invalid JSON  
→ Run: chorus resume to attempt recovery

workspace/staging/ is empty
→ This may be the first task, or prior tasks haven't been promoted yet
```

## Rules

- Never modify `.chorus/task-graph.json` — Chorus owns this file
- Never write to `.chorus/` except `done.json` (use the chorus-complete skill)
- Treat all `.chorus/` files as read-only except `done.json`
- Answer from file contents, not assumptions