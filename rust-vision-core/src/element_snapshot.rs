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
    /// Pixel-space bounding box.
    pub bbox: Region,
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
    let inner_right = inner.x.saturating_add(inner.w);
    let inner_bottom = inner.y.saturating_add(inner.h);
    let outer_right = outer.x.saturating_add(outer.w);
    let outer_bottom = outer.y.saturating_add(outer.h);
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner_right <= outer_right
        && inner_bottom <= outer_bottom
}

/// Whether two regions intersect at all (any pixel overlap).
pub fn regions_overlap(a: Region, b: Region) -> bool {
    let a_right = a.x.saturating_add(a.w);
    let a_bottom = a.y.saturating_add(a.h);
    let b_right = b.x.saturating_add(b.w);
    let b_bottom = b.y.saturating_add(b.h);
    a.x < b_right && b.x < a_right && a.y < b_bottom && b.y < a_bottom
}

/// Compute the intersection of two regions, if any.
pub fn intersection(a: Region, b: Region) -> Option<Region> {
    let a_right = a.x.saturating_add(a.w);
    let a_bottom = a.y.saturating_add(a.h);
    let b_right = b.x.saturating_add(b.w);
    let b_bottom = b.y.saturating_add(b.h);
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = a_right.min(b_right);
    let bottom = a_bottom.min(b_bottom);
    if x < right && y < bottom {
        Some(Region {
            x,
            y,
            w: right - x,
            h: bottom - y,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: u32, y: u32, w: u32, h: u32) -> Region {
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

    #[test]
    fn luminance_black_zero() {
        assert!((Rgb::new(0, 0, 0).relative_luminance() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn luminance_white_one() {
        assert!((Rgb::new(255, 255, 255).relative_luminance() - 1.0).abs() < 1e-6);
    }
}
