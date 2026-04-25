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