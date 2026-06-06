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
    /// Which capture backend produced this frame, when the frame originated
    /// from the runner's own desktop window. `None` for device / synthetic
    /// frames (and for window frames captured by paths that don't track a
    /// backend). Surfaced as response-echo + health telemetry by consumers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// The concrete mechanism used to capture a runner-window [`Frame`].
///
/// On Windows the runner prefers the occlusion-immune WebView2
/// `CapturePreview` backend and falls back to a monitor-region crop when
/// CapturePreview is unavailable or fails. The variant is recorded on
/// [`FrameSource::capture_backend`] so it can be echoed in vision responses
/// and counted in `vision/health` telemetry. Serializes to its variant name
/// (e.g. `"Webview2CapturePreview"`, `"MonitorCrop"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureBackend {
    /// WebView2 `CapturePreview` — captures the webview contents directly, so
    /// the frame is unaffected by windows occluding the runner on screen.
    Webview2CapturePreview,
    /// Monitor-region crop fallback — grabs the screen region the runner
    /// window occupies. Subject to occlusion by overlapping windows.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Region {
    /// True iff this region lies entirely within a `width × height` frame.
    pub fn fits_in(&self, width: u32, height: u32) -> bool {
        self.x
            .checked_add(self.w)
            .map(|right| right <= width)
            .unwrap_or(false)
            && self
                .y
                .checked_add(self.h)
                .map(|bottom| bottom <= height)
                .unwrap_or(false)
    }
}
