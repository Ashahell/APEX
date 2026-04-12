# Narrative Memory

## Overview

APEX implements a narrative memory system for tracking story-like temporal data and decision trails.

## Components

### Narrative Store
- Timeline-based memory entries
- Event sequencing with timestamps
- Entity tracking across sessions

### Decision Journal
- Document and track decisions
- Fields: title, context, decision, rationale, outcome, tags
- Link decisions to tasks
- Search functionality

### Integration
- Bounded memory for agent context
- Working memory for short-term
- Consolidated memory for long-term

## API Endpoints
- `/api/v1/journal` - List/create journal entries
- `/api/v1/journal/:id` - Get/update/delete
- `/api/v1/journal/search?q=query` - Search entries
- `/api/v1/narrative` - Narrative operations
- `/api/v1/memory/narrative` - Memory-narrative integration

## Features
- Automatic decision capture
- Rationale tracking
- Outcome correlation
- Tag-based organization

## Use Cases
- Track agent decision reasoning
- Post-mortem analysis
- Audit trail for compliance
- Story-like memory reconstruction