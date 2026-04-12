# NATS Integration

## Overview

APEX supports NATS for distributed deployment and message-oriented communication.

## Components

### NATS Client
- Connection management
- Subject-based messaging
- Request/reply pattern

### Message Bus
- Internal event distribution
- Cross-component communication
- Queue groups for scaling

## Configuration

### Environment Variables
| Variable | Default | Description |
|----------|---------|-------------|
| APEX_NATS_ENABLED | false | Enable NATS |
| APEX_NATS_URL | 127.0.0.1:4222 | Server URL |
| APEX_NATS_SUBJECT_PREFIX | apex | Subject prefix |

### Subject Patterns
```
apex.tasks.* - Task events
apex.skills.* - Skill execution
apex.heartbeat.* - Heartbeat events
apex.memory.* - Memory operations
```

## Features
- Distributed deployment support
- Load balancing via queue groups
- Wildcard subscriptions
- Connection pooling

## Use Cases
- Multi-instance scaling
- Cross-node communication
- Event-driven architecture
- Microservices integration