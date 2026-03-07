"""
Dynamic Tool Generation - AgentZero Core Feature

This module allows the agent to generate and execute custom tools at runtime.
"""

from __future__ import annotations
import asyncio
import uuid
import tempfile
import os
from dataclasses import dataclass, field
from typing import Any, Optional, Callable
from enum import Enum

import loguru
import httpx


class ToolLanguage(str, Enum):
    PYTHON = "python"
    BASH = "bash"


@dataclass
class GeneratedTool:
    """A dynamically generated tool."""

    id: str
    name: str
    description: str
    code: str
    language: ToolLanguage
    input_schema: dict
    output_schema: dict
    created_at: float


@dataclass
class ToolGenerationRequest:
    """Request to generate a tool."""

    task_context: str
    required_capability: str
    input_schema: dict
    output_schema: dict


@dataclass
class ToolGenerationResult:
    """Result of tool generation."""

    success: bool
    tool: Optional[GeneratedTool] = None
    error: Optional[str] = None


class ToolGenerator:
    """
    Generates tools at runtime based on task requirements.
    This is AgentZero's core distinctive feature.
    """

    SYSTEM_PROMPT = """You are a tool generator. Create a tool that solves the described problem.

Requirements:
1. The tool must be self-contained and executable
2. Follow best practices for the language
3. Include error handling
4. Return results as JSON

Response format:
{
  "name": "tool_name",
  "description": "what it does",
  "code": "the executable code",
  "language": "python" or "bash"
}"""

    def __init__(self, llm_url: str = "http://localhost:8080"):
        self.llm_url = llm_url
        self.logger = loguru.logger.bind(component="tool-generator")
        self.generated_tools: dict[str, GeneratedTool] = {}

    async def generate_tool(self, request: ToolGenerationRequest) -> ToolGenerationResult:
        """Generate a tool based on the request."""
        self.logger.info("Generating tool for: {}", request.required_capability)

        prompt = f"""{self.SYSTEM_PROMPT}

Task: {request.task_context}
Required capability: {request.required_capability}
Input schema: {request.input_schema}
Output schema: {request.output_schema}

Generate the tool now:"""

        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    f"{self.llm_url}/v1/chat/completions",
                    json={
                        "model": "qwen3-4b",
                        "messages": [
                            {"role": "system", "content": self.SYSTEM_PROMPT},
                            {"role": "user", "content": prompt},
                        ],
                        "max_tokens": 2048,
                        "temperature": 0.3,
                    },
                )
            response.raise_for_status()
            data = response.json()
            content = data["choices"][0]["message"]["content"]

            tool_data = self._parse_tool_response(content)
            if not tool_data:
                return ToolGenerationResult(
                    success=False, error="Failed to parse tool from LLM response"
                )

            tool = GeneratedTool(
                id=str(uuid.uuid4()),
                name=tool_data["name"],
                description=tool_data["description"],
                code=tool_data["code"],
                language=ToolLanguage(tool_data.get("language", "python")),
                input_schema=request.input_schema,
                output_schema=request.output_schema,
                created_at=asyncio.get_event_loop().time(),
            )

            self.generated_tools[tool.name] = tool
            self.logger.info("Generated tool: {} ({})", tool.name, tool.language)

            return ToolGenerationResult(success=True, tool=tool)

        except Exception as e:
            self.logger.error("Tool generation failed: {}", e)
            return ToolGenerationResult(success=False, error=str(e))

    def _parse_tool_response(self, content: str) -> Optional[dict]:
        """Parse the LLM response to extract tool data."""
        import json

        try:
            if "```json" in content:
                start = content.find("```json") + 7
                end = content.find("```", start)
                content = content[start:end].strip()
            elif "```" in content:
                start = content.find("```") + 3
                end = content.find("```", start)
                lang_end = content.find("\n", start)
                content = content[lang_end + 1 : end].strip()

            return json.loads(content)
        except Exception as e:
            self.logger.warning("Failed to parse tool response: {}", e)
            return None

    def get_tool(self, name: str) -> Optional[GeneratedTool]:
        """Get a previously generated tool."""
        return self.generated_tools.get(name)

    def list_tools(self) -> list[GeneratedTool]:
        """List all generated tools."""
        return list(self.generated_tools.values())

    def execute_tool(self, tool: GeneratedTool, input_data: dict) -> dict:
        """Execute a generated tool."""
        self.logger.info("Executing generated tool: {}", tool.name)

        try:
            if tool.language == ToolLanguage.PYTHON:
                return self._execute_python(tool, input_data)
            elif tool.language == ToolLanguage.BASH:
                return self._execute_bash(tool, input_data)
            else:
                return {"success": False, "error": f"Unknown language: {tool.language}"}
        except Exception as e:
            self.logger.error("Tool execution failed: {}", e)
            return {"success": False, "error": str(e)}

    def _execute_python(self, tool: GeneratedTool, input_data: dict) -> dict:
        """Execute Python tool."""
        import subprocess

        with tempfile.NamedTemporaryFile(mode="w", suffix=".py", delete=False) as f:
            f.write(tool.code)
            f.write(f"\n\n# Input: {input_data}\n")
            f.write(f"print(json.dumps({input_data.get('main_call', 'main(input_data)')}))")
            temp_file = f.name

        try:
            result = subprocess.run(
                ["python", temp_file],
                capture_output=True,
                text=True,
                timeout=30,
            )

            if result.returncode == 0:
                return {"success": True, "output": result.stdout}
            else:
                return {"success": False, "error": result.stderr}
        finally:
            os.unlink(temp_file)

    def _execute_bash(self, tool: GeneratedTool, input_data: dict) -> dict:
        """Execute Bash tool."""
        import subprocess

        with tempfile.NamedTemporaryFile(mode="w", suffix=".sh", delete=False) as f:
            f.write(tool.code)
            temp_file = f.name

        try:
            result = subprocess.run(
                ["bash", temp_file],
                capture_output=True,
                text=True,
                timeout=30,
                env={**os.environ, "INPUT_DATA": str(input_data)},
            )

            if result.returncode == 0:
                return {"success": True, "output": result.stdout}
            else:
                return {"success": False, "error": result.stderr}
        finally:
            os.unlink(temp_file)


class DynamicToolExecutor:
    """Mixes static and dynamically generated tools."""

    def __init__(self, llm_url: str = "http://localhost:8080"):
        self.llm_url = llm_url
        self.static_tools = self._register_static_tools()
        self.dynamic_tools: dict[str, GeneratedTool] = {}
        self.generator = ToolGenerator(llm_url)

    def _register_static_tools(self) -> dict[str, Callable]:
        """Register built-in static tools."""
        return {
            "code.generate": self._tool_code_generate,
            "code.review": self._tool_code_review,
            "shell.execute": self._tool_shell_execute,
            "file.read": self._tool_file_read,
            "file.write": self._tool_file_write,
        }

    async def execute(self, tool_name: str, input_data: dict) -> dict:
        """Execute a tool (static or dynamic)."""

        if tool_name in self.static_tools:
            return await self.static_tools[tool_name](input_data)

        if tool_name in self.dynamic_tools:
            tool = self.dynamic_tools[tool_name]
            return self.generator.execute_tool(tool, input_data)

        return {"success": False, "error": f"Unknown tool: {tool_name}"}

    async def generate_and_execute(
        self,
        capability: str,
        task_context: str,
        input_schema: dict,
        output_schema: dict,
        input_data: dict,
    ) -> dict:
        """Generate a tool and execute it."""

        request = ToolGenerationRequest(
            task_context=task_context,
            required_capability=capability,
            input_schema=input_schema,
            output_schema=output_schema,
        )

        result = await self.generator.generate_tool(request)

        if result.success and result.tool:
            self.dynamic_tools[result.tool.name] = result.tool
            return self.generator.execute_tool(result.tool, input_data)

        return {"success": False, "error": result.error or "Generation failed"}

    async def _tool_code_generate(self, input_data: dict) -> dict:
        """Generate code using LLM."""
        language = input_data.get("language", "python")
        description = input_data.get("description", "")

        if not description:
            return {"success": False, "error": "No description provided"}

        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    f"{self.llm_url}/v1/chat/completions",
                    json={
                        "model": "qwen3-4b",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a code generator. Output only code, no explanations.",
                            },
                            {
                                "role": "user",
                                "content": f"Generate {language} code for: {description}",
                            },
                        ],
                        "max_tokens": 1024,
                        "temperature": 0.3,
                    },
                )
            response.raise_for_status()
            code = response.json()["choices"][0]["message"]["content"]
            return {"success": True, "output": code}
        except Exception as e:
            return {"success": False, "error": str(e)}

    async def _tool_code_review(self, input_data: dict) -> dict:
        """Review code for issues."""
        code = input_data.get("code", "")

        if not code:
            return {"success": False, "error": "No code provided"}

        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                response = await client.post(
                    f"{self.llm_url}/v1/chat/completions",
                    json={
                        "model": "qwen3-4b",
                        "messages": [
                            {
                                "role": "system",
                                "content": "You are a code reviewer. Provide brief, actionable feedback.",
                            },
                            {"role": "user", "content": f"Review this code:\n\n{code[:2000]}"},
                        ],
                        "max_tokens": 512,
                        "temperature": 0.3,
                    },
                )
            response.raise_for_status()
            review = response.json()["choices"][0]["message"]["content"]
            return {"success": True, "output": review}
        except Exception as e:
            return {"success": False, "error": str(e)}

    async def _tool_shell_execute(self, input_data: dict) -> dict:
        return {"success": True, "output": "Shell execution requires T3 verification in APEX"}

    async def _tool_file_read(self, input_data: dict) -> dict:
        """Read a file."""
        import os

        path = input_data.get("path", "")
        if not path:
            return {"success": False, "error": "No path provided"}

        try:
            with open(path, "r", encoding="utf-8") as f:
                content = f.read()
            return {"success": True, "output": content[:5000]}
        except FileNotFoundError:
            return {"success": False, "error": f"File not found: {path}"}
        except Exception as e:
            return {"success": False, "error": str(e)}

    async def _tool_file_write(self, input_data: dict) -> dict:
        """Write to a file."""
        import os

        path = input_data.get("path", "")
        content = input_data.get("content", "")

        if not path:
            return {"success": False, "error": "No path provided"}

        try:
            parent_dir = os.path.dirname(path)
            if parent_dir and not os.path.exists(parent_dir):
                os.makedirs(parent_dir, exist_ok=True)

            with open(path, "w", encoding="utf-8") as f:
                f.write(content)
            return {"success": True, "output": f"Wrote {len(content)} bytes to {path}"}
        except Exception as e:
            return {"success": False, "error": str(e)}


if __name__ == "__main__":

    async def test():
        generator = ToolGenerator()

        request = ToolGenerationRequest(
            task_context="Count lines in a file",
            required_capability="count_lines",
            input_schema={"type": "object", "properties": {"path": {"type": "string"}}},
            output_schema={"type": "object", "properties": {"count": {"type": "integer"}}},
        )

        result = await generator.generate_tool(request)
        print(f"Success: {result.success}")
        if result.tool:
            print(f"Tool: {result.tool.name}")
            print(f"Code: {result.tool.code[:100]}...")

    asyncio.run(test())
