//! `qontinui-vision-core` — image pipeline + output contract.
//!
//! Composable image stages that produce bytes guaranteed to satisfy a declared
//! [`OutputContract`] (e.g. Claude vision: long-edge ≤ 1568, JPEG/WebP, no alpha,
//! no metadata, ≤ 5 MB). The terminal [`Stage::Verify`] stage round-trips the
//! produced bytes back through [`image::load_from_memory`] and asserts the
//! contract — pipelines cannot emit bytes that fail their own contract.
//!
//! Phase 1 of the UI Bridge vision pipeline plan. Standalone crate with no
//! dependency on `qontinui-types` or `qontinui-runner-client`.

#![deny(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

pub mod analyzers;
pub mod assertions;
pub mod cache;
pub mod contract;
pub mod element_snapshot;
pub mod encode;
pub mod error;
pub mod frame;
pub mod pipeline;
pub mod stage;
pub mod strip;

pub use analyzers::{AnalyzeInput, Analyzer, Finding, Severity};
pub use assertions::{
    evaluate as evaluate_assertion, Assertion, AssertionResult, BaselineEntry, EvalContext,
    OcrBlockRef, TextMatchKind, TextTarget, TypographyDimension, WcagLevel,
};
pub use cache::{sha256_of, CacheHit, CacheStats, VisionCache};
pub use contract::{AlphaPolicy, ColorSpace, EncodedFormat, MetadataPolicy, OutputContract};
pub use element_snapshot::{
    intersection, region_contains, regions_overlap, Element, ElementSnapshot, Rgb,
};
pub use error::VisionError;
pub use frame::{Frame, FrameSource, FrameSourceKind, Region};
pub use pipeline::{multi_run, MultiOutput, Pipeline};
pub use stage::{Annotation, AnnotationStyle, RedactKind, RedactRegion, ResizeStrategy, Stage};
