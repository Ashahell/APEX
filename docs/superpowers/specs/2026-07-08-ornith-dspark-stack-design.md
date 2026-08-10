# Ornith + DSpark Stack: Self-Scaffolding Agent on Accelerated Inference

**Date:** 2026-07-08
**Status:** Draft
**Repository:** `ornith-dspark-stack` (standalone, separate from APEX)
**License:** MIT (matching both Ornith and DSpark)

## Overview

Wire DeepReinforce's Ornith 1.0 (self-scaffolding coding agent) on top of DeepSeek's DSpark (lossless speculative decoding) into a single local inference stack. The model learns its own per-task execution strategy; DSpark makes it 60-85% faster. Both are MIT-licensed, open weights, and designed for Qwen 3.5 (Ornith's base).

## Hardware Target

- **CPU:** AMD 9800X3D
- **GPU:** NVIDIA RTX 5070 Ti (16 GB VRAM, Blackwell architecture, sm_120)
- **OS:** Windows 11 → WSL2 Ubuntu (NVIDIA CUDA via WSL2)
- **VRAM budget:** Ornith 9B Q4 (~6 GB) + DSpark drafter (~1-2 GB) + KV cache (~2-4 GB) = ~10-12 GB (comfortable on 16 GB)

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│  Self-Scaffolding Client (Python)                        │
│                                                          │
│  1. generate_scaffold(task) → JSON plan                  │
│  2. execute_with_scaffold(task, scaffold)                 │
│     ├─ step 1: read spec                                 │
│     ├─ step 2: implement                                 │
│     ├─ step 3: verify checkpoint                         │
│     └─ step 4: retry on failure                          │
│                                                          │
│  ┌──────────────────────────────────────────────────────┐│
│  │  OpenAI-compatible HTTP ↓                            ││
│  └──────────────────────────────────────────────────────┘│
└──────────────────────────┬───────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────┐
│  VLLM Inference Server (port 8000)                       │
│                                                          │
│  ┌─────────────┐  ┌─────────────────────┐               │
│  │ Ornith 9B   │  │ DSpark Drafter      │               │
│  │ (target)    │◄─┤ (speculative model) │               │
│  │ ~6 GB Q4    │  │ ~1-2 GB             │               │
│  └─────────────┘  └─────────────────────┘               │
│                                                          │
│  ┌──────────────────────────────────────────────────────┐│
│  │  Metrics: acceptance rate, tok/s, GPU memory         ││
│  └──────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────┘
```

## Repository Structure

```
ornith-dspark-stack/
├── flake.nix              # Nix flake: inputs, outputs, packages, devShells
├── flake.lock             # Pinned versions
├── .env.example           # ORNITH_MODEL, DSPARK_MODEL, PORT, GPU settings
├── nix/
│   ├── vllm.nix           # VLLM build with DSpark (CUDA sm_89/sm_120)
│   └── ornith.nix         # Ornith weight fetcher (fixed-output derivation)
├── config/
│   ├── vllm.yaml          # VLLM server config template
│   └── dspark.yaml        # DSpark drafter config
├── scripts/
│   ├── start.sh           # Launch VLLM with DSpark + Ornith
│   ├── stop.sh            # Graceful shutdown
│   ├── health.sh          # Health check endpoint
│   └── benchmark.sh       # Throughput comparison (with/without DSpark)
├── client/
│   ├── pyproject.toml     # Python package metadata
│   ├── requirements.txt   # openai, httpx
│   └── src/ornith_dspark/
│       ├── __init__.py
│       ├── client.py      # OpenAI-compatible chat wrapper
│       ├── scaffold.py    # Self-scaffolding loop
│       └── metrics.py     # Acceptance rate, tokens/sec tracking
└── examples/
    ├── 01-basic-chat.py
    ├── 02-tool-calling.py
    ├── 03-self-scaffolding.py
    └── 04-benchmark.py
```

## Nix Flake Design

### Inputs
- `nixpkgs` (unstable channel, CUDA-enabled)
- `flake-utils` for multi-system support
- `nixtorch` (github:hinriksnaer/nixtorch) for CUDA/PyTorch/VLLM tooling
- Ornith weights via fixed-output derivation from HuggingFace

### Outputs
- **`packages.x86_64-linux.vllm-dspark`**: VLLM built with CUDA 12.8+, `TORCH_CUDA_ARCH_LIST` set for sm_89 (Ada) and sm_120 (Blackwell). DSpark is natively included (merged into vLLM mainline as of v0.22.0, June 2026). Requires vLLM ≥ 0.22.0.
- **`packages.x86_64-linux.ornith-9b`**: Fetches `deepreinforce-ai/Ornith-1.0-9B-GGUF` Q4_K_M from HuggingFace. Note: requires `HF_TOKEN` env var if HuggingFace gates access behind login (weights are MIT but HuggingFace may require authentication). Cached in Nix store.
- **`devShells.x86_64-linux.default`**: Python venv with VLLM, HuggingFace Hub CLI, `openai` client library, benchmarking tools.
- **`apps.x86_64-linux.serve`**: `vllm serve` with all DSpark flags.
- **`apps.x86_64-linux.benchmark`**: Runs throughput comparison (DSpark on/off).

### Key Nix Config
```nix
{
  nixpkgs.config.cudaSupport = true;
  nixpkgs.config.allowUnfreePredicate = p: builtins.elem p.meta.license.shortName [
    "CUDA EULA" "cuDNN EULA"
  ];
}
```

## VLLM / DSpark Server

### Startup Command
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

### Key Arguments
| Flag | Value | Rationale |
|------|-------|-----------|
| `--speculative-model` | `dspark` | Activates DSpark speculative decoding (native in vLLM) |
| `--num-speculative-tokens` | 5 | Default draft length; lower to 3 if VRAM constrained |
| `--gpu-memory-utilization` | 0.85 | Leaves ~2.4 GB headroom for other processes |
| `--max-model-len` | 262144 | Ornith's full context window |
| `--kv-cache-dtype` | fp8 | Reduces KV cache memory by ~50% |

### DSpark Drafter
No separate download needed. DSpark is integrated directly in vLLM's speculative decoding backend. The drafter is bundled with the vLLM build and activated by `--speculative-model dspark`. For Ornith (Qwen-based), DSpark uses the Qwen3DSparkModel architecture from DeepSpec.

### Metrics Endpoint
The VLLM server exposes `/metrics` with Prometheus-format data:
- `vllm:speculative_acceptance_rate` — percentage of draft tokens accepted
- `vllm:avg_tokens_per_sec` — generation throughput
- `vllm:gpu_memory_usage` — VRAM consumption
- `vllm:num_requests_running` — current request count

## Self-Scaffolding Client

### Scaffold Schema
The client sends a meta-prompt to Ornith before task execution, asking it to produce a JSON scaffold:

```json
{
  "plan": ["read spec", "write code", "run tests", "fix failures"],
  "tools": ["file.read", "code.generate", "shell.execute"],
  "max_steps": 8,
  "checkpoints": [
    {"step": "write code", "criteria": "compiles without errors"},
    {"step": "run tests", "criteria": "all tests pass"}
  ],
  "retry_policy": {"max_retries": 2, "fallback": "try alternative approach"},
  "success_criteria": "all checkpoints pass and task goal is met"
}
```

### Client API
```python
client = OrnithClient(base_url="http://localhost:8000/v1")

# Phase 1: Generate scaffold from task description
scaffold = client.generate_scaffold(task="Implement a binary search tree")

# Phase 2: Execute with scaffold-aware loop
result = client.execute_with_scaffold(
    task="Implement a binary search tree",
    scaffold=scaffold,
    on_step=lambda s: print(f"[{s.number}] {s.action}: {s.status}")
)
# result = { "success": bool, "steps": [...], "total_cost_usd": float,
#            "timing_ms": int, "acceptance_rate": float }
```

### generate_scaffold()
Sends a meta-prompt instructing Ornith to analyze the task and return a JSON scaffold. Uses Ornith's native Qwen XML tool-calling format. Prompt template:

```
You are an agent architect. Given a task, design an execution strategy.
Return ONLY valid JSON with this schema:
{
  "plan": [array of step descriptions],
  "tools": [array of tool names to use],
  "max_steps": int,
  "checkpoints": [array of {step, criteria}],
  "retry_policy": {"max_retries": int, "fallback": str},
  "success_criteria": str
}
Task: {task}
```

### execute_with_scaffold()
Lightweight Python loop (no external deps beyond `openai`):

1. For each step in `scaffold.plan`:
   - Call LLM with step description + allowed tools + previous history
   - Execute any tool calls returned by the LLM
   - Record step result (output, cost, timing)
2. If step matches a checkpoint:
   - Evaluate criteria (currently keyword-based; extensible to LLM-judge)
   - If checkpoint fails and retries remain, re-attempt the step
   - If checkpoint fails and no retries, apply fallback strategy
3. After all steps, evaluate `success_criteria`
4. Return structured result with full step history

### Metrics Tracking
The client reads VLLM's `/metrics` endpoint and records:
- **Acceptance rate**: `vllm:speculative_acceptance_rate` gauge
- **Tokens/sec**: `vllm:avg_tokens_per_sec`
- **Speedup**: runs same prompt with and without DSpark, compares wall time

## Examples

1. **01-basic-chat.py**: Connect to VLLM, send a prompt, stream response
2. **02-tool-calling.py**: Demonstrate Ornith's Qwen XML tool-calling format
3. **03-self-scaffolding.py**: Generate scaffold, execute 3-step coding task
4. **04-benchmark.py**: Compare throughput DSpark on vs off, print table

## Error Handling

| Failure Mode | Detection | Recovery |
|-------------|-----------|----------|
| VLLM not running | Connection refused | Retry 3x with 5s backoff, then print setup instructions |
| GPU OOM | vLLM stderr "CUDA OOM" | Reduce `--gpu-memory-utilization` or `--num-speculative-tokens` |
| Model not found | HTTP 404 from VLLM | Check model path, offer to download |
| Scaffold parse failure | JSON decode error | Retry generation with stricter prompt, fallback to default scaffold |
| Checkpoint failure | Criteria not met | Apply retry policy, then fallback strategy |

## Testing

### Offline (no GPU)
- Unit tests for scaffold parsing and validation
- Unit tests for checkpoint evaluation logic
- Mock VLLM responses for client integration tests

### Online (with GPU)
- `python examples/04-benchmark.py` — runs 5 prompts with and without DSpark
- Expected: 60-85% latency reduction with DSpark on
- Assert: identical output tokens (lossless guarantee check)

## Prior Art Referenced

- **DSpark merged into vLLM** (Simon Mo, June 2026): Native speculative decoding backend
- **nixtorch** (github:hinriksnaer/nixtorch): Nix flake for VLLM/PyTorch GPU builds with CUDA 12.6-13.2
- **nixos-dgx-spark** (github:graham33/nixos-dgx-spark): Nix flake for Blackwell GPUs, CUDA sm_120
- **DeepSpec** (github:deepseek-ai/DeepSpec): DSpark training toolkit with Qwen3/Gemma4 support
- **Ornith 1.0** (huggingface.co/deepreinforce-ai): Open weights, GGUF quants, MIT license
