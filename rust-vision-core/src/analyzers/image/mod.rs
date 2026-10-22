//! The `image` analyzer: L1 pixel VALUES for every image on a page (plan
//! 2026-10-13-image-generation-and-visual-judgment, Phase 15).
//!
//! For each snapshot element carrying [`crate::ElementMedia`] it reads the
//! element's crop of the frame — the bbox mapped onto buffer pixels through
//! [`crate::FrameSource::scale_factor`] — and measures resolution, sharpness,
//! compression blockiness, an OKLab palette with a photographic/flat class,
//! busyness (also under any text laid over the image), pHash and dHash, the
//! placeholder signals, the legibility of its edge and stroke pixels against
//! what surrounds it, the part of the natural image left visible under
//! object-fit, spectral-residual saliency, and — given the frame's
//! text-hidden twin — the WCAG contrast of each text element laid over it
//! ([`text_contrast`], Phase 16). For the page it measures
//! colourfulness, clutter and empty space. Every value is an
//! [`Observation`] with provenance, carried on
//! [`super::AnalyzerResult::image_values`].
//!
//! # Values, never verdicts
//!
//! This analyzer must never gate a build through `vision-audit analyze`, which
//! the style gate runs as `--analyzer all --fail-on critical`. So:
//!
//! - it emits only [`Severity::Info`] findings;
//! - it answers `Blocked` only when the FRAME is missing (the dispatcher's
//!   arm, the one input without which nothing here can be read);
//! - a snapshot with no media element, or no snapshot at all, is
//!   `Degraded{input_missing}`, which is green — an image-free route never
//!   trips exit 2.
//!
//! A check gates only as an L1 assertion ([`crate::assertions`]'s
//! `image_palette_within`, `no_near_duplicate_images`, …), each of which
//! takes the thresholds this analyzer cannot know as parameters (PD3).
//!
//! # Confidence
//!
//! A value read from the frame is an ESTIMATE of the image: the frame shows
//! the image as composited on the page, clipped by the viewport and painted
//! over by whatever sits on top of it. Each such observation therefore carries
//! [`Provenance::confidence`] = [`Crop::confidence`], the share of the
//! image that was inside the frame and NOT covered by another element's box
//! (text, controls, other media, the element recorded in `occluded_by`) — how
//! much of what was read is the image itself. Values derived from the
//! snapshot's declared media facts alone (effective resolution, aspect
//! distortion, the visible region) are deductions and state `None`.
//!
//! # What is read
//!
//! The crop is the element's box clipped to where the natural image is DRAWN
//! ([`drawn_rect`]), so a `contain` letterbox is not read as image. That
//! applies to the REPLACED kinds (img, picture, video, canvas), the only ones
//! with an object-fit; an inline SVG or a `role=img` element paints its own
//! box, which is read whole; a CSS background is unreadable (its
//! background-size, -position and -repeat are not on the wire). When the
//! fit may letterbox and the drawn rect cannot be established, the image is
//! not read at all: its pixel values are unknown, as is every assertion over
//! it. An element that paints NOTHING — a zero-area box, or a producer-stated
//! `hidden` / `no-layout` — is UNRENDERED ([`unrendered`]): its pixel values
//! are unknown and it is left out of the page-wide checks. An element that
//! may paint but cannot be attributed — reported `occluded` (a transparent
//! overlay reads the same) or `off-screen`, `visible: false` with no reason,
//! or wholly inside its recorded occluder — is UNREADABLE ([`unreadable`]): a
//! gap, so the analyzer degrades and page-wide checks answer unknown.

pub mod pixels;
pub mod text_contrast;

pub use text_contrast::{GlyphColourSource, TextOverImageContrast};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use ::image::{GrayImage, RgbImage};

use super::{Analyzer, AnalyzerResult, Finding, Severity};
use crate::coverage::SnapshotCoverage;
use crate::element_media::{ElementMedia, MediaKind, ObjectFit};
use crate::element_snapshot::{
    intersection, region_contains, Element, ElementSnapshot, Rgb, VisibilityReason,
};
use crate::frame::{Frame, Region};
use crate::observation::{
    Observation, ObservationCoverage, Producer, Provenance, UnknownCode, UnmeasuredDimension,
};

// ---------------------------------------------------------------------------
// Value types
// ---------------------------------------------------------------------------

/// Everything the image analyzer measured on one frame. Carried on
/// [`AnalyzerResult::image_values`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageValues {
    /// One record per snapshot element carrying `media`, in snapshot order.
    pub images: Vec<ImageElementValues>,
    /// Page-level values over the whole frame.
    pub page: PageImageValues,
    /// Every pair of hashed images with its pHash and dHash distances. Left
    /// out, and not counted as considered: flat fills (a flat image's pHash is
    /// degenerate, and every pair of placeholders would read as a duplicate —
    /// `no_placeholder_images` owns them); unrendered images (see
    /// [`unrendered`]); STACKED pairs ([`pair_is_one_stack`]), whose two
    /// images share one place and one of which the producer reports covered,
    /// so their pixels are the top one's twice (counted in
    /// `provenance.source.stackedPairsSkipped`); and NESTED images
    /// ([`drop_nested`]): a wrapper whose descendant is also hashed is dropped
    /// and its children kept (counted in `nestedImagesSkipped`). Images under
    /// slot size ([`is_slot_sized`]) are icons, not compared either (counted
    /// in `smallImagesSkipped`).
    pub near_duplicates: Observation<Vec<NearDuplicatePair>>,
}

/// The per-image values. Each field is its own observation, so one input the
/// producer did not measure (say `natural_px`) makes the values that need it
/// `unknown` and leaves the rest measured.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageElementValues {
    /// The snapshot element id.
    pub element: String,
    /// Source pixels per displayed DEVICE pixel (see
    /// [`ElementMedia::effective_resolution`]): the producer's own value when
    /// it measured one, else computed from `natural_px`, `rendered_px`,
    /// `object_fit` and the frame's DPR.
    pub effective_resolution: Observation<f64>,
    /// Relative aspect distortion, `|rendered aspect ÷ natural aspect − 1|`.
    /// Only `object-fit: fill` stretches; every other fit preserves the
    /// aspect and reads 0.
    pub aspect_distortion: Observation<f64>,
    /// Variance of the Laplacian over the crop, at the rendered scale.
    pub sharpness: Observation<f64>,
    /// [`pixels::blockiness`] over the crop, the coding grid placed at the
    /// rendered scale of the natural image's 8-pixel blocks.
    pub blockiness: Observation<f64>,
    /// Up to five OKLab clusters, largest first.
    pub palette: Observation<Vec<PaletteColour>>,
    /// Photographic or flat, with the colour-bin count it rests on.
    pub class: Observation<ImageClassValue>,
    /// Edge density and entropy of the crop, and of the part under each text
    /// element laid over it.
    pub busyness: Observation<Busyness>,
    /// pHash and dHash of the crop.
    pub hashes: Observation<ImageHashes>,
    /// The placeholder signals and their verdict.
    pub placeholder: Observation<PlaceholderValues>,
    /// WCAG contrast ratio of the image's edge and stroke pixels against the
    /// surround — the legibility of a transparent image on THIS frame's
    /// background (see [`edge_contrast`]).
    pub edge_contrast: Observation<f64>,
    /// The part of the natural image left visible under object-fit and
    /// object-position, in natural-frame fractions.
    pub visible_region: Observation<NaturalRegion>,
    /// Mean spectral-residual saliency over the crop (0..=1, page maximum 1).
    pub saliency: Observation<f64>,
    /// The WCAG contrast of each text element laid over the image, measured
    /// on real pixels from the frame and its text-hidden twin
    /// ([`super::AnalyzeInput::text_hidden_frame`]) at the
    /// [`text_contrast::DEFAULT_PERCENTILE`]: see [`text_contrast`].
    ///
    /// `absent` when no text lies over the image; `unknown` with
    /// `needs_multi_frame_input` when text does and no text-hidden frame was
    /// supplied, and with `input_missing` when the one supplied cannot be
    /// paired or registered with the frame, or the image is unrendered or
    /// has no box. Computed from the image's BOX, not its crop, so a
    /// CSS-background hero and an occluded `<img>` under a scrim are measured
    /// too: the glyphs' background is read from the text-hidden frame,
    /// whatever painted it.
    ///
    /// `None` only on a payload written before Phase 16; this crate always
    /// states one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_contrast: Option<Observation<Vec<TextOverImageContrast>>>,
}

/// Page-level values over the whole frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageImageValues {
    /// Hasler–Süsstrunk colourfulness.
    pub colourfulness: Observation<f64>,
    /// Clutter, as the frame's edge density (fraction of edge pixels).
    pub clutter: Observation<f64>,
    /// Empty space: the fraction of 8-CSS-pixel tiles that are flat.
    pub white_space: Observation<f64>,
}

/// A rectangle in an image's NATURAL frame, as fractions (0 to 1) of its
/// width and height — the same wire shape as qontinui-types'
/// `ImageSlotRegion`, so a slot spec's `focal.region` is passed verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NaturalRegion {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl NaturalRegion {
    pub const WHOLE: NaturalRegion = NaturalRegion {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    fn right(&self) -> f64 {
        self.x + self.width
    }

    fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// Fraction (0..=1) of `self`'s area that lies inside `other`.
    pub fn fraction_inside(&self, other: &NaturalRegion) -> f64 {
        let area = self.width * self.height;
        if area <= 0.0 {
            return 0.0;
        }
        let w = (self.right().min(other.right()) - self.x.max(other.x)).max(0.0);
        let h = (self.bottom().min(other.bottom()) - self.y.max(other.y)).max(0.0);
        (w * h / area).clamp(0.0, 1.0)
    }
}

/// One palette cluster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaletteColour {
    /// The cluster centre, converted back to sRGB.
    pub rgb: Rgb,
    /// The cluster centre in OKLab, `[L, a, b]`.
    pub oklab: [f64; 3],
    /// Share (0..=1) of the crop's sampled pixels in this cluster.
    pub coverage: f64,
}

/// Photographic or flat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageClass {
    /// Continuous tone: hundreds of colour bins.
    Photographic,
    /// Illustration, icon, UI art: a few dozen colour bins at most.
    Flat,
}

/// [`ImageClass`] and the measurement it is decided from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageClassValue {
    pub class: ImageClass,
    /// How many 4-bit-per-channel colour bins cover 90% of the crop. The
    /// class boundary is [`FLAT_MAX_BINS`].
    pub colour_bins: u32,
}

/// The class boundary: at most this many colour bins is flat.
pub const FLAT_MAX_BINS: u32 = 32;

/// Busyness of a crop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Busyness {
    /// Fraction of edge pixels ([`pixels::edge_density`]).
    pub edge_density: f64,
    /// Luma entropy, bits (0..=8).
    pub entropy: f64,
    /// The same two values over the part of the image under each text
    /// element laid over it — where busyness costs legibility.
    pub under_text: Vec<TextOverlayBusyness>,
}

/// Busyness under one text element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextOverlayBusyness {
    pub element: String,
    pub edge_density: f64,
    pub entropy: f64,
}

/// pHash and dHash, as `imagehash` prints them (16 hex digits).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageHashes {
    pub phash: String,
    pub dhash: String,
}

/// The three placeholder signals and the verdict they add up to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceholderValues {
    /// The slot's own marker ([`ElementMedia::placeholder`]); `None` when the
    /// producer did not read it.
    pub marker: Option<bool>,
    /// The crop is a slot-sized RASTER low-entropy fill (see
    /// [`placeholder_values`] and [`is_low_entropy_fill`]).
    pub low_entropy_fill: bool,
    /// The known placeholder pattern the crop's pixels draw
    /// ([`pixels::is_crossed_box`]: `"crossed_box"`).
    pub known_pattern: Option<String>,
    /// Any signal set.
    pub placeholder: bool,
}

/// A pair of images and how far apart their hashes are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NearDuplicatePair {
    pub a: String,
    pub b: String,
    pub phash_distance: u32,
    pub dhash_distance: u32,
}

// ---------------------------------------------------------------------------
// Thresholds the analyzer itself owns (the ones it reports Info findings on;
// assertions take theirs as parameters)
// ---------------------------------------------------------------------------

/// pHash distance at or under which the analyzer reports an Info
/// `image_near_duplicate` finding. The assertion takes its own.
pub const NEAR_DUPLICATE_REPORT_HAMMING: u32 = 6;

/// Most pixels the palette reads from one crop (a regular subsample).
const PALETTE_SAMPLE: usize = 16_384;

// ---------------------------------------------------------------------------
// Where an image is drawn
// ---------------------------------------------------------------------------

/// Where the NATURAL image is drawn inside its element's box, in CSS px
/// relative to the box's top-left, and at what scale. Under `contain` it can
/// be smaller than the box (the rest is letterbox, page background, not
/// image); under `cover` larger (the overflow is cropped).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drawn {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// CSS px per source px, per axis (they differ only under `fill`).
    pub scale_x: f64,
    pub scale_y: f64,
}

/// One axis of a parsed `object-position`.
#[derive(Debug, Clone, Copy, PartialEq)]
enum PositionComponent {
    /// A fraction of the free space (`50%` → 0.5, `left` → 0).
    Fraction(f64),
    /// An offset in CSS px.
    Px(f64),
}

impl PositionComponent {
    fn offset(self, free: f64) -> f64 {
        match self {
            Self::Fraction(f) => free * f,
            Self::Px(v) => v,
        }
    }
}

/// Parse a computed `object-position` into its (x, y) components. Accepts one
/// or two of: a percentage, a px length, or a keyword. `None` for anything
/// else (calc(), four-value edge offsets, other units).
fn parse_object_position(s: &str) -> Option<(PositionComponent, PositionComponent)> {
    use PositionComponent::*;
    fn one(t: &str) -> Option<(PositionComponent, Option<char>)> {
        match t {
            "left" => Some((Fraction(0.0), Some('x'))),
            "right" => Some((Fraction(1.0), Some('x'))),
            "top" => Some((Fraction(0.0), Some('y'))),
            "bottom" => Some((Fraction(1.0), Some('y'))),
            "center" => Some((Fraction(0.5), None)),
            _ => {
                if let Some(p) = t.strip_suffix('%') {
                    p.parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .map(|v| (Fraction(v / 100.0), None))
                } else if let Some(p) = t.strip_suffix("px") {
                    p.parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .map(|v| (Px(v), None))
                } else if t == "0" {
                    Some((Px(0.0), None))
                } else {
                    None
                }
            }
        }
    }
    let toks: Vec<&str> = s.split_whitespace().collect();
    match toks.as_slice() {
        [a] => {
            let (c, axis) = one(a)?;
            Some(if axis == Some('y') {
                (Fraction(0.5), c)
            } else {
                (c, Fraction(0.5))
            })
        }
        [a, b] => {
            let (ca, aa) = one(a)?;
            let (cb, ab) = one(b)?;
            if aa == Some('y') || ab == Some('x') {
                Some((cb, ca))
            } else {
                Some((ca, cb))
            }
        }
        _ => None,
    }
}

fn missing(what: &str, consequence: &str) -> LocateError {
    (
        UnknownCode::InputMissing,
        format!("the snapshot's media carries no `{what}`, so {consequence}"),
    )
}

fn parsed_position(
    media: &ElementMedia,
    consequence: &str,
) -> Result<(PositionComponent, PositionComponent), LocateError> {
    let p = media
        .object_position
        .as_deref()
        .ok_or_else(|| missing("object_position", consequence))?;
    parse_object_position(p).ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!("object_position {p:?} is not a form this crate parses, so {consequence}"),
        )
    })
}

/// The drawn scale of an image in a `box_w` × `box_h` CSS-px box, in CSS px
/// per source px per axis, from `natural_px` and `object_fit` alone — the
/// scale does not depend on `object-position`, only the placement does.
/// `none` and `scale-down` draw at the image's CSS natural size, which media
/// does not carry (`natural_px` is SOURCE pixels), so they are unknown here.
pub fn drawn_scale(
    media: &ElementMedia,
    box_w: f64,
    box_h: f64,
) -> Result<(f64, f64), LocateError> {
    const WHAT: &str = "the image's drawn scale is unknown";
    let fit = media
        .object_fit
        .ok_or_else(|| missing("object_fit", WHAT))?;
    let n = media
        .natural_px
        .ok_or_else(|| missing("natural_px", WHAT))?;
    let (nw, nh) = (n.w as f64, n.h as f64);
    let sizes = [nw, nh, box_w, box_h];
    if !sizes.iter().all(|v| v.is_finite() && *v > 0.0) {
        return Err((
            UnknownCode::InputMissing,
            format!("a zero or non-finite size, so {WHAT}"),
        ));
    }
    match fit {
        ObjectFit::Fill => Ok((box_w / nw, box_h / nh)),
        ObjectFit::Contain => {
            let s = (box_w / nw).min(box_h / nh);
            Ok((s, s))
        }
        ObjectFit::Cover => {
            let s = (box_w / nw).max(box_h / nh);
            Ok((s, s))
        }
        ObjectFit::None | ObjectFit::ScaleDown => Err((
            UnknownCode::InputMissing,
            format!(
                "object-fit none / scale-down draw at the CSS natural size, which media does not \
                 carry, so {WHAT}"
            ),
        )),
    }
}

/// [`Drawn`] for an image in a `box_w` × `box_h` CSS-px box: [`drawn_scale`],
/// placed by `object_position` unless the fit is `fill` (which fills the box,
/// so position moves nothing).
pub fn drawn_rect(media: &ElementMedia, box_w: f64, box_h: f64) -> Result<Drawn, LocateError> {
    const WHAT: &str = "where the image is drawn in its box is unknown";
    let (scale_x, scale_y) = drawn_scale(media, box_w, box_h)?;
    if media.object_fit == Some(ObjectFit::Fill) {
        return Ok(Drawn {
            x: 0.0,
            y: 0.0,
            w: box_w,
            h: box_h,
            scale_x,
            scale_y,
        });
    }
    let n = media
        .natural_px
        .ok_or_else(|| missing("natural_px", WHAT))?;
    let (px, py) = parsed_position(media, WHAT)?;
    let (w, h) = (n.w as f64 * scale_x, n.h as f64 * scale_y);
    Ok(Drawn {
        x: px.offset(box_w - w),
        y: py.offset(box_h - h),
        w,
        h,
        scale_x,
        scale_y,
    })
}

/// The part of the natural image left visible under object-fit and
/// object-position, in natural-frame fractions — a deduction from the
/// snapshot's media facts.
///
/// Every fit that scales the whole image INTO the box (`fill`; `contain` and
/// `scale-down` placed by a percentage or keyword) shows all of it without
/// any size being known. Otherwise (`cover`, or `contain` pushed by a px
/// offset, which can move it partly out of the box) the visible part is the
/// [`Drawn`] rect clipped to the box. Uses `rendered_px` as the content box
/// (padding and borders are not on the wire).
pub fn visible_region(media: &ElementMedia) -> Result<NaturalRegion, LocateError> {
    const WHAT: &str = "the visible region was not computed";
    let fit = media
        .object_fit
        .ok_or_else(|| missing("object_fit", WHAT))?;
    let fractions_only = |m: &ElementMedia| {
        parsed_position(m, WHAT).map(|(x, y)| {
            matches!(x, PositionComponent::Fraction(f) if (0.0..=1.0).contains(&f))
                && matches!(y, PositionComponent::Fraction(f) if (0.0..=1.0).contains(&f))
        })
    };
    match fit {
        ObjectFit::Fill => return Ok(NaturalRegion::WHOLE),
        ObjectFit::Contain | ObjectFit::ScaleDown if fractions_only(media)? => {
            return Ok(NaturalRegion::WHOLE)
        }
        ObjectFit::ScaleDown | ObjectFit::None => {
            return Err((
                UnknownCode::InputMissing,
                format!(
                    "object-fit {} placed by a px offset (or not scaled at all) depends on the \
                     CSS natural size, which media does not carry, so {WHAT}",
                    if fit == ObjectFit::None {
                        "none"
                    } else {
                        "scale-down"
                    }
                ),
            ))
        }
        ObjectFit::Contain | ObjectFit::Cover => {}
    }
    let b = media
        .rendered_px
        .ok_or_else(|| missing("rendered_px", WHAT))?;
    let d = drawn_rect(media, b.w as f64, b.h as f64)?;
    let (bw, bh) = (b.w as f64, b.h as f64);
    let x0 = ((0.0 - d.x) / d.w).clamp(0.0, 1.0);
    let x1 = ((bw - d.x) / d.w).clamp(0.0, 1.0);
    let y0 = ((0.0 - d.y) / d.h).clamp(0.0, 1.0);
    let y1 = ((bh - d.y) / d.h).clamp(0.0, 1.0);
    Ok(NaturalRegion {
        x: x0,
        y: y0,
        width: (x1 - x0).max(0.0),
        height: (y1 - y0).max(0.0),
    })
}

/// Effective resolution: the producer's value when it measured one, else
/// natural ÷ (displayed × DPR) from the media facts and the frame's DPR.
/// `fill` reports the smaller axis, as [`ElementMedia::effective_resolution`]
/// does. Computed in f64 and refused when not finite: a non-finite value
/// reaches the wire as `null`, which no reader can parse back.
pub fn effective_resolution(media: &ElementMedia, dpr: f64) -> Result<f64, LocateError> {
    const WHAT: &str = "effective resolution was not computed";
    let finite = |v: f64| {
        if v.is_finite() {
            Ok(v)
        } else {
            Err((
                UnknownCode::InputMissing,
                format!("the media facts give a non-finite value ({v}), so {WHAT}"),
            ))
        }
    };
    if let Some(e) = media.effective_resolution {
        return finite(e as f64);
    }
    let b = media
        .rendered_px
        .ok_or_else(|| missing("rendered_px", WHAT))?;
    if !(dpr.is_finite() && dpr > 0.0) {
        return Err((
            UnknownCode::InputMissing,
            format!("the frame's DPR {dpr} is unusable, so {WHAT}"),
        ));
    }
    // The scale alone: effective resolution does not depend on where the
    // image is placed, so it needs no object_position.
    let (sx, sy) = drawn_scale(media, b.w as f64, b.h as f64)?;
    // Source px per device px, per axis; the smaller is the softer.
    finite((1.0 / (sx * dpr)).min(1.0 / (sy * dpr)))
}

/// Relative aspect distortion: `fill` stretches to the box; every other fit
/// preserves the natural aspect. In f64 and refused when not finite (an f32
/// ratio of two extreme sizes overflowed to `inf` and serialized as `null`).
pub fn aspect_distortion(media: &ElementMedia) -> Result<f64, LocateError> {
    const WHAT: &str = "aspect distortion was not computed";
    let fit = media
        .object_fit
        .ok_or_else(|| missing("object_fit", WHAT))?;
    if fit != ObjectFit::Fill {
        return Ok(0.0);
    }
    let n = media
        .natural_px
        .ok_or_else(|| missing("natural_px", WHAT))?;
    let b = media
        .rendered_px
        .ok_or_else(|| missing("rendered_px", WHAT))?;
    let (nw, nh, bw, bh) = (n.w as f64, n.h as f64, b.w as f64, b.h as f64);
    let v = ((bw / bh) / (nw / nh) - 1.0).abs();
    if v.is_finite() && nw > 0.0 && nh > 0.0 && bw > 0.0 && bh > 0.0 {
        Ok(v)
    } else {
        Err((
            UnknownCode::InputMissing,
            format!("natural_px or rendered_px has a zero or non-finite side, so {WHAT}"),
        ))
    }
}

// ---------------------------------------------------------------------------
// Locating an image on the frame
// ---------------------------------------------------------------------------

/// One element's pixels on the frame.
#[derive(Debug, Clone)]
pub struct Crop {
    /// The in-frame buffer region `(x, y, w, h)`.
    pub px: (u32, u32, u32, u32),
    /// The mapped (unclamped) buffer region of the IMAGE: the element's box,
    /// clipped to the [`Drawn`] rect when that is known (so a `contain`
    /// letterbox is not read as image).
    pub mapped: Region,
    pub rgb: RgbImage,
    pub luma: GrayImage,
    /// Share (0..=1) of the image that is inside the frame and not covered by
    /// another element: how much of what was read is the image itself. See
    /// the module docs.
    pub confidence: f64,
    /// The image's JPEG coding grid on this crop, when the drawn geometry is
    /// known (see [`pixels::CodingGrid`]).
    pub grid: Option<pixels::CodingGrid>,
}

/// Why an element could not be located on the frame, typed.
pub type LocateError = (UnknownCode, String);

/// Why `el` paints NOTHING on the frame, when it paints nothing: a zero-area
/// box, or a producer-stated [`VisibilityReason`] of `hidden` / `no-layout`
/// (`display: none`, `visibility: hidden`, `opacity: 0`, no box). `None` when
/// it may paint.
///
/// An unrendered image is NOT a gap in the capture: there is nothing of it to
/// read, so it is left out of the page-wide checks rather than making them
/// unknown, and its pixel values are unknown with this reason.
///
/// `visible: false` ALONE is never read as "paints nothing": the SDK also
/// says `false` for an element hidden behind a transparent overlay and for one
/// outside the viewport. See [`unreadable`].
pub fn unrendered(_snapshot: &ElementSnapshot, el: &Element) -> Option<String> {
    if let Some(r) = el.visibility_reason.as_ref().filter(|r| r.paints_nothing()) {
        return Some(format!(
            "element '{}' is reported {} ({}), so it paints nothing to read",
            el.id,
            String::from(r.clone()),
            match r {
                VisibilityReason::Hidden => "display: none, visibility: hidden or opacity: 0",
                _ => "it has no layout box",
            }
        ));
    }
    if let Some(b) = el.bbox {
        if b.w == 0 || b.h == 0 {
            return Some(format!(
                "element '{}' has a zero-area box (display: none, or collapsed), so it paints \
                 nothing to read",
                el.id
            ));
        }
    }
    None
}

/// Why `el` MAY paint but its pixels cannot be attributed to it, when that is
/// so. Unlike [`unrendered`], this is a gap: an image here is Unknown, and a
/// page-wide check over it is Unknown, never silently passed.
///
/// - `visibility_reason: occluded` — every hit-test sample landed elsewhere.
///   A transparent overlay (a stretched-link card, an inset-0 scrim) produces
///   exactly this while the image is fully painted beneath it.
/// - `visibility_reason: off-screen` — outside the viewport at capture.
/// - any other reason this crate does not know, or `visible: false` with no
///   reason at all (an older producer).
/// - a box wholly inside the box of the element the producer names in
///   `occluded_by`: the occluder may be opaque (a carousel's top slide) or
///   transparent (a scrim), and the snapshot cannot say which.
pub fn unreadable(snapshot: &ElementSnapshot, el: &Element) -> Option<String> {
    if unrendered(snapshot, el).is_some() {
        return None;
    }
    let by = el
        .occluded_by
        .as_deref()
        .map(|o| format!(" (by '{o}')"))
        .unwrap_or_default();
    match (el.visibility_reason.as_ref(), el.visible) {
        (Some(VisibilityReason::Occluded), _) => {
            return Some(format!(
                "element '{}' is reported occluded at every hit-test sample{by}; a transparent \
                 overlay reads the same while the image is fully painted beneath it, so whether \
                 the frame shows it is unknown",
                el.id
            ))
        }
        (Some(VisibilityReason::OffScreen), _) => {
            return Some(format!(
                "element '{}' was outside the viewport at capture, so the frame does not show it",
                el.id
            ))
        }
        (Some(VisibilityReason::Other(r)), _) => {
            return Some(format!(
                "element '{}' is reported not visible for reason '{r}', which this crate does \
                 not interpret, so whether the frame shows it is unknown",
                el.id
            ))
        }
        (None, Some(false)) => {
            return Some(format!(
                "element '{}' is reported visible: false with no reason (hidden, off-screen and \
                 occluded all read that way), so whether the frame shows it is unknown",
                el.id
            ))
        }
        _ => {}
    }
    let b = el.bbox?;
    let top = el
        .occluded_by
        .as_deref()
        .and_then(|id| snapshot.get(id))
        .and_then(|t| t.bbox)?;
    region_contains(top, b).then(|| {
        format!(
            "element '{}' lies wholly inside '{}', which the producer records as occluding it; \
             an opaque occluder (a carousel's top slide) and a transparent one (a scrim) read \
             the same, so whether the frame shows it is unknown",
            el.id,
            el.occluded_by.as_deref().unwrap_or_default()
        )
    })
}

/// Map a CSS-px rect (absolute, f64) onto buffer pixels, rounding outward —
/// [`Frame::css_region_to_pixels`] for a fractional rect.
fn css_rect_to_px(frame: &Frame, x0: f64, y0: f64, x1: f64, y1: f64) -> Option<Region> {
    let s = frame.source.scale_factor;
    if !(s.is_finite() && s > 0.0 && x1 > x0 && y1 > y0) {
        return None;
    }
    let (px0, py0) = ((x0 * s).floor(), (y0 * s).floor());
    let (px1, py1) = ((x1 * s).ceil(), (y1 * s).ceil());
    let fits = |v: f64| v.abs() < i32::MAX as f64 / 2.0;
    if ![px0, py0, px1, py1].iter().all(|v| fits(*v)) {
        return None;
    }
    Some(Region {
        x: px0 as i32,
        y: py0 as i32,
        w: (px1 - px0) as u32,
        h: (py1 - py0) as u32,
    })
}

/// Find `el`'s IMAGE pixels on `frame`: its box, clipped to the drawn image
/// when the media facts place it, mapped onto the buffer.
pub fn locate(
    frame: &Frame,
    snapshot: &ElementSnapshot,
    el: &Element,
) -> Result<Crop, LocateError> {
    if let Some(why) = unrendered(snapshot, el).or_else(|| unreadable(snapshot, el)) {
        return Err((UnknownCode::InputMissing, why));
    }
    let css = el.bbox.ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!(
                "element '{}' carries no bbox, so its pixels cannot be found",
                el.id
            ),
        )
    })?;
    let (bx, by, bw, bh) = (css.x as f64, css.y as f64, css.w as f64, css.h as f64);
    let drawn = match el.media.as_ref() {
        None => None,
        // An inline SVG and a `role=img` element paint their OWN box: there is
        // no replaced content laid out inside it, and the SDK records no
        // object-fit for them (it measures replaced style only for img,
        // video and canvas). Their box is the painted region, as under `fill`.
        Some(m) if matches!(m.kind, MediaKind::Svg | MediaKind::RoleImg) => None,
        // A CSS background's painted region is set by background-size,
        // -position and -repeat, none of which is on the wire: the box may be
        // part page, and nothing here can say which part.
        Some(m) if m.kind == MediaKind::CssBackground => {
            return Err((
                UnknownCode::InputMissing,
                format!(
                    "element '{}' is a CSS background image: background-size, -position and \
                     -repeat are not on the wire, so which of its box's pixels are the image \
                     is unknown",
                    el.id
                ),
            ))
        }
        Some(m) => match drawn_rect(m, bw, bh) {
            Ok(d) => Some(d),
            // `fill` and `cover` paint the whole box, so the box IS the image
            // even when its placement is unknown; every other fit (or an
            // unknown one) may leave letterbox, and a crop that might be
            // part page background is not a reading of the image.
            Err(_) if matches!(m.object_fit, Some(ObjectFit::Fill | ObjectFit::Cover)) => None,
            Err((code, why)) => {
                return Err((
                    code,
                    format!(
                        "element '{}': its fit may letterbox and {why}, so its box's pixels \
                         cannot be told apart from the page behind it",
                        el.id
                    ),
                ))
            }
        },
    };
    // The image's content rect in absolute CSS px.
    let (cx0, cy0, cx1, cy1) = match drawn {
        Some(d) => (
            bx + d.x.max(0.0),
            by + d.y.max(0.0),
            bx + (d.x + d.w).min(bw),
            by + (d.y + d.h).min(bh),
        ),
        None => (bx, by, bx + bw, by + bh),
    };
    let mapped = css_rect_to_px(frame, cx0, cy0, cx1, cy1).ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!(
                "the frame's scale_factor {} cannot place element '{}' on its pixels (or its \
                 drawn image is empty)",
                frame.source.scale_factor, el.id
            ),
        )
    })?;
    let (x, y, w, h) = mapped
        .clamp_to_frame(frame.width, frame.height)
        .filter(|(_, _, w, h)| *w > 0 && *h > 0)
        .ok_or_else(|| {
            (
                UnknownCode::InputMissing,
                format!("element '{}' lies wholly outside the frame", el.id),
            )
        })?;
    let mut rgb = RgbImage::new(w, h);
    for yy in 0..h {
        for xx in 0..w {
            let p = frame.buffer.get_pixel(x + xx, y + yy).0;
            rgb.put_pixel(xx, yy, ::image::Rgb([p[0], p[1], p[2]]));
        }
    }
    let mapped_area = mapped.w as f64 * mapped.h as f64;
    let in_frame = (w as f64 * h as f64 / mapped_area).min(1.0);
    let s = frame.source.scale_factor;
    // The coding grid: 8 source px at the drawn scale, anchored where the
    // natural image's top-left edge lands, relative to the crop.
    let grid = drawn.map(|d| pixels::CodingGrid {
        period_x: 8.0 * d.scale_x * s,
        phase_x: (bx + d.x) * s - x as f64,
        period_y: 8.0 * d.scale_y * s,
        phase_y: (by + d.y) * s - y as f64,
    });
    let luma = pixels::luma_image(&rgb);
    Ok(Crop {
        px: (x, y, w, h),
        mapped,
        rgb,
        luma,
        confidence: in_frame * (1.0 - overlaid_fraction(snapshot, el, (cx0, cy0, cx1, cy1))),
        grid,
    })
}

/// Whether two boxes overlap enough to be one stack (intersection over union
/// above 0.5). Geometry alone does not decide a pair is hidden — see
/// [`pair_is_one_stack`].
pub fn boxes_stack(a: Region, b: Region) -> bool {
    let Some(i) = intersection(a, b) else {
        return false;
    };
    let inter = i.w as f64 * i.h as f64;
    let union = a.w as f64 * a.h as f64 + b.w as f64 * b.h as f64 - inter;
    union > 0.0 && inter / union > 0.5
}

/// Whether a pair of images is one STACK whose two members the frame cannot
/// both show: their boxes stack ([`boxes_stack`]) AND the producer reports one
/// of them covered — an `occluded_by`, a non-zero `occluded_pct`, or
/// `visible: false`. Two images at one place that are both reported fully
/// visible (or about which the producer measured nothing) are compared like
/// any other pair: skipping them on geometry alone would hide a real
/// duplicate drawn twice.
pub fn pair_is_one_stack(a: &Element, b: &Element) -> bool {
    let covered = |e: &Element| {
        e.occluded_by.is_some()
            || e.occluded_pct.is_some_and(|p| p > 0.0)
            || e.visible == Some(false)
    };
    match (a.bbox, b.bbox) {
        (Some(ra), Some(rb)) => boxes_stack(ra, rb) && (covered(a) || covered(b)),
        _ => false,
    }
}

/// The snapshot's DOM ancestry, built ONCE per run: an id → parent map, so
/// an ancestor walk is O(depth) instead of a linear `ElementSnapshot::get`
/// per hop (which, on a long or cyclic `parent_id` chain, made a page-wide
/// pair check O(pairs × N²)).
#[derive(Debug)]
pub struct Ancestry<'a> {
    parents: std::collections::HashMap<&'a str, &'a str>,
}

impl<'a> Ancestry<'a> {
    pub fn of(snapshot: &'a ElementSnapshot) -> Self {
        let mut parents = std::collections::HashMap::with_capacity(snapshot.elements.len());
        for e in &snapshot.elements {
            if let Some(p) = e.parent_id.as_deref() {
                parents.insert(e.id.as_str(), p);
            }
            for c in &e.children_ids {
                parents.entry(c.as_str()).or_insert(e.id.as_str());
            }
        }
        Self { parents }
    }

    /// Every ancestor id of `id`, nearest first. A cyclic chain ends at the
    /// first repeat.
    pub fn ancestors(&self, id: &'a str) -> std::collections::HashSet<&'a str> {
        let mut seen = std::collections::HashSet::new();
        let mut cur = self.parents.get(id).copied();
        while let Some(p) = cur {
            if !seen.insert(p) {
                break;
            }
            cur = self.parents.get(p).copied();
        }
        seen
    }
}

/// Drop every WRAPPER from `items`: an element that has a descendant also in
/// `items` (`parent_id` chain or `children_ids`). A `role=img` wrapper and its
/// own SVG child, at one box, are ONE image read through two elements — the
/// child is kept and the pair is never formed; a wrapper around SEVERAL child
/// images is a group, not an image — the wrapper goes and every child stays,
/// so a duplicate pair among the children is still compared. Returns the
/// kept items and how many were dropped. Ancestor sets are computed once per
/// item.
pub fn drop_nested<'e, T>(
    ancestry: &Ancestry<'e>,
    items: Vec<(&'e Element, T)>,
) -> (Vec<(&'e Element, T)>, usize) {
    let ancestors: Vec<std::collections::HashSet<&str>> = items
        .iter()
        .map(|(e, _)| ancestry.ancestors(e.id.as_str()))
        .collect();
    let ids: Vec<&str> = items.iter().map(|(e, _)| e.id.as_str()).collect();
    // `i` wraps `j` when `i` is among `j`'s ancestors and not the other way
    // round: an ancestor that is also a descendant is a CYCLE in the
    // producer's chain, not a nesting, and neither is dropped for it.
    let wraps = |i: usize, j: usize| {
        i != j && ancestors[j].contains(ids[i]) && !ancestors[i].contains(ids[j])
    };
    let wrappers: Vec<bool> = (0..items.len())
        .map(|i| (0..items.len()).any(|j| wraps(i, j)))
        .collect();
    let before = items.len();
    let kept: Vec<_> = items
        .into_iter()
        .zip(wrappers)
        .filter(|(_, is_wrapper)| !is_wrapper)
        .map(|(item, _)| item)
        .collect();
    let dropped = before - kept.len();
    (kept, dropped)
}

/// Fraction (0..=1) of the image's content rect (absolute CSS px) covered by
/// OTHER elements laid over it, sampled on a 16×16 grid of points. An overlay
/// is the element the producer records in `occluded_by` (whatever its box),
/// or any element carrying text, interactivity or media whose box intersects
/// the image without containing it whole — a box that contains the image is
/// a container (a section with a CSS background, a `role=img` wrapper, a card
/// whose text includes the caption), not something painted over it.
fn overlaid_fraction(snapshot: &ElementSnapshot, el: &Element, r: (f64, f64, f64, f64)) -> f64 {
    let (x0, y0, x1, y1) = r;
    let css = Region {
        x: x0.floor() as i32,
        y: y0.floor() as i32,
        w: (x1.ceil() - x0.floor()).max(0.0) as u32,
        h: (y1.ceil() - y0.floor()).max(0.0) as u32,
    };
    let overlays: Vec<Region> = snapshot
        .elements
        .iter()
        .filter(|o| o.id != el.id && unrendered(snapshot, o).is_none())
        .filter_map(|o| {
            let b = o.bbox?;
            intersection(b, css)?;
            let occluder = el.occluded_by.as_deref() == Some(o.id.as_str());
            let overlay = occluder
                || ((o.text.is_some() || o.interactable || o.media.is_some())
                    && !region_contains(b, css));
            overlay.then_some(b)
        })
        .collect();
    if overlays.is_empty() || x1 <= x0 || y1 <= y0 {
        return 0.0;
    }
    let mut covered = 0;
    for i in 0..16 {
        for j in 0..16 {
            let px = x0 + (i as f64 + 0.5) * (x1 - x0) / 16.0;
            let py = y0 + (j as f64 + 0.5) * (y1 - y0) / 16.0;
            if overlays.iter().any(|b| {
                px >= b.x as f64
                    && px < b.right() as f64
                    && py >= b.y as f64
                    && py < b.bottom() as f64
            }) {
                covered += 1;
            }
        }
    }
    covered as f64 / 256.0
}

// ---------------------------------------------------------------------------
// Measurements shared with the assertions
// ---------------------------------------------------------------------------

/// A low-entropy fill: luma entropy under 0.25 bits (one level holds roughly
/// 96% of the crop or more), or a crop with no edge at all whose Laplacian
/// variance is under 0.5 (a smooth gradient fill).
pub fn is_low_entropy_fill(luma: &GrayImage) -> bool {
    let entropy = pixels::luma_entropy(luma).unwrap_or(0.0);
    if entropy < 0.25 {
        return true;
    }
    let edges = pixels::edge_density(luma).unwrap_or(0.0);
    let lap = pixels::laplacian_variance(luma).unwrap_or(0.0);
    edges == 0.0 && lap < 0.5
}

/// Smallest side, in CSS px, of a SLOT-sized image. Below it an image is an
/// icon, a swatch, a dot or a bar: a flat one is not an empty slot (the
/// placeholder fill), and identical ones (repeated check marks, rating stars)
/// are not duplicated content (the near-duplicate comparison).
pub const PLACEHOLDER_FILL_MIN_SIDE_CSS: f64 = 48.0;

/// Whether `crop`'s drawn image has both sides at least
/// [`PLACEHOLDER_FILL_MIN_SIDE_CSS`] at the frame's `scale_factor`.
pub fn is_slot_sized(crop: &Crop, scale_factor: f64) -> bool {
    let floor = PLACEHOLDER_FILL_MIN_SIDE_CSS * scale_factor;
    crop.mapped.w as f64 >= floor && crop.mapped.h as f64 >= floor
}

/// The placeholder signals for one image. The known pattern is structural
/// ([`pixels::is_crossed_box`]), not a hash match: see that function.
/// `scale_factor` is the frame's, so the size floors are in CSS px.
///
/// The low-entropy-fill signal applies only to RASTER kinds (img, picture,
/// video, canvas) at slot size (both sides at least
/// [`PLACEHOLDER_FILL_MIN_SIDE_CSS`]): a flat inline SVG or `role=img`
/// element is ordinarily a data graphic — a 0% or 100% progress bar, a
/// brand-colour swatch — and flatness is its content, not a missing asset.
pub fn placeholder_values(
    media: &ElementMedia,
    crop: &Crop,
    scale_factor: f64,
) -> PlaceholderValues {
    let raster = matches!(
        media.kind,
        MediaKind::Img | MediaKind::Picture | MediaKind::Video | MediaKind::Canvas
    );
    let slot_sized = is_slot_sized(crop, scale_factor);
    let low_entropy_fill = raster && slot_sized && is_low_entropy_fill(&crop.luma);
    let known_pattern =
        pixels::is_crossed_box(&crop.rgb, scale_factor).then(|| "crossed_box".to_string());
    let placeholder =
        media.placeholder == Some(true) || low_entropy_fill || known_pattern.is_some();
    PlaceholderValues {
        marker: media.placeholder,
        low_entropy_fill,
        known_pattern,
        placeholder,
    }
}

/// The share (0..=1) of a crop's sampled pixels within `max_delta_e`
/// (CIEDE2000) of their nearest `palette` colour, and the sampled colour
/// furthest from the palette. `None` for an empty palette or crop.
pub fn on_palette_coverage(
    crop: &Crop,
    palette: &[Rgb],
    max_delta_e: f64,
) -> Option<(f64, Rgb, f64)> {
    let sample = pixels::sample_pixels(&crop.rgb, PALETTE_SAMPLE);
    if sample.is_empty() || palette.is_empty() {
        return None;
    }
    let mut on = 0usize;
    let mut worst = (sample[0], 0.0_f64);
    let mut cache: std::collections::HashMap<Rgb, f64> = Default::default();
    for p in &sample {
        let d = *cache.entry(*p).or_insert_with(|| {
            palette
                .iter()
                .map(|q| crate::assertions::delta_e_2000(*p, *q))
                .fold(f64::INFINITY, f64::min)
        });
        if d <= max_delta_e {
            on += 1;
        }
        if d > worst.1 {
            worst = (*p, d);
        }
    }
    Some((on as f64 / sample.len() as f64, worst.0, worst.1))
}

/// The legibility of an image's edge and stroke pixels against what surrounds
/// it on THIS frame — the check a transparent icon or logo needs, because a
/// captured frame is already composited and has no alpha left to inspect.
///
/// The surround is a ring of buffer pixels just outside the crop (4 CSS px
/// wide, at least 2 buffer px), and its colour is the mean of the ring pixels
/// in the ring's most common 4-bit colour bin. Stroke pixels are the crop's
/// pixels more than 8 (RGB distance) from that colour. The value is the 90th
/// percentile of their WCAG contrast against the surround: the image's most
/// distinct marks, robust to anti-aliasing. A crop with under 0.5% stroke
/// pixels is invisible against its surround and reads 1.0.
///
/// Only the four bands of the ring are walked, each clipped to the frame, so
/// the cost is the ring's in-frame area whatever the element's size.
///
/// Returns `(ratio, ring in-frame fraction)`, or `None` when no ring pixel is
/// inside the frame.
pub fn edge_contrast(frame: &Frame, crop: &Crop) -> Option<(f64, f64)> {
    let band = ((4.0 * frame.source.scale_factor).round() as i64).max(2);
    let m = crop.mapped;
    let (l, t, r, b) = (m.x as i64, m.y as i64, m.right(), m.bottom());
    // (x0, y0, x1, y1), half-open: top, bottom, left, right.
    let bands = [
        (l - band, t - band, r + band, t),
        (l - band, b, r + band, b + band),
        (l - band, t, l, b),
        (r, t, r + band, b),
    ];
    let ring_total: i64 = bands
        .iter()
        .map(|(x0, y0, x1, y1)| (x1 - x0) * (y1 - y0))
        .sum();
    let (fw, fh) = (frame.width as i64, frame.height as i64);
    let mut ring = Vec::new();
    for (x0, y0, x1, y1) in bands {
        for y in y0.max(0)..y1.min(fh) {
            for x in x0.max(0)..x1.min(fw) {
                let p = frame.buffer.get_pixel(x as u32, y as u32).0;
                ring.push(Rgb::new(p[0], p[1], p[2]));
            }
        }
    }
    if ring.is_empty() {
        return None;
    }
    let bin = |c: &Rgb| ((c.r as u16 >> 4) << 8) | ((c.g as u16 >> 4) << 4) | (c.b as u16 >> 4);
    let mut counts: std::collections::BTreeMap<u16, u32> = Default::default();
    for c in &ring {
        *counts.entry(bin(c)).or_default() += 1;
    }
    let mode = counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(k, _)| *k)?;
    let members: Vec<&Rgb> = ring.iter().filter(|c| bin(c) == mode).collect();
    let n = members.len() as f64;
    let mean =
        |f: fn(&Rgb) -> u8| (members.iter().map(|c| f(c) as f64).sum::<f64>() / n).round() as u8;
    let surround = Rgb::new(mean(|c| c.r), mean(|c| c.g), mean(|c| c.b));

    let dist = |c: &Rgb| {
        let (dr, dg, db) = (
            c.r as f64 - surround.r as f64,
            c.g as f64 - surround.g as f64,
            c.b as f64 - surround.b as f64,
        );
        (dr * dr + dg * dg + db * db).sqrt()
    };
    let mut ratios: Vec<f64> = crop
        .rgb
        .pixels()
        .map(|p| Rgb::new(p[0], p[1], p[2]))
        .filter(|c| dist(c) > 8.0)
        .map(|c| super::color::wcag_contrast(c, surround))
        .collect();
    let ring_fraction = ring.len() as f64 / ring_total.max(1) as f64;
    let total = crop.rgb.pixels().len().max(1);
    if (ratios.len() as f64) < 0.005 * total as f64 {
        return Some((1.0, ring_fraction));
    }
    ratios.sort_by(f64::total_cmp);
    let idx = ((ratios.len() as f64 * 0.9).ceil() as usize).saturating_sub(1);
    Some((ratios[idx], ring_fraction))
}

/// The page's spectral-residual saliency map over a frame's luma, and its
/// peakedness (`1 − mean`, the map's maximum being 1): how far the map singles
/// anything out at all. A flat map ranks nothing, and every saliency
/// confidence is scaled by this.
pub fn saliency_map_of(frame_luma: &GrayImage) -> Option<(Vec<Vec<f64>>, f64)> {
    let map = pixels::spectral_residual(frame_luma)?;
    let cells = (map.len() * map.len()) as f64;
    let mean = map.iter().flatten().sum::<f64>() / cells;
    Some((map, (1.0 - mean).clamp(0.0, 1.0)))
}

/// The frame's pixels as RGB and as luma, converted once per run.
pub fn frame_planes(frame: &Frame) -> (RgbImage, GrayImage) {
    let rgb = ::image::DynamicImage::ImageRgba8(frame.buffer.clone()).to_rgb8();
    let luma = pixels::luma_image(&rgb);
    (rgb, luma)
}

/// [`saliency_map_of`] a whole frame.
pub fn saliency_map(frame: &Frame) -> Option<(Vec<Vec<f64>>, f64)> {
    saliency_map_of(&frame_planes(frame).1)
}

// ---------------------------------------------------------------------------
// Observation plumbing
// ---------------------------------------------------------------------------

/// What every observation from one run shares.
struct Stamp {
    observed_at: DateTime<Utc>,
    evaluated_at: DateTime<Utc>,
}

impl Stamp {
    fn provenance(&self, coverage: ObservationCoverage, element: Option<&str>) -> Provenance {
        let mut p = Provenance::new(
            Producer::vision_core(Analyzer::Image),
            self.evaluated_at,
            coverage,
        )
        .with_observed_at(self.observed_at);
        if let Some(id) = element {
            let mut src = Map::new();
            src.insert("element".into(), Value::String(id.to_string()));
            p = p.with_source(src);
        }
        p
    }

    fn measured<T>(&self, v: T, element: Option<&str>, confidence: Option<f64>) -> Observation<T> {
        self.measured_over(v, element, confidence, ObservationCoverage::full(1))
    }

    fn measured_over<T>(
        &self,
        v: T,
        element: Option<&str>,
        confidence: Option<f64>,
        coverage: ObservationCoverage,
    ) -> Observation<T> {
        let mut p = self.provenance(coverage, element);
        if let Some(c) = confidence {
            p = p.with_confidence(c);
        }
        Observation::measured(v, p)
    }

    /// A value read from a crop, at `confidence`. A crop that might include
    /// letterbox is never produced ([`locate`] refuses it), so a value read
    /// from one is a full measurement of its one image.
    fn over_crop<T>(&self, v: T, element: &str, _crop: &Crop, confidence: f64) -> Observation<T> {
        self.measured_over(
            v,
            Some(element),
            Some(confidence),
            ObservationCoverage::full(1),
        )
    }

    fn unknown<T>(
        &self,
        (code, detail): LocateError,
        dimension: &str,
        element: Option<&str>,
    ) -> Observation<T> {
        let coverage = ObservationCoverage {
            considered: 1,
            measured: 0,
            unmeasured: vec![UnmeasuredDimension::new(dimension, 1, code)],
        };
        Observation::unknown(code, detail, self.provenance(coverage, element))
    }

    fn of_result<T>(
        &self,
        r: Result<T, LocateError>,
        dimension: &str,
        element: &str,
    ) -> Observation<T> {
        match r {
            Ok(v) => self.measured(v, Some(element), None),
            Err(e) => self.unknown(e, dimension, Some(element)),
        }
    }
}

// ---------------------------------------------------------------------------
// The analyzer
// ---------------------------------------------------------------------------

/// What happened to one media element.
enum Located {
    Crop(Crop),
    Unrendered,
    /// Could not be read, and why.
    Unlocated(String),
}

/// The text-hidden frame as [`measure`] reads it: not supplied, or supplied
/// and either paired with the frame or refused with the reason.
type TextHidden<'a> = Option<Result<&'a Frame, LocateError>>;

/// What [`measure`] needs for the text-over-image value, built once per run.
struct TextCtx<'a> {
    hidden: TextHidden<'a>,
    index: Option<text_contrast::TextIndex<'a>>,
}

/// Run the image analyzer. The dispatcher has already answered `Blocked`
/// when there is no frame; `snapshot` may still be absent.
///
/// `text_hidden` is the frame's text-hidden twin (Phase 16), read only for
/// [`ImageElementValues::text_contrast`]. Its absence changes no verdict: the
/// value is then `unknown{needs_multi_frame_input}` where text lies over an
/// image. One that cannot be paired with the frame
/// ([`text_contrast::pair_frames`]) is an input the caller asked to be read
/// and that was not, so the run is `Degraded{input_missing}` — green.
pub fn run(
    frame: &Frame,
    snapshot: Option<&ElementSnapshot>,
    text_hidden: Option<&Frame>,
) -> AnalyzerResult {
    let text_hidden = TextCtx {
        hidden: text_hidden.map(|b| text_contrast::pair_frames(frame, b).map(|()| b)),
        index: snapshot.map(text_contrast::TextIndex::of),
    };
    let stamp = Stamp {
        observed_at: frame.source.captured_at,
        evaluated_at: Utc::now(),
    };
    let (frame_rgb, frame_luma) = frame_planes(frame);
    let page = page_values(frame, &frame_rgb, &frame_luma, &stamp);

    let Some(snapshot) = snapshot else {
        let reason = "no ElementSnapshot, so no image element was located; only the page-level \
                      values were measured";
        return AnalyzerResult::degraded(UnknownCode::InputMissing, reason, None, Vec::new())
            .with_image_values(ImageValues {
                images: Vec::new(),
                page,
                near_duplicates: stamp.unknown(
                    (UnknownCode::InputMissing, reason.to_string()),
                    "media",
                    None,
                ),
            });
    };
    let saliency = saliency_map_of(&frame_luma);
    let coverage = SnapshotCoverage::of(snapshot);
    let media_elements: Vec<(&Element, &ElementMedia)> = snapshot
        .elements
        .iter()
        .filter_map(|e| e.media.as_ref().map(|m| (e, m)))
        .collect();

    let mut images = Vec::with_capacity(media_elements.len());
    // (id, css box, phash, dhash, confidence)
    let mut hashed: Vec<(&Element, u64, u64, f64)> = Vec::new();
    let mut small = 0usize;
    let mut findings = Vec::new();
    let mut unlocated = Vec::new();
    for (el, media) in &media_elements {
        let (values, located, hashes) = measure(
            frame,
            snapshot,
            el,
            media,
            &stamp,
            saliency.as_ref(),
            &text_hidden,
        );
        match located {
            Located::Crop(crop) => {
                // A flat crop's pHash is degenerate whatever its kind, so it
                // is never compared (a placeholder or not); an image under
                // slot size is an icon, and identical icons are not
                // duplicated content.
                if let Some((ph, dh)) = hashes {
                    if !is_slot_sized(&crop, frame.source.scale_factor) {
                        small += 1;
                    } else if !is_low_entropy_fill(&crop.luma) {
                        hashed.push((el, ph, dh, crop.confidence));
                    }
                }
                if let Some(p) = values.placeholder.value().filter(|p| p.placeholder) {
                    let mut why = Vec::new();
                    if p.marker == Some(true) {
                        why.push("the slot marks it as its placeholder".to_string());
                    }
                    if p.low_entropy_fill {
                        why.push("its pixels are a low-entropy fill".to_string());
                    }
                    if let Some(k) = &p.known_pattern {
                        why.push(format!("its pixels draw the known placeholder '{k}'"));
                    }
                    let mut f = Finding::new(
                        "image_placeholder",
                        Severity::Info,
                        format!(
                            "image '{}' renders a placeholder: {}",
                            el.id,
                            why.join("; ")
                        ),
                    )
                    .with_elements(vec![el.id.clone()]);
                    // Pixel signals are estimates; the marker alone is not.
                    if p.low_entropy_fill || p.known_pattern.is_some() {
                        f = f.with_confidence(crop.confidence);
                    }
                    findings.push(f);
                }
            }
            Located::Unrendered => {}
            Located::Unlocated(why) => unlocated.push(why),
        }
        images.push(values);
    }

    // One image read through a wrapper and its own child is one image: the
    // nested element leaves the comparison set before any pair is formed.
    let ancestry = Ancestry::of(snapshot);
    let (hashed, nested) = {
        let items: Vec<(&Element, (u64, u64, f64))> = hashed
            .into_iter()
            .map(|(e, p, d, c)| (e, (p, d, c)))
            .collect();
        let (kept, n) = drop_nested(&ancestry, items);
        let kept: Vec<(&Element, u64, u64, f64)> = kept
            .into_iter()
            .map(|(e, (p, d, c))| (e, p, d, c))
            .collect();
        (kept, n)
    };
    let mut pairs = Vec::new();
    let mut stacked = 0usize;
    for i in 0..hashed.len() {
        for j in i + 1..hashed.len() {
            let (a, b) = (&hashed[i], &hashed[j]);
            // Two images at one place: the frame shows at most one of them,
            // so their hashes compare the top one with itself.
            // A stack: the frame shows one of the two, not two images.
            if pair_is_one_stack(a.0, b.0) {
                stacked += 1;
                continue;
            }
            let pair = NearDuplicatePair {
                a: a.0.id.clone(),
                b: b.0.id.clone(),
                phash_distance: pixels::hamming(a.1, b.1),
                dhash_distance: pixels::hamming(a.2, b.2),
            };
            if pair.phash_distance <= NEAR_DUPLICATE_REPORT_HAMMING {
                findings.push(
                    Finding::new(
                        "image_near_duplicate",
                        Severity::Info,
                        format!(
                            "images '{}' and '{}' look alike: pHash distance {} of 64 (dHash {})",
                            pair.a, pair.b, pair.phash_distance, pair.dhash_distance
                        ),
                    )
                    .with_elements(vec![pair.a.clone(), pair.b.clone()])
                    .with_confidence(a.3.min(b.3)),
                );
            }
            pairs.push(pair);
        }
    }
    let near_duplicates = if media_elements.is_empty() {
        stamp.unknown(
            (
                UnknownCode::InputMissing,
                "no element carries media, so no pair of images was compared".to_string(),
            ),
            "media",
            None,
        )
    } else {
        // Flat fills, unrendered images and stacked pairs are not CONSIDERED
        // (see `ImageValues::near_duplicates`); an image that could not be
        // located is considered and unmeasured.
        let measured = hashed.len() as u64;
        let considered = measured + unlocated.len() as u64;
        let mut unmeasured = Vec::new();
        if !unlocated.is_empty() {
            unmeasured.push(UnmeasuredDimension::new(
                "phash",
                unlocated.len() as u64,
                UnknownCode::InputMissing,
            ));
        }
        let mut p = stamp.provenance(
            ObservationCoverage {
                considered,
                measured,
                unmeasured,
            },
            None,
        );
        if stacked > 0 || nested > 0 || small > 0 {
            let mut src = Map::new();
            src.insert("stackedPairsSkipped".into(), Value::from(stacked as u64));
            src.insert("nestedImagesSkipped".into(), Value::from(nested as u64));
            src.insert("smallImagesSkipped".into(), Value::from(small as u64));
            p = p.with_source(src);
        }
        Observation::measured(pairs, p)
    };

    let values = ImageValues {
        images,
        page,
        near_duplicates,
    };
    // A text-hidden frame that was supplied and cannot be paired is named on
    // whatever degradation the run reports, or degrades it on its own.
    let unpaired = match &text_hidden.hidden {
        Some(Err((_, why))) => format!(
            " The text-hidden frame was not read, so no text-over-image contrast was measured: \
             {why}"
        ),
        _ => String::new(),
    };
    let result = if media_elements.is_empty() {
        AnalyzerResult::degraded(
            UnknownCode::InputMissing,
            format!(
                "no element carries media (0/{}), so no per-image value was measured; only the \
                 page-level values were. Green by design: an image-free page is not a failure.\
                 {unpaired}",
                coverage.elements
            ),
            Some(coverage),
            findings,
        )
    } else if !unlocated.is_empty() {
        AnalyzerResult::degraded(
            UnknownCode::InputMissing,
            format!(
                "{}/{} media element(s) could not be read from the frame, so their pixel values \
                 are unknown: {}.{unpaired}",
                unlocated.len(),
                media_elements.len(),
                unlocated.join("; ")
            ),
            Some(coverage),
            findings,
        )
    } else if !unpaired.is_empty() {
        AnalyzerResult::degraded(
            UnknownCode::InputMissing,
            unpaired.trim_start().to_string(),
            Some(coverage),
            findings,
        )
    } else {
        // Unrendered images (hidden, no layout, a zero-area box) paint
        // nothing, so there is nothing to have measured: not a gap.
        AnalyzerResult::checked(Some(coverage), findings)
    };
    result.with_image_values(values)
}

fn page_values(frame: &Frame, rgb: &RgbImage, luma: &GrayImage, stamp: &Stamp) -> PageImageValues {
    let empty = || {
        (
            UnknownCode::InputMissing,
            "the frame has no pixels to measure".to_string(),
        )
    };
    let tile = (8.0 * frame.source.scale_factor).round();
    let tile = if tile.is_finite() && tile >= 1.0 {
        tile as u32
    } else {
        8
    };
    PageImageValues {
        colourfulness: match pixels::colourfulness(rgb) {
            Some(v) => stamp.measured(v, None, None),
            None => stamp.unknown(empty(), "colourfulness", None),
        },
        clutter: match pixels::edge_density(luma) {
            Some(v) => stamp.measured(v, None, None),
            None => stamp.unknown(empty(), "clutter", None),
        },
        white_space: match pixels::flat_tile_fraction(luma, tile) {
            Some(v) => stamp.measured(v, None, None),
            None => stamp.unknown(empty(), "white_space", None),
        },
    }
}

/// Every per-image value for one media element, what happened when it was
/// located, and its (pHash, dHash) when measured — computed once here.
fn measure(
    frame: &Frame,
    snapshot: &ElementSnapshot,
    el: &Element,
    media: &ElementMedia,
    stamp: &Stamp,
    saliency: Option<&(Vec<Vec<f64>>, f64)>,
    text_hidden: &TextCtx<'_>,
) -> (ImageElementValues, Located, Option<(u64, u64)>) {
    let id = el.id.as_str();
    // From the image's box, not its crop: see `ImageElementValues::text_contrast`.
    let text_contrast = Some(text_contrast_value(frame, text_hidden, snapshot, el, stamp));
    let effective_resolution = stamp.of_result(
        effective_resolution(media, frame.source.scale_factor),
        "effective_resolution",
        id,
    );
    let aspect_distortion = stamp.of_result(aspect_distortion(media), "aspect_distortion", id);
    let visible_region = stamp.of_result(visible_region(media), "visible_region", id);

    let unrendered_now = unrendered(snapshot, el).is_some();
    let crop = match locate(frame, snapshot, el) {
        Ok(c) => c,
        Err(e) => {
            let located = if unrendered_now {
                Located::Unrendered
            } else {
                Located::Unlocated(e.1.clone())
            };
            return (
                ImageElementValues {
                    element: id.to_string(),
                    effective_resolution,
                    aspect_distortion,
                    sharpness: stamp.unknown(e.clone(), "sharpness", Some(id)),
                    blockiness: stamp.unknown(e.clone(), "blockiness", Some(id)),
                    palette: stamp.unknown(e.clone(), "palette", Some(id)),
                    class: stamp.unknown(e.clone(), "class", Some(id)),
                    busyness: stamp.unknown(e.clone(), "busyness", Some(id)),
                    hashes: stamp.unknown(e.clone(), "hashes", Some(id)),
                    placeholder: stamp.unknown(e.clone(), "placeholder", Some(id)),
                    edge_contrast: stamp.unknown(e.clone(), "edge_contrast", Some(id)),
                    visible_region,
                    saliency: stamp.unknown(e, "saliency", Some(id)),
                    text_contrast,
                },
                located,
                None,
            );
        }
    };
    let conf = crop.confidence;
    let too_small = |what: &str| {
        (
            UnknownCode::InputMissing,
            format!("element '{id}': the crop is too small to measure {what}"),
        )
    };

    let sharpness = match pixels::laplacian_variance(&crop.luma) {
        Some(v) => stamp.over_crop(v, id, &crop, conf),
        None => stamp.unknown(too_small("sharpness"), "sharpness", Some(id)),
    };
    let blockiness = match crop.grid {
        None => stamp.unknown(
            (
                UnknownCode::InputMissing,
                format!(
                    "element '{id}': where the natural image is drawn (natural_px, object_fit, \
                     object_position) is unknown, so its 8-pixel coding grid cannot be placed"
                ),
            ),
            "blockiness",
            Some(id),
        ),
        Some(g) => match pixels::blockiness(&crop.luma, g) {
            Some(v) => stamp.over_crop(v, id, &crop, conf),
            None => stamp.unknown(
                (
                    UnknownCode::InputMissing,
                    format!(
                        "element '{id}': the coding grid is drawn under 4 device px apart, the \
                         crop holds fewer than two grid lines, or the image has no detail off \
                         the grid to compare against"
                    ),
                ),
                "blockiness",
                Some(id),
            ),
        },
    };

    let sample = pixels::sample_pixels(&crop.rgb, PALETTE_SAMPLE);
    let palette = stamp.over_crop(
        pixels::kmeans_oklab(&sample, 5)
            .into_iter()
            .map(|c| PaletteColour {
                rgb: pixels::oklab_to_rgb(c.oklab),
                oklab: c.oklab,
                coverage: c.coverage,
            })
            .collect::<Vec<_>>(),
        id,
        &crop,
        conf,
    );
    let bins = pixels::bins_for_90_percent(&sample) as u32;
    let class = ImageClassValue {
        class: if bins <= FLAT_MAX_BINS {
            ImageClass::Flat
        } else {
            ImageClass::Photographic
        },
        colour_bins: bins,
    };
    // Margin from the class boundary on a log scale: at the boundary the
    // class is a coin toss (0); 8× either side of it is certain (1).
    let margin = ((bins.max(1) as f64 / FLAT_MAX_BINS as f64).ln().abs() / 8f64.ln()).min(1.0);
    let class = stamp.over_crop(class, id, &crop, conf * margin);

    let busyness = match (
        pixels::edge_density(&crop.luma),
        pixels::luma_entropy(&crop.luma),
    ) {
        (Some(edge_density), Some(entropy)) => stamp.over_crop(
            Busyness {
                edge_density,
                entropy,
                under_text: under_text_busyness(frame, snapshot, el),
            },
            id,
            &crop,
            conf,
        ),
        _ => stamp.unknown(too_small("busyness"), "busyness", Some(id)),
    };
    let hash_pair = pixels::phash(&crop.luma).zip(pixels::dhash(&crop.luma));
    let hashes = match hash_pair {
        Some((p, d)) => stamp.over_crop(
            ImageHashes {
                phash: pixels::hash_hex(p),
                dhash: pixels::hash_hex(d),
            },
            id,
            &crop,
            conf,
        ),
        None => stamp.unknown(too_small("hashes"), "hashes", Some(id)),
    };
    let ph = placeholder_values(media, &crop, frame.source.scale_factor);
    // Resting on the marker alone is a deduction; on pixels, an estimate.
    let placeholder = if ph.low_entropy_fill || ph.known_pattern.is_some() {
        stamp.over_crop(ph, id, &crop, conf)
    } else {
        stamp.measured(ph, Some(id), None)
    };
    let edge_contrast = match edge_contrast(frame, &crop) {
        Some((ratio, ring)) => stamp.over_crop(ratio, id, &crop, conf * ring),
        None => stamp.unknown(
            (
                UnknownCode::InputMissing,
                format!(
                    "element '{id}': no pixel around it is inside the frame, so it has no surround"
                ),
            ),
            "edge_contrast",
            Some(id),
        ),
    };
    let saliency = match saliency.and_then(|(map, peaked)| {
        pixels::grid_mean(map, frame.width, frame.height, crop.px).map(|m| (m, *peaked))
    }) {
        Some((mean, peaked)) => stamp.over_crop(mean, id, &crop, conf * peaked),
        None => stamp.unknown(
            (
                UnknownCode::InputMissing,
                "the frame is flat, so its saliency map singles nothing out".to_string(),
            ),
            "saliency",
            Some(id),
        ),
    };

    (
        ImageElementValues {
            element: id.to_string(),
            effective_resolution,
            aspect_distortion,
            sharpness,
            blockiness,
            palette,
            class,
            busyness,
            hashes,
            placeholder,
            edge_contrast,
            visible_region,
            saliency,
            text_contrast,
        },
        Located::Crop(crop),
        hash_pair,
    )
}

/// [`ImageElementValues::text_contrast`] for one image: every text element
/// laid over it, measured at [`text_contrast::DEFAULT_PERCENTILE`].
///
/// Coverage counts the text elements over the image: one whose glyphs turn
/// out not to reach it ([`text_contrast::measure`]'s `None`) was measured in
/// full and is simply not in the list; one reported not visible, or one that
/// could not be measured, is unmeasured. Confidence is the lowest of the
/// listed readings.
fn text_contrast_value(
    frame: &Frame,
    text: &TextCtx<'_>,
    snapshot: &ElementSnapshot,
    el: &Element,
    stamp: &Stamp,
) -> Observation<Vec<TextOverImageContrast>> {
    const DIM: &str = "text_contrast";
    let id = Some(el.id.as_str());
    // Without a box nothing can be over the image, and nothing was looked at:
    // that is not an absence.
    if el.bbox.is_none() {
        return stamp.unknown(
            (
                UnknownCode::InputMissing,
                format!(
                    "element '{}' carries no bbox, so which text lies over it is unknown",
                    el.id
                ),
            ),
            DIM,
            id,
        );
    }
    let built;
    let index = match &text.index {
        Some(i) => i,
        None => {
            built = text_contrast::TextIndex::of(snapshot);
            &built
        }
    };
    let over = text_contrast::texts_over(snapshot, el, index);
    let (pairs, hidden_texts) = (over.pairs, over.unreadable_texts);
    let considered = (pairs.len() + hidden_texts) as u64;
    if considered == 0 {
        return Observation::absent(stamp.provenance(ObservationCoverage::full(1), id))
            .expect("one image considered in full is a legal absence");
    }
    // Only an image that paints NOTHING is refused: an occluded or otherwise
    // unreadable one is measured, because frame B is read under the glyphs
    // and the image's crop never is.
    if let Some(why) = unrendered(snapshot, el) {
        return stamp.unknown((UnknownCode::InputMissing, why), DIM, id);
    }
    let b = match &text.hidden {
        _ if pairs.is_empty() => {
            return stamp.unknown(
                (
                    UnknownCode::InputMissing,
                    format!(
                        "the {hidden_texts} text element(s) over '{}' are reported not visible, \
                         so no glyph over it can be attributed to the frame",
                        el.id
                    ),
                ),
                DIM,
                id,
            )
        }
        None => {
            return stamp.unknown(
                (
                    UnknownCode::NeedsMultiFrameInput,
                    format!(
                        "{} text element(s) lie over '{}', and their glyphs are found only by \
                         comparing the frame with a text-hidden capture of it, which was not \
                         supplied",
                        pairs.len(),
                        el.id
                    ),
                ),
                DIM,
                id,
            )
        }
        Some(Err(e)) => return stamp.unknown(e.clone(), DIM, id),
        Some(Ok(b)) => *b,
    };
    let reg = match text_contrast::registered(frame, b, snapshot, el) {
        Ok(r) => r,
        Err(e) => return stamp.unknown(e, DIM, id),
    };
    let mut values = Vec::new();
    let mut not_over = 0u64;
    let mut failed = Vec::new();
    for p in &pairs {
        match text_contrast::measure(frame, b, p, text_contrast::DEFAULT_PERCENTILE, &reg) {
            Ok(Some(v)) => values.push(v),
            Ok(None) => not_over += 1,
            Err(e) => failed.push(e),
        }
    }
    let mut unmeasured = Vec::new();
    if hidden_texts > 0 {
        unmeasured.push(UnmeasuredDimension::new(
            "text_visibility",
            hidden_texts as u64,
            UnknownCode::InputMissing,
        ));
    }
    if !failed.is_empty() {
        unmeasured.push(UnmeasuredDimension::new(
            DIM,
            failed.len() as u64,
            UnknownCode::InputMissing,
        ));
    }
    let coverage = ObservationCoverage {
        considered,
        measured: values.len() as u64 + not_over,
        unmeasured,
    };
    if values.is_empty() {
        return match failed.into_iter().next() {
            Some((code, detail)) => {
                Observation::unknown(code, detail, stamp.provenance(coverage, id))
            }
            None => match Observation::absent(stamp.provenance(coverage.clone(), id)) {
                Ok(o) => o,
                // Unmeasured text with nothing listed: not an absence.
                Err(_) => Observation::unknown(
                    UnknownCode::InputMissing,
                    format!(
                        "no text element over '{}' could be measured or ruled out",
                        el.id
                    ),
                    stamp.provenance(coverage, id),
                ),
            },
        };
    }
    let conf = values.iter().map(|v| v.confidence).fold(1.0, f64::min);
    stamp.measured_over(values, id, Some(conf), coverage)
}

/// Busyness of the part of the image under each text element laid over it.
/// A text element whose box contains the image's whole box is a container
/// (its innerText includes the image's caption, or the image itself), not an
/// overlay, and is skipped as [`overlaid_fraction`] skips it.
fn under_text_busyness(
    frame: &Frame,
    snapshot: &ElementSnapshot,
    el: &Element,
) -> Vec<TextOverlayBusyness> {
    let Some(img) = el.bbox else {
        return Vec::new();
    };
    snapshot
        .elements
        .iter()
        .filter(|t| t.id != el.id && t.text.is_some())
        // Text that paints nothing, or that cannot be attributed to the
        // frame, is not over anything that can be measured.
        .filter(|t| unrendered(snapshot, t).is_none() && unreadable(snapshot, t).is_none())
        .filter_map(|t| {
            let tb = t.bbox?;
            if region_contains(tb, img) {
                return None;
            }
            let over = intersection(tb, img)?;
            let px = frame.css_region_to_pixels(over)?;
            let (x, y, w, h) = px.clamp_to_frame(frame.width, frame.height)?;
            let luma = GrayImage::from_fn(w, h, |xx, yy| {
                let p = frame.buffer.get_pixel(x + xx, y + yy).0;
                ::image::Luma([pixels::pil_luma(p[0], p[1], p[2])])
            });
            Some(TextOverlayBusyness {
                element: t.id.clone(),
                edge_density: pixels::edge_density(&luma)?,
                entropy: pixels::luma_entropy(&luma)?,
            })
        })
        .collect()
}
