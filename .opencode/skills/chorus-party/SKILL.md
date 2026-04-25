---
name: chorus-party
description: Participate in Chorus Party Mode multi-agent sessions. Load when current-task.md mentions a party session, round number, or collaborative alignment goal with multiple roles.
license: MIT
compatibility: opencode
metadata:
  chorus-version: "1.1"
  category: orchestration
---

# Chorus Party Mode

Party Mode is a structured multi-agent discussion format where several roles
align on a decision through sequential rounds. You take one turn at a time;
Chorus assembles the transcript and gives it to each participant in turn.

## Session Structure

```
Round 1: Opening Statements   — each role states their position
Round 2: Response             — each role responds to the others
Round 3: Consensus            — each role proposes or accepts consensus language
                              — highest-authority role resolves conflicts
```

Your `current-task.md` specifies:
- Which round you're in and what type (opening / response / consensus)
- Your role name and **authority level** (higher number = more authority)
- The session goal
- Full transcript of prior rounds (embedded directly)

## Reading the Transcript

Prior round contributions are embedded in your `current-task.md` under
`## Transcript So Far`. Read the full transcript before writing your response.

To view contributions from a specific role, search the transcript for:
```
**{Role Name} (Round {N}):**
```

## Writing Your Contribution

Write to the path specified in your task brief:
```
workspace/draft/party-{sessionId}/round{N}-{your-role}.md
```

Structure your contribution:

```markdown
## Round {N} — {Your Role} ({round type})

### Agreement
[What you agree with from other participants — be specific, reference their points]

### Concerns
[Specific disagreements or risks — quote or paraphrase their statements precisely]

### Proposal
[Your suggested path forward — make this the focus in the consensus round]
```

## Consensus Round

In Round 3, your goal is alignment, not winning. Integrate points from all participants:
1. Summarize what everyone agreed on in Round 2
2. Propose clear resolution for unresolved disagreements
3. If you are the highest-authority role and consensus is not reached naturally:
   - Make the final call explicitly: **"Final decision: [decision]"**
   - Incorporate valid concerns from others into the rationale
   - Document overridden concerns so they aren't lost

## Completing Your Party Turn

Use `chorus-complete` as normal. The task ID format for party turns is:
```
party-{sessionId}-round{N}-{role}
```

Example done.json for a party turn:
```json
{
  "taskId": "party-abc123-round2-architect",
  "status": "success",
  "summary": "Agreed on core matching scope, raised video latency concern",
  "artifacts": ["workspace/draft/party-abc123/round2-architect.md"],
  "notes": "Video concern documented — suggest deferring to v2 planning"
}
```

Chorus assembles your contribution into the next participant's `current-task.md`.

## Session End Conditions

Chorus ends the session when:
- All participants submit `status: success` in Round 3 (consensus reached)
- Any participant submits `status: interrupted` (session blocked, needs human)
- `maxDuration` configured in the party workflow YAML is reached

## Rules

- Address participants by role name, not "you" or "they"
- Reference specific statements from the transcript — do not generalize
- In consensus round, prioritize alignment — be willing to yield on minor points
- If you are not highest authority and you disagree with a final decision:
  document your concern in `notes` but do not block the session
- Never write to another participant's round file