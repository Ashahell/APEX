# Session Control

## Overview

APEX implements session control for managing agent sessions, subagents, and task decomposition.

## Components

### Session Manager
- Session creation and termination
- State tracking and persistence
- Resource allocation per session

### Subagent Pool
- Parallel task execution
- Task decomposition via LLM
- Subtask coordination

### Session Control
- `sessions_yield` - Yield control of session
- `sessions_resume` - Resume session
- Session persistence in database

## API Endpoints
- `/api/v1/sessions` - Session management
- `/api/v1/sessions/:id/yield` - Yield session
- `/api/v1/sessions/:id/resume` - Resume session
- `/api/v1/subagent/*` - Subagent operations

## Features
- Parallel subtask execution
- Session state persistence
- Yield/resume for long-running tasks
- Resource cleanup on session end

## Integration
- Works with deep task worker
- Coordinates with skill pool
- Memory persistence across sessions