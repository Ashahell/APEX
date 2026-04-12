# Deep Tasks

## Overview

APEX implements a deep task worker for complex, multi-step task execution with LLM-driven planning.

## Components

### Deep Task Worker
- Plan/act/observe loop
- Task decomposition
- Resource allocation

### Task Router
- Automatic tier classification:
  - **Instant**: Simple queries → immediate response
  - **Shallow**: Skills → skill execution
  - **Deep**: LLM → full agent loop

### Agent Loop
- Plan: Task breakdown
- Act: Execute subtasks
- Observe: Evaluate results
- Iterate until completion or max steps

## API Endpoints
- `/api/v1/tasks` - Task CRUD
- `/api/v1/deep` - Deep task execution
- `/api/v1/subagent/decompose` - Task decomposition

## Features
- Atomic task updates with decision journal
- Transaction boundaries
- Worker supervision with restart loops
- Cost estimation and budget tracking

## Configuration
- `max_steps` - Maximum iterations
- `budget_usd` - Cost limit
- `time_limit_secs` - Time constraint
- `priority` - Task priority
- `category` - Task category