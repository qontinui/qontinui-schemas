"""
Root configuration models for Qontinui automation.

This module provides the top-level configuration structure for exported
automation configurations, including metadata, settings, and the complete
QontinuiConfig that ties everything together.
"""

from enum import Enum

from pydantic import BaseModel, Field

from .state_machine import State, Transition
from .workflow import Workflow

# =============================================================================
# Enums
# =============================================================================


class ImageFormat(str, Enum):
    """Supported image formats."""

    PNG = "png"
    JPG = "jpg"
    JPEG = "jpeg"


class ImageSource(str, Enum):
    """How an image was added to the library."""

    UPLOADED = "uploaded"
    PATTERN_OPTIMIZATION = "pattern_optimization"
    IMAGE_EXTRACTION = "image_extraction"
    STATE_DISCOVERY = "state_discovery"


class FailureStrategy(str, Enum):
    """Strategy for handling action failures."""

    STOP = "stop"
    CONTINUE = "continue"
    PAUSE = "pause"


class SearchAlgorithm(str, Enum):
    """Image search algorithm."""

    TEMPLATE_MATCHING = "template_matching"
    FEATURE_MATCHING = "feature_matching"
    AI = "ai"


class ColorSpace(str, Enum):
    """Color space for image processing."""

    RGB = "rgb"
    GRAYSCALE = "grayscale"
    HSV = "hsv"


class LogLevel(str, Enum):
    """Logging levels."""

    DEBUG = "debug"
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"


# =============================================================================
# Category
# =============================================================================


class Category(BaseModel):
    """
    Workflow category for organization and automation control.

    Categories organize workflows and control which are available for
    automation in the runner. Only workflows in categories with
    automationEnabled=True are shown in the runner's workflow list.
    """

    name: str = Field(..., description="Category name (e.g., 'Main', 'Testing')")
    automation_enabled: bool = Field(
        default=True,
        alias="automationEnabled",
        description="Whether workflows in this category are available for automation",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Image Asset
# =============================================================================


class ImageAsset(BaseModel):
    """
    An image in the automation library.

    Images are stored in base64 format within the configuration and
    referenced by ID from StateImages and Patterns.

    Supports S3 storage with presigned URLs and versioning.
    """

    id: str = Field(..., description="Unique identifier for the image")
    name: str = Field(..., description="Human-readable name")
    data: str = Field(..., description="Base64 encoded image data")
    format: ImageFormat = Field(..., description="Image format (png, jpg, jpeg)")
    width: int = Field(..., gt=0, description="Image width in pixels")
    height: int = Field(..., gt=0, description="Image height in pixels")
    hash: str | None = Field(
        default=None,
        description="SHA256 hash for integrity verification",
    )
    mask: str | None = Field(
        default=None,
        description="Optional base64 encoded mask image",
    )

    # S3 storage fields
    s3_key: str | None = Field(
        default=None,
        alias="s3Key",
        description="S3 object key for cloud storage",
    )
    url_expires_at: str | None = Field(
        default=None,
        alias="urlExpiresAt",
        description="ISO 8601 timestamp when presigned URL expires",
    )

    # Versioning support
    version: int | None = Field(
        default=None,
        description="Version number (default: 1)",
    )
    parent_image_id: str | None = Field(
        default=None,
        alias="parentImageId",
        description="ID of the original image if this is a version",
    )
    versions: list[str] | None = Field(
        default=None,
        description="Array of version IDs (only on parent images)",
    )

    # Monitor assignment
    monitors: list[int] | None = Field(
        default=None,
        description="Monitor indices where this image should be used (default: [0])",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Config Metadata
# =============================================================================


class CompatibleVersions(BaseModel):
    """Version compatibility information."""

    runner: str = Field(..., description="Compatible runner version")
    website: str = Field(..., description="Compatible website version")

    model_config = {"populate_by_name": True}


class ConfigMetadata(BaseModel):
    """Metadata about the automation configuration."""

    name: str = Field(..., description="Project/configuration name")
    description: str | None = Field(default=None, description="Description")
    author: str | None = Field(default=None, description="Author name")
    created: str = Field(..., description="ISO 8601 creation timestamp")
    modified: str = Field(..., description="ISO 8601 last modified timestamp")
    tags: list[str] = Field(default_factory=list, description="Tags for categorization")
    target_application: str | None = Field(
        default=None,
        alias="targetApplication",
        description="Target application being automated",
    )
    compatible_versions: CompatibleVersions | None = Field(
        default=None,
        alias="compatibleVersions",
        description="Version compatibility information",
    )
    project_id: str | None = Field(
        default=None,
        alias="projectId",
        description="Project ID from qontinui-web for test run reporting",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Config Settings
# =============================================================================


class Resolution(BaseModel):
    """Screen resolution."""

    width: int = Field(..., gt=0, description="Width in pixels")
    height: int = Field(..., gt=0, description="Height in pixels")


class ExecutionSettings(BaseModel):
    """Execution control settings."""

    default_timeout: int = Field(
        default=30000,
        alias="defaultTimeout",
        description="Default action timeout in milliseconds",
    )
    default_retry_count: int = Field(
        default=3,
        alias="defaultRetryCount",
        description="Default number of retry attempts",
    )
    action_delay: int = Field(
        default=100,
        alias="actionDelay",
        description="Delay between actions in milliseconds",
    )
    failure_strategy: FailureStrategy = Field(
        default=FailureStrategy.STOP,
        alias="failureStrategy",
        description="How to handle action failures",
    )
    headless: bool = Field(
        default=False,
        description="Run in headless mode",
    )
    resolution: Resolution | None = Field(
        default=None,
        description="Target screen resolution",
    )

    model_config = {"populate_by_name": True}


class RecognitionSettings(BaseModel):
    """Image recognition settings."""

    default_threshold: float = Field(
        default=0.8,
        ge=0.0,
        le=1.0,
        alias="defaultThreshold",
        description="Default similarity threshold",
    )
    search_algorithm: SearchAlgorithm = Field(
        default=SearchAlgorithm.TEMPLATE_MATCHING,
        alias="searchAlgorithm",
        description="Image search algorithm",
    )
    multi_scale_search: bool = Field(
        default=False,
        alias="multiScaleSearch",
        description="Enable multi-scale search",
    )
    color_space: ColorSpace = Field(
        default=ColorSpace.RGB,
        alias="colorSpace",
        description="Color space for matching",
    )
    edge_detection: bool = Field(
        default=False,
        alias="edgeDetection",
        description="Enable edge detection preprocessing",
    )
    ocr_enabled: bool = Field(
        default=False,
        alias="ocrEnabled",
        description="Enable OCR text recognition",
    )
    ocr_language: str = Field(
        default="eng",
        alias="ocrLanguage",
        description="OCR language code",
    )

    model_config = {"populate_by_name": True}


class LoggingSettings(BaseModel):
    """Logging configuration."""

    level: LogLevel = Field(default=LogLevel.INFO, description="Log level")
    screenshot_on_error: bool = Field(
        default=True,
        alias="screenshotOnError",
        description="Capture screenshot on errors",
    )
    log_file: str | None = Field(
        default=None,
        alias="logFile",
        description="Log file path",
    )
    console_output: bool = Field(
        default=True,
        alias="consoleOutput",
        description="Output logs to console",
    )
    detailed_matching: bool = Field(
        default=False,
        alias="detailedMatching",
        description="Log detailed matching information",
    )

    model_config = {"populate_by_name": True}


class PerformanceSettings(BaseModel):
    """Performance tuning settings."""

    max_parallel_actions: int = Field(
        default=1,
        ge=1,
        alias="maxParallelActions",
        description="Maximum parallel action execution",
    )
    cpu_limit: int | None = Field(
        default=None,
        ge=1,
        le=100,
        alias="cpuLimit",
        description="CPU usage limit percentage",
    )
    memory_limit: int | None = Field(
        default=None,
        gt=0,
        alias="memoryLimit",
        description="Memory limit in MB",
    )
    cache_images: bool = Field(
        default=True,
        alias="cacheImages",
        description="Cache images in memory",
    )
    optimize_search: bool = Field(
        default=True,
        alias="optimizeSearch",
        description="Enable search optimization",
    )

    model_config = {"populate_by_name": True}


class MouseActionSettings(BaseModel):
    """Mouse action timing settings."""

    click_hold_duration: float = Field(
        default=0.05,
        description="Duration to hold mouse button",
    )
    click_release_delay: float = Field(
        default=0.05,
        description="Delay after releasing mouse button",
    )
    click_safety_release: bool = Field(
        default=True,
        description="Ensure mouse button is released",
    )
    double_click_interval: float = Field(
        default=0.1,
        description="Interval between double-click clicks",
    )
    drag_start_delay: float = Field(
        default=0.1,
        description="Delay before starting drag",
    )
    drag_end_delay: float = Field(
        default=0.1,
        description="Delay after ending drag",
    )
    drag_default_duration: float = Field(
        default=0.5,
        description="Default drag duration",
    )
    move_default_duration: float = Field(
        default=0.3,
        description="Default move duration",
    )
    safety_release_delay: float = Field(
        default=0.1,
        description="Safety release delay",
    )

    model_config = {"populate_by_name": True}


class KeyboardActionSettings(BaseModel):
    """Keyboard action timing settings."""

    key_hold_duration: float = Field(
        default=0.05,
        description="Duration to hold key",
    )
    key_release_delay: float = Field(
        default=0.05,
        description="Delay after releasing key",
    )
    typing_interval: float = Field(
        default=0.05,
        description="Interval between typed characters",
    )
    hotkey_hold_duration: float = Field(
        default=0.1,
        description="Duration to hold hotkey combination",
    )
    hotkey_press_interval: float = Field(
        default=0.05,
        description="Interval between hotkey presses",
    )

    model_config = {"populate_by_name": True}


class FindActionSettings(BaseModel):
    """Find action default settings."""

    default_timeout: int = Field(
        default=10000,
        description="Default find timeout in milliseconds",
    )
    default_retry_count: int = Field(
        default=3,
        description="Default retry count",
    )
    search_interval: int = Field(
        default=200,
        description="Search polling interval in milliseconds",
    )

    model_config = {"populate_by_name": True}


class WaitActionSettings(BaseModel):
    """Wait action default settings."""

    pause_before_action: float = Field(
        default=0.0,
        description="Default pause before action",
    )
    pause_after_action: float = Field(
        default=0.0,
        description="Default pause after action",
    )

    model_config = {"populate_by_name": True}


class ConfigSettings(BaseModel):
    """Complete settings configuration."""

    execution: ExecutionSettings = Field(
        default_factory=ExecutionSettings,
        description="Execution control settings",
    )
    recognition: RecognitionSettings = Field(
        default_factory=RecognitionSettings,
        description="Image recognition settings",
    )
    logging: LoggingSettings | None = Field(
        default=None,
        description="Logging configuration",
    )
    performance: PerformanceSettings | None = Field(
        default=None,
        description="Performance tuning settings",
    )
    mouse: MouseActionSettings | None = Field(
        default=None,
        description="Mouse action timing",
    )
    keyboard: KeyboardActionSettings | None = Field(
        default=None,
        description="Keyboard action timing",
    )
    find: FindActionSettings | None = Field(
        default=None,
        description="Find action defaults",
    )
    wait: WaitActionSettings | None = Field(
        default=None,
        description="Wait action defaults",
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Scheduler Types
# =============================================================================


class TriggerType(str, Enum):
    """Schedule trigger types."""

    TIME = "TIME"
    INTERVAL = "INTERVAL"
    STATE = "STATE"
    MANUAL = "MANUAL"


class CheckMode(str, Enum):
    """State check modes."""

    CHECK_ALL = "CHECK_ALL"
    CHECK_INACTIVE_ONLY = "CHECK_INACTIVE_ONLY"


class ScheduleType(str, Enum):
    """Schedule timing types."""

    FIXED_RATE = "FIXED_RATE"
    FIXED_DELAY = "FIXED_DELAY"


class Schedule(BaseModel):
    """Automated workflow schedule."""

    id: str = Field(..., description="Unique identifier")
    name: str = Field(..., description="Schedule name")
    workflow_id: str = Field(..., alias="workflowId", description="Workflow to run")
    description: str | None = Field(default=None, description="Description")
    trigger_type: TriggerType = Field(
        ..., alias="triggerType", description="Trigger type"
    )
    check_mode: CheckMode = Field(
        ..., alias="checkMode", description="State check mode"
    )
    schedule_type: ScheduleType = Field(
        ..., alias="scheduleType", description="Schedule type"
    )
    cron_expression: str | None = Field(
        default=None, alias="cronExpression", description="Cron expression for TIME"
    )
    interval_seconds: int | None = Field(
        default=None, alias="intervalSeconds", description="Interval in seconds"
    )
    trigger_state: str | None = Field(
        default=None, alias="triggerState", description="State that triggers execution"
    )
    max_iterations: int | None = Field(
        default=None, alias="maxIterations", description="Max iterations"
    )
    state_check_delay_seconds: int = Field(
        default=5, alias="stateCheckDelaySeconds", description="Delay between checks"
    )
    state_rebuild_delay_seconds: int = Field(
        default=30,
        alias="stateRebuildDelaySeconds",
        description="Delay before rebuilding",
    )
    failure_threshold: int = Field(
        default=3, alias="failureThreshold", description="Failures before action"
    )
    enabled: bool = Field(default=True, description="Is schedule enabled")
    created_at: str | None = Field(
        default=None, alias="createdAt", description="Creation timestamp"
    )
    last_executed_at: str | None = Field(
        default=None, alias="lastExecutedAt", description="Last execution timestamp"
    )

    model_config = {"populate_by_name": True}


class ExecutionRecord(BaseModel):
    """Record of a schedule execution."""

    id: str = Field(..., description="Unique identifier")
    schedule_id: str = Field(..., alias="scheduleId", description="Schedule ID")
    workflow_id: str = Field(..., alias="workflowId", description="Workflow ID")
    start_time: str = Field(..., alias="startTime", description="Start timestamp")
    end_time: str | None = Field(
        default=None, alias="endTime", description="End timestamp"
    )
    success: bool = Field(..., description="Whether execution succeeded")
    iteration_count: int = Field(
        default=0, alias="iterationCount", description="Number of iterations"
    )
    errors: list[str] = Field(default_factory=list, description="Error messages")
    metadata: dict[str, object] = Field(
        default_factory=dict, description="Additional metadata"
    )

    model_config = {"populate_by_name": True}


# =============================================================================
# Root Configuration
# =============================================================================


class QontinuiConfig(BaseModel):
    """
    Root configuration for Qontinui automation.

    This is the complete exported configuration that can be consumed by
    the qontinui-runner and qontinui library. It contains all images,
    workflows, states, transitions, and settings needed for automation.
    """

    version: str = Field(
        ...,
        description="Configuration schema version (semver)",
        pattern=r"^\d+\.\d+\.\d+$",
    )
    metadata: ConfigMetadata = Field(
        ...,
        description="Configuration metadata",
    )
    images: list[ImageAsset] = Field(
        default_factory=list,
        description="Image library",
    )
    workflows: list[Workflow] = Field(
        default_factory=list,
        description="Workflow definitions",
    )
    states: list[State] = Field(
        default_factory=list,
        description="State machine states",
    )
    transitions: list[Transition] = Field(
        default_factory=list,
        description="State transitions",
    )
    categories: list[Category] = Field(
        default_factory=list,
        description="Workflow categories with automation control",
    )
    settings: ConfigSettings | None = Field(
        default=None,
        description="Configuration settings",
    )
    schedules: list[Schedule] | None = Field(
        default=None,
        description="Automated schedules",
    )
    execution_records: list[ExecutionRecord] | None = Field(
        default=None,
        alias="executionRecords",
        description="Execution history",
    )

    model_config = {"populate_by_name": True}
