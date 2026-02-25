"""Unified Workflow schemas - Multi-phase workflow orchestration.

The Unified Workflow defines how Qontinui executes tasks through a repeating
loop of setup, verification, agentic, and completion phases. It is the primary
orchestration format used by qontinui-runner and qontinui-web.

Usage:
    from qontinui_schemas.unified_workflow import (
        UnifiedWorkflow, WorkflowStage, CommandStep, PromptStep, UiBridgeStep,
    )

    # Literal types for type annotations
    from qontinui_schemas.unified_workflow import (
        WorkflowPhase, StepTypeName, CheckType, TestType,
    )
"""

from qontinui_schemas.unified_workflow.models import (
    ApiAssertion,
    ApiContentType,
    ApiVariableExtraction,
    BaseStep,
    CheckType,
    CommandStep,
    HealthCheckUrl,
    HttpMethod,
    PlaywrightExecutionMode,
    PromptStep,
    StepTypeName,
    TestType,
    UiBridgeStep,
    UnifiedWorkflow,
    WorkflowPhase,
    WorkflowStage,
    WorkflowStep,
)

__all__ = [
    # Literal types
    "WorkflowPhase",
    "StepTypeName",
    "CheckType",
    "TestType",
    "PlaywrightExecutionMode",
    "HttpMethod",
    "ApiContentType",
    # Supporting models
    "HealthCheckUrl",
    "ApiVariableExtraction",
    "ApiAssertion",
    # Step models
    "BaseStep",
    "CommandStep",
    "PromptStep",
    "UiBridgeStep",
    "WorkflowStep",
    # Workflow structure
    "WorkflowStage",
    "UnifiedWorkflow",
]
