# Ornith + DSpark Stack: Engineering Specification v2

**Date:** 2026-07-08
**Status:** Draft v2
**Repository:** `ornith-dspark-stack` (standalone, MIT)
**Status labels:** `[verified]` = confirmed behavior, `[assumed]` = reasonable but unverified, `[goal]` = target, `[design]` = architecture decision

---

## 1. Vision

Wire DeepReinforce's Ornith 1.0 (self-scaffolding coding agent) with DeepSeek's DSpark (lossless speculative decoding) into a single local inference stack where the orchestration layer — not the models — is the differentiating contribution.

**What this stack enables that neither project alone provides:**
- Per-task adaptive scaffold that revises based on execution feedback
- Unified memory (working + episodic + persistent)
- Structured validator-based checkpoint engine
- Tool Runtime with sandbox, permissions, cancellation, rollback
- End-to-end benchmark harness comparing accelerated vs non-accelerated paths

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
│  OpenAI-compatible HTTP → vLLM server                           │
│  Streaming · Structured output · Tool-calling format            │
│  Swappable: local vLLM, cloud API, or Ollama backend           │
├──────────────────────────────────────────────────────────────────┤
│  ACCELERATION LAYER                                              │
│                                                                  │
│  DSpark speculative decoding [design: requires vLLM ≥ 0.22.0]  │
│  KV cache (FP8) [design]                                        │
│  Flash attention [via vLLM]                                      │
├──────────────────────────────────────────────────────────────────┤
│  INFRASTRUCTURE LAYER                                            │
│                                                                  │
│  CUDA 12.8+ · WSL2 Ubuntu · Nix flake · Prometheus metrics     │
│  Structured logging (JSON) · Config (single source of truth)    │
└──────────────────────────────────────────────────────────────────┘
```

**Design invariant:** No layer calls across more than one boundary. The Agent Runtime never talks to CUDA. The Application Layer never talks to vLLM.

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
    tokens_out: int
    acceptance_rate: float | None          # from DSpark metrics
```

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
- Stored in `~/.ornith-dspark/episodic/{session_id}.jsonl`

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
5. If validator itself errors → log warning, treat as pass (fail-open for MVP)

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
    provider: str                          # "vllm" | "openai" | "ollama"
    base_url: str
    model: str
    api_key: str | None
    max_tokens: int
    temperature: float
    streaming: bool                        # [design: MVP non-streaming]
    speculative: SpeculativeConfig | None

@dataclass
class SpeculativeConfig:
    enabled: bool
    draft_model: str | None                # None = use DSpark default
    num_speculative_tokens: int            # default 5
    acceptance_threshold: float            # default 0.9
```

### 4.2 DSpark Compatibility Requirements

`[assumed]` DSpark is available in vLLM ≥ 0.22.0 as `--speculative-model dspark`. This needs verification against the exact vLLM release.

**Compatibility matrix `[goal]`:**

| Feature | Supported | Notes |
|---------|-----------|-------|
| GPTQ/AWQ quantization | Unknown | Must test |
| FP8 KV cache | `[assumed]` | vLLM defaults |
| Streaming | `[assumed]` | Spec decode with streaming may have different behavior |
| Prefix caching | Unknown | May invalidate draft predictions |
| Chunked prefill | Unknown | Must test |
| LoRA adapters | `[assumed]` not compatible | Spec decode + LoRA rarely tested |
| Tensor parallelism | `[assumed]` no (single GPU) | N/A |
| Tool calling format | `[assumed]` yes | Ornith uses Qwen XML, passes through |
| Beam search | `[assumed]` no | Spec decode requires greedy/temperature sampling |

### 4.3 Performance Bounds

`[design]` DSpark speculative decoding improves throughput under specific conditions:

| Condition | Improvement | Confidence |
|-----------|-------------|------------|
| Long generation (256+ tokens) | 30-60% | `[assumed]` based on DSpark paper |
| Short generation (< 50 tokens) | 0-15% | `[assumed]` overhead dominates |
| High acceptance rate (> 80%) | 40-85% | `[goal]` requires well-matched draft |
| Low acceptance rate (< 50%) | 0-20% | `[assumed]` frequent rejections |
| Batch size 1 (single user) | 30-60% | `[goal]` |
| Batch size 4+ | 10-30% | `[assumed]` throughput shift |

**The "60-85%" headline is a best-case benchmark result.** Real-world improvement depends on prompt structure, generation length, draft quality, and hardware. The benchmark harness is designed to measure actual improvement per workload.

---

## 5. Acceleration Layer

### 5.1 DSpark Integration

`[design]` DSpark runs as vLLM's built-in speculative decoding backend:

```bash
vllm serve deepreinforce-ai/Ornith-1.0-9B \
  --host 0.0.0.0 --port 8000 \
  --dtype auto \
  --max-model-len 262144 \
  --gpu-memory-utilization 0.85 \
  --speculative-model dspark \
  --num-speculative-tokens 5 \
  --kv-cache-dtype fp8
```

`[assumed]` The DSpark drafter for Qwen-based models is auto-selected by vLLM when `--speculative-model dspark` is set with a Qwen target. This is based on DeepSpec's `Qwen3DSparkModel` architecture.

### 5.2 VRAM Budget

| Component | Idle | Worst-case | Notes |
|-----------|------|------------|-------|
| Ornith 9B (Q4) | 5.5 GB | 6.5 GB | GGUF Q4_K_M, ~6 GB base + overhead |
| DSpark drafter | 1.0 GB | 2.0 GB | Small draft model |
| KV cache (FP8) | 1.0 GB | 4.0 GB | Depends on context length + batching |
| CUDA allocator overhead | 0.5 GB | 1.5 GB | Fragmentation, graph capture |
| vLLM framework | 0.3 GB | 0.5 GB | Scheduler, tokenizer, etc. |
| **Total** | **8.3 GB** | **14.5 GB** | 16 GB available → 1.5 GB margin at worst |

**Mitigations for worst-case:**
- Reduce `--gpu-memory-utilization` to 0.75 (1 GB savings)
- Reduce `--num-speculative-tokens` to 3 (0.5 GB savings)
- Reduce `--max-model-len` to 131072 (1-2 GB savings)
- Disable DSpark for memory-intensive tasks (adds 1-2 GB)

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
  provider: vllm
  base_url: http://localhost:8000/v1
  model: deepreinforce-ai/Ornith-1.0-9B
  max_tokens: 4096
  temperature: 0.0

  speculative:
    enabled: true
    draft_model: dspark
    num_speculative_tokens: 5

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
  episodic_dir: ~/.ornith-dspark/episodic
  persistent_enabled: false
```

No `.env`, no CLI-only flags, no scattered config. Everything has a default in `default.yaml`.

---

## 7. Repository Layout

```
ornith-dspark-stack/
├── flake.nix                    # Nix entry point
├── config/
│   ├── default.yaml
│   ├── schema.yaml
│   └── local.yaml.example
├── sdk/                         # Python SDK (pip-installable)
│   ├── pyproject.toml
│   └── src/ornith_dspark/
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
│   └── REPORT_TEMPLATE.md
├── deployment/
│   ├── nix/                     # Nix packages
│   │   ├── vllm.nix
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
| `test_executor.py` | Step dispatch, timeout enforcement, cancellation, step ordering |
| `test_memory.py` | Working memory CRUD, episodic append/read, size limits |
| `test_checkpoint.py` | All validators (compile, pytest, regex, schema, llm_judge mock) |
| `test_tool_runtime.py` | Permission checks, sandbox launch, timeout, deny patterns |
| `test_config.py` | YAML parse, override order, schema validation |

### 8.2 Integration Tests (GPU required)

| Test | What it verifies |
|------|-----------------|
| `test_full_loop.py` | End-to-end scaffold → execute → checkpoint → revise cycle |
| `test_tool_sandbox.py` | Restricted shell executes only allowed commands, rejects forbidden |
| `test_checkpoint_validators.py` | Real compile validator catches errors, real pytest passes |
| `test_recovery.py` | Process kill + restart, scaffold state persistence across crash |

### 8.3 Stress Tests (GPU required)

| Test | What it verifies |
|------|-----------------|
| `test_long_context.py` | 100K+ token context, memory stability, no OOM |
| `test_concurrent.py` | 4 simultaneous requests, no deadlock, GPU memory stable |
| `test_oom_recovery.py` | Force OOM, verify graceful recovery and error reporting |
| `test_streaming.py` | SSE stream correctness, no truncation, DSpark on/off parity |

### 8.4 Speculative Decoding Correctness

Critical: DSpark claims lossless output. We verify this:
- Run same prompt 10× with DSpark on and 10× with DSpark off
- Compare output tokens (must be identical)
- If any divergence → log error, disable DSpark for that workload
- Track divergence rate over time

---

## 9. Benchmarking

### 9.1 Dimensions

| Metric | Collection | Baseline |
|--------|-----------|----------|
| Latency (p50/p95/p99) | Per-request timer | DSpark off |
| Throughput (tok/s) | Token count / wall time | DSpark off |
| Acceptance rate | DSpark metric from vLLM | N/A |
| GPU utilization | `nvidia-smi` polling | DSpark off |
| VRAM peak | `nvidia-smi` max | DSpark off |
| VRAM fragmentation | Pre/post allocation diff | DSpark off |
| First-token latency | Timer to first token | DSpark off |
| Time-to-first-tool | Step start to first tool call | DSpark off |
| Long-context degradation | Tok/s vs context length curve | DSpark off |
| Batch scaling | Tok/s vs concurrent requests | DSpark off |
| Prompt scaling | Tok/s vs prompt length | DSpark off |

### 9.2 Harness

`python -m benchmarks.run --dspark on --prompts prompts.json --output results/`

Output: JSON file with all metrics + optional Markdown report.

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

Denied: `rm`, `mv`, `chmod`, `chown`, `sudo`, `su`, `curl`, `wget`, `ssh`, `telnet`, `nc`, `mkfs`, `mount`, `dd`, `:(){ :|:& };:` (fork bomb), any command with `|`, `>`, `<`, `;`, `` ` ``

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
| **Transient** | vLLM connection refused | Wait 5s, retry 3× |
| | CUDA context lost | Restart vLLM, retry |
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
| DSpark not compatible with Ornith (Qwen-based) | Medium | High | Test immediately after setup. Fallback: use Eagle or draft model spec decode |
| GPU OOM under long context | Medium | Medium | Dynamic KV cache sizing, context length reduction |
| Scaffold quality poor (LLM generates bad strategy) | Medium | Medium | Default fallback scaffold, revision loop catches failures |
| 16 GB VRAM insufficient for both models | Low | High | Can disable DSpark per-task, fall back to vanilla Ornith |
| WSL2 CUDA passthrough unreliable | Low | Medium | Docker fallback, or direct Linux install |
| HuggingFace rate-limits weight download | Low | Low | Pre-download, Nix cache |
| Speculative decoding < 10% improvement | Medium | Low | Always measured; fallback path always available |
| Model license changes after download | Low | Medium | Pinned weights in Nix store, MIT license irrevocable |

---

## 14. Nix Flake

Reference: nixtorch (github:hinriksnaer/nixtorch) for CUDA/PyTorch build tooling. Graham33's nixos-dgx-spark for Blackwell GPU support.

```nix
{
  description = "Ornith + DSpark local inference stack";

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
          vllm-dspark = pkgs.vllm.overrideAttrs (old: {
            buildInputs = old.buildInputs ++ [ pkgs.cudaPackages.cudnn ];
            env = (old.env or {}) // {
              TORCH_CUDA_ARCH_LIST = "8.9;12.0";  # Ada + Blackwell
            };
          });

          ornith-9b = pkgs.fetchurl {
            url = "https://huggingface.co/deepreinforce-ai/Ornith-1.0-9B-GGUF/resolve/main/Ornith-1.0-9B-Q4_K_M.gguf";
            hash = "";  # populated after first build
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            python312
            cudaPackages.cudatoolkit
            cudaPackages.cudnn
            self.packages.${system}.vllm-dspark
            huggingface-cli
          ];
          shellHook = ''
            export CUDA_HOME=${pkgs.cudaPackages.cudatoolkit}
            export HF_HOME=$HOME/.cache/huggingface
          '';
        };
      });
}
```

---

## 15. Project Boundaries (MVP Scope)

**In scope for v0.1:**
- Nix flake building VLLM with DSpark support
- Ornith 9B weight fetching (Q4 GGUF)
- Working `vllm serve` startup command with DSpark
- Working `ornith-dspark chat` CLI
- Python SDK with OpenAI-compatible client + DSpark metrics
- Scaffold generation and adaptive revision loop
- Checkpoint engine with compile + pytest + regex validators
- Tool Runtime with sandbox + permissions + timeout
- Working memory (in-process) + episodic memory (JSONL)
- Benchmark harness (DSpark on/off comparison)
- Example scripts
- Unit tests (offline)
- Integration tests (GPU required, manual run)

**Out of scope for v0.1:**
- Persistent memory (SQLite + vector embeddings)
- Docker/Firecracker sandbox
- Multi-user or server mode
- REST API (CLI-only for MVP)
- Web UI
- Cloud provider backends
- Ornith 35B or 397B support
- Fine-tuning or RL training

---

## 16. Repository Cheat Sheet

```bash
# Setup
git clone https://github.com/<user>/ornith-dspark-stack
cd ornith-dspark-stack
nix develop                          # Enter dev shell with CUDA + VLLM

# Start server
nix run .#serve                      # vllm serve with DSpark

# Run a task
ornith-dspark run "Implement a BST in Python"

# Benchmark
ornith-dspark benchmark --dspark on --dspark off --output results.json

# Run tests
pytest tests/unit/
pytest tests/integration/            # requires GPU

# View metrics
curl http://localhost:8000/metrics
```
