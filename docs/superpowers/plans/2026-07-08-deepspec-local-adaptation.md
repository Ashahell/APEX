# DeepSpec Local Hardware Adaptation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Adapt the DeepSpec DSpark training pipeline to run on a single RTX 5070 Ti (16 GB VRAM) by replacing the 25+ TB disk cache with on-the-fly hidden state computation

**Architecture:** A `OnlineTargetTrainingMixin` overrides cache-based data loading in `BaseTrainer` to load target model in 4-bit and compute hidden states at collation time. A `OnlineCacheCollator` tokenizes raw JSONL data and runs the target forward pass with hooks. Answer generation uses llama.cpp's GGUF server instead of vLLM/SGLang.

**Tech Stack:** Python 3.13, PyTorch 2.9.1, transformers 5.10.2, bitsandbytes, llama.cpp server

**Spec:** `docs/superpowers/specs/2026-07-08-deepspec-local-adaptation-design.md`

---

## File Inventory

### New files (4)
| File | Responsibility |
|------|---------------|
| `deepspec/data/online_cache_collator.py` | Runs target model forward at collation time, returns batch with hidden states |
| `deepspec/trainer/online_target_mixin.py` | Mixin that loads 4-bit target model, replaces `CacheDataset` with `JsonLineDataset`, wires `OnlineCacheCollator` |
| `config/dspark/dspark_ornith_9b_local.py` | Training config tuned for RTX 5070 Ti (4 target layers, max_length=2048, batch=1, online mode) |
| `scripts/data/generate_with_llamacpp.py` | Answer regeneration via llama.cpp OpenAI-compatible API |

### Modified files (2)
| File | Change |
|------|--------|
| `deepspec/trainer/qwen3_5_dspark_trainer.py` | Add `OnlineQwen3_5DSparkTrainer` class that mixes in `OnlineTargetTrainingMixin` |
| `deepspec/trainer/__init__.py` | Export `OnlineQwen3_5DSparkTrainer` |

---

### Task 1: Environment Setup (torch + transformers + bitsandbytes)

**Files:** None (package installation)

- [ ] **Step 1: Create isolated Python venv for DeepSpec**

```bash
cd F:\Projects\DeepSpec
pip install virtualenv
python -m venv .venv
.venv\Scripts\Activate.ps1
```

- [ ] **Step 2: Install PyTorch 2.9.1 with CUDA 12.8**

```bash
pip install torch==2.9.1 --index-url https://download.pytorch.org/whl/cu128
```

Expected: `pip install` succeeds. Verify with:

```bash
python -c "import torch; print(f'torch {torch.__version__}, CUDA {torch.version.cuda}, device: {torch.cuda.get_device_name(0)}')"
```

Expected output: `torch 2.9.1, CUDA 12.8, device: NVIDIA GeForce RTX 5070 Ti`

- [ ] **Step 3: Install transformers 5.10.2**

```bash
pip install transformers==5.10.2
```

Verify qwen3_5 module exists:

```bash
python -c "from transformers.models.qwen3_5.modeling_qwen3_5 import Qwen3_5MLP, Qwen3_5PreTrainedModel, Qwen3_5RMSNorm, Qwen3_5TextRotaryEmbedding; print('qwen3_5 imports OK')"
```

Expected: `qwen3_5 imports OK`

- [ ] **Step 4: Install bitsandbytes + remaining dependencies**

```bash
pip install bitsandbytes
pip install numpy pyyaml tqdm tensorboard matplotlib triton sentencepiece safetensors prettytable datasets openai
```

Verify 4-bit loading works:

```bash
python -c "
from transformers import AutoModel, BitsAndBytesConfig
import torch
bnb_config = BitsAndBytesConfig(load_in_4bit=True)
try:
    model = AutoModel.from_pretrained('Qwen/Qwen3-4B', quantization_config=bnb_config, device_map='auto')
    print(f'4-bit model loaded on {model.device}, params: {sum(p.numel() for p in model.parameters())/1e9:.1f}B')
    del model
except Exception as e:
    print(f'4-bit loading failed: {e}')
"
```

Expected: `4-bit model loaded on cuda:0, params: 3.9B`

- [ ] **Step 5: Commit environment setup**

```bash
cd F:\Projects\DeepSpec
git add -f .venv/  # (optional, or use .gitignore)
git add requirements.txt
git commit -m "chore: add Python venv and requirements for local hardware"
```

---

### Task 2: Create `OnlineCacheCollator`

**Files:**
- Create: `deepspec/data/online_cache_collator.py`
- Reference: `deepspec/data/target_cache_dataset.py` (for `ConversationCollator`, `_pad_1d_batch`, `_pad_hidden_batch`)
- Reference: `scripts/data/prepare_target_cache.py` (for `run_target_forward_with_hooks`)

- [ ] **Step 1: Create `deepspec/data/online_cache_collator.py`**

```python
from typing import Dict, List, Optional

import torch

from deepspec.data.target_cache_dataset import (
    ConversationCollator,
    _pad_1d_batch,
    _pad_hidden_batch,
)
from scripts.data.prepare_target_cache import (
    TargetForwardResult,
    run_target_forward_with_hooks,
)


class OnlineCacheCollator:
    """Collator that computes target hidden states on-the-fly.

    Tokenizes raw conversation items (like ConversationCollator) then runs
    the target model forward pass with hooks to produce target_hidden_states
    and target_last_hidden_states at collation time. Returns the same batch
    dict shape as CacheCollator.
    """

    def __init__(
        self,
        target_model,
        tokenizer,
        target_layer_ids,
        chat_template: str,
        max_length: int,
        min_loss_tokens: int,
    ):
        self.target_model = target_model
        self.conversation_collator = ConversationCollator(
            tokenizer=tokenizer,
            chat_template=chat_template,
            max_length=max_length,
            min_loss_tokens=min_loss_tokens,
        )
        self.target_layer_ids = target_layer_ids

    def __call__(self, features: List[Dict]) -> Optional[Dict]:
        tokenized = self.conversation_collator(features)
        if tokenized is None:
            return None

        target_result = run_target_forward_with_hooks(
            target_model=self.target_model,
            input_ids=tokenized["input_ids"],
            attention_mask=tokenized["attention_mask"],
            target_layer_ids=self.target_layer_ids,
        )

        batch = {}
        for key in ("input_ids", "attention_mask", "loss_mask"):
            batch[key] = tokenized[key]
        batch["target_hidden_states"] = target_result.target_hidden_states
        batch["target_last_hidden_states"] = target_result.target_last_hidden_states
        return batch
```

- [ ] **Step 2: Verify syntax and imports**

```bash
cd F:\Projects\DeepSpec
python -c "from deepspec.data.online_cache_collator import OnlineCacheCollator; print('OnlineCacheCollator import OK')"
```

Expected: `OnlineCacheCollator import OK`

- [ ] **Step 3: Commit**

```bash
git add deepspec/data/online_cache_collator.py
git commit -m "feat: add OnlineCacheCollator for on-the-fly hidden state computation"
```

---

### Task 3: Create `OnlineTargetTrainingMixin`

**Files:**
- Create: `deepspec/trainer/online_target_mixin.py`
- Reference: `deepspec/data/jsonl_dataset.py` (for `JsonLineDataset`)
- Reference: `deepspec/data/cache_collator.py` → actually `data/online_cache_collator.py`
- Reference: `deepspec/utils/distributed.py` (for `init_dist`)

- [ ] **Step 1: Create `deepspec/trainer/online_target_mixin.py`**

```python
import os

import torch
from transformers import AutoModel, BitsAndBytesConfig

from deepspec.data.jsonl_dataset import JsonLineDataset
from deepspec.data.online_cache_collator import OnlineCacheCollator
from deepspec.utils import is_global_main_process


class OnlineTargetTrainingMixin:
    """Mixin for BaseTrainer that replaces disk cache with on-the-fly target computation.

    Usage:
        class OnlineQwen3_5DSparkTrainer(OnlineTargetTrainingMixin, Qwen3_5DSparkTrainer):
            pass

    Overrides:
    - __init__: loads target model in 4-bit, creates JsonLineDataset
    - _build_train_dataloader: uses OnlineCacheCollator, num_workers=0
    - build_models: skips target model download (4-bit model provides embeddings)
    """

    def __init__(self, local_rank, args):
        self.args = args
        self.device, self.global_rank, self.world_size = init_dist(local_rank)

        self.precision_dtype = {
            "bf16": torch.bfloat16,
            "fp16": torch.float16,
            "fp32": torch.float32,
        }[self.args.train.precision]

        self.checkpoint_dir_root = self.args.logging.checkpoint_dir
        self.resume_checkpoint_dir = discover_latest_checkpoint(self.checkpoint_dir_root)
        self.suspend_controller = SuspendController(device=self.device)
        self.next_micro_step = 0

        if is_global_main_process():
            ensure_dir(self.checkpoint_dir_root)
        training_logger.init(
            logging_steps=int(self.args.logging.logging_steps),
            tensorboard_dir=self.args.logging.tensorboard_dir,
        )

        # Build draft model + tokenizer (reuses BaseTrainer.build_models)
        self.draft_model, self.tokenizer = self.build_models()

        # Load target model in 4-bit for on-the-fly computation
        self._load_online_target_model()

        # Use JsonLineDataset instead of CacheDataset
        train_data_paths = (
            self.args.data.train_data_path
            if isinstance(self.args.data.train_data_path, list)
            else [self.args.data.train_data_path]
        )
        self.train_dataset = JsonLineDataset(data_paths=train_data_paths)

        # Training schedule computation (same as BaseTrainer)
        (
            self.gradient_accumulation_steps,
            self.samples_per_epoch,
            self.per_rank_samples_per_epoch,
            self.micro_batches_per_epoch,
            self.steps_per_epoch,
            self.max_train_steps,
            self.args.train.num_train_epochs,
        ) = _compute_training_schedule(
            world_size=self.world_size,
            dataset_size=len(self.train_dataset),
            local_batch_size=int(self.args.train.local_batch_size),
            global_batch_size=int(self.args.train.global_batch_size),
            num_train_epochs=int(self.args.train.num_train_epochs),
            max_train_steps=self.args.train.max_train_steps,
        )

        self.optimizer = BF16Optimizer(
            self.draft_model,
            lr=float(self.args.train.lr),
            total_steps=self.max_train_steps,
            warmup_ratio=float(self.args.train.warmup_ratio),
            weight_decay=float(self.args.train.weight_decay),
        )

        if self.resume_checkpoint_dir is not None:
            resume_state = load_training_state(
                resume_checkpoint_dir=self.resume_checkpoint_dir,
                optimizer=self.optimizer,
                global_rank=self.global_rank,
                world_size=self.world_size,
                local_batch_size=int(self.args.train.local_batch_size),
                gradient_accumulation_steps=self.gradient_accumulation_steps,
                micro_batches_per_epoch=self.micro_batches_per_epoch,
            )
            self.next_micro_step = resume_state.next_micro_step
        else:
            print_on_local_main("Training from scratch (online target mode).")

        # Wrap with FSDP (no_shard for single GPU)
        self.model = self.draft_model
        if self.args.train.torch_compile:
            self.model = torch.compile(self.model, dynamic=True)
        self.model = self._wrap_with_fsdp(self.model)

        self.info_board()

    def _load_online_target_model(self):
        """Load target model in 4-bit, keeping it on GPU for online forward passes."""
        bnb_config = BitsAndBytesConfig(load_in_4bit=True)
        self.online_target_model = AutoModel.from_pretrained(
            self.args.model.target_model_name_or_path,
            quantization_config=bnb_config,
            device_map="auto",
            torch_dtype=torch.bfloat16,
        ).eval()

    def build_models(self):
        """Build draft model. Loads a temporary full-precision target for embeddings.

        4-bit model weights are compressed and can't be used for embedding
        initialization, so we do a separate full-precision load to CPU,
        extract embeddings, then discard the temporary copy.
        """
        model_args = self.args.model

        tokenizer = AutoTokenizer.from_pretrained(
            model_args.target_model_name_or_path,
        )
        target_config = AutoConfig.from_pretrained(
            model_args.target_model_name_or_path,
        )

        draft_model = self._build_draft_model(
            target_config=target_config,
            model_args=model_args,
        )
        draft_model = draft_model.to(device=self.device, dtype=self.precision_dtype)

        # Load full-precision model temporarily on CPU for embedding extraction
        temp_target = AutoModelForCausalLM.from_pretrained(
            model_args.target_model_name_or_path,
            dtype=self.precision_dtype,
        ).to("cpu").eval()
        target_embed_tokens = temp_target.get_input_embeddings()
        target_lm_head = temp_target.get_output_embeddings()
        assert (target_lm_head is not None) and (target_embed_tokens is not None)
        draft_model.initialize_embeddings_and_head(
            embed_tokens=target_embed_tokens,
            lm_head=target_lm_head,
            freeze=True,
        )
        del temp_target
        return draft_model, tokenizer

    def _build_train_dataloader(self, start_offset_samples=0, num_samples=None):
        sampler = StatelessResumableDistributedSampler(
            dataset=self.train_dataset,
            num_replicas=self.world_size,
            rank=self.global_rank,
            total_size=self.samples_per_epoch,
            start_global_offset_samples=start_offset_samples,
            num_samples=num_samples,
        )
        return DataLoader(
            self.train_dataset,
            batch_size=int(self.args.train.local_batch_size),
            sampler=sampler,
            collate_fn=OnlineCacheCollator(
                target_model=self.online_target_model,
                tokenizer=self.tokenizer,
                target_layer_ids=self.args.model.target_layer_ids,
                chat_template=self.args.data.chat_template,
                max_length=self.args.data.max_length,
                min_loss_tokens=self.args.data.get("min_loss_tokens", 14),
            ),
            num_workers=0,
            pin_memory=False,
            drop_last=True,
        )

    def clean_up(self):
        training_logger.close()
        del self.online_target_model
        torch.cuda.empty_cache()
        dist.barrier()
        dist.destroy_process_group()
```

- [ ] **Step 2: Verify syntax**

```bash
python -c "import ast; ast.parse(open('deepspec/trainer/online_target_mixin.py').read()); print('Syntax OK')"
```

Expected: `Syntax OK`

- [ ] **Step 3: Add missing imports and verify**

The mixin uses symbols from `BaseTrainer`. Add these imports at the top of the file:

```python
import os
from contextlib import nullcontext
import math

import torch
import torch.distributed as dist
from torch.distributed.fsdp import FullyShardedDataParallel as FSDP
from torch.utils.data import DataLoader
from transformers import AutoConfig, AutoModel, AutoTokenizer, BitsAndBytesConfig

from deepspec.data.jsonl_dataset import JsonLineDataset
from deepspec.data.online_cache_collator import OnlineCacheCollator
from deepspec.trainer.ckpt_manager import (
    discover_latest_checkpoint,
    load_resume_draft_model,
    load_training_state,
    save_checkpoint,
)
from deepspec.utils import (
    BF16Optimizer,
    StatelessResumableDistributedSampler,
    ensure_dir,
    init_dist,
    is_global_main_process,
    print_on_global_main,
    print_on_local_main,
)
from deepspec.utils.hfai_suspend import SuspendController
from deepspec.trainer.base_trainer import _compute_training_schedule
import deepspec.utils.training_logger as training_logger
```

Replace the file content with this complete version.

- [ ] **Step 4: Verify import chain works (no GPU needed for import)**

```bash
cd F:\Projects\DeepSpec
python -c "from deepspec.trainer.online_target_mixin import OnlineTargetTrainingMixin; print('Import OK')"
```

Expected: `Import OK`

- [ ] **Step 5: Commit**

```bash
git add deepspec/trainer/online_target_mixin.py
git commit -m "feat: add OnlineTargetTrainingMixin for single-GPU online training"
```

---

### Task 4: Wire into Trainer + Local Config

**Files:**
- Modify: `deepspec/trainer/qwen3_5_dspark_trainer.py`
- Modify: `deepspec/trainer/__init__.py`
- Create: `config/dspark/dspark_ornith_9b_local.py`
- Create: `config/dspark/__init__.py` (if not exists)

- [ ] **Step 1: Add `OnlineQwen3_5DSparkTrainer` to `qwen3_5_dspark_trainer.py`**

Append to `F:\Projects\DeepSpec\deepspec\trainer\qwen3_5_dspark_trainer.py`:

```python
from deepspec.trainer.online_target_mixin import OnlineTargetTrainingMixin


class OnlineQwen3_5DSparkTrainer(OnlineTargetTrainingMixin, Qwen3_5DSparkTrainer):
    """Qwen3_5 DSpark trainer with on-the-fly target hidden state computation.

    For single-GPU training where pre-computing a multi-TB target cache
    is infeasible. Computes target hidden states at training time using
    a 4-bit quantized target model co-resident on GPU.
    """
    pass
```

- [ ] **Step 2: Update `deepspec/trainer/__init__.py`**

```python
from .qwen3_5_dspark_trainer import Qwen3_5DSparkTrainer, OnlineQwen3_5DSparkTrainer
```

And add `"OnlineQwen3_5DSparkTrainer"` to `__all__`.

- [ ] **Step 3: Create `config/dspark/dspark_ornith_9b_local.py`**

```python
import os

from deepspec.trainer import OnlineQwen3_5DSparkTrainer
from deepspec.utils.constant import BASE_CKPT_DIR, BASE_TB_DIR, Ornith_9B

project_name = "deepspec"
exp_name = "dspark_ornith_9b_local"
seed = 42

model = dict(
    target_model_name_or_path=Ornith_9B,
    block_size=7,
    num_draft_layers=5,
    target_layer_ids=[3, 11, 19, 27],
    mask_token_id=151669,
    num_anchors=512,

    ## markov head
    markov_rank=256,
    markov_head_type='vanilla',

    ## confidence head
    confidence_head_alpha=1.0,
    confidence_head_with_markov=True,

    ## loss
    loss_decay_gamma=4.0,
    ce_loss_alpha=0.1,
    l1_loss_alpha=0.9,

    # Online mode (ignored by cloud config)
    online_target=dict(
        enabled=True,
        train_data_path="./data/train.jsonl",
    ),
)

train = dict(
    trainer_cls=OnlineQwen3_5DSparkTrainer,
    lr=6.0e-4,
    warmup_ratio=0.04,
    weight_decay=0.0,
    precision="bf16",
    local_batch_size=1,
    global_batch_size=32,  # Reduced for single GPU
    num_train_epochs=10,
    max_train_steps=None,
    max_grad_norm=1.0,
    sharding_strategy="no_shard",
    torch_compile=False,  # Disabled for stability on first run
)

logging = dict(
    logging_steps=10,
    checkpointing_steps=500,
)

data = dict(
    target_cache_path=None,
    chat_template="qwen",
    max_length=2048,
    min_loss_tokens=14,
    num_workers=0,
    train_data_path="./data/train.jsonl",
)


def finalize_cfg(cfg):
    logging_cfg = dict(cfg["logging"])
    project_name = str(cfg['project_name'])
    exp_name = str(cfg["exp_name"])
    logging_cfg["checkpoint_dir"] = os.path.join(BASE_CKPT_DIR, project_name, exp_name)
    logging_cfg["tensorboard_dir"] = os.path.join(BASE_TB_DIR, project_name, exp_name)
    cfg["logging"] = logging_cfg
    return cfg
```

- [ ] **Step 4: Verify config loads**

```bash
cd F:\Projects\DeepSpec
python -c "
from deepspec.utils import load_config
cfg = load_config('config/dspark/dspark_ornith_9b_local.py')
print(f'Config loaded: {cfg.exp_name}')
print(f'Trainer: {cfg.train.trainer_cls.__name__}')
"
```

Expected:
```
Config loaded: dspark_ornith_9b_local
Trainer: OnlineQwen3_5DSparkTrainer
```

- [ ] **Step 5: Commit**

```bash
git add deepspec/trainer/qwen3_5_dspark_trainer.py deepspec/trainer/__init__.py config/dspark/dspark_ornith_9b_local.py
git commit -m "feat: wire OnlineQwen3_5DSparkTrainer and add local hardware config"
```

---

### Task 5: Create llama.cpp Answer Generation Script

**Files:**
- Create: `scripts/data/generate_with_llamacpp.py`
- Reference: `scripts/data/generate_train_data.py` (existing SGLang-based script)

- [ ] **Step 1: Create `scripts/data/generate_with_llamacpp.py`**

```python
import argparse
import json
import os
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed

from openai import OpenAI


def call_llamacpp(client, messages, max_tokens, temperature, top_p, top_k):
    """Send a single conversation to llama.cpp server and get response."""
    try:
        response = client.chat.completions.create(
            model="ornith",
            messages=messages,
            max_tokens=max_tokens,
            temperature=temperature,
            top_p=top_p,
            extra_body={"top_k": top_k} if top_k > 0 else {},
        )
        return response.choices[0].message.content
    except Exception as e:
        return None


def process_item(client, item, max_tokens, temperature, top_p, top_k):
    """Process a single dataset item: send to server, return augmented item."""
    messages = item.get("messages", [])
    if not messages:
        return None

    response_text = call_llamacpp(
        client, messages, max_tokens, temperature, top_p, top_k,
    )
    if response_text is None:
        return None

    result = dict(item)
    result["choices"] = [
        {
            "message": {
                "role": "assistant",
                "content": response_text,
            }
        }
    ]
    result["_regenerated"] = True
    return result


def count_lines(path):
    with open(path, "r", encoding="utf-8") as f:
        return sum(1 for _ in f)


def main():
    parser = argparse.ArgumentParser(
        description="Regenerate dataset answers using llama.cpp server"
    )
    parser.add_argument("--model", default="ornith", help="Model name (ignored, uses server's loaded model)")
    parser.add_argument("--server-address", default="http://localhost:8080/v1")
    parser.add_argument("--input-file-path", required=True)
    parser.add_argument("--output-file-path", required=True)
    parser.add_argument("--concurrency", type=int, default=4)
    parser.add_argument("--temperature", type=float, default=0.6)
    parser.add_argument("--top-p", type=float, default=0.95)
    parser.add_argument("--top-k", type=int, default=20)
    parser.add_argument("--max-tokens", type=int, default=4096)
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args()

    client = OpenAI(base_url=args.server_address, api_key="not-needed")

    # Count input lines
    total = count_lines(args.input_file_path)
    print(f"Total input lines: {total}", flush=True)

    # Resume support
    start_line = 0
    output_f = None
    error_f = None
    if args.resume and os.path.exists(args.output_file_path):
        start_line = count_lines(args.output_file_path)
        print(f"Resuming from line {start_line}", flush=True)
        output_f = open(args.output_file_path, "a", encoding="utf-8")
        error_path = args.output_file_path.replace(".jsonl", "_error.jsonl")
        error_f = open(error_path, "a", encoding="utf-8")
    else:
        output_f = open(args.output_file_path, "w", encoding="utf-8")
        error_path = args.output_file_path.replace(".jsonl", "_error.jsonl")
        error_f = open(error_path, "w", encoding="utf-8")

    assert output_f is not None and error_f is not None

    # Read items, skipping already-processed
    items = []
    with open(args.input_file_path, "r", encoding="utf-8") as f:
        for i, line in enumerate(f):
            if i < start_line:
                continue
            items.append(json.loads(line))

    processed = 0
    errors = 0
    start_time = time.time()

    with ThreadPoolExecutor(max_workers=args.concurrency) as executor:
        futures = {
            executor.submit(
                process_item, client, item,
                args.max_tokens, args.temperature, args.top_p, args.top_k,
            ): i
            for i, item in enumerate(items)
        }

        for future in as_completed(futures):
            result = future.result()
            if result is not None:
                output_f.write(json.dumps(result, ensure_ascii=False) + "\n")
                output_f.flush()
                processed += 1
            else:
                idx = futures[future]
                error_f.write(json.dumps({"index": idx, "error": "API call failed"}, ensure_ascii=False) + "\n")
                error_f.flush()
                errors += 1

            elapsed = time.time() - start_time
            rate = processed / elapsed if elapsed > 0 else 0
            if (processed + errors) % 10 == 0:
                print(
                    f"  Processed: {processed}/{len(items)}, "
                    f"Errors: {errors}, "
                    f"Rate: {rate:.1f} items/sec",
                    flush=True,
                )

    output_f.close()
    error_f.close()

    elapsed = time.time() - start_time
    print(f"\nDone. {processed}/{len(items)} processed, {errors} errors in {elapsed:.0f}s", flush=True)


if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Verify syntax**

```bash
cd F:\Projects\DeepSpec
python -c "import ast; ast.parse(open('scripts/data/generate_with_llamacpp.py').read()); print('Syntax OK')"
```

Expected: `Syntax OK`

- [ ] **Step 3: Test connection to llama.cpp server**

Start llama.cpp server in another terminal:
```bash
cd F:\Projects\llama.cpp\build_cuda13
.\bin\llama-server.exe -m "C:\Users\ashah\.ollama\models\ornith-1.0-9b-Q4_K_M.gguf" --port 8080 -c 4096
```

Then run a quick test:
```bash
cd F:\Projects\DeepSpec
python -c "
from openai import OpenAI
client = OpenAI(base_url='http://localhost:8080/v1', api_key='not-needed')
resp = client.chat.completions.create(
    model='ornith',
    messages=[{'role': 'user', 'content': 'Say hello in one word'}],
    max_tokens=10,
)
print(f'Response: {resp.choices[0].message.content}')
"
```

Expected: `Response: Hello` (or similar)

- [ ] **Step 4: Commit**

```bash
git add scripts/data/generate_with_llamacpp.py
git commit -m "feat: add llama.cpp answer generation script"
```

---

### Task 6: Integration Smoke Test

**Files:** None (verification only)

- [ ] **Step 1: Verify all new modules import cleanly**

```bash
cd F:\Projects\DeepSpec
python -c "
from deepspec.data.online_cache_collator import OnlineCacheCollator
from deepspec.trainer.online_target_mixin import OnlineTargetTrainingMixin
from deepspec.trainer import OnlineQwen3_5DSparkTrainer
print('All online training imports OK')
"
```

Expected: `All online training imports OK`

- [ ] **Step 2: Create a tiny test dataset**

```bash
cd F:\Projects\DeepSpec
mkdir -p data
python -c "
import json
# 3 tiny conversations for smoke test
samples = [
    {'messages': [{'role': 'user', 'content': 'Hello'}, {'role': 'assistant', 'content': 'Hi there!'}]},
    {'messages': [{'role': 'user', 'content': 'What is 2+2?'}, {'role': 'assistant', 'content': '4'}]},
    {'messages': [{'role': 'user', 'content': 'Say bye'}, {'role': 'assistant', 'content': 'Goodbye!'}]},
]
with open('data/test_smoke.jsonl', 'w') as f:
    for s in samples:
        f.write(json.dumps(s) + '\n')
print(f'Wrote {len(samples)} test samples')
"
```

- [ ] **Step 3: Run one training step with online target mode**

```bash
cd F:\Projects\DeepSpec
python -c "
import torch, os
os.environ['MASTER_ADDR'] = '127.0.0.1'
os.environ['MASTER_PORT'] = '29501'

from deepspec.utils import load_config
from deepspec.trainer import OnlineQwen3_5DSparkTrainer

# Quick smoke test: instantiate trainer with tiny config override
cfg = load_config('config/dspark/dspark_ornith_9b_local.py')
cfg.model.online_target.train_data_path = './data/test_smoke.jsonl'
cfg.data.train_data_path = './data/test_smoke.jsonl'
cfg.data.max_length = 128
cfg.train.max_train_steps = 1
cfg.train.num_train_epochs = 1
cfg.train.global_batch_size = 2
cfg.train.checkpointing_steps = 1000
cfg.logging.checkpointing_steps = 1000

# Run one step
trainer = OnlineQwen3_5DSparkTrainer(0, cfg)
trainer.train()
trainer.clean_up()
print('SMOKE TEST PASSED: One training step completed')
"
```

- [ ] **Step 4: Verify generated answer format**

```bash
cd F:\Projects\DeepSpec
python scripts/data/generate_with_llamacpp.py \
    --input-file-path data/test_smoke.jsonl \
    --output-file-path data/test_smoke_regen.jsonl \
    --concurrency 4 \
    --max-tokens 50
cat data/test_smoke_regen.jsonl
```

Expected: 3 lines of JSON with regenerated assistant responses.

- [ ] **Step 5: Push all commits**

```bash
cd F:\Projects\DeepSpec
git push fork ornith-dspark
```
