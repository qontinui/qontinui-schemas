//! Pipeline stages: operations on [`Frame`](crate::Frame) plus terminal byte-emitting stages.

use crate::contract::{EncodedFormat, OutputContract};
use crate::frame::Region;

/// A single pipeline stage. Most stages are pure `Frame -> Frame`. The terminal
/// stages [`Stage::Encode`] and [`Stage::Verify`] cross over to bytes.
#[derive(Debug, Clone)]
pub enum Stage {
    /// Crop to a pixel-space rectangle. Errors with
    /// [`CropOutOfBounds`](crate::VisionError::CropOutOfBounds) if the region
    /// extends beyond the frame.
    CropRegion(Region),
    /// Resize using the given strategy (Lanczos3 filter).
    Resize(ResizeStrategy),
    /// Composite RGBA over an opaque RGB background. The resulting frame is
    /// still RGBA-shaped but every alpha byte is `0xFF`.
    FlattenAlpha([u8; 3]),
    /// Draw rectangle outlines onto the frame. (Phase 1: outlines only; text
    /// labels are recorded on each annotation but not rendered — typography
    /// integration lands in Phase 6.)
    Annotate(Vec<Annotation>),
    /// Apply per-region pixel obfuscation (blur, pixelate, fill).
    Redact(Vec<RedactRegion>),
    /// Marker stage: requests metadata stripping. Actual stripping happens
    /// post-`Encode` in [`Pipeline::run`](crate::Pipeline::run).
    StripMetadata,
    /// Encode the current frame to bytes in the given format. Must precede
    /// [`Stage::Verify`] when one is present.
    Encode(EncodedFormat),
    /// Round-trip the emitted bytes through the decoder and assert the contract.
    /// Must be the terminal stage when present.
    Verify(OutputContract),
}

#[derive(Debug, Clone, Copy)]
pub enum ResizeStrategy {
    /// If `max(w, h) > n`, scale down preserving aspect ratio. Otherwise no-op.
    LongEdge(u32),
    /// Multiply both dimensions by `s` (Lanczos3).
    Scale(f64),
    /// Fit inside `(w, h)` preserving aspect ratio.
    Fit { w: u32, h: u32 },
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub region: Region,
    pub label: Option<String>,
    pub style: AnnotationStyle,
}

#[derive(Debug, Clone, Copy)]
pub struct AnnotationStyle {
    pub border_color: [u8; 4],
    pub border_thickness: u32,
    pub fill_color: Option<[u8; 4]>,
    pub label_color: [u8; 4],
}

impl AnnotationStyle {
    /// Conventional default: 2 px red border, no fill, red label.
    pub const DEFAULT_RED: AnnotationStyle = AnnotationStyle {
        border_color: [0xFF, 0x33, 0x33, 0xFF],
        border_thickness: 2,
        fill_color: None,
        label_color: [0xFF, 0x33, 0x33, 0xFF],
    };
}

#[derive(Debug, Clone, Copy)]
pub struct RedactRegion {
    pub region: Region,
    pub kind: RedactKind,
}

#[derive(Debug, Clone, Copy)]
pub enum RedactKind {
    /// Box blur with the given sigma (radius derived as `max(1, sigma)`).
    Blur { sigma: f32 },
    /// Pixelate via downscale-then-upscale with the given block size.
    Pixelate { block_size: u32 },
    /// Replace pixels with a solid color.
    Fill([u8; 4]),
}
