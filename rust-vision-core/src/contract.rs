//! Declarative output contracts. An [`OutputContract`] describes the constraints
//! a downstream consumer (e.g. Claude's vision API) places on emitted image bytes.
//!
//! Pre-defined contracts:
//!   - [`OutputContract::CLAUDE_VISION_V1`] — JPEG/WebP, ≤ 1568 long-edge, no alpha,
//!     no metadata, ≤ 5 MB.
//!   - [`OutputContract::PNG_STRICT`] — PNG only, alpha preserved, no metadata.
//!   - [`OutputContract::WEBP_LOSSY`] — WebP only (lossy semantically; the `image`
//!     crate's WebP encoder is lossless-only — see crate docs), no alpha, no metadata.

#[derive(Debug, Clone, Copy)]
pub struct OutputContract {
    pub name: &'static str,
    pub max_long_edge: u32,
    pub allowed_formats: &'static [EncodedFormat],
    pub max_bytes: usize,
    pub alpha_policy: AlphaPolicy,
    pub metadata_policy: MetadataPolicy,
    pub color_space: ColorSpace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodedFormat {
    Jpeg { quality: u8 },
    Webp { quality: u8, lossy: bool },
    Png,
}

impl EncodedFormat {
    /// Loose match: same family ignoring quality/lossy detail. We don't have a way
    /// to recover quality from emitted bytes; verifying format-family is the only
    /// honest assertion.
    pub fn matches_family(self, other: EncodedFormat) -> bool {
        matches!(
            (self, other),
            (EncodedFormat::Jpeg { .. }, EncodedFormat::Jpeg { .. })
                | (EncodedFormat::Webp { .. }, EncodedFormat::Webp { .. })
                | (EncodedFormat::Png, EncodedFormat::Png)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaPolicy {
    Preserve,
    /// Composite RGBA over the given RGB background.
    Flatten([u8; 3]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataPolicy {
    Preserve,
    /// Strip all ancillary chunks/markers. PNG → keep IHDR/PLTE/IDAT/IEND/tRNS;
    /// JPEG → drop APPn/COM; WebP → keep VP8/VP8L only.
    StripAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
    AdobeRgb,
    P3,
}

impl OutputContract {
    /// Claude vision: long edge ≤ 1568, JPEG/WebP, white-flattened alpha,
    /// metadata stripped, ≤ 5 MB.
    pub const CLAUDE_VISION_V1: OutputContract = OutputContract {
        name: "claude_vision_v1",
        max_long_edge: 1568,
        allowed_formats: &[
            EncodedFormat::Jpeg { quality: 85 },
            EncodedFormat::Webp {
                quality: 85,
                lossy: true,
            },
        ],
        max_bytes: 5 * 1024 * 1024,
        alpha_policy: AlphaPolicy::Flatten([0xFF, 0xFF, 0xFF]),
        metadata_policy: MetadataPolicy::StripAll,
        color_space: ColorSpace::Srgb,
    };

    /// PNG-only contract. Preserves alpha; no resize cap; metadata stripped.
    /// Use for callers that need lossless output (regression baselines, golden
    /// fixtures) but still want the metadata strip + verification guarantees.
    pub const PNG_STRICT: OutputContract = OutputContract {
        name: "png_strict",
        max_long_edge: u32::MAX,
        allowed_formats: &[EncodedFormat::Png],
        max_bytes: usize::MAX,
        alpha_policy: AlphaPolicy::Preserve,
        metadata_policy: MetadataPolicy::StripAll,
        color_space: ColorSpace::Srgb,
    };

    /// WebP-only contract. Semantically "lossy" — but note the `image` crate's
    /// WebP encoder is lossless-only in 0.25, so emitted bytes are lossless WebP.
    /// `Verify` accepts any WebP family member regardless.
    pub const WEBP_LOSSY: OutputContract = OutputContract {
        name: "webp_lossy",
        max_long_edge: 1568,
        allowed_formats: &[EncodedFormat::Webp {
            quality: 85,
            lossy: true,
        }],
        max_bytes: 5 * 1024 * 1024,
        alpha_policy: AlphaPolicy::Flatten([0xFF, 0xFF, 0xFF]),
        metadata_policy: MetadataPolicy::StripAll,
        color_space: ColorSpace::Srgb,
    };
}
