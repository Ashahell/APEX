"""
Unit tests for Python Sandbox.
"""

import pytest
import json
import sys
import os

# Add parent to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from apex_agent.sandbox import (
    PythonSandbox,
    SandboxConfig,
    ExecutionResult,
    ALLOWED_IMPORTS,
)


class TestSandboxConfig:
    """Tests for SandboxConfig."""

    def test_default_config(self):
        config = SandboxConfig()
        assert config.timeout_seconds == 30
        assert config.memory_limit_mb == 512

    def test_custom_config(self):
        config = SandboxConfig(timeout_seconds=10, memory_limit_mb=256)
        assert config.timeout_seconds == 10
        assert config.memory_limit_mb == 256

    def test_validate_valid_config(self):
        config = SandboxConfig(timeout_seconds=10, memory_limit_mb=256)
        issues = config.validate()
        assert len(issues) == 0

    def test_validate_invalid_timeout(self):
        config = SandboxConfig(timeout_seconds=0)
        issues = config.validate()
        assert "timeout_seconds must be positive" in issues

    def test_validate_invalid_memory(self):
        config = SandboxConfig(memory_limit_mb=0)
        issues = config.validate()
        assert "memory_limit_mb must be positive" in issues


class TestPythonSandbox:
    """Tests for PythonSandbox."""

    def test_simple_execution(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print('hello')")

        assert result.success is True
        assert "hello" in result.output

    def test_math_operations(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print(2 + 2)")

        assert result.success is True
        assert "4" in result.output

    def test_parameter_injection(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print(x + y)", {"x": 5, "y": 3})

        assert result.success is True
        assert "8" in result.output

    def test_parameter_dict_access(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print(parameters['x'])", {"x": "hello"})

        assert result.success is True
        assert "hello" in result.output

    def test_string_operations(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print('hello'.upper())")

        assert result.success is True
        assert "HELLO" in result.output

    def test_json_module(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print(json.dumps({'a': 1}))")

        assert result.success is True
        assert '{"a": 1}' in result.output

    def test_regex(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print(len(re.findall(r'\\w+', 'hello world')))")

        assert result.success is True
        assert "2" in result.output

    def test_list_operations(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("print(sorted([3, 1, 2]))")

        assert result.success is True
        assert "1" in result.output

    def test_lambda_functions(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("f = lambda x: x * 2\nprint(f(5))")

        assert result.success is True
        assert "10" in result.output

    def test_main_function(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("def main():\n    return 42\nresult = main()")

        assert result.success is True
        assert "42" in result.output

    def test_timeout(self):
        import time

        sandbox = PythonSandbox()
        result = sandbox.execute("import time\ntime.sleep(10)", timeout_seconds=1)

        assert result.success is False
        assert "timed out" in result.error.lower()

    def test_execution_time_tracked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("x = 1 + 1")

        assert result.execution_time_ms > 0

    def test_unsafe_import_os_blocked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import os\nprint(os.getcwd())")

        assert result.success is False
        assert "dangerous" in result.error.lower() or "not allowed" in result.error.lower()

    def test_unsafe_import_sys_blocked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import sys\nprint(sys.version)")

        assert result.success is False
        assert "dangerous" in result.error.lower() or "not allowed" in result.error.lower()

    def test_unsafe_subprocess_blocked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import subprocess\nsubprocess.run(['ls'])")

        assert result.success is False
        assert "dangerous" in result.error.lower() or "not allowed" in result.error.lower()

    def test_unsafe_open_blocked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("f = open('test.txt')\nf.read()")

        assert result.success is False
        assert "dangerous" in result.error.lower() or "not allowed" in result.error.lower()

    def test_unsafe_exec_blocked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("exec('print(1)')")

        assert result.success is False

    def test_unsafe_eval_blocked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("eval('1 + 1')")

        assert result.success is False

    def test_unsafe_os_system_blocked(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import os\nos.system('ls')")

        assert result.success is False
        assert "dangerous" in result.error.lower() or "not allowed" in result.error.lower()

    def test_allowed_imports(self):
        """Test that allowed imports work."""
        allowed_tests = [
            ("import json\nprint(json.dumps({}))", True),
            ("import math\nprint(math.pi)", True),
            ("import random\nprint(random.randint(1, 10))", True),
            ("import datetime\nprint(datetime.datetime.now())", True),
            ("import re\nprint(re.match(r'\\w+', 'test'))", True),
            ("import collections\nprint(collections.Counter([1,2]))", True),
            ("import itertools\nprint(list(itertools.product([1,2], [3,4])))", True),
            ("import functools\nprint(functools.reduce(lambda x,y: x+y, [1,2,3]))", True),
            ("import textwrap\nprint(textwrap.fill('hello', 10))", True),
            ("import base64\nprint(base64.b64encode(b'test'))", True),
            ("import hashlib\nprint(hashlib.sha256(b'test').hexdigest())", True),
        ]

        sandbox = PythonSandbox()
        for code, should_succeed in allowed_tests:
            result = sandbox.execute(code)
            if should_succeed:
                assert result.success, f"Expected success for: {code[:50]}, got: {result.error}"
            else:
                assert not result.success, f"Expected failure for: {code[:50]}"


class TestExecutionResult:
    """Tests for ExecutionResult."""

    def test_to_dict(self):
        result = ExecutionResult(
            success=True,
            output="test output",
            error=None,
            execution_time_ms=100,
            memory_used_mb=10.5,
        )

        d = result.to_dict()

        assert d["success"] is True
        assert d["output"] == "test output"
        assert d["error"] is None
        assert d["execution_time_ms"] == 100
        assert d["memory_used_mb"] == 10.5

    def test_error_to_dict(self):
        result = ExecutionResult(
            success=False,
            output="",
            error="Test error",
            execution_time_ms=50,
            memory_used_mb=0,
        )

        d = result.to_dict()

        assert d["success"] is False
        assert d["error"] == "Test error"


class TestDangerousPatternDetection:
    """Tests for dangerous pattern detection."""

    def test_import_os_detected(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import os")

        assert result.success is False

    def test_import_sys_detected(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import sys")

        assert result.success is False

    def test_import_subprocess_detected(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import subprocess")

        assert result.success is False

    def test_import_httpx_detected(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import httpx")

        assert result.success is False

    def test_open_detected(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("f = open('test.txt')")

        assert result.success is False

    def test_os_system_detected(self):
        sandbox = PythonSandbox()
        result = sandbox.execute("import os; os.system('ls')")

        assert result.success is False


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
