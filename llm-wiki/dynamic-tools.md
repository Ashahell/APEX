# Dynamic Tools

## Overview

APEX supports dynamic tool generation where the LLM creates Python code tools at runtime.

## Components

### Tool Generator
- LLM generates Python code
- Sandboxed execution
- Result caching

### Tool Sandbox
- Import allowlist enforcement
- Timeout (default 30s)
- Memory limit (default 512MB)

### Tool Cache
- Similar tools reused
- 24h TTL
- Automatic cleanup

## API Endpoints
- `/api/v1/dynamic-tools` - List all
- `/api/v1/dynamic-tools` (POST) - Generate new
- `/api/v1/dynamic-tools/:name` - Get specific
- `/api/v1/dynamic-tools/:name/execute` - Run tool

## Security

### Allowlist
Only these modules can be imported:
```python
# Math
math, statistics, random, itertools, functools

# Data
json, csv, datetime, re, collections

# IO
os, sys, pathlib, tempfile

# Network (sandboxed)
# No direct network access in sandbox
```

### Restrictions
- No file system outside /tmp
- No subprocess execution
- No network calls
- Memory capped at 512MB
- 30s timeout max

## Use Cases
- Data transformation
- Custom calculations
- Text processing
- Temporary analysis tools