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
}

/// Source metadata for a [`Frame`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSource {
    pub kind: FrameSourceKind,
    /// Device pixel ratio. 1.0 for unscaled captures, 2.0 for Retina, etc.
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
