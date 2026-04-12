# Soul Identity

## Overview

APEX implements an identity system called SOUL (Self Observing Understanding Layer) for agent personality and values.

## Components

### Constitution
- Core values and principles
- Behavior guidelines
- Decision constraints

### Enforcer
- Action validation against constitution
- Constraint checking
- Override requests with justification

### Loader
- Identity persistence
- Fragment management
- Auto-backup on changes

## SOUL.md Format
```yaml
---
name: Agent Name
version: 1.0.0
traits:
  - curious
  - methodical
values:
  - transparency
  - privacy
constraints:
  - never reveal secrets
  - confirm destructive actions
---
# Free-form identity description
```

## API Endpoints
- `/api/v1/soul` - Get/update SOUL identity
- `/api/v1/soul/fragments` - Get modular fragments

## Features
- Automatic backup on changes
- Fragment-based identity
- Constitution enforcement
- Override logging

## Integration
- Works with agent loop
- Enforces constraints on actions
- Tracks override justifications