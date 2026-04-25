# Log

## [2026-04-25] init | Created llm-wiki structure for APEX project
- [CLAUDE.md] created - schema layer
- [index.md] created - content catalog
- [log.md] created - activity log
- [project-overview.md] created - initial entity page

## [2026-04-25] ingest | Ingested 82 docs from docs/ directory
- [concepts/architecture.md] created - 6-layer architecture, security model
- [concepts/skills.md] created - Skill registry, SKILL.md standard, Hermes features
- [raw/] populated with 82 markdown files

## [2026-04-25] audit | Schema audit completed
- [concepts/schema-audit.md] created - Critical findings: audit chain deletion bug, weak key derivation, missing FK constraints

## [2026-04-25] fix | Schema security fixes implemented
- [concepts/schema-fix-plan.md] updated - All 4 fixes completed
- Fix 1: Audit chain archival (archive instead of delete) - `ttl_cleanup.rs`, `db.rs`
- Fix 2: Strong key derivation (Argon2id + machine token) - `secret_store.rs`, `Cargo.toml`
- Fix 3: Foreign key enforcement - `db.rs` (PRAGMA foreign_keys=ON, CASCADE DELETE)
- Fix 4: Encryption by default - `settings.rs`, `db.rs` (auto-encrypt sensitive keys)
- Tests: 69 passed (63 memory + 6 security)

## [2026-04-25] fix | Router compilation issues fixed
- [streaming.rs] Fixed: duplicate Pin import, SSE type mismatch, invalid Pin casting
- [streaming.rs] Made StreamAuthQuery public
- [streaming.rs] Added StreamExt import
- [computer_use_api.rs] Removed unused mut
- [persona_api.rs] Removed unused mut
- [signing_api.rs] Prefixed unused vars with underscore
- [story_api.rs] Prefixed unused vars with underscore
- [replay_protection.rs] Fixed unused var and doc comment
- [skill_manager.rs] Prefixed unused var with underscore
- [persona.rs] Removed unused mut (4 places)
- [skill_pool.rs] Fixed invalid drop reference
- [system_component.rs] Added #[allow(async_fn_in_trait)]
- [channels_extended.rs] Prefixed unused var with underscore
- [security/src/secret_store.rs] Removed unused import
- [memory/src/audit.rs] Removed unused mut
- [memory/src/streaming_types.rs] Removed unused mut
- [memory/src/provider_repo.rs] Removed unused import and prefixed vars
- [memory/src/dashboard_repo.rs] Removed unused import
- Result: 0 warnings, all tests passing

## [2026-04-25] fix | Expanded test suite (518 tests passing)
- Fixed streaming_integration.rs test issues: added StreamingError variants, to_sse_event() method
- Tests: 518 passed (63 memory + 339 router + 110 integration + 6 security)
- GitHub: Pushed schema-fix-plan.md and production-harden-research.md commits

## [2026-04-25] review | Production hardening research completed
- Reviewed PRODUCTION_HARDENING.md requirements
- Researched latest 2026 AI agent security best practices (Google ADK, OWASP, Cordum, Harness)
- Key insights: staged deployment gates, policy enforcement before dispatch, checkpoint-resume for fault tolerance, iteration/time/token budgets
- Infrastructure hardening (seccomp, AppArmor, K8s policies) requires deployment environment, not code changes

## [2026-04-25] fix | SSRF protection added (CVE-2026-4308 mitigation)
- Monitored OpenClaw and Agent Zero repos for security updates
- Agent Zero disclosed CVEs: CVE-2026-4307 (path traversal), CVE-2026-4308 (SSRF)
- APEX already protected against path traversal (vm_pool.rs, content_hash.rs, injection_classifier.rs)
- Implemented SSRF protection in execution/src/apex_agent/__init__.py
- Blocks: localhost, private IPs (10.x, 172.16-31.x, 192.168.x), cloud metadata (169.254.169.254)
- DNS resolution to prevent CNAME redirection attacks
- Added 12 SSRF protection tests (64 execution tests total passing)

## [2026-04-25] feat | Phase 1 UX improvements (FUTURE_WORK.md)
### Stop-Button Persistence
- Migration 025: cancellation_requests table + tasks.cancellation_requested fields
- request_cancellation(), check_cancellation(), clear_cancellation() in task_repo.rs
- skill_worker + deep_task_worker check cancellation before each step
- WebSocket sends task_cancelled on reconnect

### Lexical Skill Matching Fallback
- Migration 026: skill_triggers table with keyword→skill mapping
- 60+ default triggers across 20 categories
- calculate_lexical_score(): exact=200, name=150, desc=80, keyword=40
- API: GET/POST/DELETE /api/v1/skills/triggers
- 6 unit tests passing

### Chat Compaction (Phase 1.9)
- compaction.rs: 267 lines, 5 tests (should_compact, disabled, summary, tokens, preserves_recent)
- API: POST/GET /api/v1/sessions/:id/compact
- UI: "Compact" button in Chat header (disabled <20 messages)
- Settings: threshold (10-90%) and preserve (2-50) in Developer tab
- Toast feedback on success/failure
- All tests passing, pushed to GitHub (commit 767ee46)

## [2026-04-25] ingest | Updated llm-wiki with Phase 1 entries
- [concepts/phase-1-ux-improvements.md] created
- [concepts/chat-compaction.md] created
- [concepts/session-control.md] created
- [concepts/schema-fix-plan.md] updated
- [index.md] updated with new concept pages

## [2026-04-25] ingest | Full wiki refresh — all recent changes
### Pages Updated
- [project-overview.md] refreshed — v1.6.0 summary, test suite, build commands
- [concepts/architecture.md] refreshed — 6-layer diagram, API surface, unified config
- [concepts/skills.md] refreshed — Hermes features, auto-created skills, Skills Hub

### Pages Created
- [concepts/security.md] — T0-T3 tiers, HMAC, TOTP, VM isolation, SSRF, schema fixes
- [concepts/future-work.md] — 4-phase roadmap, Phase 1 complete, priorities
- [concepts/streaming.md] — SSE, WebSocket, TinySSE protocols
- [concepts/test-suite.md] — 461+ tests breakdown by suite
- [concepts/ssrf-protection.md] — CVE-2026-4308 mitigation

### Pages Updated
- [concepts/schema-fix-plan.md] — Security fixes complete
- [index.md] — All new pages indexed
- [log.md] — This entry