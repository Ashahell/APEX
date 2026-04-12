# MCP (Model Context Protocol)

## Overview

MCP is a protocol for tool discovery, execution, and resource management in APEX.

## Components

### MCP Server
- Tool registry with discovery and versioning
- Health checks for tool availability
- Resource management and prompt templates

### MCP Client
- Connection pooling for efficiency
- Resource limits and quotas
- Validation and sanitization

## Features

### Tool Registry
- Discovery: Auto-detect available tools
- Versioning: Track tool versions
- Health: Periodic health checks

### Execution
- Validation: Input/output schema validation
- Sandboxing: Tool execution in isolated environment
- Caching: Reuse similar tool calls

### Security
- Input sanitization on all tool inputs
- Injection detection (50+ patterns)
- Rate limiting per tool

## API Endpoints
- `/api/v1/mcp/servers` - Server management
- `/api/v1/mcp/tools` - Tool listing
- `/api/v1/mcp/registries` - Registry management

## Integration

MCP integrates with:
- Skills framework for tool execution
- Execution engine for sandboxed code
- Stream endpoints for real-time tool output