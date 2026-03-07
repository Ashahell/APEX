"""
Tool-Integrated Reasoning (TIR) - AgentZero Core Feature

TIR interleaves reasoning and tool execution in a single LLM call,
allowing the agent to "think while doing" rather than think-then-do.
"""

from __future__ import annotations
import asyncio
import json
from dataclasses import dataclass, field
from typing import AsyncIterator, Optional
from enum import Enum
import re

import loguru
import httpx


class ThoughtType(str, Enum):
    THINK = "Thought"
    ACTION = "Action"
    OBSERVATION = "Observation"
    FINAL = "Final"


@dataclass
class ThoughtStep:
    """A single step in TIR reasoning."""

    step: int
    thought_type: ThoughtType
    content: str
    tool: Optional[str] = None
    tool_input: Optional[dict] = None
    observation: Optional[str] = None


@dataclass
class TIRConfig:
    """Configuration for TIR execution."""

    llm_url: str = "http://localhost:8080"
    llm_model: str = "qwen3-4b"
    max_steps: int = 50
    timeout_seconds: int = 300


class TIRExecutor:
    """
    Tool-Integrated Reasoning executor.

    Unlike traditional ReAct (Reason + Act), TIR allows interleaved
    thought and action in a single response from the LLM.
    """

    TIR_SYSTEM_PROMPT = """You are an AI assistant using Tool-Integrated Reasoning (TIR).

In TIR, you interleave your thinking with actions in a continuous stream.

Format your response as a JSON array of steps:
[
  {"type": "Thought", "content": "Your reasoning here"},
  {"type": "Action", "tool": "tool_name", "input": {"key": "value"}},
  {"type": "Observation", "content": "Result of the action"},
  {"type": "Final", "content": "Your final answer"}
]

Available tools:
- read_file(path): Read file contents
- write_file(path, content): Write to a file  
- bash(command): Execute shell command
- search(query): Search the web
- think(content): Just think (no action)

IMPORTANT:
- Use "Observation" after every "Action" to show what happened
- Continue until you have a final answer
- Keep each step focused and concise
- Your response must be valid JSON

Task: {task}

Now respond with your TIR steps:"""

    def __init__(self, config: TIRConfig):
        self.config = config
        self.logger = loguru.logger.bind(component="tir")
        self.tools = self._default_tools()

    def _default_tools(self) -> dict:
        """Default tool implementations."""
        return {
            "read_file": self._tool_read_file,
            "write_file": self._tool_write_file,
            "bash": self._tool_bash,
            "search": self._tool_search,
            "think": self._tool_think,
        }

    def register_tool(self, name: str, func):
        """Register a custom tool."""
        self.tools[name] = func

    async def execute(self, task: str) -> AsyncIterator[ThoughtStep]:
        """Execute TIR and yield steps as they happen."""
        self.logger.info("Starting TIR execution for: {}", task[:50])

        prompt = self.TIR_SYSTEM_PROMPT.format(task=task)

        for step_num in range(self.config.max_steps):
            try:
                async with httpx.AsyncClient(timeout=self.config.timeout_seconds) as client:
                    response = await client.post(
                        f"{self.config.llm_url}/v1/chat/completions",
                        json={
                            "model": self.config.llm_model,
                            "messages": [{"role": "user", "content": prompt}],
                            "max_tokens": 2048,
                            "temperature": 0.7,
                        },
                    )
                response.raise_for_status()
                data = response.json()
                content = data["choices"][0]["message"]["content"]

                steps = self._parse_tir_response(content)

                for step in steps:
                    yield step

                    if step.thought_type == ThoughtType.FINAL:
                        return

                    if step.thought_type == ThoughtType.ACTION and step.tool:
                        observation = await self._execute_tool(step.tool, step.tool_input or {})
                        yield ThoughtStep(
                            step=step.step,
                            thought_type=ThoughtType.OBSERVATION,
                            content=observation,
                        )

            except Exception as e:
                self.logger.error("TIR step failed: {}", e)
                yield ThoughtStep(
                    step=step_num, thought_type=ThoughtType.THINK, content=f"Error: {str(e)}"
                )

    def _parse_tir_response(self, content: str) -> list[ThoughtStep]:
        """Parse TIR response into structured steps."""
        steps = []

        try:
            if "```json" in content:
                start = content.find("```json") + 7
                end = content.find("```", start)
                content = content[start:end].strip()
            elif "```" in content:
                start = content.find("```") + 3
                end = content.find("```", start)
                content = content[start:end].strip()

            parsed = json.loads(content)

            for i, item in enumerate(parsed):
                step_type = ThoughtType(item.get("type", "Thought"))
                steps.append(
                    ThoughtStep(
                        step=i,
                        thought_type=step_type,
                        content=item.get("content", ""),
                        tool=item.get("tool"),
                        tool_input=item.get("input"),
                    )
                )

        except json.JSONDecodeError:
            content_lines = content.split("\n")
            current_type = ThoughtType.THINK
            current_content = ""
            step_num = 0

            for line in content_lines:
                line = line.strip()
                if not line:
                    continue

                if line.startswith("Thought:") or line.startswith("Thinking:"):
                    if current_content:
                        steps.append(ThoughtStep(step_num, current_type, current_content))
                        step_num += 1
                    current_type = ThoughtType.THINK
                    current_content = line.split(":", 1)[1].strip()
                elif line.startswith("Action:") or line.startswith("Using:"):
                    if current_content:
                        steps.append(ThoughtStep(step_num, current_type, current_content))
                        step_num += 1
                    tool_part = line.split(":", 1)[1].strip()
                    parts = tool_part.split("(", 1)
                    tool = parts[0].strip()
                    tool_input = {}
                    if len(parts) > 1 and parts[1].endswith(")"):
                        try:
                            tool_input = json.loads(parts[1][:-1])
                        except:
                            pass
                    current_type = ThoughtType.ACTION
                    current_content = ""
                    steps.append(ThoughtStep(step_num, current_type, tool_part, tool, tool_input))
                    step_num += 1
                    current_type = ThoughtType.THINK
                elif line.startswith("Observation:") or line.startswith("Result:"):
                    if current_content:
                        steps.append(ThoughtStep(step_num, current_type, current_content))
                        step_num += 1
                    current_type = ThoughtType.OBSERVATION
                    current_content = line.split(":", 1)[1].strip()
                elif line.startswith("Final:") or line.startswith("Answer:"):
                    if current_content:
                        steps.append(ThoughtStep(step_num, current_type, current_content))
                        step_num += 1
                    current_type = ThoughtType.FINAL
                    current_content = line.split(":", 1)[1].strip()
                    steps.append(ThoughtStep(step_num, current_type, current_content))
                    return steps
                else:
                    current_content += " " + line

            if current_content:
                steps.append(ThoughtStep(step_num, current_type, current_content))

        return steps

    async def _execute_tool(self, tool_name: str, input_data: dict) -> str:
        """Execute a tool and return the observation."""
        if tool_name not in self.tools:
            return f"Error: Unknown tool '{tool_name}'"

        try:
            result = await self.tools[tool_name](input_data)
            return str(result)
        except Exception as e:
            return f"Error executing {tool_name}: {e}"

    async def _tool_read_file(self, input_data: dict) -> str:
        path = input_data.get("path", "")
        try:
            with open(path, "r") as f:
                return f.read()[:1000]
        except Exception as e:
            return f"Error reading {path}: {e}"

    async def _tool_write_file(self, input_data: dict) -> str:
        path = input_data.get("path", "")
        content = input_data.get("content", "")
        try:
            with open(path, "w") as f:
                f.write(content)
            return f"Wrote {len(content)} bytes to {path}"
        except Exception as e:
            return f"Error writing {path}: {e}"

    async def _tool_bash(self, input_data: dict) -> str:
        import subprocess

        cmd = input_data.get("command", "")
        try:
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30)
            return result.stdout[:500] or result.stderr[:500]
        except Exception as e:
            return f"Error: {e}"

    async def _tool_search(self, input_data: dict) -> str:
        query = input_data.get("query", "")
        if not query:
            return "Error: No query provided"

        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                response = await client.get(
                    "https://html.duckduckgo.com/html/",
                    params={"q": query},
                    headers={"User-Agent": "Mozilla/5.0"},
                )
            response.raise_for_status()

            from bs4 import BeautifulSoup

            soup = BeautifulSoup(response.text, "html.parser")
            results = []

            for result in soup.select(".result__body")[:3]:
                title = result.select_one(".result__a")
                snippet = result.select_one(".result__snippet")
                if title:
                    results.append(f"- {title.get_text(strip=True)}")

            if results:
                return f"Search results for '{query}':\n" + "\n".join(results)
            return f"No results found for: {query}"
        except Exception as e:
            return f"Search error: {e}"

    async def _tool_think(self, input_data: dict) -> str:
        return input_data.get("content", "")


class SubagentOrchestrator:
    """
    Manages multiple sub-agents for complex tasks.
    AgentZero can spawn sub-agents to handle subtasks in parallel.
    """

    @dataclass
    class SubagentHandle:
        id: str
        task: str
        status: str  # pending, running, completed, failed
        result: Optional[dict] = None

    def __init__(self, max_subagents: int = 5):
        self.max_subagents = max_subagents
        self.subagents: dict[str, "SubagentOrchestrator.SubagentHandle"] = {}
        self.logger = loguru.logger.bind(component="subagent-orchestrator")

    async def spawn_subagent(self, task: str, context: dict) -> SubagentHandle:
        """Spawn a new sub-agent to handle a subtask."""
        import uuid

        agent_id = str(uuid.uuid4())[:8]

        handle = self.SubagentHandle(
            id=agent_id,
            task=task,
            status="pending",
        )

        self.subagents[agent_id] = handle
        self.logger.info("Spawned subagent {} for: {}", agent_id, task[:30])

        asyncio.create_task(self._run_subagent(handle, context))

        return handle

    async def _run_subagent(self, handle: SubagentHandle, context: dict):
        """Run a subagent's task."""
        handle.status = "running"

        try:
            tir = TIRExecutor(TIRConfig())
            results = []

            async for step in tir.execute(handle.task):
                results.append(step)

                if step.thought_type == ThoughtType.FINAL:
                    break

            handle.result = {
                "status": "completed",
                "steps": len(results),
                "final": results[-1].content if results else "",
            }
            handle.status = "completed"

        except Exception as e:
            self.logger.error("Subagent {} failed: {}", handle.id, e)
            handle.result = {"status": "failed", "error": str(e)}
            handle.status = "failed"

    async def coordinate(self, subagent_ids: list[str]) -> dict:
        """Coordinate results from multiple sub-agents."""
        results = []

        for agent_id in subagent_ids:
            if agent_id in self.subagents:
                handle = self.subagents[agent_id]
                while handle.status == "running":
                    await asyncio.sleep(0.1)
                results.append(handle.result)

        return {
            "coordinated": len(results),
            "results": results,
        }

    def get_status(self, agent_id: str) -> Optional[SubagentHandle]:
        """Get status of a subagent."""
        return self.subagents.get(agent_id)


if __name__ == "__main__":

    async def test_tir():
        config = TIRConfig(max_steps=5)
        tir = TIRExecutor(config)

        async for step in tir.execute("What files are in the current directory?"):
            print(f"[{step.thought_type}] {step.content[:100]}")

    asyncio.run(test_tir())
