"""Healing event schemas for self-healing metrics.

This module defines event schemas for tracking self-healing operations
in the Qontinui automation system. These events are used by:
- qontinui (Python library) - emits events during healing operations
- qontinui-runner (Tauri app) - receives and displays healing metrics
- qontinui-web (frontend) - displays healing statistics and dashboards

The schema follows the pattern established by tree_events.py for consistency.
"""

from enum import Enum
from typing import Any

from pydantic import BaseModel, Field

# =============================================================================
# Enums
# =============================================================================


class HealingEventType(str, Enum):
    """Types of healing events emitted during self-healing operations."""

    # Cache events
    CACHE_HIT = "cache_hit"
    CACHE_MISS = "cache_miss"
    CACHE_INVALIDATED = "cache_invalidated"

    # Healing lifecycle
    HEALING_STARTED = "healing_started"
    HEALING_SUCCEEDED = "healing_succeeded"
    HEALING_FAILED = "healing_failed"

    # Strategy events
    STRATEGY_ATTEMPTED = "strategy_attempted"
    STRATEGY_SUCCEEDED = "strategy_succeeded"
    STRATEGY_FAILED = "strategy_failed"

    # Visual validation events
    VISUAL_VALIDATION_STARTED = "visual_validation_started"
    VISUAL_VALIDATION_PASSED = "visual_validation_passed"
    VISUAL_VALIDATION_FAILED = "visual_validation_failed"

    # Pattern/reliability events
    PATTERN_UPDATED = "pattern_updated"
    RELIABILITY_SCORE_CHANGED = "reliability_score_changed"


class HealingStrategy(str, Enum):
    """Strategies used during self-healing."""

    VISUAL_PATTERN = "visual_pattern"
    VISUAL_SEARCH = "visual_search"
    TEXT_SEARCH = "text_search"
    RELATIVE_POSITION = "relative_position"
    COLOR_REGION = "color_region"
    STRUCTURAL = "structural"
    LLM_VISION = "llm_vision"
    DOM_SELECTOR = "dom_selector"
    CACHE_HIT = "cache_hit"


# =============================================================================
# Nested Metadata Models
# =============================================================================


class CacheMetrics(BaseModel):
    """Metrics for cache operations."""

    cache_size: int = Field(default=0, description="Number of items in cache")
    hit_rate: float = Field(
        default=0.0, description="Cache hit rate (0.0-1.0)", ge=0.0, le=1.0
    )
    total_hits: int = Field(default=0, description="Total cache hits")
    total_misses: int = Field(default=0, description="Total cache misses")
    evictions: int = Field(default=0, description="Number of cache evictions")


class HealingAttemptInfo(BaseModel):
    """Information about a single healing attempt."""

    strategy: HealingStrategy = Field(description="Strategy that was attempted")
    success: bool = Field(description="Whether the attempt succeeded")
    confidence: float = Field(
        default=0.0, description="Match confidence (0.0-1.0)", ge=0.0, le=1.0
    )
    duration_ms: float = Field(default=0.0, description="Time taken in milliseconds")
    error: str | None = Field(default=None, description="Error message if failed")


class HealingMetrics(BaseModel):
    """Aggregate metrics for healing operations."""

    total_attempts: int = Field(default=0, description="Total healing attempts")
    successful_heals: int = Field(default=0, description="Number of successful heals")
    failed_heals: int = Field(default=0, description="Number of failed heals")
    healing_rate: float = Field(
        default=0.0, description="Success rate (0.0-1.0)", ge=0.0, le=1.0
    )
    avg_healing_time_ms: float = Field(
        default=0.0, description="Average healing time in milliseconds"
    )
    patterns_updated: int = Field(
        default=0, description="Number of patterns auto-updated"
    )
    llm_calls: int = Field(default=0, description="Number of LLM healing calls")


class ReliabilityInfo(BaseModel):
    """Information about pattern reliability."""

    pattern_id: str = Field(description="ID of the pattern")
    pattern_name: str | None = Field(
        default=None, description="Human-readable pattern name"
    )
    reliability_score: float = Field(
        description="Reliability score (0.0-1.0)", ge=0.0, le=1.0
    )
    previous_score: float | None = Field(
        default=None,
        description="Previous reliability score for comparison",
        ge=0.0,
        le=1.0,
    )
    total_uses: int = Field(default=0, description="Total times this pattern was used")
    successful_uses: int = Field(
        default=0, description="Successful matches for this pattern"
    )
    healing_required_count: int = Field(
        default=0, description="Times healing was needed for this pattern"
    )


class VisualValidationInfo(BaseModel):
    """Information about visual validation results."""

    validation_type: str = Field(description="Type of validation performed")
    expected_state: str | None = Field(
        default=None, description="Expected state identifier"
    )
    actual_state: str | None = Field(
        default=None, description="Actually detected state"
    )
    confidence: float = Field(
        default=0.0, description="Validation confidence (0.0-1.0)", ge=0.0, le=1.0
    )
    threshold: float = Field(
        default=0.0, description="Confidence threshold used", ge=0.0, le=1.0
    )
    passed: bool = Field(description="Whether validation passed")
    screenshot_reference: str | None = Field(
        default=None, description="Reference to screenshot used"
    )


# =============================================================================
# Main Event Models
# =============================================================================


class HealingEventData(BaseModel):
    """Data payload for healing events.

    Contains all information about a healing operation for logging and metrics.
    """

    # Event identification
    event_type: HealingEventType = Field(description="Type of healing event")
    timestamp: float = Field(description="Unix timestamp when event occurred")
    sequence: int = Field(default=0, description="Sequence number for ordering")

    # Context
    pattern_id: str | None = Field(
        default=None, description="ID of pattern being healed"
    )
    pattern_name: str | None = Field(
        default=None, description="Human-readable pattern name"
    )
    action_context: str | None = Field(
        default=None, description="Action that triggered healing"
    )
    state_id: str | None = Field(
        default=None, description="Current state machine state"
    )

    # Healing details
    strategy: HealingStrategy | None = Field(
        default=None, description="Strategy used/attempted"
    )
    strategies_tried: list[HealingAttemptInfo] = Field(
        default_factory=list, description="All strategies attempted"
    )
    successful_strategy: HealingStrategy | None = Field(
        default=None, description="Strategy that succeeded"
    )

    # Results
    success: bool = Field(default=False, description="Whether operation succeeded")
    confidence: float = Field(
        default=0.0, description="Match confidence (0.0-1.0)", ge=0.0, le=1.0
    )
    duration_ms: float = Field(default=0.0, description="Time taken in milliseconds")
    error_message: str | None = Field(
        default=None, description="Error message if failed"
    )

    # Location (if found)
    location_x: int | None = Field(default=None, description="X coordinate if found")
    location_y: int | None = Field(default=None, description="Y coordinate if found")
    location_width: int | None = Field(
        default=None, description="Width of found region"
    )
    location_height: int | None = Field(
        default=None, description="Height of found region"
    )

    # Metrics (for aggregate events)
    metrics: HealingMetrics | None = Field(
        default=None, description="Aggregate healing metrics"
    )
    cache_metrics: CacheMetrics | None = Field(
        default=None, description="Cache operation metrics"
    )
    reliability_info: ReliabilityInfo | None = Field(
        default=None, description="Pattern reliability information"
    )
    visual_validation: VisualValidationInfo | None = Field(
        default=None, description="Visual validation details"
    )

    # Additional data
    extra: dict[str, Any] = Field(
        default_factory=dict, description="Additional event-specific data"
    )

    class Config:
        extra = "allow"
        populate_by_name = True


class HealingEvent(BaseModel):
    """A healing event emitted during execution.

    This is the primary event type for healing operation logging, containing:
    - The event type (cache hit, healing started, etc.)
    - Full event data with metrics and context
    - Timestamp for ordering
    """

    type: str = Field(default="healing_event", description="Event type identifier")
    data: HealingEventData = Field(description="Event data payload")

    class Config:
        populate_by_name = True


# =============================================================================
# API Request/Response Models
# =============================================================================


class HealingEventCreate(BaseModel):
    """Request to store a healing event."""

    event_type: HealingEventType
    pattern_id: str | None = None
    pattern_name: str | None = None
    strategy: HealingStrategy | None = None
    success: bool = False
    confidence: float = 0.0
    duration_ms: float = 0.0
    error_message: str | None = None
    extra: dict[str, Any] = Field(default_factory=dict)


class HealingStatsResponse(BaseModel):
    """Response containing healing statistics."""

    total_events: int = Field(description="Total healing events recorded")
    cache_metrics: CacheMetrics = Field(description="Cache operation metrics")
    healing_metrics: HealingMetrics = Field(description="Healing operation metrics")
    strategy_breakdown: dict[str, int] = Field(
        default_factory=dict, description="Count of events by strategy"
    )
    recent_events: list[HealingEventData] = Field(
        default_factory=list, description="Most recent healing events"
    )
