//! [`Frame`] — an RGBA image buffer plus the metadata that says where it came from.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An RGBA image buffer plus the metadata describing its origin.
#[derive(Debug, Clone)]
pub struct Frame {
    pub buffer: image::RgbaImage,
    pub width: u32,
    pub height: u32,
    pub source: FrameSource,
}

impl Frame {
    /// Construct a [`Frame`] from an [`image::RgbaImage`] and source metadata.
    /// Width/height are derived from the buffer; callers don't supply them.
    pub fn from_rgba(buffer: image::RgbaImage, source: FrameSource) -> Self {
        let (width, height) = buffer.dimensions();
        Self {
            buffer,
            width,
            height,
            source,
        }
    }

    /// Map a CSS-pixel region — the coordinate space of every
    /// [`crate::ElementSnapshot`] bbox — onto this frame's buffer pixels,
    /// through [`FrameSource::scale_factor`].
    ///
    /// A snapshot's boxes come from `getBoundingClientRect()` and are CSS
    /// pixels; a frame captured at device pixel ratio `s` holds `s` buffer
    /// pixels per CSS pixel on each axis. Sampling a box without this mapping
    /// reads the wrong pixels on every HiDPI capture — at DPR 2 a button at CSS
    /// `(50,40)` is at buffer `(100,80)`, and buffer `(50,40)` is whatever sits
    /// up and to the left of it.
    ///
    /// The mapping rounds OUTWARD (floor the near edge, ceil the far edge), so
    /// a fractional ratio covers every buffer pixel the CSS box touches. An
    /// EMPTY axis stays empty: a box with `w == 0` (or `h == 0`) covers no
    /// pixel at any ratio, yet rounding outward around a fractional edge would
    /// give it one — at 1.5, CSS `x = 3` is buffer `4.5`, and floor 4 .. ceil 5
    /// is a column the element never painted. That axis maps to a zero extent
    /// at the scaled origin instead, exactly as it does at DPR 1. The result is
    /// still a [`Region`] that may lie partly or wholly outside the frame;
    /// pixel readers narrow it with [`Region::clamp_to_frame`] as before.
    ///
    /// `None` when the scale factor is not a positive finite number — no pixel
    /// can be located through it, and pretending `1.0` would sample the wrong
    /// ones silently — or when the mapped region does not fit [`Region`]'s
    /// integer range.
    pub fn css_region_to_pixels(&self, css: Region) -> Option<Region> {
        let s = self.source.scale_factor;
        if !s.is_finite() || s <= 0.0 {
            return None;
        }
        let x0 = (css.x as f64 * s).floor();
        let y0 = (css.y as f64 * s).floor();
        // An empty CSS axis maps to an empty buffer axis (see above).
        let x1 = if css.w == 0 {
            x0
        } else {
            (css.right() as f64 * s).ceil()
        };
        let y1 = if css.h == 0 {
            y0
        } else {
            (css.bottom() as f64 * s).ceil()
        };
        let fits_i32 = |v: f64| v >= i32::MIN as f64 && v <= i32::MAX as f64;
        let fits_u32 = |v: f64| v >= 0.0 && v <= u32::MAX as f64;
        if !(fits_i32(x0) && fits_i32(y0) && fits_u32(x1 - x0) && fits_u32(y1 - y0)) {
            return None;
        }
        Some(Region {
            x: x0 as i32,
            y: y0 as i32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }
}

/// Source metadata for a [`Frame`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSource {
    pub kind: FrameSourceKind,
    /// Device pixel ratio: buffer pixels per CSS pixel on each axis. 1.0 for
    /// unscaled captures, 2.0 for Retina, 2.625 for many phones.
    ///
    /// Load-bearing, not decorative: every analyzer that reads pixels under a
    /// snapshot bbox maps the CSS-pixel box onto the buffer through this value
    /// ([`Frame::css_region_to_pixels`]). A wrong value samples the wrong
    /// pixels; a non-positive or non-finite one samples none.
    pub scale_factor: f64,
    pub captured_at: DateTime<Utc>,
    /// Which capture backend produced this frame, when the source is the
    /// runner's own desktop window. `None` for `Device`/`Synthetic` kinds,
    /// where no runner-window backend applies.
    pub capture_backend: Option<CaptureBackend>,
}

impl FrameSource {
    /// Construct a synthetic [`FrameSource`] stamped at the current UTC time.
    /// Useful for tests and procedurally-generated frames.
    pub fn synthetic_now() -> Self {
        Self {
            kind: FrameSourceKind::Synthetic,
            scale_factor: 1.0,
            captured_at: Utc::now(),
            capture_backend: None,
        }
    }
}

/// The capture backend that produced a runner-window [`Frame`]. Distinguishes
/// the WebView2 `CapturePreview` path from the monitor-crop fallback — both of
/// which produce a [`FrameSourceKind::Window`] frame, so this is genuinely new
/// provenance, not a `kind` refinement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureBackend {
    Webview2CapturePreview,
    MonitorCrop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameSourceKind {
    Window,
    Region,
    Synthetic,
    /// A frame sourced from an external device or app (e.g. a phone serving
    /// `control/screenshot`, or an adb framebuffer capture) rather than the
    /// runner's own desktop window.
    Device,
}

/// A pixel-space rectangle.
///
/// **Two pixel spaces use this type, and they differ by the device pixel
/// ratio.** An [`crate::ElementSnapshot`] bbox is in CSS pixels; a region
/// passed to a pixel reader such as [`crate::analyzers::color::sample_dominant_two`]
/// or to a pipeline stage is in BUFFER pixels of a specific [`Frame`]. Convert
/// the first into the second with [`Frame::css_region_to_pixels`]; the two
/// coincide only at `scale_factor == 1.0`.
///
/// The ORIGIN (`x`, `y`) is **signed**: a UI element can legitimately sit
/// left of / above the viewport origin — `getBoundingClientRect()` returns
/// negative coordinates for anything scrolled or positioned off-screen, and
/// a multi-monitor virtual desktop places secondary displays at negative
/// coordinates. Storing the origin unsigned forced every producer to clamp
/// at 0, which silently reported every off-screen element as sitting flush
/// against the viewport edge and destroyed the information an analyzer needs
/// to reason about clipping, overflow, and off-screen placement.
///
/// The EXTENT (`w`, `h`) stays **unsigned**: a negative width or height is
/// meaningless, so `u32` makes it unrepresentable rather than merely invalid
/// — no producer can construct one and no consumer has to defend against it.
///
/// A region whose origin is negative therefore describes real geometry that
/// lies (partly or wholly) outside the frame. Pixel-sampling consumers must
/// intersect with the frame before indexing — see [`Region::clamp_to_frame`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Region {
    /// Exclusive right edge. Widened to `i64` so `x + w` can never overflow
    /// for any representable `Region`.
    pub fn right(&self) -> i64 {
        self.x as i64 + self.w as i64
    }

    /// Exclusive bottom edge. Widened to `i64` — see [`Region::right`].
    pub fn bottom(&self) -> i64 {
        self.y as i64 + self.h as i64
    }

    /// True iff this region lies entirely within a `width × height` frame.
    /// A negative origin is out of bounds by definition.
    pub fn fits_in(&self, width: u32, height: u32) -> bool {
        self.x >= 0 && self.y >= 0 && self.right() <= width as i64 && self.bottom() <= height as i64
    }

    /// The portion of this region that lies inside a `width × height` frame,
    /// as unsigned pixel indices ready for buffer access. `None` when the
    /// region is entirely outside the frame (including the wholly-negative
    /// case) or has zero area after clamping.
    ///
    /// This is the ONE place a signed region is narrowed to buffer indices;
    /// pixel-sampling code should call it rather than casting by hand.
    pub fn clamp_to_frame(&self, width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
        let left = self.x.max(0) as i64;
        let top = self.y.max(0) as i64;
        let right = self.right().min(width as i64);
        let bottom = self.bottom().min(height as i64);
        if right <= left || bottom <= top {
            return None;
        }
        Some((
            left as u32,
            top as u32,
            (right - left) as u32,
            (bottom - top) as u32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame_at(scale_factor: f64) -> Frame {
        Frame::from_rgba(
            image::RgbaImage::new(4, 4),
            FrameSource {
                kind: FrameSourceKind::Synthetic,
                scale_factor,
                captured_at: Utc::now(),
                capture_backend: None,
            },
        )
    }

    fn r(x: i32, y: i32, w: u32, h: u32) -> Region {
        Region { x, y, w, h }
    }

    #[test]
    fn dpr_one_is_the_identity() {
        assert_eq!(
            frame_at(1.0).css_region_to_pixels(r(50, 40, 100, 60)),
            Some(r(50, 40, 100, 60))
        );
    }

    #[test]
    fn dpr_two_doubles_origin_and_extent() {
        assert_eq!(
            frame_at(2.0).css_region_to_pixels(r(50, 40, 100, 60)),
            Some(r(100, 80, 200, 120))
        );
    }

    #[test]
    fn a_fractional_dpr_rounds_outward_so_every_touched_pixel_is_covered() {
        // 2.625 x [3, 8) = [7.875, 21.0) -> floor 7 .. ceil 21 = 14 wide.
        // 2.625 x [1, 2) = [2.625, 5.25)  -> floor 2 .. ceil 6  = 4 high.
        assert_eq!(
            frame_at(2.625).css_region_to_pixels(r(3, 1, 5, 1)),
            Some(r(7, 2, 14, 4))
        );
    }

    #[test]
    fn an_empty_axis_stays_empty_at_a_fractional_dpr() {
        // The review's case: CSS x 3, w 0 at 1.5 is buffer 4.5, and rounding
        // outward around it would invent the one-pixel column 4..5.
        assert_eq!(
            frame_at(1.5).css_region_to_pixels(r(3, 1, 0, 20)),
            Some(r(4, 1, 0, 31))
        );
        assert_eq!(
            frame_at(1.5).css_region_to_pixels(r(3, 1, 20, 0)),
            Some(r(4, 1, 31, 0))
        );
        // 2.625: x 7.875 -> 7, y 2.625 -> 2, far edge 21 x 2.625 = 55.125 -> 56.
        assert_eq!(
            frame_at(2.625).css_region_to_pixels(r(3, 1, 0, 20)),
            Some(r(7, 2, 0, 54))
        );
        assert_eq!(
            frame_at(2.625).css_region_to_pixels(r(3, 1, 20, 0)),
            Some(r(7, 2, 54, 0))
        );
        assert_eq!(
            frame_at(2.625).css_region_to_pixels(r(3, 1, 0, 0)),
            Some(r(7, 2, 0, 0))
        );
    }

    #[test]
    fn a_negative_origin_stays_negative_and_scaled() {
        // Off-screen geometry is real geometry; clamping is the reader's job.
        assert_eq!(
            frame_at(2.0).css_region_to_pixels(r(-20, -10, 40, 40)),
            Some(r(-40, -20, 80, 80))
        );
    }

    #[test]
    fn an_unusable_scale_factor_maps_nothing() {
        for bad in [0.0, -2.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                frame_at(bad).css_region_to_pixels(r(1, 1, 1, 1)),
                None,
                "scale_factor {bad} must not locate a pixel"
            );
        }
    }

    #[test]
    fn a_region_that_overflows_the_integer_range_maps_to_none() {
        assert_eq!(
            frame_at(8.0).css_region_to_pixels(r(i32::MAX - 1, 0, 1, 1)),
            None
        );
    }
}
