Title: APEX Computer Use - Internal MVP Plan (Phase-based)
Date: 2026-03-25
Status: Internal MVP planning

Overview
- This document translates the high-level architecture and 12-week roadmap from docs/COMPUTER_USE_IMPLEMENTATION.md into an executable, phased MVP plan tailored to internal development workflows. It describes how to parse the Executive Summary and Phase sections to drive concrete tasks, milestones, and gating criteria.

Phase 1: Foundation (Weeks 1-3)
- Week 1: Isolate environment for computer use; establish VM/container skeleton; implement ScreenshotManager interface and simple capture stub; create initial API surface for health/shutdown.
- Week 2: Implement ActionExecutor with basic mouse/keyboard primitives; create VLMController scaffold; wire orchestration skeleton (ComputerUseOrchestrator) to accept a task and plan actions.
- Week 3: End-to-end skeleton: confirm compile, wire components end-to-end, and enable a simple Plan->Act->Observe loop with no real UI actions yet. Prepare internal demo script.
- Deliverables: computer_use VM image or docker-compose, skeleton Rust modules under core/router/src/computer_use/, minimal API endpoints, simple unit tests scaffold.

Phase 2: Core Features (Weeks 4-6)
- Week 4: Implement screenshot streaming to UI and basic action retries; refine data models for CapturedScreenshot and ActionResult.
- Week 5: Integrate browser automation hook (Playwright) for web tasks; enable simple web navigation tasks.
- Week 6: Security hardening, add caching, and basic logging of actions and results; ensure deterministic retry behavior.
- Deliverables: functional screenshot streaming to UI; Playwright-based browser tasks; hardened VM isolation checks; tests for core flows.

Phase 3: Intelligence (Weeks 7-9)
- Week 7: Grounding model integration (UI-TARS or local VLM) for element detection; implement prompt templates.
- Week 8: Multi-step planning and self-correction; implement a simple chain-of-thought flow and error recovery best practices.
- Week 9: Performance optimization: caching of screenshots, compression tuning, and parallelization of independent steps.
- Deliverables: integrated VLM prompts; multi-step task planning; performance baseline measurements.

Phase 4: Production (Weeks 10-12)
- Week 10: Comprehensive integration tests and OSWorld-style benchmarks; refine test harness.
- Week 11: Security audit and hardening review; refine permission checks and VM safety controls.
- Week 12: Documentation and production rollout plan; enable feature flags for production gating.
- Deliverables: production-ready API surface, detailed tests, security documentation, and rollout plan.

4. API Surface (Appendix A mapped to internal surface)
- /api/v1/computer-use/execute -- POST: Start a computer-use task (payload: task, max_steps, max_cost_usd, stream)
- /api/v1/computer-use/status/:id -- GET: Query task status
- /api/v1/computer-use/cancel/:id -- POST: Cancel running task
- /api/v1/computer-use/stream/:id -- GET: WebSocket stream of live screenshots
- /api/v1/computer-use/screenshots/:id -- GET: Retrieve screenshot history

5. Parsing Guidelines (how to derive internal plan from the doc)
- Executive Summary identifies target capabilities and high-level flow.
- Section 2 (Architecture Design) enumerates core components and their responsibilities; map to modules and crates.
- Section 3 (Implementation Roadmap) provides the 4-phase, 12-week plan; convert each Phase into concrete tasks and weekly goals.
- Section 4 (Component Specifications) provides data models and interfaces to implement first (ScreenshotManager, ActionExecutor, VLMController).
- Section 5 (Integration Points) shows where to wire the new computer-use surface into Skill system, API, and event streams.

Notes
- This file is intended for internal planning and onboarding of new team members. It should be updated as concrete implementation decisions land and tests are added.
