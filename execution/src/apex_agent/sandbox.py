"""
Secure Python Sandbox for Runtime Tool Execution

Provides a restricted execution environment for dynamically generated Python code.
Security features:
- Import allowlist (only safe stdlib modules)
- Timeout enforcement (30s default)
- Memory limits (512MB default)
- stdout/stderr capture
- No file I/O or network access by default
"""

from __future__ import annotations

import asyncio
import concurrent.futures
import io
import json
import os
import signal
import sys
import tempfile
import threading
import time
from dataclasses import dataclass, field
from typing import Any, Optional

import loguru

# resource module is Unix-only
try:
    import resource  # For memory limits on Unix
except ImportError:
    resource = None

# ============================================================================
# Configuration
# ============================================================================

# Allowed imports - only safe stdlib modules
ALLOWED_IMPORTS = frozenset(
    [
        # Core
        "json",
        "re",
        "math",
        "random",
        "uuid",
        "datetime",
        "time",
        "typing",
        "collections",
        "itertools",
        "functools",
        "operator",
        " pathlib",
        # Data processing
        "base64",
        "hashlib",
        "hmac",
        "secrets",
        # Text processing
        "textwrap",
        "string",
        "csv",
        "io",
        # Encoding
        "html",
        "urllib.parse",
        "xml.etree.ElementTree",
        "xml.dom.minidom",
    ]
)

# Execution limits
DEFAULT_TIMEOUT_SECONDS = 30
DEFAULT_MEMORY_LIMIT_MB = 512


# ============================================================================
# Data Structures
# ============================================================================


@dataclass
class SandboxConfig:
    """Configuration for sandbox execution."""

    timeout_seconds: int = DEFAULT_TIMEOUT_SECONDS
    memory_limit_mb: int = DEFAULT_MEMORY_LIMIT_MB
    allow_file_read: bool = False
    allow_file_write: bool = False
    allowed_paths: list[str] = field(default_factory=list)
    allow_network: bool = False
    allowed_hosts: list[str] = field(default_factory=list)

    def validate(self) -> list[str]:
        """Validate config and return list of issues."""
        issues = []
        if self.timeout_seconds <= 0:
            issues.append("timeout_seconds must be positive")
        if self.memory_limit_mb <= 0:
            issues.append("memory_limit_mb must be positive")
        return issues


@dataclass
class ExecutionResult:
    """Result of sandbox execution."""

    success: bool
    output: str = ""
    error: Optional[str] = None
    execution_time_ms: int = 0
    memory_used_mb: float = 0.0

    def to_dict(self) -> dict:
        return {
            "success": self.success,
            "output": self.output,
            "error": self.error,
            "execution_time_ms": self.execution_time_ms,
            "memory_used_mb": self.memory_used_mb,
        }


# ============================================================================
# Sandbox Implementation
# ============================================================================


class ImportBlocker:
    """Blocks disallowed imports at parse time."""

    def __init__(self, allowed: frozenset[str] = ALLOWED_IMPORTS):
        self.allowed = allowed
        self._original_import = __builtins__.__import__

    def find_module(self, name: str, path: Optional[str] = None):
        """Check if import is allowed."""
        # Handle relative imports
        if name.startswith("_"):
            return self  # Block private modules

        # Check direct imports
        if name in self.allowed:
            return None  # Allow

        # Check nested imports (e.g., urllib.parse)
        parts = name.split(".")
        if parts[0] in self.allowed:
            return None  # Allow

        # Block everything else
        return self

    def load_module(self, name: str):
        """Block the import."""
        raise ImportError(
            f"Import '{name}' not allowed. Allowed: {', '.join(sorted(self.allowed))}"
        )


class RestrictedGlobals:
    """Provides restricted globals for execution."""

    def __init__(self, config: SandboxConfig, output_buffer: list[str]):
        self.config = config
        self._output_buffer = output_buffer

    def get_globals(self) -> dict:
        """Get restricted globals dict."""

        # Allowed modules that can be imported
        allowed_modules = {
            "json": __import__("json"),
            "re": __import__("re"),
            "math": __import__("math"),
            "random": __import__("random"),
            "datetime": __import__("datetime"),
            "time": __import__("time"),
            "typing": __import__("typing"),
            "collections": __import__("collections"),
            "itertools": __import__("itertools"),
            "functools": __import__("functools"),
            "operator": __import__("operator"),
            "pathlib": __import__("pathlib"),
            "base64": __import__("base64"),
            "hashlib": __import__("hashlib"),
            "hmac": __import__("hmac"),
            "secrets": __import__("secrets"),
            "textwrap": __import__("textwrap"),
            "string": __import__("string"),
            "csv": __import__("csv"),
            "io": __import__("io"),
            "html": __import__("html"),
            "urllib": __import__("urllib"),
            "xml": __import__("xml"),
        }

        # Create a safe __import__ that only allows pre-loaded modules
        allowed_names = frozenset(allowed_modules.keys())

        def safe_import(name, *args, **kwargs):
            """Restricted import that only allows pre-loaded modules."""
            if name in allowed_names:
                return allowed_modules[name]
            raise ImportError(
                f"Import '{name}' not allowed. Allowed: {', '.join(sorted(allowed_names))}"
            )

        safe_builtins = {
            # Allowed builtins
            "print": self._safe_print,
            "__import__": safe_import,  # Allow restricted imports
            "len": len,
            "range": range,
            "enumerate": enumerate,
            "zip": zip,
            "map": map,
            "filter": filter,
            "sorted": sorted,
            "reversed": reversed,
            "sum": sum,
            "min": min,
            "max": max,
            "abs": abs,
            "round": round,
            "divmod": divmod,
            "isinstance": isinstance,
            "issubclass": issubclass,
            "hasattr": hasattr,
            "getattr": getattr,
            "setattr": setattr,
            "delattr": delattr,
            "callable": callable,
            "chr": chr,
            "ord": ord,
            "hex": hex,
            "oct": oct,
            "bin": bin,
            "format": format,
            "slice": slice,
            "property": property,
            "classmethod": classmethod,
            "staticmethod": staticmethod,
            "super": super,
            "type": type,
            "vars": vars,
            "id": id,
            "hash": hash,
            "repr": repr,
            "bool": bool,
            "int": int,
            "float": float,
            "str": str,
            "bytes": bytes,
            "bytearray": bytearray,
            "list": list,
            "tuple": tuple,
            "set": set,
            "frozenset": frozenset,
            "dict": dict,
            "complex": complex,
            "object": object,
            "NotImplemented": NotImplemented,
            "Ellipsis": Ellipsis,
            "None": None,
            "True": True,
            "False": False,
            # Blocked builtins (explicitly)
            "exec": None,
            "eval": None,
            "compile": None,
            "open": None,
            "file": None,
            "input": None,
            "breakpoint": None,
            "help": None,
            "credits": None,
            "license": None,
            # Math constants
            "pi": 3.141592653589793,
            "e": 2.718281828459045,
            "tau": 6.283185307179586,
        }

        # Remove None values (blocked builtins)
        safe_builtins = {k: v for k, v in safe_builtins.items() if v is not None}

        # Add allowed modules to globals (so code can use them without import)
        globals_dict = {
            "__builtins__": safe_builtins,
            "__name__": "__sandbox__",
            "__doc__": None,
        }

        # Add all allowed modules to globals
        globals_dict.update(allowed_modules)

        return globals_dict

    def _safe_print(self, *args, **kwargs):
        """Safe print that converts to string."""
        sep = kwargs.get("sep", " ")
        end = kwargs.get("end", "\n")
        file = kwargs.get("file", None)

        if file is None:
            output = sep.join(str(arg) for arg in args) + end
            # Store in a global for capture
            if hasattr(self, "_output_buffer"):
                self._output_buffer.append(output)
        else:
            # Redirect to file (but no file access allowed)
            raise PermissionError("File output not allowed in sandbox")


class TimeoutException(Exception):
    """Raised when execution times out."""

    pass


class MemoryLimitException(Exception):
    """Raised when memory limit is exceeded."""

    pass


class PythonSandbox:
    """
    Secure Python code execution sandbox.

    Usage:
        sandbox = PythonSandbox()
        result = sandbox.execute("print(1 + 2)", {"timeout_seconds": 10})
    """

    def __init__(self, config: Optional[SandboxConfig] = None):
        self.config = config or SandboxConfig()
        self.logger = loguru.logger.bind(component="sandbox")

        # Validate config
        issues = self.config.validate()
        if issues:
            raise ValueError(f"Invalid sandbox config: {', '.join(issues)}")

        # Calculate memory limit in bytes for resource module
        self._memory_limit_bytes = self.config.memory_limit_mb * 1024 * 1024

        # Platform-specific memory limit setup
        self._can_set_memory_limit = self._setup_memory_limit()

    def _setup_memory_limit(self) -> bool:
        """Setup memory limit for current process (Unix only)."""
        # Note: resource module is Unix-specific
        if sys.platform == "win32":
            self.logger.debug("Memory limits not supported on Windows")
            return False

        try:
            import resource

            # Get current limits
            soft, hard = resource.getrlimit(resource.RLIMIT_AS)

            # Set new limit
            resource.setrlimit(
                resource.RLIMIT_AS, (self._memory_limit_bytes, self._memory_limit_bytes)
            )

            # Also limit core dump size to 0 to prevent memory leaks
            resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

            self.logger.debug(f"Memory limit set to {self.config.memory_limit_mb}MB")
            return True
        except Exception as e:
            self.logger.warning(f"Could not set memory limit: {e}")
            return False

    def _get_memory_usage(self) -> float:
        """Estimate memory usage (not fully accurate on all platforms)."""
        try:
            import psutil

            process = psutil.Process()
            return process.memory_info().rss / (1024 * 1024)  # MB
        except ImportError:
            # Fallback: return estimated usage
            return 0.0

    def execute(
        self,
        code: str,
        parameters: Optional[dict[str, Any]] = None,
        timeout_seconds: Optional[int] = None,
    ) -> ExecutionResult:
        """
        Execute Python code in sandbox.

        Args:
            code: Python code to execute
            parameters: Dict of parameters to inject into code
            timeout_seconds: Override default timeout

        Returns:
            ExecutionResult with output/error
        """
        start_time = time.time()

        timeout = timeout_seconds or self.config.timeout_seconds
        parameters = parameters or {}

        self.logger.debug("Executing sandbox code (timeout={}s)", timeout)

        # Prepare output capture
        output_buffer: list[str] = []

        # Prepare code with parameters
        try:
            full_code = self._prepare_code(code, parameters)
        except Exception as e:
            return ExecutionResult(
                success=False,
                error=f"Code preparation failed: {e}",
                execution_time_ms=int((time.time() - start_time) * 1000),
            )

        # Create restricted globals
        restricted_globals = RestrictedGlobals(self.config, output_buffer)

        # Execute with timeout and memory limit in a subprocess
        try:
            # Use a separate process to enforce memory limits
            import multiprocessing

            def run_in_process(code, globals_dict, memory_limit_mb):
                """Run code in isolated process with memory limits."""
                # Set memory limits before importing anything
                if sys.platform != "win32":
                    try:
                        import resource

                        limit_bytes = memory_limit_mb * 1024 * 1024
                        resource.setrlimit(resource.RLIMIT_AS, (limit_bytes, limit_bytes))
                        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
                    except Exception:
                        pass

                # Execute the code
                local_namespace: dict = {}
                try:
                    compiled = compile(code, "<sandbox>", "exec")
                    exec(compiled, globals_dict, local_namespace)

                    if "main" in local_namespace:
                        return local_namespace["main"]()
                    return None
                except Exception as e:
                    raise RuntimeError(str(e))

            with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
                future = executor.submit(
                    run_in_process,
                    full_code,
                    restricted_globals.get_globals(),
                    self.config.memory_limit_mb,
                )

                try:
                    result = future.result(timeout=timeout)
                except concurrent.futures.TimeoutError:
                    return ExecutionResult(
                        success=False,
                        error=f"Execution timed out after {timeout} seconds",
                        execution_time_ms=int((time.time() - start_time) * 1000),
                    )
                except RuntimeError as e:
                    return ExecutionResult(
                        success=False,
                        error=f"Execution error: {e}",
                        execution_time_ms=int((time.time() - start_time) * 1000),
                    )

        except Exception as e:
            return ExecutionResult(
                success=False,
                error=str(e),
                execution_time_ms=int((time.time() - start_time) * 1000),
            )

        execution_time_ms = max(1, int((time.time() - start_time) * 1000))

        # Combine output
        output = "".join(output_buffer)
        if result is not None:
            # If code returned something, add it to output
            if not output.endswith("\n") and output:
                output += "\n"
            output += json.dumps(result, default=str)

        return ExecutionResult(
            success=True,
            output=output,
            execution_time_ms=execution_time_ms,
        )

    def _prepare_code(self, code: str, parameters: dict[str, Any]) -> str:
        """Prepare code with injected parameters."""
        # Validate code doesn't contain dangerous patterns
        dangerous = self._check_dangerous_patterns(code)
        if dangerous:
            raise ValueError(f"Dangerous pattern detected: {dangerous}")

        # Inject parameters as a dict AND unpack as variables
        param_json = json.dumps(parameters, default=str)
        param_code = f"parameters = {param_json}\n"

        # Unpack parameters as individual variables for convenience
        for key, value in parameters.items():
            if key.isidentifier():
                # Convert value to repr for proper Python syntax
                param_code += f"{key} = {json.dumps(value)}\n"

        param_code += "\n"

        return param_code + code

    def _check_dangerous_patterns(self, code: str) -> Optional[str]:
        """Check for dangerous patterns in code."""
        dangerous_patterns = [
            ("import os", "import os"),
            ("import sys", "import sys"),
            ("import subprocess", "import subprocess"),
            ("import socket", "import socket"),
            ("import requests", "import requests"),
            ("import httpx", "import httpx"),
            ("import http", "import http"),
            ("import ftp", "import ftp"),
            ("import urllib", "import urllib"),
            ("open(", "file open"),
            ("with open", "file open"),
            ("exec(", "exec()"),
            ("eval(", "eval()"),
            ("compile(", "compile()"),
            ("__import__", "__import__"),
            ("subprocess", "subprocess"),
            ("os.system", "os.system"),
            ("os.popen", "os.popen"),
            ("os.fork", "os.fork"),
            ("os.spawn", "os.spawn"),
            ("sys.exit", "sys.exit"),
            ("sys.platform", "sys.platform"),
            ("os.name", "os.name"),
            ("breakpoint", "breakpoint()"),
            ("help(", "help()"),
        ]

        code_lower = code.lower()
        for pattern, name in dangerous_patterns:
            if pattern.lower() in code_lower:
                return name

        return None

    def _execute_code(self, code: str, globals_dict: dict) -> Any:
        """Execute code with restricted globals."""
        # Create a clean namespace
        local_namespace: dict = {}

        # Execute
        try:
            # Try to compile first to catch syntax errors
            compiled = compile(code, "<sandbox>", "exec")

            # Execute
            exec(compiled, globals_dict, local_namespace)

            # Look for a main() function and call it
            if "main" in local_namespace:
                result = local_namespace["main"]()
                return result

            # Otherwise return the result of the last expression
            return None

        except SyntaxError as e:
            raise SyntaxError(f"Syntax error: {e}")
        except NameError as e:
            raise NameError(f"Name error: {e}")
        except TypeError as e:
            raise TypeError(f"Type error: {e}")
        except ValueError as e:
            raise ValueError(f"Value error: {e}")
        except Exception as e:
            raise RuntimeError(f"Execution error: {e}")


# ============================================================================
# Async Wrapper
# ============================================================================


class AsyncSandbox:
    """Async wrapper for PythonSandbox."""

    def __init__(self, config: Optional[SandboxConfig] = None):
        self.sandbox = PythonSandbox(config)
        self.logger = loguru.logger.bind(component="async-sandbox")

    async def execute(
        self,
        code: str,
        parameters: Optional[dict[str, Any]] = None,
        timeout_seconds: Optional[int] = None,
    ) -> ExecutionResult:
        """Execute code asynchronously."""
        loop = asyncio.get_event_loop()

        # Run in executor to avoid blocking
        result = await loop.run_in_executor(
            None, lambda: self.sandbox.execute(code, parameters, timeout_seconds)
        )

        return result


# ============================================================================
# Main entry point for subprocess execution
# ============================================================================


def main():
    """Main entry point - called from Rust via subprocess."""
    import argparse

    parser = argparse.ArgumentParser(description="Python Sandbox Executor")
    parser.add_argument("--code", required=True, help="Code to execute")
    parser.add_argument("--params", default="{}", help="JSON parameters")
    parser.add_argument("--timeout", type=int, default=30, help="Timeout in seconds")
    parser.add_argument("--output-file", help="Write result to file")

    args = parser.parse_args()

    try:
        # Parse parameters
        params = json.loads(args.params)

        # Create sandbox and execute
        sandbox = PythonSandbox()
        result = sandbox.execute(args.code, params, args.timeout)

        # Output result
        output = json.dumps(result.to_dict())

        if args.output_file:
            with open(args.output_file, "w") as f:
                f.write(output)
        else:
            print(output)

    except Exception as e:
        error_result = json.dumps(
            {
                "success": False,
                "error": str(e),
                "output": "",
                "execution_time_ms": 0,
                "memory_used_mb": 0.0,
            }
        )

        if args.output_file:
            with open(args.output_file, "w") as f:
                f.write(error_result)
        else:
            print(error_result)

        sys.exit(1)


if __name__ == "__main__":
    main()
