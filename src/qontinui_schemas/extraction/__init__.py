"""
Extraction schemas for web extraction results.

This module provides shared data structures for:
- Extraction session management
- State and element annotations
- Transition discovery
- Import/export operations
"""

from .models import (  # Enums; Basic types; Elements; States; Transitions; Stats; Annotations; Session; Import
    BoundingBox,
    ElementAnnotation,
    ExtractedElement,
    ExtractionAnnotation,
    ExtractionSession,
    ExtractionSessionConfig,
    ExtractionSessionDetail,
    ExtractionStats,
    ExtractionStatus,
    ImportResult,
    InferredTransition,
    StateAnnotation,
    StateImportRequest,
    StateType,
    TriggerType,
)

__all__ = [
    # Enums
    "ExtractionStatus",
    "StateType",
    "TriggerType",
    # Basic types
    "BoundingBox",
    # Elements
    "ExtractedElement",
    "ElementAnnotation",
    # States
    "StateAnnotation",
    # Transitions
    "InferredTransition",
    # Stats
    "ExtractionStats",
    # Annotations
    "ExtractionAnnotation",
    # Session
    "ExtractionSessionConfig",
    "ExtractionSession",
    "ExtractionSessionDetail",
    # Import
    "StateImportRequest",
    "ImportResult",
]
