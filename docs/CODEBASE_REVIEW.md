CODEBASE_REVIEW.md

Executive summary
- This is a pre-alpha codebase with MCP scaffolding across Rust core (L2/L3), a TypeScript gateway/skills layer (L1/L4), a Python execution engine (L5), and a React UI (L6). Progress on dynamic tool discovery scaffolding and a dedicated MCP workflow has been made, with deterministic validation tests. A full end-to-end Phase 2C HTTP validation surface is planned but currently blocked by environment constraints around compile-time SQLx macro caches. The review focuses on dead code, stubs, bad practices, security risks, inefficiencies, and dataflows, with actionable remediation and a prioritized backlog.

Scope & approach
- The review covers the complete codebase as accessible, with emphasis on MCP-related components in core/router, the UI surface (ui), and supporting docs. It prioritizes concrete, testable remediation rather than high-level observations.
- The approach emphasizes actionable edits in future patches and precise references for changes.

Findings by domain (core focus first, then UI, tests, etc.)

- Core Rust (L2/L3)
  - MCP endpoints exist and are evolving; multiple patches attempted dynamic discovery and validation hooks, but the surface remains non-breaking and logs-driven for Phase 2A.
  - The codebase uses macro-based SQL (query! / query_as!) extensively. A cargo sqlx prepare is required to populate the compile-time query cache, otherwise builds fail with cache mismatch errors. This blocks patch landing that touches those macro paths in constrained environments.
  - Data model drift risk: early attempts added optional fields on MCP types (e.g., an error field on McpRegistryInfo) that were rolled back for compatibility. Any future changes should lock the field set in MCP types and use a separate error envelope for surface signals.
  - Observed maintenance friction in patching MCP modules: patch churn around validation hooks and a Phase-2C surface surface (validation endpoint) needs consolidation. Recommend a single, central patch plan with a small number of changes per patch, all isolated.
  - Quick wins: consolidate error-access into a unified Error/Validation strategy; expand unit tests to cover registry/tool flows; convert critical macro-heavy paths to runtime SQL to ease CI requirements.

- Gateway/UI (L1/L4)
  - MCP UI surface (McpMarketplace) added and aligned with AgentZero visuals; strong typing in TS; no unsafe patterns.
  - Theming updated to include an MCP badge color; UI can surface basic validation outcomes if/when HTTP error surface lands.
  - Current surface is compatible with the current backend; future changes will wire in the HTTP error envelope and UI changes with minimal risk.

- Memory/Data (L3)
  - SQLite migrations exist (012, 015) with a plan to extend for MCP registries/tools (016 planned) as tool discovery scales. Ensure migrations run in CI and maintain compatibility with existing snapshots.
  - Observed risk: migrations should be idempotent and run in a deterministic order per environment; consider a migration manifest and a CI migration check.

- Execution (L5)
  - The execution engine is wired to router; sandboxing and resource isolation remain important safety concerns; plan to prioritize a safe default sandboxing configuration for local/dev.

- Tests & QA
  - Validation tests exist and are deterministic; phase 2 test goals include coverage for endpoint-level validation and the Phase 2C envelope.
  - There’s a distribution of tests across Rust, TypeScript, Python; ensure CI runs across the full suite and cross-language integration tests where applicable.

- Security review
  - HMAC-based authentication described in docs remains the primary boundary control; consider a future RBAC surface (T0–T3) to complement the existing model. No hard-coded secrets in the codebase.
  - Input validation groundwork is in progress; a dedicated error envelope will help surface validation issues without compromising security boundaries.

- Performance & dataflow
  - Potential hot paths include dynamic discovery flows and memory-heavy operations in the MCP path. Consider caching and smart invalidation strategies as the tool registry grows.
  - If HTTP validation surface lands, ensure its path does not introduce hot CPU/IO contention in the hot path.

- Maintainability & quality
  - Patch churn around the MCP layers should be minimized; enforce a single source of truth for error handling and a stable interface for MCP flows.
- Quick Wins
  - Introduce a single validation envelope and a small HTTP surface to communicate validation status during tool/registry creation.
  - Migrate macro-based queries to runtime SQL in patches that must land in constrained environments (CI without sqlx prepare).
  - Centralize secret handling and reduce defaults in code paths.

- Medium/Long-term improvements
  - Introduce registry/versioning and migration plans for MCP registries/tools.
  - Expand a formal MCP tool marketplace with signing, trust, and governance features.
  - Build a robust end-to-end MCP test harness that coordinates Rust, TS gateway, Python execution, and UI to verify real-world workflows.

Appendices (references)
- Core MCP files:
  - core/router/src/api/mcp.rs
  - core/router/src/mcp/validation.rs
  - core/router/tests/validation_tests.rs
- UI surfaces:
  - ui/McpMarketplace.tsx (MHPC marketplace surface)
  - docs/SESSION.md (session context and pause notes)
- Architecture and docs:
  - docs/architecture.md
- Session notes:
  - docs/SESSION.md
