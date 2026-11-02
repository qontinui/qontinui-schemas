// SPDX-License-Identifier: MIT
//! The L0 image assertions (plan 2026-10-13-image-generation-and-visual-
//! judgment, Phase 8): deductions from the facts on [`Element::media`], with
//! no frame. They replace the UI Bridge SDK's `buildAccessibilityAudit` and
//! `buildPerformanceAudit`.
//!
//! Every variant takes an optional `element`. Without it the variant runs over
//! every element carrying `media`, and its detail names each failing id. The
//! DSL returns one result per assertion, so a caller wanting a per-image
//! outcome sends one assertion per id.
//!
//! How one image's verdict folds into the page's:
//!
//! - any failure fails the assertion, even when another image was unmeasured;
//! - otherwise any unmeasured image makes it Unknown, never a pass;
//! - no element carrying `media`, or only unrendered ones, is Unknown: that
//!   cannot be told apart from a producer that was not asked for media. Only
//!   [`Assertion::ImageWeightWithin`] and [`Assertion::ImageLoaded`] judge some
//!   unrendered images (below), so they can answer over those;
//! - images the check does not apply to are left out, and the pass says how
//!   many. Named explicitly, such an image is Unknown instead, because the
//!   caller asked about something this check cannot judge.
//!
//! Images that paint nothing ([`img::unrendered`]) are left out too, with two
//! exceptions. [`Assertion::ImageWeightWithin`] weighs them, because a hidden
//! image's bytes are still paid for. [`Assertion::ImageLoaded`] fails a broken
//! one, because a broken `<img alt="">` collapses to a zero-area box, and answers
//! Unknown for an `img` still loading, because one with no reserved size is
//! zero-area until it has.
//!
//! `rendered_px` is the drawn border box, transforms included, and no fact here
//! gives the content box or the transform. So [`Assertion::ImageNotDistorted`]
//! misses a non-uniform `transform` on a `contain`, `cover`, `none` or
//! `scale-down` image, which it holds without reading a size. Under `fill` it can
//! read a transform, padding, a border or a rotation as distortion. A caller
//! tolerating those sets `max_aspect_delta` to suit.
//!
//! An `img` whose `format` names no raster may be a vector: the producer leaves
//! it out when nothing gives one, and otherwise takes any URL suffix (`.php`). It
//! can hold, because a vector cannot fail these checks either, but it cannot fail:
//! where a raster would, the answer is Unknown.
//!
//! These set no [`AssertionResult::confidence`]. Nothing here is estimated, so
//! the verdict is as exact as the producer's facts.

use std::collections::HashMap;

use super::image::{media_element, threshold};
use super::{require_snapshot, Assertion, AssertionResult, EvalContext};
use crate::analyzers::image as img;
use crate::element_media::{
    ElementMedia, MediaFetchPriority, MediaKind, MediaLoadState, MediaLoading, ObjectFit,
};
use crate::element_snapshot::Element;
use crate::observation::UnknownCode;

/// One image's verdict. Each carrying string is a sentence naming the element.
enum Verdict {
    Holds,
    Breaks(String),
    Unmeasured(String),
    NotApplicable(String),
    /// Paints nothing, so left out as an unrendered image is.
    Unrendered(String),
}

/// Whether a check still judges an image that paints nothing.
#[derive(Clone, Copy, PartialEq)]
enum Unrendered {
    LeftOut,
    Judged,
}

/// Dispatch one of the L0 variants. Called by [`super::evaluate`] for those
/// variants only.
pub(super) fn evaluate(a: Assertion, ctx: &EvalContext<'_>) -> AssertionResult {
    match a.clone() {
        Assertion::ImageLoaded { element } => {
            let snapshot = ctx.snapshot;
            over_media(a, &element, ctx, Unrendered::Judged, |el, m| {
                loaded(el, m, snapshot.and_then(|s| img::unrendered(s, el)))
            })
        }
        Assertion::ImageResolutionSufficient {
            element,
            min_effective,
        } => {
            let min = match threshold(&a, min_effective, "min_effective") {
                Ok(t) => t,
                Err(r) => return r,
            };
            over_media(a, &element, ctx, Unrendered::LeftOut, |el, m| {
                resolution_sufficient(el, m, min)
            })
        }
        Assertion::ImageNotDistorted {
            element,
            max_aspect_delta,
        } => {
            let max = match threshold(&a, max_aspect_delta, "max_aspect_delta") {
                Ok(t) => t,
                Err(r) => return r,
            };
            over_media(a, &element, ctx, Unrendered::LeftOut, |el, m| {
                not_distorted(el, m, max)
            })
        }
        Assertion::ImageDimsReserved { element } => {
            over_media(a, &element, ctx, Unrendered::LeftOut, dims_reserved)
        }
        Assertion::LcpImagePrioritized { element } => {
            over_media(a, &element, ctx, Unrendered::LeftOut, lcp_prioritized)
        }
        Assertion::ImageHasTextAlternative { element } => {
            over_media(a, &element, ctx, Unrendered::LeftOut, text_alternative)
        }
        Assertion::ImageNotOversized {
            element,
            max_effective,
        } => {
            let max = match threshold(&a, max_effective, "max_effective") {
                Ok(t) => t,
                Err(r) => return r,
            };
            over_media(a, &element, ctx, Unrendered::LeftOut, |el, m| {
                not_oversized(el, m, max)
            })
        }
        Assertion::ImageWeightWithin { element, max_bytes } => {
            let Some(max) = max_bytes else {
                return super::image::undeclared(a, "max_bytes");
            };
            over_media(a, &element, ctx, Unrendered::Judged, |el, m| {
                weight_within(el, m, max)
            })
        }
        Assertion::OffscreenImageLazy { element } => {
            over_media(a, &element, ctx, Unrendered::LeftOut, offscreen_lazy)
        }
        Assertion::NoDuplicateImages { element } => {
            let holders = digest_holders(ctx);
            over_media(a, &element, ctx, Unrendered::LeftOut, |el, m| {
                not_duplicated(el, m, &holders)
            })
        }
        other => unreachable!("not an L0 image assertion: {other:?}"),
    }
}

/// Run `judge` over the named element, or over every element carrying `media`.
fn over_media(
    a: Assertion,
    element: &Option<String>,
    ctx: &EvalContext<'_>,
    unrendered: Unrendered,
    judge: impl Fn(&Element, &ElementMedia) -> Verdict,
) -> AssertionResult {
    let snap = match require_snapshot(ctx, &a) {
        Ok(s) => s,
        Err(r) => return r,
    };

    if let Some(id) = element {
        let (el, media) = match media_element(&a, snap, id) {
            Ok(found) => found,
            Err(r) => return r,
        };
        if unrendered == Unrendered::LeftOut {
            if let Some(why) = img::unrendered(snap, el) {
                return AssertionResult::unknown(a, UnknownCode::InputMissing, why);
            }
        }
        return match judge(el, media) {
            Verdict::Holds => AssertionResult::pass(a),
            Verdict::Breaks(why) => AssertionResult::fail(a, why),
            Verdict::Unmeasured(why) | Verdict::NotApplicable(why) | Verdict::Unrendered(why) => {
                AssertionResult::unknown(a, UnknownCode::InputMissing, why)
            }
        };
    }

    let mut held = 0usize;
    let mut left_out = 0usize;
    let mut not_applicable = 0usize;
    let mut breaks = Vec::new();
    let mut unmeasured = Vec::new();
    for el in &snap.elements {
        let Some(media) = el.media.as_ref() else {
            continue;
        };
        if unrendered == Unrendered::LeftOut && img::unrendered(snap, el).is_some() {
            left_out += 1;
            continue;
        }
        match judge(el, media) {
            Verdict::Holds => held += 1,
            Verdict::Breaks(why) => breaks.push(why),
            Verdict::Unmeasured(why) => unmeasured.push(why),
            Verdict::NotApplicable(_) => not_applicable += 1,
            Verdict::Unrendered(_) => left_out += 1,
        }
    }

    if !breaks.is_empty() {
        let mut detail = breaks.join("; ");
        if !unmeasured.is_empty() {
            detail.push_str(&format!(
                " ({} more could not be measured: {})",
                unmeasured.len(),
                unmeasured.join("; ")
            ));
        }
        return AssertionResult::fail(a, detail);
    }
    if !unmeasured.is_empty() {
        return AssertionResult::unknown(a, UnknownCode::InputMissing, unmeasured.join("; "));
    }
    if held == 0 && not_applicable == 0 {
        return AssertionResult::unknown(
            a,
            UnknownCode::InputMissing,
            format!(
                "no image could be judged: the snapshot has {left_out} unrendered image(s) and \
                 no other element carrying `media`, which cannot be told apart from a producer \
                 that was not asked for media"
            ),
        );
    }
    AssertionResult::pass_with(
        a,
        format!(
            "{held} image(s) hold; {not_applicable} not applicable; {left_out} unrendered left \
             out"
        ),
    )
}

/// `unrendered`: why the image paints nothing, if it does not. Only a broken
/// image is still judged then; nothing else about a load shows on the page.
fn loaded(el: &Element, m: &ElementMedia, unrendered: Option<String>) -> Verdict {
    let id = &el.id;
    if m.load_state == Some(MediaLoadState::Error) {
        return Verdict::Breaks(format!("element '{id}' failed to load"));
    }
    // An image only: a hidden `<video preload=none>` is pending and never loads.
    let img_tag = matches!(m.kind, MediaKind::Img | MediaKind::Picture);
    if img_tag && m.load_state == Some(MediaLoadState::Pending) {
        return Verdict::Unmeasured(format!("element '{id}' is still loading"));
    }
    if let Some(why) = unrendered {
        return Verdict::Unrendered(why);
    }
    match m.load_state {
        Some(MediaLoadState::Loaded) => Verdict::Holds,
        Some(MediaLoadState::Error) => unreachable!("returned above"),
        Some(MediaLoadState::Pending) => {
            Verdict::Unmeasured(format!("element '{id}' is still loading"))
        }
        Some(MediaLoadState::Lazy) if m.in_viewport == Some(false) => Verdict::NotApplicable(
            format!("element '{id}' is off-screen and lazy, so it has not been asked to load"),
        ),
        Some(MediaLoadState::Lazy) => Verdict::Unmeasured(format!(
            "element '{id}' is lazy and not known to be off-screen, so it should have started \
             loading and has not finished"
        )),
        None => Verdict::Unmeasured(format!("element '{id}' has no measured `load_state`")),
    }
}

/// Images with no resource pixel density: a vector scales, whether inline or in
/// an `<img src=x.svg>`; a CSS background exposes no intrinsic size; and a
/// `role=img` element need not be a raster.
fn has_no_density(m: &ElementMedia) -> bool {
    matches!(
        m.kind,
        MediaKind::Svg | MediaKind::CssBackground | MediaKind::RoleImg
    ) || m
        .format
        .as_deref()
        .is_some_and(|f| f.to_ascii_lowercase().starts_with("svg"))
}

/// Raster `format`s, as a MIME subtype or a URL suffix.
const RASTER_FORMATS: &[&str] = &[
    "apng",
    "avif",
    "bmp",
    "gif",
    "heic",
    "heif",
    "ico",
    "jfif",
    "jpe",
    "jpeg",
    "jpg",
    "jxl",
    "pjp",
    "pjpeg",
    "png",
    "tif",
    "tiff",
    "vnd.microsoft.icon",
    "webp",
    "x-icon",
    "x-png",
];

/// An `img` or `picture` whose `format` names no raster, so it may be an
/// `<img src=x.svg>`. A failure of a density or shape check needs this ruled out.
fn may_be_vector(m: &ElementMedia) -> bool {
    matches!(m.kind, MediaKind::Img | MediaKind::Picture)
        && !m
            .format
            .as_deref()
            .is_some_and(|f| RASTER_FORMATS.iter().any(|r| r.eq_ignore_ascii_case(f)))
}

/// Why a density or shape failure cannot be claimed of an image [`may_be_vector`].
fn unknown_format(id: &str) -> Verdict {
    Verdict::Unmeasured(format!(
        "element '{id}' reads as failing, but its `format` names no raster, so it may be a vector"
    ))
}

fn resolution_sufficient(el: &Element, m: &ElementMedia, min: f64) -> Verdict {
    let id = &el.id;
    if has_no_density(m) {
        return Verdict::NotApplicable(format!("element '{id}' has no resource pixel density"));
    }
    match m.effective_resolution {
        None => Verdict::Unmeasured(format!(
            "element '{id}' has no measured `effective_resolution`"
        )),
        Some(v) if v < min as f32 && may_be_vector(m) => unknown_format(id),
        Some(v) if v < min as f32 => Verdict::Breaks(format!(
            "element '{id}' carries {v} source pixels per device pixel, under {min}"
        )),
        Some(_) => Verdict::Holds,
    }
}

fn not_oversized(el: &Element, m: &ElementMedia, max: f64) -> Verdict {
    let id = &el.id;
    if has_no_density(m) {
        return Verdict::NotApplicable(format!("element '{id}' has no resource pixel density"));
    }
    match m.effective_resolution {
        None => Verdict::Unmeasured(format!(
            "element '{id}' has no measured `effective_resolution`"
        )),
        Some(v) if v > max as f32 && may_be_vector(m) => unknown_format(id),
        Some(v) if v > max as f32 => Verdict::Breaks(format!(
            "element '{id}' carries {v} source pixels per device pixel, over {max}"
        )),
        Some(_) => Verdict::Holds,
    }
}

fn not_distorted(el: &Element, m: &ElementMedia, max: f64) -> Verdict {
    let id = &el.id;
    if has_no_density(m) {
        return Verdict::NotApplicable(format!("element '{id}' has no intrinsic raster size"));
    }
    // These scale uniformly or not at all, so whatever the box's shape,
    // `object-fit` does not change the image's. A transform can (module docs).
    if matches!(
        m.object_fit,
        Some(ObjectFit::Contain | ObjectFit::Cover | ObjectFit::None | ObjectFit::ScaleDown)
    ) {
        return Verdict::Holds;
    }
    let (Some(natural), Some(rendered)) = (m.natural_px, m.rendered_px) else {
        return Verdict::Unmeasured(format!(
            "element '{id}' lacks a measured `natural_px` or `rendered_px`"
        ));
    };
    let positive = [natural.w, natural.h, rendered.w, rendered.h]
        .iter()
        .all(|v| *v > 0.0);
    if !positive {
        return Verdict::Unmeasured(format!(
            "element '{id}' has a zero size, so it has no aspect ratio"
        ));
    }
    let natural_aspect = f64::from(natural.w) / f64::from(natural.h);
    let rendered_aspect = f64::from(rendered.w) / f64::from(rendered.h);
    // Symmetric, so squashing and stretching by one factor read the same.
    let delta = (natural_aspect / rendered_aspect).max(rendered_aspect / natural_aspect) - 1.0;
    // At f32, the precision the sizes were measured at: 105x100 over 100x100 is
    // 0.05 as written and 0.050000000000000044 in f64.
    if delta as f32 <= max as f32 {
        return Verdict::Holds;
    }
    if m.object_fit == Some(ObjectFit::Fill) && may_be_vector(m) {
        return unknown_format(id);
    }
    match m.object_fit {
        Some(ObjectFit::Fill) => Verdict::Breaks(format!(
            "element '{id}' is stretched by `object-fit: fill`: its aspect differs by {delta:.3}, \
             over {max}"
        )),
        _ => Verdict::Unmeasured(format!(
            "element '{id}' has a box whose aspect differs by {delta:.3}, and no measured \
             `object_fit` to say whether the image is stretched to it"
        )),
    }
}

fn dims_reserved(el: &Element, m: &ElementMedia) -> Verdict {
    let id = &el.id;
    if !matches!(
        m.kind,
        MediaKind::Img | MediaKind::Picture | MediaKind::Video
    ) {
        return Verdict::NotApplicable(format!(
            "element '{id}' has no box that waits on a network load"
        ));
    }
    match m.dims_reserved {
        Some(true) => Verdict::Holds,
        Some(false) => Verdict::Breaks(format!(
            "element '{id}' reserves no space, so loading it can shift the layout"
        )),
        None => Verdict::Unmeasured(format!("element '{id}' has no measured `dims_reserved`")),
    }
}

/// An LCP image must be requested early: never lazy, and `fetchpriority=high`,
/// which is what `next/image`'s `priority` sets.
fn lcp_prioritized(el: &Element, m: &ElementMedia) -> Verdict {
    let id = &el.id;
    if !matches!(m.kind, MediaKind::Img | MediaKind::Picture) {
        return Verdict::NotApplicable(format!(
            "element '{id}' has no `loading` or `fetchpriority` attribute"
        ));
    }
    match m.is_lcp_candidate {
        None => {
            return Verdict::Unmeasured(format!(
                "element '{id}' has no measured `is_lcp_candidate`"
            ))
        }
        Some(false) => {
            return Verdict::NotApplicable(format!("element '{id}' is not the LCP element"))
        }
        Some(true) => {}
    }
    if m.loading == Some(MediaLoading::Lazy) {
        return Verdict::Breaks(format!("element '{id}' is the LCP element and is lazy"));
    }
    match m.fetch_priority {
        Some(MediaFetchPriority::Low | MediaFetchPriority::Auto) => Verdict::Breaks(format!(
            "element '{id}' is the LCP element without `fetchpriority=high`"
        )),
        Some(MediaFetchPriority::High) if m.loading.is_some() => Verdict::Holds,
        _ => Verdict::Unmeasured(format!(
            "element '{id}' is the LCP element, and its `loading` or `fetch_priority` was not \
             measured"
        )),
    }
}

/// The text that says nothing about the image. The list is
/// `GENERIC_ALT_PATTERNS` from the SDK's `buildAccessibilityAudit`, which this
/// assertion replaces.
fn is_generic_alt(text: &str) -> bool {
    const GENERIC: &[&str] = &[
        "image",
        "photo",
        "img",
        "picture",
        "icon",
        "logo",
        "banner",
        "untitled",
        "placeholder",
        "thumbnail",
        "screenshot",
    ];
    let lower = text.trim().to_lowercase();
    if GENERIC.contains(&lower.as_str()) {
        return true;
    }
    if lower.split_whitespace().eq(["screen", "shot"]) {
        return true;
    }
    !lower.is_empty() && lower.chars().all(|c| c.is_ascii_digit())
}

fn text_alternative(el: &Element, m: &ElementMedia) -> Verdict {
    let id = &el.id;
    if matches!(m.kind, MediaKind::Video | MediaKind::Canvas) {
        return Verdict::NotApplicable(format!(
            "element '{id}' is a video or canvas, which an image text alternative does not cover"
        ));
    }
    let Some(alt) = m.alt.as_ref() else {
        return Verdict::Unmeasured(format!("element '{id}' has no measured `alt`"));
    };
    if alt.decorative {
        return Verdict::Holds;
    }
    match alt.text.as_deref().map(str::trim) {
        None | Some("") => Verdict::Breaks(format!("element '{id}' has no text alternative")),
        Some(text) if is_generic_alt(text) => {
            Verdict::Breaks(format!("element '{id}' has the generic alt text '{text}'"))
        }
        Some(_) => Verdict::Holds,
    }
}

fn weight_within(el: &Element, m: &ElementMedia, max: u64) -> Verdict {
    let id = &el.id;
    if matches!(
        m.kind,
        MediaKind::Svg | MediaKind::Canvas | MediaKind::RoleImg
    ) {
        return Verdict::NotApplicable(format!("element '{id}' has no resource of its own"));
    }
    match m.transfer_bytes {
        Some(bytes) if bytes > max => {
            Verdict::Breaks(format!("element '{id}' weighs {bytes} bytes, over {max}"))
        }
        Some(_) => Verdict::Holds,
        // `lazy` alone is any unfinished lazy image, and one on screen is
        // mid-download. A zero-area box reads as off-screen wherever it sits.
        None if m.load_state == Some(MediaLoadState::Lazy)
            && m.in_viewport == Some(false)
            && el.bbox.is_some_and(|b| b.w > 0 && b.h > 0) =>
        {
            Verdict::NotApplicable(format!(
                "element '{id}' is off-screen and lazy and has not finished loading, so its \
                 weight is left out"
            ))
        }
        None => Verdict::Unmeasured(format!("element '{id}' has no measured `transfer_bytes`")),
    }
}

fn offscreen_lazy(el: &Element, m: &ElementMedia) -> Verdict {
    let id = &el.id;
    if !matches!(m.kind, MediaKind::Img | MediaKind::Picture) {
        return Verdict::NotApplicable(format!("element '{id}' has no `loading` attribute"));
    }
    if m.is_lcp_candidate == Some(true) {
        return Verdict::NotApplicable(format!(
            "element '{id}' is the LCP element, which `lcp_image_prioritized` judges"
        ));
    }
    match m.in_viewport {
        None => {
            return Verdict::Unmeasured(format!("element '{id}' has no measured `in_viewport`"))
        }
        Some(true) => return Verdict::NotApplicable(format!("element '{id}' is on-screen")),
        Some(false) => {}
    }
    match m.loading {
        Some(MediaLoading::Lazy) => Verdict::Holds,
        Some(MediaLoading::Eager) => {
            Verdict::Breaks(format!("element '{id}' is off-screen and not lazy"))
        }
        None => Verdict::Unmeasured(format!(
            "element '{id}' is off-screen with no measured `loading`"
        )),
    }
}

/// Kinds a repeated source does not make a duplicate of. SVG, canvas and
/// `role=img` have no source to digest, and one CSS background on many
/// elements is how sprites and textures work.
fn has_no_duplicable_source(kind: MediaKind) -> bool {
    matches!(
        kind,
        MediaKind::Svg | MediaKind::Canvas | MediaKind::RoleImg | MediaKind::CssBackground
    )
}

/// Every rendered element displaying each digest.
fn digest_holders<'a>(ctx: &EvalContext<'a>) -> HashMap<&'a str, Vec<&'a str>> {
    let mut holders: HashMap<&'a str, Vec<&'a str>> = HashMap::new();
    let Some(snap) = ctx.snapshot else {
        return holders;
    };
    for el in &snap.elements {
        let Some(media) = el.media.as_ref() else {
            continue;
        };
        if has_no_duplicable_source(media.kind) || img::unrendered(snap, el).is_some() {
            continue;
        }
        if let Some(digest) = media.src_digest.as_deref() {
            holders.entry(digest).or_default().push(el.id.as_str());
        }
    }
    holders
}

fn not_duplicated(el: &Element, m: &ElementMedia, holders: &HashMap<&str, Vec<&str>>) -> Verdict {
    let id = &el.id;
    if has_no_duplicable_source(m.kind) {
        return Verdict::NotApplicable(format!("element '{id}' has no source to compare"));
    }
    let Some(digest) = m.src_digest.as_deref() else {
        return Verdict::Unmeasured(format!("element '{id}' has no measured `src_digest`"));
    };
    let others: Vec<&str> = holders
        .get(digest)
        .into_iter()
        .flatten()
        .copied()
        .filter(|other| *other != id.as_str())
        .collect();
    if others.is_empty() {
        Verdict::Holds
    } else {
        Verdict::Breaks(format!(
            "element '{id}' displays the same source as {}",
            others.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::{evaluate as dispatch, AssertionOutcome};
    use super::*;
    use crate::element_media::{MediaAlt, MediaSize};
    use crate::element_snapshot::ElementSnapshot;
    use crate::frame::Region;

    fn image(id: &str, media: ElementMedia) -> Element {
        Element {
            id: id.into(),
            bbox: Some(Region {
                x: 0,
                y: 0,
                w: 100,
                h: 100,
            }),
            media: Some(media),
            ..Default::default()
        }
    }

    fn img_with() -> ElementMedia {
        ElementMedia::new(MediaKind::Img)
    }

    fn run(a: Assertion, elements: Vec<Element>) -> AssertionResult {
        let snap = ElementSnapshot {
            elements,
            ..Default::default()
        };
        dispatch(
            &a,
            &EvalContext {
                snapshot: Some(&snap),
                ..Default::default()
            },
        )
    }

    fn assert_outcome(r: &AssertionResult, want: AssertionOutcome) {
        assert_eq!(r.outcome, want, "{:?}", r.detail);
    }

    fn assert_detail_has(r: &AssertionResult, needle: &str) {
        let detail = r.detail.as_deref().unwrap_or_default();
        assert!(detail.contains(needle), "{needle:?} not in {detail:?}");
    }

    fn loaded_a() -> Assertion {
        Assertion::ImageLoaded { element: None }
    }

    // ---- the fold, on one variant ------------------------------------------

    #[test]
    fn page_wide_failure_names_each_failing_id_and_stands_over_an_unmeasured_image() {
        let broken = |id| {
            image(
                id,
                ElementMedia {
                    load_state: Some(MediaLoadState::Error),
                    ..img_with()
                },
            )
        };
        let r = run(
            loaded_a(),
            vec![broken("a"), broken("b"), image("c", img_with())],
        );
        assert_outcome(&r, AssertionOutcome::Failed);
        for id in ["'a'", "'b'", "'c'"] {
            assert_detail_has(&r, id);
        }
    }

    #[test]
    fn page_wide_unmeasured_image_is_unknown_not_a_pass() {
        let ok = image(
            "ok",
            ElementMedia {
                load_state: Some(MediaLoadState::Loaded),
                ..img_with()
            },
        );
        let r = run(loaded_a(), vec![ok, image("c", img_with())]);
        assert_outcome(&r, AssertionOutcome::Unknown);
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
        assert_detail_has(&r, "'c'");
    }

    #[test]
    fn no_element_carrying_media_is_unknown() {
        let plain = Element {
            id: "text".into(),
            ..Default::default()
        };
        let r = run(loaded_a(), vec![plain]);
        assert_outcome(&r, AssertionOutcome::Unknown);
        assert_eq!(r.code, Some(UnknownCode::InputMissing));
    }

    #[test]
    fn no_snapshot_is_unknown() {
        let r = dispatch(&loaded_a(), &EvalContext::default());
        assert_outcome(&r, AssertionOutcome::Unknown);
    }

    fn zero_area(mut el: Element) -> Element {
        el.bbox = Some(Region {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        });
        el
    }

    #[test]
    fn an_unrendered_image_is_left_out_but_the_only_image_stays_unknown() {
        let a = Assertion::ImageDimsReserved { element: None };
        let breaks = zero_area(image(
            "h",
            ElementMedia {
                dims_reserved: Some(false),
                ..img_with()
            },
        ));
        let holds = image(
            "ok",
            ElementMedia {
                dims_reserved: Some(true),
                ..img_with()
            },
        );
        assert_outcome(
            &run(a.clone(), vec![breaks.clone()]),
            AssertionOutcome::Unknown,
        );
        let r = run(a, vec![breaks, holds]);
        assert_outcome(&r, AssertionOutcome::Passed);
        assert_detail_has(&r, "1 unrendered");
    }

    #[test]
    fn image_loaded_fails_a_broken_image_that_collapsed_to_nothing() {
        let collapsed = zero_area(image(
            "dead",
            ElementMedia {
                load_state: Some(MediaLoadState::Error),
                ..img_with()
            },
        ));
        let ok = image(
            "ok",
            ElementMedia {
                load_state: Some(MediaLoadState::Loaded),
                ..img_with()
            },
        );
        let r = run(loaded_a(), vec![collapsed, ok]);
        assert_outcome(&r, AssertionOutcome::Failed);
        assert_detail_has(&r, "'dead'");
    }

    #[test]
    fn image_loaded_leaves_out_a_hidden_video_that_never_loads() {
        let video = zero_area(image(
            "v",
            ElementMedia {
                kind: MediaKind::Video,
                load_state: Some(MediaLoadState::Pending),
                ..img_with()
            },
        ));
        let ok = image(
            "ok",
            ElementMedia {
                load_state: Some(MediaLoadState::Loaded),
                ..img_with()
            },
        );
        assert_outcome(&run(loaded_a(), vec![video, ok]), AssertionOutcome::Passed);
    }

    #[test]
    fn image_loaded_is_unknown_for_a_collapsed_image_still_loading() {
        let pending = zero_area(image(
            "h",
            ElementMedia {
                load_state: Some(MediaLoadState::Pending),
                ..img_with()
            },
        ));
        let ok = image(
            "ok",
            ElementMedia {
                load_state: Some(MediaLoadState::Loaded),
                ..img_with()
            },
        );
        assert_outcome(
            &run(loaded_a(), vec![pending.clone()]),
            AssertionOutcome::Unknown,
        );
        // With no reserved size it is zero-area until it loads, so another image
        // holding says nothing about it.
        assert_outcome(
            &run(loaded_a(), vec![pending, ok]),
            AssertionOutcome::Unknown,
        );
    }

    #[test]
    fn named_element_missing_from_the_snapshot_or_without_media_is_unknown() {
        let a = Assertion::ImageLoaded {
            element: Some("hero".into()),
        };
        let r = run(a.clone(), vec![]);
        assert_outcome(&r, AssertionOutcome::Unknown);
        assert_detail_has(&r, "not in the snapshot");

        let plain = Element {
            id: "hero".into(),
            ..Default::default()
        };
        let r = run(a, vec![plain]);
        assert_outcome(&r, AssertionOutcome::Unknown);
        assert_detail_has(&r, "no `media`");
    }

    #[test]
    fn named_element_the_check_does_not_apply_to_is_unknown_and_page_wide_passes() {
        let svg = || image("s", ElementMedia::new(MediaKind::Svg));
        let named = Assertion::ImageResolutionSufficient {
            element: Some("s".into()),
            min_effective: Some(1.0),
        };
        assert_outcome(&run(named, vec![svg()]), AssertionOutcome::Unknown);
        let wide = Assertion::ImageResolutionSufficient {
            element: None,
            min_effective: Some(1.0),
        };
        let r = run(wide, vec![svg()]);
        assert_outcome(&r, AssertionOutcome::Passed);
        assert_detail_has(&r, "1 not applicable");
    }

    #[test]
    fn named_element_only_judges_that_element() {
        let broken = image(
            "broken",
            ElementMedia {
                load_state: Some(MediaLoadState::Error),
                ..img_with()
            },
        );
        let ok = image(
            "ok",
            ElementMedia {
                load_state: Some(MediaLoadState::Loaded),
                ..img_with()
            },
        );
        let a = Assertion::ImageLoaded {
            element: Some("ok".into()),
        };
        assert_outcome(&run(a, vec![broken, ok]), AssertionOutcome::Passed);
    }

    // ---- thresholds --------------------------------------------------------

    #[test]
    fn an_undeclared_threshold_is_unknown_never_a_pass() {
        let holds = image(
            "x",
            ElementMedia {
                effective_resolution: Some(1.0),
                transfer_bytes: Some(1),
                object_fit: Some(ObjectFit::Cover),
                ..img_with()
            },
        );
        for a in [
            Assertion::ImageResolutionSufficient {
                element: None,
                min_effective: None,
            },
            Assertion::ImageNotDistorted {
                element: None,
                max_aspect_delta: None,
            },
            Assertion::ImageNotOversized {
                element: None,
                max_effective: None,
            },
            Assertion::ImageWeightWithin {
                element: None,
                max_bytes: None,
            },
        ] {
            let r = run(a, vec![holds.clone()]);
            assert_outcome(&r, AssertionOutcome::Unknown);
            assert_eq!(r.code, Some(UnknownCode::InputMissing));
            assert_detail_has(&r, "undeclared in domain_spec/visual-system");
        }
    }

    #[test]
    fn a_non_finite_threshold_is_unknown() {
        let a = Assertion::ImageNotOversized {
            element: None,
            max_effective: Some(f64::NAN),
        };
        let r = run(
            a,
            vec![image(
                "x",
                ElementMedia {
                    effective_resolution: Some(9.0),
                    ..img_with()
                },
            )],
        );
        assert_outcome(&r, AssertionOutcome::Unknown);
    }

    // ---- image_loaded ------------------------------------------------------

    #[test]
    fn image_loaded_fixtures() {
        let state = |s| {
            image(
                "x",
                ElementMedia {
                    load_state: s,
                    in_viewport: Some(true),
                    ..img_with()
                },
            )
        };
        let cases = [
            (Some(MediaLoadState::Loaded), AssertionOutcome::Passed),
            (Some(MediaLoadState::Error), AssertionOutcome::Failed),
            (Some(MediaLoadState::Pending), AssertionOutcome::Unknown),
            (Some(MediaLoadState::Lazy), AssertionOutcome::Unknown),
            (None, AssertionOutcome::Unknown),
        ];
        for (s, want) in cases {
            assert_outcome(&run(loaded_a(), vec![state(s)]), want);
        }
    }

    #[test]
    fn image_loaded_leaves_out_an_offscreen_lazy_image_page_wide_only() {
        let lazy = image(
            "x",
            ElementMedia {
                load_state: Some(MediaLoadState::Lazy),
                in_viewport: Some(false),
                ..img_with()
            },
        );
        assert_outcome(
            &run(loaded_a(), vec![lazy.clone()]),
            AssertionOutcome::Passed,
        );
        let named = Assertion::ImageLoaded {
            element: Some("x".into()),
        };
        assert_outcome(&run(named, vec![lazy]), AssertionOutcome::Unknown);
    }

    // ---- resolution and oversize -------------------------------------------

    fn density(v: Option<f32>) -> Element {
        image(
            "x",
            ElementMedia {
                effective_resolution: v,
                format: Some("png".into()),
                ..img_with()
            },
        )
    }

    #[test]
    fn image_resolution_sufficient_fixtures() {
        let a = Assertion::ImageResolutionSufficient {
            element: None,
            min_effective: Some(1.0),
        };
        assert_outcome(
            &run(a.clone(), vec![density(Some(0.5))]),
            AssertionOutcome::Failed,
        );
        assert_outcome(
            &run(a.clone(), vec![density(Some(1.0))]),
            AssertionOutcome::Passed,
        );
        assert_outcome(&run(a, vec![density(None)]), AssertionOutcome::Unknown);
    }

    #[test]
    fn image_not_oversized_fixtures() {
        let a = Assertion::ImageNotOversized {
            element: None,
            max_effective: Some(2.0),
        };
        assert_outcome(
            &run(a.clone(), vec![density(Some(2.5))]),
            AssertionOutcome::Failed,
        );
        assert_outcome(
            &run(a.clone(), vec![density(Some(2.0))]),
            AssertionOutcome::Passed,
        );
        assert_outcome(&run(a, vec![density(None)]), AssertionOutcome::Unknown);
    }

    #[test]
    fn a_threshold_equal_to_the_measured_value_holds_despite_float_widening() {
        for v in [0.7_f32, 1.1] {
            let as_written: f64 = v.to_string().parse().unwrap();
            let max = Assertion::ImageNotOversized {
                element: None,
                max_effective: Some(as_written),
            };
            let min = Assertion::ImageResolutionSufficient {
                element: None,
                min_effective: Some(as_written),
            };
            assert_outcome(&run(max, vec![density(Some(v))]), AssertionOutcome::Passed);
            assert_outcome(&run(min, vec![density(Some(v))]), AssertionOutcome::Passed);
        }
    }

    #[test]
    fn an_img_with_no_format_cannot_fail_a_density_or_shape_check_but_can_hold() {
        let no_format = |el: Element| {
            let mut el = el;
            el.media.as_mut().unwrap().format = None;
            el
        };
        let min = |v| Assertion::ImageResolutionSufficient {
            element: None,
            min_effective: Some(v),
        };
        let max = |v| Assertion::ImageNotOversized {
            element: None,
            max_effective: Some(v),
        };
        let thin = || no_format(density(Some(0.25)));
        assert_outcome(&run(min(1.0), vec![thin()]), AssertionOutcome::Unknown);
        assert_outcome(&run(min(0.1), vec![thin()]), AssertionOutcome::Passed);
        assert_outcome(&run(max(0.1), vec![thin()]), AssertionOutcome::Unknown);
        assert_outcome(&run(max(1.0), vec![thin()]), AssertionOutcome::Passed);
        let stretched = no_format(sized((100.0, 100.0), (200.0, 100.0), Some(ObjectFit::Fill)));
        assert_outcome(
            &run(distorted_a(), vec![stretched]),
            AssertionOutcome::Unknown,
        );
        let even = no_format(sized((100.0, 100.0), (50.0, 50.0), Some(ObjectFit::Fill)));
        assert_outcome(&run(distorted_a(), vec![even]), AssertionOutcome::Passed);
        // Only an `img` or `picture` can hold a vector.
        let video = image(
            "x",
            ElementMedia {
                kind: MediaKind::Video,
                effective_resolution: Some(0.25),
                ..img_with()
            },
        );
        assert_outcome(&run(min(1.0), vec![video]), AssertionOutcome::Failed);
    }

    #[test]
    fn a_url_suffix_that_names_no_raster_cannot_fail_a_density_check() {
        let with_format = |f: &str| {
            let mut el = density(Some(0.25));
            el.media.as_mut().unwrap().format = Some(f.into());
            el
        };
        let min = || Assertion::ImageResolutionSufficient {
            element: None,
            min_effective: Some(1.0),
        };
        for f in ["php", "v2"] {
            assert_outcome(&run(min(), vec![with_format(f)]), AssertionOutcome::Unknown);
        }
        for f in ["png", "JPEG", "webp", "x-icon", "tif", "jfif"] {
            assert_outcome(&run(min(), vec![with_format(f)]), AssertionOutcome::Failed);
        }
    }

    #[test]
    fn a_vector_in_an_img_has_no_density_to_judge() {
        for format in ["svg+xml", "SVG+XML"] {
            let vector = image(
                "logo",
                ElementMedia {
                    format: Some(format.into()),
                    effective_resolution: Some(0.25),
                    natural_px: Some(MediaSize::new(100.0, 100.0)),
                    rendered_px: Some(MediaSize::new(400.0, 100.0)),
                    object_fit: Some(ObjectFit::Fill),
                    ..img_with()
                },
            );
            for a in [
                Assertion::ImageResolutionSufficient {
                    element: None,
                    min_effective: Some(1.0),
                },
                Assertion::ImageNotOversized {
                    element: None,
                    max_effective: Some(0.1),
                },
                distorted_a(),
            ] {
                let r = run(a, vec![vector.clone()]);
                assert_outcome(&r, AssertionOutcome::Passed);
                assert_detail_has(&r, "1 not applicable");
            }
        }
    }

    // ---- distortion --------------------------------------------------------

    fn sized(natural: (f32, f32), rendered: (f32, f32), fit: Option<ObjectFit>) -> Element {
        image(
            "x",
            ElementMedia {
                natural_px: Some(MediaSize::new(natural.0, natural.1)),
                rendered_px: Some(MediaSize::new(rendered.0, rendered.1)),
                object_fit: fit,
                format: Some("png".into()),
                ..img_with()
            },
        )
    }

    fn distorted_a() -> Assertion {
        Assertion::ImageNotDistorted {
            element: None,
            max_aspect_delta: Some(0.05),
        }
    }

    #[test]
    fn image_not_distorted_fails_a_stretched_fill_either_way_round() {
        for rendered in [(200.0, 100.0), (100.0, 200.0)] {
            let r = run(
                distorted_a(),
                vec![sized((100.0, 100.0), rendered, Some(ObjectFit::Fill))],
            );
            assert_outcome(&r, AssertionOutcome::Failed);
            assert_detail_has(&r, "1.000");
        }
    }

    #[test]
    fn image_not_distorted_holds_within_the_delta_under_fill() {
        let r = run(
            distorted_a(),
            vec![sized((100.0, 100.0), (103.0, 100.0), Some(ObjectFit::Fill))],
        );
        assert_outcome(&r, AssertionOutcome::Passed);
    }

    #[test]
    fn an_aspect_delta_equal_to_the_threshold_holds_despite_float_error() {
        for (w, max) in [(105.0, 0.05), (102.0, 0.02), (110.0, 0.1)] {
            let a = Assertion::ImageNotDistorted {
                element: None,
                max_aspect_delta: Some(max),
            };
            let el = sized((100.0, 100.0), (w, 100.0), Some(ObjectFit::Fill));
            assert_outcome(&run(a, vec![el]), AssertionOutcome::Passed);
        }
    }

    #[test]
    fn image_not_distorted_holds_for_every_uniform_fit_whatever_the_box() {
        for fit in [
            ObjectFit::Contain,
            ObjectFit::Cover,
            ObjectFit::None,
            ObjectFit::ScaleDown,
        ] {
            let r = run(
                distorted_a(),
                vec![sized((100.0, 100.0), (400.0, 100.0), Some(fit))],
            );
            assert_outcome(&r, AssertionOutcome::Passed);
        }
    }

    #[test]
    fn image_not_distorted_is_unknown_when_it_cannot_tell() {
        let no_fit = sized((100.0, 100.0), (400.0, 100.0), None);
        assert_outcome(&run(distorted_a(), vec![no_fit]), AssertionOutcome::Unknown);
        let zero = sized((0.0, 100.0), (100.0, 100.0), Some(ObjectFit::Fill));
        assert_outcome(&run(distorted_a(), vec![zero]), AssertionOutcome::Unknown);
        let no_sizes = image(
            "x",
            ElementMedia {
                object_fit: Some(ObjectFit::Fill),
                ..img_with()
            },
        );
        assert_outcome(
            &run(distorted_a(), vec![no_sizes]),
            AssertionOutcome::Unknown,
        );
    }

    // ---- dims_reserved -----------------------------------------------------

    #[test]
    fn image_dims_reserved_fixtures() {
        let a = Assertion::ImageDimsReserved { element: None };
        let with = |v| {
            image(
                "x",
                ElementMedia {
                    dims_reserved: v,
                    ..img_with()
                },
            )
        };
        assert_outcome(
            &run(a.clone(), vec![with(Some(false))]),
            AssertionOutcome::Failed,
        );
        assert_outcome(
            &run(a.clone(), vec![with(Some(true))]),
            AssertionOutcome::Passed,
        );
        assert_outcome(&run(a.clone(), vec![with(None)]), AssertionOutcome::Unknown);
        let svg = image("s", ElementMedia::new(MediaKind::Svg));
        let r = run(a, vec![svg]);
        assert_outcome(&r, AssertionOutcome::Passed);
        assert_detail_has(&r, "1 not applicable");
    }

    // ---- lcp_image_prioritized ---------------------------------------------

    fn lcp(
        candidate: Option<bool>,
        loading: Option<MediaLoading>,
        priority: Option<MediaFetchPriority>,
    ) -> Element {
        image(
            "x",
            ElementMedia {
                is_lcp_candidate: candidate,
                loading,
                fetch_priority: priority,
                ..img_with()
            },
        )
    }

    #[test]
    fn lcp_image_prioritized_fixtures() {
        use MediaFetchPriority::{Auto, High, Low};
        use MediaLoading::{Eager, Lazy};
        let a = Assertion::LcpImagePrioritized { element: None };
        let cases = [
            (
                Some(true),
                Some(Eager),
                Some(High),
                AssertionOutcome::Passed,
            ),
            (Some(true), Some(Lazy), Some(High), AssertionOutcome::Failed),
            (Some(true), Some(Lazy), None, AssertionOutcome::Failed),
            (
                Some(true),
                Some(Eager),
                Some(Auto),
                AssertionOutcome::Failed,
            ),
            (Some(true), Some(Eager), Some(Low), AssertionOutcome::Failed),
            (Some(true), Some(Eager), None, AssertionOutcome::Unknown),
            (Some(true), None, Some(High), AssertionOutcome::Unknown),
            (None, Some(Eager), Some(High), AssertionOutcome::Unknown),
            (Some(false), Some(Lazy), Some(Low), AssertionOutcome::Passed),
        ];
        for (candidate, loading, priority, want) in cases {
            let r = run(a.clone(), vec![lcp(candidate, loading, priority)]);
            assert_outcome(&r, want);
        }
    }

    // ---- image_has_text_alternative ----------------------------------------

    fn alt(decorative: bool, text: Option<&str>) -> Element {
        image(
            "x",
            ElementMedia {
                alt: Some(MediaAlt {
                    decorative,
                    text: text.map(str::to_owned),
                }),
                ..img_with()
            },
        )
    }

    #[test]
    fn image_has_text_alternative_fails_a_missing_alt() {
        let a = Assertion::ImageHasTextAlternative { element: None };
        assert_outcome(&run(a, vec![alt(false, None)]), AssertionOutcome::Failed);
    }

    #[test]
    fn image_has_text_alternative_holds_for_real_text_and_for_decorative() {
        let a = Assertion::ImageHasTextAlternative { element: None };
        for el in [
            alt(false, Some("A red bicycle against a wall")),
            alt(true, None),
            alt(true, Some("image")),
        ] {
            assert_outcome(&run(a.clone(), vec![el]), AssertionOutcome::Passed);
        }
    }

    #[test]
    fn image_has_text_alternative_fails_every_generic_alt() {
        let a = Assertion::ImageHasTextAlternative { element: None };
        for text in [
            "image",
            "Photo",
            "IMG",
            "picture",
            "icon",
            "logo",
            "banner",
            "untitled",
            "placeholder",
            "thumbnail",
            "screenshot",
            "Screen Shot",
            "screen   shot",
            "  logo  ",
            "12345",
        ] {
            let r = run(a.clone(), vec![alt(false, Some(text))]);
            assert_outcome(&r, AssertionOutcome::Failed);
            assert_detail_has(&r, "generic");
        }
    }

    #[test]
    fn image_has_text_alternative_keeps_text_that_merely_contains_a_generic_word() {
        let a = Assertion::ImageHasTextAlternative { element: None };
        for text in [
            "Company logo",
            "Photo of a bridge",
            "Top 10",
            "screens shot",
        ] {
            let r = run(a.clone(), vec![alt(false, Some(text))]);
            assert_outcome(&r, AssertionOutcome::Passed);
        }
    }

    #[test]
    fn image_has_text_alternative_is_unknown_when_alt_was_not_read() {
        let a = Assertion::ImageHasTextAlternative { element: None };
        assert_outcome(
            &run(a, vec![image("x", img_with())]),
            AssertionOutcome::Unknown,
        );
    }

    #[test]
    fn image_has_text_alternative_does_not_judge_video_or_canvas() {
        let a = Assertion::ImageHasTextAlternative { element: None };
        for kind in [MediaKind::Video, MediaKind::Canvas] {
            let r = run(a.clone(), vec![image("x", ElementMedia::new(kind))]);
            assert_outcome(&r, AssertionOutcome::Passed);
            assert_detail_has(&r, "1 not applicable");
        }
    }

    // ---- image_weight_within -----------------------------------------------

    #[test]
    fn image_weight_within_fixtures() {
        let a = Assertion::ImageWeightWithin {
            element: None,
            max_bytes: Some(1000),
        };
        let weigh = |b| {
            image(
                "x",
                ElementMedia {
                    transfer_bytes: b,
                    ..img_with()
                },
            )
        };
        assert_outcome(
            &run(a.clone(), vec![weigh(Some(1001))]),
            AssertionOutcome::Failed,
        );
        assert_outcome(
            &run(a.clone(), vec![weigh(Some(1000))]),
            AssertionOutcome::Passed,
        );
        assert_outcome(&run(a, vec![weigh(None)]), AssertionOutcome::Unknown);
    }

    #[test]
    fn image_weight_within_still_weighs_a_hidden_image() {
        let a = Assertion::ImageWeightWithin {
            element: None,
            max_bytes: Some(1000),
        };
        let heavy = zero_area(image(
            "x",
            ElementMedia {
                transfer_bytes: Some(5000),
                ..img_with()
            },
        ));
        assert_outcome(&run(a, vec![heavy]), AssertionOutcome::Failed);
    }

    #[test]
    fn image_weight_within_leaves_out_only_an_image_lazy_loading_deferred() {
        let a = Assertion::ImageWeightWithin {
            element: None,
            max_bytes: Some(1000),
        };
        let lazy = |in_viewport| {
            image(
                "x",
                ElementMedia {
                    load_state: Some(MediaLoadState::Lazy),
                    in_viewport,
                    ..img_with()
                },
            )
        };
        assert_outcome(
            &run(a.clone(), vec![lazy(Some(false))]),
            AssertionOutcome::Passed,
        );
        // `lazy` alone is any unfinished lazy image; on screen it is mid-download.
        for in_viewport in [Some(true), None] {
            assert_outcome(
                &run(a.clone(), vec![lazy(in_viewport)]),
                AssertionOutcome::Unknown,
            );
        }
        // A lazy image with no reserved size reads as off-screen wherever it sits.
        assert_outcome(
            &run(a.clone(), vec![zero_area(lazy(Some(false)))]),
            AssertionOutcome::Unknown,
        );
        let mut no_box = lazy(Some(false));
        no_box.bbox = None;
        assert_outcome(&run(a.clone(), vec![no_box]), AssertionOutcome::Unknown);
        // Lazy and loaded, weight unseen (cross-origin): it was requested.
        let unseen = zero_area(image(
            "x",
            ElementMedia {
                loading: Some(MediaLoading::Lazy),
                load_state: Some(MediaLoadState::Loaded),
                ..img_with()
            },
        ));
        assert_outcome(&run(a, vec![unseen]), AssertionOutcome::Unknown);
    }

    // ---- offscreen_image_lazy ----------------------------------------------

    #[test]
    fn offscreen_image_lazy_fixtures() {
        use MediaLoading::{Eager, Lazy};
        let a = Assertion::OffscreenImageLazy { element: None };
        let cases = [
            (Some(false), Some(Eager), AssertionOutcome::Failed),
            (Some(false), Some(Lazy), AssertionOutcome::Passed),
            (Some(false), None, AssertionOutcome::Unknown),
            (None, Some(Lazy), AssertionOutcome::Unknown),
            (Some(true), Some(Eager), AssertionOutcome::Passed),
        ];
        for (in_viewport, loading, want) in cases {
            let el = image(
                "x",
                ElementMedia {
                    in_viewport,
                    loading,
                    ..img_with()
                },
            );
            assert_outcome(&run(a.clone(), vec![el]), want);
        }
    }

    #[test]
    fn offscreen_image_lazy_leaves_the_lcp_image_to_its_own_check() {
        let a = Assertion::OffscreenImageLazy { element: None };
        let hero = image(
            "x",
            ElementMedia {
                is_lcp_candidate: Some(true),
                in_viewport: Some(false),
                loading: Some(MediaLoading::Eager),
                ..img_with()
            },
        );
        let r = run(a, vec![hero]);
        assert_outcome(&r, AssertionOutcome::Passed);
        assert_detail_has(&r, "1 not applicable");
    }

    // ---- no_duplicate_images -----------------------------------------------

    fn sourced(id: &str, kind: MediaKind, digest: Option<&str>) -> Element {
        image(
            id,
            ElementMedia {
                src_digest: digest.map(str::to_owned),
                ..ElementMedia::new(kind)
            },
        )
    }

    fn duplicates_a() -> Assertion {
        Assertion::NoDuplicateImages { element: None }
    }

    #[test]
    fn no_duplicate_images_fails_a_repeated_source_naming_both() {
        let r = run(
            duplicates_a(),
            vec![
                sourced("a", MediaKind::Img, Some("d1")),
                sourced("b", MediaKind::Picture, Some("d1")),
                sourced("c", MediaKind::Img, Some("d2")),
            ],
        );
        assert_outcome(&r, AssertionOutcome::Failed);
        assert_detail_has(&r, "'a' displays the same source as b");
        assert_detail_has(&r, "'b' displays the same source as a");
        assert!(!r.detail.unwrap().contains("'c'"));
    }

    #[test]
    fn no_duplicate_images_holds_for_distinct_sources() {
        let r = run(
            duplicates_a(),
            vec![
                sourced("a", MediaKind::Img, Some("d1")),
                sourced("b", MediaKind::Img, Some("d2")),
            ],
        );
        assert_outcome(&r, AssertionOutcome::Passed);
    }

    #[test]
    fn no_duplicate_images_is_unknown_for_an_unread_digest() {
        let r = run(
            duplicates_a(),
            vec![
                sourced("a", MediaKind::Img, Some("d1")),
                sourced("b", MediaKind::Img, None),
            ],
        );
        assert_outcome(&r, AssertionOutcome::Unknown);
        assert_detail_has(&r, "'b'");
    }

    #[test]
    fn no_duplicate_images_ignores_a_shared_css_background() {
        let r = run(
            duplicates_a(),
            vec![
                sourced("a", MediaKind::CssBackground, Some("sprite")),
                sourced("b", MediaKind::CssBackground, Some("sprite")),
            ],
        );
        assert_outcome(&r, AssertionOutcome::Passed);
    }

    #[test]
    fn no_duplicate_images_ignores_a_hidden_twin() {
        let mut twin = sourced("mobile", MediaKind::Img, Some("d1"));
        twin.bbox = Some(Region {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        });
        let r = run(
            duplicates_a(),
            vec![sourced("desktop", MediaKind::Img, Some("d1")), twin],
        );
        assert_outcome(&r, AssertionOutcome::Passed);
    }

    #[test]
    fn no_duplicate_images_named_compares_against_the_rest_of_the_page() {
        let els = vec![
            sourced("a", MediaKind::Img, Some("d1")),
            sourced("b", MediaKind::Img, Some("d1")),
            sourced("c", MediaKind::Img, Some("d2")),
        ];
        let named = |id: &str| Assertion::NoDuplicateImages {
            element: Some(id.into()),
        };
        assert_outcome(&run(named("a"), els.clone()), AssertionOutcome::Failed);
        assert_outcome(&run(named("c"), els), AssertionOutcome::Passed);
    }

    // ---- the wire ----------------------------------------------------------

    #[test]
    fn every_variant_round_trips_its_wire_spelling() {
        let json = serde_json::json!([
            {"type": "image_loaded"},
            {"type": "image_resolution_sufficient", "element": "h", "min_effective": 1.0},
            {"type": "image_not_distorted", "max_aspect_delta": 0.05},
            {"type": "image_dims_reserved"},
            {"type": "lcp_image_prioritized"},
            {"type": "image_has_text_alternative", "element": "h"},
            {"type": "image_not_oversized", "max_effective": 2.0},
            {"type": "image_weight_within", "max_bytes": 512000},
            {"type": "offscreen_image_lazy"},
            {"type": "no_duplicate_images"}
        ]);
        let parsed: Vec<Assertion> = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(parsed.len(), 10);
        let back = serde_json::to_value(&parsed).unwrap();
        for (a, b) in json
            .as_array()
            .unwrap()
            .iter()
            .zip(back.as_array().unwrap())
        {
            // `element` serializes as null when unset; a threshold must come back
            // by the same name, or `serde(default)` would read a rename as absent.
            let mut b = b.as_object().unwrap().clone();
            b.retain(|_, v| !v.is_null());
            assert_eq!(a.as_object().unwrap(), &b);
        }
    }

    #[test]
    fn a_bare_threshold_variant_parses_as_undeclared() {
        let a: Assertion =
            serde_json::from_value(serde_json::json!({"type": "image_not_oversized"})).unwrap();
        assert!(matches!(
            a,
            Assertion::ImageNotOversized {
                element: None,
                max_effective: None
            }
        ));
    }
}
