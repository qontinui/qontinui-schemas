"""TaskRun schemas - Unified task execution model.

TaskRun is THE unified run concept for all execution in Qontinui:
- AI tasks (with Claude sessions)
- Automation runs (GUI automation without AI)
- Mixed runs (AI + automation)

Design principles:
1. Always create TaskRun for any execution
2. One model, one source of output, one storage location
3. task_run_automation is a child table for automation-specific metrics

Usage:
    from qontinui_schemas.task_run import (
        TaskType, TaskRunStatus, AutomationStatus,
        TaskRunCreate, TaskRunResponse, TaskRunDetail,
        TaskRunAutomationCreate, TaskRunAutomationDetail,
    )
"""

from qontinui_schemas.task_run.enums import (
    AutomationStatus,
    TaskRunStatus,
    TaskType,
)
from qontinui_schemas.task_run.models import (
    TaskRunAutomationBase,
    TaskRunAutomationComplete,
    TaskRunAutomationCreate,
    TaskRunAutomationDetail,
    TaskRunAutomationListResponse,
    TaskRunAutomationResponse,
    TaskRunBase,
    TaskRunComplete,
    TaskRunCreate,
    TaskRunDetail,
    TaskRunListResponse,
    TaskRunReopen,
    TaskRunResponse,
    TaskRunSyncPayload,
    TaskRunUpdate,
)

__all__ = [
    # Enums
    "TaskType",
    "TaskRunStatus",
    "AutomationStatus",
    # TaskRun models
    "TaskRunBase",
    "TaskRunCreate",
    "TaskRunResponse",
    "TaskRunDetail",
    "TaskRunUpdate",
    "TaskRunComplete",
    "TaskRunReopen",
    "TaskRunListResponse",
    # TaskRunAutomation models
    "TaskRunAutomationBase",
    "TaskRunAutomationCreate",
    "TaskRunAutomationResponse",
    "TaskRunAutomationDetail",
    "TaskRunAutomationComplete",
    "TaskRunAutomationListResponse",
    # Sync
    "TaskRunSyncPayload",
]
