# Channels

## Overview

APEX implements messaging channels for multi-channel communication and adapter management.

## Adapters

| Adapter | Protocol | Status |
|---------|----------|--------|
| REST | HTTP | ✅ Built |
| Slack | Slack API | ✅ Built |
| Discord | Discord API | ✅ Built |
| Telegram | Bot API | ✅ Built |
| WhatsApp | WhatsApp API | ✅ Built |
| Email | SMTP | ✅ Built |

## Components

### Gateway Adapters
- Base adapter class for common patterns
- Protocol-specific implementations
- Message normalization across adapters

### Channel Management
- Create, edit, delete conversation channels
- List view with descriptions
- Default channels: default, general

### API Endpoints
- `/api/v1/channels` - List all channels
- `/api/v1/channels/:id` - Get channel details
- POST/PUT/DELETE channel operations
- `/api/v1/channels/:id/settings` - Channel-specific settings

## Features
- Message routing across adapters
- Channel-specific configurations
- Adapter health monitoring
- Message transformation and normalization