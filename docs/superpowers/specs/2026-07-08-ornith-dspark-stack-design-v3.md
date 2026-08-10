# Ornith + MTP Stack: Engineering Specification v3

**Date:** 2026-07-08
**Status:** Draft v4
**Repository:** `ornith-mtp-stack` (standalone, MIT)
**Status labels:** `[verified]` = confirmed behavior, `[assumed]` = reasonable but unverified, `[goal]` = target, `[design]` = architecture decision

**Revision note:** v4 replaces v3 after a brutal reality review revealed six issues: (1) vLLM natively supports Ornith MTP — the "only engine with MTP support" claim was wrong, though llama.cpp is still the right choice for 16GB GGUF inference; (2) MTP speedup numbers were off — Q4_K_M is ~17% on Blackwell, not 28%, and NVFP4 is ~52%, not 28%; (3) no competitive analysis against Aider/Cline/OpenHands existed — the largest strategic risk; (4) self-scaffolding vs adaptive-scaffold distinction was muddy; (5) vLLM GGUF was migrated to an OOT plugin, not deprecated to die; (6) VRAM budget didn't distinguish bundled vs separate GGUF files. v4 fixes all six.

---

## 1. Vision

Wire DeepReinforce's Ornith 1.0 (self-scaffolding coding agent) with its native MTP speculative decoding head into a single local inference stack where the orchestration layer — not the models — is the differentiating contribution.

**What this stack enables that neither project alone provides:**
- Per-task adaptive scaffold that revises based on execution feedback
- Unified memory (working + episodic + persistent)
- Structured validator-based checkpoint engine
- Tool Runtime with sandbox, permissions, cancellation, rollback
- End-to-end benchmark harness comparing accelerated vs non-accelerated paths

**Important distinction — Ornith's "self-scaffolding" vs this stack's adaptive scaffold:**
Ornith's self-scaffolding is a **training-time** technique: during RL, the model learns to jointly produce both solutions and the task-specific harnesses that guide them. At inference time, Ornith uses a standard agent-loop harness (OpenHands for SWE-Bench, Harbor/Terminus-2 for Terminal-Bench). This stack's Agent Runtime is an **inference-time** adaptive scaffold — it generates, executes, observes, and revises the plan per-step based on execution feedback. These are complementary layers: Ornith was trained to produce better plans within a harness; this stack provides a harness that adapts those plans dynamically. They are not redundant.

---

## 2. Five-Layer Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  APPLICATION LAYER                                               │
│                                                                  │
│  CLI (argparse)    REST API (FastAPI)    Python SDK              │
│  All entry points route through the Agent Runtime.               │
│  No business logic in this layer.                                │
├──────────────────────────────────────────────────────────────────┤
│  AGENT RUNTIME LAYER ── primary contribution                    │
│                                                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │Planner   │ │Executor  │ │Memory        │ │Checkpoint    │   │
│  │·scaffold │ │·tool mgr │ │·working      │ │Engine        │   │
│  │·adapt    │ │·sched    │ │·episodic     │ │·validators   │   │
│  │·revise   │ │·sandbox  │ │·persistent   │ │·recovery     │   │
│  └──────────┘ └──────────┘ └──────────────┘ └──────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  Tool Runtime                                            │    │
│  │  Launch · Permission · Timeout · Sandbox · Rollback · Kill │   │
│  └──────────────────────────────────────────────────────────┘    │
├──────────────────────────────────────────────────────────────────┤
│  INFERENCE LAYER                                                 │
│                                                                  │
│  llama-server (primary) → GGUF via llama.cpp                    │
│  └─ OpenAI-compatible HTTP, streaming, tool-calling (Qwen XML)  │
│  Swappable: llama-server (default), cloud API, or vLLM (future) │
├──────────────────────────────────────────────────────────────────┤
│  ACCELERATION LAYER                                              │
│                                                                  │
│  Ornith MTP self-speculative head (primary, exists today)       │
│  └─ --spec-type draft-mtp via llama.cpp                         │
│  KV cache (FP16) · Flash attention [via llama.cpp]              │
│  DSpark ← future work (when Qwen3.5 drafter ships)             │
├──────────────────────────────────────────────────────────────────┤
│  INFRASTRUCTURE LAYER                                            │
│                                                                  │
│  CUDA 12.8+ · WSL2 Ubuntu · Nix flake · Prometheus metrics     │
│  Structured logging (JSON) · Config (single source of truth)    │
└──────────────────────────────────────────────────────────────────┘
```

**Design invariant:** No layer calls across more than one boundary. The Agent Runtime never talks to CUDA. The Application Layer never talks to llama.cpp.

**Inference engine selection:**
| Engine | When to use | Rationale |
|--------|------------|-----------|
| llama.cpp (default) | GGUF quants, MTP acceleration | Faster batch-1 inference for single-user local. Best MTP support for GGUF on 16GB. |
| vLLM (alternative) | HF-format models with MTP | vLLM natively supports Ornith MTP (+49-57% speedup per protoLabsAI). Requires HF-format model (~19GB BF16) — impractical on 16GB VRAM. Viable if switching to non-GGUF in the future. GGUF via OOT plugin (`vllm-gguf-plugin`). |

---

## 3. Agent Runtime (Primary Contribution)

### 3.1 Adaptive Scaffold Loop

The scaffold is **not** static. It is generated once, then revised after each step.

```
                    ┌──────────────────┐
                    │  Task arrives    │
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
                    │  GENERATE        │
                    │  initial scaffold│
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
             ┌──────┤  EXECUTE step    │
             │      └────────┬─────────┘
             │               │
             │      ┌────────▼─────────┐
             │      │  OBSERVE result  │
             │      └────────┬─────────┘
             │               │
             │      ┌────────▼─────────┐
             │      │  CHECKPOINT      │
             │      │  validator pass? │
             │      └────────┬─────────┘
             │          │    │
             │       yes│    │no
             │          │    │
             │          │  ┌─▼───────────┐
             │          │  │ RETRY left? │───no──→ FAILURE
             │          │  └─┬───────────┘
             │          │    │yes
             │          │  ┌─▼───────────┐
             │          │  │ REVISE      │
             │          │  │ scaffold    │
             │          │  └─┬───────────┘
             │          │    │
             │     ┌────┘    │
             │     │         │
             │  ┌──▼──┐      │
             │  │more │      │
             │  │steps│yes───┘
             │  └──┬──┘
             │     │no
             │  ┌──▼──┐
             │  │DONE │
             │  └─────┘
```

### 3.2 Scaffold Data Model

```python
@dataclass
class Scaffold:
    goal: str                              # original task
    plan: list[Step]                       # ordered steps
    tools: list[str]                       # permitted tools
    memory_config: MemoryConfig            # what to persist
    termination: TerminationCondition      # when to stop
    revision_history: list[Revision]       # how scaffold evolved

@dataclass
class Step:
    id: str
    description: str
    checkpoint: Validator | None           # optional gate
    timeout_secs: int
    retry_policy: RetryPolicy
    depends_on: list[str]                  # step IDs

@dataclass
class RetryPolicy:
    max_attempts: int                      # default 2
    backoff_secs: float                    # default 5.0
    fallback_strategy: str                 # "retry" | "skip" | "abort"

@dataclass
class TerminationCondition:
    max_steps: int                         # default 20
    max_cost_usd: float                    # default 0.50
    max_wall_secs: int                     # default 300
    on_checkpoint_fail: str                # "retry" | "abort"

@dataclass
class Revision:
    step_id: str
    observation: str                       # what happened
    change: str                            # what changed in scaffold
    confidence_delta: float                # -1.0 to +1.0
```

### 3.3 Planner Subsystem

**Responsibilities:**
- Generate initial scaffold from task description
- After each step, decide whether to revise the scaffold
- Maintain confidence score for each remaining step

**generate_scaffold(task) → Scaffold:**
- Sends meta-prompt to Ornith via OpenAI-compatible API
- Meta-prompt describes the scaffold schema and asks for a plan
- Falls back to default scaffold (single-step "solve this") on parse failure

**revise_scaffold(current, step_result) → Scaffold:**
- If checkpoint failed → update retry count, lower confidence
- If step succeeded with unexpected complexity → split into sub-steps
- If pattern detected (repetition, loop, drift) → alter plan
- If 3+ consecutive checkpoint failures → abort (not infinite retry)

### 3.4 Executor Subsystem

**Responsibilities:**
- Dispatch each step to the Tool Runtime or Inference Layer
- Enforce timeouts, cancellation, resource limits
- Collect observations (stdout, stderr, exit code, metrics)
- Route results back to Planner for scaffold revision

**execute_step(step, context) → StepResult:**
```python
@dataclass
class StepResult:
    step_id: str
    status: StepStatus                     # success | fail | timeout | cancelled
    output: str                            # text output
    tools_called: list[ToolCall]
    cost_usd: float
    wall_ms: int
    tokens_in: int
    tokens_out: int                        # includes reasoning + content
    tokens_reasoning: int                  # tokens in <think> blocks, tracked separately
    acceptance_rate: float | None          # from MTP metrics (draft acceptance %)
```

**Reasoning token accounting:** Ornith emits a `<think>` block before every answer. `tokens_out` includes reasoning tokens + content tokens. `tokens_reasoning` tracks the <think> portion separately so TerminationCondition can budget for reasoning overhead. A step with 4096 max_tokens and 2000 reasoning tokens has only 2096 for actual output.

### 3.5 Memory Subsystem

Three-tier memory:

| Tier | Scope | Lifetime | Storage | Max Size |
|------|-------|----------|---------|----------|
| Working | Current task | Task lifetime | In-process dict | 10K tokens |
| Episodic | Session | Session (file) | JSON lines file | 1MB |
| Persistent | Cross-session | Indefinite | SQLite (optional) | Configurable |

**Working Memory:**
- Key-value store (Python dict)
- Planner writes scaffold state, confidence, step results
- Executor reads/writes tool outputs, intermediate data
- Checkpoint Engine reads validation results
- Survives within a single task execution

**Episodic Memory:**
- Append-only log of completed steps
- Each entry: `{step_id, task, scaffold_snapshot, result, timestamp}`
- Written after each step completes
- Used for: post-task analysis, scaffold revision patterns, debugging
- Stored in `~/.ornith-mtp/episodic/{session_id}.jsonl`

**Persistent Memory (future, not MVP):**
- SQLite database with vector embeddings
- Retrieval-augmented scaffold generation
- Not in scope for initial build

### 3.6 Checkpoint Engine

**Not keyword matching.** Each checkpoint specifies a validator.

```python
@dataclass
class Validator:
    type: str                             # "compile" | "pytest" | "regex" | "llm_judge" | "custom"
    args: dict                            # type-specific arguments
    expected: Any                         # expected result
    confidence: float                     # 0.0-1.0 how reliable this validator is
    fail_on_error: bool = True            # if True, validator crash = fail; if False (opt-in), crash = pass

# Built-in validators:
VALIDATORS = {
    "compile":     CompileValidator(),      # compiles code, returns errors
    "pytest":      PytestValidator(),       # runs pytest, returns pass/fail/count
    "regex":       RegexValidator(),        # matches pattern in output
    "schema":      JsonSchemaValidator(),   # validates JSON against schema
    "coverage":    CoverageValidator(),     # minimum code coverage %
    "benchmark":   BenchmarkValidator(),    # performance threshold
    "llm_judge":   LLMJudgeValidator(),     # LLM evaluates quality (highest cost)
    "custom":      PythonValidator(),       # user-supplied Python function
}
```

**Checkpoint resolution:**
1. Run validator with `args`
2. Compare result to `expected`
3. If match → pass
4. If mismatch → fail → trigger retry/revision
5. If validator itself errors → **fail-closed by default** (`fail_on_error=True`). A crashing validator is treated as a checkpoint failure, not silently accepted. Use `fail_on_error: False` per-validator to opt into fail-open behavior for inherently flaky validators.

### 3.7 Tool Runtime

```python
@dataclass
class ToolSpec:
    name: str
    description: str
    timeout_secs: int                      # per-call timeout
    sandbox: SandboxLevel                  # none | restricted | isolated
    permissions: list[str]                 # "filesystem.read" | "network" | "shell"
    allowed_args: list[str] | None         # None = any
    deny_args: list[str]                   # pattern-based, e.g. "rm -rf"

@dataclass
class ToolCall:
    tool: str
    args: dict
    timeout_secs: int
    call_id: str
    cancel_token: threading.Event          # shared cancellation signal

@dataclass
class ToolResult:
    call_id: str
    status: ToolStatus                     # success | timeout | cancelled | permission_denied | error
    stdout: str
    stderr: str
    exit_code: int
    wall_ms: int
```

**Tool Runtime responsibilities:**
- Launch tool in appropriate sandbox (subprocess for MVP)
- Enforce timeout via `threading.Timer` + process kill
- Check permissions before execution
- Support cancellation via shared `cancel_token`
- Record all calls to audit log
- Rollback: for destructive tools, record undo state (MVP: just log)

**Built-in tools (MVP):**
| Tool | Sandbox | Timeout | Permissions |
|------|---------|---------|-------------|
| `file.read` | none | 5s | read-only paths |
| `file.write` | none | 5s | write to cwd only |
| `code.generate` | none | 30s | — |
| `shell.execute` | restricted | 30s | allowlist of commands |
| `web.get` | network | 15s | HTTP(S) only |
| `python.eval` | isolated | 15s | restricted globals |

---

## 4. Inference Layer

### 4.1 Backend Interface

```python
@dataclass
class InferenceConfig:
    provider: str                          # "llamacpp" | "openai" | "vllm"
    base_url: str
    model: str                             # path to GGUF for llamacpp, model name for others
    api_key: str | None
    max_tokens: int
    temperature: float
    top_p: float
    top_k: int
    streaming: bool                        # [design: streaming enabled by default]
    reasoning_parser: str | None           # "qwen3" for Ornith, extracts <think> blocks
    tool_call_parser: str | None           # "qwen3_xml" for Ornith XML tool format
    speculative: SpeculativeConfig | None
```

### 4.2 Backend Selection: llama.cpp (Primary)

Ornith-1.0-9B uses the Qwen3.5 architecture (hybrid: 24 GatedDeltaNet-style linear-attention layers + 8 full-attention layers). This runs through llama.cpp, which has first-class GGUF support and is consistently faster than vLLM for batch-1 local inference.

**Why not vLLM for GGUF on this hardware:**
- vLLM's GGUF support is "highly experimental and under-optimized" per upstream docs
- RFC #39583 proposed deprecation, but GGUF was migrated to an OOT plugin (`vllm-gguf-plugin`, v0.0.3, July 2026), not removed. It remains usable but with performance caveats.
- For batch-1 inference, llama.cpp is consistently faster with GGUF. vLLM GGUF benchmarks show ~93 tok/s vs full-speed through llama.cpp.
- vLLM's +49-57% MTP speedup claim applies to HF-format models on multi-GPU setups, not GGUF on single 16GB GPU.

**Why not vLLM with HF-format Ornith:**
- Ornith 9B in BF16 requires ~19GB — exceeds 16GB VRAM
- No Ornith AWQ or NVFP4 quants exist for vLLM that fit 16GB
- If and when NVFP4 quants ship at 6-7GB, vLLM becomes a competitive alternative

**Why not cloud providers:**
- Ornith is not available on any cloud provider
- DSpark is not available on any cloud provider
- The MTP head is only usable through local inference

**llama-server invocation:**
```bash
llama-server \
  --model Ornith-1.0-9B-Q4_K_M.gguf \
  --host 0.0.0.0 --port 8000 \
  --ctx-size 128000 \
  --n-gpu-layers 99 \
  --parallel 1 \
  --cont-batching \
  --slot-save-path /tmp/llama-slots \
  --tool-call-parser qwen3_xml \
  --reasoning-parser qwen3 \
  --no-mmap
```

Flags explained:
- `--tool-call-parser qwen3_xml`: Parses Qwen3 XML tool-calling format (Ornith-native)
- `--reasoning-parser qwen3`: Extracts `<think>` blocks for separate token accounting
- `--no-mmap`: Required for CUDA offloading on WSL2 (mmap on WSL2 is unreliable)
- `--slot-save-path`: Persist slot state for resumption after crash
- `--ctx-size 128000`: Open llama.cpp bug #23658: MTP acceptance collapses near-zero at ~2048-token-aligned context boundaries. 131072 = 2048 × 64 (in the danger zone). 128000 is not a 2048-multiple. Must verify acceptance rate at this value during integration testing.

**Switching to vLLM (future, non-GGUF):**
When switching to AWQ or NVFP4 quants, swap the backend:
```bash
vllm serve <model> --port 8000 --dtype auto
```
The Agent Runtime's `InferenceConfig.provider` field makes this a config change, not a code change.

### 4.3 Performance Bounds

`[verified]` MTP speculative decoding improves throughput. Actual benchmarks from protoLabsAI on RTX 5070 Ti (Blackwell sm_120), 6 diverse code+general prompts, greedy decoding, -n 200:

| Quant | No MTP (tok/s) | +MTP (tok/s) | Speedup | Acceptance |
|-------|----------------|--------------|---------|------------|
| Q4_K_M | 205.1 | 239 (216-252) | **~16.6%** | ~0.65 @ n-max 3 |
| NVFP4 | 201.5 | 306 (287-330) | **~52%** | ~0.65 @ n-max 3 |

**NVFP4 is roughly 2× the speedup of K-quants on Blackwell** because the verify pass is nearly free on tensor-core FP4 GEMMs, while K-quants pay a ~28% dequantization penalty each verify step. The NVFP4 variant is a separate file (6.6GB vs 5.8GB for Q4_K_M).

For comparison, on Ampere (A6000, no Blackwell tensor cores): Q4_K_M 105 → 145 tok/s (+38%). The Blackwell advantage is the tensor-core FP4 path.

| Condition | Improvement (Q4_K_M) | Improvement (NVFP4) |
|-----------|---------------------|---------------------|
| Long generation (256+ tokens) | 15-20% | 45-55% |
| Short generation (< 50 tokens) | 0-10% | 10-20% |
| Batch size 1 (single user) | 15-20% | 45-55% |
| Batch size 4+ | 10-15% | 30-40% |

---

## 5. Acceleration Layer

### 5.1 Ornith MTP Self-Speculative Head (Primary)

`[verified]` Ornith ships a native MTP (Multi-Token Prediction) self-speculative draft head, KL-distilled directly against Ornith's own hidden states:

**Repository:** `protoLabsAI/Ornith-1.0-9B-MTP` (HuggingFace)
**Integration:** Works through llama.cpp's `--spec-type draft-mtp`, or merged into base model and served via vLLM's `--speculative-config '{"method":"mtp","num_speculative_tokens":3}'`
**NVFP4 variant:** Blackwell-optimized build, ~52% faster than no-MTP baseline on Blackwell

**Bundled vs standalone GGUF files:**
- **Bundled** (`Ornith-1.0-9B-MTP-Q4_K_M.gguf`, 5.8GB): MTP head embedded in same file. Simpler loading.
- **Standalone** (`Ornith-1.0-9B-Q4_K_M.gguf` + `mtp-head/mtp-Ornith-1.0-9B-head-Q8_0.gguf`): ~5.8GB + ~0.5GB, allows head to be unloaded to save VRAM when MTP is disabled.
- **NVFP4 variant** (`Ornith-1.0-9B-MTP-NVFP4.gguf`, 6.6GB): Requires llama.cpp with NVFP4 support (GGML_TYPE_NVFP4, type 40). ~52% speedup on Blackwell.

This spec defaults to the **bundled Q4_K_M** for simplicity. Users may switch to standalone if they want dynamic MTP enable/disable or to NVFP4 if they need maximum speed.

**Invocation:**
```bash
llama-server \
  --model Ornith-1.0-9B-Q4_K_M.gguf \
  --spec-type draft-mtp \
  --spec-draft-model Ornith-1.0-9B-MTP.gguf \
  --spec-draft-n-max 3 \
  --host 0.0.0.0 --port 8000 \
  --ctx-size 128000 --n-gpu-layers 99 \
  --tool-call-parser qwen3_xml \
  --reasoning-parser qwen3
```

**Why MTP over DSpark on this hardware:**

| Factor | Ornith MTP | DSpark |
|--------|-----------|--------|
| Architecture match | Purpose-trained on Ornith's hidden states | Targets Qwen3 and Gemma4 only. Ornith is Qwen3.5. |
| Training requirement | None (pre-trained, download and run) | DeepSpec pipeline: 8-GPU node, ~38TB storage for target cache |
| VRAM overhead | ~0.5-0.8 GB | ~1.0-2.0 GB (if it existed) |
| Speedup | ~52% (NVFP4), ~17% (Q4_K_M) on Blackwell | Unknown on this architecture |
| Status | Available today | Blocked: no Qwen3.5 drafter exists |

### 5.2 DSpark Integration (Future Work)

DSpark is tracked as a future acceleration path for when:
1. Someone ships a Qwen3.5-compatible DSpark drafter, OR
2. Ornith moves to plain Qwen3 architecture

**Required flags (documented for when it's relevant):**
```bash
# vLLM invocation (not llama.cpp):
vllm serve <model> \
  --speculative-config '{"method":"dspark","num_speculative_tokens":3}' \
  --kv-cache-dtype fp8
```

Note: The flag is `--speculative-config` (JSON), not `--speculative-model dspark`. (v2 had the prose description wrong — the code block was correct.)

**vLLM GGUF plugin status (2026-07):** RFC #39583 proposed deprecation, but GGUF was migrated to an OOT plugin (`vllm-gguf-plugin`, v0.0.3, July 5 2026), actively maintained by the vLLM project. The plugin is installed via `uv pip install vllm-gguf-plugin`. GGUF remains viable in vLLM, but for batch-1 single-user inference, llama.cpp is consistently faster and has better MTP support for GGUF. vLLM's strength is high-concurrency serving — not the use case for this single-user stack.

### 5.3 VRAM Budget

| Component | Idle | Worst-case | Notes |
|-----------|------|------------|-------|
| Ornith 9B (Q4_K_M) | 5.5 GB | 6.5 GB | GGUF Q4_K_M, ~6 GB base + overhead |
| Ornith MTP draft head | 0.5 GB | 0.8 GB | Small draft head, not a full model |
| KV cache (K-quant) | 1.0 GB | 3.0 GB | Depends on context length (131072 ctx) |
| llama.cpp overhead | 0.2 GB | 0.5 GB | Scheduler, tokenizer, slots, etc. |
| CUDA allocator overhead | 0.5 GB | 1.0 GB | Fragmentation, graph capture |
| **Total** | **7.7 GB** | **11.8 GB** | 16 GB available → **4.2 GB margin at worst** |

**The VRAM margin is healthy (4.2 GB worst-case vs 1.5 GB in v2).** The MTP head consumes ~0.5-0.8 GB vs DSpark's hypothetical 1.0-2.0 GB, and llama.cpp is lighter than vLLM.

**Mitigations if needed (unlikely):**
- Reduce `--ctx-size` to 65536 (saves ~1.5 GB)
- Disable MTP for memory-intensive tasks (saves ~0.5 GB)
- Drop to Q4_0 quant instead of Q4_K_M (saves ~0.5 GB)

---

## 6. Configuration Philosophy

Single source of truth, layered overrides:

```
config/
├── default.yaml              # shipped defaults
├── local.yaml                # user overrides (gitignored)
└── schema.yaml               # JSON Schema for validation
```

**Override order (later wins):**
1. `default.yaml`
2. `local.yaml`
3. Environment variables `ORNITH_*`
4. CLI flags

```yaml
# config/default.yaml
inference:
  provider: llamacpp                         # primary engine
  base_url: http://localhost:8000/v1
  model: ~/.cache/ornith/Ornith-1.0-9B-Q4_K_M.gguf
  max_tokens: 4096
  temperature: 0.6                           # Ornith recommended: 0.6
  top_p: 0.95                                # Ornith recommended: 0.95
  top_k: 20                                  # Ornith recommended: 20
  reasoning_parser: qwen3                    # parse <think> blocks
  tool_call_parser: qwen3_xml                # parse Qwen XML tool format

  speculative:
    enabled: true
    method: mtp                              # "mtp" | "none"
    draft_model: ~/.cache/ornith/Ornith-1.0-9B-MTP.gguf
    num_speculative_tokens: 3                # draft tokens per step; community configs use 2-4. Higher values risk open llama.cpp bug #23302 (token sequence corruption at n-max > 4)

runtime:
  max_steps: 20
  max_cost_usd: 0.50
  default_timeout_secs: 30
  retry_policy:
    max_attempts: 2
    backoff_secs: 5.0

memory:
  working_capacity_tokens: 10000
  episodic_enabled: true
  episodic_dir: ~/.ornith-mtp/episodic
  persistent_enabled: false
```

No `.env`, no CLI-only flags, no scattered config. Everything has a default in `default.yaml`.

---

## 7. Repository Layout

```
ornith-mtp-stack/
├── flake.nix                    # Nix entry point
├── config/
│   ├── default.yaml
│   ├── schema.yaml
│   └── local.yaml.example
├── sdk/                         # Python SDK (pip-installable)
│   ├── pyproject.toml
│   └── src/ornith_mtp/
│       ├── __init__.py
│       ├── client.py            # OpenAI-compatible wrapper
│       ├── models.py            # All @dataclass definitions
│       └── metrics.py           # Acceptance rate, tok/s tracking
├── runtime/                     # Agent Runtime (the core)
│   ├── planner.py               # Scaffold generation + revision
│   ├── executor.py              # Step dispatch, timeout, cancellation
│   ├── memory.py                # Working + episodic memory
│   ├── checkpoint.py            # Validator engine
│   ├── tool_runtime.py          # Tool sandbox, permissions
│   └── cli.py                   # CLI entry point
├── benchmarks/
│   ├── conftest.py              # Benchmark harness
│   ├── test_latency.py
│   ├── test_throughput.py
│   ├── test_acceptance_rate.py
│   ├── test_long_context.py
│   ├── test_batch_scaling.py
│   ├── test_mtp_vs_baseline.py  # MTP on/off comparison (replaces DSpark test)
│   └── REPORT_TEMPLATE.md
├── deployment/
│   ├── nix/                     # Nix packages
│   │   ├── llamacpp.nix
│   │   └── ornith.nix
│   └── scripts/
│       ├── start.sh
│       ├── stop.sh
│       ├── health.sh
│       └── monitor.sh
├── tests/
│   ├── unit/
│   │   ├── test_planner.py
│   │   ├── test_executor.py
│   │   ├── test_memory.py
│   │   ├── test_checkpoint.py
│   │   ├── test_tool_runtime.py
│   │   └── test_config.py
│   ├── integration/
│   │   ├── test_full_loop.py
│   │   ├── test_tool_sandbox.py
│   │   ├── test_checkpoint_validators.py
│   │   └── test_recovery.py
│   └── stress/
│       ├── test_long_context.py
│       ├── test_concurrent.py
│       ├── test_oom_recovery.py
│       └── test_streaming.py
├── examples/
│   ├── 01-basic-chat.py
│   ├── 02-tool-calling.py
│   ├── 03-self-scaffolding.py
│   └── 04-benchmark.py
└── docs/
    ├── ARCHITECTURE.md
    ├── CONFIG.md
    └── BENCHMARKING.md
```

---

## 8. Testing

### 8.1 Unit Tests (offline, no GPU)

| Test file | What it covers |
|-----------|---------------|
| `test_planner.py` | Scaffold generation, JSON parse, fallback, revision logic |
| `test_executor.py` | Step dispatch, timeout enforcement, cancellation, step ordering, reasoning token tracking |
| `test_memory.py` | Working memory CRUD, episodic append/read, size limits |
| `test_checkpoint.py` | All validators including fail-closed behavior on validator error |
| `test_tool_runtime.py` | Permission checks, sandbox launch, timeout, deny patterns, pipe/redirect handling |
| `test_config.py` | YAML parse, override order, schema validation, sampling defaults |

### 8.2 Integration Tests (GPU required)

| Test | What it verifies |
|------|-----------------|
| `test_full_loop.py` | End-to-end scaffold → execute → checkpoint → revise cycle |
| `test_tool_sandbox.py` | Restricted shell executes only allowed commands, rejects forbidden |
| `test_checkpoint_validators.py` | Real compile validator catches errors; validator crash = fail-closed |
| `test_recovery.py` | Process kill + restart, scaffold state persistence across crash |

### 8.3 Stress Tests (GPU required)

| Test | What it verifies |
|------|-----------------|
| `test_long_context.py` | 100K+ token context, memory stability, no OOM |
| `test_concurrent.py` | 4 simultaneous requests, no deadlock, GPU memory stable |
| `test_oom_recovery.py` | Force OOM, verify graceful recovery and error reporting |
| `test_streaming.py` | SSE stream correctness, no truncation, MTP on/off parity |

### 8.4 Speculative Decoding Correctness

MTP claims lossless output (KL-distilled), but this is conditional on `--spec-draft-n-max`. Open llama.cpp bug #23302 reports token sequence corruption at higher n-max values (5-7), where the committed token sequence differs from the non-speculative output. Community consensus uses n-max 2-4, where losslessness holds.

**Verification protocol:**
- Run same prompt 10× with MTP on and 10× with MTP off (at n-max = default 3)
- Compare output tokens (must be identical)
- If any divergence at n-max=3 → log error and disable MTP
- Optionally test n-max=5,7 separately: if divergence detected, reduce n-max rather than disabling entirely
- Track divergence rate over time by n-max setting

---

## 9. Benchmarking

### 9.1 Dimensions

| Metric | Collection | Baseline |
|--------|-----------|----------|
| Latency (p50/p95/p99) | Per-request timer | MTP off |
| Throughput (tok/s) | Token count / wall time | MTP off |
| Acceptance rate | MTP metric from llama.cpp | N/A |
| GPU utilization | `nvidia-smi` polling | MTP off |
| VRAM peak | `nvidia-smi` max | MTP off |
| VRAM fragmentation | Pre/post allocation diff | MTP off |
| First-token latency | Timer to first token | MTP off |
| Time-to-first-tool | Step start to first tool call | MTP off |
| Reasoning token ratio | tokens_reasoning / tokens_out | N/A (characterization) |
| Long-context degradation | Tok/s vs context length curve | MTP off |
| Batch scaling | Tok/s vs concurrent requests | MTP off |
| Prompt scaling | Tok/s vs prompt length | MTP off |
| MTP vs DSpark (future) | Tok/s, acceptance rate | MTP on @ settings |

### 9.2 Harness

`python -m benchmarks.run --mtp on --prompts prompts.json --output results/`

Output: JSON file with all metrics + optional Markdown report.

### 9.3 Key Benchmark: MTP Speedup at Batch Size 1

This is the primary benchmark for this project's acceleration claim. The headline metric is tok/s improvement with MTP enabled vs disabled, measured at batch size 1 with the following sub-tests:

| Sub-test | Prompt tokens | Generation tokens | Expected improvement |
|----------|--------------|-------------------|---------------------|
| Short code gen | ~200 | ~100 | 0-10% |
| Medium refactor | ~1000 | ~500 | 15-20% |
| Long scaffold | ~4000 | ~2000 | 20-28% |
| Tool-calling loop | ~500 | ~200 (per turn) | 10-15% |
| Mixed (chat) | ~300 | ~400 | 15-20% |

---

## 10. Security

### 10.1 Tool Sandbox Levels

| Level | Description | Implementation |
|-------|-------------|----------------|
| `none` | No restriction | Subprocess with same user |
| `restricted` | Allowlist-based | Subprocess + command allowlist + arg deny patterns |
| `isolated` | Full sandbox | Docker container or Firecracker micro-VM (future) |

### 10.2 File System Restrictions
- Working directory: project root only
- Read: any file under working directory
- Write: `outputs/` subdirectory only
- Deny patterns: `.env`, `*.pem`, `*.key`, `.git/`, `config/local.yaml`

### 10.3 Shell Command Allowlist

Allowed: `ls`, `cat`, `head`, `tail`, `grep`, `find`, `wc`, `echo`, `python`, `pip`, `npm`, `node`, `git status`, `git diff`, `git log`, `pwd`, `date`

Denied: `rm`, `mv`, `chmod`, `chown`, `sudo`, `su`, `curl`, `wget`, `ssh`, `telnet`, `nc`, `mkfs`, `mount`, `dd`, `:(){ :|:& };:` (fork bomb), raw `|`/`>`/`<`/`;`/`` ` `` as standalone commands

**Pipes/redirects are disallowed at the raw-subprocess level (`|`, `>`, `<` as standalone commands),** which is intentional MVP conservatism. The agent can still achieve piped workflows through the Tool Runtime's built-in composition (e.g., `file.read` + `grep` as separate steps), just not raw shell piping. This is tracked as a future enhancement: once the sandbox can validate piped commands for safety (no `rm`, no `sudo`, etc.), we can add a `pipe_safe` permission flag that allows `|` within an allowlist context.

### 10.4 Credential Isolation
- No API keys in scaffold, logs, or memory
- API key passed via environment variable only (`ORNITH_API_KEY`)
- If key appears in LLM output → log warning, redact from memory

### 10.5 Network Restrictions (MVP)
- Inference server → localhost only
- `web.get` tool → HTTP(S) to public URLs only
- No inbound network
- No background network processes

---

## 11. Logging

### 11.1 Log Streams

| Stream | Format | Destination | Retention |
|--------|--------|-------------|-----------|
| `audit` | JSON | `logs/audit.ndjson` | 30 days |
| `step` | JSON | `logs/steps.ndjson` | 7 days |
| `tool` | JSON | `logs/tools.ndjson` | 7 days |
| `llm` | JSON | `logs/llm.ndjson` | 1 day |
| `benchmark` | JSON | `logs/benchmarks.ndjson` | Indefinite |
| `performance` | JSON | `logs/perf.ndjson` | 1 day |

### 11.2 Audit Log Entry

```json
{
  "timestamp": "2026-07-08T09:15:00Z",
  "event": "step.execute",
  "task_id": "t-001",
  "step_id": "s-02",
  "tool": "shell.execute",
  "tool_args": ["ls", "-la"],
  "exit_code": 0,
  "wall_ms": 42,
  "user": "local",
  "session_id": "ses-abc"
}
```

---

## 12. Error Classification

| Category | Subtype | Response |
|----------|---------|----------|
| **Recoverable** | Tool timeout | Retry with increased timeout |
| | Network failure | Retry 3× with backoff |
| | GPU OOM | Reduce memory settings, retry |
| | Checkpoint fail | Retry step, then revise scaffold |
| **Transient** | llama-server connection refused | Wait 5s, retry 3× |
| | CUDA context lost | Restart llama-server, retry |
| | Disk full | Clean logs, retry |
| **Fatal** | Model load failure | Report, do not retry |
| | Config parse failure | Report, do not retry |
| | Permission violation | Report, do not retry |
| | Scaffold parse failure (3×) | Use default scaffold |
| **User Error** | Invalid config | Print validation errors, exit |
| | Missing HF token | Print setup instructions, exit |
| | No GPU detected | Print requirements, exit |

---

## 13. Risk Analysis

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Ornith MTP head incompatible with specific Ornith-1.0-9B build | Low | Medium | MTP is purpose-trained on Ornith's hidden states. Test immediately after setup. Fallback: disable speculative decoding. |
| GPU OOM under long context | Medium | Medium | Dynamic KV cache sizing, context length reduction. 4.2 GB margin at worst-case. |
| Scaffold quality poor (LLM generates bad strategy) | Medium | Medium | Default fallback scaffold, revision loop catches failures. |
| 16 GB VRAM insufficient with MTP + full context | Low | High | Can disable MTP per-task (saves ~0.5 GB), reduce context to 65536 (saves ~1.5 GB). 4.2 GB margin makes this unlikely. |
| WSL2 CUDA passthrough unreliable | Low | Medium | Docker fallback, or direct Linux install. |
| HuggingFace rate-limits weight download | Low | Low | Pre-download, Nix cache. |
| MTP speedup < 10% on real workloads | Medium | Low | Always measured; fallback path always available. Primary value is orchestration layer, not acceleration. |
| llama.cpp bug #23302: token sequence corruption at n-max > 4 | Medium | Medium | Default n-max=3 avoids the bug. If we increase n-max, verify losslessness per workload. |
| llama.cpp bug #23658: MTP acceptance collapse at 2048-aligned ctx-size | Medium | Medium | ctx-size=128000 avoids 2048-multiple. Add ctx-size sweep to benchmark harness to map acceptance vs size. |
| Model license changes after download | Low | Medium | Pinned weights in Nix store, MIT license irrevocable. |

---

## 14. Nix Flake

Reference: nixtorch (github:hinriksnaer/nixtorch) for CUDA build tooling. Graham33's nixos-dgx-spark for Blackwell GPU support.

**Build optimization:** `TORCH_CUDA_ARCH_LIST` set to `"12.0"` only (RTX 5070 Ti, Blackwell compute capability). No need to build for Ada (8.9) unless distributing to Ada-GPU users.

```nix
{
  description = "Ornith + MTP local inference stack";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    nixtorch.url = "github:hinriksnaer/nixtorch";
  };

  outputs = { self, nixpkgs, flake-utils, nixtorch }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            cudaSupport = true;
            allowUnfreePredicate = p:
              builtins.elem p.meta.license.shortName [
                "CUDA EULA" "cuDNN EULA"
              ];
          };
        };
      in {
        packages = {
          llamacpp = pkgs.llama-cpp.override {
            cudaSupport = true;
            # MTP speculative decoding requires llama.cpp build with
            # GGML_CUDA=ON and draft model support (default in recent builds)
          };

          ornith-9b = pkgs.fetchurl {
            url = "https://huggingface.co/deepreinforce-ai/Ornith-1.0-9B-GGUF/resolve/main/Ornith-1.0-9B-Q4_K_M.gguf";
            hash = "";
          };

          ornith-mtp = pkgs.fetchurl {
            url = "https://huggingface.co/protoLabsAI/Ornith-1.0-9B-MTP-GGUF/resolve/main/Ornith-1.0-9B-MTP-Q4_K_M.gguf";
            hash = "";
          };

          # Bundle: convenience package that starts llama-server with MTP
          serve = pkgs.writeShellScriptBin "serve" ''
            ${self.packages.${system}.llamacpp}/bin/llama-server \
              --model ${self.packages.${system}.ornith-9b} \
              --spec-type draft-mtp \
              --spec-draft-model ${self.packages.${system}.ornith-mtp} \
              --spec-draft-n-max 3 \
              --host 0.0.0.0 --port 8000 \
              --ctx-size 128000 --n-gpu-layers 99 \
              --parallel 1 --cont-batching \
              --tool-call-parser qwen3_xml \
              --reasoning-parser qwen3 \
              --no-mmap \
              "$@"
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            python312
            cudaPackages.cudatoolkit
            cudaPackages.cudnn
            self.packages.${system}.llamacpp
            huggingface-cli
          ];
          shellHook = ''
            export CUDA_HOME=${pkgs.cudaPackages.cudatoolkit}
            export HF_HOME=$HOME/.cache/huggingface
            export TORCH_CUDA_ARCH_LIST="12.0"
          '';
        };
      });
}
```

---

## 15. Project Boundaries (MVP Scope)

**In scope for v0.1:**
- Nix flake building llama.cpp with CUDA 12.0 + MTP support
- Ornith 9B Q4_K_M GGUF + MTP head weight fetching
- Working `llama-server` startup command with MTP speculative decoding
- Working `ornith-mtp chat` CLI
- Python SDK with OpenAI-compatible client + MTP metrics
- Scaffold generation and adaptive revision loop
- Checkpoint engine with fail-closed validators (compile + pytest + regex)
- Tool Runtime with sandbox + permissions + timeout + pipe/redirect block
- Working memory (in-process) + episodic memory (JSONL)
- Reasoning token accounting (separate tokens_reasoning from tokens_out)
- Benchmark harness (MTP on/off comparison)
- Example scripts
- Unit tests (offline), including fail-closed validator test
- Integration tests (GPU required, manual run)

**Out of scope for v0.1:**
- Persistent memory (SQLite + vector embeddings)
- Docker/Firecracker sandbox
- Multi-user or server mode
- REST API (CLI-only for MVP)
- Web UI
- Cloud provider backends
- Ornith 35B support
- Fine-tuning or RL training
- DSpark integration (future)
- vLLM integration (future, non-GGUF)
- Pipe/redirect support in shell tool (tracked for v0.2)

---

## 16. Competitive Landscape

### 16.1 Why build a new Agent Runtime?

Four mature open-source coding agents already work with local LLMs. Each uses a **fixed scaffold** — a predefined loop that does not adapt per-step based on execution feedback:

| Tool | Primary Contribution | Scaffold Type | Adaptive? |
|------|-------------------|--------------|-----------|
| **Aider** | Git-native CLI, surgical find-and-replace edits | Fixed: edit → commit loop | No |
| **Cline** | VS Code agent with plan/act modes, approval gates | Fixed: plan → edit → test → commit per approval | No |
| **OpenHands** | Full autonomous engineer (used by Ornith for SWE-Bench eval) | Fixed: browse → code → test → PR loop | No |
| **Continue.dev** | IDE autocomplete + chat panel | Fixed: FIM completion + context retrieval | No |

All four support any OpenAI-compatible local endpoint. All four expose tool calling. All four checkpoint via git. **None revise their plan mid-execution based on validator feedback.**

### 16.2 The adaptive scaffold differentiator

The Agent Runtime's core innovation is the **Generate → Execute → Observe → Checkpoint → Revise** loop (see §3.1). This differs from existing tools in three ways:

1. **Per-step scaffold revision:** If a compile validator fails, the scaffold is revised — not just the code. The plan itself changes (split sub-steps, alter tool selection, change strategy).
2. **Validators as first-class gates:** Checkpoints aren't just "did tests pass." They are typed validators (compile, regex, schema, benchmark, LLM judge) with configurable confidence and fail-closed semantics.
3. **Unified metric feedback:** Token acceptance rate, reasoning overhead, and tool latency feed back into scaffold decisions. No existing tool tracks MTP acceptance rate per-step.

### 16.3 When existing tools are the better choice

- **You want a mature IDE integration:** Use Cline (VS Code) or Continue (VS Code/JetBrains). This stack is CLI-only for v0.1.
- **You want git-native workflow:** Use Aider. Every change is a commit. This stack does not wrap git (the Tool Runtime calls `git` as a shell command).
- **You want production deployment:** Use OpenHands (Docker-based, Kubernetes support). This stack is single-user local for v0.1.
- **You don't need acceleration:** Any tool + llama.cpp without MTP works fine.

### 16.4 Coexistence strategy

This stack does not compete with these tools — it can serve as a **backend** for them. The SDK's OpenAI-compatible client (§7) means any of these tools can use Ornith + MTP for inference. The Agent Runtime is an additional orchestration layer for users who want adaptive scaffolding on top of the accelerated backend.

### 16.5 Risk: competitive irrelevance

If the adaptive scaffold fails to demonstrate measurable quality improvement over fixed scaffolds (e.g., Terminal-Bench 2.1 score with adaptive scaffold vs same model in OpenHands), the Agent Runtime adds no value. **Mitigation:** Benchmark the adaptive scaffold against the same model in OpenHands in the first integration test cycle. If no improvement, drop the adaptive scaffold and document the stack as "Ornith + MTP inference backend for existing tools."

---

## 17. Repository Cheat Sheet

```bash
# Setup
git clone https://github.com/<user>/ornith-mtp-stack
cd ornith-mtp-stack
nix develop                          # Enter dev shell with CUDA + llama.cpp

# Start server
nix run .#serve                      # llama-server with MTP speculative decoding

# Run a task
ornith-mtp run "Implement a BST in Python"

# Benchmark
ornith-mtp benchmark --mtp on --mtp off --output results.json

# Run tests
pytest tests/unit/
pytest tests/integration/            # requires GPU

# View metrics
curl http://localhost:8000/metrics
```

---

## Appendix A: DSpark Demotion Rationale

DSpark was the acceleration target in v1/v2. It is demoted in v3 for structural reasons:

1. **Model architecture mismatch:** Ornith-1.0-9B is Qwen3.5 (hybrid 24 linear-attention + 8 full-attention layers). DSpark's release drafters target Qwen3 (`deepseek-ai/dspark_qwen3_4b_block7`, etc.) and Gemma-4-12B-it. EAGLE-family drafters tap into the target's hidden states and don't transfer across architecture families.

2. **Refinetuning requirement:** Even if a Qwen3.5 base drafter existed, DSpark docs explicitly call out needing refinetuning for reasoning-model checkpoints (which Ornith is). This is non-trivial.

3. **Training hardware requirement:** DeepSpec's default config for a Qwen3-4B drafter needs 8-GPU node and ~38TB storage for the target cache. Our hardware (9800X3D + 5070 Ti) is in a different universe.

4. **vLLM integration maturity:** DSpark's vLLM integration was still landing via open PRs as of late June 2026. The flag is `--speculative-config '{"method":"dspark","num_speculative_tokens":7}'`, not `--speculative-model dspark` as documented in v2.

**Tracked for re-evaluation when:** Someone ships a Qwen3.5 DSpark drafter, or Ornith moves to plain Qwen3 architecture.

## Appendix B: Hardware-Config Matrix

| Component | Value | Notes |
|-----------|-------|-------|
| CPU | AMD 9800X3D | 8 cores / 16 threads |
| GPU | NVIDIA RTX 5070 Ti | Blackwell (sm_120), 16GB VRAM |
| RAM | 64 GB | System memory |
| OS | WSL2 (Ubuntu 24.04) | Under Windows 11 |
| CUDA | 12.8+ | Required for Blackwell support |
| TORCH_CUDA_ARCH_LIST | 12.0 | Blackwell only (not 8.9 Ada) |

**Inference config mapped to hardware:**

| Setting | Value | Rationale |
|---------|-------|-----------|
| Quant | Q4_K_M | Best quality/size tradeoff on 16GB. ~6GB for model. |
| Context | 128000 | Non-2048-multiple to avoid llama.cpp bug #23658 (MTP acceptance collapse at 2048-aligned boundaries). Can go to 262144 but risks OOM on long sessions and hits the bug. |
| MTP head | Q4_K_M | ~0.5-0.8 GB overhead. 4.2 GB margin at worst. |
| NVFP4 MTP | Available now | ~52% faster on Blackwell tensor cores (measured: 201→306 tok/s). Separate file (6.6GB). Requires llama.cpp with NVFP4 support. |
| Batch size | 1 | Single-user local inference. |
| GPU layers | 99 (all) | Full GPU offload via llama.cpp. |

## Appendix C: Sampling Parameter Rationale

| Parameter | Default | Reasoning model default | Rationale for change |
|-----------|---------|------------------------|---------------------|
| temperature | 0.0 (v2) | 0.6 | Ornith is a reasoning model. Greedy decoding (temp=0.0) will underperform its benchmarked behavior. Recommended: temp=0.6. |
| top_p | 1.0 (implied) | 0.95 | Narrowing nucleus sampling improves quality for reasoning tasks. |
| top_k | 0 (disabled, implied) | 20 | Limits vocabulary to top 20 tokens. Matches Ornith's published recommendations. |

Note: `temp=1.0` reproduces Ornith's published benchmarks. `temp=0.6` is the pragmatic default for coding tasks where reproducibility matters more than peak diversity.
