//! Pipeline composer + executor.

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::{DynamicImage, GenericImageView, ImageEncoder, ImageFormat, RgbaImage};

use crate::contract::{AlphaPolicy, EncodedFormat, MetadataPolicy, OutputContract};
use crate::error::VisionError;
use crate::frame::{Frame, FrameSource, Region};
use crate::stage::{Annotation, RedactKind, RedactRegion, ResizeStrategy, Stage};
use crate::strip;

/// A composable image pipeline. Build with [`Pipeline::new`] + [`Pipeline::push`],
/// then execute with [`Pipeline::run`].
#[derive(Debug, Default, Clone)]
pub struct Pipeline {
    stages: Vec<Stage>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Append a stage. Builder-style: returns `self`.
    pub fn push(mut self, stage: Stage) -> Self {
        self.stages.push(stage);
        self
    }

    /// Inspect the stage list. Useful for tests that want to assert what
    /// [`safe_image`](crate::encode::safe_image) composed.
    pub fn stages(&self) -> &[Stage] {
        &self.stages
    }

    /// Execute the pipeline.
    ///
    /// Frame-to-Frame stages mutate the working frame in order.
    /// [`Stage::Encode`] produces the byte buffer; [`Stage::Verify`] (if present
    /// and terminal) round-trips and validates against the contract.
    pub fn run(self, frame: Frame) -> Result<Vec<u8>, VisionError> {
        // Validate pipeline shape up front.
        let verify_positions: Vec<usize> = self
            .stages
            .iter()
            .enumerate()
            .filter_map(|(i, s)| matches!(s, Stage::Verify(_)).then_some(i))
            .collect();
        if let Some(&first_verify) = verify_positions.first() {
            if verify_positions.len() > 1 {
                return Err(VisionError::InvalidPipeline(
                    "more than one Verify stage".into(),
                ));
            }
            if first_verify != self.stages.len() - 1 {
                return Err(VisionError::InvalidPipeline(
                    "Verify must be the terminal stage".into(),
                ));
            }
        }
        let encode_count = self
            .stages
            .iter()
            .filter(|s| matches!(s, Stage::Encode(_)))
            .count();
        if encode_count > 1 {
            return Err(VisionError::InvalidPipeline(
                "more than one Encode stage".into(),
            ));
        }
        if !verify_positions.is_empty() && encode_count == 0 {
            return Err(VisionError::InvalidPipeline(
                "Verify present without an Encode stage".into(),
            ));
        }

        // Execute.
        let mut working = WorkingFrame::Frame(frame);
        let mut strip_requested = false;
        let mut encoded_format: Option<EncodedFormat> = None;
        let mut bytes: Option<Vec<u8>> = None;

        for stage in self.stages {
            match stage {
                Stage::CropRegion(region) => {
                    working = working.map_frame(|f| crop_region(f, region))?;
                }
                Stage::Resize(strategy) => {
                    working = working.map_frame(|f| resize_frame(f, strategy))?;
                }
                Stage::FlattenAlpha(bg) => {
                    working = working.map_frame(|f| flatten_alpha(f, bg))?;
                }
                Stage::Annotate(annotations) => {
                    working = working.map_frame(|f| annotate(f, annotations))?;
                }
                Stage::Redact(regions) => {
                    working = working.map_frame(|f| redact(f, regions))?;
                }
                Stage::StripMetadata => {
                    strip_requested = true;
                }
                Stage::Encode(format) => {
                    let frame = working.take_frame()?;
                    let raw_bytes = encode_frame(frame, format)?;
                    let final_bytes = if strip_requested {
                        strip_bytes(&raw_bytes, format)?
                    } else {
                        raw_bytes
                    };
                    encoded_format = Some(format);
                    bytes = Some(final_bytes);
                    working = WorkingFrame::Bytes;
                }
                Stage::Verify(contract) => {
                    let bs = bytes.as_ref().ok_or_else(|| {
                        VisionError::InvalidPipeline(
                            "Verify reached without bytes (no Encode)".into(),
                        )
                    })?;
                    let format = encoded_format.ok_or_else(|| {
                        VisionError::InvalidPipeline(
                            "Verify reached without recorded format".into(),
                        )
                    })?;
                    verify(bs, format, &contract, strip_requested)?;
                }
            }
        }

        if let Some(b) = bytes {
            Ok(b)
        } else {
            Err(VisionError::InvalidPipeline(
                "pipeline produced no bytes (missing Encode)".into(),
            ))
        }
    }
}

/// Internal pipeline state: either a working frame, or "already encoded, no
/// further frame stages allowed".
enum WorkingFrame {
    Frame(Frame),
    Bytes,
}

impl WorkingFrame {
    fn map_frame<F>(self, f: F) -> Result<Self, VisionError>
    where
        F: FnOnce(Frame) -> Result<Frame, VisionError>,
    {
        match self {
            WorkingFrame::Frame(frame) => Ok(WorkingFrame::Frame(f(frame)?)),
            WorkingFrame::Bytes => Err(VisionError::InvalidPipeline(
                "frame-stage applied after Encode".into(),
            )),
        }
    }

    fn take_frame(self) -> Result<Frame, VisionError> {
        match self {
            WorkingFrame::Frame(f) => Ok(f),
            WorkingFrame::Bytes => Err(VisionError::InvalidPipeline(
                "Encode reached but frame already consumed".into(),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// Stage implementations
// ---------------------------------------------------------------------------

fn crop_region(frame: Frame, region: Region) -> Result<Frame, VisionError> {
    if region.w == 0 || region.h == 0 {
        return Err(VisionError::CropOutOfBounds {
            region,
            frame: (frame.width, frame.height),
        });
    }
    if !region.fits_in(frame.width, frame.height) {
        return Err(VisionError::CropOutOfBounds {
            region,
            frame: (frame.width, frame.height),
        });
    }
    let dyn_img = DynamicImage::ImageRgba8(frame.buffer);
    let cropped = dyn_img.crop_imm(region.x, region.y, region.w, region.h);
    let rgba = cropped.to_rgba8();
    Ok(Frame::from_rgba(rgba, frame.source))
}

fn resize_frame(frame: Frame, strategy: ResizeStrategy) -> Result<Frame, VisionError> {
    let (new_w, new_h) = match strategy {
        ResizeStrategy::LongEdge(max) => {
            let long = frame.width.max(frame.height);
            if long <= max || long == 0 {
                return Ok(frame);
            }
            let ratio = max as f64 / long as f64;
            let w = ((frame.width as f64 * ratio).round() as u32).max(1);
            let h = ((frame.height as f64 * ratio).round() as u32).max(1);
            (w, h)
        }
        ResizeStrategy::Scale(s) => {
            if !s.is_finite() || s <= 0.0 {
                return Err(VisionError::InvalidPipeline(format!(
                    "Resize::Scale({s}) is not a positive finite number"
                )));
            }
            let w = ((frame.width as f64 * s).round() as u32).max(1);
            let h = ((frame.height as f64 * s).round() as u32).max(1);
            (w, h)
        }
        ResizeStrategy::Fit { w, h } => {
            if w == 0 || h == 0 {
                return Err(VisionError::InvalidPipeline(
                    "Resize::Fit dimensions must be non-zero".into(),
                ));
            }
            let rw = w as f64 / frame.width as f64;
            let rh = h as f64 / frame.height as f64;
            let ratio = rw.min(rh);
            let new_w = ((frame.width as f64 * ratio).round() as u32).max(1);
            let new_h = ((frame.height as f64 * ratio).round() as u32).max(1);
            (new_w, new_h)
        }
    };
    if new_w == frame.width && new_h == frame.height {
        return Ok(frame);
    }
    let dyn_img = DynamicImage::ImageRgba8(frame.buffer);
    let resized = dyn_img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);
    let rgba = resized.to_rgba8();
    Ok(Frame::from_rgba(rgba, frame.source))
}

fn flatten_alpha(frame: Frame, bg: [u8; 3]) -> Result<Frame, VisionError> {
    let mut out = RgbaImage::new(frame.width, frame.height);
    let br = bg[0] as u16;
    let bgc = bg[1] as u16;
    let bb = bg[2] as u16;
    for (x, y, px) in frame.buffer.enumerate_pixels() {
        let [r, g, b, a] = px.0;
        let af = a as u16;
        let inv = 255u16 - af;
        // Standard over-composite, "alpha A over RGB background" approximation.
        // (out = src * a/255 + bg * (1 - a/255))
        let nr = ((r as u16 * af + br * inv) / 255) as u8;
        let ng = ((g as u16 * af + bgc * inv) / 255) as u8;
        let nb = ((b as u16 * af + bb * inv) / 255) as u8;
        out.put_pixel(x, y, image::Rgba([nr, ng, nb, 0xFF]));
    }
    Ok(Frame::from_rgba(out, frame.source))
}

fn annotate(mut frame: Frame, annotations: Vec<Annotation>) -> Result<Frame, VisionError> {
    for ann in annotations {
        draw_rect_outline(
            &mut frame.buffer,
            ann.region,
            ann.style.border_color,
            ann.style.border_thickness,
        );
        if let Some(fill) = ann.style.fill_color {
            fill_rect(&mut frame.buffer, ann.region, fill);
            // Re-draw the outline so a fill doesn't clobber the border.
            draw_rect_outline(
                &mut frame.buffer,
                ann.region,
                ann.style.border_color,
                ann.style.border_thickness,
            );
        }
        // TODO(phase-6): render `ann.label` once typography integration lands.
        let _ = ann.label;
    }
    Ok(frame)
}

fn draw_rect_outline(buf: &mut RgbaImage, region: Region, color: [u8; 4], thickness: u32) {
    let (w, h) = buf.dimensions();
    if region.w == 0 || region.h == 0 || thickness == 0 {
        return;
    }
    let x0 = region.x.min(w);
    let y0 = region.y.min(h);
    let x1 = region.x.saturating_add(region.w).min(w);
    let y1 = region.y.saturating_add(region.h).min(h);
    let px = image::Rgba(color);
    let t = thickness;
    // Top + bottom edges
    for y in y0..(y0.saturating_add(t)).min(y1) {
        for x in x0..x1 {
            buf.put_pixel(x, y, px);
        }
    }
    for y in y1.saturating_sub(t).max(y0)..y1 {
        for x in x0..x1 {
            buf.put_pixel(x, y, px);
        }
    }
    // Left + right edges
    for x in x0..(x0.saturating_add(t)).min(x1) {
        for y in y0..y1 {
            buf.put_pixel(x, y, px);
        }
    }
    for x in x1.saturating_sub(t).max(x0)..x1 {
        for y in y0..y1 {
            buf.put_pixel(x, y, px);
        }
    }
}

fn fill_rect(buf: &mut RgbaImage, region: Region, color: [u8; 4]) {
    let (w, h) = buf.dimensions();
    let x0 = region.x.min(w);
    let y0 = region.y.min(h);
    let x1 = region.x.saturating_add(region.w).min(w);
    let y1 = region.y.saturating_add(region.h).min(h);
    let px = image::Rgba(color);
    for y in y0..y1 {
        for x in x0..x1 {
            buf.put_pixel(x, y, px);
        }
    }
}

fn redact(mut frame: Frame, regions: Vec<RedactRegion>) -> Result<Frame, VisionError> {
    for r in regions {
        if !r.region.fits_in(frame.width, frame.height) {
            // Clamp to bounds rather than error: redact is best-effort.
            continue;
        }
        match r.kind {
            RedactKind::Fill(color) => {
                fill_rect(&mut frame.buffer, r.region, color);
            }
            RedactKind::Blur { sigma } => {
                blur_region(&mut frame.buffer, r.region, sigma);
            }
            RedactKind::Pixelate { block_size } => {
                pixelate_region(&mut frame.buffer, r.region, block_size);
            }
        }
    }
    Ok(frame)
}

fn blur_region(buf: &mut RgbaImage, region: Region, sigma: f32) {
    if region.w == 0 || region.h == 0 {
        return;
    }
    let dyn_img =
        DynamicImage::ImageRgba8(buf.view(region.x, region.y, region.w, region.h).to_image());
    let blurred = dyn_img.blur(sigma.max(0.5));
    let blurred_rgba = blurred.to_rgba8();
    for (sx, sy, px) in blurred_rgba.enumerate_pixels() {
        buf.put_pixel(region.x + sx, region.y + sy, *px);
    }
}

fn pixelate_region(buf: &mut RgbaImage, region: Region, block_size: u32) {
    if region.w == 0 || region.h == 0 || block_size == 0 {
        return;
    }
    let sub = buf.view(region.x, region.y, region.w, region.h).to_image();
    let small_w = (region.w / block_size).max(1);
    let small_h = (region.h / block_size).max(1);
    let dyn_img = DynamicImage::ImageRgba8(sub);
    let small = dyn_img.resize_exact(small_w, small_h, image::imageops::FilterType::Triangle);
    let large = small.resize_exact(region.w, region.h, image::imageops::FilterType::Nearest);
    let large_rgba = large.to_rgba8();
    for (sx, sy, px) in large_rgba.enumerate_pixels() {
        buf.put_pixel(region.x + sx, region.y + sy, *px);
    }
}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

fn encode_frame(frame: Frame, format: EncodedFormat) -> Result<Vec<u8>, VisionError> {
    let mut out = Vec::new();
    match format {
        EncodedFormat::Jpeg { quality } => {
            // JPEG doesn't support alpha; pre-flatten to RGB.
            let rgb = DynamicImage::ImageRgba8(frame.buffer).to_rgb8();
            let mut encoder = JpegEncoder::new_with_quality(&mut out, quality);
            encoder
                .encode(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| VisionError::EncodeFailed(format!("jpeg: {e}")))?;
        }
        EncodedFormat::Webp { quality, lossy } => {
            // The `image` 0.25 WebP encoder is lossless-only — `quality` and
            // `lossy` flags from the contract are recorded in `EncodedFormat`
            // but the produced bytes are always lossless WebP. Documented in
            // the crate docs; Verify accepts any WebP family member.
            let _ = (quality, lossy);
            // If every alpha byte is 0xFF, encode from RGB8 so the resulting
            // WebP carries no alpha channel — required for `AlphaPolicy::Flatten`
            // contracts whose decoded image must not have alpha.
            let fully_opaque = frame.buffer.pixels().all(|p| p.0[3] == 0xFF);
            let encoder = WebPEncoder::new_lossless(&mut out);
            if fully_opaque {
                let rgb = DynamicImage::ImageRgba8(frame.buffer).to_rgb8();
                encoder
                    .write_image(
                        rgb.as_raw(),
                        rgb.width(),
                        rgb.height(),
                        image::ExtendedColorType::Rgb8,
                    )
                    .map_err(|e| VisionError::EncodeFailed(format!("webp: {e}")))?;
            } else {
                encoder
                    .write_image(
                        frame.buffer.as_raw(),
                        frame.width,
                        frame.height,
                        image::ExtendedColorType::Rgba8,
                    )
                    .map_err(|e| VisionError::EncodeFailed(format!("webp: {e}")))?;
            }
        }
        EncodedFormat::Png => {
            let encoder = PngEncoder::new(&mut out);
            encoder
                .write_image(
                    frame.buffer.as_raw(),
                    frame.width,
                    frame.height,
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| VisionError::EncodeFailed(format!("png: {e}")))?;
        }
    }
    Ok(out)
}

fn strip_bytes(bytes: &[u8], format: EncodedFormat) -> Result<Vec<u8>, VisionError> {
    match format {
        EncodedFormat::Jpeg { .. } => strip::strip_jpeg(bytes),
        EncodedFormat::Webp { .. } => strip::strip_webp(bytes),
        EncodedFormat::Png => strip::strip_png(bytes),
    }
}

// ---------------------------------------------------------------------------
// Verify
// ---------------------------------------------------------------------------

fn verify(
    bytes: &[u8],
    encoded_format: EncodedFormat,
    contract: &OutputContract,
    strip_requested: bool,
) -> Result<(), VisionError> {
    // 1. Byte length cap.
    if bytes.len() > contract.max_bytes {
        return Err(VisionError::ContractViolation {
            stage: "verify",
            reason: format!(
                "output is {} bytes; contract {} max is {}",
                bytes.len(),
                contract.name,
                contract.max_bytes
            ),
        });
    }

    // 2. Format is in allowed_formats.
    let allowed = contract
        .allowed_formats
        .iter()
        .any(|f| f.matches_family(encoded_format));
    if !allowed {
        return Err(VisionError::ContractViolation {
            stage: "verify",
            reason: format!(
                "format {:?} not in contract {} allowed_formats {:?}",
                encoded_format, contract.name, contract.allowed_formats
            ),
        });
    }

    // 3. Decode and check dimensions + alpha policy.
    let image_format = match encoded_format {
        EncodedFormat::Jpeg { .. } => ImageFormat::Jpeg,
        EncodedFormat::Webp { .. } => ImageFormat::WebP,
        EncodedFormat::Png => ImageFormat::Png,
    };
    let decoded = image::load_from_memory_with_format(bytes, image_format).map_err(|e| {
        VisionError::DecodeFailed(format!("verify decode ({:?}): {}", image_format, e))
    })?;
    let (w, h) = decoded.dimensions();
    let long = w.max(h);
    if long > contract.max_long_edge {
        return Err(VisionError::ContractViolation {
            stage: "verify",
            reason: format!(
                "decoded long-edge {} > contract {} max_long_edge {}",
                long, contract.name, contract.max_long_edge
            ),
        });
    }

    // 4. Alpha policy: if Flatten, the *decoded* image must not carry alpha
    //    (JPEG never does; PNG-with-RGBA fails; WebP-with-alpha fails).
    if let AlphaPolicy::Flatten(_) = contract.alpha_policy {
        if decoded.color().has_alpha() {
            return Err(VisionError::ContractViolation {
                stage: "verify",
                reason: format!(
                    "decoded image has alpha channel but contract {} demands Flatten",
                    contract.name
                ),
            });
        }
    }

    // 5. Metadata policy: if StripAll requested *and* the pipeline ran a
    //    StripMetadata stage, re-parse and assert no disallowed bits remain.
    if matches!(contract.metadata_policy, MetadataPolicy::StripAll) {
        if !strip_requested {
            return Err(VisionError::ContractViolation {
                stage: "verify",
                reason: format!(
                    "contract {} requires StripAll but pipeline omitted StripMetadata",
                    contract.name
                ),
            });
        }
        let check = match encoded_format {
            EncodedFormat::Png => strip::assert_stripped_png(bytes),
            EncodedFormat::Jpeg { .. } => strip::assert_stripped_jpeg(bytes),
            EncodedFormat::Webp { .. } => strip::assert_stripped_webp(bytes),
        };
        if let Err(reason) = check {
            return Err(VisionError::ContractViolation {
                stage: "verify",
                reason: format!("metadata strip incomplete: {reason}"),
            });
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Test-only conveniences. Kept here (rather than in tests/) so unit tests can
// reach pipeline internals without making them public.
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub fn _test_frame_from_color(w: u32, h: u32, color: [u8; 4]) -> Frame {
    let mut buf = RgbaImage::new(w, h);
    for px in buf.pixels_mut() {
        *px = image::Rgba(color);
    }
    Frame::from_rgba(buf, FrameSource::synthetic_now())
}
