# Session Control

**Status:** Implemented
**Date:** 2026-04-25

## Overview

Session control provides yield, resume, and compaction capabilities for multi-turn agent conversations. Allows pausing a session and resuming it later, with optional context compaction.

## Sub-Features

### Yield (sessions_yield)
Pause a running session and create a child session for subagent execution.

- `POST /api/v1/sessions/:session_id/yield`
- `GET /api/v1/sessions/:session_id/yields`
- Creates child session ID for subagent
- Logs yield for inter-worker signaling

### Resume (sessions_resume)
Resume from a yielded session or archived session.

- `POST /api/v1/sessions/resume`
- `GET /api/v1/sessions/:session_id/resume-history`
- Links original → child session
- Transfers context summary

### Compaction
See [chat-compaction.md](chat-compaction.md) for details.

## Related
- [chat-compaction.md](concepts/chat-compaction.md)
- [phase-1-ux-improvements.md](concepts/phase-1-ux-improvements.md)