"""
Expectation and checkpoint configuration models for workflow validation.

This module provides models for defining expectations, checkpoints, and
success criteria for workflow execution validation.
"""

from typing import Any, Literal

from pydantic import BaseModel, Field


class ScreenRegion(BaseModel):
    """Rectangular screen region for OCR operations."""

    x: int
    y: int
    width: int
    height: int


class TextPresentAssertion(BaseModel):
    """Assertion that text must be present."""

    type: Literal["text_present"] = "text_present"
    pattern: str
    regex: bool | None = None
    case_sensitive: bool | None = Field(None, alias="caseSensitive")
    description: str | None = None
    is_critical: bool | None = Field(None, alias="isCritical")

    model_config = {"populate_by_name": True}


class TextAbsentAssertion(BaseModel):
    """Assertion that text must be absent."""

    type: Literal["text_absent"] = "text_absent"
    pattern: str
    regex: bool | None = None
    case_sensitive: bool | None = Field(None, alias="caseSensitive")
    description: str | None = None
    is_critical: bool | None = Field(None, alias="isCritical")

    model_config = {"populate_by_name": True}


class NoDuplicateMatchesAssertion(BaseModel):
    """Assertion that pattern should not match more than once."""

    type: Literal["no_duplicate_matches"] = "no_duplicate_matches"
    pattern: str
    regex: bool | None = None
    case_sensitive: bool | None = Field(None, alias="caseSensitive")
    description: str | None = None
    is_critical: bool | None = Field(None, alias="isCritical")

    model_config = {"populate_by_name": True}


class TextCountAssertion(BaseModel):
    """Assertion that checks exact or bounded count of matches."""

    type: Literal["text_count"] = "text_count"
    pattern: str
    regex: bool | None = None
    case_sensitive: bool | None = Field(None, alias="caseSensitive")
    description: str | None = None
    is_critical: bool | None = Field(None, alias="isCritical")
    count: int | None = None
    min_count: int | None = Field(None, alias="minCount")
    max_count: int | None = Field(None, alias="maxCount")

    model_config = {"populate_by_name": True}


class TextInRegionAssertion(BaseModel):
    """Assertion that text must appear in a specific screen region."""

    type: Literal["text_in_region"] = "text_in_region"
    pattern: str
    regex: bool | None = None
    case_sensitive: bool | None = Field(None, alias="caseSensitive")
    description: str | None = None
    is_critical: bool | None = Field(None, alias="isCritical")
    region: ScreenRegion

    model_config = {"populate_by_name": True}


# Union type of all OCR assertion types
OcrAssertion = (
    TextPresentAssertion
    | TextAbsentAssertion
    | NoDuplicateMatchesAssertion
    | TextCountAssertion
    | TextInRegionAssertion
)


class CheckpointDefinition(BaseModel):
    """Named checkpoint with assertions and validation rules."""

    ocr_assertions: list[OcrAssertion] | None = Field(None, alias="ocrAssertions")
    claude_review: list[str] | None = Field(None, alias="claudeReview")
    screenshot_required: bool | None = Field(None, alias="screenshotRequired")
    max_wait_ms: int | None = Field(None, alias="maxWaitMs")
    retry_interval_ms: int | None = Field(None, alias="retryIntervalMs")
    description: str | None = None

    model_config = {"populate_by_name": True}


class GlobalExpectations(BaseModel):
    """Global expectations that apply to entire workflow execution."""

    no_console_errors: bool | None = Field(None, alias="noConsoleErrors")
    no_network_errors: bool | None = Field(None, alias="noNetworkErrors")
    max_action_duration_ms: int | None = Field(None, alias="maxActionDurationMs")
    max_total_duration_ms: int | None = Field(None, alias="maxTotalDurationMs")
    allow_partial_matches: bool | None = Field(None, alias="allowPartialMatches")
    min_confidence_threshold: float | None = Field(None, alias="minConfidenceThreshold")

    model_config = {"populate_by_name": True}


class AllActionsPassCriteria(BaseModel):
    """All actions must pass."""

    type: Literal["all_actions_pass"] = "all_actions_pass"
    description: str | None = None


class MinMatchesCriteria(BaseModel):
    """Minimum number of pattern matches required."""

    type: Literal["min_matches"] = "min_matches"
    min_matches: int = Field(alias="minMatches")
    description: str | None = None

    model_config = {"populate_by_name": True}


class MaxFailuresCriteria(BaseModel):
    """Maximum number of failures allowed."""

    type: Literal["max_failures"] = "max_failures"
    max_failures: int = Field(alias="maxFailures")
    description: str | None = None

    model_config = {"populate_by_name": True}


class CheckpointPassedCriteria(BaseModel):
    """Specific checkpoint(s) must pass."""

    type: Literal["checkpoint_passed"] = "checkpoint_passed"
    checkpoint_name: str | None = Field(None, alias="checkpointName")
    checkpoints: list[str] | None = None
    description: str | None = None

    model_config = {"populate_by_name": True}


class RequiredStatesCriteria(BaseModel):
    """Specific states must be visited during workflow."""

    type: Literal["required_states"] = "required_states"
    required_states: list[str] = Field(alias="requiredStates")
    description: str | None = None

    model_config = {"populate_by_name": True}


class CustomCriteria(BaseModel):
    """Custom expression for success evaluation."""

    type: Literal["custom"] = "custom"
    custom_expression: str = Field(alias="customExpression")
    description: str | None = None

    model_config = {"populate_by_name": True}


# Union type of all success criteria
SuccessCriteria = (
    AllActionsPassCriteria
    | MinMatchesCriteria
    | MaxFailuresCriteria
    | CheckpointPassedCriteria
    | RequiredStatesCriteria
    | CustomCriteria
)


class ActionDefaults(BaseModel):
    """Default settings for action-level expectations."""

    is_terminal_on_failure: bool | None = Field(None, alias="isTerminalOnFailure")
    capture_checkpoint_on_failure: bool | None = Field(
        None, alias="captureCheckpointOnFailure"
    )
    capture_checkpoint_after: bool | None = Field(None, alias="captureCheckpointAfter")
    max_retries: int | None = Field(None, alias="maxRetries")
    retry_delay_ms: int | None = Field(None, alias="retryDelayMs")

    model_config = {"populate_by_name": True}


class ActionExpectations(BaseModel):
    """Expectations that can be attached to individual actions."""

    is_terminal_on_failure: bool | None = Field(None, alias="isTerminalOnFailure")
    capture_checkpoint_on_failure: bool | None = Field(
        None, alias="captureCheckpointOnFailure"
    )
    capture_checkpoint_after: bool | None = Field(None, alias="captureCheckpointAfter")
    checkpoint_name: str | None = Field(None, alias="checkpointName")
    max_retries: int | None = Field(None, alias="maxRetries")
    retry_delay_ms: int | None = Field(None, alias="retryDelayMs")
    max_duration_ms: int | None = Field(None, alias="maxDurationMs")
    expected_state_after: str | None = Field(None, alias="expectedStateAfter")

    model_config = {"populate_by_name": True}


class WorkflowExpectations(BaseModel):
    """Complete expectations configuration for a workflow."""

    global_: GlobalExpectations | None = Field(None, alias="global")
    checkpoints: dict[str, CheckpointDefinition] | None = None
    success_criteria: SuccessCriteria | None = Field(None, alias="successCriteria")
    action_defaults: ActionDefaults | None = Field(None, alias="actionDefaults")

    model_config = {"populate_by_name": True}


class AssertionResult(BaseModel):
    """Result of a single assertion."""

    type: str
    pattern: str
    passed: bool
    description: str | None = None
    actual_value: Any = Field(None, alias="actualValue")
    expected_value: Any = Field(None, alias="expectedValue")
    error: str | None = None

    model_config = {"populate_by_name": True}


class ClaudeReviewResult(BaseModel):
    """Result of a Claude review."""

    instruction: str
    passed: bool
    observations: str
    confidence: float | None = None

    model_config = {"populate_by_name": True}


class CheckpointValidationResult(BaseModel):
    """Validation result for a checkpoint."""

    checkpoint_name: str = Field(alias="checkpointName")
    passed: bool
    assertion_results: list[AssertionResult] = Field(alias="assertionResults")
    screenshot_path: str | None = Field(None, alias="screenshotPath")
    claude_review_results: list[ClaudeReviewResult] | None = Field(
        None, alias="claudeReviewResults"
    )
    duration_ms: int = Field(alias="durationMs")
    error: str | None = None

    model_config = {"populate_by_name": True}


class WorkflowExecutionResult(BaseModel):
    """Overall workflow execution result with expectations."""

    success: bool
    success_criteria: SuccessCriteria | None = Field(None, alias="successCriteria")
    checkpoint_results: list[CheckpointValidationResult] = Field(
        alias="checkpointResults"
    )
    actions_passed: int = Field(alias="actionsPassed")
    actions_failed: int = Field(alias="actionsFailed")
    total_duration_ms: int = Field(alias="totalDurationMs")
    exceeded_max_duration: bool = Field(alias="exceededMaxDuration")
    console_errors: list[str] | None = Field(None, alias="consoleErrors")
    network_errors: list[str] | None = Field(None, alias="networkErrors")
    states_visited: list[str] = Field(alias="statesVisited")
    error: str | None = None

    model_config = {"populate_by_name": True}
