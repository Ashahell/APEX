# VM Sandbox Integration (MVP)

- Concept: minimal sandbox configuration for Hands execution and LLM tasks.
- MVP: a small struct describing CPU/memory limits and a simple flag for network isolation; apply boundary during execution in the MVP path.
- Integration points: apply to HandRunner/LMS tasks; log sandbox config usage in audit trail.
