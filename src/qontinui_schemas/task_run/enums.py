"""TaskRun-related enumerations.

These enums define the unified task run model used across:
- qontinui-runner (Rust + TypeScript)
- qontinui-web backend (Python/Pydantic)
- qontinui library (Python)

TaskRun is THE unified run concept. GUI automation is one aspect of a task.
"""

from enum import Enum


class TaskType(str, Enum):
    """Type of task run.

    Determines the nature and behavior of the task.
    """

    TASK = "task"
    """General task - may include AI sessions, automation, or both."""

    AUTOMATION = "automation"
    """Pure automation run without AI sessions."""

    SCHEDULED = "scheduled"
    """Scheduled/recurring task execution."""


class TaskRunStatus(str, Enum):
    """Status of a task run.

    Lifecycle: running -> (complete | failed | stopped)
    """

    RUNNING = "running"
    """Task is currently executing."""

    COMPLETE = "complete"
    """Task completed successfully (found [TASK_COMPLETE] marker)."""

    FAILED = "failed"
    """Task failed due to an error."""

    STOPPED = "stopped"
    """Task was manually stopped by user."""


class AutomationStatus(str, Enum):
    """Status of an automation execution within a task run.

    Used for task_run_automation child records.
    """

    RUNNING = "running"
    """Automation is currently executing."""

    SUCCESS = "success"
    """Automation completed successfully."""

    FAILED = "failed"
    """Automation failed."""

    TIMEOUT = "timeout"
    """Automation timed out."""

    CANCELLED = "cancelled"
    """Automation was cancelled."""


__all__ = [
    "TaskType",
    "TaskRunStatus",
    "AutomationStatus",
]
