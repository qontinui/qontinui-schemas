"""
Template Capture schemas for the click-to-template system.

These schemas represent template candidates extracted from click events
during capture sessions, and application profiles for optimized detection.

Used by:
- qontinui library (produces candidates)
- qontinui-runner (orchestrates processing, sends to web)
- qontinui-web (stores, displays, allows review)
"""

from .models import (
    ApplicationProfile,
    ApplicationProfileCreate,
    ApplicationProfileListResponse,
    ApplicationProfileResponse,
    ApplicationProfileUpdate,
    ApprovedTemplateData,
    CandidateBoundingBox,
    CandidateStatus,
    DetectionStrategyType,
    ElementType,
    GenerateStateMachineRequest,
    GenerateStateMachineResponse,
    GroupingMethod,
    InferenceConfigSchema,
    StateDefResponse,
    StateImageDefResponse,
    StateMachineConfigResponse,
    TemplateCandidateBatchCreate,
    TemplateCandidateCreate,
    TemplateCandidateDetail,
    TemplateCandidateListResponse,
    TemplateCandidateResponse,
    TemplateCandidateSummary,
    TemplateCandidateUpdate,
    TransitionDefResponse,
    TuningMetrics,
    TuningRequest,
    TuningResult,
)

__all__ = [
    # Enums
    "ElementType",
    "DetectionStrategyType",
    "CandidateStatus",
    "GroupingMethod",
    # Bounding box
    "CandidateBoundingBox",
    # Template candidate schemas
    "TemplateCandidateCreate",
    "TemplateCandidateBatchCreate",
    "TemplateCandidateResponse",
    "TemplateCandidateDetail",
    "TemplateCandidateSummary",
    "TemplateCandidateUpdate",
    "TemplateCandidateListResponse",
    # Application profile schemas
    "InferenceConfigSchema",
    "TuningMetrics",
    "ApplicationProfileCreate",
    "ApplicationProfileResponse",
    "ApplicationProfileUpdate",
    "ApplicationProfile",
    "ApplicationProfileListResponse",
    # Tuning
    "TuningRequest",
    "TuningResult",
    # State machine generation
    "ApprovedTemplateData",
    "GenerateStateMachineRequest",
    "GenerateStateMachineResponse",
    "StateImageDefResponse",
    "StateDefResponse",
    "TransitionDefResponse",
    "StateMachineConfigResponse",
]
