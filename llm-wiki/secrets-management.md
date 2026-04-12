# Secrets Management

## Overview

APEX implements encrypted secrets storage with AES-256-GCM encryption.

## Components

### Secret Store
- AES-256-GCM encryption
- Key derivation (secure)
- Access logging

### Secrets Repository
- CRUD operations
- Rotation tracking
- Access history

## API Endpoints

### Management
- `GET /api/v1/secrets` - List all
- `GET /api/v1/secrets/:id` - Get specific
- `PUT /api/v1/secrets/:id` - Update
- `DELETE /api/v1/secrets/:id` - Delete

### Categories
- `GET /api/v1/secrets/categories` - List categories
- `GET /api/v1/secrets/category/:cat` - Get by category

### Rotation
- `GET /api/v1/secrets/rotation/:name` - Rotation history
- `GET /api/v1/secrets/rotation/recent` - Recent rotations

### Access
- `GET /api/v1/secrets/access/:id` - Access history
- `GET /api/v1/secrets/access/recent` - Recent accesses
- `GET /api/v1/secrets/access/failed` - Failed attempts

## Secrets Categories (64 targets)
- Database credentials
- API keys
- OAuth tokens
- SSH keys
- TLS certificates
- Encryption keys

## Security

### Encryption
- Algorithm: AES-256-GCM
- Key derivation: Secure KDF
- Nonce: Unique per encryption

### Access Control
- HMAC-signed requests
- Access logging
- Failed attempt tracking

### Rotation
- Automatic rotation support
- Version tracking
- Rollback capability

## Docker Integration
- Docker secrets supported
- File-based secrets (`secrets/`)
- Environment variable injection