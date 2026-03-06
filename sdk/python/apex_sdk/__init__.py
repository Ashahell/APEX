"""
APEX Python SDK

A Python client for interacting with the APEX Router API.
"""

import hmac
import hashlib
import json
import time
from dataclasses import dataclass
from typing import Any, Optional
import urllib.request
import urllib.error
import urllib.parse


@dataclass
class Config:
    """Configuration for APEX client."""

    base_url: str = "http://localhost:3000"
    shared_secret: str = ""
    timeout: int = 30


@dataclass
class TaskRequest:
    """Request to create a task."""

    content: str
    channel: Optional[str] = None
    thread_id: Optional[str] = None
    author: Optional[str] = None
    max_steps: Optional[int] = None
    budget_usd: Optional[float] = None
    time_limit_secs: Optional[int] = None
    project: Optional[str] = None
    priority: Optional[str] = None
    category: Optional[str] = None

    def to_dict(self) -> dict:
        result = {"content": self.content}
        for field in [
            "channel",
            "thread_id",
            "author",
            "max_steps",
            "budget_usd",
            "time_limit_secs",
            "project",
            "priority",
            "category",
        ]:
            value = getattr(self, field)
            if value is not None:
                result[field] = value
        return result


@dataclass
class TaskResponse:
    """Response from creating a task."""

    task_id: str
    status: str
    tier: str
    capability_token: Optional[str] = None
    instant_response: Optional[str] = None


@dataclass
class TaskStatusResponse:
    """Task status response."""

    task_id: str
    status: str
    content: Optional[str] = None
    output: Optional[str] = None
    error: Optional[str] = None
    project: Optional[str] = None
    priority: Optional[str] = None
    category: Optional[str] = None
    created_at: Optional[str] = None


@dataclass
class Skill:
    """Skill information."""

    name: str
    version: str
    tier: str
    description: Optional[str] = None
    healthy: bool = False
    last_health_check: Optional[str] = None


@dataclass
class Metrics:
    """System metrics."""

    tasks: int
    by_tier: dict
    by_status: dict
    total_cost_usd: float


class ApexClient:
    """APEX Router API client."""

    def __init__(self, config: Config):
        self.config = config

    def _sign_request(self, method: str, path: str, body: str) -> tuple[str, str]:
        """Sign request with HMAC-SHA256."""
        timestamp = str(int(time.time()))
        message = timestamp + method + path + body

        h = hmac.new(
            self.config.shared_secret.encode(), message.encode(), hashlib.sha256
        )
        signature = h.hexdigest()

        return signature, timestamp

    def _request(self, method: str, path: str, body: Optional[dict] = None) -> dict:
        """Make HTTP request to APEX API."""
        body_str = json.dumps(body) if body else ""
        signature, timestamp = self._sign_request(method, path, body_str)

        headers = {
            "Content-Type": "application/json",
            "X-APEX-Signature": signature,
            "X-APEX-Timestamp": timestamp,
        }

        url = self.config.base_url + path
        data = body_str.encode() if body_str else None

        req = urllib.request.Request(url, data, headers, method=method)
        req.timeout = self.config.timeout

        try:
            with urllib.request.urlopen(req) as response:
                return json.loads(response.read().decode())
        except urllib.error.HTTPError as e:
            error_body = e.read().decode() if e.fp else ""
            raise Exception(f"API error: {e.code} - {error_body}")

    def create_task(self, request: TaskRequest) -> TaskResponse:
        """Create a new task."""
        response = self._request("POST", "/api/v1/tasks", request.to_dict())
        return TaskResponse(**response)

    def get_task(self, task_id: str) -> TaskStatusResponse:
        """Get task status."""
        response = self._request("GET", f"/api/v1/tasks/{task_id}")
        return TaskStatusResponse(**response)

    def list_tasks(
        self,
        project: Optional[str] = None,
        status: Optional[str] = None,
        priority: Optional[str] = None,
        category: Optional[str] = None,
        limit: int = 100,
        offset: int = 0,
    ) -> list[TaskStatusResponse]:
        """List tasks with filters."""
        params = {}
        if project:
            params["project"] = project
        if status:
            params["status"] = status
        if priority:
            params["priority"] = priority
        if category:
            params["category"] = category
        if limit != 100:
            params["limit"] = limit
        if offset:
            params["offset"] = offset

        query = "?" + urllib.parse.urlencode(params) if params else ""
        response = self._request("GET", f"/api/v1/tasks{query}")
        return [TaskStatusResponse(**t) for t in response]

    def cancel_task(self, task_id: str) -> TaskStatusResponse:
        """Cancel a task."""
        response = self._request("POST", f"/api/v1/tasks/{task_id}/cancel")
        return TaskStatusResponse(**response)

    def list_skills(self) -> list[Skill]:
        """List all skills."""
        response = self._request("GET", "/api/v1/skills")
        return [Skill(**s) for s in response]

    def get_skill(self, name: str) -> Skill:
        """Get skill by name."""
        response = self._request("GET", f"/api/v1/skills/{name}")
        return Skill(**response)

    def execute_skill(self, skill_name: str, input_data: dict) -> dict:
        """Execute a skill."""
        return self._request(
            "POST",
            "/api/v1/skills/execute",
            {"skill_name": skill_name, "input": input_data},
        )

    def get_metrics(self) -> Metrics:
        """Get system metrics."""
        response = self._request("GET", "/api/v1/metrics")
        return Metrics(**response)

    def health_check(self) -> dict:
        """Check router health."""
        return self._request("GET", "/health")

    def get_filter_options(self) -> dict:
        """Get available filter options."""
        return self._request("GET", "/api/v1/tasks/filter-options")


def create_client(
    shared_secret: str, base_url: str = "http://localhost:3000"
) -> ApexClient:
    """Create an APEX client with the given shared secret."""
    config = Config(base_url=base_url, shared_secret=shared_secret)
    return ApexClient(config)
