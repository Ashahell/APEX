# Execution Engine

## Overview

APEX provides a secure execution environment for running agent-generated code and tools.

## Backends

| Backend | Isolation | Use Case |
|---------|-----------|----------|
| Docker | Container | Production |
| Firecracker | MicroVM | High security |
| gVisor | Sandbox | Linux containers |
| Mock | None | Testing |

## Components

### Tool Sandbox
- Secure Python execution with import allowlist
- Timeout enforcement (default 30s)
- Memory limits (default 512MB)

### Dynamic Tool Generation
- LLM generates Python code
- Executes in sandboxed environment
- 24h TTL with auto-cleanup

### VM Pool
- Pre-warmed execution slots
- Connection pooling
- Health monitoring

## Security

- Import allowlist: Only approved modules
- Timeout: Configurable per execution
- Memory: Hard limits enforced
- Network: Optional isolation

## API Endpoints
- `/api/v1/dynamic-tools` - Tool management
- `/api/v1/vm/stats` - VM pool statistics