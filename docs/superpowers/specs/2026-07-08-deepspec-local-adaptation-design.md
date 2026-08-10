# DeepSpec DSpark Training on Local Hardware (9800X3D + RTX 5070 Ti)

**Date**: 2026-07-08
**Status**: Design approved, pre-implementation
**Target**: Run the full DSpark training pipeline for Ornith-1.0-9B from `dspark-ornith-plan.md` on a single RTX 5070 Ti (16 GB VRAM)

---

## Hardware Constraints

| Component | Spec | Bottleneck |
|-----------|------|------------|
| GPU | RTX 5070 Ti, 16 GB VRAM, Blackwell sm_120 | Cannot load 9B bf16 model (18 GB) |
| CPU | Ryzen 7 9800X3D, 8C/16T | Adequate for data pipeline |
| RAM | 31 GB | Tight but workable with CPU offloading |
| Disk | 336 GB free | 75,000× short of 25 TB target cache |
| CUDA | 13.3.1 active (torch 2.9.1 ships CUDA 12.8 — compatible) | No conflict with torch |

## Tech Stack Decision

| Component | Engine | Why |
|-----------|--------|-----|
| Phase 3b (answer gen) | **llama.cpp** (existing Ornith Q4_K_M GGUF, 5.24 GiB) | Already downloaded, fits VRAM, avoids 18.8 GB safetensors download |
| Phase 3c (target cache) | **Skipped** — replaced by on-the-fly computation | 25+ TB cache is impossible on local disk |
| Phase 4 (training) | **HuggingFace + bitsandbytes 4-bit** | Target model fits in ~5 GB VRAM at 4-bit |
| Phase 5 (inference) | **vLLM** (remote) | Only engine with native DSpark support; can't run locally but required for deployment |

---

## Architecture: On-the-Fly Hidden State Computation

The core innovation: replace the 25+ TB pre-computed disk cache with live GPU computation at training time.

### Before (plan's default)

```
JSONL → Phase 3c (8 GPUs, 25+ TB cache) → disk → Phase 4 (read cache, train draft)
```

### After (local adaptation)

```
JSONL → Phase 4 (load JSONL, compute hidden states on-the-fly via 4-bit target, train draft)
                                          ↑
                                4-bit target model (~5 GB VRAM)
                                co-resident with draft model (~2 GB VRAM)
```

### Memory Budget

| Component | VRAM | Notes |
|-----------|------|-------|
| Target model (4-bit) | ~5 GB | `BitsAndBytesConfig(load_in_4bit=True)` |
| Draft model (bf16) | ~2 GB | 5 layers × 4096 hidden, ~1B params |
| Activations + optimizer states | ~4 GB | Single micro-batch, bf16 + Adam |
| CUDA overhead + buffers | ~2 GB | |
| **Total** | **~13 GB** | Fits in 16 GB with 3 GB headroom |

### Throughput Trade-off

On-the-fly computation adds ~1 target forward pass per training step (same batch size). The target (9B, 4-bit) is ~2× the draft (1B, bf16). Each training step does ~3× the compute of cache-based training. On a single 5070 Ti (vs 8× A100), this adds hours but not days for a micro test.

---

## Components

### C1. `OnlineTargetTrainingMixin` (new file: `deepspec/trainer/online_target_mixin.py`)

A Python mixin for `BaseTrainer` that replaces cache-based data loading with online computation.

**Overrides:**
- `__init__()` (replaces `BaseTrainer.__init__`) — calls individual init methods selectively:
  1. Calls `init_dist()`, `build_models()`, optimizer setup (same as Base)
  2. Loads target model in 4-bit via `AutoModel.from_pretrained(quantization_config=BitsAndBytesConfig(load_in_4bit=True))`
  3. Creates `JsonLineDataset` instead of `CacheDataset`
  4. Skips `validate_train_cache()` (no disk cache)
- `_build_train_dataloader()` — uses `JsonLineDataset` instead of `CacheDataset`, `OnlineCacheCollator` instead of `CacheCollator`, `num_workers=0`

**Usage:**
```python
class OnlineQwen3_5DSparkTrainer(OnlineTargetTrainingMixin, Qwen3_5DSparkTrainer):
    pass  # All overrides come from the mixin
```

**No changes needed to `run_batch()`** — the batch dict has the same shape.

### C2. `OnlineCacheCollator` (new file: `deepspec/data/online_cache_collator.py`)

Runs the target model forward pass at collation time.

```python
class OnlineCacheCollator:
    def __init__(self, target_model, tokenizer, target_layer_ids, chat_template, max_length, min_loss_tokens):
        self.target_model = target_model
        self.conversation_collator = ConversationCollator(tokenizer, chat_template, max_length, min_loss_tokens)
        self.target_layer_ids = target_layer_ids

    def __call__(self, batch):
        # 1. Tokenize via ConversationCollator
        tokenized = self.conversation_collator(batch)

        # 2. Run target forward with hooks (reuse run_target_forward_with_hooks)
        target_result = run_target_forward_with_hooks(
            target_model=self.target_model,
            input_ids=tokenized["input_ids"],
            attention_mask=tokenized["attention_mask"],
            target_layer_ids=self.target_layer_ids,
        )

        # 3. Return full batch dict (same shape CacheCollator returns)
        return {
            "input_ids": tokenized["input_ids"],
            "attention_mask": tokenized["attention_mask"],
            "loss_mask": tokenized["loss_mask"],
            "target_hidden_states": target_result.target_hidden_states,
            "target_last_hidden_states": target_result.target_last_hidden_states,
        }
```

### C3. `generate_with_llamacpp.py` (new file: `scripts/data/generate_with_llamacpp.py`)

Phase 3b answer generation using the Ornith Q4_K_M GGUF via llama.cpp's OpenAI-compatible API.

```python
# Pseudo:
client = OpenAI(base_url="http://localhost:8080/v1", api_key="not-needed")
for item in dataset:
    response = client.chat.completions.create(
        model="ornith",
        messages=item["messages"],
        max_tokens=4096,
        temperature=0.6,
    )
    output.append({**item, "choices": [{"message": response.choices[0].message}]})
```

### C4. `dspark_ornith_9b_local.py` (new config: `config/dspark/dspark_ornith_9b_local.py`)

Local hardware training config:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `target_layer_ids` | `[3, 11, 19, 27]` | 4 layers (plan's recommended trade-off) |
| `num_draft_layers` | `5` | Standard |
| `block_size` | `7` | Standard |
| `max_length` | `2048` | Halves per-sample compute vs 4096 |
| `local_batch_size` | `1` | VRAM constraint |
| `global_batch_size` | `32` | Gradient accumulation over 32 micro-batches |
| `num_workers` | `0` | GPU collator needs main process |
| `sharding_strategy` | `no_shard` | Single GPU |
| `online_target.enabled` | `true` | Enables online mode (read by mixin `__init__` to choose `JsonLineDataset` vs `CacheDataset`) |
| `online_target.train_data_path` | `./data/train.jsonl` | Path to training JSONL (required when `online_target.enabled=true`) |
| `target_cache_path` | `None` | No disk cache (ignored when `online_target.enabled=true`) |

---

## Implementation Plan

### Phase A: Environment Setup

1. Install `torch==2.9.1` (CUDA 12.8 wheel — self-contained, doesn't conflict with system CUDA 13)
2. Install `transformers==5.10.2` (required for qwen3_5 module)
3. Install `bitsandbytes` (4-bit quantization)
4. Install remaining requirements from DeepSpec's `requirements.txt`
5. Verify `from transformers.models.qwen3_5.modeling_qwen3_5 import Qwen3_5MLP` works

### Phase B: Online Training Pipeline

1. Create `deepspec/data/online_cache_collator.py`
2. Create `deepspec/trainer/online_target_mixin.py`
3. Add `use_online_target` config option
4. Wire the mixin into the actual trainer (`OnlineQwen3_5DSparkTrainer`)
5. Create `config/dspark/dspark_ornith_9b_local.py`

### Phase C: Answer Generation Script

1. Create `scripts/data/generate_with_llamacpp.py`
2. Start llama.cpp server with Ornith GGUF
3. Test with a few prompts

### Phase D: Verify

1. Import all modules successfully
2. Run a single training step and verify shapes
3. Generate answers for 10 prompts via llama.cpp

---

## Key Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `bitsandbytes` doesn't support Windows + sm_120 | **High** | Fallback to 8-bit or CPU offloading (slower but works) |
| `OnlineTargetTrainingMixin.__init__` conflicts with `BaseTrainer.__init__` | Medium | Can fully override `__init__` and call individual methods |
| Training loop too slow for practical use | Medium | Acceptable for testing; production training on cloud |
| target model `model_type` mismatch in hooks | Low | Already confirmed: qwen3_5 VL model uses `model.model` backbone |
| `torch.compile` not compatible with dynamic shapes | Low | Disable `torch_compile` in local config |
