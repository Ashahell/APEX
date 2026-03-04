"""
APEX Execution Engine - Agent Zero Fork

This module implements the autonomous agent loop for Deep tasks.
It follows Agent Zero's plan → act → observe → reflect pattern,
adapted for APEX's Firecracker VM isolation and permission system.
"""

import asyncio
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional
import json

import loguru
import requests


class TaskStatus(str, Enum):
    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"
    BUDGET_EXCEEDED = "budget_exceeded"


@dataclass
class AgentConfig:
    """Configuration for the agent execution."""

    max_steps: int = 50
    max_cost_usd: float = 5.0
    max_cost_cents: int = 500
    allowed_domains: list[str] = field(default_factory=list)
    allowed_skills: list[str] = field(default_factory=list)
    timeout_seconds: int = 300
    llm_url: str = "http://localhost:8080"
    llm_model: str = "qwen3-4b"
    context_window_tokens: int = 32768  # 32k tokens (qwen3-4b supports up to 32k-128k)


@dataclass
class ToolResult:
    """Result from a tool execution."""

    tool_name: str
    success: bool
    output: str | None = None
    error: str | None = None
    artifacts: list[dict[str, Any]] = field(default_factory=list)
    cost_cents: int = 0


@dataclass
class AgentContext:
    """Context for the current agent execution."""

    task: str
    history: list[dict[str, Any]] = field(default_factory=list)
    tools_executed: list[str] = field(default_factory=list)
    total_cost_cents: int = 0
    step: int = 0

    def add_user_message(self, content: str) -> None:
        self.history.append({"role": "user", "content": content, "step": self.step})

    def add_assistant_message(self, content: str) -> None:
        self.history.append({"role": "assistant", "content": content, "step": self.step})

    def add_tool_result(self, tool_name: str, result: ToolResult) -> None:
        self.history.append(
            {
                "role": "tool",
                "tool_name": tool_name,
                "content": result.output or result.error or "",
                "success": result.success,
                "step": self.step,
            }
        )
        self.tools_executed.append(tool_name)
        self.total_cost_cents += result.cost_cents


class ApexAgent:
    """
    Autonomous agent implementing the execution loop.

    This is a fork/modification of Agent Zero's agent loop,
    adapted for APEX's single-user model and Firecracker isolation.
    """

    SYSTEM_PROMPT = """You are an autonomous AI agent that executes tasks by planning, acting, observing, and reflecting.

Your goal is to complete user tasks by breaking them down into steps and executing tools.

## Available Tools
- code.generate: Generate code from natural language
- code.review: Review code for issues
- docs.read: Read documentation
- shell.execute: Execute shell commands (requires T3 verification)
- file.read: Read files from the filesystem
- file.write: Write content to files
- web.search: Search the web for information
- web.fetch: Fetch content from URLs

## Loop Pattern
1. PLAN: Analyze the task and decide on the next action
2. ACT: Execute the chosen action using a tool
3. OBSERVE: Check the result of the action
4. REFLECT: Determine if the task is complete or if more steps are needed

## Guidelines
- Break complex tasks into smaller steps
- Execute one action at a time
- Check your work after each step
- Ask for confirmation for destructive operations
- Stay within budget limits
- Return a final response when the task is complete

When you have completed the task, respond with "TASK_COMPLETE: <summary of what was done>" """

    def __init__(self, config: AgentConfig):
        self.config = config
        self.logger = loguru.logger.bind(component="apex-agent")
        self.tools: dict[str, Any] = {}
        self._register_default_tools()

    def _register_default_tools(self) -> None:
        """Register the default available tools."""
        self.tools = {
            "code.generate": self._tool_code_generate,
            "code.review": self._tool_code_review,
            "docs.read": self._tool_docs_read,
            "shell.execute": self._tool_shell_execute,
            "file.read": self._tool_file_read,
            "file.write": self._tool_file_write,
            "web.search": self._tool_web_search,
            "web.fetch": self._tool_web_fetch,
        }

    async def run(self, task: str) -> dict[str, Any]:
        """
        Execute a deep task using the agent loop.

        Returns:
            dict with keys: status, output, artifacts, cost
        """
        self.logger.info("Starting agent execution for task: {}", task[:100])

        context = AgentContext(task=task)
        context.add_user_message(task)

        try:
            result = await self._execute_loop(context)

            return {
                "status": TaskStatus.COMPLETED,
                "output": result,
                "steps": context.step,
                "tools_used": context.tools_executed,
                "cost_cents": context.total_cost_cents,
                "cost_usd": context.total_cost_cents / 100.0,
            }
        except BudgetExceededError as e:
            self.logger.warning("Budget exceeded: {} cents", e.cents)
            return {
                "status": TaskStatus.BUDGET_EXCEEDED,
                "output": None,
                "error": f"Budget exceeded: {e.cents} cents",
                "steps": context.step,
                "tools_used": context.tools_executed,
                "cost_cents": context.total_cost_cents,
                "cost_usd": context.total_cost_cents / 100.0,
            }
        except Exception as e:
            self.logger.exception("Agent execution failed")
            return {
                "status": TaskStatus.FAILED,
                "output": None,
                "error": str(e),
                "steps": context.step,
                "tools_used": context.tools_executed,
                "cost_cents": context.total_cost_cents,
                "cost_usd": context.total_cost_cents / 100.0,
            }

    async def _execute_loop(self, context: AgentContext) -> str:
        """Main agent loop: plan → act → observe → reflect."""

        for step in range(self.config.max_steps):
            context.step = step
            self.logger.debug("Agent step {}/{}", step + 1, self.config.max_steps)

            if context.total_cost_cents >= self.config.max_cost_cents:
                raise BudgetExceededError(context.total_cost_cents)

            # Plan: Decide what to do
            plan = await self._plan(context)

            if plan.get("action") == "respond":
                response = plan.get("content", "")
                context.add_assistant_message(response)

                if "TASK_COMPLETE:" in response or self._is_complete(response):
                    return response.replace("TASK_COMPLETE:", "").strip()
                continue

            # Act: Execute the planned action
            result = await self._act(plan, context)

            # Observe & Reflect: Check the result
            context.add_tool_result(plan.get("tool", "unknown"), result)

            if result.success:
                context.add_assistant_message(f"Executed {plan.get('tool')}: {result.output}")

                if self._is_complete(result.output or ""):
                    return result.output or "Task completed"
            else:
                self.logger.warning("Tool execution failed: {}", result.error)

            # Reflect: Check if we should continue
            if self._should_stop(step):
                break

        return "Task did not complete within step limit"

    async def _plan(self, context: AgentContext) -> dict[str, Any]:
        """Use LLM to decide the next action."""

        messages = [
            {"role": "system", "content": self.SYSTEM_PROMPT},
        ]

        for msg in context.history[-10:]:
            if msg["role"] == "tool":
                messages.append(
                    {
                        "role": "tool",
                        "content": f"[{msg['tool_name']}] {msg['content']}",
                    }
                )
            else:
                messages.append({"role": msg["role"], "content": msg["content"]})

        messages.append(
            {
                "role": "user",
                "content": f"What is the next action? Respond in JSON format: {self._get_plan_format()}",
            }
        )

        try:
            response = requests.post(
                f"{self.config.llm_url}/v1/chat/completions",
                json={
                    "model": self.config.llm_model,
                    "messages": messages,
                    "temperature": 0.7,
                    "max_tokens": 512,
                },
                timeout=30,
            )
            response.raise_for_status()
            data = response.json()
            content = data["choices"][0]["message"]["content"]

            try:
                plan = json.loads(content)
                return plan
            except json.JSONDecodeError:
                self.logger.warning("Failed to parse LLM response as JSON: {}", content[:100])
                return {"action": "respond", "content": content}

        except requests.RequestException as e:
            self.logger.error("LLM request failed: {}", e)
            return {"action": "respond", "content": f"Error: LLM unavailable - {e}"}

    def _get_plan_format(self) -> str:
        """Return the expected JSON format for plans."""
        return json.dumps(
            {
                "action": "execute_tool | respond",
                "tool": "tool_name (if execute_tool)",
                "input": {"key": "value"},
                "content": "response text (if respond)",
            }
        )

    async def _act(self, plan: dict[str, Any], context: AgentContext) -> ToolResult:
        """Execute the planned action."""

        action = plan.get("action")

        if action == "execute_tool":
            tool_name = plan.get("tool", "")
            tool_input = plan.get("input", {})

            if tool_name in self.tools:
                try:
                    return await self.tools[tool_name](tool_input, context)
                except Exception as e:
                    return ToolResult(
                        tool_name=tool_name,
                        success=False,
                        error=f"Tool execution error: {str(e)}",
                        cost_cents=1,
                    )
            else:
                return ToolResult(
                    tool_name=tool_name,
                    success=False,
                    error=f"Unknown tool: {tool_name}",
                    cost_cents=0,
                )

        elif action == "respond":
            return ToolResult(
                tool_name="respond",
                success=True,
                output=plan.get("content", ""),
                cost_cents=1,
            )

        return ToolResult(
            tool_name=action or "unknown",
            success=False,
            error=f"Unknown action: {action}",
            cost_cents=0,
        )

    async def _tool_code_generate(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Generate code."""
        language = input_data.get("language", "python")
        description = input_data.get("description", "")

        self.logger.info("Generating {} code: {}", language, description[:50])

        return ToolResult(
            tool_name="code.generate",
            success=True,
            output=f"# Generated {language} code\n# Description: {description}\n\n# Implementation placeholder",
            cost_cents=5,
        )

    async def _tool_code_review(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Review code."""
        code = input_data.get("code", "")

        self.logger.info("Reviewing code: {} chars", len(code))

        return ToolResult(
            tool_name="code.review",
            success=True,
            output="Code review: No major issues found. Consider adding type hints.",
            cost_cents=3,
        )

    async def _tool_docs_read(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Read documentation."""
        topic = input_data.get("topic", "")

        return ToolResult(
            tool_name="docs.read",
            success=True,
            output=f"Documentation for {topic}: (placeholder)",
            cost_cents=1,
        )

    async def _tool_shell_execute(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Execute shell command."""
        command = input_data.get("command", "")

        self.logger.info("Executing shell: {}", command[:100])

        return ToolResult(
            tool_name="shell.execute",
            success=True,
            output="Shell execution requires T3 verification in APEX",
            cost_cents=10,
        )

    async def _tool_file_read(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Read a file."""
        path = input_data.get("path", "")

        return ToolResult(
            tool_name="file.read",
            success=True,
            output=f"File content from {path}: (placeholder)",
            cost_cents=1,
        )

    async def _tool_file_write(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Write to a file."""
        path = input_data.get("path", "")
        content = input_data.get("content", "")

        return ToolResult(
            tool_name="file.write",
            success=True,
            output=f"Written to {path}: {len(content)} bytes",
            cost_cents=2,
        )

    async def _tool_web_search(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Search the web."""
        query = input_data.get("query", "")

        return ToolResult(
            tool_name="web.search",
            success=True,
            output=f"Search results for '{query}': (placeholder)",
            cost_cents=5,
        )

    async def _tool_web_fetch(self, input_data: Any, context: AgentContext) -> ToolResult:
        """Fetch content from a URL."""
        url = input_data.get("url", "")

        return ToolResult(
            tool_name="web.fetch",
            success=True,
            output=f"Content from {url}: (placeholder)",
            cost_cents=3,
        )

    def _is_complete(self, output: str) -> bool:
        """Check if the task is complete."""
        if "TASK_COMPLETE:" in output:
            return True
        complete_indicators = ["completed", "done", "finished", "success", "created", "implemented"]
        return any(indicator in output.lower() for indicator in complete_indicators)

    def _should_stop(self, current_step: int) -> bool:
        """Determine if the agent should stop."""
        return current_step >= self.config.max_steps - 1


class BudgetExceededError(Exception):
    """Raised when the agent exceeds its budget."""

    def __init__(self, cents: int):
        self.cents = cents
        super().__init__(f"Budget exceeded: {cents} cents")


async def main() -> None:
    """Entry point for the execution engine."""
    config = AgentConfig(
        max_steps=50,
        max_cost_usd=5.0,
        max_cost_cents=500,
    )

    agent = ApexAgent(config)
    result = await agent.run("Build a simple web server in Python")

    print(json.dumps(result, indent=2, default=str))


if __name__ == "__main__":
    asyncio.run(main())
