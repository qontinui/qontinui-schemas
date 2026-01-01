"""Description models for AI verification agent feature.

These models provide rich descriptions for states, actions, transitions, and workflows
that enable an AI verification agent to explore applications using the state machine
structure and verify that reality matches the described expectations.

The descriptions are designed to be:
- Human-readable for documentation purposes
- Machine-readable for AI agents to understand intent
- Comprehensive enough to detect deviations from expected behavior
"""

from pydantic import BaseModel, Field


class StateDescription(BaseModel):
    """Rich description for a state in the state machine.

    Provides context for an AI verification agent to understand what a state
    represents and how to verify it matches expectations.
    """

    summary: str = Field(
        ...,
        description="Brief description of the state (1-2 sentences)",
    )
    expected_elements: list[str] | None = Field(
        None,
        description="UI elements that should be visible in this state (e.g., 'Login button', 'Username field')",
    )
    unexpected_elements: list[str] | None = Field(
        None,
        description="UI elements that should NOT be visible in this state - helps detect error dialogs or wrong states",
    )
    user_goal: str | None = Field(
        None,
        description="Business context - what the user is trying to accomplish when in this state",
    )
    verification_prompt: str | None = Field(
        None,
        description="Custom hints for AI verification - specific things to check or look for",
    )


class ActionDescription(BaseModel):
    """Rich description for an action in a workflow.

    Provides context for an AI verification agent to understand what an action
    is supposed to accomplish and how to verify it succeeded.
    """

    intent: str = Field(
        ...,
        description="What this action is supposed to accomplish (e.g., 'Click the submit button to proceed')",
    )
    preconditions: list[str] | None = Field(
        None,
        description="Conditions that should be true before this action executes",
    )
    postconditions: list[str] | None = Field(
        None,
        description="Conditions that should be true after this action completes successfully",
    )
    failure_modes: list[str] | None = Field(
        None,
        description="Known ways this action can fail (e.g., 'Button may be disabled', 'Network timeout')",
    )


class TransitionDescription(BaseModel):
    """Rich description for a transition between states.

    Provides context for an AI verification agent to understand what a transition
    accomplishes and how to verify it completed successfully.
    """

    intent: str = Field(
        ...,
        description="What this transition accomplishes (e.g., 'Navigate from login to dashboard')",
    )
    preconditions: list[str] | None = Field(
        None,
        description="Conditions that must be true before this transition can occur",
    )
    postconditions: list[str] | None = Field(
        None,
        description="Conditions that should be true after this transition completes",
    )
    failure_modes: list[str] | None = Field(
        None,
        description="Known ways this transition can fail (e.g., 'Authentication error', 'Server unavailable')",
    )
    expected_duration_ms: int | None = Field(
        None,
        ge=0,
        description="Typical duration of this transition in milliseconds - helps detect performance issues",
    )


class WorkflowDescription(BaseModel):
    """Rich description for an entire workflow.

    Provides high-level context for an AI verification agent to understand
    the overall purpose and success criteria of a workflow.
    """

    purpose: str = Field(
        ...,
        description="Overall goal of the workflow (e.g., 'Complete user registration process')",
    )
    success_criteria: str = Field(
        ...,
        description="How to determine if the workflow succeeded (e.g., 'User sees welcome message')",
    )
    steps_summary: list[str] | None = Field(
        None,
        description="Human-readable summary of the workflow steps for quick understanding",
    )
    business_context: str | None = Field(
        None,
        description="Why this workflow exists and its importance in the application",
    )


__all__ = [
    "StateDescription",
    "ActionDescription",
    "TransitionDescription",
    "WorkflowDescription",
]
