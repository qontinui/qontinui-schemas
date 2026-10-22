//! Text over images, measured on real pixels (plan
//! 2026-10-13-image-generation-and-visual-judgment, Phase 16).
//!
//! A text element laid over an image has no single background colour: the
//! snapshot's `bg_color` is the resolved-up-the-tree opaque ancestor, which a
//! hero image is not, and sampling the frame under the text reads the glyphs
//! themselves. So the measurement takes TWO frames of one snapshot:
//!
//! - frame A, the page as it renders;
//! - frame B, the same page with the text hidden (the SDK's dev-only
//!   `control/text-visibility` makes glyphs transparent and restores them).
//!
//! # The two frames must be one layout
//!
//! [`pair_frames`] refuses a pair of different sizes or scale factors, and
//! [`registered`] refuses one whose IMAGE pixels differ away from the text:
//! inside the image's box, outside every text box (each widened by
//! [`REGISTRATION_MARGIN_CSS`] for anti-aliasing and shadows), at most
//! [`MAX_UNREGISTERED_SHARE`] of the pixels may differ. A frame B shifted by
//! one pixel, zoomed, captured mid-animation or taken from another page would
//! otherwise turn the image's own texture into a "glyph mask" and fail the
//! text at full confidence. Text over the image that the snapshot does not
//! carry also differs there, and is refused the same way: what hid is not
//! known.
//!
//! # Where the glyphs are
//!
//! The two frames differ exactly where glyphs paint, so the GLYPH MASK of a
//! text element is the set of buffer pixels inside its box where frames A and
//! B differ by more than [`GLYPH_DIFF_MIN`]. The pixels sampled are the
//! mask's pixels over the image that MOVED TOWARD THE GLYPH COLOUR (closer to
//! it in A than in B): a glyph and its anti-aliased edge did, while a dark
//! `text-shadow` halo over a light image moved away and is left out. Frame B
//! is sampled under them: what the glyphs actually sit on. Sampling the whole
//! box instead would read unrelated parts of the image — a block-level
//! heading's box spans the hero's full width while its glyphs cover the left
//! third.
//!
//! A mask is usable when it holds at least [`MIN_GLYPH_CSS_PX2`] CSS px² of
//! glyphs (scaled by the frame's scale factor squared, so it means the same at
//! every DPR) and covers at most [`MAX_GLYPH_SHARE`] of the box. There is no
//! floor on the SHARE: a two-letter label in a full-width block box covers a
//! fraction of a percent of it. An unusable mask is UNKNOWN, never sampled
//! some other way: an empty or sparse one means the text was not hidden in
//! frame B (a hide that missed gradient, canvas or iframe text, a restore
//! before the capture, a capture without hiding) or that the text is invisible
//! against the image, and the two cannot be told apart — sampling the box of
//! a text that was not hidden reads its own glyphs and reports ~1:1.
//!
//! When the box shows glyphs and fewer than that floor of them reach the
//! image, the text is not over the image (a label whose line box overhangs a
//! thumbnail), and is not measured.
//!
//! Only LEAF text elements are measured ([`TextIndex`]): a wrapper whose
//! innerText includes a child's is measured through the child, against the
//! child's colour. Text a wrapper paints directly beside its children is not
//! measured; the snapshot does not separate it.
//!
//! # Against which colour
//!
//! The ratio is WCAG 2.x contrast of the glyph colour against each sampled
//! frame-B pixel. The glyph colour is the element's declared `fg_color` when
//! the snapshot carries one: it is the colour WCAG's definition is stated in,
//! it is exact, and it is free of the anti-aliasing that blends every edge
//! pixel of a glyph in frame A into the background. Only where `fg_color` is
//! absent is the colour MEASURED from frame A, as the per-channel median of
//! the mask's core (the quarter of mask pixels that differ most between the
//! frames, i.e. the least blended); that reading is an estimate, and its
//! confidence is the share of core pixels within [`GLYPH_COLOUR_TOLERANCE`]
//! of the median. A heavy `text-shadow` on a light image differs from frame B
//! by more than the glyph does and can take the core, so a measured colour
//! under such a shadow reads the shadow's colour: declare `fg_color`. The
//! declared colour cannot see a translucent ancestor (`opacity`) or a blend
//! mode; that is the price of preferring it.
//!
//! # What is reported
//!
//! The minimum ratio and the `percentile`-th percentile (nearest rank;
//! [`DEFAULT_PERCENTILE`] = 5) over the sampled pixels. The minimum is one
//! pixel's word — a single bright speck under a stroke sets it — so a check
//! gates on the percentile, and the minimum is reported beside it.
//!
//! APCA is not computed: the APCA-W3 reference implementation is not under a
//! permissive open-source licence, and the plan adds it only after a licence
//! read admits it.

use std::collections::HashSet;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::{unreadable, unrendered, Ancestry, LocateError};
use crate::element_snapshot::{intersection, region_contains, Element, ElementSnapshot, Rgb};
use crate::frame::{Frame, Region};
use crate::observation::UnknownCode;

/// The percentile reported (and gated on, unless the caller says otherwise).
pub const DEFAULT_PERCENTILE: f64 = 5.0;

/// Smallest RGB (Euclidean) distance between frames A and B that marks a
/// glyph pixel. The same 8 the image analyzer's `edge_contrast` uses to tell
/// stroke pixels from their surround: above lossless-capture noise, below
/// the difference any visible glyph paints.
pub const GLYPH_DIFF_MIN: f64 = 8.0;

/// Fewest glyph pixels a usable mask holds, in CSS px² (a dot of a 6 px
/// font's `i`); the buffer floor is this times the scale factor squared.
pub const MIN_GLYPH_CSS_PX2: f64 = 6.0;

/// Largest share of the text box a usable mask covers. Glyphs of even a
/// heavy face in a tight box cover well under this; a larger difference is
/// not glyphs.
pub const MAX_GLYPH_SHARE: f64 = 0.70;

/// Largest share of an image's pixels outside its text that may differ
/// between frames A and B for the two to count as registered.
pub const MAX_UNREGISTERED_SHARE: f64 = 0.005;

/// How far, in CSS px, each text box is widened before the registration
/// check, so anti-aliasing and a modest `text-shadow` are not counted as
/// misregistration.
pub const REGISTRATION_MARGIN_CSS: f64 = 4.0;

/// Fewest image pixels outside the text, in CSS px², the registration check
/// needs before it can confirm anything.
pub const MIN_REGISTRATION_CSS_PX2: f64 = 64.0;

/// RGB distance within which a core glyph pixel agrees with the measured
/// glyph colour.
pub const GLYPH_COLOUR_TOLERANCE: f64 = 24.0;

/// Where the glyph colour came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlyphColourSource {
    /// The snapshot's `fg_color`.
    Declared,
    /// Measured from frame A under the glyph mask (no `fg_color`).
    Measured,
}

/// The contrast of one text element over one image.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextOverImageContrast {
    /// The text element's id.
    pub element: String,
    /// The image element's id.
    pub image: String,
    /// The lowest WCAG ratio over the sampled pixels.
    pub min_ratio: f64,
    /// The percentile reported in [`Self::percentile_ratio`] (0..=100).
    pub percentile: f64,
    /// The `percentile`-th percentile WCAG ratio (nearest rank).
    pub percentile_ratio: f64,
    /// The colour the ratios are computed against.
    pub glyph_colour: Rgb,
    pub glyph_colour_source: GlyphColourSource,
    /// How many frame-B pixels were sampled (glyph pixels over the image).
    pub sampled_px: u64,
    /// The examined buffer region: the text box clipped to the image box,
    /// mapped onto the frame and clamped to it.
    pub region: Region,
    /// 0..=1: the share of the examined box inside the frame, times the
    /// measured glyph colour's agreement when it was measured.
    pub confidence: f64,
}

/// Whether `b` can serve as `a`'s text-hidden twin: the same buffer size and
/// the same scale factor. A pair that differs in either is not two captures
/// of one layout, and no pixel of one can be read at the other's position.
/// That the CONTENT lines up is [`registered`]'s question.
pub fn pair_frames(a: &Frame, b: &Frame) -> Result<(), LocateError> {
    if (a.width, a.height) != (b.width, b.height) {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "the text-hidden frame is {}x{} px and the frame is {}x{} px: two captures of \
                 one layout share their size, so no glyph pixel can be paired across them",
                b.width, b.height, a.width, a.height
            ),
        ));
    }
    let (sa, sb) = (a.source.scale_factor, b.source.scale_factor);
    if !(sa.is_finite() && sb.is_finite() && (sa - sb).abs() <= 1e-9 * sa.abs().max(1.0)) {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "the text-hidden frame was captured at scale factor {sb} and the frame at {sa}: \
                 the same CSS box lands on different pixels, so no glyph pixel can be paired"
            ),
        ));
    }
    Ok(())
}

/// One text element over one image, and the CSS-px region where they overlap.
#[derive(Debug, Clone, Copy)]
pub struct TextOverImage<'a> {
    pub text: &'a Element,
    pub image: &'a Element,
    pub css: Region,
}

/// Whether `t` is a text element that paints: non-blank text, not itself
/// media, not [`unrendered`].
fn paints_text(snapshot: &ElementSnapshot, t: &Element) -> bool {
    t.media.is_none()
        && t.text.as_deref().is_some_and(|s| !s.trim().is_empty())
        && unrendered(snapshot, t).is_none()
}

/// Which painted text elements are LEAVES: no other painted text element
/// descends from them (`parent_id` chain or `children_ids`, as
/// [`super::drop_nested`] reads it). Built once per snapshot.
#[derive(Debug)]
pub struct TextIndex<'a> {
    leaves: HashSet<&'a str>,
}

impl<'a> TextIndex<'a> {
    pub fn of(snapshot: &'a ElementSnapshot) -> Self {
        let ancestry = Ancestry::of(snapshot);
        let texts: Vec<&'a str> = snapshot
            .elements
            .iter()
            .filter(|t| paints_text(snapshot, t))
            .map(|t| t.id.as_str())
            .collect();
        let ancestors: Vec<HashSet<&'a str>> =
            texts.iter().map(|id| ancestry.ancestors(id)).collect();
        // `w` wraps `c` when `w` is among `c`'s ancestors and not the other
        // way round (a cycle in the producer's chain is not a nesting).
        let wrappers: HashSet<&'a str> = texts
            .iter()
            .enumerate()
            .filter(|(wi, w)| {
                texts.iter().enumerate().any(|(ci, _)| {
                    ci != *wi && ancestors[ci].contains(*w) && !ancestors[*wi].contains(texts[ci])
                })
            })
            .map(|(_, w)| *w)
            .collect();
        Self {
            leaves: texts
                .into_iter()
                .filter(|t| !wrappers.contains(t))
                .collect(),
        }
    }

    /// Whether `id` is a painted leaf text element.
    pub fn is_leaf(&self, id: &str) -> bool {
        self.leaves.contains(id)
    }
}

/// The LEAF text elements laid over `image` ([`TextIndex`]), each with the
/// overlap region, and how many were left out because the TEXT cannot be
/// attributed to the frame ([`unreadable`]). A text box that contains the
/// image whole is a container (a section, a card, a caption wrapper), not an
/// overlay, as the image analyzer's busyness-under-text reads it.
///
/// The image itself is gated by the caller on [`unrendered`] only: an
/// OCCLUDED image (the usual `<img>` hero under an inset-0 scrim or text
/// layer) is measured, because nothing here reads the image's crop — frame B
/// shows whatever the glyphs sit on.
pub fn texts_over<'a>(
    snapshot: &'a ElementSnapshot,
    image: &'a Element,
    index: &TextIndex<'_>,
) -> (Vec<TextOverImage<'a>>, usize) {
    let Some(img) = image.bbox else {
        return (Vec::new(), 0);
    };
    let mut out = Vec::new();
    let mut unreadable_text = 0usize;
    for t in &snapshot.elements {
        if t.id == image.id || !index.is_leaf(&t.id) {
            continue;
        }
        let Some(tb) = t.bbox else { continue };
        if region_contains(tb, img) {
            continue;
        }
        let Some(css) = intersection(tb, img) else {
            continue;
        };
        if unreadable(snapshot, t).is_some() {
            unreadable_text += 1;
            continue;
        }
        out.push(TextOverImage {
            text: t,
            image,
            css,
        });
    }
    (out, unreadable_text)
}

/// Every text-over-image pair on the page, and what was left out.
#[derive(Debug, Default)]
pub struct PagePairs<'a> {
    pub pairs: Vec<TextOverImage<'a>>,
    /// Unrendered images (they paint nothing) with text over them.
    pub skipped_images: usize,
    /// Text elements over an image that cannot themselves be attributed to
    /// the frame.
    pub skipped_texts: usize,
}

impl PagePairs<'_> {
    /// The disclosure a page-wide detail carries.
    pub fn note(&self) -> String {
        let mut note = String::new();
        if self.skipped_images > 0 {
            note.push_str(&format!(
                " ({} unrendered image(s) with text over them skipped)",
                self.skipped_images
            ));
        }
        if self.skipped_texts > 0 {
            note.push_str(&format!(
                " ({} text element(s) over an image skipped: reported not visible)",
                self.skipped_texts
            ));
        }
        note
    }

    /// The distinct images the pairs lie over, in snapshot order.
    pub fn images(&self) -> Vec<&Element> {
        let mut seen = HashSet::new();
        self.pairs
            .iter()
            .filter(|p| seen.insert(p.image.id.as_str()))
            .map(|p| p.image)
            .collect()
    }
}

/// [`texts_over`] every rendered media element of the snapshot.
pub fn page_pairs(snapshot: &ElementSnapshot) -> PagePairs<'_> {
    let index = TextIndex::of(snapshot);
    let mut page = PagePairs::default();
    for image in snapshot.elements.iter().filter(|e| e.media.is_some()) {
        let (pairs, skipped_texts) = texts_over(snapshot, image, &index);
        if unrendered(snapshot, image).is_some() {
            if !pairs.is_empty() || skipped_texts > 0 {
                page.skipped_images += 1;
            }
            continue;
        }
        page.skipped_texts += skipped_texts;
        page.pairs.extend(pairs);
    }
    page
}

// ---------------------------------------------------------------------------
// Pixels
// ---------------------------------------------------------------------------

fn rgb_at(frame: &Frame, x: u32, y: u32) -> Rgb {
    let p = frame.buffer.get_pixel(x, y).0;
    Rgb::new(p[0], p[1], p[2])
}

fn dist2(a: Rgb, b: Rgb) -> f64 {
    let (dr, dg, db) = (
        a.r as f64 - b.r as f64,
        a.g as f64 - b.g as f64,
        a.b as f64 - b.b as f64,
    );
    dr * dr + dg * dg + db * db
}

/// The sRGB channel linearisation, once per value: the same formula as
/// [`Rgb::relative_luminance`], so [`contrast`] equals
/// [`crate::analyzers::color::wcag_contrast`] exactly.
fn linear(c: u8) -> f64 {
    static LUT: OnceLock<[f64; 256]> = OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0.0; 256];
        for (i, v) in t.iter_mut().enumerate() {
            let s = i as f64 / 255.0;
            *v = if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            };
        }
        t
    })[c as usize]
}

fn luminance(c: Rgb) -> f64 {
    0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
}

/// WCAG contrast between two relative luminances.
fn contrast(la: f64, lb: f64) -> f64 {
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// The nearest-rank `percentile` (0..=100) of `sorted` (ascending, non-empty).
/// The rank is `percentile * n / 100`, multiplied first: `p / 100 * n` reads
/// 7 / 100 * 100 as 7.000000000000001 and takes the eighth value.
pub fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    let n = sorted.len();
    let rank = (percentile * n as f64 / 100.0).ceil() as usize;
    sorted[rank.clamp(1, n) - 1]
}

/// The buffer-px floor of a usable glyph mask at scale factor `s`.
pub fn min_glyph_px(s: f64) -> usize {
    ((MIN_GLYPH_CSS_PX2 * s * s).ceil() as usize).max(1)
}

/// Whether a glyph mask of `glyphs` pixels over a box of `area` pixels reads
/// as glyphs at scale factor `s` (see the module docs).
pub fn mask_usable(glyphs: usize, area: usize, s: f64) -> bool {
    glyphs >= min_glyph_px(s) && glyphs as f64 <= MAX_GLYPH_SHARE * area.max(1) as f64
}

/// A CSS-px box on the frame: mapped through the scale factor and clamped,
/// as `(x, y, w, h)` buffer indices plus the share of it inside the frame.
fn on_frame(frame: &Frame, css: Region) -> Option<((u32, u32, u32, u32), f64)> {
    let mapped = frame.css_region_to_pixels(css)?;
    let px = mapped
        .clamp_to_frame(frame.width, frame.height)
        .filter(|(_, _, w, h)| *w > 0 && *h > 0)?;
    let in_frame = (px.2 as f64 * px.3 as f64 / (mapped.w as f64 * mapped.h as f64)).min(1.0);
    Some((px, in_frame))
}

/// The pixels of `px` where frames `a` and `b` differ by more than
/// [`GLYPH_DIFF_MIN`], as `(x, y, squared distance)`. `None` as soon as more
/// than `cap` pixels differ: the caller has no use for a mask that dense.
fn diff_mask(
    a: &Frame,
    b: &Frame,
    (x0, y0, w, h): (u32, u32, u32, u32),
    cap: usize,
) -> Option<Vec<(u32, u32, f64)>> {
    let min2 = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
    let mut mask = Vec::new();
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let d = dist2(rgb_at(a, x, y), rgb_at(b, x, y));
            if d > min2 {
                if mask.len() == cap {
                    return None;
                }
                mask.push((x, y, d));
            }
        }
    }
    Some(mask)
}

/// Whether frames `a` and `b` are REGISTERED over `image`: inside its box,
/// outside every text box over it (each widened by
/// [`REGISTRATION_MARGIN_CSS`]), at most [`MAX_UNREGISTERED_SHARE`] of the
/// pixels differ. Every painted text box counts here, wrappers and
/// not-visible ones included, so no glyph is mistaken for misregistration.
/// The caller has already paired the frames ([`pair_frames`]).
pub fn registered(
    a: &Frame,
    b: &Frame,
    snapshot: &ElementSnapshot,
    image: &Element,
) -> Result<(), LocateError> {
    let id = &image.id;
    let s = a.source.scale_factor;
    let img = image.bbox.ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!(
                "image '{id}' carries no bbox, so the two frames' registration over it is unknown"
            ),
        )
    })?;
    let (ix, iy, iw, ih) = on_frame(a, img).map(|p| p.0).ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!("image '{id}' lies outside the frame, so the two frames cannot be registered over it"),
        )
    })?;
    let margin = (REGISTRATION_MARGIN_CSS * s).ceil() as i64;
    let holes: Vec<(i64, i64, i64, i64)> = snapshot
        .elements
        .iter()
        .filter(|t| t.id != image.id && paints_text(snapshot, t))
        .filter_map(|t| {
            let tb = t.bbox?;
            if region_contains(tb, img) {
                return None;
            }
            intersection(tb, img)?;
            let r = a.css_region_to_pixels(tb)?;
            Some((
                r.x as i64 - margin,
                r.y as i64 - margin,
                r.right() + margin,
                r.bottom() + margin,
            ))
        })
        .collect();
    let min2 = GLYPH_DIFF_MIN * GLYPH_DIFF_MIN;
    let (mut considered, mut differing) = (0usize, 0usize);
    for y in iy..iy + ih {
        for x in ix..ix + iw {
            let (xi, yi) = (x as i64, y as i64);
            if holes
                .iter()
                .any(|(x0, y0, x1, y1)| xi >= *x0 && xi < *x1 && yi >= *y0 && yi < *y1)
            {
                continue;
            }
            considered += 1;
            if dist2(rgb_at(a, x, y), rgb_at(b, x, y)) > min2 {
                differing += 1;
            }
        }
    }
    let floor = (MIN_REGISTRATION_CSS_PX2 * s * s).ceil() as usize;
    if considered < floor.max(1) {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "only {considered} pixel(s) of image '{id}' lie outside its text, under the {floor} \
                 needed to confirm the text-hidden frame is registered with the frame"
            ),
        ));
    }
    let share = differing as f64 / considered as f64;
    if share > MAX_UNREGISTERED_SHARE {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "the text-hidden frame is not registered with the frame over image '{id}': \
                 {:.2}% of its pixels away from any text differ (at most {:.2}% may), so it is \
                 shifted, rescaled, mid-animation, another capture, or hides text the snapshot \
                 does not carry",
                share * 100.0,
                MAX_UNREGISTERED_SHARE * 100.0
            ),
        ));
    }
    Ok(())
}

/// The glyph colour measured from frame A over the mask's core, and how far
/// the core agrees with it (0..=1). `mask` is `(x, y, squared A−B distance)`.
fn measured_glyph_colour(a: &Frame, mask: &[(u32, u32, f64)]) -> (Rgb, f64) {
    let mut by_diff: Vec<&(u32, u32, f64)> = mask.iter().collect();
    by_diff.sort_by(|p, q| q.2.total_cmp(&p.2));
    let core: Vec<Rgb> = by_diff[..mask.len().div_ceil(4)]
        .iter()
        .map(|(x, y, _)| rgb_at(a, *x, *y))
        .collect();
    let median = |f: fn(&Rgb) -> u8| {
        let mut v: Vec<u8> = core.iter().map(f).collect();
        v.sort_unstable();
        v[v.len() / 2]
    };
    let colour = Rgb::new(median(|c| c.r), median(|c| c.g), median(|c| c.b));
    let tol2 = GLYPH_COLOUR_TOLERANCE * GLYPH_COLOUR_TOLERANCE;
    let agree =
        core.iter().filter(|c| dist2(**c, colour) <= tol2).count() as f64 / core.len() as f64;
    (colour, agree)
}

/// Measure one text element over one image on the frame pair `(a, b)`, which
/// the caller has already paired ([`pair_frames`]) and registered over the
/// image ([`registered`]). See the module docs.
///
/// `Ok(None)` when the text's box shows glyphs and fewer than the glyph floor
/// of them reach the image: not text over it. `Err` (Unknown) when the box
/// shows no usable mask, or when the pixels over the image did not move
/// toward the glyph colour.
pub fn measure(
    a: &Frame,
    b: &Frame,
    pair: &TextOverImage<'_>,
    percentile: f64,
) -> Result<Option<TextOverImageContrast>, LocateError> {
    let id = pair.text.id.as_str();
    let s = a.source.scale_factor;
    let over_frame = |what: &str| {
        (
            UnknownCode::InputMissing,
            format!(
                "{what} of text '{id}' over '{}' cannot be placed on the frame (it lies outside \
                 it, or the scale factor {s} is unusable)",
                pair.image.id
            ),
        )
    };
    let text_box = pair.text.bbox.ok_or_else(|| over_frame("the box"))?;
    let (over_px, in_frame) = on_frame(a, pair.css).ok_or_else(|| over_frame("the overlap"))?;
    let (box_px, _) = on_frame(a, text_box).ok_or_else(|| over_frame("the box"))?;
    let area = box_px.2 as usize * box_px.3 as usize;
    let floor = min_glyph_px(s);
    let cap = (MAX_GLYPH_SHARE * area as f64) as usize;
    let box_mask = diff_mask(a, b, box_px, cap).ok_or_else(|| {
        (
            UnknownCode::InputMissing,
            format!(
                "frames A and B differ at more than {:.0}% of text '{id}''s box, which is not \
                 glyphs, so no contrast was measured",
                MAX_GLYPH_SHARE * 100.0
            ),
        )
    })?;
    if !mask_usable(box_mask.len(), area, s) {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "frames A and B differ at {} pixel(s) of text '{id}''s box, fewer than the {floor} \
                 a glyph mask needs: either the text was not hidden in the text-hidden frame or it \
                 is invisible against the image, and the two cannot be told apart, so no contrast \
                 was measured",
                box_mask.len()
            ),
        ));
    }

    let (ox0, oy0, ow, oh) = over_px;
    let over = |x: u32, y: u32| x >= ox0 && x < ox0 + ow && y >= oy0 && y < oy0 + oh;
    if box_mask.iter().filter(|(x, y, _)| over(*x, *y)).count() < floor {
        return Ok(None);
    }

    let (glyph_colour, source, colour_conf) = match pair.text.fg_color {
        Some(fg) => (fg, GlyphColourSource::Declared, 1.0),
        None => {
            let (c, agree) = measured_glyph_colour(a, &box_mask);
            (c, GlyphColourSource::Measured, agree)
        }
    };
    let lg = luminance(glyph_colour);
    let mut ratios: Vec<f64> = box_mask
        .iter()
        .filter(|(x, y, _)| over(*x, *y))
        .filter_map(|(x, y, _)| {
            let (pa, pb) = (rgb_at(a, *x, *y), rgb_at(b, *x, *y));
            // Moved toward the glyph colour: a glyph or its edge, not a halo.
            (dist2(pa, glyph_colour) < dist2(pb, glyph_colour)).then(|| contrast(lg, luminance(pb)))
        })
        .collect();
    if ratios.len() < floor {
        return Err((
            UnknownCode::InputMissing,
            format!(
                "the pixels where text '{id}' differs over '{}' did not move toward its glyph \
                 colour rgb({},{},{}) ({} of them did, under the {floor} needed): the declared \
                 colour is not what paints, or only a shadow lies over the image",
                pair.image.id,
                glyph_colour.r,
                glyph_colour.g,
                glyph_colour.b,
                ratios.len()
            ),
        ));
    }
    ratios.sort_by(f64::total_cmp);
    Ok(Some(TextOverImageContrast {
        element: id.to_string(),
        image: pair.image.id.clone(),
        min_ratio: ratios[0],
        percentile,
        percentile_ratio: nearest_rank(&ratios, percentile),
        glyph_colour,
        glyph_colour_source: source,
        sampled_px: ratios.len() as u64,
        region: Region {
            x: ox0 as i32,
            y: oy0 as i32,
            w: ow,
            h: oh,
        },
        confidence: (in_frame * colour_conf).clamp(0.0, 1.0),
    }))
}

/// A caller-supplied percentile, refused unless it is a finite 0..=100.
pub fn valid_percentile(p: f64) -> Result<f64, LocateError> {
    if p.is_finite() && (0.0..=100.0).contains(&p) {
        Ok(p)
    } else {
        Err((
            UnknownCode::InputMissing,
            format!("percentile {p} is not a number in 0..=100, so nothing was measured"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzers::color::wcag_contrast;

    #[test]
    fn nearest_rank_reads_the_minimum_at_zero_and_the_maximum_at_100() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(nearest_rank(&v, 0.0), 1.0);
        assert_eq!(nearest_rank(&v, 5.0), 5.0);
        assert_eq!(nearest_rank(&v, 50.0), 50.0);
        assert_eq!(nearest_rank(&v, 100.0), 100.0);
        assert_eq!(nearest_rank(&[3.0], 5.0), 3.0);
    }

    /// `7 / 100 * 100` is 7.000000000000001 in f64, whose ceiling is the
    /// eighth rank; multiplying first gives exactly 7.
    #[test]
    fn nearest_rank_is_not_off_by_one_from_float_error() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        for p in 1..=100 {
            assert_eq!(nearest_rank(&v, p as f64), p as f64, "p{p}");
        }
    }

    #[test]
    fn a_mask_is_usable_between_its_absolute_floor_and_its_share_ceiling() {
        assert!(!mask_usable(0, 1000, 1.0));
        assert!(!mask_usable(5, 100, 1.0), "under MIN_GLYPH_CSS_PX2");
        assert!(
            mask_usable(100, 1280 * 28, 1.0),
            "no share floor: a short label in a wide box"
        );
        assert!(!mask_usable(800, 1000, 1.0), "over MAX_GLYPH_SHARE");
        assert_eq!(min_glyph_px(2.0), 24, "the floor scales with DPR²");
        assert!(!mask_usable(10, 1000, 2.0));
    }

    #[test]
    fn the_luminance_table_matches_wcag_contrast_exactly() {
        for &(a, b) in &[
            (Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)),
            (Rgb::new(250, 250, 250), Rgb::new(215, 210, 225)),
            (Rgb::new(10, 200, 30), Rgb::new(9, 10, 11)),
            (Rgb::new(128, 128, 128), Rgb::new(128, 128, 128)),
        ] {
            assert_eq!(contrast(luminance(a), luminance(b)), wcag_contrast(a, b));
        }
    }

    #[test]
    fn a_percentile_outside_0_to_100_is_refused() {
        for bad in [-1.0, 100.5, f64::NAN, f64::INFINITY] {
            assert!(valid_percentile(bad).is_err(), "{bad}");
        }
        assert_eq!(valid_percentile(0.0).unwrap(), 0.0);
        assert_eq!(valid_percentile(100.0).unwrap(), 100.0);
    }
}
