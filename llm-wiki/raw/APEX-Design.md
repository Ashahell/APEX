Table of Contents
Executive Summary
Parent System Reality Check
Build Environment
High-Level Architecture
Security Architecture
Memory & Storage
Skill Registry
Task Router — L2
Messaging Gateway — L1
Execution Engine — L5
Error Handling & Observability
Repository Structure
Phased Delivery Plan
Risk Register

## 1. Executive Summary

### Vision: OpenClaw + AgentZero + Security-First

APEX combines the **best of OpenClaw and AgentZero** with **significantly stronger security**.

| Reference | What We Take |
|-----------|-------------|
| **OpenClaw** | Open architecture, extensibility, community-driven plugin ecosystem, messaging adapters |
| **AgentZero** | Dark navy/cyan aesthetic, polished UI, smooth UX patterns, agent loop logic |
| **Security-first** | Hardened beyond both — T0-T3 permission tiers, HMAC auth, TOTP verification, input sanitization, connection pooling |

APEX is **more secure than both** by design:
- Single-user architecture (no multi-tenancy attack surface)
- Hardened MCP with connection pooling and input validation
- Firecracker/gVisor isolation for code execution (Docker fallback on Windows)
- Audit trail with decision journal and reflection tracking

### Core Principles
- Single-tenant by design — one user, one database, one instance. No multi-user complexity.
- Creative focus — optimized for coding, AI media generation, content creation, and marketing workflows.
- Security by isolation — Firecracker/gVisor micro-VMs for code execution, Docker fallback.
- Curated over comprehensive — 33 high-quality skills (T0-T3 tiers) instead of thousands of unmaintained adapters.

### What This Is NOT
- A multi-user SaaS platform
- A general-purpose assistant (no restaurant booking, no flight search)
- A low-code tool for non-technical users
- A solution for enterprise teams

### Implementation Status (2026-03-09)
- **Version**: v1.3.0 (Pre-Alpha)
- **Skills**: 33 (T0-T3 tiers)
- **MCP**: Full implementation with pooling, resources, prompts, security
- **Memory**: Vector search (sqlite_vec), hybrid search, embedder
- **Execution**: Docker (default), Firecracker/WSL2 (ready)
- **Tests**: 158 passing

Deployment model: Self-hosted on user's hardware (desktop, laptop, or personal cloud VM). Single binary + Docker for dependencies.
2. Parent System Reality Check
Honest assessment of upstream dependencies as of March 2026:
OpenClaw — github.com/openclaw/openclaw
The hype: 200,000+ GitHub stars, 84 days from launch to 200k stars (fastest growing repo in GitHub history), featured in Super Bowl commercial for AI.com, creator joining OpenAI.
The reality:
Launched November 24, 2025 — less than 4 months old as of this specification
11,000+ commits but codebase is evolving rapidly with breaking changes
Skills ecosystem is unproven — "5,700+ skills" exist but quality varies wildly; many are simple API wrappers with no error handling
Architecture is messaging-first, not execution-first — skills are shallow LLM prompts, not autonomous agents
Stability risk: High. Rapid growth means API instability, security issues not yet discovered, and potential for project abandonment or direction changes (especially with founder joining OpenAI)
APEX strategy: Fork OpenClaw's core messaging adapters (Slack, Discord, Telegram, WhatsApp) but do not inherit the skill ecosystem. Reimplement only the messaging layer, ignore the skills.
Agent Zero — github.com/agent0ai/agent-zero
The reality:
13,200 GitHub stars, actively maintained since 2024
Experimental-grade software — explicitly not production-hardened
No multi-tenancy, no user management, no audit trail — matches APEX's single-user model
Docker-based execution — insufficient isolation for untrusted code; APEX replaces with Firecracker
APEX strategy: Fork the agent loop and tool generation logic. Wrap in Firecracker isolation. Discard the web UI (APEX provides messaging interfaces instead).
3. Build Environment
Language Assignments
Table
Subsystem	Language	Justification
Core Daemon / Task Router / Memory	Rust	Security-critical single-user daemon; memory safety; no GC pauses; Tokio async
Messaging Gateway / Skill Runner	TypeScript (Node.js)	OpenClaw messaging adapters are TypeScript; skill ecosystem in JS/TS
Execution Agent (sandboxed)	Python	Agent Zero is Python; AI/ML tools are Python-native; isolated in micro-VM
Why Three Languages Is Acceptable Here
In a single-user deployment, operational complexity is lower. No multi-tenant orchestration, no fleet management. The user runs one binary on their machine. The cost of three languages is paid once at build time, not continuously in operations.
Build Tooling
plain
Copy
Rust:        Cargo workspace · Tokio · Axum · Serde · sqlx (SQLite)
TypeScript:  pnpm monorepo · Fastify · Zod · Vitest
Python:      Poetry · FastAPI (agent sidecar) · pytest · ruff · mypy
VMM:         Firecracker (production) · gVisor (development fallback)
Messaging:   NATS JetStream (single-node for single-user)
Observability: Prometheus metrics endpoint (user-facing dashboard)
4. High-Level Architecture
APEX runs as a single daemon process with internal service separation via NATS (single-node). No distributed systems concerns.
plain
Copy
┌─────────────────────────────────────────────────────────────┐
│  L1 · Messaging Gateway (TypeScript)                        │
│  Slack · Discord · Telegram · WhatsApp · Email · REST API │
└──────────────────────┬──────────────────────────────────────┘
                       │  TaskRequest → NATS: apex.tasks.inbound
┌──────────────────────▼──────────────────────────────────────┐
│  L2 · Task Router (Rust)                                    │
│  Rule-based classification · Permission enforcement           │
│  Routing: Instant / Shallow / Deep                          │
└──────────┬────────────────────────────┬───────────────────────┘
           │ apex.tasks.shallow       │ apex.tasks.deep
┌──────────▼──────────┐    ┌───────────▼──────────────────────┐
│  L4 · Skill Runner  │    │  L5 · Execution Engine          │
│  (TypeScript)       │    │  (Python in Firecracker microVM)│
│  Curated skill set  │    │  Agent Zero loop                │
│  (see Section 7)    │    │  Autonomous tool generation     │
└──────────┬──────────┘    └───────────┬──────────────────────┘
           │ apex.results               │ apex.results
┌──────────▼─────────────────────────────▼─────────────────────┐
│  L3 · Memory & State (Rust)                                  │
│  SQLite (single-user, file-based)                          │
│  Vector index for semantic search                            │
│  Append-only audit log                                       │
└─────────────────────────────────────────────────────────────┘
Single-User Simplifications
No user ID in any data structure. The single user is implicit. Database has no users table.
No authentication between layers. Capability tokens still exist for defense-in-depth, but there's no multi-user identity provider.
No network ACLs between services. All services bind to localhost only; single machine deployment.
SQLite file-based. No PostgreSQL, no connection pooling complexity. User's data lives in ~/.apex/data/.
5. Security Architecture
5.1 Threat Model for Single-User Deployment
Primary threats:
Malicious code execution — LLM generates harmful code, user accidentally runs it
Supply chain attacks — compromised dependency steals user's local files
Prompt injection — malicious message content tricks agent into harmful actions
Data exfiltration — rogue skill sends user's code/documents to external server
Non-threats (out of scope):
Multi-tenant isolation (no other users)
Privilege escalation between users (single user)
Side-channel attacks from other tenants (single machine)
5.2 Firecracker Micro-VM Isolation (L5)
Every Deep task runs in a dedicated Firecracker micro-VM:
Dedicated kernel — Linux 6.6 minimal config, no shared host kernel
No network by default — outbound internet requires explicit domain allowlist in task config
Ephemeral storage — overlay filesystem, destroyed with VM
Resource limits — 1 vCPU, 512MB RAM, 5-minute max runtime (configurable)
125ms cold start — acceptable for non-interactive Deep tasks
Development fallback: gVisor (runsc) for machines without KVM (macOS, some cloud VMs). 15% performance penalty, equivalent isolation properties.
5.3 Capability Tokens (Simplified)
PASETO v4 tokens signed by L2, scoped per-task:
JSON
Copy
{
  "task_id": "ulid",
  "tier": "T1|T2|T3",
  "allowed_skills": ["code.generate", "music.generate"],
  "allowed_domains": ["api.openai.com", "api.replicate.com"],
  "expires_at": "timestamp",
  "max_cost_usd": 5.00
}
No user ID field. No workspace ID. Single-user deployment.
5.4 Permission Tiers (Single-User Context)
Table
Tier	Actions	Gate
T0	Read-only queries, file reads, search	None
T1	File writes in workspace, draft generation	In-chat confirmation tap
T2	External API calls (sending messages, posting), git push	Typed confirmation: confirm: [action]
T3	Destructive operations (rm -rf, production deploys, cost >$10)	TOTP verification + 5-minute delay
Single-user implication: T3 gates are speed bumps against the user's own mistakes or a compromised account, not protection against malicious other users.
5.5 Prompt Injection Defense
Acknowledged limitation: Complete prompt injection prevention is unsolved. APEX implements layered defense:
Pre-scan — regex/keyword detection of known jailbreak patterns (DAN, "ignore previous instructions", etc.)
Context separation — system prompts never concatenated with user input; tool outputs in delimited blocks
VM isolation — successful injection can only affect the micro-VM, not host
No silent execution — T1+ actions always require confirmation, injection can't bypass this without user interaction
6. Memory & Storage
6.1 Single-User SQLite Architecture
One SQLite database file: ~/.apex/data/apex.db
Tables:
tasks — task history, outcomes, routing class
audit_log — append-only, tamper-evident (hash chain)
conversation_history — message turns, 90-day retention
skill_registry — installed skills, versions, health status
vector_cache — embeddings for semantic search (using sqlite-vec extension)
preferences — user settings, tone profile, API keys (encrypted)
No GDPR complexity: User owns their database file. Delete the file to erase all data. No data residency concerns (data never leaves user's machine unless they configure external APIs).
6.2 Memory Tiers
Table
Tier	Storage	TTL	Contents
Working	Micro-VM RAM	Task lifetime	Active execution context
Session	SQLite	7 days inactive	Conversation turns, corrections
Project	SQLite + vectors	90 days	Task outcomes, code generated, patterns
Long-term	SQLite	Permanent	Preferences, trusted domains, promoted tools
6.3 English-Only Language Processing
No i18n framework. All system prompts, error messages, and skill interfaces are English.
No translation layer. If user inputs non-English text, LLM handles it natively; APEX doesn't care.
No locale-aware formatting. Dates/times in ISO 8601 or user preference stored in preferences.
No right-to-left text support. UI assumes left-to-right.
7. Skill Registry
7.1 Curated Skill Set (~50 Total)
Instead of inheriting OpenClaw's 5,700+ unvetted skills, APEX implements a curated set focused on creative/technical workflows:
Software Development (15 skills)
Table
Skill	Function	Tier
code.generate	Generate code from description	T1
code.review	Review code for bugs/style	T0
code.refactor	Refactor code structure	T1
code.document	Generate documentation	T1
code.test	Generate and run tests	T2
git.commit	Stage, commit, push changes	T2
repo.search	Search codebases semantically	T0
deps.check	Check for vulnerable dependencies	T0
shell.execute	Run shell commands (sandboxed)	T2
docker.build	Build container images	T2
api.design	Design API schemas (OpenAPI)	T1
db.schema	Design database schemas	T1
db.migrate	Generate migration scripts	T2
ci.configure	Generate CI/CD configurations	T1
docs.read	Read and summarize documentation	T0
AI Music Generation (10 skills)
Table
Skill	Function	Tier
music.generate	Generate music from text prompt	T1
music.extend	Extend/continue existing track	T1
music.remix	Remix existing audio	T1
music.stem_separate	Separate audio into stems	T1
music.master	Master audio for distribution	T1
music.analyze	Analyze key, BPM, structure	T0
music.convert	Convert formats (wav/mp3/flac)	T1
music.tag	Generate ID3/metadata tags	T1
music.distribute	Upload to distribution platforms	T2
music.collab	Prepare stems for collaboration	T1
AI Video Generation (8 skills)
Table
Skill	Function	Tier
video.generate	Generate video from text/image	T1
video.edit	Edit/trim/combine clips	T1
video.upscale	Upscale resolution	T1
video.style_transfer	Apply artistic styles	T1
video.caption	Generate captions/subtitles	T1
video.thumbnail	Generate thumbnail images	T1
video.render	Render final output	T1
video.publish	Upload to platforms (YouTube, etc.)	T2
Script Writing (8 skills)
Table
Skill	Function	Tier
script.outline	Generate story outlines	T0
script.draft	Write scene drafts	T1
script.dialogue	Generate dialogue	T1
script.format	Format to industry standard	T1
script.analyze	Analyze structure/beats	T0
script.convert	Convert between formats (FDX, PDF)	T1
script.collab	Prepare for collaboration	T1
script.register	Register with WGA/copyright	T2
Marketing (9 skills)
Table
Skill	Function	Tier
copy.generate	Generate marketing copy	T0
copy.ab_test	Generate A/B test variants	T0
seo.optimize	Optimize content for SEO	T1
social.schedule	Schedule social posts	T2
social.analyze	Analyze engagement metrics	T0
email.campaign	Draft email campaigns	T1
ad.copy	Generate ad copy	T0
brand.voice	Maintain brand voice consistency	T1
competitor.monitor	Monitor competitor mentions	T0
7.2 Skill Implementation
Each skill is a TypeScript module implementing:
TypeScript
Copy
interface Skill {
  name: string;
  version: string;
  tier: PermissionTier;
  inputSchema: z.ZodSchema;
  outputSchema: z.ZodSchema;
  execute(ctx: TaskContext, input: unknown): Promise<SkillResult>;
  healthCheck(): Promise<boolean>;
}
No auto-inheritance. Skills are hand-written, tested, and versioned with APEX. Upstream OpenClaw skill changes are ignored; APEX maintains its own fork of messaging adapters only.
8. Task Router — L2
8.1 Rule-Based Classification
Three routing classes determined by pattern matching:
Instant — Answer from cache or deterministic rule
Keywords: "what is", "how do I", "status", "list", "show me"
No file modification, no external API calls
Target: <100ms
Shallow — Single skill invocation
Pattern: One clear intent, single skill can handle
Examples: "generate a Python function", "write marketing copy", "separate this audio into stems"
Target: <3 seconds
Deep — Multi-step autonomous execution
Pattern: Complex goal requiring planning, multiple tools, file manipulation
Examples: "build a website for my band", "create a marketing campaign for this script"
Routed to L5 Firecracker VM
Target: minutes to hours (async with progress updates)
8.2 Cost Estimation
Before dispatching Deep tasks, L2 estimates cost based on:
Model selection (GPT-4o vs Claude 3.5 Sonnet vs local model)
Estimated steps (from prompt complexity heuristic)
User's configured monthly budget (hard stop at limit)
Single-user implication: User configures their own API keys and budgets. APEX tracks spending but doesn't enforce platform-wide limits.
9. Messaging Gateway — L1
9.1 Supported Channels (Day 1)
Table
Channel	Status	Notes
Slack	✅	Primary development target
Discord	✅	Rich embed support for media previews
Telegram	✅	Bot API, good for mobile
WhatsApp	✅	Business API requires Meta approval
Email (IMAP/SMTP)	✅	For formal communications
REST API	✅	Localhost-only, for integrations
Removed from v2.0: Signal (no API), iMessage (no API), Microsoft Teams (enterprise focus, not single-user).
9.2 Channel Adapters
Each adapter normalizes to TaskRequest:
TypeScript
Copy
interface TaskRequest {
  messageId: string;
  channel: 'slack' | 'discord' | 'telegram' | 'whatsapp' | 'email' | 'api';
  threadId: string;  // conversation grouping
  author: string;    // display name
  content: string;   // text content
  attachments: Attachment[]; // files, images, audio
  timestamp: Date;
}
10. Execution Engine — L5
10.1 Firecracker VM Management
VM Pool: 2-3 pre-warmed idle micro-VMs maintained for instant task start. Pool size configurable.
Per-task VM:
Fresh VM from snapshot (not reused between tasks)
Agent Zero Python environment pre-installed
NATS client for L3 communication
Resource limits enforced by cgroups inside VM
10.2 Agent Loop
Standard Agent Zero loop: plan → act → observe → reflect
Modifications for APEX:
Tool outputs streamed to L3 via NATS (survives VM crashes)
Network egress blocked by default; allowlist from capability token
Budget checks every step (LLM calls, API calls, time)
10.3 Dynamic Tool Promotion
When agent creates a useful tool:
Tool saved to candidate_tools table in L3
After 5+ successful uses across different tasks, flagged for review
Manual review required — APEX developer inspects code, approves or rejects
Approved tools added to skill registry as promoted category
No auto-promotion. Human review is mandatory gate.
11. Error Handling & Observability
11.1 Failure Modes
L3 Unavailable: L2 queues tasks in memory (100 task limit), retries every 5 seconds. If persistent, tasks fail with error message to user.
L5 VM Crash: Task marked FAILED in L3. User receives partial results + explanation. Intermediate results preserved.
Skill Failure: Circuit breaker opens after 3 consecutive failures. Skill marked DEGRADED in registry. Fallback skill attempted if available.
11.2 Observability
Single-user dashboard (local web UI on localhost:8080):
Task history with full traces
Cost tracking (monthly spend, per-task cost)
Skill health status
VM pool metrics
Audit log viewer
No external telemetry. No phone-home. All metrics local.
12. Repository Structure
plain
Copy
apex/
├── core/                    # Rust workspace
│   ├── router/              # L2 Task Router
│   ├── memory/              # L3 Memory service
│   └── security/            # Capability tokens, audit
├── gateway/                 # L1 Messaging (TypeScript)
│   └── adapters/
│       ├── slack/
│       ├── discord/
│       ├── telegram/
│       ├── whatsapp/
│       └── email/
├── skills/                  # L4 Skill Runner (TypeScript)
│   ├── core/                # Development skills
│   ├── music/               # AI music generation
│   ├── video/               # AI video generation
│   ├── script/              # Script writing
│   ├── marketing/           # Marketing/copywriting
│   └── promoted/            # User-generated tools
├── execution/               # L5 Execution (Python)
│   ├── agent/               # Agent Zero fork
│   ├── tools/               # Tool library
│   └── vm-images/           # Firecracker base images
├── upstream/                # Fork tracking
│   ├── openclaw-messaging/  # Only messaging layer
│   └── agent-zero/          # Full fork
├── infra/
│   ├── docker-compose.yml   # Local dev dependencies
│   └── firecracker/         # VM management scripts
└── docs/
    ├── skills/              # Skill development guide
    └── api/                 # Internal API docs
13. Phased Delivery Plan
Team: 4 senior engineers (1 Rust, 2 TypeScript, 1 Python/DevOps). Timeline: 12 months to v1.0.
Phase 1 — Core & Messaging (Months 1-4)
Rust L2/L3 core with SQLite
NATS single-node messaging
Slack adapter only
5 core development skills (code.generate, code.review, shell.execute, git.commit, docs.read)
Firecracker VM harness (gVisor fallback)
Local observability dashboard
Deliverable: Slack bot that can generate code, execute in sandbox, commit to git.
Phase 2 — Creative Skills (Months 5-8)
Discord, Telegram, WhatsApp adapters
Music generation skills (integration with Suno, Udio, or local models)
Video generation skills (integration with Runway, Pika, or local models)
Script writing skills
Marketing/copywriting skills
Decision Journal feature
Cost tracking
Deliverable: Multi-channel creative assistant with media generation capabilities.
Phase 3 — Polish & Scale (Months 9-12)
Email adapter
REST API for integrations
Workflow templates (YAML-based automation)
Self-healing skill recovery
Security audit
Documentation
Release v1.0
14. Risk Register
Table
Risk	Likelihood	Impact	Mitigation
OpenClaw API instability	High	Medium	Only forked messaging layer; minimal surface area
Firecracker KVM unavailable	Medium	High	gVisor fallback; document macOS limitations
AI music/video API changes	Medium	Medium	Abstract provider interface; multiple backend support
LLM prompt injection	High	Medium	Layered defense; VM isolation; no silent execution
Single-user scope creep	Medium	High	Explicit "not multi-user" in all docs; reject PRs adding multi-tenancy
Dependency vulnerability	Medium	High	Weekly dependency audit; automated CVE scanning
12-month timeline overrun	Medium	Medium	Cut Phase 3 features before Phase 1-2 scope; ship MVP
APEX Design Specification · v3.0 · Single-User Edition
Parent repositories: OpenClaw · Agent Zero
While we do not want or need to inherit OpenClaw's 5000+ skills, we do need a flexible skill system, like the parents have. Both parents have a UI, APEX needs one as well
Edit
Copy
Share
I'll analyze the specification and provide a complete v4.0 that addresses: flexible skill system architecture (without inheriting 5000+ skills), and a built-in UI requirement.
Search
OpenClaw web UI interface features 2025 Agent Zero web UI dashboard capabilities skill runner architecture plugin system design patterns local AI assistant web UI frameworks 2024
25 results
Based on my research, I now have a clear picture:
OpenClaw UI: Has a local web UI (localhost dashboard) that serves as an "agent control dashboard" — more than just chat, includes tool configuration, logs, task viewing. However, it's described as having a "black box" problem with poor visibility of system status.
Agent Zero UI: Has a sophisticated Web UI with real-time streaming, process groups, file browser, code editor, memory dashboard, scheduler, WebSocket infrastructure. This is a substantial UI feature set.
Key insight: Both parents have significant web UIs, and the user is correct that APEX needs one too. The v3.0 spec completely omitted a UI layer, which is a major gap.
Now I'll write the complete v4.0 specification with:
Flexible skill system — plugin architecture, SKILL.md standard, hot-reloading
Built-in Web UI — React-based, single-page application, real-time via WebSocket
Single-user, English-only, curated skills as requested
APEX — Autonomous Platform for Execution & Communication
Architecture & Technical Design Specification · v4.0 · Single-User Creative Edition
Parent Systems
OpenClaw — Personal AI assistant, messaging-native, 200k+ stars (viral growth, ~4 months old), experimental maturity, TypeScript
Agent Zero — Containerised autonomous agent framework, 13k+ stars, experimental-grade, Python, sophisticated Web UI
Document Status: Architectural specification for single-user, self-hosted deployment with built-in Web UI and flexible skill plugin system. Multi-tenancy explicitly out of scope.
Changelog from v3.0
v4.0 adds the missing Web UI layer and formalizes the flexible skill system:
Added L6 · Web UI Layer. Previous versions omitted a UI, assuming messaging-only interaction. Both parents have sophisticated web interfaces; APEX requires one too. New React-based SPA with real-time WebSocket connection to core.
Formalized skill plugin architecture. Replaced static skill list with dynamic SKILL.md standard (inherited from Agent Zero's approach), hot-reloading skill registry, and skill marketplace framework (user-curated, not inherited from OpenClaw).
Unified interface model. Web UI and messaging gateways are both "clients" — they consume the same L2 API via different transports (WebSocket vs. NATS). Core remains transport-agnostic.
Skill development kit. Documentation and tooling for users to author custom skills in TypeScript or Python, with local testing harness.
Table of Contents
Executive Summary
Parent System Assessment
Build Environment
High-Level Architecture
Skill System Architecture
Web UI Layer — L6
Security Architecture
Memory & Storage
Task Router — L2
Messaging Gateway — L1
Execution Engine — L5
Error Handling & Observability
Repository Structure
Phased Delivery Plan
Risk Register
1. Executive Summary
APEX is a single-user autonomous creative assistant combining secure code execution, AI media generation, and content creation tools. It is designed for individual developers, musicians, video creators, writers, and marketers who want a local-first AI agent with both messaging and web interfaces.
Core capabilities:
Creative/technical skill set — software development, AI music generation, AI video generation, script writing, marketing (curated, extensible)
Dual interface — Web UI (primary) + messaging integrations (Slack, Discord, Telegram, WhatsApp, Email)
Secure execution — Firecracker micro-VMs for autonomous code execution
Flexible skill system — SKILL.md standard, hot-reload, user-authored plugins
Single-tenant — one user, one database, full local control
What this is not:
Multi-user SaaS platform
General-purpose assistant (no travel booking, no restaurant reservations)
Enterprise team solution
Cloud-hosted service
2. Parent System Assessment
OpenClaw — Reality Check (March 2026)
The hype: 200,000+ GitHub stars in 84 days (fastest-growing repo in history), Super Bowl commercial, founder joining OpenAI.
The reality:
Age: ~4 months old (launched November 2025). Extreme youth = API instability, undiscovered security issues, potential abandonment risk.
Skills: "5,700+ skills" exist but quality is inconsistent; many are simple API wrappers.
UI: Local web dashboard (localhost) for agent control, configuration, logs. Described as having "black box" visibility problems.
Architecture: Messaging-first, shallow skills (not autonomous agents).
APEX strategy: Fork only the messaging adapter patterns. Ignore the skill ecosystem. Do not inherit upstream skills due to quality/instability concerns.
Agent Zero — Reality Check
Strengths:
Mature Web UI with real-time streaming, process groups, file browser, code editor, memory dashboard, scheduler
SKILL.md standard for portable, structured agent capabilities
WebSocket infrastructure for UI synchronization
Autonomous execution with tool generation
APEX strategy: Adopt SKILL.md standard for APEX skill system. Fork execution loop and UI patterns. Replace Docker with Firecracker for isolation.
3. Build Environment
Language Assignments
Table
Subsystem	Language	Justification
Core Daemon / Router / Memory	Rust	Security-critical; memory safety; Tokio async
Messaging Gateway / Skill Runner	TypeScript (Node.js)	Skill ecosystem; messaging adapters; UI API
Web UI	React + TypeScript	Modern SPA framework; Agent Zero UI patterns
Execution Agent (sandboxed)	Python	Agent Zero compatibility; AI/ML tools
VM Orchestration	Rust	Firecracker SDK integration; resource management
Build Tooling
plain
Copy
Rust:        Cargo workspace · Tokio · Axum · sqlx · firecracker-rs SDK
TypeScript:  pnpm monorepo · Fastify · Zod · Vitest · tsup (bundling)
Web UI:      React 18 · Vite · Tailwind CSS · Radix UI · WebSocket client
Python:      Poetry · FastAPI · pytest · ruff · mypy
VMM:         Firecracker (production) · gVisor (development)
Messaging:   NATS JetStream (single-node) · WebSocket (UI real-time)
4. High-Level Architecture
APEX runs as a single daemon with internal service separation. The Web UI is a first-class interface layer, not an afterthought.
plain
Copy
┌─────────────────────────────────────────────────────────────────────┐
│  L6 · Web UI (React SPA)                                            │
│  Real-time chat · Skill marketplace · File browser · Memory viewer  │
│  Workflow editor · Cost dashboard · Settings                        │
└──────────────────────┬────────────────────────────────────────────────┘
                       │  WebSocket → L2 API
┌──────────────────────▼─────────────────────────────────────────────┐
│  L1 · Messaging Gateway (TypeScript)                               │
│  Slack · Discord · Telegram · WhatsApp · Email                       │
└──────────────────────┬───────────────────────────────────────────────┘
                       │  TaskRequest → NATS: apex.tasks.inbound
┌──────────────────────▼─────────────────────────────────────────────┐
│  L2 · Task Router (Rust)                                           │
│  Intent classification · Permission enforcement · Cost estimation  │
│  Routing: Instant / Shallow / Deep                                   │
└──────────┬────────────────────────────┬──────────────────────────────┘
           │ apex.tasks.shallow       │ apex.tasks.deep
┌──────────▼──────────┐    ┌───────────▼─────────────────────────────┐
│  L4 · Skill Runner  │    │  L5 · Execution Engine                 │
│  (TypeScript)       │    │  (Python in Firecracker microVM)       │
│  SKILL.md plugins   │    │  Agent Zero loop · Autonomous tools    │
│  Hot-reload registry│    │  Dynamic tool generation               │
└──────────┬──────────┘    └───────────┬─────────────────────────────┘
           │ apex.results               │ apex.results
┌──────────▼─────────────────────────────▼────────────────────────────┐
│  L3 · Memory & State (Rust)                                        │
│  SQLite · Vector search · Audit log · Skill registry               │
└─────────────────────────────────────────────────────────────────────┘
Interface Unification
Web UI and messaging gateways are both clients of the core API:
Web UI: Connects via WebSocket (bidirectional streaming, real-time updates)
Messaging: Connects via NATS (async message bus)
Core API (L2): Transport-agnostic; exposes same operations regardless of client type
This allows features built for Web UI (file browser, memory viewer) to be accessible via messaging commands, and vice versa.
5. Skill System Architecture
APEX implements a flexible, plugin-based skill system using the SKILL.md standard. Skills are not inherited from OpenClaw; they are curated, user-installable, and extensible.
5.1 SKILL.md Standard
Each skill is a directory containing:
plain
Copy
skill-name/
├── SKILL.md           # Manifest and behavior definition
├── schema.json        # Input/output JSON schemas
├── src/
│   ├── index.ts       # TypeScript implementation (L4 skills)
│   └── ...            # Or Python (L5 tools)
├── tests/
│   └── skill.test.ts  # Test suite
└── assets/            # Icons, examples, documentation
SKILL.md format:
Markdown
Fullscreen 
Download 
Fit
Code
Preview
Purpose
Example 1: Simple function
Examples
If description is ambiguous, asks clarifying question
If language is unsupported, returns error with supported list
Error Handling
code.generate
5.2 Skill Registry
Hot-reload capability: Skills are loaded dynamically without core restart:
Discovery: Skills directory scanned on startup and file watcher monitors for changes
Validation: Schema validation, permission tier checks, dependency resolution
Registration: Valid skills added to in-memory registry (SQLite-backed for persistence)
Activation: Available immediately for task routing
Skill sources:
Built-in: Core skills shipped with APEX (coding, music, video, script, marketing)
User-installed: Downloaded from user-curated marketplace or local development
Generated: Promoted from L5 autonomous tool generation (after human review)
5.3 Curated Built-in Skills
Core set (~50 skills) organized by domain:
Development (15 skills): code.generate, code.review, code.refactor, code.document, code.test, git.commit, repo.search, deps.check, shell.execute, docker.build, api.design, db.schema, db.migrate, ci.configure, docs.read
AI Music (10 skills): music.generate, music.extend, music.remix, music.stem_separate, music.master, music.analyze, music.convert, music.tag, music.distribute, music.collab_prep
AI Video (8 skills): video.generate, video.edit, video.upscale, video.style_transfer, video.caption, video.thumbnail, video.render, video.publish
Script Writing (8 skills): script.outline, script.draft, script.dialogue, script.format, script.analyze, script.convert, script.collab_prep, script.register
Marketing (9 skills): copy.generate, copy.ab_test, seo.optimize, social.schedule, social.analyze, email.campaign, ad.copy, brand.voice, competitor.monitor
5.4 Skill Development Kit
Users can author custom skills:
Local testing harness: apex skill test ./my-skill spins up isolated test environment
Debug mode: Step-through execution, mock inputs, inspection of intermediate states
Publishing: Export as tarball or submit to user-curated marketplace (manual review)
6. Web UI Layer — L6
APEX includes a built-in React web interface served by the core daemon. This is not optional; it is a primary interaction mode.
6.1 UI Architecture
plain
Copy
┌─────────────────────────────────────────┐
│  React SPA (Vite + TypeScript)          │
│  Served by L2 at localhost:8080         │
├─────────────────────────────────────────┤
│  Real-time Layer (WebSocket)            │
│  - Bi-directional streaming             │
│  - Task progress updates                │
│  - Agent thought streaming              │
├─────────────────────────────────────────┤
│  State Management (Zustand)             │
│  - Local UI state                       │
│  - Optimistic updates                   │
│  - Sync with server state               │
├─────────────────────────────────────────┤
│  Feature Modules                        │
│  - Chat interface                       │
│  - Skill marketplace                    │
│  - File browser/editor                  │
│  - Memory viewer                        │
│  - Workflow visualizer                  │
│  - Cost dashboard                       │
│  - Settings                             │
└─────────────────────────────────────────┘
6.2 Core UI Features
Chat Interface:
Message history with markdown rendering
Code blocks with syntax highlighting, copy button
File attachments (drag-and-drop)
Tool call visualization (expandable process groups)
Real-time streaming of agent responses
Intervention capability (pause, edit, redirect)
Skill Marketplace:
Browse built-in skills by category
Install/remove user skills
Skill detail view (SKILL.md rendered as documentation)
Test skill with example inputs
File Browser:
Navigate workspace directory tree
View and edit files with Monaco Editor (VS Code core)
Image/audio/video previews
Git status indicators
Memory Viewer:
Search conversation history
Browse project memory (90-day window)
Manage long-term preferences
Delete specific memories (GDPR-style control for local data)
Workflow Visualizer:
YAML workflow editor with live preview
Graph view of step dependencies
Execution tracing (see which steps ran, in what order)
Schedule configuration (cron expressions)
Cost Dashboard:
Monthly spend tracking
Per-task cost breakdown
Model usage statistics
Budget configuration and alerts
Settings:
API key management (encrypted in SQLite)
Channel configuration (connect Slack, Discord, etc.)
Permission tier overrides (per-skill customization)
VM resource limits (CPU, memory, timeout)
Appearance (themes, density)
6.3 Real-Time Synchronization
WebSocket connection enables features impossible in pure messaging:
Live agent thought streaming: See the agent's planning process, tool selection reasoning, intermediate results as they happen (not just final output)
Progress bars: Long-running tasks (video generation, code compilation) show real-time progress
Interactive approvals: T2/T3 actions pop modal confirmations immediately, not async messages
Collaborative debugging: User can pause agent, inspect state, modify context, resume
7. Security Architecture
7.1 Single-User Threat Model
Primary threats:
Malicious code execution via LLM-generated code
Supply chain attacks in dependencies
Prompt injection via crafted input
Data exfiltration from sandboxed execution
Security boundaries:
Firecracker micro-VMs: Kernel-level isolation for L5 execution
Capability tokens: Scoped, time-limited permissions per task
Permission tiers: T0 (silent) → T1 (tap) → T2 (type) → T3 (TOTP + delay)
7.2 Web UI Security
Localhost-only binding: UI not exposed to network by default; configurable bind address for remote access (with warning)
No authentication: Single-user system; whoever can access localhost owns the instance
CSP headers: Strict Content-Security-Policy on UI assets
WebSocket origin validation: Reject connections from unexpected origins
8. Memory & Storage
8.1 Single-User SQLite
One database: ~/.apex/data/apex.db
Tables:
tasks — task history, routing, outcomes
messages — conversation turns (WebSocket + messaging unified)
audit_log — append-only, tamper-evident
skills — installed skill registry
memories — vector embeddings for semantic search
preferences — user settings, encrypted API keys
workflows — saved workflow definitions
8.2 Memory Tiers
Table
Tier	Storage	TTL	Contents
Working	VM RAM	Task lifetime	Active execution context
Session	SQLite	7 days inactive	Conversation turns
Project	SQLite + vectors	90 days	Generated artefacts, patterns
Long-term	SQLite	Permanent	Preferences, workflows, promoted tools
9. Task Router — L2
9.1 Routing Classes
Instant: Cached answers, deterministic rules (<100ms)
Shallow: Single skill invocation (<3 seconds)
Deep: Multi-step autonomous execution (Firecracker VM, minutes to hours)
9.2 Cost Intelligence
Before Deep dispatch:
Estimate cost across candidate models/paths
Check user budget (soft warning at 80%, hard stop at 100%)
Route to lowest-cost path meeting quality threshold
10. Messaging Gateway — L1
Channels: Slack, Discord, Telegram, WhatsApp, Email, REST API.
Unified with Web UI: Messages from any channel appear in Web UI chat history; responses routed back to originating channel.
11. Execution Engine — L5
Firecracker micro-VMs per task:
125ms boot time
Dedicated kernel (no host sharing)
Network allowlist from capability token
Resource limits: 1 vCPU, 512MB RAM, 5-minute default timeout
12. Error Handling & Observability
Failure modes: L3 unavailable (queue + retry), L5 crash (partial result preservation), skill failure (circuit breaker).
Observability: Prometheus metrics endpoint (localhost:9090), structured JSON logs, WebSocket-based real-time status in UI.
13. Repository Structure
plain
Copy
apex/
├── core/                    # Rust
│   ├── router/              # L2 Task Router
│   ├── memory/              # L3 Memory service
│   ├── security/            # Tokens, audit
│   └── vm/                  # Firecracker orchestration
├── gateway/                 # TypeScript
│   ├── messaging/           # L1 adapters
│   └── skills/              # L4 Skill Runner
├── skills/                  # Built-in skills (SKILL.md)
│   ├── core/                # Development
│   ├── music/               # AI music
│   ├── video/               # AI video
│   ├── script/              # Script writing
│   ├── marketing/           # Marketing
│   └── registry/            # Skill marketplace index
├── ui/                      # React Web UI (L6)
│   ├── src/
│   ├── public/
│   └── package.json
├── execution/               # Python (L5)
│   ├── agent/               # Agent Zero fork
│   └── tools/
├── sdk/                     # Skill Development Kit
│   ├── templates/           # Skill scaffolding
│   └── test-harness/        # Local testing
├── upstream/                # Fork tracking
│   └── agent-zero/
└── infra/
    └── docker-compose.yml
14. Phased Delivery Plan
Team: 5 senior engineers (2 Rust, 2 TypeScript/React, 1 Python). Timeline: 14 months.
Phase 1 — Core + UI Foundation (Months 1-4): Rust core, SQLite L3, WebSocket server, React UI shell, basic chat, 5 core dev skills.
Phase 2 — Creative Skills + Messaging (Months 5-8): Music/video/script/marketing skills, Slack/Discord/Telegram adapters, file browser, memory viewer.
Phase 3 — Skill System + Polish (Months 9-11): SKILL.md standard, hot-reload registry, skill marketplace UI, workflow visualizer, cost dashboard.
Phase 4 — Hardening (Months 12-14): Firecracker integration, security audit, WhatsApp/Email adapters, documentation, v1.0 release.
15. Risk Register
Table
Risk	Likelihood	Impact	Mitigation
OpenClaw instability	High	Medium	Minimal dependency; fork only messaging patterns
Firecracker complexity	Medium	High	gVisor fallback; phased integration (Phase 4)
Skill ecosystem quality	Medium	Medium	Curated built-in set; manual marketplace review
UI scope creep	Medium	Medium	Fixed feature set per phase; "MVP first" discipline
14-month timeline	Medium	Medium	Cut Phase 3 marketplace features before core
APEX Design Specification · v4.0 · Single-User Creative Edition