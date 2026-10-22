//! The L1 image assertions (plan 2026-10-13-image-generation-and-visual-
//! judgment, Phase 15): the first evaluators that read
//! [`EvalContext::frame`].
//!
//! They share their measurements with [`crate::analyzers::image`], so the
//! value an assertion judges is the value the analyzer reports. Every
//! threshold is a parameter (PD3): the analyzer cannot see a tenant's visual
//! system or a slot spec, and an undeclared threshold answers
//! `Unknown{input_missing}` — never a pass.
//!
//! [`Assertion::TextOverImageContrast`] (Phase 16) lives here too: it reads
//! [`EvalContext::frame`] and its text-hidden twin, and measures through
//! [`crate::analyzers::image::text_contrast`].
//!
//! Confidence: see [`AssertionResult::confidence`]. Each evaluator that
//! reads pixels sets it; each one's doc says what scales it.

use super::{missing_operand, require_snapshot, Assertion, AssertionResult, EvalContext};
use crate::analyzers::image::text_contrast::{self as tc, TextOverImageContrast};
use crate::analyzers::image::{self as img, pixels, Crop, NaturalRegion};
use crate::element_media::{ElementMedia, MediaKind};
use crate::element_snapshot::{region_contains, Element, ElementSnapshot, Rgb};
use crate::frame::Frame;
use crate::observation::UnknownCode;

/// Dispatch one of the image variants. Called by [`super::evaluate`] for
/// those variants only.
pub(super) fn evaluate(a: Assertion, ctx: &EvalContext<'_>) -> AssertionResult {
    match a.clone() {
        Assertion::ImagePaletteWithin {
            element,
            palette,
            max_delta_e,
            min_coverage,
        } => palette_within(a, &element, &palette, max_delta_e, min_coverage, ctx),
        Assertion::NoNearDuplicateImages { max_hamming } => no_near_duplicates(a, max_hamming, ctx),
        Assertion::NoPlaceholderImages { element } => no_placeholders(a, element.as_deref(), ctx),
        Assertion::ImageFocalVisible { element, focal } => focal_visible(a, &element, focal, ctx),
        Assertion::TransparentImageLegible { element, min_ratio } => {
            transparent_legible(a, &element, min_ratio, ctx)
        }
        Assertion::SalienceOrder { cta_id } => salience_order(a, &cta_id, ctx),
        Assertion::ImageSharpnessAtLeast {
            element,
            min_sharpness,
        } => sharpness_at_least(a, &element, min_sharpness, ctx),
        Assertion::ImageBlockinessWithin {
            element,
            max_blockiness,
        } => blockiness_within(a, &element, max_blockiness, ctx),
        Assertion::TextOverImageContrast {
            min_ratio,
            percentile,
        } => text_over_image(a, min_ratio, percentile, ctx),
        other => unreachable!("not an image assertion: {other:?}"),
    }
}

/// The detail every undeclared threshold answers with (plan Phase 8's rule,
/// shared here).
fn undeclared(a: Assertion, name: &str) -> AssertionResult {
    AssertionResult::unknown(
        a,
        UnknownCode::InputMissing,
        format!(
            "threshold `{name}` undeclared in domain_spec/visual-system (or the slot spec), so \
             nothing was judged — an undeclared threshold is never a pass"
        ),
    )
}

/// A required threshold, finite.
fn threshold(a: &Assertion, v: Option<f64>, name: &str) -> Result<f64, AssertionResult> {
    match v {
        Some(t) if t.is_finite() => Ok(t),
        Some(t) => Err(AssertionResult::unknown(
            a.clone(),
            UnknownCode::InputMissing,
            format!("threshold `{name}` is {t}, not a finite number, so nothing was judged"),
        )),
        None => Err(undeclared(a.clone(), name)),
    }
}

fn require_frame<'a>(
    a: &Assertion,
    ctx: &'a EvalContext<'_>,
) -> Result<&'a Frame, AssertionResult> {
    ctx.frame.ok_or_else(|| {
        AssertionResult::unknown(
            a.clone(),
            UnknownCode::InputMissing,
            "no frame supplied, so the image's pixels were never read",
        )
    })
}

/// The named element, with its media.
fn media_element<'a>(
    a: &Assertion,
    snap: &'a ElementSnapshot,
    id: &str,
) -> Result<(&'a Element, &'a ElementMedia), AssertionResult> {
    let el = snap.get(id).ok_or_else(|| {
        AssertionResult::unknown(
            a.clone(),
            UnknownCode::InputMissing,
            missing_operand(id, "no image was measured"),
        )
    })?;
    let media = el.media.as_ref().ok_or_else(|| {
        AssertionResult::unknown(
            a.clone(),
            UnknownCode::InputMissing,
            format!(
                "element '{id}' carries no `media`, so it is not known to be an image and was \
                 not measured"
            ),
        )
    })?;
    Ok((el, media))
}

fn crop_of(
    a: &Assertion,
    frame: &Frame,
    snap: &ElementSnapshot,
    el: &Element,
) -> Result<Crop, AssertionResult> {
    img::locate(frame, snap, el)
        .map_err(|(code, detail)| AssertionResult::unknown(a.clone(), code, detail))
}

/// Everything an element-scoped pixel assertion needs.
fn located<'a>(
    a: &Assertion,
    id: &str,
    ctx: &'a EvalContext<'_>,
) -> Result<
    (
        &'a Frame,
        &'a ElementSnapshot,
        &'a Element,
        &'a ElementMedia,
        Crop,
    ),
    AssertionResult,
> {
    let snap = require_snapshot(ctx, a)?;
    let (el, media) = media_element(a, snap, id)?;
    let frame = require_frame(a, ctx)?;
    let crop = crop_of(a, frame, snap, el)?;
    Ok((frame, snap, el, media, crop))
}

macro_rules! tri {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(r) => return r,
        }
    };
}

// ---------------------------------------------------------------------------

/// Confidence: the crop's.
fn palette_within(
    a: Assertion,
    id: &str,
    palette: &[Rgb],
    max_delta_e: Option<f64>,
    min_coverage: Option<f64>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let max_de = tri!(threshold(&a, max_delta_e, "max_delta_e"));
    let min_cov = tri!(threshold(&a, min_coverage, "min_coverage"));
    if palette.is_empty() {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            "the declared palette is empty, so there is nothing to be within",
        );
    }
    let (_, _, _, _, crop) = tri!(located(&a, id, ctx));
    let Some((coverage, worst, worst_de)) = img::on_palette_coverage(&crop, palette, max_de) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!("element '{id}' has no pixels"),
        );
    };
    let conf = crop.confidence;
    if coverage + 1e-12 < min_cov {
        AssertionResult::fail(
            a,
            format!(
                "{:.1}% of '{id}' is within ΔE2000 {max_de} of the palette, below the {:.1}% \
                 required; furthest colour rgb({},{},{}) at ΔE2000 {worst_de:.1}",
                coverage * 100.0,
                min_cov * 100.0,
                worst.r,
                worst.g,
                worst.b
            ),
        )
        .with_confidence(conf)
    } else {
        AssertionResult::pass_with(
            a,
            format!(
                "{:.1}% of '{id}' is on-palette (ΔE2000 ≤ {max_de})",
                coverage * 100.0
            ),
        )
        .with_confidence(conf)
    }
}

/// A media element and its crop.
type ImageCrop<'a> = (&'a Element, &'a ElementMedia, Crop);

/// What a page-wide image assertion examined, and what it left out.
struct PageCrops<'a> {
    /// Every examined image and its crop.
    crops: Vec<ImageCrop<'a>>,
    /// Images left out as unrendered ([`img::unrendered`]: a zero-area box,
    /// or reported `hidden` / `no-layout` — nothing of them is on the frame).
    unrendered: usize,
    /// CSS-background rows, left out with a disclosed note: their painted
    /// region is set by background-size, -position and -repeat, which no
    /// producer puts on the wire, so making them Unknown would make every
    /// page-wide image check Unknown on every page that has one, forever.
    /// Their pixel-free placeholder MARKER is still read
    /// ([`no_placeholders`]).
    css_backgrounds: Vec<&'a Element>,
}

impl PageCrops<'_> {
    /// The disclosure every page-wide detail carries.
    fn note(&self) -> String {
        let mut note = unrendered_note(self.unrendered);
        if !self.css_backgrounds.is_empty() {
            note.push_str(&format!(
                " ({} CSS-background image(s) not examined: background geometry is not on the \
                 wire)",
                self.css_backgrounds.len()
            ));
        }
        note
    }
}

/// Every media element on the page, sorted into examined, unrendered and
/// CSS-background ([`PageCrops`]). An UNREADABLE image ([`img::unreadable`]:
/// reported occluded or off-screen, `visible: false` with no reason, wholly
/// inside its occluder), or any other image that paints but could not be
/// read, is a gap rather than an exclusion: the result is Unknown.
fn all_crops<'a>(
    a: &Assertion,
    frame: &Frame,
    snap: &'a ElementSnapshot,
) -> Result<PageCrops<'a>, AssertionResult> {
    let media: Vec<(&Element, &ElementMedia)> = snap
        .elements
        .iter()
        .filter_map(|e| e.media.as_ref().map(|m| (e, m)))
        .collect();
    if media.is_empty() {
        return Err(AssertionResult::unknown(
            a.clone(),
            UnknownCode::InputMissing,
            "no element carries `media`, so no image was examined",
        ));
    }
    let mut page = PageCrops {
        crops: Vec::new(),
        unrendered: 0,
        css_backgrounds: Vec::new(),
    };
    let mut missing = Vec::new();
    for (el, m) in media {
        if img::unrendered(snap, el).is_some() {
            page.unrendered += 1;
            continue;
        }
        if m.kind == MediaKind::CssBackground {
            page.css_backgrounds.push(el);
            continue;
        }
        match img::locate(frame, snap, el) {
            Ok(c) => page.crops.push((el, m, c)),
            Err((_, why)) => missing.push(why),
        }
    }
    if !missing.is_empty() {
        return Err(AssertionResult::unknown(
            a.clone(),
            UnknownCode::InputMissing,
            format!(
                "{} image(s) could not be read from the frame, so the page was not fully \
                 examined: {}",
                missing.len(),
                missing.join("; ")
            ),
        ));
    }
    Ok(page)
}

/// The Unknown for a page on which no image was examined at all.
fn nothing_examined(a: Assertion, page: &PageCrops<'_>) -> AssertionResult {
    AssertionResult::unknown(
        a,
        UnknownCode::InputMissing,
        format!("no image on the frame could be examined{}", page.note()),
    )
}

/// The detail suffix for images left out as unrendered.
fn unrendered_note(n: usize) -> String {
    if n > 0 {
        format!(
            " ({n} unrendered image(s) left out: a zero-area box, or reported hidden / no-layout)"
        )
    } else {
        String::new()
    }
}

/// Confidence: the lower crop confidence of the offending pair on a failure;
/// the lowest over every image read on a pass (the pass rests on all of them).
///
/// Fewer than two comparable images PASSES: [`all_crops`] has already
/// answered Unknown for any image it could not read, so every image on the
/// page was examined and there is no pair to be a duplicate. A STACKED pair
/// ([`img::pair_is_one_stack`]) is not compared — the frame shows at most one
/// of the two. When two or more images were comparable but EVERY pair was a
/// stack, no comparison happened at all, and that is Unknown, never a pass.
fn no_near_duplicates(
    a: Assertion,
    max_hamming: Option<u32>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let Some(max) = max_hamming else {
        return undeclared(a, "max_hamming");
    };
    let snap = tri!(require_snapshot(ctx, &a));
    let frame = tri!(require_frame(&a, ctx));
    let page = tri!(all_crops(&a, frame, snap));
    if page.crops.is_empty() {
        return nothing_examined(a, &page);
    }
    let crops = &page.crops;
    let read_conf = crops.iter().map(|c| c.2.confidence).fold(1.0, f64::min);
    // Under slot size an image is an icon: identical check marks or rating
    // stars are not duplicated content, so they are left out (and said so).
    let slot: Vec<&ImageCrop<'_>> = crops
        .iter()
        .filter(|(_, _, c)| img::is_slot_sized(c, frame.source.scale_factor))
        .collect();
    let small = crops.len() - slot.len();
    let hashed: Vec<(&Element, (u64, f64))> = slot
        .iter()
        .filter(|(_, _, c)| !img::is_low_entropy_fill(&c.luma))
        .filter_map(|(e, _, c)| Some((*e, (pixels::phash(&c.luma)?, c.confidence))))
        .collect();
    let excluded = slot.len() - hashed.len();
    // One image read through a wrapper and its own child is ONE image: the
    // nested element leaves the set before anything is counted.
    let (hashed, nested) = img::drop_nested(&img::Ancestry::of(snap), hashed);
    let hashed: Vec<(&Element, u64, f64)> =
        hashed.into_iter().map(|(e, (h, c))| (e, h, c)).collect();
    let mut note = if excluded > 0 {
        format!(" ({excluded} flat fill(s) excluded; no_placeholder_images owns them)")
    } else {
        String::new()
    };
    if nested > 0 {
        note.push_str(&format!(
            " ({nested} nested wrapper(s) not compared: their child images are)"
        ));
    }
    if small > 0 {
        let floor = img::PLACEHOLDER_FILL_MIN_SIDE_CSS;
        note.push_str(&format!(
            " ({small} image(s) under {floor} CSS px left out: icons, not slot images)"
        ));
    }
    if hashed.len() < 2 {
        note.push_str(&page.note());
        let among = if page.css_backgrounds.is_empty() {
            "so no pair is possible"
        } else {
            "so there is no pair among the examined images"
        };
        return AssertionResult::pass_with(
            a,
            format!(
                "fewer than two comparable images ({}), {among}{note}",
                hashed.len()
            ),
        )
        .with_confidence(read_conf);
    }
    let mut worst: Option<(&str, &str, u32, f64)> = None;
    let (mut stacked, mut compared) = (0usize, 0usize);
    for i in 0..hashed.len() {
        for j in i + 1..hashed.len() {
            let ((ea, ha, ca), (eb, hb, cb)) = (hashed[i], hashed[j]);
            if img::pair_is_one_stack(ea, eb) {
                stacked += 1;
                continue;
            }
            compared += 1;
            let d = pixels::hamming(ha, hb);
            if d <= max && worst.is_none_or(|w| d < w.2) {
                worst = Some((ea.id.as_str(), eb.id.as_str(), d, ca.min(cb)));
            }
        }
    }
    if stacked > 0 {
        note.push_str(&format!(
            " ({stacked} stacked pair(s) not compared: the frame shows at most one of each)"
        ));
    }
    note.push_str(&page.note());
    if compared == 0 {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "{} images, but every pair is a stack the frame shows only one of, so no pair \
                 was compared{note}",
                hashed.len()
            ),
        );
    }
    match worst {
        Some((x, y, d, conf)) => AssertionResult::fail(
            a,
            format!("images '{x}' and '{y}' are near duplicates: pHash distance {d} ≤ {max}{note}"),
        )
        .with_confidence(conf),
        None => AssertionResult::pass_with(
            a,
            format!("{compared} pair(s) compared, none within pHash {max}{note}"),
        )
        .with_confidence(hashed.iter().map(|h| h.2).fold(1.0, f64::min)),
    }
}

/// The detail of a placeholder failure.
fn placeholder_why(p: &img::PlaceholderValues) -> String {
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
    why.join("; ")
}

/// The failure for a row whose slot marks it a placeholder: a pixel-free
/// deduction, so it needs no frame and no geometry (a CSS background is
/// covered too) and states no confidence.
fn marker_fail(a: Assertion, el: &Element) -> AssertionResult {
    let css = el.media.as_ref().map(|m| m.kind) == Some(MediaKind::CssBackground);
    AssertionResult::fail(
        a,
        format!(
            "image '{}'{} renders a placeholder: the slot marks it as its placeholder",
            el.id,
            if css { " (a CSS background)" } else { "" }
        ),
    )
}

/// Confidence: on a failure, the offending image's crop confidence when a
/// PIXEL signal tripped, `None` when only the slot's marker did (a
/// deduction); on a pass, the lowest crop confidence of the images read.
///
/// Slot MARKERS are read first, for every rendered media row (CSS backgrounds
/// included): a marked one fails before any pixel is read, so an unreadable
/// image elsewhere cannot turn that failure into Unknown. CSS backgrounds are
/// otherwise not examined for pixels (see [`PageCrops`]), so the
/// single-element form over an unmarked one is Unknown.
fn no_placeholders(a: Assertion, only: Option<&str>, ctx: &EvalContext<'_>) -> AssertionResult {
    let snap = tri!(require_snapshot(ctx, &a));
    // The slot's placeholder MARKER is a pixel-free deduction, so it is read
    // FIRST, for every rendered media row of any kind: a marked placeholder is
    // a failure whether or not some other image on the page can be read, and
    // an Unknown from an unreadable image must not mask it.
    let marked = snap.elements.iter().find(|e| {
        e.media.as_ref().and_then(|m| m.placeholder) == Some(true)
            && img::unrendered(snap, e).is_none()
            && only.is_none_or(|id| e.id == id)
    });
    if let Some(el) = marked {
        return marker_fail(a, el);
    }
    let frame = tri!(require_frame(&a, ctx));
    let page = match only {
        Some(id) => {
            let (el, media) = tri!(media_element(&a, snap, id));
            let crop = tri!(crop_of(&a, frame, snap, el));
            PageCrops {
                crops: vec![(el, media, crop)],
                unrendered: 0,
                css_backgrounds: Vec::new(),
            }
        }
        None => tri!(all_crops(&a, frame, snap)),
    };
    for (el, media, crop) in &page.crops {
        let p = img::placeholder_values(media, crop, frame.source.scale_factor);
        if p.placeholder {
            let r = AssertionResult::fail(
                a,
                format!(
                    "image '{}' renders a placeholder: {}",
                    el.id,
                    placeholder_why(&p)
                ),
            );
            return if p.low_entropy_fill || p.known_pattern.is_some() {
                r.with_confidence(crop.confidence)
            } else {
                r
            };
        }
    }
    if page.crops.is_empty() {
        return nothing_examined(a, &page);
    }
    let conf = page
        .crops
        .iter()
        .map(|c| c.2.confidence)
        .fold(1.0, f64::min);
    AssertionResult::pass_with(
        a,
        format!(
            "{} image(s) checked, none a placeholder{}",
            page.crops.len(),
            page.note()
        ),
    )
    .with_confidence(conf)
}

/// No confidence: a deduction from the snapshot's media facts.
fn focal_visible(
    a: Assertion,
    id: &str,
    focal: NaturalRegion,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let valid = [focal.x, focal.y, focal.width, focal.height]
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        && focal.width > 0.0
        && focal.height > 0.0
        && focal.x + focal.width <= 1.0 + 1e-9
        && focal.y + focal.height <= 1.0 + 1e-9;
    if !valid {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            "the focal region is not a positive-area rectangle inside the unit square, so it \
             was not compared",
        );
    }
    let snap = tri!(require_snapshot(ctx, &a));
    let (_, media) = tri!(media_element(&a, snap, id));
    let visible = match img::visible_region(media) {
        Ok(v) => v,
        Err((code, detail)) => return AssertionResult::unknown(a, code, detail),
    };
    let inside = focal.fraction_inside(&visible);
    if inside < 1.0 - 1e-6 {
        AssertionResult::fail(
            a,
            format!(
                "only {:.1}% of '{id}''s focal region survives the crop: visible x {:.3}..{:.3}, \
                 y {:.3}..{:.3} of the natural image",
                inside * 100.0,
                visible.x,
                visible.x + visible.width,
                visible.y,
                visible.y + visible.height
            ),
        )
    } else {
        AssertionResult::pass_with(a, format!("'{id}''s focal region is wholly visible"))
    }
}

/// Confidence: the crop's, times the share of the surround ring inside the
/// frame.
fn transparent_legible(
    a: Assertion,
    id: &str,
    min_ratio: Option<f64>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let min = tri!(threshold(&a, min_ratio, "min_ratio"));
    let (frame, _, _, _, crop) = tri!(located(&a, id, ctx));
    let Some((ratio, ring)) = img::edge_contrast(frame, &crop) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!("no pixel around '{id}' is inside the frame, so it has no surround to contrast with"),
        );
    };
    let conf = crop.confidence * ring;
    if ratio < min {
        AssertionResult::fail(
            a,
            format!(
                "'{id}''s edge and stroke pixels reach {ratio:.2}:1 against their surround, below \
                 {min:.2}:1 — it does not read on this frame's background"
            ),
        )
        .with_confidence(conf)
    } else {
        AssertionResult::pass_with(
            a,
            format!("'{id}''s strokes reach {ratio:.2}:1 against their surround"),
        )
        .with_confidence(conf)
    }
}

/// Confidence: the CTA's crop confidence × the saliency map's peakedness ×
/// the winning margin `|s_cta − s_rival| / max(s_cta, s_rival)` — a near tie
/// is a coin toss however clean the inputs, and says so.
fn salience_order(a: Assertion, cta_id: &str, ctx: &EvalContext<'_>) -> AssertionResult {
    let snap = tri!(require_snapshot(ctx, &a));
    let frame = tri!(require_frame(&a, ctx));
    let Some(cta) = snap.get(cta_id) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            missing_operand(cta_id, "no salience was compared"),
        );
    };
    let cta_crop = tri!(crop_of(&a, frame, snap, cta));
    let Some((map, peaked)) = img::saliency_map(frame) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            "the frame is flat, so its saliency map singles nothing out",
        );
    };
    let mean = |px: (u32, u32, u32, u32)| pixels::grid_mean(&map, frame.width, frame.height, px);
    let Some(s_cta) = mean(cta_crop.px) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!("'{cta_id}' has no pixels"),
        );
    };
    let cta_box = cta.bbox.expect("located, so it has a bbox");
    let mut rival: Option<(String, f64)> = None;
    let mut compared = 0usize;
    for el in &snap.elements {
        if el.id == cta.id || !(el.interactable || el.media.is_some()) {
            continue;
        }
        let Some(b) = el.bbox else { continue };
        if region_contains(b, cta_box)
            || region_contains(cta_box, b)
            || img::unrendered(snap, el).is_some()
            || img::unreadable(snap, el).is_some()
        {
            continue;
        }
        // Only the rival's POSITION is needed: no crop is cut for it.
        let Some(px) = frame
            .css_region_to_pixels(b)
            .and_then(|r| r.clamp_to_frame(frame.width, frame.height))
        else {
            continue;
        };
        let Some(s) = mean(px) else { continue };
        compared += 1;
        if rival.as_ref().is_none_or(|r| s > r.1) {
            rival = Some((el.id.clone(), s));
        }
    }
    let Some((rival_id, s_rival)) = rival else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "no other interactive or media element is on the frame, so '{cta_id}' has \
                 nothing to be more salient than"
            ),
        );
    };
    let top = s_cta.max(s_rival);
    let margin = if top > 0.0 {
        ((s_cta - s_rival).abs() / top).min(1.0)
    } else {
        0.0
    };
    let conf = cta_crop.confidence * peaked * margin;
    if s_cta > s_rival {
        AssertionResult::pass_with(
            a,
            format!(
                "'{cta_id}' is the most salient of {} element(s): {s_cta:.3} vs '{rival_id}' {s_rival:.3}",
                compared + 1
            ),
        )
        .with_confidence(conf)
    } else {
        AssertionResult::fail(
            a,
            format!(
                "'{rival_id}' ({s_rival:.3}) out-draws the call to action '{cta_id}' ({s_cta:.3})"
            ),
        )
        .with_confidence(conf)
    }
}

/// Confidence: the crop's.
fn sharpness_at_least(
    a: Assertion,
    id: &str,
    min: Option<f64>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let min = tri!(threshold(&a, min, "min_sharpness"));
    let (_, _, _, _, crop) = tri!(located(&a, id, ctx));
    let Some(v) = pixels::laplacian_variance(&crop.luma) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!("'{id}''s crop is under 3×3 pixels, so sharpness was not measured"),
        );
    };
    let conf = crop.confidence;
    if v < min {
        AssertionResult::fail(
            a,
            format!("'{id}' reads soft: Laplacian variance {v:.1} < {min}"),
        )
        .with_confidence(conf)
    } else {
        AssertionResult::pass_with(a, format!("'{id}' Laplacian variance {v:.1}"))
            .with_confidence(conf)
    }
}

/// Confidence: the crop's.
fn blockiness_within(
    a: Assertion,
    id: &str,
    max: Option<f64>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let max = tri!(threshold(&a, max, "max_blockiness"));
    let (_, _, _, _, crop) = tri!(located(&a, id, ctx));
    let Some(grid) = crop.grid else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "where '{id}''s natural image is drawn (natural_px, object_fit, object_position) \
                 is unknown, so its 8-pixel coding grid cannot be placed"
            ),
        );
    };
    let Some(v) = pixels::blockiness(&crop.luma, grid) else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "'{id}''s coding grid is drawn under 4 device px apart, the crop holds fewer than \
                 two grid lines, or the image has no detail off the grid, so blockiness was not \
                 measured"
            ),
        );
    };
    let conf = crop.confidence;
    if v > max {
        AssertionResult::fail(
            a,
            format!("'{id}' shows compression blocks: blockiness {v:.2} > {max}"),
        )
        .with_confidence(conf)
    } else {
        AssertionResult::pass_with(a, format!("'{id}' blockiness {v:.2}")).with_confidence(conf)
    }
}

/// Confidence: the deciding reading's on a failure (the worst text over an
/// image); the lowest reading's on a pass, which rests on all of them. Each
/// reading's confidence is [`TextOverImageContrast::confidence`].
///
/// Checked in this order, each an Unknown: `min_ratio` declared and finite;
/// the percentile in 0..=100; the text-hidden frame present
/// (`needs_multi_frame_input`); the frame present; the two paired (one size,
/// one scale factor); a snapshot. Text over an unrendered or unreadable
/// image, or reported not visible itself, is skipped and disclosed; text
/// whose box touches an image while its glyphs do not is not text over it.
fn text_over_image(
    a: Assertion,
    min_ratio: Option<f64>,
    percentile: Option<f64>,
    ctx: &EvalContext<'_>,
) -> AssertionResult {
    let min = tri!(threshold(&a, min_ratio, "min_ratio"));
    let pct = match tc::valid_percentile(percentile.unwrap_or(tc::DEFAULT_PERCENTILE)) {
        Ok(p) => p,
        Err((code, detail)) => return AssertionResult::unknown(a, code, detail),
    };
    let Some(hidden) = ctx.text_hidden_frame else {
        return AssertionResult::unknown(
            a,
            UnknownCode::NeedsMultiFrameInput,
            "no text-hidden frame supplied: the glyphs are found only where the frame and a \
             capture of the same page with its text hidden differ, so no contrast was measured",
        );
    };
    let frame = tri!(require_frame(&a, ctx));
    if let Err((code, detail)) = tc::pair_frames(frame, hidden) {
        return AssertionResult::unknown(a, code, detail);
    }
    let snap = tri!(require_snapshot(ctx, &a));
    let page = tc::page_pairs(snap);
    let mut readings: Vec<TextOverImageContrast> = Vec::new();
    let mut not_over = 0usize;
    let mut unmeasured = Vec::new();
    for p in &page.pairs {
        match tc::measure(frame, hidden, p, pct) {
            Ok(Some(r)) => readings.push(r),
            Ok(None) => not_over += 1,
            Err((_, why)) => unmeasured.push(why),
        }
    }
    let mut note = page.note();
    if not_over > 0 {
        note.push_str(&format!(
            " ({not_over} text box(es) touch an image while their glyphs do not reach it)"
        ));
    }
    let worst = readings
        .iter()
        .min_by(|x, y| x.percentile_ratio.total_cmp(&y.percentile_ratio));
    let describe = |r: &TextOverImageContrast| {
        format!(
            "p{} {:.2}:1, minimum {:.2}:1 over {} {} pixel(s) against its {} glyph colour \
             rgb({},{},{})",
            r.percentile,
            r.percentile_ratio,
            r.min_ratio,
            r.sampled_px,
            match r.method {
                tc::SampleMethod::GlyphMask => "glyph",
                tc::SampleMethod::BboxFallback => "box (no usable glyph mask: fallback)",
            },
            match r.glyph_colour_source {
                tc::GlyphColourSource::Declared => "declared",
                tc::GlyphColourSource::Measured => "measured",
            },
            r.glyph_colour.r,
            r.glyph_colour.g,
            r.glyph_colour.b
        )
    };
    if let Some(w) = worst.filter(|w| w.percentile_ratio < min) {
        let also = if unmeasured.is_empty() {
            String::new()
        } else {
            format!(
                " ({} other text element(s) could not be measured)",
                unmeasured.len()
            )
        };
        return AssertionResult::fail(
            a,
            format!(
                "text '{}' over image '{}' reads {}, below {min:.2}:1{also}{note}",
                w.element,
                w.image,
                describe(w)
            ),
        )
        .with_confidence(w.confidence);
    }
    if !unmeasured.is_empty() {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "{} text element(s) over an image could not be measured, so the page was not \
                 fully examined: {}{note}",
                unmeasured.len(),
                unmeasured.join("; ")
            ),
        );
    }
    let Some(w) = worst else {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!("no text lies over a readable image, so no contrast was measured{note}"),
        );
    };
    let conf = readings.iter().map(|r| r.confidence).fold(1.0, f64::min);
    AssertionResult::pass_with(
        a,
        format!(
            "{} text-over-image reading(s) reach {min:.2}:1; the lowest, text '{}' over '{}', \
             reads {}{note}",
            readings.len(),
            w.element,
            w.image,
            describe(w)
        ),
    )
    .with_confidence(conf)
}
