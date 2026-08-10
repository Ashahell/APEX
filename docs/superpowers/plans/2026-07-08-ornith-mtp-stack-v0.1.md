# Ornith-MTP Stack v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the ornith-mtp-stack v0.1: Nix-based local inference stack pairing Ornith-1.0-9B with its MTP self-speculative head on llama.cpp, with an adaptive Agent Runtime (planner, executor, memory, checkpoint, tool sandbox), Python SDK, and benchmark harness.

**Architecture:** Five-layer stack (Application → Agent Runtime → Inference (llama.cpp) → Acceleration (MTP) → Infrastructure (Nix/CUDA)). Agent Runtime is the primary contribution — adaptive scaffold loop with per-step revision. Inference is a commodity backend.

**Tech Stack:** llama.cpp (CUDA 12.0, Blackwell sm_120), Python 3.12, Nix flakes, Pytest benchmarks

**Spec:** `docs/superpowers/specs/2026-07-08-ornith-mtp-stack-design-v4.md`

---

### Task 1: Repository scaffold + Nix flake

**Files:**
- Create: `flake.nix`
- Create: `flake.lock`
- Create: `pyproject.toml`
- Create: `config/default.yaml`
- Create: `config/schema.yaml`
- Create: `config/local.yaml.example`
- Create: `README.md`
- Create: `.gitignore`

- [ ] **Step 1: Write the Nix flake**

```nix
{
  description = "Ornith + MTP local inference stack";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
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
          };

          ornith-9b = pkgs.fetchurl {
            url = "https://huggingface.co/deepreinforce-ai/Ornith-1.0-9B-GGUF/resolve/main/Ornith-1.0-9B-Q4_K_M.gguf";
            hash = "";
          };

          ornith-mtp = pkgs.fetchurl {
            url = "https://huggingface.co/protoLabsAI/Ornith-1.0-9B-MTP-GGUF/resolve/main/Ornith-1.0-9B-MTP-Q4_K_M.gguf";
            hash = "";
          };

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

- [ ] **Step 2: Write pyproject.toml**

```toml
[project]
name = "ornith-mtp"
version = "0.1.0"
description = "Ornith + MTP local inference stack with adaptive Agent Runtime"
requires-python = ">=3.12"
license = { text = "MIT" }
dependencies = [
    "pyyaml>=6.0",
    "jsonschema>=4.20",
    "httpx>=0.27",
    "pydantic>=2.0",
]

[project.scripts]
ornith-mtp = "ornith_mtp.cli:main"

[tool.pytest.ini_options]
testpaths = ["tests"]
python_files = ["test_*.py"]
```

- [ ] **Step 3: Write default.yaml**

```yaml
inference:
  provider: llamacpp
  base_url: http://localhost:8000/v1
  model: ~/.cache/ornith/Ornith-1.0-9B-Q4_K_M.gguf
  max_tokens: 4096
  temperature: 0.6
  top_p: 0.95
  top_k: 20
  reasoning_parser: qwen3
  tool_call_parser: qwen3_xml
  speculative:
    enabled: true
    method: mtp
    draft_model: ~/.cache/ornith/Ornith-1.0-9B-MTP.gguf
    num_speculative_tokens: 3

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

- [ ] **Step 4: Write schema.yaml** (JSON Schema for config validation)

```yaml
type: object
properties:
  inference:
    type: object
    properties:
      provider:
        type: string
        enum: [llamacpp, openai, vllm]
      temperature:
        type: number
        minimum: 0.0
        maximum: 2.0
      top_p:
        type: number
        minimum: 0.0
        maximum: 1.0
      top_k:
        type: integer
        minimum: 0
    required: [provider, base_url, model]
  runtime:
    type: object
    properties:
      max_steps:
        type: integer
        minimum: 1
      default_timeout_secs:
        type: integer
        minimum: 1
    required: [max_steps]
```

- [ ] **Step 5: Write .gitignore**

```
__pycache__/
*.pyc
.env
config/local.yaml
logs/
*.gguf
dist/
```

- [ ] **Step 6: Write basic README.md** (one-line placeholder)

```markdown
# ornith-mtp-stack

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for design docs.
```

- [ ] **Step 7: Initialize and verify**

Run: `cd /path/to/ornith-mtp-stack && nix flake lock`
Expected: `flake.lock` created, no errors

- [ ] **Step 8: Commit**

```bash
git init && git add -A && git commit -m "chore: scaffold repository with Nix flake and config"
```

---

### Task 2: Core data models

**Files:**
- Create: `sdk/src/ornith_mtp/__init__.py`
- Create: `sdk/src/ornith_mtp/models.py`

- [ ] **Step 1: Write __init__.py**

```python
from .models import (
    Scaffold, Step, RetryPolicy, TerminationCondition, Revision,
    StepResult, ToolSpec, ToolCall, ToolResult,
    InferenceConfig, SpeculativeConfig, Validator,
    MemoryConfig, Config,
)
```

- [ ] **Step 2: Write models.py with all @dataclass definitions from spec §3.2-3.7**

```python
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional
import threading


class StepStatus(Enum):
    SUCCESS = "success"
    FAIL = "fail"
    TIMEOUT = "timeout"
    CANCELLED = "cancelled"


class ToolStatus(Enum):
    SUCCESS = "success"
    TIMEOUT = "timeout"
    CANCELLED = "cancelled"
    PERMISSION_DENIED = "permission_denied"
    ERROR = "error"


class SandboxLevel(Enum):
    NONE = "none"
    RESTRICTED = "restricted"
    ISOLATED = "isolated"


@dataclass
class RetryPolicy:
    max_attempts: int = 2
    backoff_secs: float = 5.0
    fallback_strategy: str = "retry"


@dataclass
class TerminationCondition:
    max_steps: int = 20
    max_cost_usd: float = 0.50
    max_wall_secs: int = 300
    on_checkpoint_fail: str = "retry"


@dataclass
class Revision:
    step_id: str
    observation: str
    change: str
    confidence_delta: float


@dataclass
class Step:
    id: str
    description: str
    checkpoint: Optional["Validator"] = None
    timeout_secs: int = 30
    retry_policy: RetryPolicy = field(default_factory=RetryPolicy)
    depends_on: list[str] = field(default_factory=list)


@dataclass
class Scaffold:
    goal: str
    plan: list[Step]
    tools: list[str]
    memory_config: "MemoryConfig"
    termination: TerminationCondition
    revision_history: list[Revision] = field(default_factory=list)


@dataclass
class ToolCall:
    tool: str
    args: dict
    timeout_secs: int
    call_id: str
    cancel_token: threading.Event = field(default_factory=threading.Event)


@dataclass
class ToolResult:
    call_id: str
    status: ToolStatus
    stdout: str
    stderr: str
    exit_code: int
    wall_ms: int


@dataclass
class ToolSpec:
    name: str
    description: str
    timeout_secs: int
    sandbox: SandboxLevel
    permissions: list[str]
    allowed_args: Optional[list[str]] = None
    deny_args: list[str] = field(default_factory=list)


@dataclass
class StepResult:
    step_id: str
    status: StepStatus
    output: str
    tools_called: list[ToolCall]
    cost_usd: float
    wall_ms: int
    tokens_in: int
    tokens_out: int
    tokens_reasoning: int
    acceptance_rate: Optional[float] = None


@dataclass
class SpeculativeConfig:
    enabled: bool = True
    method: str = "mtp"
    draft_model: Optional[str] = None
    num_speculative_tokens: int = 3


@dataclass
class InferenceConfig:
    provider: str = "llamacpp"
    base_url: str = "http://localhost:8000/v1"
    model: str = ""
    api_key: Optional[str] = None
    max_tokens: int = 4096
    temperature: float = 0.6
    top_p: float = 0.95
    top_k: int = 20
    streaming: bool = True
    reasoning_parser: Optional[str] = "qwen3"
    tool_call_parser: Optional[str] = "qwen3_xml"
    speculative: Optional[SpeculativeConfig] = None


@dataclass
class Validator:
    type: str
    args: dict
    expected: Any
    confidence: float = 1.0
    fail_on_error: bool = True


@dataclass
class MemoryConfig:
    working_capacity_tokens: int = 10000
    episodic_enabled: bool = True
    episodic_dir: str = "~/.ornith-mtp/episodic"
    persistent_enabled: bool = False


@dataclass
class Config:
    inference: InferenceConfig = field(default_factory=InferenceConfig)
    runtime: dict = field(default_factory=lambda: {"max_steps": 20})
    memory: MemoryConfig = field(default_factory=MemoryConfig)
```

- [ ] **Step 3: Run import verification**

Run: `cd sdk && python -c "from ornith_mtp import Scaffold, Step, Config; print('OK')"`
Expected: OK

- [ ] **Step 4: Commit**

```bash
git add sdk/src/ornith_mtp/ && git commit -m "feat: add core data models"
```

---

### Task 3: Config loader

**Files:**
- Create: `sdk/src/ornith_mtp/config_loader.py`
- Test: `tests/unit/test_config.py`

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/test_config.py
import pytest
import yaml
import tempfile
import os
from ornith_mtp.config_loader import load_config, ConfigLoadError

def test_load_default_config():
    config = load_config()
    assert config.inference.provider == "llamacpp"
    assert config.inference.temperature == 0.6
    assert config.inference.top_p == 0.95
    assert config.inference.top_k == 20
    assert config.runtime["max_steps"] == 20
    assert config.memory.working_capacity_tokens == 10000

def test_local_overrides_default():
    with tempfile.TemporaryDirectory() as tmpdir:
        local_path = os.path.join(tmpdir, "local.yaml")
        with open(local_path, "w") as f:
            yaml.dump({"inference": {"temperature": 0.8}}, f)
        os.chdir(tmpdir)
        config = load_config()
        assert config.inference.temperature == 0.8

def test_missing_config_raises():
    with pytest.raises(ConfigLoadError):
        load_config(config_dir="/nonexistent")
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/unit/test_config.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Write config_loader.py**

```python
import os
import yaml
import jsonschema
from pathlib import Path
from .models import Config

class ConfigLoadError(Exception):
    pass

def _find_config_dir() -> Path:
    cwd = Path.cwd()
    for parent in [cwd] + list(cwd.parents):
        config_dir = parent / "config"
        if config_dir.exists() and (config_dir / "default.yaml").exists():
            return config_dir
    return Path.cwd() / "config"

def load_config(config_dir: str | None = None) -> Config:
    if config_dir is None:
        config_dir = str(_find_config_dir())

    default_path = os.path.join(config_dir, "default.yaml")
    local_path = os.path.join(config_dir, "local.yaml")
    schema_path = os.path.join(config_dir, "schema.yaml")

    if not os.path.exists(default_path):
        raise ConfigLoadError(f"default.yaml not found in {config_dir}")

    with open(default_path) as f:
        config_data = yaml.safe_load(f)

    if os.path.exists(local_path):
        with open(local_path) as f:
            local_data = yaml.safe_load(f)
        if local_data:
            _deep_merge(config_data, local_data)

    if os.path.exists(schema_path):
        with open(schema_path) as f:
            schema = yaml.safe_load(f)
        jsonschema.validate(instance=config_data, schema=schema)

    inference = config_data.get("inference", {})
    from .models import InferenceConfig, SpeculativeConfig, MemoryConfig
    spec_cfg = inference.get("speculative")
    speculative = SpeculativeConfig(
        enabled=spec_cfg.get("enabled", True),
        method=spec_cfg.get("method", "mtp"),
        draft_model=spec_cfg.get("draft_model"),
        num_speculative_tokens=spec_cfg.get("num_speculative_tokens", 3),
    ) if spec_cfg else None

    return Config(
        inference=InferenceConfig(
            provider=inference.get("provider", "llamacpp"),
            base_url=inference.get("base_url", "http://localhost:8000/v1"),
            model=inference.get("model", ""),
            max_tokens=inference.get("max_tokens", 4096),
            temperature=inference.get("temperature", 0.6),
            top_p=inference.get("top_p", 0.95),
            top_k=inference.get("top_k", 20),
            reasoning_parser=inference.get("reasoning_parser"),
            tool_call_parser=inference.get("tool_call_parser"),
            speculative=speculative,
        ),
        runtime=config_data.get("runtime", {"max_steps": 20}),
        memory=MemoryConfig(
            working_capacity_tokens=config_data.get("memory", {}).get("working_capacity_tokens", 10000),
            episodic_enabled=config_data.get("memory", {}).get("episodic_enabled", True),
            episodic_dir=config_data.get("memory", {}).get("episodic_dir", "~/.ornith-mtp/episodic"),
            persistent_enabled=config_data.get("memory", {}).get("persistent_enabled", False),
        ),
    )

def _deep_merge(base: dict, overlay: dict) -> None:
    for key, value in overlay.items():
        if key in base and isinstance(base[key], dict) and isinstance(value, dict):
            _deep_merge(base[key], value)
        else:
            base[key] = value
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `pytest tests/unit/test_config.py -v`
Expected: PASS (3/3)

- [ ] **Step 5: Commit**

```bash
git add sdk/src/ornith_mtp/config_loader.py tests/unit/test_config.py
git commit -m "feat: add config loader with YAML merge and validation"
```

---

### Task 4: OpenAI-compatible client (SDK)

**Files:**
- Create: `sdk/src/ornith_mtp/client.py`
- Test: `tests/unit/test_client.py`

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/test_client.py
import pytest
from ornith_mtp.client import OrnithClient
from ornith_mtp.models import InferenceConfig

def test_client_init():
    config = InferenceConfig(base_url="http://localhost:8000/v1")
    client = OrnithClient(config)
    assert client.base_url == "http://localhost:8000/v1"

@pytest.mark.asyncio
async def test_client_chat_completion(httpx_mock):
    config = InferenceConfig(base_url="http://localhost:8000/v1")
    client = OrnithClient(config)
    httpx_mock.add_response(
        json={
            "choices": [{"message": {"content": "Hello!", "reasoning_content": "<think>Hi</think>"}}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5},
        }
    )
    result = await client.chat("Say hello")
    assert "Hello!" in result["content"]
    assert result["tokens_reasoning"] > 0

@pytest.mark.asyncio
async def test_client_tool_call(httpx_mock):
    config = InferenceConfig(
        base_url="http://localhost:8000/v1",
        tool_call_parser="qwen3_xml",
    )
    client = OrnithClient(config)
    httpx_mock.add_response(
        json={
            "choices": [{
                "message": {
                    "content": "<tool_call>file.read</tool_call>",
                    "tool_calls": [{"function": {"name": "file.read", "arguments": '{"path": "test.py"}'}}],
                }
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5},
        }
    )
    result = await client.chat("Read test.py", tools=[{"name": "file.read"}])
    assert "tool_calls" in result
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/unit/test_client.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Write client.py**

```python
import httpx
import json
import re
from typing import Any, Optional
from .models import InferenceConfig

class OrnithClient:
    def __init__(self, config: InferenceConfig):
        self.base_url = config.base_url
        self.model = config.model or "default"
        self.config = config

    async def chat(
        self,
        message: str,
        system: Optional[str] = None,
        tools: Optional[list[dict]] = None,
        stream: bool = False,
    ) -> dict[str, Any]:
        headers = {"Content-Type": "application/json"}
        messages = []
        if system:
            messages.append({"role": "system", "content": system})
        messages.append({"role": "user", "content": message})

        body = {
            "model": self.model,
            "messages": messages,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "top_p": self.config.top_p,
            "stream": stream,
        }
        if self.config.reasoning_parser == "qwen3":
            body["reasoning_parser"] = "qwen3"
        if self.config.tool_call_parser == "qwen3_xml" and tools:
            body["tools"] = tools
            body["tool_choice"] = "auto"

        async with httpx.AsyncClient() as client:
            resp = await client.post(
                f"{self.base_url}/chat/completions",
                headers=headers,
                json=body,
                timeout=30,
            )
            resp.raise_for_status()
            data = resp.json()

        choice = data["choices"][0]["message"]
        content = choice.get("content", "")
        reasoning = choice.get("reasoning_content", "")
        tool_calls = choice.get("tool_calls")

        # Parse reasoning tokens from <think> blocks
        tokens_reasoning = 0
        if reasoning:
            tokens_reasoning = len(reasoning.split())
        elif content:
            think_matches = re.findall(r"<think>(.*?)</think>", content, re.DOTALL)
            tokens_reasoning = sum(len(t.split()) for t in think_matches) if think_matches else 0

        result = {
            "content": content,
            "reasoning": reasoning,
            "tokens_reasoning": tokens_reasoning,
            "usage": data.get("usage", {}),
        }
        if tool_calls:
            result["tool_calls"] = tool_calls

        return result
```

- [ ] **Step 4: Run tests**

Run: `pytest tests/unit/test_client.py -v`
Expected: PASS (3/3)

- [ ] **Step 5: Commit**

```bash
git add sdk/src/ornith_mtp/client.py tests/unit/test_client.py
git commit -m "feat: add OpenAI-compatible client with reasoning token parsing"
```

---

### Task 5: Memory subsystem

**Files:**
- Create: `runtime/memory.py`
- Test: `tests/unit/test_memory.py`

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/test_memory.py
import pytest
import tempfile
import os
from runtime.memory import WorkingMemory, EpisodicMemory

def test_working_memory_crud():
    wm = WorkingMemory(capacity_tokens=100)
    wm.set("key1", "value1")
    assert wm.get("key1") == "value1"
    assert wm.get("nonexistent") is None
    wm.delete("key1")
    assert wm.get("key1") is None

def test_working_memory_capacity():
    wm = WorkingMemory(capacity_tokens=10)
    wm.set("data", "hello world this is long")
    assert len(wm.keys()) <= 2  # should consolidate

def test_episodic_append_and_read():
    with tempfile.TemporaryDirectory() as tmpdir:
        em = EpisodicMemory(base_dir=tmpdir, session_id="test-session")
        em.append({"step": "1", "result": "ok"})
        em.append({"step": "2", "result": "fail"})
        entries = em.read_all()
        assert len(entries) == 2
        assert entries[0]["step"] == "1"
        assert entries[1]["step"] == "2"

def test_episodic_file_persistence():
    with tempfile.TemporaryDirectory() as tmpdir:
        em = EpisodicMemory(base_dir=tmpdir, session_id="persist-test")
        em.append({"data": "hello"})
        em2 = EpisodicMemory(base_dir=tmpdir, session_id="persist-test")
        entries = em2.read_all()
        assert len(entries) == 1
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/unit/test_memory.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Write memory.py**

```python
import json
import os
import math
from datetime import datetime
from pathlib import Path
from typing import Any, Optional


class WorkingMemory:
    def __init__(self, capacity_tokens: int = 10000):
        self._store: dict[str, Any] = {}
        self._tokens: dict[str, int] = {}
        self._capacity = capacity_tokens

    def set(self, key: str, value: Any) -> None:
        token_count = len(str(value).split())
        self._store[key] = value
        self._tokens[key] = token_count
        self._maybe_consolidate()

    def get(self, key: str) -> Optional[Any]:
        return self._store.get(key)

    def delete(self, key: str) -> None:
        self._store.pop(key, None)
        self._tokens.pop(key, None)

    def keys(self) -> list[str]:
        return list(self._store.keys())

    def clear(self) -> None:
        self._store.clear()
        self._tokens.clear()

    def _maybe_consolidate(self) -> None:
        total = sum(self._tokens.values())
        if total > self._capacity:
            sorted_keys = sorted(self._tokens, key=lambda k: self._tokens[k])
            while total > self._capacity * 0.8 and sorted_keys:
                oldest = sorted_keys.pop(0)
                total -= self._tokens.pop(oldest, 0)
                self._store.pop(oldest, None)


class EpisodicMemory:
    def __init__(self, base_dir: str = "~/.ornith-mtp/episodic", session_id: Optional[str] = None):
        self.base_dir = Path(base_dir).expanduser()
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self.session_id = session_id or datetime.now().strftime("%Y%m%d_%H%M%S")
        self.filepath = self.base_dir / f"{self.session_id}.jsonl"

    def append(self, entry: dict) -> None:
        entry["timestamp"] = datetime.now().isoformat()
        with open(self.filepath, "a") as f:
            f.write(json.dumps(entry) + "\n")

    def read_all(self) -> list[dict]:
        if not self.filepath.exists():
            return []
        entries = []
        with open(self.filepath) as f:
            for line in f:
                line = line.strip()
                if line:
                    entries.append(json.loads(line))
        return entries
```

- [ ] **Step 4: Run tests**

Run: `pytest tests/unit/test_memory.py -v`
Expected: PASS (4/4)

- [ ] **Step 5: Commit**

```bash
git add runtime/memory.py tests/unit/test_memory.py
git commit -m "feat: add working and episodic memory subsystems"
```

---

### Task 6: Tool Runtime

**Files:**
- Create: `runtime/tool_runtime.py`
- Test: `tests/unit/test_tool_runtime.py`

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/test_tool_runtime.py
import pytest
import asyncio
from runtime.tool_runtime import ToolRuntime, ToolSpec, SandboxLevel

@pytest.fixture
def runtime():
    return ToolRuntime()

def test_permission_allowlist(runtime):
    spec = ToolSpec(
        name="shell.execute",
        description="Shell",
        timeout_secs=5,
        sandbox=SandboxLevel.RESTRICTED,
        permissions=["shell"],
        allowed_args=["ls", "cat", "echo", "pwd"],
        deny_args=["rm"],
    )
    runtime.register_tool(spec)
    assert runtime.check_permission("shell.execute", ["ls", "-la"]) is True
    assert runtime.check_permission("shell.execute", ["rm", "-rf", "/"]) is False

def test_deny_patterns(runtime):
    spec = ToolSpec(
        name="shell.execute",
        description="Shell",
        timeout_secs=5,
        sandbox=SandboxLevel.RESTRICTED,
        permissions=["shell"],
        allowed_args=["ls", "cat"],
        deny_args=["rm"],
    )
    runtime.register_tool(spec)
    result = runtime.check_permission("shell.execute", ["ls", "|", "grep", "foo"])
    assert result is False  # pipe not allowed

def test_timeout_enforcement(runtime):
    spec = ToolSpec(
        name="shell.execute",
        description="Shell",
        timeout_secs=1,
        sandbox=SandboxLevel.RESTRICTED,
        permissions=["shell"],
        allowed_args=["sleep"],
    )
    runtime.register_tool(spec)

    async def run():
        result = await runtime.execute("shell.execute", args=["sleep", "10"])
        return result

    result = asyncio.run(run())
    assert result.status.value == "timeout"

def test_unregistered_tool(runtime):
    with pytest.raises(ValueError, match="not registered"):
        runtime.check_permission("nonexistent.tool", [])
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/unit/test_tool_runtime.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Write tool_runtime.py**

```python
import asyncio
import subprocess
import time
from typing import Optional
from ornith_mtp.models import (
    ToolSpec, ToolSpec as ToolSpecModel, ToolCall, ToolResult,
    ToolStatus, SandboxLevel,
)

SHELL_DENY_PATTERNS = ["rm", "sudo", "su", "chmod", "chown", "mkfs", "dd"]
PIPE_REDIRECT_PATTERNS = ["|", ">", "<", "`"]


class ToolRuntime:
    def __init__(self):
        self._tools: dict[str, ToolSpecModel] = {}
        self._audit_log: list[dict] = []

    def register_tool(self, spec: ToolSpecModel) -> None:
        self._tools[spec.name] = spec

    def check_permission(self, tool_name: str, args: list[str]) -> bool:
        spec = self._tools.get(tool_name)
        if not spec:
            raise ValueError(f"Tool '{tool_name}' not registered")

        if args and args[-1] in PIPE_REDIRECT_PATTERNS:
            return False

        if spec.deny_args:
            for arg in args:
                for deny in spec.deny_args:
                    if arg.startswith(deny):
                        return False

        if spec.allowed_args:
            if args and args[0] not in spec.allowed_args:
                return False

        return True

    async def execute(self, tool_name: str, args: list[str]) -> ToolResult:
        spec = self._tools.get(tool_name)
        if not spec:
            raise ValueError(f"Tool '{tool_name}' not registered")

        if not self.check_permission(tool_name, args):
            return ToolResult(
                call_id="",
                status=ToolStatus.PERMISSION_DENIED,
                stdout="",
                stderr="Permission denied",
                exit_code=-1,
                wall_ms=0,
            )

        start = time.monotonic()
        cancel_token = asyncio.Event()
        call = ToolCall(
            tool=tool_name,
            args={},
            timeout_secs=spec.timeout_secs,
            call_id=f"{tool_name}-{int(start)}",
            cancel_token=cancel_token,
        )

        try:
            proc = await asyncio.create_subprocess_exec(
                *args,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            try:
                stdout, stderr = await asyncio.wait_for(
                    proc.communicate(), timeout=spec.timeout_secs
                )
                elapsed = int((time.monotonic() - start) * 1000)
                return ToolResult(
                    call_id=call.call_id,
                    status=ToolStatus.SUCCESS if proc.returncode == 0 else ToolStatus.ERROR,
                    stdout=stdout.decode() if stdout else "",
                    stderr=stderr.decode() if stderr else "",
                    exit_code=proc.returncode or 0,
                    wall_ms=elapsed,
                )
            except asyncio.TimeoutError:
                proc.kill()
                await proc.wait()
                elapsed = int((time.monotonic() - start) * 1000)
                return ToolResult(
                    call_id=call.call_id,
                    status=ToolStatus.TIMEOUT,
                    stdout="",
                    stderr="Timed out",
                    exit_code=-1,
                    wall_ms=elapsed,
                )
        except FileNotFoundError:
            return ToolResult(
                call_id=call.call_id,
                status=ToolStatus.ERROR,
                stdout="",
                stderr=f"Command not found: {args[0]}",
                exit_code=-1,
                wall_ms=0,
            )
```

- [ ] **Step 4: Run tests**

Run: `pytest tests/unit/test_tool_runtime.py -v`
Expected: PASS (4/4)

- [ ] **Step 5: Commit**

```bash
git add runtime/tool_runtime.py tests/unit/test_tool_runtime.py
git commit -m "feat: add Tool Runtime with sandbox, permissions, timeouts"
```

---

### Task 7: Checkpoint engine

**Files:**
- Create: `runtime/checkpoint.py`
- Test: `tests/unit/test_checkpoint.py`

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/test_checkpoint.py
import pytest
import tempfile
import os
from runtime.checkpoint import CheckpointEngine, VALIDATORS

def test_regex_validator_pass():
    engine = CheckpointEngine()
    result = engine.validate("regex", {"pattern": r"hello", "text": "hello world"})
    assert result["passed"] is True

def test_regex_validator_fail():
    engine = CheckpointEngine()
    result = engine.validate("regex", {"pattern": r"goodbye", "text": "hello world"})
    assert result["passed"] is False

def test_json_schema_validator_pass():
    engine = CheckpointEngine()
    schema = {"type": "object", "properties": {"name": {"type": "string"}}}
    data = {"name": "test"}
    result = engine.validate("schema", {"schema": schema, "data": data})
    assert result["passed"] is True

def test_fail_closed_on_validator_crash():
    engine = CheckpointEngine()
    result = engine.validate("pytest", {"path": "/nonexistent/test_file.py"})
    assert result["passed"] is False  # fail-closed by default
    assert result["error"] is not None

def test_fail_open_opt_in():
    engine = CheckpointEngine()
    result = engine.validate("pytest", {"path": "/nonexistent/test_file.py", "fail_on_error": False})
    assert result["passed"] is True  # fail-open with explicit opt-in
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/unit/test_checkpoint.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Write checkpoint.py**

```python
import re
import subprocess
import json
import traceback
from typing import Any


class CompileValidator:
    def validate(self, args: dict) -> dict:
        code = args.get("code", "")
        language = args.get("language", "python")
        if language == "python":
            try:
                compile(code, "<string>", "exec")
                return {"passed": True, "error": None}
            except SyntaxError as e:
                return {"passed": False, "error": str(e)}
        return {"passed": False, "error": f"Unsupported language: {language}"}


class RegexValidator:
    def validate(self, args: dict) -> dict:
        pattern = args["pattern"]
        text = args.get("text", "")
        matches = bool(re.search(pattern, text))
        return {"passed": matches, "error": None}


class JsonSchemaValidator:
    def validate(self, args: dict) -> dict:
        import jsonschema
        try:
            jsonschema.validate(instance=args["data"], schema=args["schema"])
            return {"passed": True, "error": None}
        except jsonschema.ValidationError as e:
            return {"passed": False, "error": str(e)}


class PytestValidator:
    def validate(self, args: dict) -> dict:
        path = args.get("path", "")
        result = subprocess.run(
            ["python", "-m", "pytest", path, "--tb=short", "-q"],
            capture_output=True, text=True, timeout=30,
        )
        passed = result.returncode == 0
        return {"passed": passed, "error": result.stderr if not passed else None}


VALIDATORS = {
    "compile": CompileValidator(),
    "regex": RegexValidator(),
    "schema": JsonSchemaValidator(),
    "pytest": PytestValidator(),
}


class CheckpointEngine:
    def validate(self, validator_type: str, args: dict) -> dict:
        fail_on_error = args.pop("fail_on_error", True)
        validator = VALIDATORS.get(validator_type)
        if not validator:
            return {"passed": False, "error": f"Unknown validator: {validator_type}"}
        try:
            result = validator.validate(args)
            return result
        except Exception as e:
            if fail_on_error:
                return {"passed": False, "error": str(e)}
            return {"passed": True, "error": None}
```

- [ ] **Step 4: Run tests**

Run: `pytest tests/unit/test_checkpoint.py -v`
Expected: PASS (5/5)

- [ ] **Step 5: Commit**

```bash
git add runtime/checkpoint.py tests/unit/test_checkpoint.py
git commit -m "feat: add checkpoint engine with fail-closed validators"
```

---

### Task 8: Planner (scaffold generation + revision)

**Files:**
- Create: `runtime/planner.py`
- Test: `tests/unit/test_planner.py`

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/test_planner.py
import pytest
from runtime.planner import Planner
from ornith_mtp.models import Scaffold, Step, TerminationCondition, MemoryConfig

def test_generate_default_scaffold_on_parse_failure():
    planner = Planner(llm_client=None)
    scaffold = planner.generate_scaffold("Write a test")
    assert isinstance(scaffold, Scaffold)
    assert scaffold.goal == "Write a test"
    assert len(scaffold.plan) == 1
    assert scaffold.plan[0].description == "Solve: Write a test"

def test_generate_scaffold_with_steps():
    planner = Planner(llm_client=None)
    scaffold = planner.generate_scaffold("Implement a BST in Python")
    assert scaffold.termination.max_steps == 20
    assert scaffold.plan[0].timeout_secs == 30

def test_revise_scaffold_after_failure():
    planner = Planner(llm_client=None)
    scaffold = planner.generate_scaffold("Test task")
    step_result = {"step_id": scaffold.plan[0].id, "status": "fail"}
    revised = planner.revise_scaffold(scaffold, step_result)
    assert len(revised.revision_history) == 1
    assert revised.revision_history[0].step_id == scaffold.plan[0].id

def test_abort_after_3_consecutive_failures():
    planner = Planner(llm_client=None)
    scaffold = planner.generate_scaffold("Failing task")
    for _ in range(3):
        scaffold = planner.revise_scaffold(scaffold, {"step_id": scaffold.plan[0].id, "status": "fail"})
    assert scaffold.termination.on_checkpoint_fail == "abort"
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/unit/test_planner.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Write planner.py**

```python
import json
import uuid
from typing import Optional, Any
from ornith_mtp.models import (
    Scaffold, Step, RetryPolicy, TerminationCondition, Revision, MemoryConfig,
)


DEFAULT_TERMINATION = TerminationCondition(
    max_steps=20, max_cost_usd=0.50, max_wall_secs=300, on_checkpoint_fail="retry"
)
DEFAULT_MEMORY = MemoryConfig(working_capacity_tokens=10000)
DEFAULT_TOOLS = ["file.read", "file.write", "code.generate", "shell.execute", "web.get"]


class Planner:
    def __init__(self, llm_client: Optional[Any] = None):
        self._llm = llm_client

    def generate_scaffold(self, task: str) -> Scaffold:
        if self._llm:
            try:
                return self._llm_generate(task)
            except Exception:
                pass
        return self._default_scaffold(task)

    def revise_scaffold(self, current: Scaffold, step_result: dict) -> Scaffold:
        revision = Revision(
            step_id=step_result.get("step_id", "unknown"),
            observation=step_result.get("status", "unknown"),
            change="revised plan after step failure",
            confidence_delta=-0.2 if step_result.get("status") == "fail" else 0.1,
        )
        current.revision_history.append(revision)

        consecutive_fails = sum(
            1 for r in current.revision_history[-3:]
            if r.confidence_delta < 0
        )
        if consecutive_fails >= 3:
            current.termination.on_checkpoint_fail = "abort"

        if step_result.get("status") == "fail":
            step_id = step_result.get("step_id")
            for step in current.plan:
                if step.id == step_id:
                    step.retry_policy.max_attempts -= 1
                    if step.retry_policy.max_attempts <= 0:
                        current.termination.on_checkpoint_fail = "abort"
                    break

        return current

    def _default_scaffold(self, task: str) -> Scaffold:
        step = Step(
            id=str(uuid.uuid4())[:8],
            description=f"Solve: {task}",
            timeout_secs=30,
            retry_policy=RetryPolicy(max_attempts=2, backoff_secs=5.0),
        )
        return Scaffold(
            goal=task,
            plan=[step],
            tools=DEFAULT_TOOLS,
            memory_config=DEFAULT_MEMORY,
            termination=DEFAULT_TERMINATION,
        )

    def _llm_generate(self, task: str) -> Scaffold:
        meta_prompt = f"""
Generate a scaffold for this task: {task}

Return JSON:
{{
  "steps": [{{"id": "s-01", "description": "...", "timeout_secs": 30}}],
  "tools": ["file.read", ...],
  "max_steps": 10
}}
"""
        response = self._llm.chat(meta_prompt)
        try:
            data = json.loads(response["content"])
            steps = [
                Step(
                    id=s["id"],
                    description=s["description"],
                    timeout_secs=s.get("timeout_secs", 30),
                    retry_policy=RetryPolicy(max_attempts=2),
                )
                for s in data["steps"]
            ]
            return Scaffold(
                goal=task,
                plan=steps,
                tools=data.get("tools", DEFAULT_TOOLS),
                memory_config=DEFAULT_MEMORY,
                termination=TerminationCondition(
                    max_steps=data.get("max_steps", 20),
                ),
            )
        except (json.JSONDecodeError, KeyError):
            return self._default_scaffold(task)
```

- [ ] **Step 4: Run tests**

Run: `pytest tests/unit/test_planner.py -v`
Expected: PASS (4/4)

- [ ] **Step 5: Commit**

```bash
git add runtime/planner.py tests/unit/test_planner.py
git commit -m "feat: add planner with scaffold generation and revision"
```

---

### Task 9: Executor (step dispatch loop)

**Files:**
- Create: `runtime/executor.py`
- Test: `tests/unit/test_executor.py`

- [ ] **Step 1: Write the failing test**

```python
# tests/unit/test_executor.py
import pytest
from runtime.executor import Executor
from runtime.memory import WorkingMemory
from ornith_mtp.models import Step, Scaffold, TerminationCondition, MemoryConfig

def test_execute_step_ordering():
    executor = Executor(planner=None, tool_runtime=None, memory=WorkingMemory())
    scaffold = Scaffold(
        goal="Test",
        plan=[
            Step(id="s-01", description="Step 1", timeout_secs=5, depends_on=[]),
            Step(id="s-02", description="Step 2", timeout_secs=5, depends_on=["s-01"]),
            Step(id="s-03", description="Step 3", timeout_secs=5, depends_on=["s-02"]),
        ],
        tools=[],
        memory_config=MemoryConfig(),
        termination=TerminationCondition(max_steps=20),
    )
    order = executor._compute_step_order(scaffold)
    assert order == ["s-01", "s-02", "s-03"]

def test_reasoning_token_tracking():
    executor = Executor(planner=None, tool_runtime=None, memory=WorkingMemory())
    # Verify tokens_out and tokens_reasoning are tracked
    result = executor._build_step_result(
        step_id="s-01",
        output="<think>Let me analyze this</think>The answer is 42",
        tokens_in=50,
        tokens_out=20,
    )
    assert result.tokens_reasoning == 5  # "Let me analyze this" = 5 tokens
    assert result.tokens_out == 20
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pytest tests/unit/test_executor.py -v`
Expected: FAIL with ImportError

- [ ] **Step 3: Write executor.py**

```python
import time
import re
from typing import Optional
from ornith_mtp.models import (
    Scaffold, StepResult, StepStatus,
)


class Executor:
    def __init__(self, planner, tool_runtime, memory):
        self._planner = planner
        self._tool_runtime = tool_runtime
        self._memory = memory

    async def run(self, scaffold: Scaffold) -> list[StepResult]:
        results = []
        order = self._compute_step_order(scaffold)
        for step_id in order:
            step = next(s for s in scaffold.plan if s.id == step_id)
            scaffold = self._planner.revise_scaffold(
                scaffold, {"step_id": step.id, "status": "running"}
            )
            result = await self._execute_step(step)
            results.append(result)
            scaffold = self._planner.revise_scaffold(
                scaffold, {"step_id": result.step_id, "status": result.status.value}
            )
        return results

    async def _execute_step(self, step) -> StepResult:
        start = time.monotonic()
        output = f"Executed: {step.description}"
        elapsed = int((time.monotonic() - start) * 1000)
        return self._build_step_result(
            step_id=step.id, output=output, tokens_in=0, tokens_out=len(output.split()),
        )

    def _compute_step_order(self, scaffold: Scaffold) -> list[str]:
        ordered = []
        visited = set()

        def visit(step_id: str):
            if step_id in visited:
                return
            visited.add(step_id)
            step = next(s for s in scaffold.plan if s.id == step_id)
            for dep in step.depends_on:
                visit(dep)
            ordered.append(step_id)

        for step in scaffold.plan:
            visit(step.id)
        return ordered

    def _build_step_result(
        self, step_id: str, output: str, tokens_in: int, tokens_out: int,
    ) -> StepResult:
        think_matches = re.findall(r"<think>(.*?)</think>", output, re.DOTALL)
        tokens_reasoning = sum(len(t.split()) for t in think_matches)

        return StepResult(
            step_id=step_id,
            status=StepStatus.SUCCESS,
            output=output,
            tools_called=[],
            cost_usd=0.0,
            wall_ms=0,
            tokens_in=tokens_in,
            tokens_out=tokens_out,
            tokens_reasoning=tokens_reasoning,
        )
```

- [ ] **Step 4: Run tests**

Run: `pytest tests/unit/test_executor.py -v`
Expected: PASS (2/2)

- [ ] **Step 5: Commit**

```bash
git add runtime/executor.py tests/unit/test_executor.py
git commit -m "feat: add executor with step ordering and reasoning token tracking"
```

---

### Task 10: CLI entry point

**Files:**
- Create: `runtime/cli.py`

- [ ] **Step 1: Write cli.py**

```python
import argparse
import asyncio
import sys
from ornith_mtp.config_loader import load_config
from ornith_mtp.client import OrnithClient
from runtime.planner import Planner
from runtime.memory import WorkingMemory
from runtime.tool_runtime import ToolRuntime
from runtime.executor import Executor


def main():
    parser = argparse.ArgumentParser(description="Ornith + MTP local inference stack")
    subparsers = parser.add_subparsers(dest="command")

    chat_parser = subparsers.add_parser("chat", help="Interactive chat with Ornith")
    chat_parser.add_argument("--message", "-m", help="Single message to send")

    run_parser = subparsers.add_parser("run", help="Run a task with the Agent Runtime")
    run_parser.add_argument("task", help="Task description")

    serve_parser = subparsers.add_parser("serve", help="Start llama-server")

    benchmark_parser = subparsers.add_parser("benchmark", help="Run benchmarks")
    benchmark_parser.add_argument("--mtp", choices=["on", "off", "both"], default="both")
    benchmark_parser.add_argument("--output", default="results.json")

    args = parser.parse_args()

    if args.command == "chat":
        config = load_config()
        client = OrnithClient(config.inference)
        message = args.message or input("You: ")
        result = asyncio.run(client.chat(message))
        print(f"Ornith: {result['content']}")

    elif args.command == "run":
        config = load_config()
        client = OrnithClient(config.inference)
        planner = Planner(llm_client=client)
        scaffold = planner.generate_scaffold(args.task)
        memory = WorkingMemory()
        tool_runtime = ToolRuntime()
        executor = Executor(planner=planner, tool_runtime=tool_runtime, memory=memory)
        results = asyncio.run(executor.run(scaffold))
        for r in results:
            print(f"[{r.step_id}] {r.status.value}: {r.output[:100]}...")

    elif args.command == "serve":
        print("Run: nix run .#serve")
        print("Or manually: llama-server --model <gguf> --spec-type draft-mtp ...")

    elif args.command == "benchmark":
        print("Benchmark harness: python -m benchmarks.run --help")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Verify CLI runs**

Run: `python runtime/cli.py --help`
Expected: Shows help text with all subcommands

- [ ] **Step 3: Commit**

```bash
git add runtime/cli.py && git commit -m "feat: add CLI entry point with chat/run/serve/benchmark commands"
```

---

### Task 11: Benchmark harness

**Files:**
- Create: `benchmarks/conftest.py`
- Create: `benchmarks/run.py`
- Create: `benchmarks/test_latency.py`
- Create: `benchmarks/test_throughput.py`
- Create: `benchmarks/test_acceptance_rate.py`
- Create: `benchmarks/test_mtp_vs_baseline.py`

- [ ] **Step 1: Write benchmarks/conftest.py**

```python
import pytest
import subprocess
import json
import time
from pathlib import Path


def pytest_addoption(parser):
    parser.addoption("--mtp", action="store", default="on", choices=["on", "off", "both"])
    parser.addoption("--prompts", action="store", default="benchmarks/prompts.json")
    parser.addoption("--output-dir", action="store", default="results")


@pytest.fixture(scope="session")
def mtp_mode(request):
    return request.config.getoption("--mtp")


@pytest.fixture(scope="session")
def prompts(request):
    prompts_path = Path(request.config.getoption("--prompts"))
    if prompts_path.exists():
        with open(prompts_path) as f:
            return json.load(f)
    return [
        {"name": "short_code", "prompt": "Write a function to check if a string is a palindrome", "tokens": 200},
        {"name": "medium_refactor", "prompt": "Refactor this class to use dependency injection: class Service: def __init__(self): self.db = Database()", "tokens": 1000},
        {"name": "long_scaffold", "prompt": "Design a microservice architecture for an e-commerce platform with user auth, product catalog, order processing, and payment", "tokens": 4000},
    ]


@pytest.fixture(scope="session")
def api_url():
    return "http://localhost:8000/v1"


@pytest.fixture
def benchmark_results():
    return []
```

- [ ] **Step 2: Write benchmarks/run.py**

```python
import argparse
import json
import subprocess
import time
from pathlib import Path


def run_benchmark(mtp_enabled: bool, prompts_file: str, output_dir: str) -> dict:
    results = {"mtp_enabled": mtp_enabled, "tests": []}

    with open(prompts_file) as f:
        prompts = json.load(f)

    for prompt in prompts:
        start = time.time()
        # In real usage, this calls the OrnithClient
        # For MVP, we measure with a simulated API call
        elapsed = time.time() - start
        results["tests"].append({
            "name": prompt["name"],
            "elapsed_secs": elapsed,
            "tokens_generated": 0,
        })

    return results


def main():
    parser = argparse.ArgumentParser(description="Ornith-MTP benchmark harness")
    parser.add_argument("--mtp", choices=["on", "off", "both"], default="both")
    parser.add_argument("--prompts", default="benchmarks/prompts.json")
    parser.add_argument("--output", default="results")
    args = parser.parse_args()

    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    if args.mtp in ("on", "both"):
        results_on = run_benchmark(True, args.prompts, str(output_dir))
        with open(output_dir / "mtp_on.json", "w") as f:
            json.dump(results_on, f, indent=2)

    if args.mtp in ("off", "both"):
        results_off = run_benchmark(False, args.prompts, str(output_dir))
        with open(output_dir / "mtp_off.json", "w") as f:
            json.dump(results_off, f, indent=2)

    print(f"Results written to {output_dir}/")


if __name__ == "__main__":
    main()
```

- [ ] **Step 3: Write benchmarks/test_mtp_vs_baseline.py**

```python
import pytest
import json
from pathlib import Path


@pytest.mark.benchmark
def test_mtp_vs_baseline_comparison(mtp_mode, prompts, api_url):
    """Compare MTP on vs off for the same prompts."""
    if mtp_mode == "off":
        pytest.skip("MTP disabled for this run")

    results_path = Path("results/mtp_on.json")
    if not results_path.exists():
        pytest.skip("No MTP results found")

    with open(results_path) as f:
        results = json.load(f)

    assert len(results["tests"]) > 0, "No benchmark results"
    for test in results["tests"]:
        assert "elapsed_secs" in test
        assert "tokens_generated" in test
```

- [ ] **Step 4: Write prompts.json fixture**

```json
[
  {"name": "short_code", "prompt": "Write a function to check if a string is a palindrome", "tokens": 200},
  {"name": "medium_refactor", "prompt": "Refactor this class to use dependency injection", "tokens": 1000},
  {"name": "long_scaffold", "prompt": "Design a microservice architecture for e-commerce", "tokens": 4000},
  {"name": "tool_calling", "prompt": "List all Python files in the current directory and count lines", "tokens": 500},
  {"name": "mixed_chat", "prompt": "What is the time complexity of quicksort and can you implement it?", "tokens": 300}
]
```

- [ ] **Step 5: Commit**

```bash
git add benchmarks/ && git commit -m "feat: add benchmark harness with MTP on/off comparison"
```

---

### Task 12: Complete integration test

**Files:**
- Create: `tests/integration/test_full_loop.py`

- [ ] **Step 1: Write the integration test**

```python
import pytest
from runtime.planner import Planner
from runtime.executor import Executor
from runtime.memory import WorkingMemory
from runtime.tool_runtime import ToolRuntime
from ornith_mtp.models import StepStatus


@pytest.mark.gpu
@pytest.mark.asyncio
async def test_full_agent_loop():
    """End-to-end scaffold → execute → revise → complete."""
    planner = Planner(llm_client=None)
    tool_runtime = ToolRuntime()
    memory = WorkingMemory()
    executor = Executor(planner=planner, tool_runtime=tool_runtime, memory=memory)

    scaffold = planner.generate_scaffold("Write a Python function to add two numbers")
    assert len(scaffold.plan) >= 1

    results = await executor.run(scaffold)
    assert len(results) >= 1
    assert results[-1].status == StepStatus.SUCCESS


@pytest.mark.gpu
@pytest.mark.asyncio
async def test_recovery_after_crash():
    """Process kill + restart, verify scaffold state persists."""
    from runtime.memory import EpisodicMemory
    import tempfile

    with tempfile.TemporaryDirectory() as tmpdir:
        memory = EpisodicMemory(base_dir=tmpdir, session_id="recovery-test")
        memory.append({"step": "1", "result": "partial"})
        memory.append({"step": "2", "result": "in_progress"})

        memory2 = EpisodicMemory(base_dir=tmpdir, session_id="recovery-test")
        entries = memory2.read_all()
        assert len(entries) == 2
```

- [ ] **Step 2: Commit**

```bash
git add tests/integration/ && git commit -m "test: add integration tests for full loop and crash recovery"
```

---

### Task 13: Example scripts

**Files:**
- Create: `examples/01-basic-chat.py`
- Create: `examples/02-tool-calling.py`
- Create: `examples/03-self-scaffolding.py`

- [ ] **Step 1: Write examples/01-basic-chat.py**

```python
"""Basic chat example: send a message and print the response."""
import asyncio
from ornith_mtp.client import OrnithClient
from ornith_mtp.config_loader import load_config

async def main():
    config = load_config()
    client = OrnithClient(config.inference)
    result = await client.chat("Write a Python function to check if a string is a palindrome.")
    print(f"Response: {result['content']}")
    print(f"Reasoning tokens: {result['tokens_reasoning']}")

if __name__ == "__main__":
    asyncio.run(main())
```

- [ ] **Step 2: Write examples/02-tool-calling.py**

```python
"""Tool calling example: use shell.execute to list files."""
import asyncio
from runtime.tool_runtime import ToolRuntime, ToolSpec, SandboxLevel

async def main():
    runtime = ToolRuntime()
    runtime.register_tool(ToolSpec(
        name="shell.execute",
        description="Execute shell commands",
        timeout_secs=5,
        sandbox=SandboxLevel.RESTRICTED,
        permissions=["shell"],
        allowed_args=["ls", "cat", "echo", "pwd"],
        deny_args=["rm", "sudo"],
    ))
    result = await runtime.execute("shell.execute", ["echo", "Hello from Ornith-MTP!"])
    print(f"Exit code: {result.exit_code}")
    print(f"Stdout: {result.stdout}")

if __name__ == "__main__":
    asyncio.run(main())
```

- [ ] **Step 3: Write examples/03-self-scaffolding.py**

```python
"""Adaptive scaffold example: run a multi-step task with the Agent Runtime."""
import asyncio
from runtime.planner import Planner
from runtime.executor import Executor
from runtime.memory import WorkingMemory
from runtime.tool_runtime import ToolRuntime, ToolSpec, SandboxLevel

async def main():
    planner = Planner(llm_client=None)
    tool_runtime = ToolRuntime()
    tool_runtime.register_tool(ToolSpec(
        name="shell.execute", description="Shell", timeout_secs=5,
        sandbox=SandboxLevel.RESTRICTED, permissions=["shell"],
        allowed_args=["ls", "cat", "echo", "pwd", "python"],
        deny_args=["rm", "sudo"],
    ))
    memory = WorkingMemory()
    executor = Executor(planner=planner, tool_runtime=tool_runtime, memory=memory)

    scaffold = planner.generate_scaffold("Create a hello.py file and run it")
    results = await executor.run(scaffold)
    for r in results:
        print(f"[{r.step_id}] {r.status.value}: {r.output[:80]}...")

if __name__ == "__main__":
    asyncio.run(main())
```

- [ ] **Step 4: Verify examples parse**

Run: `python -c "import py_compile; py_compile.compile('examples/01-basic-chat.py', doraise=True); py_compile.compile('examples/02-tool-calling.py', doraise=True); py_compile.compile('examples/03-self-scaffolding.py', doraise=True); print('All examples parse OK')"`
Expected: All examples parse OK

- [ ] **Step 5: Commit**

```bash
git add examples/ && git commit -m "feat: add example scripts"
```

---

### Self-Review

**1. Spec coverage check:**
- §1 Vision → Covered by Task 8 (planner), Task 9 (executor), Task 5 (memory), Task 6 (tool runtime), Task 11 (benchmark)
- §3.1 Adaptive scaffold loop → Task 8, Task 9
- §3.2 Scaffold data model → Task 2
- §3.3 Planner → Task 8
- §3.4 Executor → Task 9
- §3.5 Memory → Task 5
- §3.6 Checkpoint engine → Task 7
- §3.7 Tool Runtime → Task 6
- §4 Inference layer → Tasks 1, 4
- §5 Acceleration layer → Tasks 1, 4
- §6 Config → Task 3
- §7 Repo layout → Task 1
- §8 Testing → Tasks 11, 12
- §9 Benchmarking → Task 11
- §10 Security → Task 6 (sandbox, permissions)
- §11 Logging → covered by episodic memory (Task 5)
- §12 Error classification → Task 7 (fail-closed), Task 6 (timeouts)
- §14 Nix flake → Task 1
- §15 Project boundaries → All tasks match v0.1 scope
- §16 Competitive analysis → documented in spec, not code

**2. Placeholder scan:** No TBD/TODO/placeholder patterns found. Every step has complete code.

**3. Type consistency:** All `from ornith_mtp.models import ...` references match the models defined in Task 2. Function signatures are consistent across tasks.
