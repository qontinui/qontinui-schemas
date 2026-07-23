//! Source-agnostic structural snapshot of a UI page.
//!
//! [`ElementSnapshot`] is the input the analyzers + assertion DSL consume.
//! It's deliberately decoupled from any specific snapshot source (the
//! runner's `discover`, native a11y trees, mobile UIBridge, etc) — every
//! one of those callers projects into this shape before invoking
//! `vision/analyze` or `vision/assert`.
//!
//! Pixel-space units throughout. Geometry uses [`Region`] (from `frame.rs`);
//! visual properties (color, font size, etc) are optional because not every
//! snapshot source can populate them — analyzers/assertions that depend on
//! a missing field surface a soft "skipped: missing X" finding instead of
//! erroring.

use serde::{Deserialize, Serialize};

use crate::frame::Region;

/// Snapshot of one rendered page worth of elements, ready to feed
/// [`crate::analyzers`] or [`crate::assertions`]. Caller is responsible
/// for capturing this in lockstep with the [`crate::Frame`] that will be
/// used in the same `analyze`/`assert` call — analyzers do not re-fetch
/// from the runner, they trust the snapshot supplied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementSnapshot {
    pub elements: Vec<Element>,
}

impl ElementSnapshot {
    /// Find an element by id. O(n) — snapshots are small (typically <500
    /// elements) and assertions look up only a handful per call.
    pub fn get(&self, id: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    /// Stable identifier. Matches the SDK's element registry id where one
    /// exists; otherwise an opaque token.
    pub id: String,
    /// Pixel-space bounding box. `None` when the snapshot source could not
    /// measure geometry (e.g. a mobile `discover` snapshot that includes
    /// hidden/unmeasured elements). Geometry-based analyzers/assertions
    /// SKIP bbox-less elements (they surface no spurious findings); the
    /// element is still counted for non-geometry checks (interactivity,
    /// text presence, typography). Web snapshots always populate this, so
    /// `Option` is a strict superset — existing payloads round-trip
    /// byte-identically (`skip_serializing_if` omits `None`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Region>,
    /// Visible text content (innerText / accessibilityLabel equivalent).
    /// `None` when the element has no text or the snapshot source didn't
    /// populate it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// ARIA role / native-platform equivalent. e.g., `"button"`, `"link"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// True if the element accepts pointer/key input (button, link, input,
    /// etc). Drives the "interactive coverage" + WCAG-target-size analyses.
    #[serde(default)]
    pub interactable: bool,
    /// Foreground (text / icon) color as RGB. None when the snapshot
    /// source can't determine computed style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg_color: Option<Rgb>,
    /// Effective background color the element renders against — usually
    /// the resolved-up-the-tree opaque ancestor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<Rgb>,
    /// Computed font size in CSS pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size_px: Option<f32>,
    /// Font family declaration as the SDK saw it. May be a comma-list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// Computed line-height in CSS pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height_px: Option<f32>,
    /// Parent id, when known. Lets `no_clipping` walk up to the clip
    /// ancestor without re-running the layout engine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    /// Children ids, when known.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_ids: Vec<String>,
}

/// Linear-light RGB color (no alpha). Used for both `fg_color` and
/// `bg_color`; assertions/analyzers convert to the appropriate space
/// (Lab/sRGB) on demand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// W3C relative luminance, sRGB. Used for WCAG contrast.
    pub fn relative_luminance(self) -> f64 {
        fn channel(c: u8) -> f64 {
            let s = c as f64 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(self.r) + 0.7152 * channel(self.g) + 0.0722 * channel(self.b)
    }
}

/// Whether a value lies in a bounded region. Distinct from
/// [`Region::fits_in`] which checks against frame dimensions — this is
/// "does element B fully nest inside element A".
pub fn region_contains(outer: Region, inner: Region) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

/// Whether two regions intersect at all (any pixel overlap).
pub fn regions_overlap(a: Region, b: Region) -> bool {
    (a.x as i64) < b.right()
        && (b.x as i64) < a.right()
        && (a.y as i64) < b.bottom()
        && (b.y as i64) < a.bottom()
}

/// Compute the intersection of two regions, if any.
pub fn intersection(a: Region, b: Region) -> Option<Region> {
    let x = a.x.max(b.x) as i64;
    let y = a.y.max(b.y) as i64;
    let right = a.right().min(b.right());
    let bottom = a.bottom().min(b.bottom());
    if x < right && y < bottom {
        Some(Region {
            // `x`/`y` are the max of two `i32`s, so they fit back in `i32`;
            // `right - x` / `bottom - y` are positive and bounded by the
            // smaller of the two extents, so they fit in `u32`.
            x: x as i32,
            y: y as i32,
            w: (right - x) as u32,
            h: (bottom - y) as u32,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: i32, y: i32, w: u32, h: u32) -> Region {
        Region { x, y, w, h }
    }

    #[test]
    fn overlap_detected() {
        assert!(regions_overlap(region(0, 0, 10, 10), region(5, 5, 10, 10)));
    }

    #[test]
    fn touching_edges_not_overlap() {
        assert!(!regions_overlap(
            region(0, 0, 10, 10),
            region(10, 0, 10, 10)
        ));
    }

    #[test]
    fn nested_overlap() {
        assert!(regions_overlap(
            region(0, 0, 100, 100),
            region(40, 40, 10, 10)
        ));
    }

    #[test]
    fn region_contains_full_nest() {
        assert!(region_contains(
            region(0, 0, 100, 100),
            region(10, 10, 20, 20)
        ));
    }

    #[test]
    fn region_does_not_contain_partial() {
        assert!(!region_contains(
            region(0, 0, 50, 50),
            region(40, 40, 20, 20)
        ));
    }

    #[test]
    fn intersection_computed() {
        let i = intersection(region(0, 0, 10, 10), region(5, 5, 10, 10)).unwrap();
        assert_eq!(i, region(5, 5, 5, 5));
    }

    #[test]
    fn intersection_disjoint() {
        assert!(intersection(region(0, 0, 5, 5), region(10, 10, 5, 5)).is_none());
    }

    // ---------------------------------------------------------------
    // Signed origin. `getBoundingClientRect()` returns negative x/y for any
    // element scrolled or positioned off the top/left of the viewport. Before
    // the origin was widened to i32, producers had to clamp those to 0, which
    // reported every off-screen element as flush against the viewport edge.
    // These tests pin the TRUE coordinate surviving the wire round-trip and
    // the geometry helpers behaving correctly across the origin.
    // ---------------------------------------------------------------

    #[test]
    fn negative_origin_survives_json_round_trip() {
        // Exactly what a `left: -9999px` a11y-hidden node measures as.
        let json = r#"{"id":"offscreen","bbox":{"x":-9999,"y":-48,"w":120,"h":32}}"#;
        let el: Element = serde_json::from_str(json).expect("negative origin must deserialize");
        let bbox = el.bbox.expect("bbox present");
        assert_eq!(
            bbox.x, -9999,
            "true negative x must survive, not clamp to 0"
        );
        assert_eq!(bbox.y, -48, "true negative y must survive, not clamp to 0");
        assert_eq!(bbox.w, 120);
        assert_eq!(bbox.h, 32);

        // ...and re-serializes verbatim, so a snapshot can be persisted and
        // re-read without drifting toward the origin.
        let re = serde_json::to_string(&bbox).unwrap();
        let back: Region = serde_json::from_str(&re).unwrap();
        assert_eq!(back, bbox);
    }

    #[test]
    fn negative_extent_is_unrepresentable() {
        // `w`/`h` stay unsigned on purpose: a negative extent is meaningless,
        // so the type rejects it rather than a consumer having to defend.
        let json = r#"{"x":0,"y":0,"w":-5,"h":10}"#;
        assert!(serde_json::from_str::<Region>(json).is_err());
    }

    #[test]
    fn geometry_helpers_work_across_the_origin() {
        // A sticky header scrolled half off the top: y = -20, h = 40.
        let partly_offscreen = region(-20, -20, 40, 40);
        let viewport_corner = region(0, 0, 100, 100);
        assert!(regions_overlap(partly_offscreen, viewport_corner));
        let i = intersection(partly_offscreen, viewport_corner).unwrap();
        assert_eq!(i, region(0, 0, 20, 20));

        // Wholly off-screen: no overlap with the viewport at all. Under the
        // old clamped encoding this collapsed to (0,0) and produced a
        // spurious overlap against every top-left element.
        let wholly_offscreen = region(-9999, -9999, 120, 32);
        assert!(!regions_overlap(wholly_offscreen, viewport_corner));
        assert!(intersection(wholly_offscreen, viewport_corner).is_none());

        // Containment across the origin: an off-canvas drawer contains its
        // own child even though both sit at negative coordinates.
        assert!(region_contains(
            region(-200, -100, 300, 200),
            region(-150, -50, 50, 50)
        ));
        assert!(!region_contains(
            region(-200, -100, 300, 200),
            region(-250, -50, 50, 50)
        ));
    }

    #[test]
    fn fits_in_rejects_negative_origin() {
        assert!(region(0, 0, 10, 10).fits_in(100, 100));
        assert!(!region(-1, 0, 10, 10).fits_in(100, 100));
        assert!(!region(0, -1, 10, 10).fits_in(100, 100));
    }

    #[test]
    fn clamp_to_frame_narrows_signed_origin_to_buffer_indices() {
        // Straddling the origin -> the in-frame part only.
        assert_eq!(
            region(-20, -10, 40, 40).clamp_to_frame(100, 100),
            Some((0, 0, 20, 30))
        );
        // Wholly outside -> nothing to sample.
        assert_eq!(region(-9999, -9999, 120, 32).clamp_to_frame(100, 100), None);
        assert_eq!(region(200, 0, 10, 10).clamp_to_frame(100, 100), None);
        // Fully inside -> unchanged.
        assert_eq!(
            region(10, 10, 5, 5).clamp_to_frame(100, 100),
            Some((10, 10, 5, 5))
        );
    }

    #[test]
    fn luminance_black_zero() {
        assert!((Rgb::new(0, 0, 0).relative_luminance() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn luminance_white_one() {
        assert!((Rgb::new(255, 255, 255).relative_luminance() - 1.0).abs() < 1e-6);
    }
}
