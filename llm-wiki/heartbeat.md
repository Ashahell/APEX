# Heartbeat System

## Overview

APEX implements a heartbeat daemon for autonomous agent wake cycles and scheduled tasks.

## Components

### Heartbeat Daemon
- Configurable interval (default 60 minutes)
- Jitter for randomness (default 10%)
- Cooldown between actions (default 300s)
- Max actions per wake cycle (default 3)

### Scheduler
- Task scheduling with cron-like expressions
- Conditional triggers based on system state
- Action queue management

### Configuration
- `APEX_HEARTBEAT_ENABLED` - Enable/disable daemon
- `APEX_HEARTBEAT_INTERVAL` - Interval in minutes
- `APEX_HEARTBEAT_JITTER` - Jitter percentage
- `APEX_HEARTBEAT_COOLDOWN` - Cooldown in seconds
- `APEX_HEARTBEAT_MAX_ACTIONS` - Max actions per wake

## API Endpoints
- `/api/v1/heartbeat/config` - Get/update configuration
- `/api/v1/heartbeat/stats` - Get statistics
- `/api/v1/heartbeat/trigger` - Manual trigger
- `/api/v1/heartbeat/toggle` - Enable/disable

## Use Cases
- Periodic health checks
- Scheduled task execution
- Memory consolidation triggers
- Session cleanup

## Integration
- Works with bounded memory consolidation
- Triggers skill pool pre-warming
- Coordinates with narrative memory