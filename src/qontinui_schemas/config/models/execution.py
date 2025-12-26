"""
Execution control models for action behavior.

This module provides models for controlling how actions are executed,
including timing, repetition, retries, and error handling.

It also includes state machine execution result types for state navigation
and transition execution.
"""

from typing import Literal

from pydantic import BaseModel, Field

from .logging import LoggingOptions


class RepetitionOptions(BaseModel):
    """Repetition configuration for actions."""

    count: int | None = None
    pause_between: int | None = Field(None, alias="pauseBetween")
    stop_on_success: bool | None = Field(None, alias="stopOnSuccess")
    stop_on_failure: bool | None = Field(None, alias="stopOnFailure")

    model_config = {"populate_by_name": True}


class BaseActionSettings(BaseModel):
    """Base settings that apply to all actions."""

    pause_before_begin: int | None = Field(None, alias="pauseBeforeBegin")
    pause_after_end: int | None = Field(None, alias="pauseAfterEnd")
    illustrate: Literal["YES", "NO", "USE_GLOBAL"] | None = None
    logging_options: LoggingOptions | None = Field(None, alias="loggingOptions")

    model_config = {"populate_by_name": True}


class ExecutionSettings(BaseModel):
    """Execution control settings.

    Note: Model-based GUI automation is resilient by design - workflows always
    continue executing even if individual actions fail. There is no option to
    stop workflow execution on action failure.
    """

    timeout: int | None = None
    retry_count: int | None = Field(None, alias="retryCount")
    repetition: RepetitionOptions | None = None

    model_config = {"populate_by_name": True}


# ============================================================================
# State Machine Execution Results
# ============================================================================


class TransitionInfo(BaseModel):
    """Information about a single transition."""

    id: str = Field(..., description="Unique identifier for the transition")
    from_state: str = Field(
        ..., alias="fromState", description="ID of the source state"
    )
    to_state: str | None = Field(
        ...,
        alias="toState",
        description="ID of the destination state (may be null for transitions without a fixed destination)",
    )
    workflows: list[str] = Field(
        ..., description="List of workflow IDs that can trigger this transition"
    )

    model_config = {"populate_by_name": True}


class TransitionExecutionResult(BaseModel):
    """Result of executing a state transition.

    Contains information about the transition execution including success status,
    the states that were activated/deactivated, and any error messages.
    """

    success: bool = Field(
        ..., description="Whether the transition executed successfully"
    )
    transition_id: str = Field(
        ..., alias="transitionId", description="ID of the transition that was executed"
    )
    active_states: list[str] = Field(
        ...,
        alias="activeStates",
        description="List of state IDs that are currently active after transition",
    )
    error: str | None = Field(
        None,
        description="Error message if the transition failed (undefined if success is true)",
    )

    model_config = {"populate_by_name": True}


class NavigationResult(BaseModel):
    """Result of navigating to one or more states.

    Contains the navigation path taken, current active states, and success status.
    """

    success: bool = Field(
        ..., description="Whether the navigation completed successfully"
    )
    path: list[str] = Field(
        ...,
        description="The path of states traversed during navigation (empty if navigation failed or no path was needed)",
    )
    active_states: list[str] = Field(
        ...,
        alias="activeStates",
        description="List of state IDs that are currently active after navigation",
    )
    target_state: str | None = Field(
        None,
        alias="targetState",
        description="ID of the target state (single state navigation, undefined for multi-state)",
    )
    results: list["NavigationResult"] | None = Field(
        None,
        description="Detailed results for each state in multi-state navigation (undefined for single state)",
    )
    error: str | None = Field(
        None,
        description="Error message if navigation failed (undefined if success is true)",
    )

    model_config = {"populate_by_name": True}


class ActiveStatesResult(BaseModel):
    """Result of querying currently active states.

    Provides information about which states are currently active in the state machine.
    """

    success: bool = Field(..., description="Whether the query executed successfully")
    active_states: list[str] = Field(
        ..., alias="activeStates", description="List of currently active state IDs"
    )
    current_state: str | None = Field(
        None,
        alias="currentState",
        description="The primary current state (if single-state mode, may be null if no state is active)",
    )
    state_history: list[str] | None = Field(
        None,
        alias="stateHistory",
        description="History of previously active states (most recent first)",
    )
    error: str | None = Field(
        None,
        description="Error message if the query failed (undefined if success is true)",
    )

    model_config = {"populate_by_name": True}


class AvailableTransitionsResult(BaseModel):
    """Result of querying available transitions from current state.

    Provides information about which transitions can be executed from the current state.
    """

    success: bool = Field(..., description="Whether the query executed successfully")
    transitions: list[TransitionInfo] = Field(
        ...,
        description="List of available transitions (empty if no transitions are available or query failed)",
    )
    current_state: str | None = Field(
        None,
        alias="currentState",
        description="The current state these transitions are available from (may be undefined if no current state)",
    )
    message: str | None = Field(
        None, description='Informational message (e.g., "No current state")'
    )
    error: str | None = Field(
        None,
        description="Error message if the query failed (undefined if success is true)",
    )

    model_config = {"populate_by_name": True}
