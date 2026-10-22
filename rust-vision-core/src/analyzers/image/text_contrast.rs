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
//! # Where the glyphs are
//!
//! The two frames differ exactly where glyphs paint, so the GLYPH MASK of a
//! text element is the set of buffer pixels inside its box (clipped to the
//! image it lies over) where frames A and B differ by more than
//! [`GLYPH_DIFF_MIN`]. Frame B is then sampled under that mask: what the
//! glyphs actually sit on. Sampling the whole box instead would read
//! unrelated parts of the image — a block-level heading's box spans the
//! hero's full width while its glyphs cover the left third.
//!
//! The whole box is a FALLBACK only, at lower confidence
//! ([`BBOX_FALLBACK_CONFIDENCE`]): when the mask over the text's WHOLE box
//! is empty or too sparse to be glyphs (fewer than [`MIN_GLYPH_PIXELS`]
//! pixels, or under [`MIN_GLYPH_SHARE`] of the box — the text was not
//! hidden, or it is invisible against the image), or so dense it cannot be
//! glyphs (over [`MAX_GLYPH_SHARE`] — the image itself changed between the
//! captures). When the whole box does show glyphs and they do not reach the
//! image, the text is not over the image at all, and is not measured (see
//! [`measure`]).
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
//! of the median. The declared colour cannot see a translucent ancestor
//! (`opacity`) or a blend mode; that is the price of preferring it, and it is
//! stated here rather than discovered.
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

use serde::{Deserialize, Serialize};

use super::{unreadable, unrendered, LocateError};
use crate::analyzers::color::wcag_contrast;
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

/// Fewest glyph pixels a usable mask holds.
pub const MIN_GLYPH_PIXELS: usize = 4;

/// Smallest share of the examined box a usable mask covers (as
/// `edge_contrast`'s 0.5% stroke floor).
pub const MIN_GLYPH_SHARE: f64 = 0.005;

/// Largest share of the examined box a usable mask covers. Glyphs of even a
/// heavy face in a tight box cover well under this; a larger difference is
/// the image changing between the captures (a carousel, a video).
pub const MAX_GLYPH_SHARE: f64 = 0.70;

/// The confidence ceiling of the whole-box fallback: it reads every pixel of
/// the box, glyph-free ones included, so it is ranked below any mask reading.
pub const BBOX_FALLBACK_CONFIDENCE: f64 = 0.5;

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

/// Which pixels of frame B were sampled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleMethod {
    /// Under the glyph mask (where frames A and B differ).
    GlyphMask,
    /// The whole examined box: the mask was empty or unusable.
    BboxFallback,
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
    pub method: SampleMethod,
    /// How many frame-B pixels were sampled.
    pub sampled_px: u64,
    /// The examined buffer region: the text box clipped to the image box,
    /// mapped onto the frame and clamped to it.
    pub region: Region,
    /// 0..=1: the share of the examined box inside the frame, times the
    /// glyph colour's agreement when it was measured, times
    /// [`BBOX_FALLBACK_CONFIDENCE`] on the fallback.
    pub confidence: f64,
}

/// Whether `b` can serve as `a`'s text-hidden twin: the same buffer size and
/// the same scale factor. A pair that differs in either is not two captures
/// of one layout, and no pixel of one can be read at the other's position.
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

/// Why an image takes no part: it paints nothing ([`unrendered`]) or its
/// pixels cannot be attributed ([`unreadable`]). `None` for a readable image.
pub fn image_unreadable(snapshot: &ElementSnapshot, image: &Element) -> Option<String> {
    unrendered(snapshot, image).or_else(|| unreadable(snapshot, image))
}

/// The text elements laid over `image`, each with the overlap region, and
/// how many were left out because the TEXT cannot be attributed to the frame
/// ([`unreadable`]). A text element qualifies when it carries non-blank text,
/// is not itself media, paints ([`unrendered`] is `None`), and its box
/// intersects the image's without containing it whole — a box that contains
/// the image is a container (a section, a card, a caption wrapper), not an
/// overlay, as the image analyzer's busyness-under-text reads it.
///
/// The image's own readability is the caller's to check
/// ([`image_unreadable`]); this reads geometry only.
pub fn texts_over<'a>(
    snapshot: &'a ElementSnapshot,
    image: &'a Element,
) -> (Vec<TextOverImage<'a>>, usize) {
    let Some(img) = image.bbox else {
        return (Vec::new(), 0);
    };
    let mut out = Vec::new();
    let mut unreadable_text = 0usize;
    for t in &snapshot.elements {
        if t.id == image.id
            || t.media.is_some()
            || t.text.as_deref().is_none_or(|s| s.trim().is_empty())
            || unrendered(snapshot, t).is_some()
        {
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
    /// Images that paint nothing or cannot be attributed, with text over them.
    pub skipped_images: usize,
    /// Text elements over a readable image that cannot themselves be
    /// attributed to the frame.
    pub skipped_texts: usize,
}

impl PagePairs<'_> {
    /// The disclosure a page-wide detail carries.
    pub fn note(&self) -> String {
        let mut note = String::new();
        if self.skipped_images > 0 {
            note.push_str(&format!(
                " ({} image(s) with text over them skipped: unrendered or unreadable)",
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
}

/// [`texts_over`] every readable media element of the snapshot.
pub fn page_pairs(snapshot: &ElementSnapshot) -> PagePairs<'_> {
    let mut page = PagePairs::default();
    for image in snapshot.elements.iter().filter(|e| e.media.is_some()) {
        let (pairs, skipped_texts) = texts_over(snapshot, image);
        if image_unreadable(snapshot, image).is_some() {
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

fn rgb_at(frame: &Frame, x: u32, y: u32) -> Rgb {
    let p = frame.buffer.get_pixel(x, y).0;
    Rgb::new(p[0], p[1], p[2])
}

fn dist(a: Rgb, b: Rgb) -> f64 {
    let (dr, dg, db) = (
        a.r as f64 - b.r as f64,
        a.g as f64 - b.g as f64,
        a.b as f64 - b.b as f64,
    );
    (dr * dr + dg * dg + db * db).sqrt()
}

/// The nearest-rank `percentile` (0..=100) of `sorted` (ascending, non-empty).
pub fn nearest_rank(sorted: &[f64], percentile: f64) -> f64 {
    let n = sorted.len();
    let rank = (percentile / 100.0 * n as f64).ceil() as usize;
    sorted[rank.clamp(1, n) - 1]
}

/// Whether a glyph mask of `glyphs` pixels over a box of `area` pixels reads
/// as glyphs (see the module docs).
pub fn mask_usable(glyphs: usize, area: usize) -> bool {
    let share = glyphs as f64 / area.max(1) as f64;
    glyphs >= MIN_GLYPH_PIXELS && (MIN_GLYPH_SHARE..=MAX_GLYPH_SHARE).contains(&share)
}

/// The glyph colour measured from frame A over the mask's core, and how far
/// the core agrees with it (0..=1). `mask` is `(x, y, A−B distance)`.
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
    let agree = core
        .iter()
        .filter(|c| dist(**c, colour) <= GLYPH_COLOUR_TOLERANCE)
        .count() as f64
        / core.len() as f64;
    (colour, agree)
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
/// [`GLYPH_DIFF_MIN`], as `(x, y, distance)`.
fn diff_mask(a: &Frame, b: &Frame, (x0, y0, w, h): (u32, u32, u32, u32)) -> Vec<(u32, u32, f64)> {
    let mut mask = Vec::new();
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let d = dist(rgb_at(a, x, y), rgb_at(b, x, y));
            if d > GLYPH_DIFF_MIN {
                mask.push((x, y, d));
            }
        }
    }
    mask
}

/// Measure one text element over one image on the frame pair `(a, b)`,
/// which the caller has already paired ([`pair_frames`]). See the module docs.
///
/// The mask is read twice: over the text's WHOLE box, to establish that the
/// hide worked (a usable mask there means the glyphs were found), and over
/// the part of the box that lies over the image, which is what is sampled.
/// When the whole box shows glyphs and fewer than [`MIN_GLYPH_PIXELS`] of
/// them fall over the image, the text's box touches the image but its
/// glyphs do not (a label beside an avatar, a heading whose box overhangs a
/// thumbnail): that is `Ok(None)`, not text over an image. The whole-box
/// fallback is taken only when the whole box shows no usable mask.
pub fn measure(
    a: &Frame,
    b: &Frame,
    pair: &TextOverImage<'_>,
    percentile: f64,
) -> Result<Option<TextOverImageContrast>, LocateError> {
    let id = pair.text.id.as_str();
    let over_frame = |what: &str| {
        (
            UnknownCode::InputMissing,
            format!(
                "{what} of text '{id}' over '{}' cannot be placed on the frame (it lies outside \
                 it, or the scale factor {} is unusable)",
                pair.image.id, a.source.scale_factor
            ),
        )
    };
    let text_box = pair.text.bbox.ok_or_else(|| over_frame("the box"))?;
    let (over_px, in_frame) = on_frame(a, pair.css).ok_or_else(|| over_frame("the overlap"))?;
    let (box_px, _) = on_frame(a, text_box).ok_or_else(|| over_frame("the box"))?;
    let box_mask = diff_mask(a, b, box_px);
    let hide_worked = mask_usable(box_mask.len(), box_px.2 as usize * box_px.3 as usize);
    let over_mask = diff_mask(a, b, over_px);
    if hide_worked && over_mask.len() < MIN_GLYPH_PIXELS {
        return Ok(None);
    }

    let (glyph_colour, source, colour_conf) = match pair.text.fg_color {
        Some(fg) => (fg, GlyphColourSource::Declared, 1.0),
        None if hide_worked => {
            let (c, agree) = measured_glyph_colour(a, &box_mask);
            (c, GlyphColourSource::Measured, agree)
        }
        None => {
            return Err((
                UnknownCode::InputMissing,
                format!(
                    "text '{id}' declares no fg_color, and frames A and B differ at {} pixel(s) \
                     of its box, which is not a glyph mask (the text was not hidden, or it is \
                     invisible against the image), so its glyph colour could not be measured",
                    box_mask.len()
                ),
            ))
        }
    };

    let (method, mut ratios): (SampleMethod, Vec<f64>) = if hide_worked {
        (
            SampleMethod::GlyphMask,
            over_mask
                .iter()
                .map(|(x, y, _)| wcag_contrast(glyph_colour, rgb_at(b, *x, *y)))
                .collect(),
        )
    } else {
        let (x0, y0, w, h) = over_px;
        let mut all = Vec::with_capacity(w as usize * h as usize);
        for y in y0..y0 + h {
            for x in x0..x0 + w {
                all.push(wcag_contrast(glyph_colour, rgb_at(b, x, y)));
            }
        }
        (SampleMethod::BboxFallback, all)
    };
    ratios.sort_by(f64::total_cmp);
    let fallback = match method {
        SampleMethod::BboxFallback => BBOX_FALLBACK_CONFIDENCE,
        SampleMethod::GlyphMask => 1.0,
    };
    Ok(Some(TextOverImageContrast {
        element: id.to_string(),
        image: pair.image.id.clone(),
        min_ratio: ratios[0],
        percentile,
        percentile_ratio: nearest_rank(&ratios, percentile),
        glyph_colour,
        glyph_colour_source: source,
        method,
        sampled_px: ratios.len() as u64,
        region: Region {
            x: over_px.0 as i32,
            y: over_px.1 as i32,
            w: over_px.2,
            h: over_px.3,
        },
        confidence: (in_frame * colour_conf * fallback).clamp(0.0, 1.0),
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

    #[test]
    fn nearest_rank_reads_the_minimum_at_zero_and_the_maximum_at_100() {
        let v: Vec<f64> = (1..=100).map(f64::from).collect();
        assert_eq!(nearest_rank(&v, 0.0), 1.0);
        assert_eq!(nearest_rank(&v, 5.0), 5.0);
        assert_eq!(nearest_rank(&v, 50.0), 50.0);
        assert_eq!(nearest_rank(&v, 100.0), 100.0);
        assert_eq!(nearest_rank(&[3.0], 5.0), 3.0);
    }

    #[test]
    fn a_mask_is_usable_only_between_its_floors() {
        assert!(!mask_usable(0, 1000));
        assert!(!mask_usable(3, 100), "under MIN_GLYPH_PIXELS");
        assert!(!mask_usable(4, 10_000), "under MIN_GLYPH_SHARE");
        assert!(mask_usable(100, 1000));
        assert!(!mask_usable(800, 1000), "over MAX_GLYPH_SHARE");
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
