# VM Sandbox Final MVP

- Purpose: provide a minimal, production-like sandbox config surface for MVP Hands/LLM execution.
- MVP: a simple VM sandbox config struct with fields: cpu_limit, memory_limit_mb, timeout_sec, network_isolation.
- Usage: apply the config to execution nodes when running hands tasks and LLM prompts; log outcomes for audit.
- Next steps: connect this config to the execution engine and add validation tests.
