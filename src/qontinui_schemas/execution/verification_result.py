"""Verification Result schemas.

Defines Pydantic models for verification phase results from unified workflow
execution. These match the JSON structure produced by the runner's Rust types
(VerificationPhaseResult, StepExecutionResult, etc.) and serve as the
single source of truth for the data contract between runner and web backend.

Used by:
- qontinui-web backend: API request/response validation, DB storage
- qontinui-web frontend: TypeScript types (via pydantic2ts generation)
- qontinui-runner: JSON contract reference (Rust types serialize to this shape)
"""

from datetime import datetime
from typing import Any
from uuid import UUID

from pydantic import BaseModel, Field


# =============================================================================
# Core Result Types (matching runner's Rust structs)
# =============================================================================


class CheckIssueDetail(BaseModel):
    """Details of an individual issue found by a check."""

    file: str = Field(..., description="File path where the issue was found")
    line: int | None = Field(None, description="Line number (1-based)")
    column: int | None = Field(None, description="Column number (1-based)")
    code: str | None = Field(
        None, description="Rule code (e.g., 'E501', 'no-unused-vars')"
    )
    message: str = Field(..., description="Issue message")
    severity: str = Field(..., description="Severity level: error, warning, info")
    fixable: bool = Field(False, description="Whether this issue is fixable")


class IndividualCheckResult(BaseModel):
    """Individual check result within a check group."""

    name: str = Field(..., description="Check name")
    status: str = Field(..., description="Status: passed, failed, skipped")
    duration_ms: int = Field(..., ge=0, description="Duration in milliseconds")
    issues_found: int = Field(0, ge=0, description="Number of issues found")
    issues_fixed: int = Field(0, ge=0, description="Number of issues fixed")
    files_checked: int = Field(0, ge=0, description="Number of files checked")
    error_message: str | None = Field(None, description="Error message if failed")
    output: str | None = Field(None, description="Raw output from the check tool")
    issues: list[CheckIssueDetail] = Field(
        default_factory=list, description="Individual issues found"
    )


class VerificationStepDetails(BaseModel):
    """Verification-specific details for test and check steps."""

    step_id: str = Field(..., description="Step ID from the workflow")
    phase: str = Field(..., description="Phase this step belongs to")
    stdout: str | None = Field(None, description="Standard output from the step")
    stderr: str | None = Field(None, description="Standard error from the step")
    assertions_passed: int | None = Field(
        None, description="Number of assertions passed"
    )
    assertions_total: int | None = Field(None, description="Total number of assertions")
    console_output: str | None = Field(
        None, description="Console output from browser/runtime"
    )
    page_snapshot: str | None = Field(
        None, description="Page snapshot (YAML accessibility tree)"
    )
    exit_code: int | None = Field(None, description="Exit code from command execution")
    check_results: list[IndividualCheckResult] | None = Field(
        None, description="Individual check results for check_group steps"
    )


class StepExecutionConfig(BaseModel):
    """Step configuration captured for AI visibility."""

    action_type: str | None = None
    target_image_id: str | None = None
    target_image_name: str | None = None
    monitor_index: int | None = None
    screenshot_delay: int | None = None
    timeout_seconds: int | None = None
    playwright_script_id: str | None = None
    initial_state_ids: list[str] | None = None
    check_type: str | None = None

    model_config = {"extra": "allow"}


class VerificationStepResult(BaseModel):
    """Result of executing a single verification step."""

    step_index: int = Field(..., ge=0, description="Step index (0-based)")
    step_type: str = Field(..., description="Step type that was executed")
    step_name: str = Field(..., description="Step name for display")
    step_id: str | None = Field(
        None, description="Step ID from the workflow definition"
    )
    success: bool = Field(..., description="Whether the step succeeded")
    error: str | None = Field(None, description="Error message if failed")
    screenshot_path: str | None = Field(
        None, description="Path to screenshot if captured"
    )
    started_at: str | None = Field(None, description="When this step started (ISO)")
    ended_at: str | None = Field(None, description="When this step ended (ISO)")
    duration_ms: int = Field(..., ge=0, description="Execution duration in ms")
    config: StepExecutionConfig = Field(
        default_factory=StepExecutionConfig, description="Step configuration"
    )
    verification_details: VerificationStepDetails | None = Field(
        None, description="Verification-specific fields"
    )
    output_data: dict[str, Any] | None = Field(
        None, description="Additional output data from the step handler"
    )


class GateEvaluationResult(BaseModel):
    """Result of evaluating a gate step."""

    gate_name: str = Field(..., description="Gate step name")
    required_step_ids: list[str] = Field(
        default_factory=list, description="Step IDs that the gate requires"
    )
    passed_step_ids: list[str] = Field(
        default_factory=list, description="Step IDs that passed"
    )
    failed_step_ids: list[str] = Field(
        default_factory=list, description="Step IDs that failed"
    )
    missing_step_ids: list[str] = Field(
        default_factory=list, description="Step IDs not found in results"
    )
    passed: bool = Field(
        ..., description="Whether the gate passed (all required steps passed)"
    )


class VerificationPhaseResult(BaseModel):
    """Result of running all verification steps in a unified workflow iteration."""

    iteration: int = Field(..., ge=1, description="Iteration number (1-indexed)")
    all_passed: bool = Field(..., description="Whether all verification steps passed")
    total_steps: int = Field(..., ge=0, description="Total number of steps")
    passed_steps: int = Field(..., ge=0, description="Number of steps that passed")
    failed_steps: int = Field(..., ge=0, description="Number of steps that failed")
    skipped_steps: int = Field(
        0, ge=0, description="Number of steps skipped (gate stop_on_failure)"
    )
    total_duration_ms: int = Field(
        ..., ge=0, description="Total execution time in milliseconds"
    )
    step_results: list[VerificationStepResult] = Field(
        default_factory=list, description="Individual step results"
    )
    critical_failure: bool = Field(
        False, description="Whether a critical gate failure occurred"
    )
    gate_results: list[GateEvaluationResult] = Field(
        default_factory=list, description="Gate evaluation results"
    )
    gate_based_evaluation: bool = Field(
        False, description="Whether evaluation was gate-based"
    )


# =============================================================================
# API Envelope Types (for backend endpoints)
# =============================================================================


class VerificationResultCreate(BaseModel):
    """Single verification result for upsert."""

    iteration: int = Field(..., ge=1, description="Iteration number")
    result: VerificationPhaseResult = Field(
        ..., description="Full verification phase result"
    )


class VerificationResultsBatchRequest(BaseModel):
    """Request to batch upsert verification results."""

    results: list[VerificationResultCreate] = Field(
        ..., min_length=1, max_length=100, description="Results to upsert"
    )


class VerificationResultResponse(BaseModel):
    """Response for a single stored verification result."""

    id: UUID = Field(..., description="Record ID")
    task_run_id: UUID = Field(..., description="Task run ID")
    iteration: int = Field(..., description="Iteration number")
    all_passed: bool = Field(..., description="Whether all steps passed")
    total_steps: int = Field(..., description="Total steps")
    passed_steps: int = Field(..., description="Steps that passed")
    failed_steps: int = Field(..., description="Steps that failed")
    skipped_steps: int = Field(..., description="Steps that were skipped")
    total_duration_ms: int = Field(..., description="Total duration in ms")
    critical_failure: bool = Field(..., description="Whether critical failure occurred")
    result_json: VerificationPhaseResult = Field(..., description="Full result data")
    created_at: datetime = Field(..., description="When the record was created")


class VerificationResultsListResponse(BaseModel):
    """Response for listing verification results for a task run."""

    task_run_id: UUID = Field(..., description="Task run ID")
    results: list[VerificationResultResponse] = Field(
        ..., description="Verification results ordered by iteration"
    )
    count: int = Field(..., description="Total number of results")
    passed_iterations: int = Field(
        ..., description="Number of iterations where all steps passed"
    )
    failed_iterations: int = Field(
        ..., description="Number of iterations with failures"
    )


__all__ = [
    # Core result types
    "CheckIssueDetail",
    "IndividualCheckResult",
    "VerificationStepDetails",
    "StepExecutionConfig",
    "VerificationStepResult",
    "GateEvaluationResult",
    "VerificationPhaseResult",
    # API envelope types
    "VerificationResultCreate",
    "VerificationResultsBatchRequest",
    "VerificationResultResponse",
    "VerificationResultsListResponse",
]
