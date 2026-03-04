"""
Curriculum Agent - Continuous Learning System

The Curriculum Agent analyzes execution history and improves the agent's
strategy over time. This is AgentZero's meta-learning layer.
"""

import json
import os
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from pathlib import Path
from typing import Any, Optional

import loguru


class LessonType(str, Enum):
    SUCCESS = "success"
    FAILURE = "failure"
    PATTERN = "pattern"
    IMPROVEMENT = "improvement"
    CORRECTION = "correction"


@dataclass
class Lesson:
    """A learned lesson from execution."""

    id: str
    lesson_type: LessonType
    description: str
    evidence: list[str]
    timestamp: str
    task_context: str
    applicability: float = 1.0


@dataclass
class Strategy:
    """An execution strategy that can be improved."""

    id: str
    name: str
    description: str
    success_rate: float = 0.0
    use_count: int = 0
    improvements: list[str] = field(default_factory=list)
    created_at: str
    last_used: Optional[str] = None


@dataclass
class ExecutionRecord:
    """Record of a single execution for analysis."""

    task_id: str
    task_description: str
    success: bool
    steps: int
    tools_used: list[str]
    errors: list[str]
    duration_seconds: float
    timestamp: str
    strategy_id: Optional[str] = None


class CurriculumAgent:
    """
    Curriculum Agent - Learns from execution history and improves strategies.

    This is AgentZero's meta-learning layer that enables continuous improvement.
    """

    def __init__(self, storage_path: str = "~/.apex/curriculum"):
        self.storage_path = Path(storage_path).expanduser()
        self.storage_path.mkdir(parents=True, exist_ok=True)

        self.logger = loguru.logger.bind(component="curriculum")

        self.lessons_file = self.storage_path / "lessons.jsonl"
        self.strategies_file = self.storage_path / "strategies.json"
        self.history_file = self.storage_path / "history.jsonl"

        self.lessons: list[Lesson] = self._load_lessons()
        self.strategies: dict[str, Strategy] = self._load_strategies()

        self._init_default_strategies()

    def _init_default_strategies(self):
        """Initialize default strategies if none exist."""
        if not self.strategies:
            self.strategies = {
                "direct": Strategy(
                    id="direct",
                    name="Direct Execution",
                    description="Try to complete the task directly without sub-planning",
                    created_at=datetime.utcnow().isoformat(),
                ),
                "decompose": Strategy(
                    id="decompose",
                    name="Task Decomposition",
                    description="Break complex tasks into smaller steps",
                    created_at=datetime.utcnow().isoformat(),
                ),
                "explore_first": Strategy(
                    id="explore_first",
                    name="Explore Before Action",
                    description="Gather information before taking action",
                    created_at=datetime.utcnow().isoformat(),
                ),
                "retry": Strategy(
                    id="retry",
                    name="Retry with Variation",
                    description="Retry failed approaches with modifications",
                    created_at=datetime.utcnow().isoformat(),
                ),
            }
            self._save_strategies()

    def _load_lessons(self) -> list[Lesson]:
        """Load lessons from storage."""
        if not self.lessons_file.exists():
            return []

        lessons = []
        with open(self.lessons_file) as f:
            for line in f:
                try:
                    data = json.loads(line)
                    lessons.append(Lesson(**data))
                except:
                    pass
        return lessons

    def _save_lessons(self):
        """Save lessons to storage."""
        with open(self.lessons_file, "w") as f:
            for lesson in self.lessons[-1000:]:
                f.write(json.dumps(lesson.__dict__) + "\n")

    def _load_strategies(self) -> dict[str, Strategy]:
        """Load strategies from storage."""
        if not self.strategies_file.exists():
            return {}

        with open(self.strategies_file) as f:
            data = json.load(f)
            return {k: Strategy(**v) for k, v in data.items()}

    def _save_strategies(self):
        """Save strategies to storage."""
        with open(self.strategies_file, "w") as f:
            json.dump({k: v.__dict__ for k, v in self.strategies.items()}, f, indent=2)

    def record_execution(self, record: ExecutionRecord):
        """Record an execution for future analysis."""
        self.logger.info(
            "Recording execution: {} - {}",
            record.task_id,
            "success" if record.success else "failure",
        )

        with open(self.history_file, "a") as f:
            f.write(json.dumps(record.__dict__) + "\n")

        if not record.success:
            self.analyze_failure(record)
        else:
            self.analyze_success(record)

    def analyze_success(self, record: ExecutionRecord):
        """Analyze a successful execution to extract lessons."""
        if record.steps < 3:
            lesson = Lesson(
                id=f"lesson_{datetime.utcnow().timestamp()}",
                lesson_type=LessonType.SUCCESS,
                description=f"Completed task in {record.steps} steps - efficient approach",
                evidence=[f"steps: {record.steps}", f"tools: {', '.join(record.tools_used)}"],
                timestamp=datetime.utcnow().isoformat(),
                task_context=record.task_description,
            )
            self.lessons.append(lesson)
            self.logger.info("Extracted lesson: {}", lesson.description)

        if record.steps > 10:
            lesson = Lesson(
                id=f"lesson_{datetime.utcnow().timestamp()}",
                lesson_type=LessonType.IMPROVEMENT,
                description=f"Task took {record.steps} steps - consider decomposition",
                evidence=[f"steps: {record.steps}"],
                timestamp=datetime.utcnow().isoformat(),
                task_context=record.task_description,
                applicability=0.7,
            )
            self.lessons.append(lesson)

        self._save_lessons()

    def analyze_failure(self, record: ExecutionRecord):
        """Analyze a failed execution to extract lessons."""
        for error in record.errors:
            lesson = Lesson(
                id=f"lesson_{datetime.utcnow().timestamp()}",
                lesson_type=LessonType.FAILURE,
                description=f"Failed with error: {error[:100]}",
                evidence=record.errors,
                timestamp=datetime.utcnow().isoformat(),
                task_context=record.task_description,
            )
            self.lessons.append(lesson)
            self.logger.warning("Failure lesson: {}", error[:100])

        if record.strategy_id and record.strategy_id in self.strategies:
            strategy = self.strategies[record.strategy_id]
            strategy.use_count += 1
            self._save_strategies()

        self._save_lessons()

    def get_relevant_lessons(self, task_description: str, limit: int = 5) -> list[Lesson]:
        """Get lessons relevant to the current task."""
        keywords = set(task_description.lower().split())

        scored = []
        for lesson in self.lessons[-100:]:
            score = 0.0

            lesson_keywords = set(lesson.description.lower().split())
            overlap = len(keywords & lesson_keywords)
            score += overlap * 0.3

            score += lesson.applicability * 0.5

            lesson_age = datetime.fromisoformat(lesson.timestamp)
            days_old = (datetime.utcnow() - lesson_age).days
            score += max(0, (30 - days_old) / 30) * 0.2

            scored.append((score, lesson))

        scored.sort(key=lambda x: x[0], reverse=True)
        return [l for _, l in scored[:limit]]

    def suggest_strategy(self, task_description: str) -> Strategy:
        """Suggest the best strategy based on learned lessons."""
        relevant = self.get_relevant_lessons(task_description)

        success_lessons = [l for l in relevant if l.lesson_type == LessonType.SUCCESS]
        failure_lessons = [l for l in relevant if l.lesson_type == LessonType.FAILURE]

        if len(success_lessons) > len(failure_lessons):
            return self.strategies.get("direct", list(self.strategies.values())[0])

        if any(
            "steps" in l.description.lower()
            for l in relevant
            if l.lesson_type == LessonType.IMPROVEMENT
        ):
            return self.strategies.get("decompose", list(self.strategies.values())[0])

        return list(self.strategies.values())[0]

    def improve_strategy(self, strategy_id: str, improvement: str) -> bool:
        """Add an improvement to a strategy."""
        if strategy_id not in self.strategies:
            return False

        strategy = self.strategies[strategy_id]
        strategy.improvements.append(
            {
                "improvement": improvement,
                "timestamp": datetime.utcnow().isoformat(),
            }
        )

        strategy.success_rate = min(1.0, strategy.success_rate + 0.05)

        self._save_strategies()
        self.logger.info("Improved strategy {}: {}", strategy_id, improvement)
        return True

    def get_stats(self) -> dict:
        """Get curriculum statistics."""
        return {
            "total_lessons": len(self.lessons),
            "total_strategies": len(self.strategies),
            "lessons_by_type": {
                "success": len([l for l in self.lessons if l.lesson_type == LessonType.SUCCESS]),
                "failure": len([l for l in self.lessons if l.lesson_type == LessonType.FAILURE]),
                "improvement": len(
                    [l for l in self.lessons if l.lesson_type == LessonType.IMPROVEMENT]
                ),
            },
            "strategies": {
                k: {"success_rate": v.success_rate, "use_count": v.use_count}
                for k, v in self.strategies.items()
            },
        }


class ReflectionGenerator:
    """Generates reflections on agent behavior."""

    def __init__(self, curriculum: CurriculumAgent):
        self.curriculum = curriculum
        self.logger = loguru.logger.bind(component="reflection")

    def generate_reflection(self, task: str, result: dict) -> str:
        """Generate a reflection on the execution."""
        relevant_lessons = self.curriculum.get_relevant_lessons(task, limit=3)

        if not relevant_lessons:
            return "This was a straightforward task."

        reflection = f"Reflecting on this task:\n"

        for lesson in relevant_lessons:
            if lesson.lesson_type == LessonType.SUCCESS:
                reflection += f"- Success pattern: {lesson.description}\n"
            elif lesson.lesson_type == LessonType.FAILURE:
                reflection += f"- Avoid: {lesson.description}\n"
            elif lesson.lesson_type == LessonType.IMPROVEMENT:
                reflection += f"- Consider: {lesson.description}\n"

        return reflection


if __name__ == "__main__":
    curriculum = CurriculumAgent()

    curriculum.record_execution(
        ExecutionRecord(
            task_id="test-1",
            task_description="Write a Python function",
            success=True,
            steps=2,
            tools_used=["code.generate"],
            errors=[],
            duration_seconds=5.0,
            timestamp=datetime.utcnow().isoformat(),
        )
    )

    print(json.dumps(curriculum.get_stats(), indent=2))
