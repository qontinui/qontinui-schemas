//! Convenience entry point: build a [`Pipeline`](crate::Pipeline) from an
//! [`OutputContract`](crate::OutputContract) and run it.
//!
//! Most callers should use [`safe_image`] rather than building pipelines by hand.

use crate::contract::{AlphaPolicy, EncodedFormat, MetadataPolicy, OutputContract};
use crate::error::VisionError;
use crate::frame::Frame;
use crate::pipeline::Pipeline;
use crate::stage::{ResizeStrategy, Stage};

/// Build a pipeline that satisfies `contract` and run it against `frame`.
///
/// The pipeline composed is:
///   1. [`Stage::FlattenAlpha`] iff `contract.alpha_policy` is
///      [`AlphaPolicy::Flatten`].
///   2. [`Stage::Resize`] with [`ResizeStrategy::LongEdge`] iff the frame's
///      long edge exceeds `contract.max_long_edge`.
///   3. [`Stage::StripMetadata`] iff `contract.metadata_policy` is
///      [`MetadataPolicy::StripAll`].
///   4. [`Stage::Encode`] using the first entry in `contract.allowed_formats`.
///   5. [`Stage::Verify`] with the contract.
///
/// Returns the verified bytes, or [`VisionError::ContractViolation`] if Verify
/// rejects them.
pub fn safe_image(frame: Frame, contract: &OutputContract) -> Result<Vec<u8>, VisionError> {
    let format = *contract.allowed_formats.first().ok_or_else(|| {
        VisionError::InvalidPipeline(format!(
            "contract {:?} has no allowed_formats",
            contract.name
        ))
    })?;

    let mut pipeline = Pipeline::new();

    if let AlphaPolicy::Flatten(bg) = contract.alpha_policy {
        pipeline = pipeline.push(Stage::FlattenAlpha(bg));
    }

    let long_edge = frame.width.max(frame.height);
    if long_edge > contract.max_long_edge {
        pipeline = pipeline.push(Stage::Resize(ResizeStrategy::LongEdge(
            contract.max_long_edge,
        )));
    }

    if matches!(contract.metadata_policy, MetadataPolicy::StripAll) {
        pipeline = pipeline.push(Stage::StripMetadata);
    }

    pipeline = pipeline
        .push(Stage::Encode(format))
        .push(Stage::Verify(*contract));

    pipeline.run(frame)
}

/// Re-export of the default encoded format selection logic, exposed so callers
/// (or tests) can inspect what `safe_image` *would* do without running it.
pub fn default_format(contract: &OutputContract) -> Option<EncodedFormat> {
    contract.allowed_formats.first().copied()
}
