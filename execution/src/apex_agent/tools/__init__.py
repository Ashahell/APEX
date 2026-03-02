"""
Tool definitions for the execution engine.
"""

from collections.abc import Callable
from typing import Any


class Tool:
    """Base class for agent tools."""

    def __init__(
        self,
        name: str,
        description: str,
        func: Callable[..., Any],
        parameters: dict[str, Any],
    ):
        self.name = name
        self.description = description
        self.func = func
        self.parameters = parameters

    async def execute(self, **kwargs: Any) -> Any:
        """Execute the tool with given parameters."""
        return await self.func(**kwargs)


# Example tools
class FileTool(Tool):
    """Tool for file operations."""

    def __init__(self) -> None:
        super().__init__(
            name="file",
            description="Read or write files",
            func=self._execute,
            parameters={
                "operation": {"type": "string", "enum": ["read", "write"]},
                "path": {"type": "string"},
                "content": {"type": "string"},
            },
        )

    async def _execute(self, operation: str, path: str, content: str = "") -> str:
        if operation == "read":
            # Would read from filesystem
            return f"Content of {path}"
        elif operation == "write":
            # Would write to filesystem
            return f"Wrote to {path}"
        return "Unknown operation"


class ShellTool(Tool):
    """Tool for shell command execution."""

    def __init__(self) -> None:
        super().__init__(
            name="shell",
            description="Execute shell commands",
            func=self._execute,
            parameters={
                "command": {"type": "string"},
                "timeout": {"type": "number", "default": 30},
            },
        )

    async def _execute(self, command: str, timeout: int = 30) -> str:
        # Would execute in sandboxed environment
        return f"Executed: {command}"


# Registry of available tools
TOOLS: dict[str, Tool] = {
    "file": FileTool(),
    "shell": ShellTool(),
}


def get_tool(name: str) -> Tool | None:
    """Get a tool by name."""
    return TOOLS.get(name)


def list_tools() -> list[Tool]:
    """List all available tools."""
    return list(TOOLS.values())
