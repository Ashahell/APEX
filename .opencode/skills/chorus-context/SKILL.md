---
name: chorus-context
description: Load and prioritize relevant artifacts from prior Chorus tasks within a token budget. Use when you need context from previous workflow steps before starting your task.
license: MIT
compatibility: opencode
metadata:
  chorus-version: "1.1"
  category: orchestration
---

# Chorus Context Loading

Load prior task artifacts intelligently. Too much context loses focus; too little
misses critical prior work. This skill gives you a prioritized loading strategy.

## Token Budget Allocation

```
Total artifact budget: 8,000 tokens (configurable in .chorus/config.json)

Priority 1 — Immediate predecessor   : up to 3,000 tokens  (load in full)
Priority 2 — Keyword-matched prior   : up to 3,000 tokens  (scored by relevance)
Priority 3 — Older artifacts         : remaining budget     (truncated from top)
```

Your task instruction (from `current-task.md`) does not count against this budget.

## Loading Protocol

### Step 1: Identify your predecessors
Read `.chorus/task-graph.json`, find your task, read its `dependsOn` list.
For each dependency, find its promoted artifacts in `workspace/staging/{taskId}/`.

### Step 2: Load immediate predecessor in full
The task you directly depend on is always loaded completely (up to 3,000 tokens).
If it exceeds the limit, load the most recent portion and note the truncation.

### Step 3: Score other available artifacts
For each artifact in `workspace/staging/` from non-immediate predecessors:

```
Relevance score = (keyword matches) / (total task keywords)

Where:
- task keywords = words >4 chars from your task description, stopwords removed
- keyword matches = count of task keywords found in the artifact text
- Threshold: score > 0.3 = relevant, load within remaining budget
- Sort relevant artifacts by task recency (newer = higher priority)
```

### Step 4: Report what was loaded

```
Context loaded for T2 (architect):

Loaded (2,341 tokens):
  workspace/staging/T1/findings.md    1,847 tokens  [immediate predecessor, full]
  workspace/staging/T0/brief.md         494 tokens  [score: 0.41, truncated to fit]

Omitted (budget exhausted or score < 0.3):
  workspace/staging/T0/raw-notes.md   score: 0.12 — not relevant
  
Budget: 2,341 / 8,000 tokens used
```

## Truncation Rules

When an artifact must be truncated to fit the budget:
1. Keep the **last 80%** of the file (most recent content is usually most relevant)
2. Insert at the truncation point: `[... {N} tokens truncated from beginning ...]`
3. Never truncate the immediate predecessor — reduce keyword-matched artifacts first
4. If the immediate predecessor alone exceeds budget, truncate it from the top (keep end)

## Force-Loading

If you need a specific artifact regardless of score, load it directly by path.
Check if it fits in remaining budget; if not, warn and truncate from the top.

## Error States

```
workspace/staging/ is empty or does not exist
→ No prior artifacts to load. This may be the first task in the workflow.

workspace/staging/{taskId}/ exists but contains no files
→ Prior task completed but produced no artifacts, or promotion failed.
→ Check: chorus status

File is unreadable (permissions, binary, etc.)
→ Skip and note in your working log. Do not fail silently.
```