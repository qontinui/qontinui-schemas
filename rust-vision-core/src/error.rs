//! Structured error type for the vision pipeline.

use crate::Region;
use thiserror::Error;

/// Every failure mode the pipeline can produce.
#[derive(Debug, Error)]
pub enum VisionError {
    /// The pipeline produced bytes that violated its declared contract.
    /// The terminal [`Stage::Verify`](crate::Stage::Verify) stage emits this.
    #[error("contract violation in stage {stage}: {reason}")]
    ContractViolation { stage: &'static str, reason: String },

    /// Encoding the frame to bytes failed (I/O or codec).
    #[error("encode failed: {0}")]
    EncodeFailed(String),

    /// Decoding the emitted bytes during verify failed.
    #[error("decode failed: {0}")]
    DecodeFailed(String),

    /// Pipeline was constructed with an invalid stage sequence.
    /// E.g. [`Verify`](crate::Stage::Verify) without a preceding
    /// [`Encode`](crate::Stage::Encode), or [`Verify`] not in terminal position.
    #[error("invalid pipeline: {0}")]
    InvalidPipeline(String),

    /// [`CropRegion`](crate::Stage::CropRegion) referenced pixels outside the frame.
    #[error(
        "crop region ({}, {}) {}x{} out of frame bounds ({}x{})",
        region.x,
        region.y,
        region.w,
        region.h,
        frame.0,
        frame.1
    )]
    CropOutOfBounds { region: Region, frame: (u32, u32) },

    /// I/O error in the disk cache layer.
    #[error("cache I/O: {0}")]
    CacheIo(String),
}
