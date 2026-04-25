# Chat Compaction

**Status:** Implemented (Phase 1.9)
**Date:** 2026-04-25
**Source:** [core/router/src/compaction.rs](core/router/src/compaction.rs)

## Overview

Chat compaction reduces context window usage by summarizing older messages. Inspired by Hermes chat compaction plugin. Part of Phase 1 UX improvements.

## How It Works

1. **Trigger**: Manual button in Chat header (auto-trigger at threshold planned for Phase 2)
2. **Preserve**: Keeps N most recent messages (configurable, default 10)
3. **Summarize**: LLM summarizes older messages into a compact `[Summary of N messages: ...]` block
4. **Replace**: Older messages replaced by summary; removed IDs tracked for audit

## Configuration

| Setting | Default | Range | Storage |
|---------|---------|-------|---------|
| Threshold (%) | 50 | 10-90 | localStorage + API |
| Preserve Recent | 10 | 2-50 | localStorage + API |

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/sessions/:session_id/compact` | Compact messages |
| GET | `/api/v1/sessions/:session_id/compact-status` | Check if compaction needed |

### Request
```json
{
  "threshold_percent": 50,
  "preserve_recent": 10
}
```

### Response
```json
{
  "session_id": "...",
  "original_count": 100,
  "compacted_count": 12,
  "summary": "[Summary of 88 messages: ...]",
  "removed_count": 88
}
```

## Files

| File | Purpose |
|------|---------|
| `core/router/src/compaction.rs` | Compaction service (267 lines, 5 tests) |
| `core/router/src/api/sessions.rs` | API handlers |
| `ui/src/components/chat/Chat.tsx` | "Compact" button + handler |
| `ui/src/components/settings/Settings.tsx` | Threshold/preserve settings |

## UI Behavior

- Button appears in Chat header bar (right side)
- Disabled when message count < 20
- Shows spinner during compaction
- Toast notification on success/failure
- Threshold and preserve settings in Developer tab

## Future Enhancements

- Auto-trigger at threshold (Phase 2)
- LLM-powered summarization (currently uses heuristic summary)
- Integration with actual message repository
- Compaction history/undo

## Related
- [phase-1-ux-improvements.md](concepts/phase-1-ux-improvements.md)
- [concepts/session-control.md](concepts/session-control.md) (yield/resume/compact)